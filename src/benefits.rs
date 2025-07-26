#![allow(non_snake_case)]
use crate::mt_config::MortTableConfig;
use crate::survivals::{tpx, tqx};
use polars::prelude::*;

// =======================================
// PUBLIC FUNCTIONS
// =======================================

//-----------------Basic------------------

/// Immediate pure endowment:
/// ₙEₓ = vⁿ . ₙpₓ
/// ₜ|ₙEₓ = ₜ+ₙEₓ = vⁿ⁺ᵗ . ₜ+ₙpₓ
///
/// Present value of $1 paid if and only if the insured survives n years.
/// **Note**:
/// Due to the fact that ₜ|ₙEₓ = ₜ+ₙEₓ we remove the t parameter. No deferred pure endowment is supported.
/// Moment is also barely used for pure endowment, so we ignore it
pub fn nEx(
    config: &MortTableConfig,
    x: u32,
    n: u32,
    moment: u32,
    entry_age: Option<u32>,
) -> PolarsResult<f64> {
    // Moment must be greater than 0
    if moment == 0 {
        return Err(PolarsError::ComputeError(
            "Moment must be greater than 0".into(),
        ));
    }

    let v = 1.0 / (1.0 + config.int_rate.unwrap_or(0.0));
    let n_f64 = n as f64;
    let moment_f64 = moment as f64;

    // Present value factor for n years
    let discount_factor = v.powf(moment_f64 * n_f64);

    // Probability of surviving n years from age x
    let prob = tpx(config, x as f64, n_f64, 0.0, entry_age)?;
    Ok(discount_factor * prob)
}

/// Immediate whole life insurance:
/// Present value of $1 paid only if death occurs
/// ₜ|Aₓ⁽ᵐ⁾ = ₜEₓ · Σₖ₌₀^∞ [v⁽ᵏ⁺¹⁾/ᵐ . ₖ/ₘ|₁/ₘqₓ₊ₜ]
/// Note: for higher moment replace v with v^(moment)
pub fn Ax(
    config: &MortTableConfig,
    x: u32,
    t: u32,
    m: u32,
    moment: u32,
    entry_age: Option<u32>,
) -> PolarsResult<f64> {
    // Moment must be greater than 0
    if m == 0 {
        return Err(PolarsError::ComputeError(
            "m  must be greater than 0".into(),
        ));
    }

    // Moment must be greater than 0
    if moment == 0 {
        return Err(PolarsError::ComputeError(
            "Moment must be greater than 0".into(),
        ));
    }

    // Get the max age from the config
    let df = &config.xml.tables[0].values;
    let max_age = df.column("age")?.u32()?.max().unwrap();

    let v = 1.0 / (1.0 + config.int_rate.unwrap_or(0.0));
    let m_f64 = m as f64;
    let moment_f64 = moment as f64;

    // Intialize
    let mut summation = 0.0;
    let mut k = 0.0;

    // Main loop to calculate the sum
    loop {
        // Break if x + t + k/m >  max_age then break the loop
        if ((x + t) as f64 + k / m_f64) > (max_age as f64) {
            break;
        }

        // Discount factor for the k-th moment v^[(moment·(k+1))/m]
        let discount_factor_with_moment = v.powf(moment_f64 * (k + 1.0) / m_f64);
        // Probability of death ₖ/ₘ|₁/ₘqₓ₊ₜ
        let probability = tqx(config, (x + t) as f64, 1.0 / m_f64, k / m_f64, entry_age)?;
        // Aggregate the result
        summation += discount_factor_with_moment * probability;

        // Next k
        k += 1.0;
    }

    let result = summation * nEx(&config, x, t, moment, entry_age)?; // ₜEₓ
    Ok(result)
}

/// Immediate term life insurance:
/// ₜ|A¹ₓ:ₙ̅⁽ᵐ⁾ = ₜpₓ · Σₖ₌₀^{mn-1} [v⁽ᵏ⁺¹⁾/ᵐ · ₖ/ₘ|₁/ₘqₓ₊ₜ]
/// Note: for higher moment replace v with v^(moment)
pub fn Ax1n(
    config: &MortTableConfig,
    x: u32,
    n: u32,
    t: u32,
    m: u32,
    moment: u32,
    entry_age: Option<u32>,
) -> PolarsResult<f64> {
    // Moment must be greater than 0
    if m == 0 {
        return Err(PolarsError::ComputeError(
            "m  must be greater than 0".into(),
        ));
    }

    // Moment must be greater than 0
    if moment == 0 {
        return Err(PolarsError::ComputeError(
            "Moment must be greater than 0".into(),
        ));
    }

    // Get the max age from the config
    let df = &config.xml.tables[0].values;
    let max_age = df.column("age")?.u32()?.max().unwrap();

    // x + n must not be greater than max age
    if x + n > max_age {
        return Err(PolarsError::ComputeError(
            "x + n must not be greater than max age".into(),
        ));
    }

    let v = 1.0 / (1.0 + config.int_rate.unwrap_or(0.0));
    let m_f64 = m as f64;
    let moment_f64 = moment as f64;

    // Intialize
    let mut summation = 0.0;

    // Main loop to calculate the sum
    for k in 0..(n * m - 1) {
        let k_f64 = k as f64;

        // Discount factor for the k-th moment
        let discount_factor_with_moment = v.powf(moment_f64 * (k_f64 + 1.0) / m_f64); // v⁽ᵏ⁺¹⁾/ᵐ
        // Probability of death
        let probability = tqx(
            config,
            (x + t) as f64,
            1.0 / m_f64,
            k_f64 / m_f64,
            entry_age,
        )?; // ₖ/ₘ|₁/ₘqₓ₊ₜ
        // Aggregate the result
        summation += discount_factor_with_moment * probability;
    }

    let result = summation * nEx(&config, x, t, moment, entry_age)?; // ₜEₓ
    Ok(result)
}

/// Immediate Endowment insurance:
/// ₜ|Aₓ:ₙ̅⁽ᵐ⁾ = ₜ|A¹ₓ:ₙ̅⁽ᵐ⁾ + ₜ|ₙEₓ
///
/// $1 paid at death (if within n years) OR at survival to n years.
pub fn Axn(
    config: &MortTableConfig,
    x: u32,
    n: u32,
    t: u32,
    m: u32,
    moment: u32,
    entry_age: Option<u32>,
) -> PolarsResult<f64> {
    let term = Ax1n(config, x, n, t, m, moment, entry_age)?;
    let pure_endowment = nEx(config, x, t + n, moment, entry_age)?;
    let result = term + pure_endowment;
    Ok(result)
}

//-----------------Increasing------------------

/// Immediate increasing whole life:
/// ₜ|(IA)ₓ⁽ᵐ⁾ = ₜEₓ · Σₖ₌₀^∞ (v⁽ᵏ⁺¹⁾/ᵐ . ₖ/ₘ|₁/ₘqₓ₊ₜ . [(k // m) + 1] / m)
/// Eg: for m=12, k=0, 1, ..., 11 the death benefit is 1/m, while k= 12, 13, ..., 23 the death benefit is 2/m, etc.
/// Death benefit increases by 1 each year: k paid if death in k-th year.
pub fn IAx(
    config: &MortTableConfig,
    x: u32,
    t: u32,
    m: u32,
    moment: u32,
    entry_age: Option<u32>,
) -> PolarsResult<f64> {
    // Moment must be greater than 0
    if m == 0 {
        return Err(PolarsError::ComputeError(
            "m  must be greater than 0".into(),
        ));
    }

    // Moment must be greater than 0
    if moment == 0 {
        return Err(PolarsError::ComputeError(
            "Moment must be greater than 0".into(),
        ));
    }

    // Get the max age from the config
    let df = &config.xml.tables[0].values;
    let max_age = df.column("age")?.u32()?.max().unwrap();

    let v = 1.0 / (1.0 + config.int_rate.unwrap_or(0.0));
    let m_f64 = m as f64;
    let moment_f64 = moment as f64;

    // Intialize
    let mut summation = 0.0;
    let mut k = 0.0;

    // Main loop to calculate the sum
    loop {
        // Break if x + t + k/m >=  max_age then break the loop
        if ((x + t) as f64 + k / m_f64) >= max_age as f64 {
            break;
        }
        // Discount factor for the k-th moment v⁽ᵏ⁺¹⁾/ᵐ
        let discount_factor_with_moment = v.powf(moment_f64 * (k + 1.0) / m_f64);
        // Probability of death ₖ/ₘ|₁/ₘqₓ₊ₜ
        let probability = tqx(config, (x + t) as f64, 1.0 / m_f64, k / m_f64, entry_age)?;
        // Amount of benefit [(k // m) + 1] / m
        let benefit_amount = ((k / m_f64).floor() + 1.0) / m_f64;
        // Aggregate the result
        summation += discount_factor_with_moment * probability * benefit_amount;

        // Next k
        k += 1.0;
    }

    let result = summation * nEx(&config, x, t, moment, entry_age)?; // ₜEₓ
    Ok(result)
}

/// Immediate increasing term:
/// ₜ|(IA)¹ₓ:ₙ̅⁽ᵐ⁾ = ₜEₓ ·  Σₖ₌₀^{mn-1} (v⁽ᵏ⁺¹⁾/ᵐ . ₖ/ₘ|₁/ₘqₓ₊ₜ . [(k // m) + 1] / m)
/// Eg: for m=12, k=0, 1, ..., 11 the death benefit is 1/m, while k= 12, 13, ..., 23 the death benefit is 2/m, etc.
/// Death benefit increases by 1 each year, pays only if death within n years.
pub fn IAx1n(
    config: &MortTableConfig,
    x: u32,
    n: u32,
    t: u32,
    m: u32,
    moment: u32,
    entry_age: Option<u32>,
) -> PolarsResult<f64> {
    // Moment must be greater than 0
    if m == 0 {
        return Err(PolarsError::ComputeError(
            "m  must be greater than 0".into(),
        ));
    }

    // Moment must be greater than 0
    if moment == 0 {
        return Err(PolarsError::ComputeError(
            "Moment must be greater than 0".into(),
        ));
    }

    // Get the max age from the config
    let df = &config.xml.tables[0].values;
    let max_age = df.column("age")?.u32()?.max().unwrap();

    // x + n must not be greater than max age
    if x + n > max_age {
        return Err(PolarsError::ComputeError(
            "x + n must not be greater than max age".into(),
        ));
    }

    let v = 1.0 / (1.0 + config.int_rate.unwrap_or(0.0));
    let m_f64 = m as f64;
    let moment_f64 = moment as f64;

    // Intialize
    let mut summation = 0.0;

    // Main loop to calculate the sum
    for k in 0..(n * m - 1) {
        let k_f64 = k as f64;

        // Discount factor for the k-th moment v⁽ᵏ⁺¹⁾/ᵐ
        let discount_factor_with_moment = v.powf(moment_f64 * (k_f64 + 1.0) / m_f64);
        // Probability of death ₖ/ₘ|₁/ₘqₓ₊ₜ
        let probability = tqx(
            config,
            (x + t) as f64,
            1.0 / m_f64,
            k_f64 / m_f64,
            entry_age,
        )?;
        // Amount of benefit [(k // m) + 1] / m
        let benefit_amount = ((k_f64 / m_f64).floor() + 1.0) / m_f64;
        // Aggregate the result
        summation += discount_factor_with_moment * probability * benefit_amount;
    }

    let result = summation * nEx(&config, x, t, moment, entry_age)?; // ₜEₓ
    Ok(result)
}

/// Immediate endowment insurance:
/// ₜ|IAₓ:ₙ̅ = ₜ|IA¹ₓ:ₙ̅ + ₜ|IA¹ₓ:ₙ̅
/// $1 paid at death (if within n years) OR at survival to n years.
pub fn IAxn(
    config: &MortTableConfig,
    x: u32,
    n: u32,
    t: u32,
    m: u32,
    moment: u32,
    entry_age: Option<u32>,
) -> PolarsResult<f64> {
    // Decide if selected table is used
    let term = IAx1n(config, x, n, t, m, moment, entry_age)?;
    let pure_endowment = (n as f64) * nEx(config, x, n, moment, entry_age)?;
    let result = term + pure_endowment;
    Ok(result)
}

//-----------------Decreasing------------------
// Note: There should starting amount hence DAₓ is not applicable

/// Immediate decreasing term:
/// ₜ|(DA¹)ₓ:ₙ̅ = ₜEₓ ·  ∏ₖ₌₀^{mn-1} (v⁽ᵏ⁺¹⁾/ᵐ . ₖ/ₘ|₁/ₘqₓ₊ₜ . [n-(k // m)] / m)
/// Eg: for m=12, k=0, 1, ..., 11 the death benefit is n/m, while k= 12, 13, ..., 23 the death benefit is (n-1)/m, etc.
/// Death benefit decreases by 1 each policy year (n in year 1, n-1 in year 2, ..., 1 in year n),
/// pays only if death occurs within n years.
pub fn DAx1n(
    config: &MortTableConfig,
    x: u32,
    n: u32,
    t: u32,
    m: u32,
    moment: u32,
    entry_age: Option<u32>,
) -> PolarsResult<f64> {
    // Moment must be greater than 0
    if m == 0 {
        return Err(PolarsError::ComputeError(
            "m  must be greater than 0".into(),
        ));
    }

    // Moment must be greater than 0
    if moment == 0 {
        return Err(PolarsError::ComputeError(
            "Moment must be greater than 0".into(),
        ));
    }

    // Get the max age from the config
    let df = &config.xml.tables[0].values;
    let max_age = df.column("age")?.u32()?.max().unwrap();

    // x + n must not be greater than max age
    if x + n > max_age {
        return Err(PolarsError::ComputeError(
            "x + n must not be greater than max age".into(),
        ));
    }

    let v = 1.0 / (1.0 + config.int_rate.unwrap_or(0.0));
    let m_f64 = m as f64;
    let moment_f64 = moment as f64;
    let n_f64 = n as f64;

    // Intialize
    let mut summation = 0.0;

    // Main loop to calculate the sum
    for k in 0..(n * m - 1) {
        let k_f64 = k as f64;

        // Discount factor for the k-th moment v⁽ᵏ⁺¹⁾/ᵐ
        let discount_factor_with_moment = v.powf(moment_f64 * (k_f64 + 1.0) / m_f64);
        // Probability of death ₖ/ₘ|₁/ₘqₓ₊ₜ
        let probability = tqx(
            config,
            (x + t) as f64,
            1.0 / m_f64,
            k_f64 / m_f64,
            entry_age,
        )?;
        // Amount of benefit [n-(k // m)] / m
        let benefit_amount = (n_f64 - (k_f64 / m_f64).floor()) / m_f64;
        // Aggregate the result
        summation += discount_factor_with_moment * probability * benefit_amount;
    }

    let result = summation * nEx(&config, x, t, moment, entry_age)?; // ₜEₓ
    Ok(result)
}

/// Immediate decreasing endowment insurance:
/// ₜ|(DA)ₓ:ₙ̅ = ₜ|(DA)¹ₓ:ₙ̅ + ₜ|ₙEₓ
/// $1 paid at death (if within n years) OR at survival to n years.
pub fn DAxn(
    config: &MortTableConfig,
    x: u32,
    n: u32,
    t: u32,
    m: u32,
    moment: u32,
    entry_age: Option<u32>,
) -> PolarsResult<f64> {
    let term = DAx1n(config, x, n, t, m, moment, entry_age)?;
    let pure_endowment = nEx(config, x, t + n, moment, entry_age)?;
    let result = term + pure_endowment;
    Ok(result)
}

//-----------------Geometric increasing------------------

/// Immediate geometric whole life:
/// Aₓ⁽ᵍ⁾ with adjusted interest rate i′ = (1+i)/(1+g) - 1
///
/// Death benefit grows geometrically at rate g each year.
pub fn gAx(
    config: &MortTableConfig,
    x: u32,
    g: f64,
    t: u32,
    m: u32,
    moment: u32,
    entry_age: Option<u32>,
) -> PolarsResult<f64> {
    let new_config = get_new_config_geometric_functions(config, g)?;
    let result = Ax(&new_config, x, t, m, moment, entry_age)?;
    Ok(result)
}

/// Immediate geometric n-year term:
/// A¹ₓ:ₙ̅⁽ᵍ⁾ with adjusted interest rate i′ = (1+i)/(1+g) - 1
///
/// Death benefit grows geometrically at rate g for n years.
pub fn gAx1n(
    config: &MortTableConfig,
    x: u32,
    n: u32,
    g: f64,
    t: u32,
    m: u32,
    moment: u32,
    entry_age: Option<u32>,
) -> PolarsResult<f64> {
    let new_config = get_new_config_geometric_functions(config, g)?;
    let result = Ax1n(&new_config, x, n, t, m, moment, entry_age)?;
    Ok(result)
}

/// Immediate geometric n-year pure endowment:
/// Aₓ:ₙ̅¹⁽ᵍ⁾ with adjusted interest rate i′ = (1+i)/(1+g) - 1
///
/// Death benefit grows geometrically at rate g for n years.
pub fn gnEx(
    config: &MortTableConfig,
    x: u32,
    n: u32,
    g: f64,
    moment: u32,
    entry_age: Option<u32>,
) -> PolarsResult<f64> {
    let new_config = get_new_config_geometric_functions(config, g)?;
    let result = nEx(&new_config, x, n, moment, entry_age)?;
    Ok(result)
}

/// Immediate geometric n-year endowment:
/// Aₓ:ₙ̅⁽ᵍ⁾ = A¹ₓ:ₙ̅⁽ᵍ⁾ + Aₓ:ₙ̅¹⁽ᵍ⁾
///
/// Death benefit grows geometrically at rate g for n years.
pub fn gAxn(
    config: &MortTableConfig,
    x: u32,
    n: u32,
    g: f64,
    t: u32,
    m: u32,
    moment: u32,
    entry_age: Option<u32>,
) -> PolarsResult<f64> {
    let term = gAx1n(config, x, n, g, t, m, moment, entry_age)?;
    let pure_endowment = gnEx(config, x, n, g, t, entry_age)?;
    let result = term + pure_endowment;
    Ok(result)
}

// ================================================
// PRIVATE FUNCTIONS
// ================================================
fn get_new_config_geometric_functions(
    config: &MortTableConfig,
    g: f64,
) -> PolarsResult<MortTableConfig> {
    // Replace the effective interest rate with the adjusted one
    let i = config.int_rate.unwrap();
    let int_rate = (1.0 + i) / (1.0 + g) - 1.0;
    let mut new_config = config.clone();
    new_config.int_rate = Some(int_rate);
    Ok(new_config)
}

// ================================================
// UNIT TESTS
// ================================================
#[cfg(test)]
mod tests {
    use super::*;
    use crate::mt_config::{AssumptionEnum, MortTableConfig};
    use crate::xml::MortXML;
    use approx::assert_abs_diff_eq;

    #[test]
    fn test_benefit_cm1_01() {
        // April 2025 CM1 question 1
        #[allow(non_snake_case)]
        // Load AM92 selected table
        let am92_xml = MortXML::from_xlsx("data/am92.xlsx", "am92")
            .expect("Failed to load AM92 selected table");

        // Create MortTableConfig
        let config = MortTableConfig {
            xml: am92_xml,
            int_rate: Some(0.05),
            assumption: Some(AssumptionEnum::UDD),
            ..Default::default()
        };

        // Calculate  A₍₇₀₎:₃
        // Read as Axn age 70, term 3, deffered 0, payble per yearly (end of year), moment 1 (expectation), entry age 70
        let ans = Axn(&config, 70, 3, 0, 1, 1, Some(70)).unwrap();
        let expected = 0.8663440;
        assert_abs_diff_eq!(ans, expected, epsilon = 1e-6);
    }
}
