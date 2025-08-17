use crate::RSLifeResult;
use crate::helpers::get_new_config_with_selected_table;
use crate::mt_config::{AssumptionEnum, MortTableConfig};
use crate::params::SurvivalFunctionParams;
use bon::builder;

// =======================================
// PUBLIC FUNCTIONS
// =======================================
/// Survival probability: ₜpₓ (probability of surviving t years from age x, fractional ages supported)
///
/// Computes the probability that a life aged `x` survives for `t` years, supporting both integer and fractional ages/times.
/// This is a fundamental building block for all life insurance and annuity calculations.
///
/// # Formula
///
/// **For x and t are integer**:
///
/// ```text
/// ₜpₓ = ∏(k=0 to t-1) (1 - qₓ₊ₖ)
/// ₖ|ₜp = ₖ₊ₜpₓ =  ∏(k=0 to t+k-1) (1 - qₓ₊ₖ₊ₜ)
/// ```
///
/// **For fractional x and t, the function supports:**
///
/// - UDD (Uniform Distribution of Deaths):
/// ```text
/// ₜqₓ₊ₛ = t · qₓ / (1 - s · qₓ)
/// ₜpₓ₊ₛ = 1 - t · qₓ / (1 - s · qₓ)
/// ```
///
/// - CFM (Constant Force of Mortality):
/// ```text
/// ₜpₓ₊ₛ = pₓᵗ
/// ₜpₓ₊ₛ = (1 - qₓ)ᵗ
/// ```
///
/// - HPB (Hyperbolic):
/// ```text
/// ₜqₓ₊ₛ = t · qₓ / (1 + s · qₓ)
/// ₜpₓ₊ₛ = 1 - t · qₓ / (1 + s · qₓ)
/// ```
///
/// # Examples
///
/// ## Basic Survival Probability
/// ```rust
/// # use rslife::prelude::*;
/// # let mort_data = MortData::from_soa_url_id(1704)?;
/// # let config = MortTableConfig::builder()
/// #     .data(mort_data)
/// #     .radix(100_000)
/// #     .pct(1.0)
/// #     .assumption(AssumptionEnum::UDD)
/// #     .build()
/// #     .unwrap();
/// // Probability of surviving 10 years from age 40
/// let prob = tpx().mt(&config).x(40.0).t(10.0).call()?;
/// println!("10-year survival probability: {:.6}", prob);
/// # RSLifeResult::Ok(())
/// ```
///
/// ## Fractional Age Survival
/// ```rust
/// # use rslife::prelude::*;
/// # let mort_data = MortData::from_soa_url_id(1704)?;
/// # let config = MortTableConfig::builder()
/// #     .data(mort_data)
/// #     .radix(100_000)
/// #     .pct(1.0)
/// #     .assumption(AssumptionEnum::CFM)
/// #     .build()
/// #     .unwrap();
/// let prob = tpx().mt(&config).x(60.0).t(2.5).call()?;
/// println!("2.5-year survival from age 60: {:.6}", prob);
/// # RSLifeResult::Ok(())
/// ```
#[builder]
pub fn tpx(
    mt: &MortTableConfig,
    x: f64,
    #[builder(default = 1.0)] t: f64,
    #[builder(default = 0.0)] k: f64,
    entry_age: Option<u32>,
    #[builder(default = true)] validate: bool,
) -> RSLifeResult<f64> {
    // ✅ ₖ|ₜp = ₖ₊ₜpₓ =  ∏ₖ₌₀^{t+k-1} (1 - qₓ₊ₖ₊ₜ)
    // Validate parameters
    if validate {
        let params = SurvivalFunctionParams {
            mt: mt.clone(),
            x,
            t,
            k,
            entry_age,
        };

        params
            .validate_all()
            .map_err(|err| Box::new(err) as Box<dyn std::error::Error>)?;
    }

    // Decide if selected table is used
    let mt = get_new_config_with_selected_table(mt, entry_age)?;

    // Combine t and k
    let t = t + k;

    // Handle special case for whole numbers right at the start
    if x.fract() == 0.0 && t.fract() == 0.0 {
        return tpx_whole(&mt, x as u32, t as u32);
    }

    // If not start to handle fractional ages
    let x_whole = x.floor() as u32; // n
    let x_frac = x.fract(); // s
    let time_to_next_age = 1.0 - x_frac; // always between 0 and 1

    if t <= time_to_next_age {
        tpx_frac_t(&mt, x, t)
    } else {
        // Calculate survival to next integer age using builder pattern, split into multiple lines
        let survival_to_next_age = tpx_frac_t(&mt, x, time_to_next_age)?;

        // Break remain time into whole and fractional parts
        // - Part 1: Survival for whole part from age x_whole + 1 to x_whole + 1 + remaining_time_whole
        // - Part 2: Survival for fractional part from age x_whole + 1 + remaining_time_whole
        let remaining_time = t - time_to_next_age;
        let remaining_time_whole = remaining_time.floor() as u32;
        let remaining_time_frac = remaining_time.fract();
        let part1 = tpx_whole(&mt, x_whole + 1, remaining_time_whole)?;

        // If remaining time is whole, we can just return part1
        let survival_for_remaining_time = if remaining_time_frac == 0.0 {
            part1
        } else {
            let part2 = tpx_frac_t(
                &mt,
                x_whole as f64 + 1.0 + remaining_time_whole as f64,
                remaining_time_frac,
            )?;
            part1 * part2
        };

        Ok(survival_to_next_age * survival_for_remaining_time)
    }
}

/// Cumulative mortality probability: ₜqₓ (probability of dying within t years from age x, fractional ages supported)
///
/// Computes the probability that a life aged `x` dies within `t` years, supporting both integer and fractional ages/times.
/// This is the complement to the survival probability, and is used in all life insurance and risk calculations.
///
/// # Formula
/// ```text
/// ₜqₓ = 1 - ₜpₓ
/// ₖ|ₜqₓ = ₖpₓ - ₖ|ₜpₓ
/// ```
///
/// Refer to `tpx` for more details as this function is based on it.
///
/// # Examples
///
/// ## Basic Mortality Probability
/// ```rust
/// # use rslife::prelude::*;
/// # let mort_data = MortData::from_soa_url_id(1704)?;
/// # let config = MortTableConfig::builder()
/// #     .data(mort_data)
/// #     .radix(100_000)
/// #     .pct(1.0)
/// #     .assumption(AssumptionEnum::UDD)
/// #     .build()
/// #     .unwrap();
/// // Probability of dying within 5 years from age 50
/// let prob = tqx().mt(&config).x(50.0).t(5.0).call()?;
/// println!("5-year mortality probability: {:.6}", prob);
/// # RSLifeResult::Ok(())
/// ```
///
/// ## Deferred Mortality Probability (e.g., probability of dying between years 3 and 8 from age 55)
/// ```rust
/// # use rslife::prelude::*;
/// # let mort_data = MortData::from_soa_url_id(1704)?;
/// # let config = MortTableConfig::builder().data(mort_data).build()?;
/// let prob = tqx().mt(&config).x(55.0).t(5.0).k(3.0).call()?;
/// println!("Probability of dying between years 3 and 8: {:.6}", prob);
/// # RSLifeResult::Ok(())
/// ```
#[builder]
pub fn tqx(
    mt: &MortTableConfig,
    x: f64,
    #[builder(default = 1.0)] t: f64,
    #[builder(default = 0.0)] k: f64,
    entry_age: Option<u32>,
    #[builder(default = true)] validate: bool,
) -> RSLifeResult<f64> {
    let kpx_built = tpx().mt(mt).x(x).t(k).k(0.0).validate(validate);
    let kpx = match entry_age {
        Some(age) => kpx_built.entry_age(age).call()?,
        None => kpx_built.call()?,
    };

    let ktpx_built = tpx().mt(mt).x(x).t(t).k(k).validate(validate);
    let ktpx = match entry_age {
        Some(age) => ktpx_built.entry_age(age).call()?,
        None => ktpx_built.call()?,
    };

    // ✅ ₖ|ₜpₓ +  ₖ|ₜqₓ=   ₖpₓ =>  ₖ|ₜqₓ = ₖpₓ - ₖ|ₜpₓ
    Ok(kpx - ktpx)
}

// =======================================
// PRIVATE FUNCTIONS
// =======================================
/// Calculate ₜpₓ: probability of surviving t years from age x (whole ages only).
///
/// Formula: ₜpₓ = lₓ₊ₜ / lₓ
fn tpx_whole(mt: &MortTableConfig, x: u32, t: u32) -> RSLifeResult<f64> {
    let l_x_t = mt.lx().x(x + t).call()?;
    let l_x = mt.lx().x(x).call()?;
    Ok(l_x_t / l_x)
}

/// Calculate ₜpₓ: probability of surviving t years from age x (t is fractional < 1).
///
/// Formula: ₜpₓ = ∏(k=0 to t-1) (1 - qₓ₊ₖ)
fn tpx_frac_t(mt: &MortTableConfig, x: f64, t: f64) -> RSLifeResult<f64> {
    let x_whole = x.floor() as u32;
    let x_frac = x.fract();

    let qx = mt.qx().x(x_whole).call()?;

    let survival_rate = match mt.assumption {
        // ------UDD------:
        // ₜqₓ₊ₛ = t · qₓ / (1 - s · qₓ)
        // ₜpₓ₊ₛ = 1 - t · qₓ / (1 - s · qₓ)
        AssumptionEnum::UDD => 1.0 - t * qx / (1.0 - x_frac * qx),

        // ------CFM------:
        // ₜpₓ₊ₛ = (1 - qₓ)ᵗ
        AssumptionEnum::CFM => (1.0 - qx).powf(t),

        // ------HPB-------:
        // ₜqₓ₊ₛ = t · qₓ / (1 + s · qₓ)
        // ₜpₓ₊ₛ = 1 - t · qₓ / (1 + s · qₓ)
        _ => 1.0 - t * qx / (1.0 + x_frac * qx),
    };

    Ok(survival_rate)
}

// ================================================
// UNIT TESTS
// ================================================
#[cfg(test)]
mod tests {
    use super::*;
    use crate::mt_config::mt_data::MortData;
    use crate::mt_config::{AssumptionEnum, MortTableConfig};
    use approx::assert_abs_diff_eq;

    #[test]
    fn test_survival_cm1_01() {
        // This is obtain from CM1 study package 2019 Chapter 15 The Life Table
        let am92 = MortData::from_soa_url_id(1704).expect("Failed to load EL15 No.15 Female table");
        let mt = MortTableConfig::builder().data(am92).build().unwrap();
        let ans = tpx().mt(&mt).x(58.0).t(0.5).k(0.0).call().unwrap();
        let expected = 0.99670;
        assert_abs_diff_eq!(ans, expected, epsilon = 1e-6);
    }

    #[test]
    fn test_survival_cm1_02() {
        // This is obtain from CM1 study package 2019 Chapter 15 The Life Table
        // CFM assumption with PFA92C20
        let pfa92c20 =
            MortData::from_ifoa_custom("PFA92C20").expect("Failed to load PFA92C20 table");
        let mt = MortTableConfig::builder()
            .data(pfa92c20)
            .assumption(AssumptionEnum::CFM)
            .build()
            .unwrap();

        // Calculate  ₃p₆₂.₅
        let ans = tpx().mt(&mt).x(62.5).t(3.0).k(0.0).call().unwrap();
        let expected = 0.988861;
        assert_abs_diff_eq!(ans, expected, epsilon = 1e-5);
    }

    #[test]
    fn test_survival_cm1_03() {
        // This is obtain from CM1 study package 2019 Chapter 15 The Life Table
        // UDD assumtion with PFA92C20
        let pfa92c20 =
            MortData::from_ifoa_custom("PFA92C20").expect("Failed to load PFA92C20 table");
        let mt = MortTableConfig::builder().data(pfa92c20).build().unwrap();

        // Calculate  ₃p₆₂.₅
        let ans = tpx().mt(&mt).x(62.5).t(3.0).k(0.0).call().unwrap();
        let expected = 0.988863;
        assert_abs_diff_eq!(ans, expected, epsilon = 1e-6);
    }

    #[test]
    fn test_survival_cm1_04() {
        // This is obtain from CM1 study package 2019 Chapter 15 The Life Table
        let am92 = MortData::from_ifoa_url_id("AM92").expect("Failed to load AM92 selected table");
        let mt = MortTableConfig::builder().data(am92).build().unwrap();
        // Calculate  ₃p₆₂.₅
        let ans = tpx()
            .mt(&mt)
            .x(42.0)
            .t(2.0)
            .k(0.0)
            .entry_age(42)
            .call()
            .unwrap();
        let expected = 0.997929;
        assert_abs_diff_eq!(ans, expected, epsilon = 1e-6);
    }

    #[test]
    fn test_survival_cm1_05() {
        // This is obtain from CM1 study package 2019 Chapter 15 The Life Table
        let am92 = MortData::from_ifoa_url_id("AM92").expect("Failed to load AM92 selected table");
        let mt = MortTableConfig::builder().data(am92).build().unwrap();
        // Calculate ₃q(₄₀)₊₁
        let ans = tqx()
            .mt(&mt)
            .x(41.0)
            .t(3.0)
            .k(0.0)
            .entry_age(40)
            .call()
            .unwrap();
        let expected = 0.003270;
        assert_abs_diff_eq!(ans, expected, epsilon = 1e-6);
    }

    #[test]
    fn test_survival_cm1_06() {
        // This is obtain from CM1 study package 2019 Chapter 15 The Life Table
        let am92 = MortData::from_ifoa_url_id("AM92").expect("Failed to load AM92 selected table");
        let mt = MortTableConfig::builder().data(am92).build().unwrap();
        // Calculate ₂|q(₄₁)₊₁
        let ans = tqx().mt(&mt).x(42.0).k(2.0).entry_age(41).call().unwrap();
        let expected = 0.001324;
        assert_abs_diff_eq!(ans, expected, epsilon = 1e-6);
    }
}
