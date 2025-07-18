use super::*;

/// HPB (Hyperbolic/Balmer) Implementation
///
/// Under HPB assumption, the survival function follows a hyperbolic distribution.
/// Also known as the Balmer assumption.
///
/// ## Key formulas:
/// - For fractional time s at age x: ${}_{s}q_x = s \cdot q_x / (1 + s \cdot q_x)$
/// - For fractional time s at fractional age x+r: ${}_{s}q_{x+r} = s \cdot q_x / (1 + r \cdot q_x)$
/// - Survival probability: ${}_{s}p_{x+r} = 1 - {}_{s}q_{x+r}$
/// - Force of mortality under HPB: $\mu_{x+t} = q_x / (1 + t \cdot q_x)$ for $0 \leq t < 1$
///
/// Calculate ${}_{t}p_x$ under HPB assumption
///
/// # Arguments
/// * `config` - Mortality table configuration
/// * `t` - Time period (can be fractional)
/// * `x` - Starting age (can be fractional)
///
/// # Returns
/// Probability of surviving t years starting at age x
pub fn tpx_hpb(config: &MortTableConfig, t: f64, x: f64) -> PolarsResult<f64> {
    if t <= 0.0 {
        return Ok(1.0); // Probability of surviving 0 time is 1
    }

    // Case 1: Both t and x are whole numbers
    if is_whole_number(x) && is_whole_number(t) {
        return tpx_whole_hpb(config, t as i32, x as i32);
    }

    // Case 2: x is fractional, t can be anything
    if !is_whole_number(x) {
        return tpx_fractional_age_hpb(config, t, x);
    }

    // Case 3: x is whole, t is fractional
    tpx_fractional_time_hpb(config, t, x)
}

/// HPB Case 1: Both age and time are whole numbers
/// Formula: t*p_x = ∏(k=0 to t-1) p_(x+k)
fn tpx_whole_hpb(config: &MortTableConfig, t: i32, x: i32) -> PolarsResult<f64> {
    let mut survival_prob = 1.0;

    for age in x..(x + t) {
        let mortality_rate = qx(config, age)?;
        let px = 1.0 - mortality_rate;
        survival_prob *= px;
    }

    Ok(survival_prob)
}

/// HPB Case 2: Age x is fractional (x = n + f where n is whole, 0 < f < 1)
/// Need to handle survival from fractional age through potentially multiple years
fn tpx_fractional_age_hpb(config: &MortTableConfig, t: f64, x: f64) -> PolarsResult<f64> {
    let x_whole = x.floor() as i32; // n
    let x_frac = x.fract(); // f
    let time_to_next_age = 1.0 - x_frac;

    // Get mortality rate for age n (percentage already applied in qx function)
    let mortality_rate = qx(config, x_whole)?;

    if t <= time_to_next_age {
        // Case 2a: Time period ends before reaching next whole age
        // Formula: t*p_(x+f) = 1 - t*q_(x+f)
        // where t*q_(x+f) = t*q_x / (1 + f*q_x)
        let tqx_nf = t * mortality_rate / (1.0 + x_frac * mortality_rate);
        Ok(1.0 - tqx_nf)
    } else {
        // Case 2b: Time period crosses into next age year(s)
        // First, survive to age x+1 from age x+f
        // Formula: (1-f)*q_(x+f) = (1-f)*q_x / (1 + f*q_x)
        let survival_to_next_age =
            1.0 - ((1.0 - x_frac) * mortality_rate / (1.0 + x_frac * mortality_rate));

        // Then calculate survival for remaining time from age n+1
        let remaining_time = t - time_to_next_age;
        let survival_after = tpx_hpb(config, remaining_time, (x_whole + 1) as f64)?; // Fixed order

        Ok(survival_to_next_age * survival_after)
    }
}

/// HPB Case 3: Age x is whole, time t is fractional
/// Formula: t*p_x = 1 - t*q_x / (1 + 0*q_x) = 1 - t*q_x
fn tpx_fractional_time_hpb(config: &MortTableConfig, t: f64, x: f64) -> PolarsResult<f64> {
    let x_whole = x as i32;
    let t_whole = t.floor() as i32;
    let t_frac = t.fract();

    if t_frac == 0.0 {
        // t is actually whole
        return tpx_whole_hpb(config, t_whole, x_whole);
    }

    // First survive the whole years
    let mut survival_prob = tpx_whole_hpb(config, t_whole, x_whole)?;

    // Then survive the fractional part of the last year
    if t_frac > 0.0 {
        let last_age = x_whole + t_whole;
        let mortality_rate = qx(config, last_age)?;
        // HPB formula for fractional time at whole age: t*p_x = 1 - t*q_x
        // Since we're starting at whole age, no hyperbolic adjustment needed
        let fractional_survival = 1.0 - (t_frac * mortality_rate);
        survival_prob *= fractional_survival;
    }

    Ok(survival_prob)
}

/// Calculate t*q_x under HPB assumption
/// Formula: t*q_x = 1 - t*p_x
pub fn tqx_hpb(config: &MortTableConfig, t: f64, x: f64) -> PolarsResult<f64> {
    Ok(1.0 - tpx_hpb(config, t, x)?) // Fixed order
}

/// Calculate conditional probability: t*q_(x+s) given survival to age x+s
/// Formula under HPB: t*q_(x+s) = t*q_x / (1 + s*q_x)
pub fn conditional_tqx_hpb(config: &MortTableConfig, t: f64, x: f64, s: f64) -> PolarsResult<f64> {
    let x_whole = x.floor() as i32;
    let mortality_rate = qx(config, x_whole)?;

    // HPB conditional mortality formula
    Ok(t * mortality_rate / (1.0 + s * mortality_rate))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::xml::MortXML;

    #[test]
    fn test_hpb_whole_numbers() {
        let xml = MortXML::from_url_id(912).expect("Failed to load XML");
        let config = MortTableConfig {
            xml,
            l_x_init: 100_000,
            pct: Some(1.0),
            int_rate: None,
            assumption: Some(AssumptionEnum::HPB),
        };

        // Test whole number case: 5*p_30
        let survival_prob = tpx_hpb(&config, 5.0, 30.0).unwrap();
        println!("HPB: 5*p_30 = {:.6}", survival_prob);
        assert!(survival_prob > 0.0 && survival_prob <= 1.0);
    }

    #[test]
    fn test_hpb_fractional_time() {
        let xml = MortXML::from_url_id(912).expect("Failed to load XML");
        let config = MortTableConfig {
            xml,
            l_x_init: 100_000,
            pct: Some(1.0),
            int_rate: None,
            assumption: Some(AssumptionEnum::HPB),
        };

        // Test fractional time: 0.5*p_30
        let survival_prob = tpx_hpb(&config, 0.5, 30.0).unwrap();
        println!("HPB: 0.5*p_30 = {:.6}", survival_prob);
        assert!(survival_prob > 0.0 && survival_prob <= 1.0);
    }

    #[test]
    fn test_hpb_fractional_age() {
        let xml = MortXML::from_url_id(912).expect("Failed to load XML");
        let config = MortTableConfig {
            xml,
            l_x_init: 100_000,
            pct: Some(1.0),
            int_rate: None,
            assumption: Some(AssumptionEnum::HPB),
        };

        // Test fractional age: 1*p_30.25
        let survival_prob = tpx_hpb(&config, 1.0, 30.25).unwrap();
        println!("HPB: 1*p_30.25 = {:.6}", survival_prob);
        assert!(survival_prob > 0.0 && survival_prob <= 1.0);
    }

    #[test]
    fn test_hpb_both_fractional() {
        let xml = MortXML::from_url_id(912).expect("Failed to load XML");
        let config = MortTableConfig {
            xml,
            l_x_init: 100_000,
            pct: Some(1.0),
            int_rate: None,
            assumption: Some(AssumptionEnum::HPB),
        };

        // Test both fractional: 1.5*p_30.25
        let survival_prob = tpx_hpb(&config, 1.5, 30.25).unwrap();
        println!("HPB: 1.5*p_30.25 = {:.6}", survival_prob);
        assert!(survival_prob > 0.0 && survival_prob <= 1.0);
    }

    #[test]
    fn test_hpb_mortality_probability() {
        let xml = MortXML::from_url_id(912).expect("Failed to load XML");
        let config = MortTableConfig {
            xml,
            l_x_init: 100_000,
            pct: Some(1.0),
            int_rate: None,
            assumption: Some(AssumptionEnum::HPB),
        };

        // Test mortality probability: 0.5*q_30
        let mortality_prob = tqx_hpb(&config, 0.5, 30.0).unwrap();
        let survival_prob = tpx_hpb(&config, 0.5, 30.0).unwrap();

        println!("HPB: 0.5*q_30 = {:.6}", mortality_prob);
        println!("HPB: 0.5*p_30 = {:.6}", survival_prob);

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
            assumption: Some(AssumptionEnum::HPB),
        };

        // Test conditional mortality: 0.5*q_(30.25) given survival to age 30.25
        let conditional_mortality = conditional_tqx_hpb(&config, 0.5, 30.0, 0.25).unwrap();
        println!("HPB: 0.5*q_(30.25) = {:.6}", conditional_mortality);
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
            assumption: Some(AssumptionEnum::HPB),
        };

        // Compare HPB survival probabilities for fractional time
        let hpb_05 = tpx_hpb(&config, 0.5, 30.0).unwrap();
        let hpb_025 = tpx_hpb(&config, 0.25, 30.0).unwrap();
        let hpb_075 = tpx_hpb(&config, 0.75, 30.0).unwrap();

        println!("HPB: 0.25*p_30 = {:.6}", hpb_025);
        println!("HPB: 0.5*p_30 = {:.6}", hpb_05);
        println!("HPB: 0.75*p_30 = {:.6}", hpb_075);

        // HPB typically gives survival probabilities between UDD and CFM
        assert!(hpb_025 > hpb_05);
        assert!(hpb_05 > hpb_075);
    }
}
