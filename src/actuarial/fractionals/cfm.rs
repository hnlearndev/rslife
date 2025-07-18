use super::*;

/// CFM (Constant Force of Mortality) Implementation
///
/// Under CFM assumption, the force of mortality μ is constant within each age interval.
///
/// ## Key formulas:
/// - Force of mortality: $\mu_x = -\ln(1 - q_x)$
/// - For fractional time s at age x: ${}_{s}q_x = 1 - (1 - q_x)^s$
/// - For fractional time s at fractional age x+r: ${}_{s}q_{x+r} = 1 - (1 - q_x)^s$
/// - Survival probability: ${}_{s}p_{x+r} = (1 - q_x)^s$
///
/// Calculate ${}_{t}p_x$ under CFM assumption
///
/// # Arguments
/// * `config` - Mortality table configuration
/// * `t` - Time period (can be fractional)
/// * `x` - Starting age (can be fractional)
///
/// # Returns
/// Probability of surviving t years starting at age x
pub fn tpx_cfm(config: &MortTableConfig, t: f64, x: f64) -> PolarsResult<f64> {
    if t <= 0.0 {
        return Err(PolarsError::ComputeError(
            "Time period t must be positive".into(),
        ));
    }

    // Case 1: Both t and x are whole numbers
    if is_whole_number(x) && is_whole_number(t) {
        return tpx_whole_cfm(config, t as i32, x as i32);
    }

    // Case 2: x is fractional, t can be anything
    if !is_whole_number(x) {
        return tpx_fractional_age_cfm(config, t, x);
    }

    // Case 3: x is whole, t is fractional
    tpx_fractional_time_cfm(config, t, x)
}

/// CFM Case 1: Both age and time are whole numbers
///
/// **Formula**: ${}_{t}p_x = \prod_{k=0}^{t-1} p_{x+k}$
fn tpx_whole_cfm(config: &MortTableConfig, t: i32, x: i32) -> PolarsResult<f64> {
    let mut survival_prob = 1.0;

    for age in x..(x + t) {
        let mortality_rate = qx(config, age)?;
        let px = 1.0 - mortality_rate;
        survival_prob *= px;
    }

    Ok(survival_prob)
}

/// CFM Case 2: Age x is fractional (x = n + f where n is whole, 0 < f < 1)
///
/// Need to handle survival from fractional age through potentially multiple years.
///
/// **Case 2a**: ${}_{t}p_{x+f} = (p_x)^t$ when t ≤ (1-f)
///
/// **Case 2b**: ${}_{t}p_{x+f} = (p_x)^{1-f} \cdot {}_{t-(1-f)}p_{x+1}$ when t > (1-f)
fn tpx_fractional_age_cfm(config: &MortTableConfig, t: f64, x: f64) -> PolarsResult<f64> {
    let x_whole = x.floor() as i32; // n
    let x_frac = x.fract(); // f
    let time_to_next_age = 1.0 - x_frac;

    // Get mortality rate for age n (percentage already applied in qx function)
    let mortality_rate = qx(config, x_whole)?;
    let px = 1.0 - mortality_rate;

    if t <= time_to_next_age {
        // Case 2a: Time period ends before reaching next whole age
        // CFM Formula: ${}_{t}p_{x+f} = (p_x)^t$
        // Since force is constant, fractional age doesn't affect the formula
        Ok(px.powf(t))
    } else {
        // Case 2b: Time period crosses into next age year(s)
        // First, survive to age n+1 from age n+f
        let survival_to_next_age = px.powf(time_to_next_age);

        // Then calculate survival for remaining time from age n+1
        let remaining_time = t - time_to_next_age;
        let survival_after = tpx_cfm(config, remaining_time, (x_whole + 1) as f64)?;
        let result = survival_to_next_age * survival_after;
        Ok(result)
    }
}

/// CFM Case 3: Age x is whole, time t is fractional
///
/// **Formula**: ${}_{t}p_x = (p_x)^t$ (since force of mortality is constant)
fn tpx_fractional_time_cfm(config: &MortTableConfig, t: f64, x: f64) -> PolarsResult<f64> {
    let x_whole = x as i32;
    let t_whole = t.floor() as i32;
    let t_frac = t.fract();

    if t_frac == 0.0 {
        // t is actually whole
        return tpx_whole_cfm(config, t_whole, x_whole);
    }

    // First survive the whole years
    let mut survival_prob = tpx_whole_cfm(config, t_whole, x_whole)?;

    // Then survive the fractional part of the last year
    if t_frac > 0.0 {
        let last_age = x_whole + t_whole;
        let mortality_rate = qx(config, last_age)?;
        let px = 1.0 - mortality_rate;
        // CFM formula for fractional time: ${}_{t}p_x = (p_x)^t$
        let fractional_survival = px.powf(t_frac);
        survival_prob *= fractional_survival;
    }

    Ok(survival_prob)
}

/// Calculate ${}_{t}q_x$ under CFM assumption
///
/// **Formula**: ${}_{t}q_x = 1 - {}_{t}p_x = 1 - (p_x)^t$
pub fn tqx_cfm(config: &MortTableConfig, t: f64, x: f64) -> PolarsResult<f64> {
    let result = 1.0 - tpx_cfm(config, t, x)?;
    Ok(result)
}

/// Calculate conditional probability: ${}_{t}q_{x+s}$ given survival to age x+s
///
/// **Formula under CFM**: ${}_{t}q_{x+s} = 1 - (p_x)^t$
///
/// Since force is constant, fractional starting age doesn't change the formula
pub fn conditional_tqx_cfm(config: &MortTableConfig, t: f64, x: f64, _s: f64) -> PolarsResult<f64> {
    let x_whole = x.floor() as i32;
    let mortality_rate = qx(config, x_whole)?;
    let px = 1.0 - mortality_rate;

    // CFM conditional mortality formula: same as regular since force is constant
    let result = 1.0 - px.powf(t);
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::xml::MortXML;

    #[test]
    fn test_cfm_whole_numbers() {
        let xml = MortXML::from_url_id(912).expect("Failed to load XML");
        let config = MortTableConfig {
            xml,
            l_x_init: 100_000,
            pct: Some(1.0),
            int_rate: None,
            assumption: Some(AssumptionEnum::CFM),
        };

        // Test whole number case: ${}_{5}p_{30}$
        let survival_prob = tpx_cfm(&config, 5.0, 30.0).unwrap();
        println!("CFM: ₅p₃₀ = {:.6}", survival_prob);
        assert!(survival_prob > 0.0 && survival_prob <= 1.0);
    }

    #[test]
    fn test_cfm_fractional_time() {
        let xml = MortXML::from_url_id(912).expect("Failed to load XML");
        let config = MortTableConfig {
            xml,
            l_x_init: 100_000,
            pct: Some(1.0),
            int_rate: None,
            assumption: Some(AssumptionEnum::CFM),
        };

        // Test fractional time: ${}_{0.5}p_{30}$
        let survival_prob = tpx_cfm(&config, 0.5, 30.0).unwrap();
        println!("CFM: ₀.₅p₃₀ = {:.6}", survival_prob);
        assert!(survival_prob > 0.0 && survival_prob <= 1.0);
    }

    #[test]
    fn test_cfm_fractional_age() {
        let xml = MortXML::from_url_id(912).expect("Failed to load XML");
        let config = MortTableConfig {
            xml,
            l_x_init: 100_000,
            pct: Some(1.0),
            int_rate: None,
            assumption: Some(AssumptionEnum::CFM),
        };

        // Test fractional age: 1*p_30.25
        let survival_prob = tpx_cfm(&config, 1.0, 30.25).unwrap();
        println!("CFM: 1*p_30.25 = {:.6}", survival_prob);
        assert!(survival_prob > 0.0 && survival_prob <= 1.0);
    }

    #[test]
    fn test_cfm_both_fractional() {
        let xml = MortXML::from_url_id(912).expect("Failed to load XML");
        let config = MortTableConfig {
            xml,
            l_x_init: 100_000,
            pct: Some(1.0),
            int_rate: None,
            assumption: Some(AssumptionEnum::CFM),
        };

        // Test both fractional: 1.5*p_30.25
        let survival_prob = tpx_cfm(&config, 1.5, 30.25).unwrap();
        println!("CFM: 1.5*p_30.25 = {:.6}", survival_prob);
        assert!(survival_prob > 0.0 && survival_prob <= 1.0);
    }

    #[test]
    fn test_cfm_mortality_probability() {
        let xml = MortXML::from_url_id(912).expect("Failed to load XML");
        let config = MortTableConfig {
            xml,
            l_x_init: 100_000,
            pct: Some(1.0),
            int_rate: None,
            assumption: Some(AssumptionEnum::CFM),
        };

        // Test mortality probability: 0.5*q_30
        let mortality_prob = tqx_cfm(&config, 0.5, 30.0).unwrap();
        let survival_prob = tpx_cfm(&config, 0.5, 30.0).unwrap();

        println!("CFM: 0.5*q_30 = {:.6}", mortality_prob);
        println!("CFM: 0.5*p_30 = {:.6}", survival_prob);

        // They should sum to 1
        assert!((mortality_prob + survival_prob - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_conditional_mortality() {
        let xml = MortXML::from_url_id(912).expect("Failed to load XML");
        let config = MortTableConfig {
            xml,
            l_x_init: 100_000,
            pct: Some(1.0),
            int_rate: None,
            assumption: Some(AssumptionEnum::CFM),
        };

        // Test conditional mortality: 0.5*q_(30.25) given survival to age 30.25
        let conditional_mortality = conditional_tqx_cfm(&config, 0.5, 30.0, 0.25).unwrap();
        println!("CFM: 0.5*q_(30.25) = {:.6}", conditional_mortality);
        assert!(conditional_mortality >= 0.0 && conditional_mortality <= 1.0);
    }

    #[test]
    fn test_assumption_comparison() {
        let xml = MortXML::from_url_id(912).expect("Failed to load XML");
        let config = MortTableConfig {
            xml,
            l_x_init: 100_000,
            pct: Some(1.0),
            int_rate: None,
            assumption: Some(AssumptionEnum::CFM),
        };

        // Compare CFM survival probabilities for fractional time
        let cfm_05 = tpx_cfm(&config, 0.5, 30.0).unwrap();
        let cfm_025 = tpx_cfm(&config, 0.25, 30.0).unwrap();
        let cfm_075 = tpx_cfm(&config, 0.75, 30.0).unwrap();

        println!("CFM: 0.25*p_30 = {:.6}", cfm_025);
        println!("CFM: 0.5*p_30 = {:.6}", cfm_05);
        println!("CFM: 0.75*p_30 = {:.6}", cfm_075);

        // CFM survival should decrease with longer time periods
        assert!(cfm_025 > cfm_05);
        assert!(cfm_05 > cfm_075);
    }

    #[test]
    fn test_error_handling() {
        let xml = MortXML::from_url_id(912).expect("Failed to load XML");
        let config = MortTableConfig {
            xml,
            l_x_init: 100_000,
            pct: Some(1.0),
            int_rate: None,
            assumption: Some(AssumptionEnum::CFM),
        };

        // Test that negative time period returns error
        let result = tpx_cfm(&config, -1.0, 30.0);
        assert!(result.is_err());

        // Test that zero time period returns error
        let result = tpx_cfm(&config, 0.0, 30.0);
        assert!(result.is_err());
    }
}
