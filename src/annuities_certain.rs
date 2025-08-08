use crate::RSLifeResult;
use crate::int_rate_convert::eff_i_to_nom_i;
use bon::builder;

/// Annuity-certain in arrears
///
/// Present value of an annuity-certain in arrears: $1 per period, paid m times per year for n years, starting after t periods.
///
/// # Formula
/// ```text
/// ₜ| aₙ⁽ᵐ⁾ = vᵗ · (1 - vⁿ) / i⁽ᵐ⁾
/// ```
/// where:
/// - `v = 1/(1+i)` is the discount factor
/// - `i` is the effective annual interest rate
/// - `i⁽ᵐ⁾` is the nominal rate convertible m times per year
/// - `n` is the number of periods
/// - `t` is the deferral period (default 0)
/// - `m` is the number of payments per year (default 1)
///
/// # Examples
///
/// ## Basic Annuity-Certain in Arrears
/// ```rust
/// # use rslife::prelude::*;
/// let annuity = an().i(0.03).n(10).call()?;
/// println!("Annuity-certain in arrears: {:.6}", annuity);
/// # RSLifeResult::Ok(())
/// ```
#[builder]
pub fn an(
    i: f64,
    n: u32,
    #[builder(default = 0)] t: u32,
    #[builder(default = 1)] m: u32,
) -> RSLifeResult<f64> {
    if n == 0 {
        return Ok(0.0);
    }

    let v = 1.0 / (1.0 + i);
    let nom_i = eff_i_to_nom_i(i, m);
    let n = n as f64;
    let t = t as f64;

    let result = v.powf(t) * (1.0 - v.powf(n)) / nom_i;
    Ok(result)
}

/// Annuity-certain due
///
/// Present value of an annuity-certain due: $1 per period, paid m times per year for n years, starting immediately (first payment at time 0).
///
/// # Formula
/// ```text
/// ₜ| äₙ⁽ᵐ⁾ = vᵗ · (1 - vⁿ) / d⁽ᵐ⁾
/// ₜ| äₙ⁽ᵐ⁾ = ₜ| aₙ⁽ᵐ⁾ · i⁽ᵐ⁾ / d⁽ᵐ⁾
/// ```
/// where:
/// - `v = 1/(1+i)` is the discount factor
/// - `i` is the effective annual interest rate
/// - `d⁽ᵐ⁾` is the nominal rate of discount convertible m times per year
/// - `i⁽ᵐ⁾` is the nominal rate convertible m times per year
/// - `n` is the number of periods
/// - `t` is the deferral period (default 0)
/// - `m` is the number of payments per year (default 1)
///
/// # Examples
///
/// ## Basic Annuity-Certain Due
/// ```rust
/// # use rslife::prelude::*;
/// let annuity_due = aan().i(0.03).n(10).call()?;
/// println!("Annuity-certain due: {:.6}", annuity_due);
/// # RSLifeResult::Ok(())
/// ```
#[builder]
pub fn aan(
    i: f64,
    n: u32,
    #[builder(default = 0)] t: u32,
    #[builder(default = 1)] m: u32,
) -> RSLifeResult<f64> {
    let d_m = eff_i_to_nom_i(i, m);
    let i_m = eff_i_to_nom_i(i, m);
    let an = an().i(i).n(n).t(t).m(m).call()?;
    let result = an * i_m / d_m;
    Ok(result)
}
