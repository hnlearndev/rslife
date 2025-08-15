#![allow(non_snake_case)]

use crate::RSLifeResult;
use crate::int_rate_convert::{eff_i_to_nom_d, eff_i_to_nom_i};
use bon::builder;

/// Annuity-certain due/in advance
/// Present value of an annuity-certain in arrears: $1 per period, paid m times per year for n years, starting after t periods.
///
/// # Formula
/// ```text
/// ₜ| äₙ⁽ᵐ⁾ = vᵗ · (1 - vⁿ) / d⁽ᵐ⁾
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
/// Present value of an annuity-certain due (in advance).
///
/// Calculates the present value of an annuity-certain due: $1 per period, paid m times per year for n years, starting immediately (first payment at time 0).
///
/// # Formula
/// $$
/// 	ext{PV} = v^t \cdot \frac{1 - v^n}{d^{(m)}}
/// $$
/// where $v = 1/(1+i)$, $d^{(m)}$ is the nominal rate of discount convertible m times per year.
///
/// # Parameters
/// - `i`: Effective annual interest rate
/// - `n`: Number of periods
/// - `t`: Deferral period (default 0)
/// - `m`: Number of payments per year (default 1)
#[builder]
pub fn aan(
    i: f64,
    n: u32,
    #[builder(default = 0)] t: u32,
    #[builder(default = 1)] m: u32,
) -> RSLifeResult<f64> {
    if n == 0 {
        return Ok(0.0);
    }

    let v = 1.0 / (1.0 + i);
    let nom_d = eff_i_to_nom_d(i, m);
    let n = n as f64;
    let t = t as f64;
    // ₜ| äₙ⁽ᵐ⁾ = vᵗ · (1 - vⁿ) / d⁽ᵐ⁾
    let result = v.powf(t) * (1.0 - v.powf(n)) / nom_d;
    Ok(result)
}

/// Annuity-certain immeditate/in-arrears
///
/// Present value of an annuity-certain due: $1 per period, paid m times per year for n years, starting immediately (first payment at time 0).
///
/// # Formula
/// ```text
/// ₜ| aₙ⁽ᵐ⁾ = vᵗ · (1 - vⁿ) / i⁽ᵐ⁾
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
/// Present value of an annuity-certain immediate (in arrears).
///
/// Calculates the present value of an annuity-certain immediate: $1 per period, paid m times per year for n years, with payments at the end of each period.
///
/// # Formula
/// $$
/// 	ext{PV} = v^t \cdot \frac{1 - v^n}{i^{(m)}}
/// $$
/// where $v = 1/(1+i)$, $i^{(m)}$ is the nominal rate convertible m times per year.
///
/// # Parameters
/// - `i`: Effective annual interest rate
/// - `n`: Number of periods
/// - `t`: Deferral period (default 0)
/// - `m`: Number of payments per year (default 1)
#[builder]
pub fn an(
    i: f64,
    n: u32,
    #[builder(default = 0)] t: u32,
    #[builder(default = 1)] m: u32,
) -> RSLifeResult<f64> {
    let nom_i = eff_i_to_nom_i(i, m);
    let nom_d = eff_i_to_nom_d(i, m);
    let due = aan().i(i).n(n).t(t).m(m).call()?;
    // ₜ| aₙ⁽ᵐ⁾ = vᵗ · (1 - vⁿ) / i⁽ᵐ⁾
    let result = due * nom_d / nom_i;
    Ok(result)
}

/// Present value of an increasing annuity-certain due (in advance).
///
/// Calculates the present value of an increasing annuity-certain due: payments increase by $1 each period, paid m times per year for n years, starting immediately (first payment at time 0).
///
/// # Formula
/// $$
/// 	ext{PV} = v^t \cdot \frac{\text{aan} - n v^n}{d^{(m)}}
/// $$
/// where $v = 1/(1+i)$, $d^{(m)}$ is the nominal rate of discount convertible m times per year.
///
/// # Parameters
/// - `i`: Effective annual interest rate
/// - `n`: Number of periods
/// - `t`: Deferral period (default 0)
/// - `m`: Number of payments per year (default 1)
#[builder]
pub fn Iaan(
    i: f64,
    n: u32,
    #[builder(default = 0)] t: u32,
    #[builder(default = 1)] m: u32,
) -> RSLifeResult<f64> {
    if n == 0 {
        return Ok(0.0);
    }

    let v = 1.0 / (1.0 + i);
    let nom_d = eff_i_to_nom_d(i, m);
    let aan = aan().i(i).n(n).t(t).call()?;
    let n = n as f64;
    let t = t as f64;
    let result = v.powf(t) * (aan - n * v.powf(n)) / nom_d;
    Ok(result)
}

/// Present value of an increasing annuity-certain immediate (in arrears).
///
/// Calculates the present value of an increasing annuity-certain immediate: payments increase by $1 each period, paid m times per year for n years, with payments at the end of each period.
///
/// # Formula
/// $$
/// 	ext{PV} = v^t \cdot \frac{1 - v^n}{i^{(m)}}
/// $$
/// where $v = 1/(1+i)$, $i^{(m)}$ is the nominal rate convertible m times per year.
///
/// # Parameters
/// - `i`: Effective annual interest rate
/// - `n`: Number of periods
/// - `t`: Deferral period (default 0)
/// - `m`: Number of payments per year (default 1)
#[builder]
pub fn Ian(
    i: f64,
    n: u32,
    #[builder(default = 0)] t: u32,
    #[builder(default = 1)] m: u32,
) -> RSLifeResult<f64> {
    let nom_i = eff_i_to_nom_i(i, m);
    let nom_d = eff_i_to_nom_d(i, m);
    let due = Iaan().i(i).n(n).t(t).m(m).call()?;
    let result = due * nom_d / nom_i;
    Ok(result)
}

/// Present value of a decreasing annuity-certain immediate (in arrears).
///
/// Calculates the present value of a decreasing annuity-certain immediate: payments decrease by $1 each period, paid m times per year for n years, with payments at the end of each period.
///
/// # Formula
/// $$
/// 	ext{PV} = v^t \cdot \frac{n - \text{an}}{i^{(m)}}
/// $$
/// where $v = 1/(1+i)$, $i^{(m)}$ is the nominal rate convertible m times per year.
///
/// # Parameters
/// - `i`: Effective annual interest rate
/// - `n`: Number of periods
/// - `t`: Deferral period (default 0)
/// - `m`: Number of payments per year (default 1)
#[builder]
pub fn Dan(
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
    let an = an().i(i).n(n).t(t).call()?;
    let n = n as f64;
    let t = t as f64;
    let result = v.powf(t) * (n - an) / nom_i;
    Ok(result)
}

/// Present value of a decreasing annuity-certain due (in advance).
///
/// Calculates the present value of a decreasing annuity-certain due: payments decrease by $1 each period, paid m times per year for n years, starting immediately (first payment at time 0).
///
/// # Formula
/// $$
/// 	ext{PV} = \text{Dan} \cdot \frac{i^{(m)}}{d^{(m)}}
/// $$
/// where $i^{(m)}$ is the nominal rate convertible m times per year, $d^{(m)}$ is the nominal rate of discount convertible m times per year.
///
/// # Parameters
/// - `i`: Effective annual interest rate
/// - `n`: Number of periods
/// - `t`: Deferral period (default 0)
/// - `m`: Number of payments per year (default 1)
#[builder]
pub fn Daan(
    i: f64,
    n: u32,
    #[builder(default = 0)] t: u32,
    #[builder(default = 1)] m: u32,
) -> RSLifeResult<f64> {
    let nom_i = eff_i_to_nom_i(i, m);
    let nom_d = eff_i_to_nom_d(i, m);
    let immediate = Dan().i(i).n(n).t(t).m(m).call()?;
    let result = immediate * nom_i / nom_d;
    Ok(result)
}

/// Accumulated value of an annuity-certain due (in advance).
///
/// Calculates the accumulated value at the end of n periods for an annuity-certain due: $1 per period, paid m times per year for n years, starting immediately.
///
/// # Formula
/// $$
/// 	ext{AV} = \text{aan} \cdot (1 + i)^n
/// $$
/// where $i$ is the effective annual interest rate.
///
/// # Parameters
/// - `i`: Effective annual interest rate
/// - `n`: Number of periods
/// - `t`: Deferral period (default 0)
/// - `m`: Number of payments per year (default 1)
#[builder]
pub fn ssn(
    i: f64,
    n: u32,
    #[builder(default = 0)] t: u32,
    #[builder(default = 1)] m: u32,
) -> RSLifeResult<f64> {
    let annuity = aan().i(i).n(n).t(t).m(m).call()?;
    let factor = (1.0 + i).powf(n as f64);
    Ok(annuity * factor)
}

/// Accumulated value of an annuity-certain immediate (in arrears).
///
/// Calculates the accumulated value at the end of n periods for an annuity-certain immediate: $1 per period, paid m times per year for n years, with payments at the end of each period.
///
/// # Formula
/// $$
/// 	ext{AV} = \text{an} \cdot (1 + i)^n
/// $$
/// where $i$ is the effective annual interest rate.
///
/// # Parameters
/// - `i`: Effective annual interest rate
/// - `n`: Number of periods
/// - `t`: Deferral period (default 0)
/// - `m`: Number of payments per year (default 1)
#[builder]
pub fn sn(
    i: f64,
    n: u32,
    #[builder(default = 0)] t: u32,
    #[builder(default = 1)] m: u32,
) -> RSLifeResult<f64> {
    let annuity = an().i(i).n(n).t(t).m(m).call()?;
    let factor = (1.0 + i).powf(n as f64);
    Ok(annuity * factor)
}

/// Accumulated value of an increasing annuity-certain due (in advance).
///
/// Calculates the accumulated value at the end of n periods for an increasing annuity-certain due: payments increase by $1 each period, paid m times per year for n years, starting immediately.
///
/// # Formula
/// $$
/// 	ext{AV} = \text{Iaan} \cdot (1 + i)^n
/// $$
/// where $i$ is the effective annual interest rate.
///
/// # Parameters
/// - `i`: Effective annual interest rate
/// - `n`: Number of periods
/// - `t`: Deferral period (default 0)
/// - `m`: Number of payments per year (default 1)
#[builder]
pub fn Issn(
    i: f64,
    n: u32,
    #[builder(default = 0)] t: u32,
    #[builder(default = 1)] m: u32,
) -> RSLifeResult<f64> {
    let annuity = Iaan().i(i).n(n).t(t).m(m).call()?;
    let factor = (1.0 + i).powf(n as f64);
    Ok(annuity * factor)
}

/// Accumulated value of an increasing annuity-certain immediate (in arrears).
///
/// Calculates the accumulated value at the end of n periods for an increasing annuity-certain immediate: payments increase by $1 each period, paid m times per year for n years, with payments at the end of each period.
///
/// # Formula
/// $$
/// 	ext{AV} = \text{Ian} \cdot (1 + i)^n
/// $$
/// where $i$ is the effective annual interest rate.
///
/// # Parameters
/// - `i`: Effective annual interest rate
/// - `n`: Number of periods
/// - `t`: Deferral period (default 0)
/// - `m`: Number of payments per year (default 1)
#[builder]
pub fn Isn(
    i: f64,
    n: u32,
    #[builder(default = 0)] t: u32,
    #[builder(default = 1)] m: u32,
) -> RSLifeResult<f64> {
    let annuity = Ian().i(i).n(n).t(t).m(m).call()?;
    let factor = (1.0 + i).powf(n as f64);
    Ok(annuity * factor)
}

/// Accumulated value of a decreasing annuity-certain due (in advance).
///
/// Calculates the accumulated value at the end of n periods for a decreasing annuity-certain due: payments decrease by $1 each period, paid m times per year for n years, starting immediately.
///
/// # Formula
/// $$
/// 	ext{AV} = \text{Daan} \cdot (1 + i)^n
/// $$
/// where $i$ is the effective annual interest rate.
///
/// # Parameters
/// - `i`: Effective annual interest rate
/// - `n`: Number of periods
/// - `t`: Deferral period (default 0)
/// - `m`: Number of payments per year (default 1)
#[builder]
pub fn Dssn(
    i: f64,
    n: u32,
    #[builder(default = 0)] t: u32,
    #[builder(default = 1)] m: u32,
) -> RSLifeResult<f64> {
    let annuity = Daan().i(i).n(n).t(t).m(m).call()?;
    let factor = (1.0 + i).powf(n as f64);
    Ok(annuity * factor)
}

/// Accumulated value of a decreasing annuity-certain immediate (in arrears).
///
/// Calculates the accumulated value at the end of n periods for a decreasing annuity-certain immediate: payments decrease by $1 each period, paid m times per year for n years, with payments at the end of each period.
///
/// # Formula
/// $$
/// 	ext{AV} = \text{Dan} \cdot (1 + i)^n
/// $$
/// where $i$ is the effective annual interest rate.
///
/// # Parameters
/// - `i`: Effective annual interest rate
/// - `n`: Number of periods
/// - `t`: Deferral period (default 0)
/// - `m`: Number of payments per year (default 1)
#[builder]
pub fn Dsn(
    i: f64,
    n: u32,
    #[builder(default = 0)] t: u32,
    #[builder(default = 1)] m: u32,
) -> RSLifeResult<f64> {
    let annuity = Dan().i(i).n(n).t(t).m(m).call()?;
    let factor = (1.0 + i).powf(n as f64);
    Ok(annuity * factor)
}

// ================================================
// UNIT TESTS
// ================================================
#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_abs_diff_eq;

    #[test]
    fn test_fn_an_01() {
        let rates = [0.005, 0.01, 0.015, 0.02, 0.025];
        let terms = [1, 20, 41, 80, 100];
        let expected = [0.9950, 18.0456, 30.4590, 39.7445, 36.6141];
        for (i, (rate, term)) in rates.iter().zip(terms.iter()).enumerate() {
            let ans = an().i(*rate).n(*term).call().unwrap();
            let exp = expected[i];
            assert_abs_diff_eq!(ans, exp, epsilon = 1e-4);
        }
    }

    #[test]
    fn test_fn_Ian_01() {
        let rates = [0.03, 0.04, 0.05, 0.06, 0.07];
        let terms = [1, 23, 48, 70, 100];
        let expected = [0.9709, 152.9852, 287.3239, 269.7117, 216.4693];
        for (i, (rate, term)) in rates.iter().zip(terms.iter()).enumerate() {
            let ans = Ian().i(*rate).n(*term).call().unwrap();
            let exp = expected[i];
            assert_abs_diff_eq!(ans, exp, epsilon = 1e-4);
        }
    }

    #[test]
    fn test_fn_Dan_01() {
        let rates = [0.08, 0.09, 0.1, 0.12, 0.15];
        let terms = [1, 34, 70, 100, 50];
        let expected = [0.9259, 260.9129, 600.1266, 763.8897, 288.9299];
        for (i, (rate, term)) in rates.iter().zip(terms.iter()).enumerate() {
            let ans = Dan().i(*rate).n(*term).call().unwrap();
            let exp = expected[i];
            assert_abs_diff_eq!(ans, exp, epsilon = 1e-4);
        }
    }
}
