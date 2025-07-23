use super::*;

//-----------------Basic------------------

/// Deferred life annuity-due:
/// ₜ|äₓ⁽ᵐ⁾ = äₓ⁽ᵐ⁾ - äₓ:ₜ̅⁽ᵐ⁾
///
/// Present value of 1/m paid m times per year for life, with payments at the beginning of each period,
/// starting after a deferment period of t years.
pub fn t_aa_x_(
    config: &MortTableConfig,
    entry_age: i32,
    t: i32,
    x: i32,
    m: i32,
) -> PolarsResult<f64> {
    let new_config = get_new_config_with_selected_table(config, entry_age)?;
    whole::t_aa_x(&new_config, t, x, m)
}

/// Deferred temporary annuity-due:
/// ₜ|äₓ:ₙ̅⁽ᵐ⁾ = ₜ|äₓ⁽ᵐ⁾ - ₜ₊ₙ|äₓ⁽ᵐ⁾
///
/// Present value of 1/m paid m times per year for n years, with payments at the beginning of each period,
/// starting after a deferment period of t years.
pub fn t_aa_x_n_(
    config: &MortTableConfig,
    entry_age: i32,
    t: i32,
    x: i32,
    n: i32,
    m: i32,
) -> PolarsResult<f64> {
    let new_config = get_new_config_with_selected_table(config, entry_age)?;
    whole::t_aa_x_n(&new_config, t, x, n, m)
}

//-----------------Increasing------------------

/// Deferred increasing life annuity-due:
/// ₜ|(Iä)ₓ⁽ᵐ⁾ = (Iä)ₓ⁽ᵐ⁾ - (Iä)ₓ:ₜ̅⁽ᵐ⁾
///
/// Present value of increasing payments (1/m, 2/m, 3/m, ...) paid m times per year for life,
/// with payments at the beginning of each period, starting after a deferment period of t years.
/// The payment increases by 1/m each sub-period.
pub fn t_Iaa_x_(
    config: &MortTableConfig,
    entry_age: i32,
    t: i32,
    x: i32,
    m: i32,
) -> PolarsResult<f64> {
    let new_config = get_new_config_with_selected_table(config, entry_age)?;
    whole::t_Iaa_x(&new_config, t, x, m)
}

/// Deferred due increasing temporary annuity:
/// ₜ|(Iä)ₓ:ₙ̅⁽ᵐ⁾ = (Dₓ₊ₜ/Dₓ) · (Iä)ₓ₊ₜ:ₙ̅⁽ᵐ⁾
///
/// Present value of increasing payments (1/m, 2/m, 3/m, ..., n/m) paid m times per year for n years,
/// with payments at the beginning of each period, starting after a deferment period of t years.
/// The payment increases by 1/m each sub-period.
pub fn t_Iaa_x_n_(
    config: &MortTableConfig,
    entry_age: i32,
    t: i32,
    x: i32,
    n: i32,
    m: i32,
) -> PolarsResult<f64> {
    let new_config = get_new_config_with_selected_table(config, entry_age)?;
    whole::t_Iaa_x_n(&new_config, t, x, n, m)
}

//-----------------Decreasing------------------

/// Deferred due decreasing temporary annuity:
/// ₜ|(Dä)ₓ:ₙ̅⁽ᵐ⁾ = (Dₓ₊ₜ/Dₓ) · (Dä)ₓ₊ₜ:ₙ̅⁽ᵐ⁾
///
/// Present value of decreasing payments (n/m, (n-1)/m, (n-2)/m, ..., 1/m) paid m times per year for n years,
/// with payments at the beginning of each period, starting after a deferment period of t years.
/// The payment decreases by 1/m each sub-period.
pub fn t_Daa_x_n_(
    config: &MortTableConfig,
    entry_age: i32,
    t: i32,
    x: i32,
    n: i32,
    m: i32,
) -> PolarsResult<f64> {
    let new_config = get_new_config_with_selected_table(config, entry_age)?;
    whole::t_Daa_x_n(&new_config, t, x, n, m)
}

//-----------------Geometric increasing------------------

/// Deferred due geometric life annuity:
/// ₜ|äₓ⁽ᵍ⁾ with adjusted interest rate i′ = (1+i)/(1+g) - 1
///
/// Present value of geometrically increasing payments paid m times per year for life,
/// with payments at the beginning of each period, starting after a deferment period of t years.
/// The payment grows geometrically at rate g each year.
pub fn t_gaa_x_(
    config: &MortTableConfig,
    entry_age: i32,
    t: i32,
    x: i32,
    m: i32,
    g: f64,
) -> PolarsResult<f64> {
    let new_config = get_new_config_with_selected_table(config, entry_age)?;
    whole::t_gaa_x(&new_config, t, x, m, g)
}

/// Deferred due geometric temporary annuity:
/// ₜ|äₓ:ₙ̅⁽ᵍ⁾ with adjusted interest rate i′ = (1+i)/(1+g) - 1
///
/// Present value of geometrically increasing payments paid m times per year for n years,
/// with payments at the beginning of each period, starting after a deferment period of t years.
/// The payment grows geometrically at rate g each year.
pub fn t_gaa_x_n_(
    config: &MortTableConfig,
    entry_age: i32,
    t: i32,
    x: i32,
    n: i32,
    m: i32,
    g: f64,
) -> PolarsResult<f64> {
    let new_config = get_new_config_with_selected_table(config, entry_age)?;
    whole::t_gaa_x_n(&new_config, t, x, n, m, g)
}
