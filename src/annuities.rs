#![allow(non_snake_case)]
use crate::benefits::Exn;
use crate::param_config::ParamConfig;
use crate::survivals::tpx;
use polars::prelude::*;

//-----------------Basic------------------
/// Life annuity-due payable m times per year:
/// ₜ|äₓ⁽ᵐ⁾ = ₜEₓ · Σₖ₌₀^∞ (1/m . vᵏ/ᵐ . ₖ/ₘpₓ₊ₜ)
/// Present value of 1/m paid m times per year for life.
pub fn aax(params: &ParamConfig) -> PolarsResult<f64> {
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

        // Discount factor for the k-th moment
        let discount_factor_with_moment = v.powf(moment * k / m); //
        // Probability of death  ₖ/ₘpₓ₊ₜ
        let probability = tpx(mt, x + t, k / m, 0.0, entry_age)?;
        // Annuity payment amount 1/m
        let benefit_amount = 1.0 / m;
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

/// Due temporary annuity-due payable m times per year:
/// ₜ|äₓ:ₙ̅⁽ᵐ⁾ = ₜEₓ · Σₖ₌₀^∞ (1/m . vᵏ/ᵐ . ₖ/ₘpₓ₊ₜ)
/// Present value of 1/m paid m times per year for up to n years.
pub fn aaxn(params: &ParamConfig) -> PolarsResult<f64> {
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

        // Discount factor for the k-th moment
        let discount_factor_with_moment = v.powf(moment * k_f64 / m); // vᵏ/ᵐ
        // Probability of death
        let probability = tpx(mt, x + t, k_f64 / m, 0.0, entry_age)?; // ₖ/ₘpₓ₊ₜ
        // Annuity payment amount
        let benefit_amount = 1.0 / m; // 1/m
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

//-----------------Increasing------------------
/// Increasing life annuity-due payable m times per year:
/// ₜ|(Iä)ₓ⁽ᵐ⁾ = ₜEₓ · Σₖ₌₀^∞ ([(k // m) + 1] / m . vᵏ/ᵐ . ₖ/ₘpₓ₊ₜ)
/// Eg: for m=12, k=0, 1, ..., 11 annuity is 1/m, while k= 12, 13, ..., 23 the annuity is 2/m, etc.
/// Present value of an increasing life annuity-due: payments of 1/m made m times per year for life, with each annual payment increasing by 1.
pub fn Iaax(params: &ParamConfig) -> PolarsResult<f64> {
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

        // Discount factor for the k-th moment vᵏ/ᵐ
        let discount_factor_with_moment = v.powf(moment * k / m);
        // Probability of death ₖ/ₘpₓ₊ₜ
        let probability = tpx(mt, (x + t) as f64, k / m, 0.0, entry_age)?;
        // Annuity payment amount [(k // m) + 1] / m
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

/// Due increasing temporary life annuity-due payable m times per year:
/// ₜ|(Iä)ₓ⁽ᵐ⁾ = ₜEₓ · Σₖ₌₀^{mn-1} ([(k // m) + 1] / m . vᵏ/ᵐ . ₖ/ₘpₓ₊ₜ)
/// Eg: for m=12, k=0, 1, ..., 11 annuity is 1/m, while k= 12, 13, ..., 23 the annuity is 2/m, etc.
/// Present value of an increasing life annuity-due: payments of 1/m made m times per year for n years, with each annual payment increasing by 1.
pub fn Iaaxn(params: &ParamConfig) -> PolarsResult<f64> {
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

        // Discount factor for the k-th moment vᵏ/ᵐ
        let discount_factor_with_moment = v.powf(moment * k_f64 / m);
        // Probability of death ₖ/ₘpₓ₊ₜ
        let probability = tpx(mt, (x + t) as f64, k_f64 / m, 0.0, entry_age)?;
        // Annuity payment amount [(k // m) + 1] / m
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

//-----------------Decreasing------------------
/// Decreasing temporary life annuity-due:
/// ₜ|(Iä)ₓ⁽ᵐ⁾ = ₜEₓ · Σₖ₌₀^{mn-1} ([n-(k // m)] / m. vᵏ/ᵐ . ₖ/ₘpₓ₊ₜ)
/// Eg: for m=12, k=0, 1, ..., 11 annuity is 1/m, while k= 12, 13, ..., 23 the annuity is 2/m, etc.
/// Present value of an decreasing life annuity-due: payments of n/m made m times per year for n years, with each annual payment decreasing by 1.
pub fn Daaxn(params: &ParamConfig) -> PolarsResult<f64> {
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

        // Discount factor for the k-th moment vᵏ/ᵐ
        let discount_factor_with_moment = v.powf(moment * k_f64 / m);
        // Probability of death ₖ/ₘpₓ₊ₜ
        let probability = tpx(mt, (x + t) as f64, k_f64 / m, 0.0, entry_age)?;
        // Annuity payment amount [n-(k // m) + 1] / m
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

//-----------------Geometric increasing------------------
/// Geometric  increasing life annuity-due:
/// äₓ⁽ᵍ⁾ with adjusted interest rate i′ = (1+i)/(1+g) - 1
///
/// Payments of 1/m are made m times per year for life, starting immediately,
/// with each annual payment increasing by a factor of (1+g) each year (i.e., geometric progression).
pub fn gaax(params: &ParamConfig, g: f64) -> PolarsResult<f64> {
    // Replace the effective interest rate with the adjusted one
    let new_int_rate = (1.0 + params.i) / (1.0 + g) - 1.0;
    let mut new_params = params.clone();
    new_params.i = new_int_rate;

    let result = aax(&new_params)?;
    Ok(result)
}

/// Geometric increasing temporary annuity-due:
/// äₓ:ₙ̅⁽ᵍ⁾ with adjusted interest rate i′ = (1+i)/(1+g) - 1
///
/// Payments of 1/m are made m times per year for n years, starting immediately,
/// with each annual payment increasing by a factor of (1+g) each year (i.e., geometric progression).
pub fn gaaxn(params: &ParamConfig, g: f64) -> PolarsResult<f64> {
    // Replace the effective interest rate with the adjusted one
    let new_int_rate = (1.0 + params.i) / (1.0 + g) - 1.0;
    let mut new_params = params.clone();
    new_params.i = new_int_rate;

    let result = aaxn(&new_params)?;
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
    fn test_fn_Iax_annuities_cm1() {
        // April 2025 CM1 question 1
        #[allow(non_snake_case)]
        // Load AM92 selected table
        let am92_xml = MortXML::from_xlsx("data/am92.xlsx", "am92")
            .expect("Failed to load AM92 selected table");

        // Create MortTableConfig
        let mt = MortTableConfig::builder().xml(am92_xml).build();

        // Create ParamConfig
        let params = ParamConfig::builder().mt(mt).i(0.04).x(50).build();

        // Calculate  A₍₇₀₎:₃
        let ans = Iaax(&params).unwrap();
        let expected = 231.007;
        // Lower down the precision to 4 decimal places since the expected value is rounded
        assert_abs_diff_eq!(ans, expected, epsilon = 1e-4);
    }
}
