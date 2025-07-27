#![allow(non_snake_case)]
use crate::param_config::ParamConfig;
use crate::survivals::{tpx, tqx};
use polars::prelude::*;

// =======================================
// PUBLIC FUNCTIONS
// =======================================

//-----------------Basic------------------

/// Immediate pure endowment:
/// ₜ|ₙEₓ = ₜ+ₙEₓ = vⁿ⁺ᵗ . ₜ+ₙpₓ ✅
///
/// Present value of $1 paid if and only if the insured survives n years.
pub fn Exn(params: &ParamConfig) -> PolarsResult<f64> {
    // Vaidate the parameters
    params.validate_all()?;

    // Required paramenters: i,x,n
    if params.n.is_none() {
        return Err(PolarsError::ComputeError(
            "n must be provided for Exn".into(),
        ));
    }

    let n = params.n.unwrap() as f64;
    let x = params.x as f64;
    let v = 1.0 / (1.0 + params.i);

    // Default if not provided
    let t = params.t.unwrap_or(0) as f64;
    let moment = params.moment.unwrap_or(1) as f64;

    // As provided - no default
    let mt = &params.mt;
    let entry_age = params.entry_age;

    // Calculation
    let discount_factor = v.powf(moment * (n + t));
    let prob = tpx(mt, x, n, t, entry_age)?;
    let result = discount_factor * prob;
    Ok(result)
}

/// Immediate whole life insurance:
/// Present value of $1 paid only if death occurs
/// ₜ|Aₓ⁽ᵐ⁾ = ₜEₓ · Σₖ₌₀^∞ [v⁽ᵏ⁺¹⁾/ᵐ . ₖ/ₘ|₁/ₘqₓ₊ₜ]
/// Note: for higher moment replace v with v^(moment)
pub fn Ax(params: &ParamConfig) -> PolarsResult<f64> {
    // Vaidate the parameters
    params.validate_all()?;

    // Required paramenters: i,x
    let x = params.x as f64;
    let v = 1.0 / (1.0 + params.i);

    // Default if not provided
    let m = params.m.unwrap_or(1) as f64; // Default to Annual
    let t = params.t.unwrap_or(0) as f64; // Default to 0 (no deferral)
    let moment = params.moment.unwrap_or(1) as f64; // Default moment is 1 (mean)

    // As provided - no default
    let mt = &params.mt;
    let entry_age = params.entry_age;

    // Others
    let max_age = mt.max_age() as f64;

    // Intialize
    let mut summation = 0.0;
    let mut k = 0.0;

    // Main loop to calculate the sum
    loop {
        // Break if x + t + k/m >  max_age then break the loop
        if (x + t + k / m) > max_age {
            break;
        }

        // Discount factor for the k-th moment v^[(moment·(k+1))/m]
        let discount_factor_with_moment = v.powf(moment * (k + 1.0) / m);
        // Probability of death ₖ/ₘ|₁/ₘqₓ₊ₜ
        let probability = tqx(mt, x + t, 1.0 / m, k / m, entry_age)?;
        // Aggregate the result
        summation += discount_factor_with_moment * probability;

        // Exnt k
        k += 1.0;
    }

    // Calculation of ₜEₓ - only term n = t. The rest are intact
    let mut ext_param = params.clone();
    ext_param.n = Some(t as u32);

    let Ext = Exn(&ext_param)?;

    // Final result
    let result = summation * Ext; //
    Ok(result)
}

/// Immediate term life insurance:
/// ₜ|A¹ₓ:ₙ̅⁽ᵐ⁾ = ₜpₓ · Σₖ₌₀^{mn-1} [v⁽ᵏ⁺¹⁾/ᵐ · ₖ/ₘ|₁/ₘqₓ₊ₜ]
/// Note: for higher moment replace v with v^(moment)
pub fn Ax1n(params: &ParamConfig) -> PolarsResult<f64> {
    // Vaidate the parameters
    params.validate_all()?;

    // Required paramenters: i,x,n
    if params.n.is_none() {
        return Err(PolarsError::ComputeError(
            "n must be provided for Ax1n".into(),
        ));
    }

    let n = params.n.unwrap() as f64;
    let x = params.x as f64;
    let v = 1.0 / (1.0 + params.i);

    // Default if not provided
    let m = params.m.unwrap_or(1) as f64; // Default to Annual
    let t = params.t.unwrap_or(0) as f64; // Default to 0 (no deferral)
    let moment = params.moment.unwrap_or(1) as f64; // Default moment is 1 (mean)

    // As provided - no default
    let mt = &params.mt;
    let entry_age = params.entry_age;

    // Intialize
    let mut summation = 0.0;

    // Main loop to calculate the sum
    for k in 0..(n * m) as u32 {
        let k_f64 = k as f64;

        // Discount factor for the k-th moment v⁽ᵏ⁺¹⁾/ᵐ
        let discount_factor_with_moment = v.powf(moment * (k_f64 + 1.0) / m);
        // Probability of death ₖ/ₘ|₁/ₘqₓ₊ₜ
        let probability = tqx(mt, x + t, 1.0 / m, k_f64 / m, entry_age)?;
        // Aggregate the result
        summation += discount_factor_with_moment * probability;
    }

    // Calculation of ₜEₓ - only term n = t. The rest are intact
    let mut ext_param = params.clone();
    ext_param.n = Some(t as u32);

    let Ext = Exn(&ext_param)?;

    // Final result
    let result = summation * Ext;
    Ok(result)
}

/// Immediate Endowment insurance:
/// ₜ|Aₓ:ₙ̅⁽ᵐ⁾ = ₜ|A¹ₓ:ₙ̅⁽ᵐ⁾ + ₜ|ₙEₓ
///
/// $1 paid at death (if within n years) OR at survival to n years.
pub fn Axn(params: &ParamConfig) -> PolarsResult<f64> {
    let term = Ax1n(params)?;
    let pure_endowment = Exn(params)?;
    let result = term + pure_endowment;
    Ok(result)
}

//-----------------Increasing------------------

/// Immediate increasing whole life:
/// ₜ|(IA)ₓ⁽ᵐ⁾ = ₜEₓ · Σₖ₌₀^∞ (v⁽ᵏ⁺¹⁾/ᵐ . ₖ/ₘ|₁/ₘqₓ₊ₜ . [(k // m) + 1] / m)
/// Eg: for m=12, k=0, 1, ..., 11 the death benefit is 1/m, while k= 12, 13, ..., 23 the death benefit is 2/m, etc.
/// Death benefit increases by 1 each year: k paid if death in k-th year.
pub fn IAx(params: &ParamConfig) -> PolarsResult<f64> {
    // Vaidate the parameters
    params.validate_all()?;

    // Required paramenters: i,x
    let x = params.x as f64;
    let v = 1.0 / (1.0 + params.i);

    // Default if not provided
    let m = params.m.unwrap_or(1) as f64; // Default to Annual
    let t = params.t.unwrap_or(0) as f64; // Default to 0 (no deferral)
    let moment = params.moment.unwrap_or(1) as f64; // Default moment is 1 (mean)

    // As provided - no default
    let mt = &params.mt;
    let entry_age = params.entry_age;

    // Others
    let max_age = mt.max_age() as f64;

    // Intialize
    let mut summation = 0.0;
    let mut k = 0.0;

    // Main loop to calculate the sum
    loop {
        // Break if x + t + k/m >=  max_age then break the loop
        if (x + t + k / m) >= max_age {
            break;
        }

        // Discount factor for the k-th moment v⁽ᵏ⁺¹⁾/ᵐ
        let discount_factor_with_moment = v.powf(moment * (k + 1.0) / m);
        // Probability of death ₖ/ₘ|₁/ₘqₓ₊ₜ
        let probability = tqx(mt, (x + t) as f64, 1.0 / m, k / m, entry_age)?;
        // Amount of benefit [(k // m) + 1] / m
        let benefit_amount = ((k / m).floor() + 1.0) / m;
        // Aggregate the result
        summation += discount_factor_with_moment * probability * benefit_amount;

        // Exnt k
        k += 1.0;
    }

    // Calculation of ₜEₓ - only term n = t. The rest are intact
    let mut ext_param = params.clone();
    ext_param.n = Some(t as u32);

    let Ext = Exn(&ext_param)?;

    // Final result
    let result = summation * Ext;
    Ok(result)
}

/// Immediate increasing term:
/// ₜ|(IA)¹ₓ:ₙ̅⁽ᵐ⁾ = ₜEₓ ·  Σₖ₌₀^{mn-1} (v⁽ᵏ⁺¹⁾/ᵐ . ₖ/ₘ|₁/ₘqₓ₊ₜ . [(k // m) + 1] / m)
/// Eg: for m=12, k=0, 1, ..., 11 the death benefit is 1/m, while k= 12, 13, ..., 23 the death benefit is 2/m, etc.
/// Death benefit increases by 1 each year, pays only if death within n years.
pub fn IAx1n(params: &ParamConfig) -> PolarsResult<f64> {
    // Vaidate the parameters
    params.validate_all()?;

    // Required paramenters: i,x,n
    if params.n.is_none() {
        return Err(PolarsError::ComputeError(
            "n must be provided for IAx1n".into(),
        ));
    }

    let n = params.n.unwrap() as f64;
    let x = params.x as f64;
    let v = 1.0 / (1.0 + params.i);

    // Default if not provided
    let m = params.m.unwrap_or(1) as f64; // Default to Annual
    let t = params.t.unwrap_or(0) as f64; // Default to 0 (no deferral)
    let moment = params.moment.unwrap_or(1) as f64; // Default moment is 1 (mean)

    // As provided - no default
    let mt = &params.mt;
    let entry_age = params.entry_age;

    // Intialize
    let mut summation = 0.0;

    // Main loop to calculate the sum
    for k in 0..(n * m) as u32 {
        let k_f64 = k as f64;

        // Discount factor for the k-th moment v⁽ᵏ⁺¹⁾/ᵐ
        let discount_factor_with_moment = v.powf(moment * (k_f64 + 1.0) / m);
        // Probability of death ₖ/ₘ|₁/ₘqₓ₊ₜ
        let probability = tqx(mt, x + t, 1.0 / m, k_f64 / m, entry_age)?;
        // Amount of benefit [(k // m) + 1] / m
        let benefit_amount = ((k_f64 / m).floor() + 1.0) / m;
        // Aggregate the result
        summation += discount_factor_with_moment * probability * benefit_amount;
    }

    // Calculation of ₜEₓ - only term n = t. The rest are intact
    let mut ext_param = params.clone();
    ext_param.n = Some(t as u32);

    let Ext = Exn(&ext_param)?;

    // Final result
    let result = summation * Ext;
    Ok(result)
}

/// Immediate endowment insurance:
/// ₜ|IAₓ:ₙ̅ = ₜ|IA¹ₓ:ₙ̅ + ₜ|IA¹ₓ:ₙ̅
/// $1 paid at death (if within n years) OR at survival to n years.
pub fn IAxn(params: &ParamConfig) -> PolarsResult<f64> {
    // Decide if selected table is used
    let term = IAx1n(params)?;

    let n = params.n.unwrap() as f64; // n is already validated and required in IAx1n
    let pure_endowment = n * Exn(params)?;

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
pub fn DAx1n(params: &ParamConfig) -> PolarsResult<f64> {
    // Vaidate the parameters
    params.validate_all()?;

    // Required paramenters: i,x,n
    if params.n.is_none() {
        return Err(PolarsError::ComputeError(
            "n must be provided for IAx1n".into(),
        ));
    }

    let n = params.n.unwrap() as f64;
    let x = params.x as f64;
    let v = 1.0 / (1.0 + params.i);

    // Default if not provided
    let m = params.m.unwrap_or(1) as f64; // Default to Annual
    let t = params.t.unwrap_or(0) as f64; // Default to 0 (no deferral)
    let moment = params.moment.unwrap_or(1) as f64; // Default moment is 1 (mean)

    // As provided - no default
    let mt = &params.mt;
    let entry_age = params.entry_age;

    // Intialize
    let mut summation = 0.0;

    // Main loop to calculate the sum
    for k in 0..(n * m) as u32 {
        let k_f64 = k as f64;

        // Discount factor for the k-th moment v⁽ᵏ⁺¹⁾/ᵐ
        let discount_factor_with_moment = v.powf(moment * (k_f64 + 1.0) / m);
        // Probability of death ₖ/ₘ|₁/ₘqₓ₊ₜ
        let probability = tqx(mt, x + t, 1.0 / m, k_f64 / m, entry_age)?;
        // Amount of benefit [n-(k // m)] / m
        let benefit_amount = (n - (k_f64 / m).floor()) / m;
        // Aggregate the result
        summation += discount_factor_with_moment * probability * benefit_amount;
    }

    // Calculation of ₜEₓ - only term n = t. The rest are intact
    let mut ext_param = params.clone();
    ext_param.n = Some(t as u32);

    let Ext = Exn(&ext_param)?;

    // Final result
    let result = summation * Ext;
    Ok(result)
}

/// Immediate decreasing endowment insurance:
/// ₜ|(DA)ₓ:ₙ̅ = ₜ|(DA)¹ₓ:ₙ̅ + ₜ|ₙEₓ
/// $1 paid at death (if within n years) OR at survival to n years.
pub fn DAxn(params: &ParamConfig) -> PolarsResult<f64> {
    let term = DAx1n(params)?;
    let pure_endowment = Exn(params)?;
    let result = term + pure_endowment;
    Ok(result)
}

//-----------------Geometric increasing------------------

/// Immediate geometric whole life:
/// Aₓ⁽ᵍ⁾ with adjusted interest rate i′ = (1+i)/(1+g) - 1
///
/// Death benefit grows geometrically at rate g each year.
pub fn gAx(params: &ParamConfig, g: f64) -> PolarsResult<f64> {
    // Replace the effective interest rate with the adjusted one
    let new_int_rate = (1.0 + params.i) / (1.0 + g) - 1.0;
    let mut new_params = params.clone();
    new_params.i = new_int_rate;

    let result = Ax(&new_params)?;
    Ok(result)
}

/// Immediate geometric n-year term:
/// A¹ₓ:ₙ̅⁽ᵍ⁾ with adjusted interest rate i′ = (1+i)/(1+g) - 1
///
/// Death benefit grows geometrically at rate g for n years.
pub fn gAx1n(params: &ParamConfig, g: f64) -> PolarsResult<f64> {
    // Replace the effective interest rate with the adjusted one
    let new_int_rate = (1.0 + params.i) / (1.0 + g) - 1.0;
    let mut new_params = params.clone();
    new_params.i = new_int_rate;

    let result = Ax1n(&new_params)?;
    Ok(result)
}

/// Immediate geometric n-year pure endowment:
/// Aₓ:ₙ̅¹⁽ᵍ⁾ with adjusted interest rate i′ = (1+i)/(1+g) - 1
///
/// Death benefit grows geometrically at rate g for n years.
pub fn gExn(params: &ParamConfig, g: f64) -> PolarsResult<f64> {
    // Replace the effective interest rate with the adjusted one
    let new_int_rate = (1.0 + params.i) / (1.0 + g) - 1.0;
    let mut new_params = params.clone();
    new_params.i = new_int_rate;

    let result = Exn(&new_params)?;
    Ok(result)
}

/// Immediate geometric n-year endowment:
/// Aₓ:ₙ̅⁽ᵍ⁾ = A¹ₓ:ₙ̅⁽ᵍ⁾ + Aₓ:ₙ̅¹⁽ᵍ⁾
///
/// Death benefit grows geometrically at rate g for n years.
pub fn gAxn(params: &ParamConfig, g: f64) -> PolarsResult<f64> {
    let term = gAx1n(params, g)?;
    let pure_endowment = gExn(params, g)?;
    let result = term + pure_endowment;
    Ok(result)
}

// ================================================
// UNIT TESTS
// ================================================
#[cfg(test)]
mod tests {
    use super::*;
    use crate::mt_config::MortTableConfig;
    use crate::xml::MortXML;
    use approx::assert_abs_diff_eq;

    #[test]
    fn test_fn_A1xn_n_is_0() {
        // An edge case where n is 0 fror Ax1n
        // Load AM92 selected table
        let am92_xml = MortXML::from_xlsx("data/am92.xlsx", "am92")
            .expect("Failed to load AM92 selected table");

        // Create MortTableConfig
        let mt = MortTableConfig::builder().xml(am92_xml).build();

        // Create ParamConfig
        let params = ParamConfig::builder()
            .mt(mt)
            .i(0.05)
            .x(70)
            .n(0)
            .entry_age(70)
            .build();

        // Calculate  A₍₇₀₎:₀
        let ans = Ax1n(&params).unwrap();
        let expected = 0.0;
        assert_abs_diff_eq!(ans, expected, epsilon = 1e-6);
    }

    #[test]
    fn test_fn_Exn_n_is_0() {
        // An edge case where n is 0 fror Exn
        // Load AM92 selected table
        let am92_xml = MortXML::from_xlsx("data/am92.xlsx", "am92")
            .expect("Failed to load AM92 selected table");

        // Create MortTableConfig
        let mt = MortTableConfig::builder().xml(am92_xml).build();

        // Create ParamConfig
        let params = ParamConfig::builder()
            .mt(mt)
            .i(0.05)
            .x(70)
            .n(0)
            .entry_age(70)
            .build();

        let ans = Exn(&params).unwrap();
        let expected = 1.0;
        assert_abs_diff_eq!(ans, expected, epsilon = 1e-6);
    }

    #[test]
    fn test_fn_Axn_benefit_cm1() {
        // April 2025 CM1 question 1
        // Load AM92 selected table
        let am92_xml = MortXML::from_xlsx("data/am92.xlsx", "am92")
            .expect("Failed to load AM92 selected table");

        // Create MortTableConfig
        let mt = MortTableConfig::builder().xml(am92_xml).build();

        // Create ParamConfig
        let params = ParamConfig::builder()
            .mt(mt)
            .i(0.05)
            .x(70)
            .n(3)
            .entry_age(70)
            .build();

        // Calculate  A₍₇₀₎:₃
        let ans = Axn(&params).unwrap();
        let expected = 0.8663440;
        assert_abs_diff_eq!(ans, expected, epsilon = 1e-6);
    }

    #[test]
    fn test_fn_IAx1n_benefit_cm1() {
        // This is obtain from CM1 study package 2019 Chapter 19 The Life Table
        // Load AM92 selected table
        let am92_xml = MortXML::from_xlsx("data/am92.xlsx", "am92")
            .expect("Failed to load AM92 selected table");

        // Create MortTableConfig
        let mt = MortTableConfig::builder().xml(am92_xml).build();

        // Create ParamConfig
        let params = ParamConfig::builder().mt(mt).i(0.04).x(50).n(10).build();

        let ans = IAx1n(&params).unwrap();
        let expected = 8.55929 - (882.85 / 1366.61) * (8.36234 + 10.0 * 0.45640);
        // Since the solution using commutation Dx rounded to 2 decimal places.
        // We should use a precision of 1e-4 instead of 1e-6
        assert_abs_diff_eq!(ans, expected, epsilon = 1e-4);
    }
}
