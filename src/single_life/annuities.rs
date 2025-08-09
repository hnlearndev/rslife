#![allow(non_snake_case)]
#![allow(clippy::too_many_arguments)]

use super::survivals::tpx;
use crate::RSLifeResult;
use crate::helpers::get_new_config_with_selected_table;
use crate::mt_config::MortTableConfig;
use crate::params::SingleLifeParams;
use bon::builder;

/// Calculate the present value of a life annuity-immediate (in arrears).
///
/// This function computes the actuarial present value of a life annuity where payments are made at the end of each period (in arrears), contingent on survival.
///
/// # Parameters
/// - `mt`: Mortality table configuration (see [`MortTableConfig`])
/// - `age`: Age at entry
/// - `n`: Number of payment periods
/// - `i`: Interest rate per period (as a decimal, e.g., 0.03 for 3%)
///
/// # Returns
/// - Present value of the annuity (f64)
///
/// # Formula
/// The present value of an annuity-immediate (in arrears) is:
/// $$
/// \ddot{a}_{x:\overline{n}} = \sum_{k=1}^{n} v^k \, {}_k p_x
/// $$
/// where:
/// - $v = 1 / (1 + i)$ is the discount factor
/// - ${}_k p_x$ is the probability of survival to time $k$
///
/// # Example
/// ```rust
/// # use rslife::prelude::*;
/// //let pv = annuity_immediate(mt, 65, 10, 0.03)?;
/// //println!("Present value: {:.2}", pv);
/// # RSLifeResult::Ok(())
/// ```
///
/// # See Also
/// - [`annuity_due`] for payments in advance
/// - [`MortTableConfig`] for mortality table setup

// ==================In arrears==================
#[builder]
pub fn axn(
    mt: &MortTableConfig,
    i: f64,
    x: u32,
    n: u32,
    #[builder(default = 0)] t: u32,
    #[builder(default = 1)] m: u32,
    #[builder(default = 1)] moment: u32,
    entry_age: Option<u32>,
    #[builder(default = true)] validate: bool,
) -> RSLifeResult<f64> {
    annuity_procedure(
        mt,
        i,
        x,
        n,
        t,
        m,
        moment,
        entry_age,
        validate,
        CashFlowStructure::Flat,
        CashFlowTiming::InArrears,
    )
}

#[builder]
pub fn ax(
    mt: &MortTableConfig,
    i: f64,
    x: u32,
    #[builder(default = 0)] t: u32,
    #[builder(default = 1)] m: u32,
    #[builder(default = 1)] moment: u32,
    entry_age: Option<u32>,
    #[builder(default = true)] validate: bool,
) -> RSLifeResult<f64> {
    let max_age = mt.max_age()?;
    let n = max_age - x - t;
    annuity_procedure(
        mt,
        i,
        x,
        n,
        t,
        m,
        moment,
        entry_age,
        validate,
        CashFlowStructure::Flat,
        CashFlowTiming::InArrears,
    )
}

#[builder]
pub fn Iaxn(
    mt: &MortTableConfig,
    i: f64,
    x: u32,
    n: u32,
    #[builder(default = 0)] t: u32,
    #[builder(default = 1)] m: u32,
    #[builder(default = 1)] moment: u32,
    entry_age: Option<u32>,
    #[builder(default = true)] validate: bool,
) -> RSLifeResult<f64> {
    annuity_procedure(
        mt,
        i,
        x,
        n,
        t,
        m,
        moment,
        entry_age,
        validate,
        CashFlowStructure::Increasing,
        CashFlowTiming::InArrears,
    )
}

#[builder]
pub fn Iax(
    mt: &MortTableConfig,
    i: f64,
    x: u32,
    #[builder(default = 0)] t: u32,
    #[builder(default = 1)] m: u32,
    #[builder(default = 1)] moment: u32,
    entry_age: Option<u32>,
    #[builder(default = true)] validate: bool,
) -> RSLifeResult<f64> {
    let max_age = mt.max_age()?;
    let n = max_age - x - t;
    annuity_procedure(
        mt,
        i,
        x,
        n,
        t,
        m,
        moment,
        entry_age,
        validate,
        CashFlowStructure::Increasing,
        CashFlowTiming::InArrears,
    )
}

#[builder]
pub fn Daxn(
    mt: &MortTableConfig,
    i: f64,
    x: u32,
    n: u32,
    #[builder(default = 0)] t: u32,
    #[builder(default = 1)] m: u32,
    #[builder(default = 1)] moment: u32,
    entry_age: Option<u32>,
    #[builder(default = true)] validate: bool,
) -> RSLifeResult<f64> {
    annuity_procedure(
        mt,
        i,
        x,
        n,
        t,
        m,
        moment,
        entry_age,
        validate,
        CashFlowStructure::Decreasing,
        CashFlowTiming::InArrears,
    )
}

#[builder]
pub fn gaxn(
    mt: &MortTableConfig,
    i: f64,
    x: u32,
    n: u32,
    #[builder(default = 0)] t: u32,
    #[builder(default = 1)] m: u32,
    #[builder(default = 1)] moment: u32,
    entry_age: Option<u32>,
    #[builder(default = true)] validate: bool,
    g: f64,
) -> RSLifeResult<f64> {
    // Replace the effective interest rate with the adjusted one
    let new_i = (1.0 + i) / (1.0 + g) - 1.0;

    let built = axn()
        .mt(mt)
        .i(new_i)
        .x(x)
        .n(n)
        .t(t)
        .m(m)
        .moment(moment)
        .validate(validate);

    match entry_age {
        Some(age) => built.entry_age(age).call(),
        None => built.call(),
    }
}

#[builder]
pub fn gax(
    mt: &MortTableConfig,
    i: f64,
    x: u32,
    #[builder(default = 0)] t: u32,
    #[builder(default = 1)] m: u32,
    #[builder(default = 1)] moment: u32,
    entry_age: Option<u32>,
    #[builder(default = true)] validate: bool,
    g: f64,
) -> RSLifeResult<f64> {
    // Replace the effective interest rate with the adjusted one
    let new_i = (1.0 + i) / (1.0 + g) - 1.0;

    let built = ax()
        .mt(mt)
        .i(new_i)
        .x(x)
        .t(t)
        .m(m)
        .moment(moment)
        .validate(validate);

    match entry_age {
        Some(age) => built.entry_age(age).call(),
        None => built.call(),
    }
}

// ==================In advance==================
//-----------------Basic------------------

/// Due temporary annuity-due payable m times per year:
///
/// Present value of 1/m paid m times per year for up to n years, starting immediately, provided the insured is alive at each payment time.
///
/// # Formula
/// ```text
/// ₜ|äₓ:ₙ̅⁽ᵐ⁾ = ₜEₓ · Σₖ₌₀^{mn-1} [1/m · vᵏ/ᵐ · ₖ/ₘpₓ₊ₜ]
/// ```
/// where:
/// - `v = 1/(1+i)` is the discount factor
/// - `ₜEₓ` is the probability of surviving from age x to age x+t
/// - `ₖ/ₘpₓ₊ₜ` is the probability of surviving k/m years after age x+t
/// - `n` is the term of the annuity
/// - `t` is the deferral period (default 0)
/// - `m` is the number of payments per year (default 1)
/// - `moment` is the moment to calculate (default 1, i.e., mean)
/// - `entry_age` is the age at which the insured enters the policy (default None, uses ultimate table)
///
/// # Examples
///
/// ## Basic Temporary Annuity-Due
/// ```rust
/// # use rslife::prelude::*;
/// # let mort_data = MortData::from_soa_url_id(1704)?;
/// # let config = MortTableConfig::builder().data(mort_data).build().unwrap();
/// let temp_annuity = aaxn().mt(&config).i(0.03).x(40).n(10).call()?;
/// println!("Temporary annuity-due: {:.6}", temp_annuity);
/// # RSLifeResult::Ok(())
/// ```
#[builder]
pub fn aaxn(
    mt: &MortTableConfig,
    i: f64,
    x: u32,
    n: u32,
    #[builder(default = 0)] t: u32,
    #[builder(default = 1)] m: u32,
    #[builder(default = 1)] moment: u32,
    entry_age: Option<u32>,
    #[builder(default = true)] validate: bool,
) -> RSLifeResult<f64> {
    annuity_procedure(
        mt,
        i,
        x,
        n,
        t,
        m,
        moment,
        entry_age,
        validate,
        CashFlowStructure::Flat,
        CashFlowTiming::InAdvance,
    )
}

/// Life annuity-due payable m times per year:
///
/// Present value of 1/m paid m times per year for life, starting immediately, provided the insured is alive at each payment time.
///
/// # Formula
/// ```text
/// ₜ|äₓ⁽ᵐ⁾ = ₜEₓ · Σₖ₌₀^∞ [1/m · vᵏ/ᵐ · ₖ/ₘpₓ₊ₜ]
/// ```
/// where:
/// - `v = 1/(1+i)` is the discount factor
/// - `ₜEₓ` is the probability of surviving from age x to age x+t
/// - `ₖ/ₘpₓ₊ₜ` is the probability of surviving k/m years after age x+t
/// - `t` is the deferral period (default 0)
/// - `m` is the number of payments per year (default 1)
/// - `moment` is the moment to calculate (default 1, i.e., mean)
/// - `entry_age` is the age at which the insured enters the policy (default None, uses ultimate table)
///
/// # Examples
///
/// ## Basic Life Annuity-Due
/// ```rust
/// # use rslife::prelude::*;
/// # let mort_data = MortData::from_soa_url_id(1704)?;
/// # let config = MortTableConfig::builder().data(mort_data).build().unwrap();
/// let annuity = aax().mt(&config).i(0.03).x(40).call()?;
/// println!("Life annuity-due: {:.6}", annuity);
/// # RSLifeResult::Ok(())
/// ```
#[builder]
pub fn aax(
    mt: &MortTableConfig,
    i: f64,
    x: u32,
    #[builder(default = 0)] t: u32,
    #[builder(default = 1)] m: u32,
    #[builder(default = 1)] moment: u32,
    entry_age: Option<u32>,
    #[builder(default = true)] validate: bool,
) -> RSLifeResult<f64> {
    let max_age = mt.max_age()?;
    let n = max_age - x - t;
    annuity_procedure(
        mt,
        i,
        x,
        n,
        t,
        m,
        moment,
        entry_age,
        validate,
        CashFlowStructure::Flat,
        CashFlowTiming::InAdvance,
    )
}

//-----------------Increasing------------------
/// Due increasing temporary life annuity-due payable m times per year:
///
/// Present value of an increasing life annuity-due: payments of 1/m made m times per year for n years, with each annual payment increasing by 1 (i.e., k-th annual payment is k).
/// For m=12, k=0..11 the annuity is 1/m, k=12..23 the annuity is 2/m, etc.
///
/// # Formula
/// ```text
/// ₜ|(Iä)ₓ:ₙ̅⁽ᵐ⁾ = ₜEₓ · Σₖ₌₀^{mn-1} [((k // m) + 1) / m · vᵏ/ᵐ · ₖ/ₘpₓ₊ₜ]
/// ```
/// where:
/// - `v = 1/(1+i)` is the discount factor
/// - `ₜEₓ` is the probability of surviving from age x to age x+t
/// - `ₖ/ₘpₓ₊ₜ` is the probability of surviving k/m years after age x+t
/// - `n` is the term of the annuity
/// - `m` is the number of payments per year (default 1)
/// - `t` is the deferral period (default 0)
/// - `moment` is the moment to calculate (default 1, i.e., mean)
/// - `entry_age` is the age at which the insured enters the policy (default None, uses ultimate table)
///
/// # Examples
///
/// ## Basic Increasing Temporary Annuity-Due
/// ```rust
/// # use rslife::prelude::*;
/// # let mort_data = MortData::from_soa_url_id(1704)?;
/// # let config = MortTableConfig::builder().data(mort_data).build().unwrap();
/// let inc_temp_annuity = Iaaxn().mt(&config).i(0.03).x(40).n(10).call()?;
/// println!("Increasing temporary annuity-due: {:.6}", inc_temp_annuity);
/// # RSLifeResult::Ok(())
/// ```
#[builder]
pub fn Iaaxn(
    mt: &MortTableConfig,
    i: f64,
    x: u32,
    n: u32,
    #[builder(default = 0)] t: u32,
    #[builder(default = 1)] m: u32,
    #[builder(default = 1)] moment: u32,
    entry_age: Option<u32>,
    #[builder(default = true)] validate: bool,
) -> RSLifeResult<f64> {
    annuity_procedure(
        mt,
        i,
        x,
        n,
        t,
        m,
        moment,
        entry_age,
        validate,
        CashFlowStructure::Increasing,
        CashFlowTiming::InAdvance,
    )
}

/// Increasing life annuity-due payable m times per year:
///
/// Present value of an increasing life annuity-due: payments of 1/m made m times per year for life, with each annual payment increasing by 1 (i.e., k-th annual payment is k).
/// For m=12, k=0..11 the annuity is 1/m, k=12..23 the annuity is 2/m, etc.
///
/// # Formula
/// ```text
/// ₜ|(Iä)ₓ⁽ᵐ⁾ = ₜEₓ · Σₖ₌₀^∞ [((k // m) + 1) / m · vᵏ/ᵐ · ₖ/ₘpₓ₊ₜ]
/// ```
/// where:
/// - `v = 1/(1+i)` is the discount factor
/// - `ₜEₓ` is the probability of surviving from age x to age x+t
/// - `ₖ/ₘpₓ₊ₜ` is the probability of surviving k/m years after age x+t
/// - `m` is the number of payments per year (default 1)
/// - `t` is the deferral period (default 0)
/// - `moment` is the moment to calculate (default 1, i.e., mean)
/// - `entry_age` is the age at which the insured enters the policy (default None, uses ultimate table)
///
/// # Examples
///
/// ## Basic Increasing Life Annuity-Due
/// ```rust
/// # use rslife::prelude::*;
/// # let mort_data = MortData::from_soa_url_id(1704)?;
/// # let config = MortTableConfig::builder().data(mort_data).build().unwrap();
/// let inc_annuity = Iaax().mt(&config).i(0.03).x(40).call()?;
/// println!("Increasing life annuity-due: {:.6}", inc_annuity);
/// # RSLifeResult::Ok(())
/// ```
#[builder]
pub fn Iaax(
    mt: &MortTableConfig,
    i: f64,
    x: u32,
    #[builder(default = 0)] t: u32,
    #[builder(default = 1)] m: u32,
    #[builder(default = 1)] moment: u32,
    entry_age: Option<u32>,
    #[builder(default = true)] validate: bool,
) -> RSLifeResult<f64> {
    let max_age = mt.max_age()?;
    let n = max_age - x - t;
    annuity_procedure(
        mt,
        i,
        x,
        n,
        t,
        m,
        moment,
        entry_age,
        validate,
        CashFlowStructure::Increasing,
        CashFlowTiming::InAdvance,
    )
}

//-----------------Decreasing------------------
/// Decreasing temporary life annuity-due:
///
/// Present value of a decreasing life annuity-due: payments of n/m made m times per year for n years, with each annual payment decreasing by 1 (i.e., k-th annual payment is n-k+1).
/// For m=12, k=0..11 the annuity is n/m, k=12..23 the annuity is (n-1)/m, etc.
///
/// # Formula
/// ```text
/// ₜ|(Dä)ₓ:ₙ̅⁽ᵐ⁾ = ₜEₓ · Σₖ₌₀^{mn-1} [(n - (k // m)) / m · vᵏ/ᵐ · ₖ/ₘpₓ₊ₜ]
/// ```
/// where:
/// - `v = 1/(1+i)` is the discount factor
/// - `ₜEₓ` is the probability of surviving from age x to age x+t
/// - `ₖ/ₘpₓ₊ₜ` is the probability of surviving k/m years after age x+t
/// - `n` is the term of the annuity
/// - `m` is the number of payments per year (default 1)
/// - `t` is the deferral period (default 0)
/// - `moment` is the moment to calculate (default 1, i.e., mean)
/// - `entry_age` is the age at which the insured enters the policy (default None, uses ultimate table)
///
/// # Examples
///
/// ## Basic Decreasing Temporary Annuity-Due
/// ```rust
/// # use rslife::prelude::*;
/// # let mort_data = MortData::from_soa_url_id(1704)?;
/// # let config = MortTableConfig::builder().data(mort_data).build().unwrap();
/// let dec_temp_annuity = Daaxn().mt(&config).i(0.03).x(40).n(10).call()?;
/// println!("Decreasing temporary annuity-due: {:.6}", dec_temp_annuity);
/// # RSLifeResult::Ok(())
/// ```
#[builder]
pub fn Daaxn(
    mt: &MortTableConfig,
    i: f64,
    x: u32,
    n: u32,
    #[builder(default = 0)] t: u32,
    #[builder(default = 1)] m: u32,
    #[builder(default = 1)] moment: u32,
    entry_age: Option<u32>,
    #[builder(default = true)] validate: bool,
) -> RSLifeResult<f64> {
    annuity_procedure(
        mt,
        i,
        x,
        n,
        t,
        m,
        moment,
        entry_age,
        validate,
        CashFlowStructure::Decreasing,
        CashFlowTiming::InAdvance,
    )
}

//-----------------Geometric increasing------------------
/// Geometric increasing life annuity-due:
///
/// Present value of a geometric increasing life annuity-due: payments of 1/m made m times per year for life, with each annual payment increasing by a factor of (1+g) each year (i.e., geometric progression).
/// The effective interest rate is adjusted: i' = (1+i)/(1+g) - 1.
///
/// # Formula
/// ```text
/// äₓ⁽ᵍ⁾ = aax(i')
/// where i' = (1+i)/(1+g) - 1
/// ```
/// - `g` is the geometric growth rate of the annuity
/// - All other parameters as in aax
///
/// # Examples
///
/// ## Basic Geometric Life Annuity-Due
/// ```rust
/// # use rslife::prelude::*;
/// # let mort_data = MortData::from_soa_url_id(1704)?;
/// # let config = MortTableConfig::builder().data(mort_data).build().unwrap();
/// let geom_annuity = gaax().mt(&config).i(0.03).x(40).g(0.02).call()?;
/// println!("Geometric life annuity-due: {:.6}", geom_annuity);
/// # RSLifeResult::Ok(())
/// ```
#[builder]
pub fn gaax(
    mt: &MortTableConfig,
    i: f64,
    x: u32,
    #[builder(default = 0)] t: u32,
    #[builder(default = 1)] m: u32,
    #[builder(default = 1)] moment: u32,
    entry_age: Option<u32>,
    #[builder(default = true)] validate: bool,
    g: f64,
) -> RSLifeResult<f64> {
    // Replace the effective interest rate with the adjusted one
    let new_i = (1.0 + i) / (1.0 + g) - 1.0;

    let built = aax()
        .mt(mt)
        .i(new_i)
        .x(x)
        .t(t)
        .m(m)
        .moment(moment)
        .validate(validate);

    match entry_age {
        Some(age) => built.entry_age(age).call(),
        None => built.call(),
    }
}

/// Geometric increasing temporary annuity-due:
///
/// Present value of a geometric increasing temporary annuity-due: payments of 1/m made m times per year for n years, with each annual payment increasing by a factor of (1+g) each year (i.e., geometric progression).
/// The effective interest rate is adjusted: i' = (1+i)/(1+g) - 1.
///
/// # Formula
/// ```text
/// äₓ:ₙ̅⁽ᵍ⁾ = aaxn(i')
/// where i' = (1+i)/(1+g) - 1
/// ```
/// - `g` is the geometric growth rate of the annuity
/// - All other parameters as in aaxn
///
/// # Examples
///
/// ## Basic Geometric Temporary Annuity-Due
/// ```rust
/// # use rslife::prelude::*;
/// # let mort_data = MortData::from_ifoa_url_id("AM92")?;
/// # let config = MortTableConfig::builder().data(mort_data).build().unwrap();
/// let geom_temp_annuity = gaaxn().mt(&config).i(0.03).x(40).n(10).g(0.02).call()?;
/// println!("Geometric temporary annuity-due: {:.6}", geom_temp_annuity);
/// # RSLifeResult::Ok(())
/// ```
#[builder]
pub fn gaaxn(
    mt: &MortTableConfig,
    i: f64,
    x: u32,
    n: u32,
    #[builder(default = 0)] t: u32,
    #[builder(default = 1)] m: u32,
    #[builder(default = 1)] moment: u32,
    entry_age: Option<u32>,
    #[builder(default = true)] validate: bool,
    g: f64,
) -> RSLifeResult<f64> {
    // Replace the effective interest rate with the adjusted one
    let new_i = (1.0 + i) / (1.0 + g) - 1.0;

    let built = aaxn()
        .mt(mt)
        .i(new_i)
        .x(x)
        .n(n)
        .t(t)
        .m(m)
        .moment(moment)
        .validate(validate);

    match entry_age {
        Some(age) => built.entry_age(age).call(),
        None => built.call(),
    }
}

// ================================================
// PRIVATE FUNCTIONS
// ================================================

#[derive(PartialEq)]
enum CashFlowStructure {
    Flat,
    Increasing,
    Decreasing,
}

#[derive(PartialEq)]
enum CashFlowTiming {
    InAdvance,
    InArrears,
}

fn annuity_procedure(
    mt: &MortTableConfig,
    i: f64,
    x: u32,
    n: u32,
    t: u32,
    m: u32,
    moment: u32,
    entry_age: Option<u32>,
    validate: bool,
    structure: CashFlowStructure,
    timing: CashFlowTiming,
) -> RSLifeResult<f64> {
    if validate {
        // Validate the parameters
        let params = SingleLifeParams {
            mt: mt.clone(),
            i,
            x,
            n,
            t,
            m,
            moment,
            entry_age,
        };

        params
            .validate_all()
            .map_err(|err| Box::new(err) as Box<dyn std::error::Error>)?;
    }

    // Decide if selected table is used
    let mt = get_new_config_with_selected_table(mt, entry_age)?;

    // Initialize k array
    let k_arr_0: Vec<f64> = (0..n * m).map(|k| k as f64).collect();
    let k_arr: Vec<f64> = match timing {
        CashFlowTiming::InAdvance => k_arr_0.clone(),
        CashFlowTiming::InArrears => (1..n * m + 1).map(|k| k as f64).collect(),
    };

    // Convert parameters to f64 for calculations
    let v = 1.0 / (1.0 + i);
    let x = f64::from(x);
    let n = f64::from(n);
    let t = f64::from(t);
    let m = f64::from(m);
    let moment = f64::from(moment);

    // ----------Discount factor----------
    let discount_factors: Vec<f64> = k_arr.iter().map(|&k| v.powf(moment * k / m)).collect();

    // ----------Probabilities vectors----------
    let probabilities: Vec<f64> = k_arr
        .iter()
        .map(|&k| {
            tpx()
                .mt(&mt)
                .x(x + t)
                .t(k / m)
                .validate(false)
                .call()
                .unwrap_or(0.0)
        })
        .collect();

    // ----------Benefit vector----------
    let mut amounts: Vec<f64> = match structure {
        CashFlowStructure::Flat => vec![1.0; (n * m) as usize],
        CashFlowStructure::Increasing => k_arr_0.iter().map(|&k| (k / m).floor() + 1.0).collect(),
        CashFlowStructure::Decreasing => k_arr_0.iter().map(|&k| n - (k / m).floor()).collect(),
    };

    // The amount is very much alike with benfits except that we divide by m - m-payable
    amounts.iter_mut().for_each(|x| *x /= m);

    // Calculate the summation
    let summation: f64 = discount_factors
        .iter()
        .zip(probabilities.iter())
        .zip(amounts.iter())
        .map(|((df, prob), amount)| df * prob * amount)
        .sum();

    // Deferred factor: ₜEₓ = vᵗ · ₜpₓ
    let deferred_factor = v.powf(t) * tpx().mt(&mt).x(x).t(t).validate(false).call()?;

    // Final result
    Ok(summation * deferred_factor)
}

// ================================================
// UNIT TESTS
// ================================================
#[cfg(test)]
mod tests {
    use super::*;
    use crate::mt_config::MortTableConfig;
    use crate::mt_config::mt_data::MortData;
    use approx::assert_abs_diff_eq;

    #[test]
    fn test_fn_aax_01() {
        // CM1 2019 Chapter 17
        // Create MortTableConfig with AM92
        let am92 = MortData::from_ifoa_url_id("AM92").expect("Failed to load AM92 selected table");
        let mt = MortTableConfig::builder().data(am92).build().unwrap();
        let ans = aax().mt(&mt).i(0.04).x(30).call().unwrap();
        let expected = 21.834;
        assert_abs_diff_eq!(ans, expected, epsilon = 1e-3);
    }

    #[test]
    fn test_fn_aax_02() {
        // CM1 2019 Chapter 17
        // Create MortTableConfig with PMA92C20
        let pma92c20 =
            MortData::from_ifoa_custom("PMA92C20").expect("Failed to load PMA92C20 table");
        let mt = MortTableConfig::builder().data(pma92c20).build().unwrap();
        let ans = aax().mt(&mt).i(0.04).x(75).call().unwrap();
        let expected = 9.456;
        assert_abs_diff_eq!(ans, expected, epsilon = 1e-3);
    }

    #[test]
    fn test_fn_aax_03() {
        // April 2025 CM1 question 3
        // Create MortTableConfig with AM92
        let am92 = MortData::from_ifoa_url_id("AM92").expect("Failed to load AM92 selected table");
        let mt = MortTableConfig::builder().data(am92).build().unwrap();
        let ans = aax().mt(&mt).i(0.04).x(50).entry_age(50).call().unwrap();
        let expected = 17.454;
        assert_abs_diff_eq!(ans, expected, epsilon = 1e-3);
    }

    #[test]
    fn test_fn_aax_04() {
        // Testing relationship between in arrear and in advance
        // Create MortTableConfig with AM92
        let am92 = MortData::from_ifoa_url_id("AM92").expect("Failed to load AM92 selected table");
        let mt = MortTableConfig::builder().data(am92).build().unwrap();
        let advance = aax().mt(&mt).i(0.04).x(50).call().unwrap();
        let arrear = ax().mt(&mt).i(0.04).x(50).call().unwrap();
        // ₜ|äₓ⁽ᵐ⁾ =vᵗₜpₓ(aₓ₊ₜ⁽ᵐ⁾ + 1/m) = ₜ|aₓ⁽ᵐ⁾ + vᵗₜpₓ/m
        // For t = 0 , m=1:  äₓ⁽ᵐ⁾ = aₓ⁽ᵐ⁾ + 1
        assert_abs_diff_eq!(advance - arrear, 1.0, epsilon = 1e-4);
    }

    #[test]
    fn test_fn_aax_05() {
        // Testing relationship between in arrear and in advance
        // Create MortTableConfig with AM92
        let am92 = MortData::from_ifoa_url_id("AM92").expect("Failed to load AM92 selected table");
        let mt = MortTableConfig::builder().data(am92).build().unwrap();
        let advance = aaxn().mt(&mt).i(0.04).x(50).n(20).m(12).call().unwrap();
        let arrear = axn().mt(&mt).i(0.04).x(50).n(20).m(12).call().unwrap();
        // ₜ|äₓ:ₙ̅⁽ᵐ⁾ = vᵗₜpₓ[äₓ₊ₜ:ₙ̅⁽ᵐ⁾ + 1/m(1-vⁿₙpₓ)] = ₜ|aₓ:ₙ̅⁽ᵐ⁾ + vᵗₜpₓ(1-vⁿₙpₓ)/m
        // For t = 0, m=12:  the diffrence = (1-vⁿₙpₓ)/12
        let vn = (1.0f64 / (1.0f64 + 0.04f64)).powf(20.0f64);
        let pxn = tpx()
            .mt(&mt)
            .x(50.0)
            .t(20.0)
            .validate(false)
            .call()
            .unwrap();
        let expected_diff = (1.0 - vn * pxn) / 12.0;
        assert_abs_diff_eq!(advance - arrear, expected_diff, epsilon = 1e-4);
    }

    #[test]
    fn test_fn_Iaax_annuities() {
        // April 2025 CM1 question 1
        // Create MortTableConfig with AM92
        let am92 = MortData::from_ifoa_url_id("AM92").expect("Failed to load AM92 selected table");
        let mt = MortTableConfig::builder().data(am92).build().unwrap();
        // Calculate increasing life annuity (Ia)x for age 50
        // Note: No entry_age needed for whole life increasing annuity
        let ans = Iaax().mt(&mt).i(0.04).x(50).call().unwrap();

        let expected = 231.007;
        // Lower down the precision to 4 decimal places since the expected value is rounded
        assert_abs_diff_eq!(ans, expected, epsilon = 1e-4);
    }
}
