use super::*;

//-----------------Basic------------------

/// Deferred immediate life annuity:
/// ₜ|aₓ⁽ᵐ⁾ = aₓ⁽ᵐ⁾ - aₓ:ₜ̅⁽ᵐ⁾ =
///
/// Present value of 1/m paid m times per year for life, with payments at the end of each period,
/// starting after a deferment period of t years.
pub fn t_a_x_(
    config: &MortTableConfig,
    entry_age: i32,
    t: i32,
    x: i32,
    m: i32,
) -> PolarsResult<f64> {
    let new_config = get_new_config_with_selected_table(config, entry_age)?;
    whole::t_a_x(&new_config, t, x, m)
}

/// Deferred immediate temporary annuity:
/// ₜ|aₓ:ₙ̅⁽ᵐ⁾ = (Dₓ₊ₜ/Dₓ) · aₓ₊ₜ:ₙ̅⁽ᵐ⁾
///
/// Present value of 1/m paid m times per year for n years, with payments at the end of each period,
/// starting after a deferment period of t years.
pub fn t_a_x_n_(
    config: &MortTableConfig,
    entry_age: i32,
    t: i32,
    x: i32,
    n: i32,
    m: i32,
) -> PolarsResult<f64> {
    let new_config = get_new_config_with_selected_table(config, entry_age)?;
    whole::t_a_x_n(&new_config, t, x, n, m)
}

//-----------------Increasing------------------

/// Deferred immediate increasing life annuity:
/// ₜ|Iaₓ⁽ᵐ⁾ = (Dₓ₊ₜ/Dₓ) · Iaₓ₊ₜ⁽ᵐ⁾
///
/// Present value of increasing payments (1/m, 2/m, 3/m, ...) paid m times per year for life,
/// with payments at the end of each period, starting after a deferment period of t years.
/// The payment increases by 1/m each sub-period.
pub fn t_Ia_x_(
    config: &MortTableConfig,
    entry_age: i32,
    t: i32,
    x: i32,
    m: i32,
) -> PolarsResult<f64> {
    let new_config = get_new_config_with_selected_table(config, entry_age)?;
    whole::t_Ia_x(&new_config, t, x, m)
}

/// Deferred immediate increasing temporary annuity:
/// ₜ|(Ia)ₓ:ₙ̅⁽ᵐ⁾ = (Dₓ₊ₜ/Dₓ) · (Ia)ₓ₊ₜ:ₙ̅⁽ᵐ⁾
///
/// Present value of increasing payments (1/m, 2/m, 3/m, ..., n/m) paid m times per year for n years,
/// with payments at the end of each period, starting after a deferment period of t years.
/// The payment increases by 1/m each sub-period.
pub fn t_Ia_x_n_(
    config: &MortTableConfig,
    entry_age: i32,
    t: i32,
    x: i32,
    n: i32,
    m: i32,
) -> PolarsResult<f64> {
    let new_config = get_new_config_with_selected_table(config, entry_age)?;
    whole::t_Ia_x_n(&new_config, t, x, n, m)
}

//-----------------Decreasing------------------

/// Deferred immediate decreasing temporary annuity:
/// ₜ|(Da)ₓ:ₙ̅⁽ᵐ⁾ = (Dₓ₊ₜ/Dₓ) · (Da)ₓ₊ₜ:ₙ̅⁽ᵐ⁾
///
/// Present value of decreasing payments (n/m, (n-1)/m, (n-2)/m, ..., 1/m) paid m times per year for n years,
/// with payments at the end of each period, starting after a deferment period of t years.
/// The payment decreases by 1/m each sub-period.
pub fn t_Da_x_n_(
    config: &MortTableConfig,
    entry_age: i32,
    t: i32,
    x: i32,
    n: i32,
    m: i32,
) -> PolarsResult<f64> {
    let new_config = get_new_config_with_selected_table(config, entry_age)?;
    whole::t_Da_x_n(&new_config, t, x, n, m)
}

//-----------------Geometric increasing------------------

/// Deferred immediate geometric life annuity:
/// ₜ|(ga)ₓ⁽ᵐ⁾ = (ga)ₓ⁽ᵐ⁾ - (ga)ₓ:ₜ̅⁽ᵐ⁾
///
/// Present value of geometrically increasing payments paid m times per year for life,
/// with payments at the end of each period, starting after a deferment period of t years.
/// The payment grows geometrically at rate g each year.
pub fn t_ga_x_(
    config: &MortTableConfig,
    entry_age: i32,
    t: i32,
    x: i32,
    m: i32,
    g: f64,
) -> PolarsResult<f64> {
    let new_config = get_new_config_with_selected_table(config, entry_age)?;
    whole::t_ga_x(&new_config, t, x, m, g)
}

/// Deferred immediate geometric temporary annuity:
/// ₜ|(ga)ₓ:ₙ̅⁽ᵐ⁾ = (Dₓ₊ₜ/Dₓ) · (ga)ₓ₊ₜ:ₙ̅⁽ᵐ⁾
///
/// Present value of geometrically increasing payments paid m times per year for n years,
/// with payments at the end of each period, starting after a deferment period of t years.
/// The payment grows geometrically at rate g each year.
pub fn t_ga_x_n_(
    config: &MortTableConfig,
    entry_age: i32,
    t: i32,
    x: i32,
    n: i32,
    m: i32,
    g: f64,
) -> PolarsResult<f64> {
    let new_config = get_new_config_with_selected_table(config, entry_age)?;
    whole::t_ga_x_n(&new_config, t, x, n, m, g)
}
