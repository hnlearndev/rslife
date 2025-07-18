use super::*;

/// UDD (Uniform Distribution of Deaths) Implementation
///
/// Under UDD assumption, deaths are uniformly distributed within each age interval.
///
/// ## Key formulas:
/// - For fractional time s at age x: ${}_{s}q_x = s \cdot q_x / (1 - 0 \cdot q_x) = s \cdot q_x$
/// - For fractional time s at fractional age x+r: ${}_{s}q_{x+r} = s \cdot q_x / (1 - r \cdot q_x)$
/// - Survival probability: ${}_{s}p_{x+r} = 1 - {}_{s}q_{x+r}$
/// - Force of mortality under UDD: $\mu_{x+t} = q_x / (1 - t \cdot q_x)$ for $0 \leq t < 1$
///
/// Calculate ${}_{t}p_x$ under UDD assumption
///
/// # Arguments
/// * `config` - Mortality table configuration
/// * `t` - Time period (can be fractional)
/// * `x` - Starting age (can be fractional)
///
/// # Returns
/// Probability of surviving t years starting at age x
pub fn tpx_udd(config: &MortTableConfig, t: f64, x: f64) -> PolarsResult<f64> {
    if t <= 0.0 {
        return Err(PolarsError::ComputeError(
            "Time period t must be positive".into(),
        ));
    }

    // Case 1: Both t and x are whole numbers
    if is_whole_number(x) && is_whole_number(t) {
        return tpx_whole_udd(config, t as i32, x as i32);
    }

    // Case 2: x is fractional, t can be anything
    if !is_whole_number(x) {
        return tpx_fractional_age_udd(config, t, x);
    }

    // Case 3: x is whole, t is fractional
    tpx_fractional_time_udd(config, t, x)
}

/// UDD Case 1: Both age and time are whole numbers
/// Formula: t*p_x = ∏(k=0 to t-1) p_(x+k)
fn tpx_whole_udd(config: &MortTableConfig, t: i32, x: i32) -> PolarsResult<f64> {
    let mut survival_prob = 1.0;

    for age in x..(x + t) {
        let mortality_rate = qx(config, age)?;
        let px = 1.0 - mortality_rate;
        survival_prob *= px;
    }

    Ok(survival_prob)
}

/// UDD Case 2: Age x is fractional (x = n + f where n is whole, 0 < f < 1)
/// Need to handle survival from fractional age through potentially multiple years
fn tpx_fractional_age_udd(config: &MortTableConfig, t: f64, x: f64) -> PolarsResult<f64> {
    let x_whole = x.floor() as i32; // n
    let x_frac = x.fract(); // f
    let time_to_next_age = 1.0 - x_frac;

    // Get mortality rate for age n (percentage already applied in qx function)
    let mortality_rate = qx(config, x_whole).unwrap_or(0.0);

    if t <= time_to_next_age {
        // Case 2a: Time period ends before reaching next whole age
        // Formula: t*p_(x+f) = 1 - t*q_(x+f)
        // where t*q_(x+f) = t*q_x / (1 - f*q_x)
        let tqx_nf = t * mortality_rate / (1.0 - x_frac * mortality_rate);
        let result = 1.0 - tqx_nf;
        Ok(result)
    } else {
        // Case 2b: Time period crosses into next age year(s)
        // First, survive to age x+1
        // Formula:
        // t*p_(x+f) = survival to age x+1 from age x+f * survival to age (t) from age x+1
        let survival_to_next_age =
            1.0 - ((1.0 - x_frac) * mortality_rate / (1.0 - x_frac * mortality_rate));
        // Then calculate survival for remaining time from age n+1
        let remaining_time = t - time_to_next_age;
        let survival_after = tpx_udd(config, remaining_time, (x_whole + 1) as f64)?;
        let result = survival_to_next_age * survival_after;
        Ok(result)
    }
}

/// UDD Case 3: Age x is whole, time t is fractional
/// Formula: t*p_x = 1 - t*q_x (since at whole age, no prior fraction to consider)
fn tpx_fractional_time_udd(config: &MortTableConfig, t: f64, x: f64) -> PolarsResult<f64> {
    let x_whole = x as i32;
    let t_whole = t.floor() as i32;
    let t_frac = t.fract();

    if t_frac == 0.0 {
        // t is actually whole
        return tpx_whole_udd(config, t_whole, x_whole);
    }

    // First survive the whole years
    let mut survival_prob = tpx_whole_udd(config, t_whole, x_whole)?;

    // Then survive the fractional part of the last year
    if t_frac > 0.0 {
        let last_age = x_whole + t_whole;
        let mortality_rate = qx(config, last_age)?;
        // UDD formula for fractional time at whole age: t*p_x = 1 - t*q_x
        let fractional_survival = 1.0 - (t_frac * mortality_rate);
        survival_prob *= fractional_survival;
    }

    Ok(survival_prob)
}

/// Calculate t*q_x under UDD assumption
/// Formula: t*q_x = 1 - t*p_x
pub fn tqx_udd(config: &MortTableConfig, t: f64, x: f64) -> PolarsResult<f64> {
    let result = 1.0 - tpx_udd(config, t, x)?;
    Ok(result)
}

/// Calculate conditional probability: t*q_(x+s) given survival to age x+s
/// Formula under UDD:
/// t+s p_x = t p_x × (1 - s × q_(x+t))
/// t+s q_x = 1 - t+s p_x = 1 - t p_x × (1 - s × q_(x+t))
pub fn conditional_tqx_udd(config: &MortTableConfig, t: f64, x: f64, s: f64) -> PolarsResult<f64> {
    let x_whole = x.floor() as i32;
    let mortality_rate = qx(config, x_whole).unwrap_or(0.0);

    // UDD conditional mortality formula
    let result = t * mortality_rate / (1.0 - s * mortality_rate);
    Ok(result)
}

/// Calculate force of mortality μ_(x+t) at fractional age under UDD assumption
/// Formula: μ_(x+t) = q_x / (1 - t*q_x) for 0 ≤ t < 1
///
/// # Arguments
/// * `config` - Mortality table configuration
/// * `x` - Whole age
/// * `t` - Fractional time within the age interval (0 ≤ t < 1)
///
/// # Returns
/// Force of mortality at age x+t
pub fn force_of_mortality_udd(config: &MortTableConfig, t: f64, x: i32) -> f64 {
    let mortality_rate = qx(config, x).unwrap_or(0.0);

    if !(0.0..1.0).contains(&t) {
        panic!("Fractional time t must be in range [0, 1) for UDD force calculation");
    }

    let denominator = 1.0 - t * mortality_rate;
    if denominator <= 0.0 {
        f64::INFINITY // If denominator approaches 0, force approaches infinity
    } else {
        mortality_rate / denominator
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::xml::MortXML;

    #[test]
    fn test_udd_whole_numbers() {
        let xml = MortXML::from_url_id(912).expect("Failed to load XML");
        let config = MortTableConfig {
            xml,
            l_x_init: 100_000,
            pct: Some(1.0),
            int_rate: None,
            assumption: Some(AssumptionEnum::UDD),
        };

        // Test whole number case: 5*p_30
        let survival_prob = tpx_udd(&config, 5.0, 30.0).unwrap();
        println!("UDD: 5*p_30 = {:.6}", survival_prob);
        assert!(survival_prob > 0.0 && survival_prob <= 1.0);
    }

    #[test]
    fn test_udd_fractional_time() {
        let xml = MortXML::from_url_id(912).expect("Failed to load XML");
        let config = MortTableConfig {
            xml,
            l_x_init: 100_000,
            pct: Some(1.0),
            int_rate: None,
            assumption: Some(AssumptionEnum::UDD),
        };

        // Test fractional time: 0.5*p_30
        let survival_prob = tpx_udd(&config, 0.5, 30.0).unwrap();
        println!("UDD: 0.5*p_30 = {:.6}", survival_prob);
        assert!(survival_prob > 0.0 && survival_prob <= 1.0);
    }

    #[test]
    fn test_udd_fractional_age() {
        let xml = MortXML::from_url_id(912).expect("Failed to load XML");
        let config = MortTableConfig {
            xml,
            l_x_init: 100_000,
            pct: Some(1.0),
            int_rate: None,
            assumption: Some(AssumptionEnum::UDD),
        };

        // Test fractional age: 1*p_30.25
        let survival_prob = tpx_udd(&config, 1.0, 30.25).unwrap();
        println!("UDD: 1*p_30.25 = {:.6}", survival_prob);
        assert!(survival_prob > 0.0 && survival_prob <= 1.0);
    }

    #[test]
    fn test_udd_both_fractional() {
        let xml = MortXML::from_url_id(912).expect("Failed to load XML");
        let config = MortTableConfig {
            xml,
            l_x_init: 100_000,
            pct: Some(1.0),
            int_rate: None,
            assumption: Some(AssumptionEnum::UDD),
        };

        // Test both fractional: 1.5*p_30.25
        let survival_prob = tpx_udd(&config, 1.5, 30.25).unwrap();
        println!("UDD: 1.5*p_30.25 = {:.6}", survival_prob);
        assert!(survival_prob > 0.0 && survival_prob <= 1.0);
    }

    #[test]
    fn test_udd_mortality_probability() {
        let xml = MortXML::from_url_id(912).expect("Failed to load XML");
        let config = MortTableConfig {
            xml,
            l_x_init: 100_000,
            pct: Some(1.0),
            int_rate: None,
            assumption: Some(AssumptionEnum::UDD),
        };

        // Test mortality probability: 0.5*q_30
        let mortality_prob = tqx_udd(&config, 0.5, 30.0).unwrap();
        let survival_prob = tpx_udd(&config, 0.5, 30.0).unwrap();

        println!("UDD: 0.5*q_30 = {:.6}", mortality_prob);
        println!("UDD: 0.5*p_30 = {:.6}", survival_prob);

        // They should sum to 1
        assert!((mortality_prob + survival_prob - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_percentage_adjustment() {
        let xml = MortXML::from_url_id(912).expect("Failed to load XML");

        // Test with 50% of base rates
        let config_50 = MortTableConfig {
            xml: xml.clone(),
            l_x_init: 100_000,
            pct: Some(0.5),
            int_rate: None,
            assumption: Some(AssumptionEnum::UDD),
        };

        // Test with 100% of base rates
        let config_100 = MortTableConfig {
            xml,
            l_x_init: 100_000,
            pct: Some(1.0),
            int_rate: None,
            assumption: Some(AssumptionEnum::UDD),
        };

        let survival_50 = tpx_udd(&config_50, 1.0, 30.0).unwrap();
        let survival_100 = tpx_udd(&config_100, 1.0, 30.0).unwrap();

        // 50% rates should give higher survival probability
        assert!(survival_50 > survival_100);

        println!("UDD: 1*p_30 with 50% rates = {:.6}", survival_50);
        println!("UDD: 1*p_30 with 100% rates = {:.6}", survival_100);
    }

    #[test]
    fn test_conditional_mortality() {
        let xml = MortXML::from_url_id(912).expect("Failed to load XML");
        let config = MortTableConfig {
            xml,
            l_x_init: 100_000,
            pct: Some(1.0),
            int_rate: None,
            assumption: Some(AssumptionEnum::UDD),
        };

        // Test conditional mortality: 0.5*q_(30.25) given survival to age 30.25
        let conditional_mortality = conditional_tqx_udd(&config, 0.5, 30.0, 0.25).unwrap();
        println!("UDD: 0.5*q_(30.25) = {:.6}", conditional_mortality);
        assert!(conditional_mortality >= 0.0 && conditional_mortality <= 1.0);
    }

    #[test]
    fn test_udd_force_of_mortality() {
        let xml = MortXML::from_url_id(912).expect("Failed to load XML");
        let config = MortTableConfig {
            xml,
            l_x_init: 100_000,
            pct: Some(1.0),
            int_rate: None,
            assumption: Some(AssumptionEnum::UDD),
        };

        // Test instantaneous force at different points in the year
        let mu_30_0 = force_of_mortality_udd(&config, 0.0, 30); // Fixed parameter order
        let mu_30_25 = force_of_mortality_udd(&config, 0.25, 30); // Fixed parameter order
        let mu_30_5 = force_of_mortality_udd(&config, 0.5, 30); // Fixed parameter order
        let mu_30_75 = force_of_mortality_udd(&config, 0.75, 30); // Fixed parameter order

        println!("UDD μ_30.0 = {:.8}", mu_30_0);
        println!("UDD μ_30.25 = {:.8}", mu_30_25);
        println!("UDD μ_30.5 = {:.8}", mu_30_5);
        println!("UDD μ_30.75 = {:.8}", mu_30_75);

        // Under UDD, force should increase throughout the year
        assert!(mu_30_0 < mu_30_25);
        assert!(mu_30_25 < mu_30_5);
        assert!(mu_30_5 < mu_30_75);
    }

    #[test]
    fn test_assumption_comparison() {
        let xml = MortXML::from_url_id(912).expect("Failed to load XML");
        let config = MortTableConfig {
            xml,
            l_x_init: 100_000,
            pct: Some(1.0),
            int_rate: None,
            assumption: Some(AssumptionEnum::UDD),
        };

        // Compare UDD survival probabilities for fractional time
        let udd_05 = tpx_udd(&config, 0.5, 30.0).unwrap();
        let udd_025 = tpx_udd(&config, 0.25, 30.0).unwrap();
        let udd_075 = tpx_udd(&config, 0.75, 30.0).unwrap();

        println!("UDD: 0.25*p_30 = {:.6}", udd_025);
        println!("UDD: 0.5*p_30 = {:.6}", udd_05);
        println!("UDD: 0.75*p_30 = {:.6}", udd_075);

        // UDD survival should decrease with longer time periods
        assert!(udd_025 > udd_05);
        assert!(udd_05 > udd_075);
    }

    #[test]
    fn test_error_handling() {
        let xml = MortXML::from_url_id(912).expect("Failed to load XML");
        let config = MortTableConfig {
            xml,
            l_x_init: 100_000,
            pct: Some(1.0),
            int_rate: None,
            assumption: Some(AssumptionEnum::UDD),
        };

        // Test that negative time period returns error
        let result = tpx_udd(&config, -1.0, 30.0);
        assert!(result.is_err());

        // Test that zero time period returns error
        let result = tpx_udd(&config, 0.0, 30.0);
        assert!(result.is_err());
    }
}
