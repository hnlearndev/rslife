use self::helpers::is_table_layout_approved;
use super::*;

/// Calculate ₜpₓ: probability of surviving t years from age x (fractional ages supported).
///
/// Uses UDD, CFM, or HPB formulas for fractional ages/times; delegates to whole ages if both are integers.
pub fn tpx(config: &MortTableConfig, t: f64, x: f64, entry_age: Option<u32>) -> PolarsResult<f64> {
    if !is_table_layout_approved(config) {
        return Err(PolarsError::ComputeError(
            "Mortality table XML layout is not suitable for calculations".into(),
        ));
    }

    // Handle special case for whole numbers right at the start
    if x.fract() == 0.0 && t.fract() == 0.0 {
        return whole::tpx(config, t as u32, x as u32, entry_age);
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
        let survival_to_next_age = tpx(config, time_to_next_age, x, entry_age)?;
        let remaining_time = t - time_to_next_age;
        let survival_after = tpx(config, remaining_time, (x_whole + 1) as f64, entry_age)?;
        let result = survival_to_next_age * survival_after;
        Ok(result)
    }
}

/// Calculate ₜqₓ - probability of dying within t years starting at age x (fractional ages supported).
///
/// This is the complement of [`tpx`]: ₜqₓ = 1 - ₜpₓ.
pub fn tqx(config: &MortTableConfig, t: f64, x: f64, entry_age: Option<u32>) -> PolarsResult<f64> {
    let result = 1.0 - tpx(config, t, x, entry_age)?;
    Ok(result)
}

//-----------------------------------------------------------
// UNIT TESTS
//-----------------------------------------------------------
#[cfg(test)]
mod tests {
    // use super::*;
    // use crate::xml::MortXML;
}
