use self::helpers::is_table_layout_approved;
use super::*;

/// Calculate ₜpₓ: probability of surviving t years from age x (fractional ages supported).
///
/// Uses UDD, CFM, or HPB formulas for fractional ages/times; delegates to whole ages if both are integers.
pub fn tpx(config: &MortTableConfig, t: f64, x: f64) -> PolarsResult<f64> {
    if !is_table_layout_approved(config) {
        return Err(PolarsError::ComputeError(
            "Mortality table XML layout is not suitable for calculations".into(),
        ));
    }

    // Handle special case for whole numbers right at the start
    if x.fract() == 0.0 && t.fract() == 0.0 {
        return whole::survivals::tpx(config, t as i32, x as i32);
    }

    // If not start to handle fractional ages
    let x_whole = x.floor() as i32; // n
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
        let survival_to_next_age = tpx(config, time_to_next_age, x)?;
        let remaining_time = t - time_to_next_age;
        let survival_after = tpx(config, remaining_time, (x_whole + 1) as f64)?;
        let result = survival_to_next_age * survival_after;
        Ok(result)
    }
}

/// Calculate ₜqₓ - probability of dying within t years starting at age x (fractional ages supported).
///
/// This is the complement of [`tpx`]: ₜqₓ = 1 - ₜpₓ.
pub fn tqx(config: &MortTableConfig, t: f64, x: f64) -> PolarsResult<f64> {
    let result = 1.0 - tpx(config, t, x)?;
    Ok(result)
}

//-----------------------------------------------------------
// UNIT TESTS
//-----------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;
    use crate::xml::MortXML;

    #[test]
    fn test_udd_whole_numbers() {
        let xml = MortXML::from_url_id(912).expect("Failed to load XML");
        let config = MortTableConfig {
            xml,
            radix: Some(100_000),
            pct: Some(1.0),
            int_rate: None,
            assumption: Some(AssumptionEnum::UDD),
        };

        // Test whole number case: ₅p₃₀
        let survival_prob = tpx(&config, 5.0, 30.0).unwrap();
        println!("UDD: ₅p₃₀ = {survival_prob:.6}");
        assert!(survival_prob > 0.0 && survival_prob <= 1.0);
    }

    #[test]
    fn test_udd_fractional_time() {
        let xml = MortXML::from_url_id(912).expect("Failed to load XML");
        let config = MortTableConfig {
            xml,
            radix: Some(100_000),
            pct: Some(1.0),
            int_rate: None,
            assumption: Some(AssumptionEnum::UDD),
        };

        // Test fractional time: ₀.₅p₃₀
        let survival_prob = tpx(&config, 0.5, 30.0).unwrap();
        println!("UDD: ₀.₅p₃₀ = {survival_prob:.6}");
        assert!(survival_prob > 0.0 && survival_prob <= 1.0);
    }

    #[test]
    fn test_udd_fractional_age() {
        let xml = MortXML::from_url_id(912).expect("Failed to load XML");
        let config = MortTableConfig {
            xml,
            radix: Some(100_000),
            pct: Some(1.0),
            int_rate: None,
            assumption: Some(AssumptionEnum::UDD),
        };

        // Test fractional age: ₁p₃₀.₂₅
        let survival_prob = tpx(&config, 1.5, 30.25).unwrap();
        println!("UDD: ₁.₅p₃₀.₂₅ = {survival_prob:.6}");
        assert!(survival_prob > 0.0 && survival_prob <= 1.0);
    }

    #[test]
    fn test_udd_both_fractional() {
        let xml = MortXML::from_url_id(912).expect("Failed to load XML");
        let config = MortTableConfig {
            xml,
            radix: Some(100_000),
            pct: Some(1.0),
            int_rate: None,
            assumption: Some(AssumptionEnum::UDD),
        };

        // Test both fractional: ₁.₅p₃₀.₂₅
        let survival_prob = tpx(&config, 1.5, 30.25).unwrap();
        println!("UDD: ₁.₅p₃₀.₂₅ = {survival_prob:.6}");
        assert!(survival_prob > 0.0 && survival_prob <= 1.0);
    }

    #[test]
    fn test_udd_mortality_probability() {
        let xml = MortXML::from_url_id(912).expect("Failed to load XML");
        let config = MortTableConfig {
            xml,
            radix: Some(100_000),
            pct: Some(1.0),
            int_rate: None,
            assumption: Some(AssumptionEnum::UDD),
        };

        // Test mortality probability: ₀.₅q₃₀
        let mortality_prob = tqx(&config, 0.5, 30.0).unwrap();
        let survival_prob = tpx(&config, 0.5, 30.0).unwrap();

        println!("UDD: ₀.₅q₃₀ = {mortality_prob:.6}");
        println!("UDD: ₀.₅p₃₀ = {survival_prob:.6}");

        // They should sum to 1
        assert!((mortality_prob + survival_prob - 1.0).abs() < 1e-10);
    }

    #[test]
    #[ignore]
    fn test_percentage_adjustment() {
        let xml = MortXML::from_url_id(912).expect("Failed to load XML");

        // Test with 50% of base rates
        let config_50 = MortTableConfig {
            xml: xml.clone(),
            radix: Some(100_000),
            pct: Some(0.5),
            int_rate: None,
            assumption: Some(AssumptionEnum::UDD),
        };

        // Test with 100% of base rates
        let config_100 = MortTableConfig {
            xml,
            radix: Some(100_000),
            pct: Some(1.0),
            int_rate: None,
            assumption: Some(AssumptionEnum::UDD),
        };

        let survival_50 = tpx(&config_50, 1.0, 30.0).unwrap();
        let survival_100 = tpx(&config_100, 1.0, 30.0).unwrap();

        // 50% rates should give higher survival probability than 100%
        assert!(
            survival_50 > survival_100,
            "Expected survival_50 ({survival_50}) > survival_100 ({survival_100})"
        );

        println!("UDD: ₁p₃₀ with 50% rates = {survival_50:.6}");
        println!("UDD: ₁p₃₀ with 100% rates = {survival_100:.6}");
    }

    #[test]
    fn test_cfm_assumption() {
        let xml = MortXML::from_url_id(912).expect("Failed to load XML");
        let config = MortTableConfig {
            xml,
            radix: Some(100_000),
            pct: Some(1.0),
            int_rate: None,
            assumption: Some(AssumptionEnum::CFM),
        };

        // Test CFM fractional age: ₁p₃₀.₂₅
        let survival_prob = tpx(&config, 1.0, 30.25).unwrap();
        println!("CFM: ₁p₃₀.₂₅ = {survival_prob:.6}");
        assert!(survival_prob > 0.0 && survival_prob <= 1.0);

        // Test CFM fractional time: ₀.₅p₃₀
        let survival_prob_frac = tpx(&config, 0.5, 30.0).unwrap();
        println!("CFM: ₀.₅p₃₀ = {survival_prob_frac:.6}");
        assert!(survival_prob_frac > 0.0 && survival_prob_frac <= 1.0);
    }

    #[test]
    fn test_hpb_assumption() {
        let xml = MortXML::from_url_id(912).expect("Failed to load XML");
        let config = MortTableConfig {
            xml,
            radix: Some(100_000),
            pct: Some(1.0),
            int_rate: None,
            assumption: Some(AssumptionEnum::HPB),
        };

        // Test HPB fractional age: ₁p₃₀.₂₅
        let survival_prob = tpx(&config, 1.0, 30.25).unwrap();
        println!("HPB: ₁p₃₀.₂₅ = {survival_prob:.6}");
        assert!(survival_prob > 0.0 && survival_prob <= 1.0);

        // Test HPB fractional time: ₀.₅p₃₀
        let survival_prob_frac = tpx(&config, 0.5, 30.0).unwrap();
        println!("HPB: ₀.₅p₃₀ = {survival_prob_frac:.6}");
        assert!(survival_prob_frac > 0.0 && survival_prob_frac <= 1.0);
    }

    #[test]
    fn test_assumption_comparison() {
        let xml = MortXML::from_url_id(912).expect("Failed to load XML");

        let config_udd = MortTableConfig {
            xml: xml.clone(),
            radix: Some(100_000),
            pct: Some(1.0),
            int_rate: None,
            assumption: Some(AssumptionEnum::UDD),
        };

        let config_cfm = MortTableConfig {
            xml: xml.clone(),
            radix: Some(100_000),
            pct: Some(1.0),
            int_rate: None,
            assumption: Some(AssumptionEnum::CFM),
        };

        let config_hpb = MortTableConfig {
            xml,
            radix: Some(100_000),
            pct: Some(1.0),
            int_rate: None,
            assumption: Some(AssumptionEnum::HPB),
        };

        // Compare survival probabilities for fractional time across assumptions
        let udd_05 = tpx(&config_udd, 0.5, 30.0).unwrap();
        let cfm_05 = tpx(&config_cfm, 0.5, 30.0).unwrap();
        let hpb_05 = tpx(&config_hpb, 0.5, 30.0).unwrap();

        println!("UDD: ₀.₅p₃₀ = {udd_05:.6}");
        println!("CFM: ₀.₅p₃₀ = {cfm_05:.6}");
        println!("HPB: ₀.₅p₃₀ = {hpb_05:.6}");

        // All should be valid probabilities
        assert!(udd_05 > 0.0 && udd_05 <= 1.0);
        assert!(cfm_05 > 0.0 && cfm_05 <= 1.0);
        assert!(hpb_05 > 0.0 && hpb_05 <= 1.0);

        // Compare fractional age scenarios
        let udd_frac_age = tpx(&config_udd, 0.75, 30.25).unwrap();
        let cfm_frac_age = tpx(&config_cfm, 0.75, 30.25).unwrap();
        let hpb_frac_age = tpx(&config_hpb, 0.75, 30.25).unwrap();

        println!("UDD: ₀.₇₅p₃₀.₂₅ = {udd_frac_age:.6}");
        println!("CFM: ₀.₇₅p₃₀.₂₅ = {cfm_frac_age:.6}");
        println!("HPB: ₀.₇₅p₃₀.₂₅ = {hpb_frac_age:.6}");
    }

    #[test]
    fn test_error_handling() {
        let xml = MortXML::from_url_id(912).expect("Failed to load XML");
        let config = MortTableConfig {
            xml,
            radix: Some(100_000),
            pct: Some(1.0),
            int_rate: None,
            assumption: Some(AssumptionEnum::UDD),
        };

        // Test unsupported assumption
        let config_invalid = MortTableConfig {
            xml: config.xml.clone(),
            radix: Some(100_000),
            pct: Some(1.0),
            int_rate: None,
            assumption: None, // No assumption specified
        };

        let result = tpx(&config_invalid, 0.5, 30.25);
        assert!(result.is_err());
    }
}
