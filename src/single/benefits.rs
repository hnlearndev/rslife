#![allow(non_snake_case)]
use crate::helpers::get_new_config_with_selected_table;
use crate::mt_config::MortTableConfig;
use crate::params::SingleLifeParams;
use crate::single::survivals::{tpx, tqx};
use bon::builder;

// =======================================
// PUBLIC FUNCTIONS
// =======================================

//-----------------Basic------------------

/// Immediate pure endowment:
/// ₜ|ₙEₓ = ₜ+ₙEₓ = vⁿ⁺ᵗ . ₜ+ₙpₓ ✅
///
/// Present value of $1 paid if and only if the insured survives n years.
#[builder]
pub fn Exn(
    mt: &MortTableConfig,
    i: f64,
    x: u32,
    n: u32,
    t: Option<u32>,
    moment: Option<u32>,
    entry_age: Option<u32>,
) -> Result<f64, Box<dyn std::error::Error>> {
    // Vaidate the parameters
    let params = SingleLifeParams {
        mt: mt.clone(),
        i,
        x,
        n: Some(n), // Required for Exn but Optional in struct,
        t,
        m: None, // Not used since pure endowment is paid at the n of the term
        moment,
        entry_age,
    };

    params
        .validate_all()
        .map_err(|err| Box::new(err) as Box<dyn std::error::Error>)?;

    // Unwrap to obtain default values
    let t = t.unwrap_or(0) as f64; // Default to 0 (no deferral)
    let moment = moment.unwrap_or(1) as f64; // Default moment is 1 (mean)

    // As provided - no default
    let x = x as f64;
    let n = n as f64;
    let v = 1.0 / (1.0 + i);

    // Decide if selected table is used
    let mt = get_new_config_with_selected_table(mt, entry_age)?;

    // vⁿ⁺ᵗ
    let discount_factor = v.powf(moment * (t + n));
    // ₜ+ₙpₓ
    let prob = tpx().mt(&mt).x(x).t(t + n).call()?;
    let result = discount_factor * prob;
    Ok(result)
}

/// Immediate whole life insurance:
/// Present value of $1 paid only if death occurs
/// ₜ|Aₓ⁽ᵐ⁾ = ₜEₓ · Σₖ₌₀^∞ [v⁽ᵏ⁺¹⁾/ᵐ . ₖ/ₘ|₁/ₘqₓ₊ₜ]
/// Note: for higher moment replace v with v^(moment)
#[builder]
pub fn Ax(
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
        // Break if x + t + k/m >  max_age then break the loop
        if (x + t + k / m) > max_age {
            break;
        }

        // Discount factor for the k-th moment v^[(moment·(k+1))/m]
        let discount_factor_with_moment = v.powf(moment * (k + 1.0) / m);
        // Probability of death ₖ/ₘ|₁/ₘqₓ₊ₜ
        let probability = tqx().mt(&mt).x(x + t).t(1.0 / m).k(k / m).call()?;
        // Aggregate the result
        summation += discount_factor_with_moment * probability;

        // Exnt k
        k += 1.0;
    }

    // Final result
    let result = summation * Ext;
    Ok(result)
}

/// Immediate term life insurance:
/// ₜ|A¹ₓ:ₙ̅⁽ᵐ⁾ = ₜpₓ · Σₖ₌₀^{mn-1} [v⁽ᵏ⁺¹⁾/ᵐ · ₖ/ₘ|₁/ₘqₓ₊ₜ]
/// Note: for higher moment replace v with v^(moment)
#[builder]
pub fn Ax1n(
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

        // Discount factor for the k-th moment v⁽ᵏ⁺¹⁾/ᵐ
        let discount_factor_with_moment = v.powf(moment * (k_f64 + 1.0) / m);
        // Probability of death ₖ/ₘ|₁/ₘqₓ₊ₜ
        let probability = tqx().mt(&mt).x(x + t).t(1.0 / m).k(k_f64 / m).call()?;
        // Aggregate the result
        summation += discount_factor_with_moment * probability;
    }

    // Final result
    let result = summation * Ext;
    Ok(result)
}

/// Immediate Endowment insurance:
/// ₜ|Aₓ:ₙ̅⁽ᵐ⁾ = ₜ|A¹ₓ:ₙ̅⁽ᵐ⁾ + ₜ|ₙEₓ
///
/// $1 paid at death (if within n years) OR at survival to n years.
#[builder]
pub fn Axn(
    mt: &MortTableConfig,
    i: f64,
    x: u32,
    n: u32,
    t: Option<u32>,
    m: Option<u32>,
    moment: Option<u32>,
    entry_age: Option<u32>,
) -> Result<f64, Box<dyn std::error::Error>> {
    // Decide if selected table is used
    let mt = get_new_config_with_selected_table(mt, entry_age)?;

    // Unwrap to obtain default values
    let t = t.unwrap_or(0); // Default to 0 (no deferral)
    let m = m.unwrap_or(1); // Default to Annual
    let moment = moment.unwrap_or(1); // Default moment is 1 (mean)

    // Build Ax1n using builder pattern with all parameters
    let term = Ax1n()
        .mt(&mt)
        .i(i)
        .x(x)
        .n(n)
        .t(t)
        .m(m)
        .moment(moment)
        .call()?;

    let pure_endowment = Exn().mt(&mt).i(i).x(x).n(n).t(t).moment(moment).call()?;

    let result = term + pure_endowment;

    Ok(result)
}

//-----------------Increasing------------------

/// Immediate increasing whole life:
/// ₜ|(IA)ₓ⁽ᵐ⁾ = ₜEₓ · Σₖ₌₀^∞ (v⁽ᵏ⁺¹⁾/ᵐ . ₖ/ₘ|₁/ₘqₓ₊ₜ . [(k // m) + 1] / m)
/// Eg: for m=12, k=0, 1, ..., 11 the death benefit is 1/m, while k= 12, 13, ..., 23 the death benefit is 2/m, etc.
/// Death benefit increases by 1 each year: k paid if death in k-th year.
#[builder]
pub fn IAx(
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
        // Break if x + t + k/m >=  max_age then break the loop
        if (x + t + k / m) >= max_age {
            break;
        }

        // Discount factor for the k-th moment v⁽ᵏ⁺¹⁾/ᵐ
        let discount_factor_with_moment = v.powf(moment * (k + 1.0) / m);
        // Probability of death ₖ/ₘ|₁/ₘqₓ₊ₜ
        let probability = tqx().mt(&mt).x(x + t).t(1.0 / m).k(k / m).call()?;
        // Amount of benefit [(k // m) + 1] / m
        let benefit_amount = ((k / m).floor() + 1.0) / m;
        // Aggregate the result
        summation += discount_factor_with_moment * probability * benefit_amount;

        // Exnt k
        k += 1.0;
    }

    // Final result
    let result = summation * Ext;
    Ok(result)
}

/// Immediate increasing term:
/// ₜ|(IA)¹ₓ:ₙ̅⁽ᵐ⁾ = ₜEₓ ·  Σₖ₌₀^{mn-1} (v⁽ᵏ⁺¹⁾/ᵐ . ₖ/ₘ|₁/ₘqₓ₊ₜ . [(k // m) + 1] / m)
/// Eg: for m=12, k=0, 1, ..., 11 the death benefit is 1/m, while k= 12, 13, ..., 23 the death benefit is 2/m, etc.
/// Death benefit increases by 1 each year, pays only if death within n years.
#[builder]
pub fn IAx1n(
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

        // Discount factor for the k-th moment v⁽ᵏ⁺¹⁾/ᵐ
        let discount_factor_with_moment = v.powf(moment * (k_f64 + 1.0) / m);
        // Probability of death ₖ/ₘ|₁/ₘqₓ₊ₜ
        let probability = tqx().mt(&mt).x(x + t).t(1.0 / m).k(k_f64 / m).call()?;
        // Amount of benefit [(k // m) + 1] / m
        let benefit_amount = ((k_f64 / m).floor() + 1.0) / m;
        // Aggregate the result
        summation += discount_factor_with_moment * probability * benefit_amount;
    }

    // Final result
    let result = summation * Ext;
    Ok(result)
}

/// Immediate endowment insurance:
/// ₜ|IAₓ:ₙ̅ = ₜ|IA¹ₓ:ₙ̅ + ₜ|IA¹ₓ:ₙ̅
/// $1 paid at death (if within n years) OR at survival to n years.
#[builder]
pub fn IAxn(
    mt: &MortTableConfig,
    i: f64,
    x: u32,
    n: u32,
    t: Option<u32>,
    m: Option<u32>,
    moment: Option<u32>,
    entry_age: Option<u32>,
) -> Result<f64, Box<dyn std::error::Error>> {
    // Decide if selected table is used
    let mt = get_new_config_with_selected_table(mt, entry_age)?;

    // Unwrap to obtain default values
    let t = t.unwrap_or(0); // Default to 0 (no deferral)
    let m = m.unwrap_or(1); // Default to Annual
    let moment = moment.unwrap_or(1); // Default moment is 1 (mean)

    // Build Ax1n using builder pattern with all parameters
    let term = IAx1n()
        .mt(&mt)
        .i(i)
        .x(x)
        .n(n)
        .t(t)
        .m(m)
        .moment(moment)
        .call()?;

    let pure_endowment = Exn().mt(&mt).i(i).x(x).n(n).t(t).moment(moment).call()?;

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
#[builder]
pub fn DAx1n(
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

        // Discount factor for the k-th moment v⁽ᵏ⁺¹⁾/ᵐ
        let discount_factor_with_moment = v.powf(moment * (k_f64 + 1.0) / m);
        // Probability of death ₖ/ₘ|₁/ₘqₓ₊ₜ
        let probability = tqx().mt(&mt).x(x + t).t(1.0 / m).k(k_f64 / m).call()?;
        // Amount of benefit [n-(k // m)] / m
        let benefit_amount = (n - (k_f64 / m).floor()) / m;
        // Aggregate the result
        summation += discount_factor_with_moment * probability * benefit_amount;
    }

    // Final result
    let result = summation * Ext;
    Ok(result)
}

/// Immediate decreasing endowment insurance:
/// ₜ|(DA)ₓ:ₙ̅ = ₜ|(DA)¹ₓ:ₙ̅ + ₜ|ₙEₓ
/// $1 paid at death (if within n years) OR at survival to n years.
#[builder]
pub fn DAxn(
    mt: &MortTableConfig,
    i: f64,
    x: u32,
    n: u32,
    t: Option<u32>,
    m: Option<u32>,
    moment: Option<u32>,
    entry_age: Option<u32>,
) -> Result<f64, Box<dyn std::error::Error>> {
    // Decide if selected table is used
    let mt = get_new_config_with_selected_table(mt, entry_age)?;

    // Unwrap to obtain default values
    let t = t.unwrap_or(0); // Default to 0 (no deferral)
    let m = m.unwrap_or(1); // Default to Annual
    let moment = moment.unwrap_or(1); // Default moment is 1 (mean)

    // Build Ax1n using builder pattern with all parameters
    let term = DAx1n()
        .mt(&mt)
        .i(i)
        .x(x)
        .n(n)
        .t(t)
        .m(m)
        .moment(moment)
        .call()?;

    let pure_endowment = Exn().mt(&mt).i(i).x(x).n(n).t(t).moment(moment).call()?;

    let result = term + pure_endowment;

    Ok(result)
}

//-----------------Geometric increasing------------------

/// Immediate geometric whole life:
/// Aₓ⁽ᵍ⁾ with adjusted interest rate i′ = (1+i)/(1+g) - 1
///
/// Death benefit grows geometrically at rate g each year.
#[builder]
pub fn gAx(
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

    let result = Ax().mt(&mt).i(new_i).x(x).t(t).m(m).moment(moment).call()?;
    Ok(result)
}

/// Immediate geometric n-year term:
/// A¹ₓ:ₙ̅⁽ᵍ⁾ with adjusted interest rate i′ = (1+i)/(1+g) - 1
///
/// Death benefit grows geometrically at rate g for n years.
#[builder]
pub fn gAx1n(
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

    let result = Ax1n()
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

/// Immediate geometric n-year pure endowment:
/// Aₓ:ₙ̅¹⁽ᵍ⁾ with adjusted interest rate i′ = (1+i)/(1+g) - 1
///
/// Death benefit grows geometrically at rate g for n years.
#[builder]
pub fn gExn(
    mt: &MortTableConfig,
    i: f64,
    x: u32,
    n: Option<u32>,
    t: Option<u32>,
    moment: Option<u32>,
    entry_age: Option<u32>,
    g: f64,
) -> Result<f64, Box<dyn std::error::Error>> {
    // Vaidate the parameters
    let params = SingleLifeParams {
        mt: mt.clone(),
        i,
        x,
        n,
        t,
        m: None, // Not used
        moment,
        entry_age,
    };

    params
        .validate_all()
        .map_err(|err| Box::new(err) as Box<dyn std::error::Error>)?;

    // Unwrap to obtain default values
    let n = n.unwrap_or(0); // Default to 0 (no term)
    let t = t.unwrap_or(0); // Default to 0 (no deferral)
    let moment = moment.unwrap_or(1); // Default moment is 1 (mean)

    // Decide if selected table is used
    let mt = get_new_config_with_selected_table(mt, entry_age)?;

    // Replace the effective interest rate with the adjusted one
    let new_i = (1.0 + i) / (1.0 + g) - 1.0;

    let result = Exn()
        .mt(&mt)
        .i(new_i)
        .x(x)
        .n(n)
        .t(t)
        .moment(moment)
        .call()?;

    Ok(result)
}

/// Immediate geometric n-year endowment:
/// Aₓ:ₙ̅⁽ᵍ⁾ = A¹ₓ:ₙ̅⁽ᵍ⁾ + Aₓ:ₙ̅¹⁽ᵍ⁾
///
/// Death benefit grows geometrically at rate g for n years.
#[builder]
pub fn gAxn(
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
    // Decide if selected table is used
    let mt = get_new_config_with_selected_table(mt, entry_age)?;

    // Unwrap to obtain default values
    let t = t.unwrap_or(0); // Default to 0 (no deferral)
    let m = m.unwrap_or(1); // Default to Annual
    let moment = moment.unwrap_or(1); // Default moment is 1 (mean)

    // Build Ax1n using builder pattern with all parameters
    let term = gAx1n()
        .mt(&mt)
        .i(i)
        .x(x)
        .n(n)
        .t(t)
        .m(m)
        .moment(moment)
        .g(g)
        .call()?;

    let pure_endowment = gExn()
        .mt(&mt)
        .i(i)
        .x(x)
        .n(n)
        .t(t)
        .moment(moment)
        .g(g)
        .call()?;

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
    use crate::mt_data::MortData;
    use approx::assert_abs_diff_eq;

    #[test]
    fn test_fn_A1xn_n_is_0() {
        // Load the AM92 mortality table
        let am92 = MortData::from_xlsx("data/am92.xlsx", "am92")
            .expect("Failed to load AM92 selected table");

        // Create a MortTableConfig with the loaded data
        let mt = MortTableConfig::builder().data(am92).build();

        // Call the Ax1n function with n = 0
        let ans = Ax1n().mt(&mt).i(0.05).x(70).n(0).call().unwrap();
        let expected = 0.0;
        assert_abs_diff_eq!(ans, expected, epsilon = 1e-6);
    }

    #[test]
    fn test_fn_Exn_n_is_0() {
        // Load the AM92 mortality table
        let am92 = MortData::from_xlsx("data/am92.xlsx", "am92")
            .expect("Failed to load AM92 selected table");

        // Create a MortTableConfig with the loaded data
        let mt = MortTableConfig::builder().data(am92).build();

        // Call the Exn function with n = 0
        let ans = Exn().mt(&mt).i(0.05).x(70).n(0).call().unwrap();

        let expected = 1.0;

        assert_abs_diff_eq!(ans, expected, epsilon = 1e-6);
    }

    #[test]
    fn test_fn_Axn_benefit_cm1() {
        // Load the AM92 mortality table
        let am92 = MortData::from_xlsx("data/am92.xlsx", "am92")
            .expect("Failed to load AM92 selected table");

        // Create a MortTableConfig with the loaded data
        let mt = MortTableConfig::builder().data(am92).build();

        // Calculate  A₍₇₀₎:₃
        let ans = Axn()
            .mt(&mt)
            .i(0.05)
            .x(70)
            .n(3)
            .entry_age(70)
            .call()
            .unwrap();

        let expected = 0.8663440;

        assert_abs_diff_eq!(ans, expected, epsilon = 1e-6);
    }

    #[test]
    fn test_fn_IAx1n_benefit_cm1() {
        // Load the AM92 mortality table
        let am92 = MortData::from_xlsx("data/am92.xlsx", "am92")
            .expect("Failed to load AM92 selected table");

        // Create a MortTableConfig with the loaded data
        let mt = MortTableConfig::builder().data(am92).build();

        // Call the IAx1n function with n = 3
        let ans = IAx1n().mt(&mt).i(0.04).x(50).n(10).call().unwrap();

        // Expected value calculated manually or from a reliable source
        let expected = 8.55929 - (882.85 / 1366.61) * (8.36234 + 10.0 * 0.45640);

        assert_abs_diff_eq!(ans, expected, epsilon = 1e-4);
    }
}
