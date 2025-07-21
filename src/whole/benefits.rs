#![allow(non_snake_case)]

use super::*;

/// # Insurance Benefits Module
///
/// Actuarial functions for calculating present values of life insurance benefits.
/// Uses commutation functions from mortality tables with interest rates.
///
/// ## Core Functions
///
/// - [`Ax`] - Whole life insurance: Aₓ = Mₓ/Dₓ
/// - [`Axn`] - Term insurance: A¹ₓ:ₙ
/// - [`AExn`] - Endowment insurance: Aₓ:ₙ
/// - [`IAx`] - Increasing whole life: (IA)ₓ
/// - [`gAx`] - Geometric growth: Aₓ⁽ᵍ⁾
///
/// ## Example
///
/// ```rust
/// use rslife::prelude::*;
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let xml = MortXML::from_url_id(1704)?;
/// let config = MortTableConfig {
///     xml,
///     l_x_init: 100_000,
///     pct: Some(1.0),
///     int_rate: Some(0.03),
///     assumption: Some(AssumptionEnum::UDD),
/// };
///
/// let whole_life = rslife::whole::benefits::Ax(&config, 30)?;      // ~$0.40
/// let term_20 = rslife::whole::benefits::Axn(&config, 30, 20)?;   // ~$0.15
/// # Ok(())
/// # }
/// ```
///
/// Whole life insurance: Aₓ = Mₓ/Dₓ
///
/// Present value of $1 paid at death, regardless of when death occurs.
///
/// # Example
/// ```rust
/// use rslife::prelude::*;
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let xml = MortXML::from_url_id(1704)?;
/// let config = MortTableConfig {
///     xml, l_x_init: 100_000, pct: Some(1.0),
///     int_rate: Some(0.03), assumption: Some(AssumptionEnum::UDD),
/// };
/// let value = rslife::whole::benefits::Ax(&config, 30)?; // ~$0.40 for age 30
/// # Ok(())
/// # }
/// ```
pub fn Ax(config: &MortTableConfig, x: i32) -> PolarsResult<f64> {
    get_value(config, x, "Ax")
}

/// Term life insurance: A¹ₓ:ₙ = Aₓ - Aₓ₊ₙ·Eₓ:ₙ
///
/// Present value of $1 paid only if death occurs within n years.
pub fn Axn(config: &MortTableConfig, x: i32, n: i32) -> PolarsResult<f64> {
    let ax = Ax(config, x)?;
    let axn = Ax(config, x + n)?;
    let exn = Exn(config, x, n)?;
    let result = ax - axn * exn;
    Ok(result)
}

/// Pure endowment: Eₓ:ₙ = Dₓ₊ₙ/Dₓ
///
/// Present value of $1 paid if and only if the insured survives n years.
pub fn Exn(config: &MortTableConfig, x: i32, n: i32) -> PolarsResult<f64> {
    let dx = get_value(config, x, "Dx")?;
    let dxn = get_value(config, x + n, "Dx")?;
    let result = dxn / dx;
    Ok(result)
}

/// Endowment insurance: Aₓ:ₙ = A¹ₓ:ₙ + Eₓ:ₙ
///
/// $1 paid at death (if within n years) OR at survival to n years.
pub fn AExn(config: &MortTableConfig, x: i32, n: i32) -> PolarsResult<f64> {
    let axn = Axn(config, x, n)?;
    let exn = Exn(config, x, n)?;
    let result = axn + exn;
    Ok(result)
}

/// Deferred whole life: ₜAₓ = Mₓ₊ₜ/Dₓ
///
/// $1 paid at death, but only if death occurs after t years.
pub fn tAx(config: &MortTableConfig, x: i32, t: i32) -> PolarsResult<f64> {
    let mx_t = get_value(config, x + t, "Mx")?;
    let dx = get_value(config, x, "Dx")?;
    let result = mx_t / dx;
    Ok(result)
}

/// Deferred term: ₜA¹ₓ:ₙ = ₜAₓ - ₜ₊ₙAₓ
///
/// $1 paid at death between t and t+n years after issue.
pub fn tAxn(config: &MortTableConfig, x: i32, n: i32, t: i32) -> PolarsResult<f64> {
    let tax = tAx(config, x, t)?;
    let taxn = tAx(config, x, t + n)?;
    let result = tax - taxn;
    Ok(result)
}

/// Deferred pure endowment: ₜEₓ:ₙ = Dₓ₊ₙ₊ₜ/Dₓ
///
/// $1 paid if insured survives both deferral period t and additional period n.
pub fn tExn(config: &MortTableConfig, x: i32, n: i32, t: i32) -> PolarsResult<f64> {
    let dx = get_value(config, x, "Dx")?;
    let dxnt = get_value(config, x + n + t, "Dx")?;
    let result = dxnt / dx;
    Ok(result)
}

/// Deferred endowment: ₜAₓ:ₙ = ₜA¹ₓ:ₙ + ₜEₓ:ₙ
pub fn tAExn(config: &MortTableConfig, x: i32, n: i32, t: i32) -> PolarsResult<f64> {
    let taxn = tAxn(config, x, n, t)?;
    let texn = tExn(config, x, n, t)?;
    let result = taxn + texn;
    Ok(result)
}

/// Increasing whole life: (IA)ₓ = Sₓ/Dₓ
///
/// Death benefit increases by 1 each year: k paid if death in k-th year.
pub fn IAx(config: &MortTableConfig, x: i32) -> PolarsResult<f64> {
    get_value(config, x, "IAx")
}

/// Increasing term: (IA)¹ₓ:ₙ = (Sₓ - Sₓ₊ₙ - n·Mₓ₊ₙ)/Dₓ
///
/// Death benefit increases by 1 each year, pays only if death within n years.
pub fn IAxn(config: &MortTableConfig, x: i32, n: i32) -> PolarsResult<f64> {
    let sx = get_value(config, x, "Sx")?;
    let sxn = get_value(config, x + n, "Sx")?;
    let mxn = get_value(config, x + n, "Mx")?;
    let dx = get_value(config, x, "Dx")?;
    let result = (sx - sxn - n as f64 * mxn) / dx;
    Ok(result)
}

/// Geometric whole life: Aₓ⁽ᵍ⁾ with adjusted interest rate i′ = (1+i)/(1+g) - 1
///
/// Death benefit grows geometrically at rate g each year.
pub fn gAx(config: &MortTableConfig, x: i32, g: f64) -> PolarsResult<f64> {
    let new_config = get_new_config(config, g);
    let result = Ax(&new_config, x)?;
    Ok(result)
}

/// Geometric n-year term: Aₓ:ₙ⁽ᵍ⁾ with adjusted interest rate i′ = (1+i)/(1+g) - 1
///
/// Death benefit grows geometrically at rate g for n years.
pub fn gAxn(config: &MortTableConfig, x: i32, n: i32, g: f64) -> PolarsResult<f64> {
    let new_config = get_new_config(config, g);
    let result = Axn(&new_config, x, n)?;
    Ok(result)
}
