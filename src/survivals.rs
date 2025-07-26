use crate::helpers::{get_new_config_with_selected_table, get_value};
use crate::mt_config::{AssumptionEnum, MortTableConfig};
use polars::prelude::*;

// =======================================
// PUBLIC FUNCTIONS
// =======================================
/// Calculate ₜpₓ: probability of surviving t years from age x (fractional ages supported).
/// ₖ|ₜp = ₖ₊ₜpₓ =  ∏ₖ₌₀^{t+k-1} (1 - qₓ₊ₖ₊ₜ)
/// Uses UDD, CFM, or HPB formulas for fractional ages/times; delegates to whole ages if both are integers.
pub fn tpx(
    config: &MortTableConfig,
    x: f64,
    t: f64,
    k: f64,
    entry_age: Option<u32>,
) -> PolarsResult<f64> {
    // Decide if selected table is used
    let new_config = get_new_config_with_selected_table(config, entry_age)?;

    // Combine t and k
    let t = t + k;

    // Handle special case for whole numbers right at the start
    if x.fract() == 0.0 && t.fract() == 0.0 {
        return _tpx_whole(&new_config, t as u32, x as u32);
    }

    // If not start to handle fractional ages
    let x_whole = x.floor() as u32; // n
    let x_frac = x.fract(); // s
    let time_to_next_age = 1.0 - x_frac; // always between 0 and 1

    // Get mortality rate for age n (percentage already applied in qx function)
    let qx = get_value(config, x_whole, "qx").unwrap_or(0.0);

    if t <= time_to_next_age {
        // Case 2a: when t ≤ (1-s) or t <= time_to_next_age
        // ------UDD------:
        // ₜqₓ₊ₛ = t · qₓ / (1 - s · qₓ)
        // ₜpₓ₊ₛ = 1 - t · qₓ / (1 - s · qₓ)
        // ------CFM------:
        // ₜpₓ₊ₛ = (1 - qₓ)ᵗ
        // ------HPB-------:
        // ₜqₓ₊ₛ = t · qₓ / (1 + s · qₓ)
        // ₜpₓ₊ₛ = 1 - t · qₓ / (1 + s · qₓ)
        let survival_rate = match config.assumption {
            Some(AssumptionEnum::UDD) => 1.0 - t * qx / (1.0 - x_frac * qx),
            Some(AssumptionEnum::CFM) => (1.0 - qx).powf(t),
            Some(AssumptionEnum::HPB) => 1.0 - t * qx / (1.0 + x_frac * qx),
            _ => {
                return Err(PolarsError::ComputeError(
                    "Unsupported assumption for fractional age".into(),
                ));
            }
        };
        Ok(survival_rate)
    } else {
        // Case 2b:  when t > (1-s) or t > time_to_next_age
        let survival_to_next_age = tpx(&new_config, time_to_next_age, x, 0.0, None)?;
        let remaining_time = t - time_to_next_age;
        let survival_after = tpx(&new_config, remaining_time, (x_whole + 1) as f64, 0.0, None)?;
        let result = survival_to_next_age * survival_after;
        Ok(result)
    }
}

/// Calculate ₜqₓ - probability of dying within t years starting at age x (fractional ages supported).
/// ₖ|ₜpₓ +  ₖ|ₜqₓ =  ₖpₓ
/// This is the complement of [`tpx`]: ₜqₓ = 1 - ₜpₓ.
pub fn tqx(
    config: &MortTableConfig,
    x: f64,
    t: f64,
    k: f64,
    entry_age: Option<u32>,
) -> PolarsResult<f64> {
    let result = tpx(config, x, k, 0.0, entry_age) - tpx(config, x, t, k, entry_age)?;
    Ok(result)
}

// =======================================
// PRIVATE FUNCTIONS
// =======================================
/// Calculate ₜpₓ: probability of surviving t years from age x (whole ages only).
///
/// Formula: ₜpₓ = ∏(k=0 to t-1) (1 - qₓ₊ₖ)
fn _tpx_whole(config: &MortTableConfig, x: u32, t: u32) -> PolarsResult<f64> {
    let mut result = 1.0;
    for age in x..(x + t) {
        let qx = get_value(&config, age, "qx")?;
        let px = 1.0 - qx;
        result *= px;
    }

    Ok(result)
}
