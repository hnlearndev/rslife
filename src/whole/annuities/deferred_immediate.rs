use self::immediate::*;
use super::*;

//-----------------Basic------------------

/// Deferred immediate life annuity:
/// ₜ|aₓ⁽ᵐ⁾ = aₓ⁽ᵐ⁾ - aₓ:ₜ̅⁽ᵐ⁾ =
///
/// Present value of 1/m paid m times per year for life, with payments at the end of each period,
/// starting after a deferment period of t years.
pub fn t_a_x(config: &MortTableConfig, t: i32, x: i32, m: i32) -> PolarsResult<f64> {
    let result = a_x(config, x, m)? - a_x_n(config, x, t, m)?;
    Ok(result)
}

/// Deferred immediate temporary annuity:
/// ₜ|aₓ:ₙ̅⁽ᵐ⁾ = (Dₓ₊ₜ/Dₓ) · aₓ₊ₜ:ₙ̅⁽ᵐ⁾
///
/// Present value of 1/m paid m times per year for n years, with payments at the end of each period,
/// starting after a deferment period of t years.
pub fn t_a_x_n(config: &MortTableConfig, t: i32, x: i32, n: i32, m: i32) -> PolarsResult<f64> {
    let dxt = get_value(config, x + t, "Dx")?;
    let dx = get_value(config, x, "Dx")?;
    let a_xt_n = a_x_n(config, x + t, n, m)?;
    let result = (dxt / dx) * a_xt_n;
    Ok(result)
}

//-----------------Increasing------------------

/// Deferred immediate increasing life annuity:
/// ₜ|Iaₓ⁽ᵐ⁾ = (Dₓ₊ₜ/Dₓ) · Iaₓ₊ₜ⁽ᵐ⁾
///
/// Present value of increasing payments (1/m, 2/m, 3/m, ...) paid m times per year for life,
/// with payments at the end of each period, starting after a deferment period of t years.
/// The payment increases by 1/m each sub-period.
pub fn t_Ia_x(config: &MortTableConfig, t: i32, x: i32, m: i32) -> PolarsResult<f64> {
    let dxt = get_value(config, x + t, "Dx")?;
    let dx = get_value(config, x, "Dx")?;
    let ia_xt = Ia_x(config, x + t, m)?;
    let result = (dxt / dx) * ia_xt;
    Ok(result)
}

/// Deferred immediate increasing temporary annuity:
/// ₜ|(Ia)ₓ:ₙ̅⁽ᵐ⁾ = (Dₓ₊ₜ/Dₓ) · (Ia)ₓ₊ₜ:ₙ̅⁽ᵐ⁾
///
/// Present value of increasing payments (1/m, 2/m, 3/m, ..., n/m) paid m times per year for n years,
/// with payments at the end of each period, starting after a deferment period of t years.
/// The payment increases by 1/m each sub-period.
pub fn t_Ia_x_n(config: &MortTableConfig, t: i32, x: i32, n: i32, m: i32) -> PolarsResult<f64> {
    let dxt = get_value(config, x + t, "Dx")?;
    let dx = get_value(config, x, "Dx")?;
    let ia_xt_n = Ia_x_n(config, x + t, n, m)?;
    let result = (dxt / dx) * ia_xt_n;
    Ok(result)
}

//-----------------Decreasing------------------

/// Deferred immediate decreasing temporary annuity:
/// ₜ|(Da)ₓ:ₙ̅⁽ᵐ⁾ = (Dₓ₊ₜ/Dₓ) · (Da)ₓ₊ₜ:ₙ̅⁽ᵐ⁾
///
/// Present value of decreasing payments (n/m, (n-1)/m, (n-2)/m, ..., 1/m) paid m times per year for n years,
/// with payments at the end of each period, starting after a deferment period of t years.
/// The payment decreases by 1/m each sub-period.
pub fn t_Da_x_n(config: &MortTableConfig, t: i32, x: i32, n: i32, m: i32) -> PolarsResult<f64> {
    let dxt = get_value(config, x + t, "Dx")?;
    let dx = get_value(config, x, "Dx")?;
    let da_xt_n = Da_x_n(config, x + t, n, m)?;
    let result = (dxt / dx) * da_xt_n;
    Ok(result)
}

//-----------------Geometric increasing------------------

/// Deferred immediate geometric life annuity:
/// ₜ|(ga)ₓ⁽ᵐ⁾ = (ga)ₓ⁽ᵐ⁾ - (ga)ₓ:ₜ̅⁽ᵐ⁾
///
/// Present value of geometrically increasing payments paid m times per year for life,
/// with payments at the end of each period, starting after a deferment period of t years.
/// The payment grows geometrically at rate g each year.
pub fn t_ga_x(config: &MortTableConfig, t: i32, x: i32, m: i32, g: f64) -> PolarsResult<f64> {
    let result = ga_x(config, x, m, g)? - ga_x_n(config, x, t, m, g)?;
    Ok(result)
}

/// Deferred immediate geometric temporary annuity:
/// ₜ|(ga)ₓ:ₙ̅⁽ᵐ⁾ = (Dₓ₊ₜ/Dₓ) · (ga)ₓ₊ₜ:ₙ̅⁽ᵐ⁾
///
/// Present value of geometrically increasing payments paid m times per year for n years,
/// with payments at the end of each period, starting after a deferment period of t years.
/// The payment grows geometrically at rate g each year.
pub fn t_ga_x_n(
    config: &MortTableConfig,
    t: i32,
    x: i32,
    n: i32,
    m: i32,
    g: f64,
) -> PolarsResult<f64> {
    let dxt = get_value(config, x + t, "Dx")?;
    let dx = get_value(config, x, "Dx")?;
    let ga_xt_n = ga_x_n(config, x + t, n, m, g)?;
    let result = (dxt / dx) * ga_xt_n;
    Ok(result)
}
