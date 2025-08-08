#![allow(non_snake_case)]

use crate::RSLifeResult;
use crate::helpers::get_new_config_with_selected_table;
use crate::mt_config::MortTableConfig;
use crate::params::SingleLifeParams;
use bon::builder;

/// Commutation function Cₓ
///
/// Present value commutation function used in actuarial calculations.
///
/// # Formula
/// ```text
/// Cₓ = vˣ⁺¹ · dₓ
/// ```
/// where:
/// - `v = 1/(1+i)` is the discount factor
/// - `dₓ` is the number of deaths at age x
/// - `x` is the age
/// - `i` is the interest rate
/// - `entry_age` is the age at which the insured enters the policy (optional)
///
/// # Example
/// ```rust
/// # use rslife::prelude::*;
/// # let mort_data = MortData::from_ifoa_url_id("AM92")?;
/// # let config = MortTableConfig::builder().data(mort_data).build()?;
/// let cx = Cx().mt(&config).i(0.04).x(23).call()?;
/// println!("Cₓ: {:.2}", cx);
/// # RSLifeResult::Ok(())
/// ```
#[builder]
pub fn Cx(
    mt: &MortTableConfig,
    i: f64,
    x: u32,
    entry_age: Option<u32>,
    #[builder(default = true)] validate: bool,
) -> RSLifeResult<f64> {
    if validate {
        let params = SingleLifeParams {
            mt: mt.clone(),
            i,
            x,
            n: 0,      // Not used
            t: 0,      // Not used
            m: 1,      // Not used
            moment: 1, // Not used
            entry_age,
        };

        params
            .validate_all()
            .map_err(|err| Box::new(err) as Box<dyn std::error::Error>)?;
    }

    // As provided - no default
    let x = x as f64;
    let v = 1.0 / (1.0 + i);

    // Cₓ = vˣ⁺¹ * dₓ
    let discount_factor = v.powf(x + 1.0);
    let built = mt.dx().x(x as u32).validate(false);
    let dx = match entry_age {
        Some(age) => built.entry_age(age).call()?,
        None => built.call()?,
    };
    let result = discount_factor * dx;
    Ok(result)
}

/// Commutation function Dₓ
///
/// Discounted number of lives at age x.
///
/// # Formula
/// ```text
/// Dₓ = vˣ · lₓ
/// ```
/// where:
/// - `v = 1/(1+i)` is the discount factor
/// - `lₓ` is the number of lives at age x
/// - `x` is the age
/// - `i` is the interest rate
/// - `entry_age` is the age at which the insured enters the policy (optional)
///
/// # Example
/// ```rust
/// # use rslife::prelude::*;
/// # let mort_data = MortData::from_ifoa_url_id("AM92")?;
/// # let config = MortTableConfig::builder().data(mort_data).build()?;
/// let dx = Dx().mt(&config).i(0.04).x(47).entry_age(46).call()?;
/// println!("Dₓ: {:.2}", dx);
/// # RSLifeResult::Ok(())
/// ```
#[builder]
pub fn Dx(
    mt: &MortTableConfig,
    i: f64,
    x: u32,
    entry_age: Option<u32>,
    #[builder(default = true)] validate: bool,
) -> RSLifeResult<f64> {
    if validate {
        let params = SingleLifeParams {
            mt: mt.clone(),
            i,
            x,
            n: 0,      // Not used
            t: 0,      // Not used
            m: 1,      // Not used
            moment: 1, // Not used
            entry_age,
        };

        params
            .validate_all()
            .map_err(|err| Box::new(err) as Box<dyn std::error::Error>)?;
    };

    // As provided - no default
    let v = 1.0 / (1.0 + i);
    let x = x as f64;

    // Dₓ = vˣ * lₓ
    let discount_factor = v.powf(x);
    let built = mt.lx().x(x as u32).validate(false);
    let lx = match entry_age {
        Some(age) => built.entry_age(age).call()?,
        None => built.call()?,
    };
    let result = discount_factor * lx;
    Ok(result)
}

/// Commutation function Mₓ
///
/// Sum of Cₖ for k ≥ x, used for present value calculations of life insurance and annuities.
///
/// # Formula
/// ```text
/// Mₓ = Σₖ₌ₓ^∞ Cₖ = Σₖ₌ₓ^∞ (vᵏ⁺¹ · dₖ)
/// ```
/// where:
/// - `v = 1/(1+i)` is the discount factor
/// - `dₖ` is the number of deaths at age k
/// - `x` is the age
/// - `i` is the interest rate
/// - `entry_age` is the age at which the insured enters the policy (optional)
///
/// # Example
/// ```rust
/// # use rslife::prelude::*;
/// # let mort_data = MortData::from_ifoa_url_id("AM92")?;
/// # let config = MortTableConfig::builder().data(mort_data).build()?;
/// let mx = Mx().mt(&config).i(0.04).x(62).entry_age(60).call()?;
/// println!("Mₓ: {:.2}", mx);
/// # RSLifeResult::Ok(())
/// ```
#[builder]
pub fn Mx(
    mt: &MortTableConfig,
    i: f64,
    x: u32,
    entry_age: Option<u32>,
    #[builder(default = true)] validate: bool,
) -> RSLifeResult<f64> {
    if validate {
        let params = SingleLifeParams {
            mt: mt.clone(),
            i,
            x,
            n: 0,      // Not used
            t: 0,      // Not used
            m: 1,      // Not used
            moment: 1, // Not used
            entry_age,
        };

        params
            .validate_all()
            .map_err(|err| Box::new(err) as Box<dyn std::error::Error>)?;
    };

    // Decide if we need to use entry_age or not - Prebuilt to reduce procedure inside loops
    let mt = get_new_config_with_selected_table(mt, entry_age)?;

    // Mₓ = Σₖ₌ₓ^∞ Cₖ
    let max_age = mt.max_age()? as u32;
    let summation = (x..=max_age).fold(0.0, |acc, k| {
        let Cx = Cx().mt(&mt).i(i).x(k).validate(false).call().unwrap_or(0.0);
        acc + Cx
    });

    // Final result
    Ok(summation)
}

/// Commutation function Nₓ
///
/// Sum of Dₖ for k ≥ x, used for present value calculations of annuities.
///
/// # Formula
/// ```text
/// Nₓ = Σₖ₌ₓ^∞ Dₖ = Σₖ₌ₓ^∞ (vᵏ · lₖ)
/// ```
/// where:
/// - `v = 1/(1+i)` is the discount factor
/// - `lₖ` is the number of lives at age k
/// - `x` is the age
/// - `i` is the interest rate
/// - `entry_age` is the age at which the insured enters the policy (optional)
///
/// # Example
/// ```rust
/// # use rslife::prelude::*;
/// # let mort_data = MortData::from_ifoa_url_id("AM92")?;
/// # let config = MortTableConfig::builder().data(mort_data).build()?;
/// let nx = Nx().mt(&config).i(0.04).x(33).entry_age(32).call()?;
/// println!("Nₓ: {:.2}", nx);
/// # RSLifeResult::Ok(())
/// ```
#[builder]
pub fn Nx(
    mt: &MortTableConfig,
    i: f64,
    x: u32,
    entry_age: Option<u32>,
    #[builder(default = true)] validate: bool,
) -> RSLifeResult<f64> {
    if validate {
        let params = SingleLifeParams {
            mt: mt.clone(),
            i,
            x,
            n: 0,      // Not used
            t: 0,      // Not used
            m: 1,      // Not used
            moment: 1, // Not used
            entry_age,
        };

        params
            .validate_all()
            .map_err(|err| Box::new(err) as Box<dyn std::error::Error>)?;
    };

    // Decide if we need to use entry_age or not - Prebuilt to reduce procedure inside loops
    let mt = get_new_config_with_selected_table(mt, entry_age)?;

    // Nₓ = Σₖ₌ₓ^∞ Dₖ
    let max_age = mt.max_age()? as u32;
    let summation = (x..=max_age).fold(0.0, |acc, k| {
        let Dx = Dx().mt(&mt).i(i).x(k).validate(false).call().unwrap_or(0.0);
        acc + Dx
    });

    // Final result
    Ok(summation)
}

/// Commutation function Rₓ
///
/// Double summation of Cₖ for k ≥ x, used for present value calculations of increasing life insurance.
///
/// # Formula
/// ```text
/// Rₓ = Σₖ₌ₓ^∞ Mₖ = Σₖ₌ₓ^∞ (Σⱼ₌ₖ^∞ (vʲ⁺¹ · dⱼ))
/// ```
/// where:
/// - `v = 1/(1+i)` is the discount factor
/// - `dⱼ` is the number of deaths at age j
/// - `x` is the age
/// - `i` is the interest rate
/// - `entry_age` is the age at which the insured enters the policy (optional)
///
/// # Example
/// ```rust
/// # use rslife::prelude::*;
/// # let mort_data = MortData::from_ifoa_url_id("AM92")?;
/// # let config = MortTableConfig::builder().data(mort_data).build()?;
/// let rx = Rx().mt(&config).i(0.04).x(26).entry_age(26).call()?;
/// println!("Rₓ: {:.2}", rx);
/// # RSLifeResult::Ok(())
/// ```
#[builder]
pub fn Rx(
    mt: &MortTableConfig,
    i: f64,
    x: u32,
    entry_age: Option<u32>,
    #[builder(default = true)] validate: bool,
) -> RSLifeResult<f64> {
    if validate {
        let params = SingleLifeParams {
            mt: mt.clone(),
            i,
            x,
            n: 0,      // Not used
            t: 0,      // Not used
            m: 1,      // Not used
            moment: 1, // Not used
            entry_age,
        };

        params
            .validate_all()
            .map_err(|err| Box::new(err) as Box<dyn std::error::Error>)?;
    };

    // Decide if we need to use entry_age or not - Prebuilt to reduce procedure inside loops
    let mt = get_new_config_with_selected_table(mt, entry_age)?;

    // Rₓ = Σₖ₌ₓ^∞ Mₖ
    let max_age = mt.max_age()? as u32;
    let summation = (x..=max_age).fold(0.0, |acc, k| {
        let Mx = Mx().mt(&mt).i(i).x(k).validate(false).call().unwrap_or(0.0);
        acc + Mx
    });

    // Final result
    Ok(summation)
}

/// Commutation function Sₓ
///
/// Double summation of Dₖ for k ≥ x, used for present value calculations of increasing annuities.
///
/// # Formula
/// ```text
/// Sₓ = Σₖ₌ₓ^∞ Nₖ = Σₖ₌ₓ^∞ (Σⱼ₌ₖ^∞ (vʲ · lⱼ))
/// ```
/// where:
/// - `v = 1/(1+i)` is the discount factor
/// - `lⱼ` is the number of lives at age j
/// - `x` is the age
/// - `i` is the interest rate
/// - `entry_age` is the age at which the insured enters the policy (optional)
///
/// # Example
/// ```rust
/// # use rslife::prelude::*;
/// # let mort_data = MortData::from_ifoa_url_id("AM92")?;
/// # let config = MortTableConfig::builder().data(mort_data).build()?;
/// let sx = Sx().mt(&config).i(0.04).x(65).entry_age(65).validate(false).call()?;
/// println!("Sₓ: {:.2}", sx);
/// # RSLifeResult::Ok(())
/// ```
#[builder]
pub fn Sx(
    mt: &MortTableConfig,
    i: f64,
    x: u32,
    entry_age: Option<u32>,
    #[builder(default = true)] validate: bool,
) -> RSLifeResult<f64> {
    if validate {
        let params = SingleLifeParams {
            mt: mt.clone(),
            i,
            x,
            n: 0,      // Not used
            t: 0,      // Not used
            m: 1,      // Not used
            moment: 1, // Not used
            entry_age,
        };

        params
            .validate_all()
            .map_err(|err| Box::new(err) as Box<dyn std::error::Error>)?;
    };

    // Decide if we need to use entry_age or not - Prebuilt to reduce procedure inside loops
    let mt = get_new_config_with_selected_table(mt, entry_age)?;

    // Sₓ = Σₖ₌ₓ^∞ Nₖ
    let max_age = mt.max_age()? as u32;
    let summation = (x..=max_age).fold(0.0, |acc, k| {
        let Nx = Nx().mt(&mt).i(i).x(k).validate(false).call().unwrap_or(0.0);
        acc + Nx
    });

    // Final result
    Ok(summation)
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
    fn test_fn_Cx() {
        // Create MortTableConfig with AM92
        let am92 = MortData::from_ifoa_url_id("AM92").expect("Failed to load AM92 selected table");
        let mt = MortTableConfig::builder()
            .data(am92)
            .radix(10_000)
            .build()
            .unwrap();
        let ans = Cx().mt(&mt).i(0.04).x(23).call().unwrap();
        let expected = 2.21;
        assert_abs_diff_eq!(ans, expected, epsilon = 1e-2);
    }

    #[test]
    fn test_fn_Dx() {
        // Create MortTableConfig with AM92
        let am92 = MortData::from_ifoa_url_id("AM92").expect("Failed to load AM92 selected table");
        let mt = MortTableConfig::builder()
            .data(am92)
            .radix(10_000)
            .build()
            .unwrap();
        let ans = Dx().mt(&mt).i(0.04).x(47).entry_age(46).call().unwrap();
        let expected = 1546.49;
        assert_abs_diff_eq!(ans, expected, epsilon = 1e-2);
    }

    #[test]
    fn test_fn_Mx() {
        // Create MortTableConfig with AM92
        let am92 = MortData::from_ifoa_url_id("AM92").expect("Failed to load AM92 selected table");
        let mt = MortTableConfig::builder()
            .data(am92)
            .radix(10_000)
            .build()
            .unwrap();
        let ans = Mx().mt(&mt).i(0.04).x(62).entry_age(60).call().unwrap();
        let expected = 388.83;
        assert_abs_diff_eq!(ans, expected, epsilon = 1e-2);
    }

    #[test]
    fn test_fn_Nx() {
        // Create MortTableConfig with AM92
        let am92 = MortData::from_ifoa_url_id("AM92").expect("Failed to load AM92 selected table");
        let mt = MortTableConfig::builder()
            .data(am92)
            .radix(10_000)
            .build()
            .unwrap();
        let ans = Nx().mt(&mt).i(0.04).x(33).entry_age(32).call().unwrap();
        let expected = 57987.98;
        assert_abs_diff_eq!(ans, expected, epsilon = 1e-2);
    }

    #[test]
    fn test_fn_Rx() {
        // Create MortTableConfig with AM92
        let am92 = MortData::from_ifoa_url_id("AM92").expect("Failed to load AM92 selected table");
        let mt = MortTableConfig::builder()
            .data(am92)
            .radix(10_000)
            .build()
            .unwrap();
        let ans = Rx().mt(&mt).i(0.04).x(26).entry_age(26).call().unwrap();
        let expected = 23141.58;
        assert_abs_diff_eq!(ans, expected, epsilon = 1e-2);
    }

    #[test]
    fn test_fn_Sx() {
        // Create MortTableConfig with AM92
        let am92 = MortData::from_ifoa_url_id("AM92").expect("Failed to load AM92 selected table");
        let mt = MortTableConfig::builder()
            .data(am92)
            .radix(10_000)
            .build()
            .unwrap();
        let ans = Sx().mt(&mt).i(0.04).x(65).entry_age(65).call().unwrap();
        let expected = 78505.54;
        assert_abs_diff_eq!(ans, expected, epsilon = 1e-2);
    }
}
