#![allow(non_snake_case)]
use crate::benefits::nEx;
use crate::int_rate_convert::eff_i_to_nom_i;
use crate::mt_config::MortTableConfig;
use crate::survivals::tpx;
use polars::prelude::*;

// ================================================
// ANNUITY-CERTAIN
// ================================================

// Annuity-certain in arrears:
// ₜ| aₙ⁽ᵐ⁾ =  vᵗ . (1 - vⁿ) / i⁽ᵐ⁾
pub fn an(n: u32, t: u32, m: u32, eff_i: f64) -> PolarsResult<f64> {
    if n == 0 {
        return Ok(0.0);
    }

    if eff_i < 0.0 {
        return Err(PolarsError::ComputeError(
            "Effective interest rate must be non-negative".into(),
        ));
    }

    if m == 0 {
        return Err(PolarsError::ComputeError(
            "Number of payments per year must be greater than zero".into(),
        ));
    }

    let i_m = eff_i_to_nom_i(eff_i, m);
    let v = 1.0 / (1.0 + eff_i);
    let n_f64 = n as f64;
    let t_f64 = t as f64;

    let result = v.powf(t_f64) * (1.0 - v.powf(n_f64)) / i_m;
    Ok(result)
}

// Annuity-certain due:
// ₜ| äₙ⁽ᵐ⁾ = vᵗ · (1 - vⁿ) / d⁽ᵐ⁾ = ₜ| aₙ⁽ᵐ⁾ * i⁽ᵐ⁾ / d⁽ᵐ⁾
pub fn aan(n: u32, t: u32, m: u32, eff_i: f64) -> PolarsResult<f64> {
    let d_m = eff_i_to_nom_i(eff_i, m);
    let i_m = eff_i_to_nom_i(eff_i, m);
    let an_value = an(n, t, m, eff_i)?;
    let result = an_value * i_m / d_m;
    Ok(result)
}

// =======================================
// LIFE ANNUITY
// =======================================

//-----------------Basic------------------
/// Life annuity-due payable m times per year:
/// ₜ|äₓ⁽ᵐ⁾ = ₜEₓ · Σₖ₌₀^∞ (1/m . vᵏ/ᵐ . ₖ/ₘpₓ₊ₜ)
/// Present value of 1/m paid m times per year for life.
pub fn aax(
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

        // Discount factor for the k-th moment
        let discount_factor_with_moment = v.powf(moment_f64 * k / m_f64); // vᵏ/ᵐ
        // Probability of death
        let probability = tpx(config, (x + t) as f64, k / m_f64, 0.0, entry_age)?; // ₖ/ₘpₓ₊ₜ
        // Annuity payment amount
        let benefit_amount = 1.0 / m_f64; // 1/m
        // Aggregate the result
        summation += discount_factor_with_moment * probability * benefit_amount;

        // Next k
        k += 1.0;
    }

    let result = summation * nEx(&config, x, t, moment, entry_age)?; // ₜEₓ
    Ok(result)
}

/// Due temporary annuity-due payable m times per year:
/// ₜ|äₓ:ₙ̅⁽ᵐ⁾ = ₜEₓ · Σₖ₌₀^∞ (1/m . vᵏ/ᵐ . ₖ/ₘpₓ₊ₜ)
/// Present value of 1/m paid m times per year for up to n years.
pub fn aaxn(
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
        let discount_factor_with_moment = v.powf(moment_f64 * k_f64 / m_f64); // vᵏ/ᵐ
        // Probability of death
        let probability = tpx(config, (x + t) as f64, k_f64 / m_f64, 0.0, entry_age)?; // ₖ/ₘpₓ₊ₜ
        // Annuity payment amount
        let benefit_amount = 1.0 / m_f64; // 1/m
        // Aggregate the result
        summation += discount_factor_with_moment * probability * benefit_amount;
    }

    let result = summation * nEx(&config, x, t, moment, entry_age)?; // ₜEₓ
    Ok(result)
}

//-----------------Increasing------------------
/// Increasing life annuity-due payable m times per year:
/// ₜ|(Iä)ₓ⁽ᵐ⁾ = ₜEₓ · Σₖ₌₀^∞ ([(k // m) + 1] / m . vᵏ/ᵐ . ₖ/ₘpₓ₊ₜ)
/// Eg: for m=12, k=0, 1, ..., 11 annuity is 1/m, while k= 12, 13, ..., 23 the annuity is 2/m, etc.
/// Present value of an increasing life annuity-due: payments of 1/m made m times per year for life, with each annual payment increasing by 1.
pub fn Iaax(
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

        // Discount factor for the k-th moment vᵏ/ᵐ
        let discount_factor_with_moment = v.powf(moment_f64 * k / m_f64);
        // Probability of death ₖ/ₘpₓ₊ₜ
        let probability = tpx(config, (x + t) as f64, k / m_f64, 0.0, entry_age)?;
        // Annuity payment amount [(k // m) + 1] / m
        let benefit_amount = ((k / m_f64).floor() + 1.0) / m_f64;
        // Aggregate the result
        summation += discount_factor_with_moment * probability * benefit_amount;

        // Next k
        k += 1.0;
    }

    let result = summation * nEx(&config, x, t, moment, entry_age)?; // ₜEₓ
    Ok(result)
}

/// Due increasing temporary life annuity-due payable m times per year:
/// ₜ|(Iä)ₓ⁽ᵐ⁾ = ₜEₓ · Σₖ₌₀^{mn-1} ([(k // m) + 1] / m . vᵏ/ᵐ . ₖ/ₘpₓ₊ₜ)
/// Eg: for m=12, k=0, 1, ..., 11 annuity is 1/m, while k= 12, 13, ..., 23 the annuity is 2/m, etc.
/// Present value of an increasing life annuity-due: payments of 1/m made m times per year for n years, with each annual payment increasing by 1.
pub fn Iaaxn(
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

        // Discount factor for the k-th moment vᵏ/ᵐ
        let discount_factor_with_moment = v.powf(moment_f64 * k_f64 / m_f64);
        // Probability of death ₖ/ₘpₓ₊ₜ
        let probability = tpx(config, (x + t) as f64, k_f64 / m_f64, 0.0, entry_age)?;
        // Annuity payment amount [(k // m) + 1] / m
        let benefit_amount = ((k_f64 / m_f64).floor() + 1.0) / m_f64;
        // Aggregate the result
        summation += discount_factor_with_moment * probability * benefit_amount;
    }

    let result = summation * nEx(&config, x, t, moment, entry_age)?; // ₜEₓ
    Ok(result)
}

//-----------------Decreasing------------------
/// Decreasing temporary life annuity-due:
/// ₜ|(Iä)ₓ⁽ᵐ⁾ = ₜEₓ · Σₖ₌₀^{mn-1} ([n-(k // m)] / m. vᵏ/ᵐ . ₖ/ₘpₓ₊ₜ)
/// Eg: for m=12, k=0, 1, ..., 11 annuity is 1/m, while k= 12, 13, ..., 23 the annuity is 2/m, etc.
/// Present value of an decreasing life annuity-due: payments of n/m made m times per year for n years, with each annual payment decreasing by 1.
pub fn Daaxn(
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

        // Discount factor for the k-th moment vᵏ/ᵐ
        let discount_factor_with_moment = v.powf(moment_f64 * k_f64 / m_f64);
        // Probability of death ₖ/ₘpₓ₊ₜ
        let probability = tpx(config, (x + t) as f64, k_f64 / m_f64, 0.0, entry_age)?;
        // Annuity payment amount [n-(k // m) + 1] / m
        let benefit_amount = (n_f64 - (k_f64 / m_f64).floor()) / m_f64;
        // Aggregate the result
        summation += discount_factor_with_moment * probability * benefit_amount;
    }

    let result = summation * nEx(&config, x, t, moment, entry_age)?; // ₜEₓ
    Ok(result)
}

//-----------------Geometric increasing------------------
/// Geometric  increasing life annuity-due:
/// äₓ⁽ᵍ⁾ with adjusted interest rate i′ = (1+i)/(1+g) - 1
///
/// Payments of 1/m are made m times per year for life, starting immediately,
/// with each annual payment increasing by a factor of (1+g) each year (i.e., geometric progression).
pub fn gaax(
    config: &MortTableConfig,
    x: u32,
    g: f64,
    t: u32,
    m: u32,
    moment: u32,
    entry_age: Option<u32>,
) -> PolarsResult<f64> {
    let new_config = get_new_config_geometric_functions(config, g)?;
    let result = aax(&new_config, x, t, m, moment, entry_age)?;
    Ok(result)
}

/// Geometric increasing temporary annuity-due:
/// äₓ:ₙ̅⁽ᵍ⁾ with adjusted interest rate i′ = (1+i)/(1+g) - 1
///
/// Payments of 1/m are made m times per year for n years, starting immediately,
/// with each annual payment increasing by a factor of (1+g) each year (i.e., geometric progression).
pub fn gaaxn(
    config: &MortTableConfig,
    x: u32,
    n: u32,
    g: f64,
    t: u32,
    m: u32,
    moment: u32,
    entry_age: Option<u32>,
) -> PolarsResult<f64> {
    let adjusted_config = get_new_config_geometric_functions(config, g)?;
    let result = aaxn(&adjusted_config, x, n, t, m, moment, entry_age)?;
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
