#![allow(non_snake_case)]
#![allow(clippy::too_many_arguments)]

use super::survivals::{tpx, tqx};
use crate::RSLifeResult;
use crate::helpers::get_new_config_with_selected_table;
use crate::mt_config::MortTableConfig;
use crate::params::SingleLifeParams;
use bon::builder;

// =======================================
// PUBLIC FUNCTIONS
// =======================================

//-----------------Basic------------------

/// Immediate pure endowment
///
/// Present value of $1 paid if and only if the insured survives n years.
/// This is a fundamental building block for endowment insurance calculations.
///
/// # Formula
/// ```text
/// ₜ|ₙEₓ =  ₜ+ₙE = vⁿ⁺ᵗ · ₜ+ₙpₓ
/// ```
/// where:
/// - `v = 1/(1+i)` is the discount factor
/// - `ₜ+ₙpₓ` is the probability of surviving from age x to age x+t+n
/// - `t` is the deferral period (default 0)
/// - `n` is the endowment period
/// - `moment` is the moment to calculate (default 1, i.e., mean)
/// - `entry_age` is the age at which the insured enters the policy (default None, uses ultimate table)
///
/// # Examples
///
/// ## Basic Pure Endowment
/// ```rust
/// # use rslife::prelude::*;
/// # let mort_data = MortData::from_soa_url_id(1704)?;
/// # let config = MortTableConfig::builder()
/// #     .data(mort_data)
/// #     .build()
/// #     .unwrap();
///
/// // 10-year pure endowment for age 30
/// let pure_endowment = Exn()
///     .mt(&config)
///     .i(0.03)    // 3% interest rate
///     .x(30)      // age 30
///     .n(10)      // 10 years
///     .call()?;
///
/// println!("10-year pure endowment: {:.6}", pure_endowment);
/// # RSLifeResult::Ok(())
/// ```
///
/// ## Deferred Pure Endowment
/// ```rust
/// # use rslife::prelude::*;
/// # let mort_data = MortData::from_soa_url_id(1704)?;
/// # let config = MortTableConfig::builder().data(mort_data).build()?;
///
/// // 20-year pure endowment deferred 5 years for age 25
/// let deferred_endowment = Exn()
///     .mt(&config)
///     .i(0.04)
///     .x(25)
///     .n(20)
///     .t(5)  // 5-year deferral
///     .call()?;
///
/// println!("Deferred pure endowment: {:.6}", deferred_endowment);
/// # RSLifeResult::Ok(())
/// ```
///
/// ## Higher Moments
/// ```rust
/// # use rslife::prelude::*;
/// # let mort_data = MortData::from_soa_url_id(1704)?;
/// # let config = MortTableConfig::builder().data(mort_data).build()?;
///
/// // Second moment for variance calculations
/// let second_moment = Exn()
///     .mt(&config)
///     .i(0.03)
///     .x(40)
///     .n(15)
///     .moment(2)  // Second moment
///     .call()?;
///
/// println!("Second moment: {:.6}", second_moment);
/// # RSLifeResult::Ok(())
/// ```
#[builder]
pub fn Exn(
    mt: &MortTableConfig,
    i: f64,
    x: u32,
    n: u32,
    #[builder(default = 0)] t: u32,
    #[builder(default = 1)] moment: u32,
    entry_age: Option<u32>,
    #[builder(default = true)] validate: bool,
) -> RSLifeResult<f64> {
    // Validate the parameters
    if validate {
        let params = SingleLifeParams {
            mt: mt.clone(),
            i,
            x,
            n,
            t,
            m: 1, // Default to Annual since pure endowment is paid at the n of the term
            moment,
            entry_age,
        };

        params
            .validate_all()
            .map_err(|err| Box::new(err) as Box<dyn std::error::Error>)?;
    };

    // As provided - no default
    let v = 1.0 / (1.0 + i);
    let x = x as f64;
    let t = t as f64;
    let n = n as f64;
    let moment = moment as f64;

    // Decide if selected table is used
    let mt = get_new_config_with_selected_table(mt, entry_age)?;
    let discount_factor = v.powf(moment * (t + n)); // vⁿ⁺ᵗ
    let prob = tpx().mt(&mt).x(x).t(t + n).call()?; // ₜ+ₙpₓ
    let result = discount_factor * prob;
    Ok(result)
}

/// Axn1 is an alternative name for [`Exn`] (pure endowment).
///
/// This function is provided for compatibility with actuarial notation.
#[builder]
pub fn Axn1(
    mt: &MortTableConfig,
    i: f64,
    x: u32,
    n: u32,
    #[builder(default = 0)] t: u32,
    #[builder(default = 1)] moment: u32,
    entry_age: Option<u32>,
    #[builder(default = true)] validate: bool,
) -> RSLifeResult<f64> {
    // Using Exn to calculate Axn1
    let built = Exn()
        .mt(mt)
        .i(i)
        .x(x)
        .n(n)
        .t(t)
        .moment(moment)
        .validate(validate);

    match entry_age {
        Some(age) => built.entry_age(age).call(),
        None => built.call(),
    }
}

/// Immediate term life insurance
///
/// Present value of $1 paid at the moment of death, provided death occurs within n years after time t.
/// This is a fundamental building block for term life insurance calculations.
///
/// # Formula
/// ```text
/// ₜ|A¹ₓ:ₙ̅⁽ᵐ⁾ = ₜpₓ · Σₖ₌₀^{mn-1} [v⁽ᵏ⁺¹⁾/ᵐ · ₖ/ₘ|₁/ₘqₓ₊ₜ]
/// ```
/// where:
/// - `v = 1/(1+i)` is the discount factor
/// - `ₜpₓ` is the probability of surviving from age x to age x+t
/// - `ₖ/ₘ|₁/ₘqₓ₊ₜ` is the probability of dying in the (k/m)-th fraction of a year after age x+t
/// - `n` is the term of the insurance
/// - `t` is the deferral period (default 0)
/// - `m` is the number of payments per year (default 1, i.e., annual)
/// - `moment` is the moment to calculate (default 1, i.e., mean)
/// - `entry_age` is the age at which the insured enters the policy (default None, uses ultimate table)
///
/// # Examples
///
/// ## Basic Term Life Insurance
/// ```rust
/// # use rslife::prelude::*;
/// # let mort_data = MortData::from_soa_url_id(1704)?;
/// # let config = MortTableConfig::builder()
/// #     .data(mort_data)
/// #     .radix(100_000)
/// #     .pct(1.0)
/// #     .assumption(AssumptionEnum::UDD)
/// #     .build()?;
///
/// // 10-year term insurance for age 35
/// let term_life = Ax1n()
///     .mt(&config)
///     .i(0.03)    // 3% interest rate
///     .x(35)      // age 35
///     .n(10)      // 10-year term
///     .call()?;
///
/// println!("10-year term insurance: {:.6}", term_life);
/// # RSLifeResult::Ok(())
/// ```
///
/// ## Deferred Term Life Insurance
/// ```rust
/// # use rslife::prelude::*;
/// # let mort_data = MortData::from_soa_url_id(1704)?;
/// # let config = MortTableConfig::builder()
/// #     .data(mort_data)
/// #     .radix(100_000)
/// #     .pct(1.0)
/// #     .assumption(AssumptionEnum::UDD)
/// #     .build()?;
///
/// // 20-year term insurance deferred 5 years for age 40
/// let deferred_term = Ax1n()
///     .mt(&config)
///     .i(0.04)
///     .x(40)
///     .n(20)
///     .t(5)  // 5-year deferral
///     .call()?;
///
/// println!("Deferred term insurance: {:.6}", deferred_term);
/// # RSLifeResult::Ok(())
/// ```
///
/// ## Higher Moments
/// ```rust
/// # use rslife::prelude::*;
/// # let mort_data = MortData::from_soa_url_id(1704)?;
/// # let config = MortTableConfig::builder()
/// #     .data(mort_data)
/// #     .radix(100_000)
/// #     .pct(1.0)
/// #     .assumption(AssumptionEnum::UDD)
/// #     .build()?;
///
/// // Second moment for variance calculations
/// let second_moment = Ax1n()
///     .mt(&config)
///     .i(0.03)
///     .x(45)
///     .n(15)
///     .moment(2)  // Second moment
///     .call()?;
///
/// println!("Second moment: {:.6}", second_moment);
/// # RSLifeResult::Ok(())
/// ```
#[builder]
pub fn Ax1n(
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
    benefit_procedure(
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
    )
}

/// Immediate whole life insurance
///
/// Present value of $1 paid at the moment of death, provided death occurs after time t.
/// This is a fundamental building block for life insurance calculations.
///
/// # Formula
/// ```text
/// ₜ|Aₓ⁽ᵐ⁾ = ₜEₓ · Σₖ₌₀^∞ [v⁽ᵏ⁺¹⁾/ᵐ · ₖ/ₘ|₁/ₘqₓ₊ₜ]
/// ```
/// where:
/// - `v = 1/(1+i)` is the discount factor
/// - `ₜEₓ` is the probability of surviving from age x to age x+t
/// - `ₖ/ₘ|₁/ₘqₓ₊ₜ` is the probability of dying in the (k/m)-th fraction of a year after age x+t
/// - `t` is the deferral period (default 0)
/// - `m` is the number of payments per year (default 1, i.e., annual)
/// - `moment` is the moment to calculate (default 1, i.e., mean)
/// - `entry_age` is the age at which the insured enters the policy (default None, uses ultimate table)
///
/// # Examples
///
/// ## Basic Whole Life Insurance
/// ```rust
/// # use rslife::prelude::*;
/// # let mort_data = MortData::from_soa_url_id(1704)?;
/// # let config = MortTableConfig::builder()
/// #     .data(mort_data)
/// #     .build()?;
///
/// // Whole life insurance for age 40
/// let whole_life = Ax()
///     .mt(&config)
///     .i(0.03)    // 3% interest rate
///     .x(40)      // age 40
///     .call()?;
///
/// println!("Whole life insurance: {:.6}", whole_life);
/// # RSLifeResult::Ok(())
/// ```
///
/// ## Deferred Whole Life Insurance
/// ```rust
/// # use rslife::prelude::*;
/// # let mort_data = MortData::from_soa_url_id(1704)?;
/// # let config = MortTableConfig::builder()
/// #     .data(mort_data)
/// #     .build()?;
///
/// // Whole life insurance deferred 10 years for age 35
/// let deferred_whole_life = Ax()
///     .mt(&config)
///     .i(0.04)
///     .x(35)
///     .t(10)
///     .call()?;
///
/// println!("Deferred whole life insurance: {:.6}", deferred_whole_life);
/// # RSLifeResult::Ok(())
/// ```
///
/// ## Higher Moments
/// ```rust
/// # use rslife::prelude::*;
/// # let mort_data = MortData::from_soa_url_id(1704)?;
/// # let config = MortTableConfig::builder()
/// #     .data(mort_data)
/// #     .radix(100_000)
/// #     .pct(1.0)
/// #     .assumption(AssumptionEnum::UDD)
/// #     .build()?;
///
/// // Second moment for variance calculations
/// let second_moment = Ax()
///     .mt(&config)
///     .i(0.03)
///     .x(50)
///     .moment(2)
///     .call()?;
///
/// println!("Second moment: {:.6}", second_moment);
/// # RSLifeResult::Ok(())
/// ```
#[builder]
pub fn Ax(
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
    benefit_procedure(
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
    )
}

/// Immediate endowment insurance
///
/// Present value of $1 paid at the moment of death (if death occurs within n years after time t), or at the end of n years if the insured survives.
/// This is a fundamental building block for endowment insurance calculations.
///
/// # Formula
/// ```text
/// ₜ|Aₓ:ₙ̅⁽ᵐ⁾ = ₜ|A¹ₓ:ₙ̅⁽ᵐ⁾ + ₜ|ₙEₓ
/// ```
/// where:
/// - `ₜ|A¹ₓ:ₙ̅⁽ᵐ⁾` is the present value of $1 paid at death if death occurs within n years after time t (see `Ax1n`)
/// - `ₜ|ₙEₓ` is the present value of $1 paid at the end of n years if the insured survives (see `Exn`)
/// - `v = 1/(1+i)` is the discount factor
/// - `t` is the deferral period (default 0)
/// - `n` is the term of the insurance
/// - `m` is the number of payments per year (default 1, i.e., annual)
/// - `moment` is the moment to calculate (default 1, i.e., mean)
/// - `entry_age` is the age at which the insured enters the policy (default None, uses ultimate table)
///
/// # Examples
///
/// ## Basic Endowment Insurance
/// ```rust
/// # use rslife::prelude::*;
/// # let mort_data = MortData::from_soa_url_id(1704)?;
/// # let config = MortTableConfig::builder().data(mort_data).build()?;
///
/// // 10-year endowment insurance for age 30
/// let endowment = Axn()
///     .mt(&config)
///     .i(0.03)    // 3% interest rate
///     .x(30)      // age 30
///     .n(10)      // 10-year term
///     .call()?;
///
/// println!("10-year endowment insurance: {:.6}", endowment);
/// # RSLifeResult::Ok(())
/// ```
///
/// ## Deferred Endowment Insurance
/// ```rust
/// # use rslife::prelude::*;
/// # let mort_data = MortData::from_soa_url_id(1704)?;
/// # let config = MortTableConfig::builder().data(mort_data).build()?;
///
/// // 20-year endowment insurance deferred 5 years for age 40
/// let deferred_endowment = Axn()
///     .mt(&config)
///     .i(0.04)
///     .x(40)
///     .n(20)
///     .t(5)  // 5-year deferral
///     .call()?;
///
/// println!("Deferred endowment insurance: {:.6}", deferred_endowment);
/// # RSLifeResult::Ok(())
/// ```
///
/// ## Higher Moments
/// ```rust
/// # use rslife::prelude::*;
/// # let mort_data = MortData::from_soa_url_id(1704)?;
/// # let config = MortTableConfig::builder().data(mort_data).build()?;
///
/// // Second moment for variance calculations
/// let second_moment = Axn()
///     .mt(&config)
///     .i(0.03)
///     .x(45)
///     .n(15)
///     .moment(2)  // Second moment
///     .call()?;
///
/// println!("Second moment: {:.6}", second_moment);
/// # RSLifeResult::Ok(())
/// ```
#[builder]
pub fn Axn(
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
    // Decide if we need to use entry_age or not
    // Prebuilt to reduce procedure repeated twice
    let mt = get_new_config_with_selected_table(mt, entry_age)?;

    // ₜ|Aₓ:ₙ̅⁽ᵐ⁾ = ₜ|A¹ₓ:ₙ̅⁽ᵐ⁾ + ₜ|ₙEₓ
    let term = Ax1n()
        .mt(&mt)
        .i(i)
        .x(x)
        .n(n)
        .t(t)
        .m(m)
        .moment(moment)
        .validate(validate)
        .call()?;

    let pure_endowment = Exn()
        .mt(&mt)
        .i(i)
        .x(x)
        .n(n)
        .t(t)
        .moment(moment)
        .validate(validate)
        .call()?;

    Ok(term + pure_endowment)
}

//-----------------Increasing------------------

/// Immediate increasing term
///
/// Present value of a benefit increasing by 1 each year, paid at the moment of death, provided death occurs within n years after time t.
/// For example, for m=12, k=0..11 the death benefit is 1/m, k=12..23 the benefit is 2/m, etc.
///
/// # Formula
/// ```text
/// ₜ|(IA)¹ₓ:ₙ̅⁽ᵐ⁾ = ₜEₓ · Σₖ₌₀^{mn-1} [v⁽ᵏ⁺¹⁾/ᵐ · ₖ/ₘ|₁/ₘqₓ₊ₜ · ((k // m) + 1) / m]
/// ```
/// where:
/// - `v = 1/(1+i)` is the discount factor
/// - `ₜEₓ` is the probability of surviving from age x to age x+t
/// - `ₖ/ₘ|₁/ₘqₓ₊ₜ` is the probability of dying in the (k/m)-th fraction of a year after age x+t
/// - `n` is the term of the insurance
/// - `t` is the deferral period (default 0)
/// - `m` is the number of payments per year (default 1, i.e., annual)
/// - `moment` is the moment to calculate (default 1, i.e., mean)
/// - `entry_age` is the age at which the insured enters the policy (default None, uses ultimate table)
///
/// # Examples
///
/// ## Basic Increasing Term
/// ```rust
/// # use rslife::prelude::*;
/// # let mort_data = MortData::from_soa_url_id(1704)?;
/// # let config = MortTableConfig::builder()
/// #     .data(mort_data)
/// #     .radix(100_000)
/// #     .pct(1.0)
/// #     .assumption(AssumptionEnum::UDD)
/// #     .build()?;
/// let inc_term = IAx1n().mt(&config).i(0.03).x(40).n(10).call()?;
/// println!("Increasing term: {:.6}", inc_term);
/// # RSLifeResult::Ok(())
/// ```
#[builder]
pub fn IAx1n(
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
    benefit_procedure(
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
    )
}

/// Immediate increasing whole life
///
/// Present value of a benefit increasing by 1 each year, paid at the moment of death, provided death occurs after time t.
/// For example, for m=12, k=0..11 the death benefit is 1/m, k=12..23 the benefit is 2/m, etc.
///
/// # Formula
/// ```text
/// ₜ|(IA)ₓ⁽ᵐ⁾ = ₜEₓ · Σₖ₌₀^∞ [v⁽ᵏ⁺¹⁾/ᵐ · ₖ/ₘ|₁/ₘqₓ₊ₜ · ((k // m) + 1) / m]
/// ```
/// where:
/// - `v = 1/(1+i)` is the discount factor
/// - `ₜEₓ` is the probability of surviving from age x to age x+t
/// - `ₖ/ₘ|₁/ₘqₓ₊ₜ` is the probability of dying in the (k/m)-th fraction of a year after age x+t
/// - `m` is the number of payments per year (default 1)
/// - `t` is the deferral period (default 0)
/// - `moment` is the moment to calculate (default 1)
/// - `entry_age` is the age at which the insured enters the policy (default None)
///
/// # Examples
///
/// ## Basic Increasing Whole Life
/// ```rust
/// # use rslife::prelude::*;
/// # let mort_data = MortData::from_soa_url_id(1704)?;
/// # let config = MortTableConfig::builder().data(mort_data).build()?;
/// let inc_whole_life = IAx().mt(&config).i(0.03).x(40).call()?;
/// println!("Increasing whole life: {:.6}", inc_whole_life);
/// # RSLifeResult::Ok(())
/// ```
#[builder]
pub fn IAx(
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
    benefit_procedure(
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
    )
}

/// Immediate increasing endowment insurance
///
/// Present value of a benefit increasing by 1 each year, paid at the moment of death (if death occurs within n years after time t), or at the end of n years if the insured survives.
///
/// # Formula
/// ```text
/// ₜ|IAₓ:ₙ̅ = ₜ|IA¹ₓ:ₙ̅ + ₜ|ₙEₓ
/// ```
/// where:
/// - `ₜ|IA¹ₓ:ₙ̅` is the present value of increasing benefit paid at death within n years (see IAx1n)
/// - `ₜ|ₙEₓ` is the present value of $1 paid at the end of n years if the insured survives (see Exn)
/// - `v = 1/(1+i)` is the discount factor
/// - `t` is the deferral period (default 0)
/// - `n` is the term of the insurance
/// - `m` is the number of payments per year (default 1)
/// - `moment` is the moment to calculate (default 1, i.e., mean)
/// - `entry_age` is the age at which the insured enters the policy (default None, uses ultimate table)
///
/// # Examples
///
/// ## Basic Increasing Endowment
/// ```rust
/// # use rslife::prelude::*;
/// # let mort_data = MortData::from_soa_url_id(1704)?;
/// # let config = MortTableConfig::builder().data(mort_data).build()?;
/// let inc_endowment = IAxn().mt(&config).i(0.03).x(40).n(10).call()?;
/// println!("Increasing endowment: {:.6}", inc_endowment);
/// # RSLifeResult::Ok(())
/// ```
#[builder]
pub fn IAxn(
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
    // Decide if we need to use entry_age or not
    // Prebuilt to reduce procedure repeated twice
    let mt = get_new_config_with_selected_table(mt, entry_age)?;

    let term = IAx1n()
        .mt(&mt)
        .i(i)
        .x(x)
        .n(n)
        .t(t)
        .m(m)
        .moment(moment)
        .validate(validate)
        .call()?;

    let pure_endowment = Exn()
        .mt(&mt)
        .i(i)
        .x(x)
        .n(n)
        .t(t)
        .moment(moment)
        .validate(validate)
        .call()?;

    Ok(term + pure_endowment)
}

//-----------------Decreasing------------------
// Note: There should starting amount hence DAₓ is not applicable

/// Immediate decreasing term
///
/// Present value of a benefit decreasing by 1 each year, paid at the moment of death, provided death occurs within n years after time t.
/// For example, for m=12, k=0..11 the death benefit is n/m, k=12..23 the benefit is (n-1)/m, etc.
///
/// # Formula
/// ```text
/// ₜ|(DA¹)ₓ:ₙ̅ = ₜEₓ · Σₖ₌₀^{mn-1} [v⁽ᵏ⁺¹⁾/ᵐ · ₖ/ₘ|₁/ₘqₓ₊ₜ · (n - (k // m)) / m]
/// ```
/// where:
/// - `v = 1/(1+i)` is the discount factor
/// - `ₜEₓ` is the probability of surviving from age x to age x+t
/// - `ₖ/ₘ|₁/ₘqₓ₊ₜ` is the probability of dying in the (k/m)-th fraction of a year after age x+t
/// - `n` is the term of the insurance
/// - `m` is the number of payments per year (default 1)
/// - `t` is the deferral period (default 0)
/// - `moment` is the moment to calculate (default 1)
/// - `entry_age` is the age at which the insured enters the policy (default None)
///
/// # Examples
///
/// ## Basic Decreasing Term
/// ```rust
/// # use rslife::prelude::*;
/// # let mort_data = MortData::from_soa_url_id(1704)?;
/// # let config = MortTableConfig::builder().data(mort_data).build()?;
/// let dec_term = DAx1n().mt(&config).i(0.03).x(40).n(10).call()?;
/// println!("Decreasing term: {:.6}", dec_term);
/// # RSLifeResult::Ok(())
/// ```
#[builder]
pub fn DAx1n(
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
    benefit_procedure(
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
    )
}

/// Immediate decreasing endowment insurance
///
/// Present value of a benefit decreasing by 1 each year, paid at the moment of death (if death occurs within n years after time t), or at the end of n years if the insured survives.
///
/// # Formula
/// ```text
/// ₜ|(DA)ₓ:ₙ̅ = ₜ|(DA)¹ₓ:ₙ̅ + ₜ|ₙEₓ
/// ```
/// where:
/// - `ₜ|(DA)¹ₓ:ₙ̅` is the present value of decreasing benefit paid at death within n years (see DAx1n)
/// - `ₜ|ₙEₓ` is the present value of $1 paid at the end of n years if the insured survives (see Exn)
/// - `v = 1/(1+i)` is the discount factor
/// - `t` is the deferral period (default 0)
/// - `n` is the term of the insurance
/// - `m` is the number of payments per year (default 1)
/// - `moment` is the moment to calculate (default 1, i.e., mean)
/// - `entry_age` is the age at which the insured enters the policy (default None, uses ultimate table)
///
/// # Examples
///
/// ## Basic Decreasing Endowment
/// ```rust
/// # use rslife::prelude::*;
/// # let mort_data = MortData::from_soa_url_id(1704)?;
/// # let config = MortTableConfig::builder().data(mort_data).build()?;
/// let dec_endowment = DAxn().mt(&config).i(0.03).x(40).n(10).call()?;
/// println!("Decreasing endowment: {:.6}", dec_endowment);
/// # RSLifeResult::Ok(())
/// ```
#[builder]
pub fn DAxn(
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
    // Decide if we need to use entry_age or not
    // Prebuilt to reduce procedure repeated twice
    let mt = get_new_config_with_selected_table(mt, entry_age)?;

    let term = DAx1n()
        .mt(&mt)
        .i(i)
        .x(x)
        .n(n)
        .t(t)
        .m(m)
        .moment(moment)
        .validate(validate)
        .call()?;

    let pure_endowment = Exn()
        .mt(&mt)
        .i(i)
        .x(x)
        .n(n)
        .t(t)
        .moment(moment)
        .validate(validate)
        .call()?;

    Ok(term + pure_endowment)
}

//-----------------Geometric increasing------------------

/// Immediate geometric whole life
///
/// Present value of a benefit growing geometrically at rate g each year, paid at the moment of death, provided death occurs after time t.
/// The effective interest rate is adjusted: i' = (1+i)/(1+g) - 1.
///
/// # Formula
/// ```text
/// Aₓ⁽ᵍ⁾ = Ax(i')
/// where i' = (1+i)/(1+g) - 1
/// ```
/// - `g` is the geometric growth rate of the benefit
/// - All other parameters as in Ax
///
/// # Examples
///
/// ## Basic Geometric Whole Life
/// ```rust
/// # use rslife::prelude::*;
/// # let mort_data = MortData::from_soa_url_id(1704)?;
/// # let config = MortTableConfig::builder().data(mort_data).build()?;
/// let geom_whole_life = gAx().mt(&config).i(0.03).x(40).g(0.02).call()?;
/// println!("Geometric whole life: {:.6}", geom_whole_life);
/// # RSLifeResult::Ok(())
/// ```
#[builder]
pub fn gAx(
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

    let built = Ax()
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

/// Immediate geometric n-year term
///
/// Present value of a benefit growing geometrically at rate g each year, paid at the moment of death, provided death occurs within n years after time t.
/// The effective interest rate is adjusted: i' = (1+i)/(1+g) - 1.
///
/// # Formula
/// ```text
/// A¹ₓ:ₙ̅⁽ᵍ⁾ = Ax1n(i')
/// where i' = (1+i)/(1+g) - 1
/// ```
/// - `g` is the geometric growth rate of the benefit
/// - All other parameters as in Ax1n
///
/// # Examples
///
/// ## Basic Geometric Term
/// ```rust
/// # use rslife::prelude::*;
/// # let mort_data = MortData::from_soa_url_id(1704)?;
/// # let config = MortTableConfig::builder().data(mort_data).build()?;
/// let geom_term = gAx1n().mt(&config).i(0.03).x(40).n(10).g(0.02).call()?;
/// println!("Geometric term: {:.6}", geom_term);
/// # RSLifeResult::Ok(())
/// ```
#[builder]
pub fn gAx1n(
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

    let built = Ax1n()
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

/// Immediate geometric n-year endowment
///
/// Present value of a benefit growing geometrically at rate g each year, paid at the moment of death (if death occurs within n years after time t), or at the end of n years if the insured survives.
/// The effective interest rate is adjusted: i' = (1+i)/(1+g) - 1.
///
/// # Formula
/// ```text
/// Aₓ:ₙ̅⁽ᵍ⁾ = A¹ₓ:ₙ̅⁽ᵍ⁾ + Aₓ:ₙ̅¹⁽ᵍ⁾
/// where i' = (1+i)/(1+g) - 1
/// ```
/// - `g` is the geometric growth rate of the benefit
/// - All other parameters as in gAx1n and gExn
///
/// # Examples
///
/// ## Basic Geometric Endowment
/// ```rust
/// # use rslife::prelude::*;
/// # let mort_data = MortData::from_soa_url_id(1704)?;
/// # let config = MortTableConfig::builder().data(mort_data).build()?;
/// let geom_endowment = gAxn().mt(&config).i(0.03).x(40).n(10).g(0.02).call()?;
/// println!("Geometric endowment: {:.6}", geom_endowment);
/// # RSLifeResult::Ok(())
/// ```
#[builder]
pub fn gAxn(
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

    let built = Axn()
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

fn benefit_procedure(
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
    let k_arr: Vec<f64> = (0..n * m).map(|k| k as f64).collect();

    // Convert parameters to f64 for calculations
    let v = 1.0 / (1.0 + i);
    let x = f64::from(x);
    let n = f64::from(n);
    let t = f64::from(t);
    let m = f64::from(m);
    let moment = f64::from(moment);

    // ----------Discount factor----------
    let discount_factors: Vec<f64> = k_arr
        .iter()
        .map(|&k| v.powf(moment * ((k + 1.0) / m)))
        .collect();

    // ----------Probabilities vectors----------
    //ₖ/ₘ|₁/ₘqₓ₊ₜ
    let probabilities: Vec<f64> = k_arr
        .iter()
        .map(|&k| {
            tqx()
                .mt(&mt)
                .x(x + t)
                .t(1.0 / m)
                .k(k / m)
                .validate(false)
                .call()
                .unwrap_or(0.0)
        })
        .collect();

    // ----------Benefit vector----------
    let amounts: Vec<f64> = match structure {
        CashFlowStructure::Flat => vec![1.0; (n * m) as usize],
        CashFlowStructure::Increasing => k_arr.iter().map(|&k| (k / m).floor() + 1.0).collect(),
        CashFlowStructure::Decreasing => k_arr.iter().map(|&k| n - (k / m).floor()).collect(),
    };

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

    // #[test]
    // fn test_fn_A1xn_cont_theory() {
    //     let am92 = MortData::from_ifoa_url_id("AM92").expect("Failed to load AM92 selected table");
    //     let mt = MortTableConfig::builder().data(am92).build().unwrap();
    //     let regular = Ax1n().mt(&mt).i(0.05).x(70).n(20).call().unwrap();
    //     let cont = Ax1n()
    //         .mt(&mt)
    //         .i(0.05)
    //         .x(70)
    //         .n(20)
    //         .t(1)
    //         .m(1000)
    //         .call()
    //         .unwrap();
    //     let expected = 1.05_f64.powf(0.5);
    //     assert_abs_diff_eq!(cont / regular, expected, epsilon = 1e-3);
    // }

    #[test]
    fn test_fn_A1xn_n_is_0() {
        // Edge case where n = 0 should return 0
        let am92 = MortData::from_ifoa_url_id("AM92").expect("Failed to load AM92 selected table");
        let mt = MortTableConfig::builder().data(am92).build().unwrap();
        let ans = Ax1n().mt(&mt).i(0.05).x(70).n(0).call().unwrap();
        let expected = 0.0;
        assert_abs_diff_eq!(ans, expected, epsilon = 1e-6);
    }

    #[test]
    fn test_fn_Exn_n_is_0() {
        // Edge case where n = 0 should return 1
        let am92 = MortData::from_ifoa_url_id("AM92").expect("Failed to load AM92 selected table");
        let mt = MortTableConfig::builder().data(am92).build().unwrap();
        let ans = Exn().mt(&mt).i(0.05).x(70).n(0).call().unwrap();
        let expected = 1.0;
        assert_abs_diff_eq!(ans, expected, epsilon = 1e-6);
    }

    #[test]
    fn test_fn_Ax_benefit_01() {
        // From Formulae and Tables for Actuarial Examinations AM92 table
        let am92 = MortData::from_ifoa_url_id("AM92").expect("Failed to load AM92 selected table");
        let mt = MortTableConfig::builder().data(am92).build().unwrap();

        let age = [17, 36, 71, 90];

        // Value from examination material
        let expected_ultimate = [0.10127, 0.19933, 0.61548, 0.84196];
        let expected_selected = [0.10108, 0.19921, 0.61093, 0.82362];
        let expected_second_moment_ultimate = [0.01716, 0.05207, 0.40686, 0.71874];
        let expected_second_moment_selected = [0.01696, 0.05193, 0.40012, 0.68768];

        for (i, &a) in age.iter().enumerate() {
            let ultimate = Ax().mt(&mt).i(0.04).x(a).call().unwrap();
            assert_abs_diff_eq!(ultimate, expected_ultimate[i], epsilon = 1e-5);

            let selected = Ax().mt(&mt).i(0.04).x(a).entry_age(a).call().unwrap();
            assert_abs_diff_eq!(selected, expected_selected[i], epsilon = 1e-5);

            let second_moment_ultimate = Ax().mt(&mt).i(0.04).x(a).moment(2).call().unwrap();
            assert_abs_diff_eq!(
                second_moment_ultimate,
                expected_second_moment_ultimate[i],
                epsilon = 1e-5
            );

            let second_moment_selected = Ax()
                .mt(&mt)
                .i(0.04)
                .x(a)
                .moment(2)
                .entry_age(a)
                .call()
                .unwrap();
            assert_abs_diff_eq!(
                second_moment_selected,
                expected_second_moment_selected[i],
                epsilon = 1e-5
            );
        }
    }

    #[test]
    fn test_fn_Ax_benefit_02() {
        // From Standard Ultimate Life Table
        let sult =
            MortData::from_soa_custom("SULT").expect("Failed to load Standard Ultimate Life Table");
        let mt = MortTableConfig::builder().data(sult).build().unwrap();

        let age = [20, 27, 63, 84, 100];

        // Values from examination material
        let expected = [0.04922, 0.06725, 0.32785, 0.65990, 0.87068];
        let expected_second_moment = [0.00580, 0.00900, 0.13421, 0.46137, 0.76427];

        for (i, &a) in age.iter().enumerate() {
            let ultimate = Ax().mt(&mt).i(0.05).x(a).call().unwrap();
            assert_abs_diff_eq!(ultimate, expected[i], epsilon = 1e-5);

            let second_moment = Ax().mt(&mt).i(0.05).x(a).moment(2).call().unwrap();
            assert_abs_diff_eq!(second_moment, expected_second_moment[i], epsilon = 1e-5);
        }
    }

    #[test]
    fn test_fn_Axn_benefit_01() {
        // From Formulae and Tables for Actuarial Examinations
        let am92 = MortData::from_ifoa_url_id("AM92").expect("Failed to load AM92 selected table");
        let mt = MortTableConfig::builder().data(am92).build().unwrap();

        let age = [17, 29, 43, 59];

        // Values from examination material
        let expected_ultimate = [0.19475, 0.30525, 0.52073, 0.96154];
        let expected_selected = [0.19459, 0.30515, 0.52061, 0.96154];

        for (i, &a) in age.iter().enumerate() {
            let ultimate = Axn().mt(&mt).i(0.04).x(a).n(60 - a).call().unwrap();
            assert_abs_diff_eq!(ultimate, expected_ultimate[i], epsilon = 1e-5);

            let selected = Axn()
                .mt(&mt)
                .i(0.04)
                .x(a)
                .n(60 - a)
                .entry_age(a)
                .call()
                .unwrap();
            assert_abs_diff_eq!(selected, expected_selected[i], epsilon = 1e-5);
        }
    }

    #[test]
    fn test_fn_Axn_benefit_02() {
        // From Standard Ultimate Life Table
        let sult =
            MortData::from_soa_custom("SULT").expect("Failed to load Standard Ultimate Life Table");
        let mt = MortTableConfig::builder().data(sult).build().unwrap();

        let age = [20, 34, 57, 91, 100];

        // Values from examination material
        let expected_n10 = [0.61433, 0.61460, 0.61914, 0.77609, 0.87078];
        let expected_n20 = [0.37829, 0.37961, 0.40118, 0.76735, 0.87068];

        for (i, &a) in age.iter().enumerate() {
            let n10 = Axn().mt(&mt).i(0.05).x(a).n(10).call().unwrap();
            assert_abs_diff_eq!(n10, expected_n10[i], epsilon = 1e-5);

            let n20 = Axn().mt(&mt).i(0.05).x(a).n(20).call().unwrap();
            assert_abs_diff_eq!(n20, expected_n20[i], epsilon = 1e-5);
        }
    }

    #[test]
    fn test_fn_IAx_benefit_01() {
        // From Formulae and Tables for Actuarial Examinations
        let am92 = MortData::from_ifoa_url_id("AM92").expect("Failed to load AM92 selected table");
        let mt = MortTableConfig::builder().data(am92).build().unwrap();
        let ans = IAx().mt(&mt).i(0.06).x(110).call().unwrap();
        let expected = 1.42096;
        // During the testing, accuracy was drops
        // to 1e-4 at age 110
        // to 1e-3 at age 115
        // to 0.0 at age 118
        // For a reasonable age range from 0 to 110 the result is matchedx
        assert_abs_diff_eq!(ans, expected, epsilon = 1e-4);
    }

    #[test]
    fn test_fn_IAx1n_benefit_01() {
        // From April 2025 CM1 Question 3 part i
        let am92 = MortData::from_ifoa_url_id("AM92").expect("Failed to load AM92 selected table");
        let mt = MortTableConfig::builder().data(am92).build().unwrap();
        let ans = IAx1n().mt(&mt).i(0.04).x(50).n(10).call().unwrap();
        let expected = 8.55929 - (882.85 / 1366.61) * (8.36234 + 10.0 * 0.45640);
        assert_abs_diff_eq!(ans, expected, epsilon = 1e-4);
    }

    #[test]
    fn test_debug_am92_structure() {
        let am92 = MortData::from_ifoa_url_id("AM92").expect("Failed to load AM92 selected table");
        let mt = MortTableConfig::builder().data(am92).build().unwrap();
        // Debug output
        println!("DataFrame shape: {:?}", mt.data.dataframe.shape());
        println!("Column names: {:?}", mt.data.dataframe.get_column_names());
        println!("Min age: {:?}", mt.min_age());
        println!("Max age: {:?}", mt.max_age());

        // Print first few rows
        println!("First 5 rows:\n{}", mt.data.dataframe.head(Some(5)));

        // Check if age column exists and has data
        if let Ok(age_col) = mt.data.dataframe.column("age") {
            println!("Age column type: {:?}", age_col.dtype());
            if let Ok(age_series) = age_col.u32() {
                let values: Vec<_> = age_series.iter().take(10).collect();
                println!("First 10 age values: {values:?}");
            }
        }

        assert!(
            mt.min_age().unwrap() > 0,
            "Min age should be greater than 0"
        );
        assert!(
            mt.max_age().unwrap() > mt.min_age().unwrap(),
            "Max age should be greater than min age"
        );
    }

    #[test]
    fn test_debug_selected_table_processing() {
        // Create a MortTableConfig with AM92 data
        let am92 = MortData::from_ifoa_url_id("AM92").expect("Failed to load AM92 selected table");
        let mt = MortTableConfig::builder().data(am92).build().unwrap();

        println!("Original table:");
        println!("  Shape: {:?}", mt.data.dataframe.shape());
        println!("  Min age: {:?}, Max age: {:?}", mt.min_age(), mt.max_age());
        println!(
            "  Min duration: {:?}, Max duration: {:?}",
            mt.min_duration(),
            mt.max_duration()
        );

        // Test with entry_age = 70 (same as failing test)
        let selected_mt = get_new_config_with_selected_table(&mt, Some(70)).unwrap();

        println!("Selected table (entry_age=70):");
        println!("  Shape: {:?}", selected_mt.data.dataframe.shape());
        println!(
            "  Min age: {:?}, Max age: {:?}",
            selected_mt.min_age(),
            selected_mt.max_age()
        );
        println!(
            "  Column names: {:?}",
            selected_mt.data.dataframe.get_column_names()
        );
        println!(
            "  First 10 rows:\n{}",
            selected_mt.data.dataframe.head(Some(10))
        );

        assert!(
            selected_mt.min_age().unwrap() > 0,
            "Selected table min age should be greater than 0"
        );
        assert!(
            selected_mt.max_age().unwrap() >= selected_mt.min_age().unwrap(),
            "Selected table max age should be >= min age"
        );
    }
}
