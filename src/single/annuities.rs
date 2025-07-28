#![allow(non_snake_case)]
use crate::helpers::get_new_config_with_selected_table;
use crate::mt_config::MortTableConfig;
use crate::params::SingleLifeParams;
use crate::single::benefits::Exn;
use crate::single::survivals::tpx;
use bon::builder;

//-----------------Basic------------------
/// Life annuity-due payable m times per year:
/// ₜ|äₓ⁽ᵐ⁾ = ₜEₓ · Σₖ₌₀^∞ (1/m . vᵏ/ᵐ . ₖ/ₘpₓ₊ₜ)
/// Present value of 1/m paid m times per year for life.
#[builder]
pub fn aax(
    mt: &MortTableConfig,
    i: f64,
    x: u32,
    t: Option<u32>,
    m: Option<u32>,
    moment: Option<u32>,
    entry_age: Option<u32>,
) -> Result<f64, Box<dyn std::error::Error>> {
    // Vaidate the parameters
    let params = SingleLifeParams {
        mt: mt.clone(),
        i,
        x,
        n: None, // Not used
        t,
        m,
        moment,
        entry_age,
    };

    params
        .validate_all()
        .map_err(|err| Box::new(err) as Box<dyn std::error::Error>)?;

    // Unwrap to obtain default values
    let t = t.unwrap_or(0) as f64; // Default to 0 (no deferral)
    let m = m.unwrap_or(1) as f64; // Default to Annual
    let moment = moment.unwrap_or(1) as f64; // Default moment is 1 (mean)

    // As provided - no default
    let x = x as f64;
    let v = 1.0 / (1.0 + i);

    // Decide if selected table is used
    let mt = get_new_config_with_selected_table(mt, entry_age)?;

    // Others
    let max_age = mt.max_age() as f64;

    // Calculation of ₜEₓ - only term n = t. The rest are intact
    let Ext = Exn().mt(&mt).i(i).x(x as u32).n(t as u32).call()?;

    // Intialize
    let mut summation = 0.0;
    let mut k = 0.0;

    // Main loop to calculate the sum
    loop {
        // Discount factor for the k-th moment
        let discount_factor_with_moment = v.powf(moment * k / m); //
        // Probability of survival  ₖ/ₘpₓ₊ₜ
        let probability = tpx().mt(&mt).x(x + t).t(k / m).call()?;
        // Annuity payment amount 1/m
        let benefit_amount = 1.0 / m;
        // Aggregate the result
        summation += discount_factor_with_moment * probability * benefit_amount;

        // Exnt k
        k += 1.0;

        // Break if x + t + k/m >  max_age then break the loop
        if (x + t + k / m) > max_age {
            break;
        }
    }

    // Final result
    let result = summation * Ext;
    Ok(result)
}

/// Due temporary annuity-due payable m times per year:
/// ₜ|äₓ:ₙ̅⁽ᵐ⁾ = ₜEₓ · Σₖ₌₀^∞ (1/m . vᵏ/ᵐ . ₖ/ₘpₓ₊ₜ)
/// Present value of 1/m paid m times per year for up to n years.
#[builder]
pub fn aaxn(
    mt: &MortTableConfig,
    i: f64,
    x: u32,
    n: u32,
    t: Option<u32>,
    m: Option<u32>,
    moment: Option<u32>,
    entry_age: Option<u32>,
) -> Result<f64, Box<dyn std::error::Error>> {
    // Vaidate the parameters
    let params = SingleLifeParams {
        mt: mt.clone(),
        i,
        x,
        n: Some(n), // Required for Ax1n but Optional in struct
        t,
        m,
        moment,
        entry_age,
    };

    params
        .validate_all()
        .map_err(|err| Box::new(err) as Box<dyn std::error::Error>)?;

    // Unwrap to obtain default values
    let t = t.unwrap_or(0) as f64; // Default to 0 (no deferral)
    let m = m.unwrap_or(1) as f64; // Default to Annual
    let moment = moment.unwrap_or(1) as f64; // Default moment is 1 (mean)

    // As provided - no default
    let x = x as f64;
    let n = n as f64;
    let v = 1.0 / (1.0 + i);

    // Decide if selected table is used
    let mt = get_new_config_with_selected_table(mt, entry_age)?;

    // Calculation of ₜEₓ - only term n = t. The rest are intact
    let Ext = Exn().mt(&mt).i(i).x(x as u32).n(t as u32).call()?;

    // Intialize
    let mut summation = 0.0;

    // Main loop to calculate the sum
    for k in 0..(n * m) as u32 {
        let k_f64 = k as f64;

        // Discount factor for the k-th moment  vᵏ/ᵐ
        let discount_factor_with_moment = v.powf(moment * k_f64 / m);
        // Probability of survival ₖ/ₘpₓ₊ₜ
        let probability = tpx().mt(&mt).x(x + t).t(k_f64 / m).call()?;
        // Annuity payment amount  1/m
        let benefit_amount = 1.0 / m;
        // Aggregate the result
        summation += discount_factor_with_moment * probability * benefit_amount;
    }

    // Final result
    let result = summation * Ext;
    Ok(result)
}

//-----------------Increasing------------------
/// Increasing life annuity-due payable m times per year:
/// ₜ|(Iä)ₓ⁽ᵐ⁾ = ₜEₓ · Σₖ₌₀^∞ ([(k // m) + 1] / m . vᵏ/ᵐ . ₖ/ₘpₓ₊ₜ)
/// Eg: for m=12, k=0, 1, ..., 11 annuity is 1/m, while k= 12, 13, ..., 23 the annuity is 2/m, etc.
/// Present value of an increasing life annuity-due: payments of 1/m made m times per year for life, with each annual payment increasing by 1.
#[builder]
pub fn Iaax(
    mt: &MortTableConfig,
    i: f64,
    x: u32,
    t: Option<u32>,
    m: Option<u32>,
    moment: Option<u32>,
    entry_age: Option<u32>,
) -> Result<f64, Box<dyn std::error::Error>> {
    // Vaidate the parameters
    let params = SingleLifeParams {
        mt: mt.clone(),
        i,
        x,
        n: None, // Not used
        t,
        m,
        moment,
        entry_age,
    };

    params
        .validate_all()
        .map_err(|err| Box::new(err) as Box<dyn std::error::Error>)?;

    // Unwrap to obtain default values
    let t = t.unwrap_or(0) as f64; // Default to 0 (no deferral)
    let m = m.unwrap_or(1) as f64; // Default to Annual
    let moment = moment.unwrap_or(1) as f64; // Default moment is 1 (mean)

    // As provided - no default
    let x = x as f64;
    let v = 1.0 / (1.0 + i);

    // Decide if selected table is used
    let mt = get_new_config_with_selected_table(mt, entry_age)?;

    // Others
    let max_age = mt.max_age() as f64;

    // Calculation of ₜEₓ - only term n = t. The rest are intact
    let Ext = Exn().mt(&mt).i(i).x(x as u32).n(t as u32).call()?;

    // Intialize
    let mut summation = 0.0;
    let mut k = 0.0;
    // Main loop to calculate the sum
    loop {
        // Break if current k would exceed max_age
        if (x + t + k / m) > max_age {
            break;
        }

        // Discount factor for the k-th moment vᵏ/ᵐ
        let discount_factor_with_moment = v.powf(moment * k / m);
        // Probability of survival ₖ/ₘpₓ₊ₜ
        let probability = tpx().mt(&mt).x(x + t).t(k / m).call()?;
        // Annuity payment amount [(k // m) + 1] / m
        let benefit_amount = ((k / m).floor() + 1.0) / m;
        // Aggregate the result
        summation += discount_factor_with_moment * probability * benefit_amount;

        // Increment k
        k += 1.0;
    }

    // Final result
    let result = summation * Ext;
    Ok(result)
}

/// Due increasing temporary life annuity-due payable m times per year:
/// ₜ|(Iä)ₓ⁽ᵐ⁾ = ₜEₓ · Σₖ₌₀^{mn-1} ([(k // m) + 1] / m . vᵏ/ᵐ . ₖ/ₘpₓ₊ₜ)
/// Eg: for m=12, k=0, 1, ..., 11 annuity is 1/m, while k= 12, 13, ..., 23 the annuity is 2/m, etc.
/// Present value of an increasing life annuity-due: payments of 1/m made m times per year for n years, with each annual payment increasing by 1.
#[builder]
pub fn Iaaxn(
    mt: &MortTableConfig,
    i: f64,
    x: u32,
    n: u32,
    t: Option<u32>,
    m: Option<u32>,
    moment: Option<u32>,
    entry_age: Option<u32>,
) -> Result<f64, Box<dyn std::error::Error>> {
    // Vaidate the parameters
    let params = SingleLifeParams {
        mt: mt.clone(),
        i,
        x,
        n: Some(n), // Required for Ax1n but Optional in struct
        t,
        m,
        moment,
        entry_age,
    };

    params
        .validate_all()
        .map_err(|err| Box::new(err) as Box<dyn std::error::Error>)?;

    // Unwrap to obtain default values
    let t = t.unwrap_or(0) as f64; // Default to 0 (no deferral)
    let m = m.unwrap_or(1) as f64; // Default to Annual
    let moment = moment.unwrap_or(1) as f64; // Default moment is 1 (mean)

    // As provided - no default
    let x = x as f64;
    let n = n as f64;
    let v = 1.0 / (1.0 + i);

    // Decide if selected table is used
    let mt = get_new_config_with_selected_table(mt, entry_age)?;

    // Calculation of ₜEₓ - only term n = t. The rest are intact
    let Ext = Exn().mt(&mt).i(i).x(x as u32).n(t as u32).call()?;

    // Intialize
    let mut summation = 0.0;

    // Main loop to calculate the sum
    for k in 0..(n * m) as u32 {
        let k_f64 = k as f64;

        // Discount factor for the k-th moment vᵏ/ᵐ
        let discount_factor_with_moment = v.powf(moment * k_f64 / m);
        // Probability of death ₖ/ₘpₓ₊ₜ
        let probability = tpx().mt(&mt).x(x + t).t(k_f64 / m).call()?;
        // Annuity payment amount [(k // m) + 1] / m
        let benefit_amount = ((k_f64 / m).floor() + 1.0) / m;
        // Aggregate the result
        summation += discount_factor_with_moment * probability * benefit_amount;
    }

    // Final result
    let result = summation * Ext;
    Ok(result)
}

//-----------------Decreasing------------------
/// Decreasing temporary life annuity-due:
/// ₜ|(Iä)ₓ⁽ᵐ⁾ = ₜEₓ · Σₖ₌₀^{mn-1} ([n-(k // m)] / m. vᵏ/ᵐ . ₖ/ₘpₓ₊ₜ)
/// Eg: for m=12, k=0, 1, ..., 11 annuity is 1/m, while k= 12, 13, ..., 23 the annuity is 2/m, etc.
/// Present value of an decreasing life annuity-due: payments of n/m made m times per year for n years, with each annual payment decreasing by 1.
#[builder]
pub fn Daaxn(
    mt: &MortTableConfig,
    i: f64,
    x: u32,
    n: u32,
    t: Option<u32>,
    m: Option<u32>,
    moment: Option<u32>,
    entry_age: Option<u32>,
) -> Result<f64, Box<dyn std::error::Error>> {
    // Vaidate the parameters
    let params = SingleLifeParams {
        mt: mt.clone(),
        i,
        x,
        n: Some(n), // Required for Ax1n but Optional in struct
        t,
        m,
        moment,
        entry_age,
    };

    params
        .validate_all()
        .map_err(|err| Box::new(err) as Box<dyn std::error::Error>)?;

    // Unwrap to obtain default values
    let t = t.unwrap_or(0) as f64; // Default to 0 (no deferral)
    let m = m.unwrap_or(1) as f64; // Default to Annual
    let moment = moment.unwrap_or(1) as f64; // Default moment is 1 (mean)

    // As provided - no default
    let x = x as f64;
    let n = n as f64;
    let v = 1.0 / (1.0 + i);

    // Decide if selected table is used
    let mt = get_new_config_with_selected_table(mt, entry_age)?;

    // Calculation of ₜEₓ - only term n = t. The rest are intact
    let Ext = Exn().mt(&mt).i(i).x(x as u32).n(t as u32).call()?;

    // Intialize
    let mut summation = 0.0;

    // Main loop to calculate the sum
    for k in 0..(n * m) as u32 {
        let k_f64 = k as f64;

        // Discount factor for the k-th moment vᵏ/ᵐ
        let discount_factor_with_moment = v.powf(moment * k_f64 / m);
        // Probability of death ₖ/ₘpₓ₊ₜ
        let probability = tpx().mt(&mt).x(x + t).t(k_f64 / m).call()?;
        // Annuity payment amount [n-(k // m) + 1] / m
        let benefit_amount = (n - (k_f64 / m).floor()) / m;
        // Aggregate the result
        summation += discount_factor_with_moment * probability * benefit_amount;
    }

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
#[builder]
pub fn gaax(
    mt: &MortTableConfig,
    i: f64,
    x: u32,
    t: Option<u32>,
    m: Option<u32>,
    moment: Option<u32>,
    entry_age: Option<u32>,
    g: f64,
) -> Result<f64, Box<dyn std::error::Error>> {
    // Vaidate the parameters
    let params = SingleLifeParams {
        mt: mt.clone(),
        i,
        x,
        n: None, // Not used
        t,
        m,
        moment,
        entry_age,
    };

    params
        .validate_all()
        .map_err(|err| Box::new(err) as Box<dyn std::error::Error>)?;

    // Unwrap to obtain default values
    let t = t.unwrap_or(0); // Default to 0 (no deferral)
    let m = m.unwrap_or(1); // Default to Annual
    let moment = moment.unwrap_or(1); // Default moment is 1 (mean)

    // Decide if selected table is used
    let mt = get_new_config_with_selected_table(mt, entry_age)?;

    // Replace the effective interest rate with the adjusted one
    let new_i = (1.0 + i) / (1.0 + g) - 1.0;

    let result = aax()
        .mt(&mt)
        .i(new_i)
        .x(x)
        .t(t)
        .m(m)
        .moment(moment)
        .call()?;
    Ok(result)
}

/// Geometric increasing temporary annuity-due:
/// äₓ:ₙ̅⁽ᵍ⁾ with adjusted interest rate i′ = (1+i)/(1+g) - 1
///
/// Payments of 1/m are made m times per year for n years, starting immediately,
/// with each annual payment increasing by a factor of (1+g) each year (i.e., geometric progression).
#[builder]
pub fn gaaxn(
    mt: &MortTableConfig,
    i: f64,
    x: u32,
    n: u32,
    t: Option<u32>,
    m: Option<u32>,
    moment: Option<u32>,
    entry_age: Option<u32>,
    g: f64,
) -> Result<f64, Box<dyn std::error::Error>> {
    // Vaidate the parameters
    let params = SingleLifeParams {
        mt: mt.clone(),
        i,
        x,
        n: Some(n), // Required for Ax1n but Optional in struct
        t,
        m,
        moment,
        entry_age,
    };

    params
        .validate_all()
        .map_err(|err| Box::new(err) as Box<dyn std::error::Error>)?;

    // Unwrap to obtain default values
    let t = t.unwrap_or(0); // Default to 0 (no deferral)
    let m = m.unwrap_or(1); // Default to Annual
    let moment = moment.unwrap_or(1); // Default moment is 1 (mean)

    // Decide if selected table is used
    let mt = get_new_config_with_selected_table(mt, entry_age)?;

    // Replace the effective interest rate with the adjusted one
    let new_i = (1.0 + i) / (1.0 + g) - 1.0;

    let result = aaxn()
        .mt(&mt)
        .i(new_i)
        .x(x)
        .n(n)
        .t(t)
        .m(m)
        .moment(moment)
        .call()?;

    Ok(result)
}

// ================================================
// UNIT TESTS
// ================================================
#[cfg(test)]
mod tests {
    use super::*;
    use crate::mt_config::MortTableConfig;
    use crate::mt_data::MortData;
    use approx::assert_abs_diff_eq;

    #[test]
    fn test_fn_Iax_annuities_cm1() {
        // April 2025 CM1 question 1
        // Load AM92 selected table
        let am92 = MortData::from_xlsx("data/am92.xlsx", "am92")
            .expect("Failed to load AM92 selected table");

        // Create MortTableConfig
        let mt = MortTableConfig::builder().data(am92).build();

        // Calculate increasing life annuity (Ia)x for age 50
        // Note: No entry_age needed for whole life increasing annuity
        let ans = Iaax().mt(&mt).i(0.04).x(50).call().unwrap();

        let expected = 231.007;
        // Lower down the precision to 4 decimal places since the expected value is rounded
        assert_abs_diff_eq!(ans, expected, epsilon = 1e-4);
    }
}
