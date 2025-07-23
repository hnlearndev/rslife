use self::due::*;
use super::*;

//-----------------Basic------------------

/// Deferred life annuity-due:
/// ₜ|äₓ⁽ᵐ⁾ = äₓ⁽ᵐ⁾ - äₓ:ₜ̅⁽ᵐ⁾
///
/// Present value of 1/m paid m times per year for life, with payments at the beginning of each period,
/// starting after a deferment period of t years.
pub fn t_aa_x(config: &MortTableConfig, t: i32, x: i32, m: i32) -> PolarsResult<f64> {
    let result = aa_x(config, x, m)? - aa_x_n(config, x, t, m)?;
    Ok(result)
}

/// Deferred temporary annuity-due:
/// ₜ|äₓ:ₙ̅⁽ᵐ⁾ = ₜ|äₓ⁽ᵐ⁾ - ₜ₊ₙ|äₓ⁽ᵐ⁾
///
/// Present value of 1/m paid m times per year for n years, with payments at the beginning of each period,
/// starting after a deferment period of t years.
pub fn t_aa_x_n(config: &MortTableConfig, t: i32, x: i32, n: i32, m: i32) -> PolarsResult<f64> {
    let result = t_aa_x(config, t, x, m)? - t_aa_x(config, t + n, x, m)?;
    Ok(result)
}

//-----------------Increasing------------------

/// Deferred increasing life annuity-due:
/// ₜ|(Iä)ₓ⁽ᵐ⁾ = (Iä)ₓ⁽ᵐ⁾ - (Iä)ₓ:ₜ̅⁽ᵐ⁾
///
/// Present value of increasing payments (1/m, 2/m, 3/m, ...) paid m times per year for life,
/// with payments at the beginning of each period, starting after a deferment period of t years.
/// The payment increases by 1/m each sub-period.
pub fn t_Iaa_x(config: &MortTableConfig, t: i32, x: i32, m: i32) -> PolarsResult<f64> {
    let result = Iaa_x(config, x, m)? - Iaa_x_n(config, x, t, m)?;
    Ok(result)
}

/// Deferred due increasing temporary annuity:
/// ₜ|(Iä)ₓ:ₙ̅⁽ᵐ⁾ = (Dₓ₊ₜ/Dₓ) · (Iä)ₓ₊ₜ:ₙ̅⁽ᵐ⁾
///
/// Present value of increasing payments (1/m, 2/m, 3/m, ..., n/m) paid m times per year for n years,
/// with payments at the beginning of each period, starting after a deferment period of t years.
/// The payment increases by 1/m each sub-period.
pub fn t_Iaa_x_n(config: &MortTableConfig, t: i32, x: i32, n: i32, m: i32) -> PolarsResult<f64> {
    let dxt = get_value(config, x + t, "Dx")?;
    let dx = get_value(config, x, "Dx")?;
    let iaa_xt_n = Iaa_x_n(config, x + t, n, m)?;
    let result = (dxt / dx) * iaa_xt_n;
    Ok(result)
}

//-----------------Decreasing------------------

/// Deferred due decreasing temporary annuity:
/// ₜ|(Dä)ₓ:ₙ̅⁽ᵐ⁾ = (Dₓ₊ₜ/Dₓ) · (Dä)ₓ₊ₜ:ₙ̅⁽ᵐ⁾
///
/// Present value of decreasing payments (n/m, (n-1)/m, (n-2)/m, ..., 1/m) paid m times per year for n years,
/// with payments at the beginning of each period, starting after a deferment period of t years.
/// The payment decreases by 1/m each sub-period.
pub fn t_Daa_x_n(config: &MortTableConfig, t: i32, x: i32, n: i32, m: i32) -> PolarsResult<f64> {
    let dxt = get_value(config, x + t, "Dx")?;
    let dx = get_value(config, x, "Dx")?;
    let daa_xt_n = Daa_x_n(config, x + t, n, m)?;
    let result = (dxt / dx) * daa_xt_n;
    Ok(result)
}

//-----------------Geometric increasing------------------

/// Deferred due geometric life annuity:
/// ₜ|äₓ⁽ᵍ⁾ with adjusted interest rate i′ = (1+i)/(1+g) - 1
///
/// Present value of geometrically increasing payments paid m times per year for life,
/// with payments at the beginning of each period, starting after a deferment period of t years.
/// The payment grows geometrically at rate g each year.
pub fn t_gaa_x(config: &MortTableConfig, t: i32, x: i32, m: i32, g: f64) -> PolarsResult<f64> {
    let new_config = get_new_config(config, g);
    let result = t_aa_x(&new_config, t, x, m)?;
    Ok(result)
}

/// Deferred due geometric temporary annuity:
/// ₜ|äₓ:ₙ̅⁽ᵍ⁾ with adjusted interest rate i′ = (1+i)/(1+g) - 1
///
/// Present value of geometrically increasing payments paid m times per year for n years,
/// with payments at the beginning of each period, starting after a deferment period of t years.
/// The payment grows geometrically at rate g each year.
pub fn t_gaa_x_n(
    config: &MortTableConfig,
    t: i32,
    x: i32,
    n: i32,
    m: i32,
    g: f64,
) -> PolarsResult<f64> {
    let new_config = get_new_config(config, g);
    let result = t_aa_x_n(&new_config, t, x, n, m)?;
    Ok(result)
}
