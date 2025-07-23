use super::*;

// Note: due.rs and immediate.rs are kept independent.
// Their relationship will be used for testing purposes.

//-----------------Basic------------------

/// Life annuity-immediate:
/// aₓ⁽ᵐ⁾ = (1/m) × (v¹ᐟᵐ - vˣ)/(1 - v¹ᐟᵐ)
///
/// Present value of 1/m paid m times per year for life, with payments at the end of each period.
pub fn a_x_(config: &MortTableConfig, entry_age: i32, x: i32, m: i32) -> PolarsResult<f64> {
    let new_config = get_new_config_with_selected_table(config, entry_age)?;
    whole::a_x(&new_config, x, m)
}

/// Temporary annuity-immediate:
/// aₓ:ₙ̅⁽ᵐ⁾ = aₓ⁽ᵐ⁾ - (Dₓ₊ₜ/Dₓ) · aₓ₊ₙ⁽ᵐ⁾
///
/// Present value of 1/m paid m times per year for n years, with payments at the end of each period.
pub fn a_x_n_(
    config: &MortTableConfig,
    entry_age: i32,
    x: i32,
    n: i32,
    m: i32,
) -> PolarsResult<f64> {
    let new_config = get_new_config_with_selected_table(config, entry_age)?;
    whole::a_x_n(&new_config, x, n, m)
}

//-----------------Increasing------------------

/// Increasing life annuity-immediate:
/// Iaₓ⁽ᵐ⁾ = (Rₓ - Nₓ · i⁽ᵐ⁾/(m·i))/(Dₓ · d⁽ᵐ⁾)
/// where d⁽ᵐ⁾ = m[1 - (1+i)⁻¹/ᵐ]
/// and i⁽ᵐ⁾ = m[(1+i)¹/ᵐ - 1]
///
/// Present value of an increasing life annuity-immediate: payments of 1/m made m times per year for life, with each annual payment increasing by 1.
pub fn Ia_x_(config: &MortTableConfig, entry_age: i32, x: i32, m: i32) -> PolarsResult<f64> {
    let new_config = get_new_config_with_selected_table(config, entry_age)?;
    whole::Ia_x(&new_config, x, m)
}

///  Increasing temporary annuity-immediate:
/// (Ia)ₓ:ₙ̅⁽ᵐ⁾ = (Ia)ₓ⁽ᵐ⁾ - (Dₓ₊ₙ/Dₓ) · [(Ia)ₓ₊ₙ⁽ᵐ⁾ + (n/m) · aₓ₊ₙ⁽ᵐ⁾]
///
/// Present value of an increasing life annuity-immediate: payments of 1/m made m times per year for n years, with each annual payment increasing by 1.
pub fn Ia_x_n_(
    config: &MortTableConfig,
    entry_age: i32,
    x: i32,
    n: i32,
    m: i32,
) -> PolarsResult<f64> {
    let new_config = get_new_config_with_selected_table(config, entry_age)?;
    whole::Ia_x_n(&new_config, x, n, m)
}

//-----------------Decreasing------------------

///  Decreasing temporary annuity-immediate:
/// Daₓ:ₙ̅⁽ᵐ⁾  = (n+1) · aₓ:ₙ̅⁽ᵐ⁾ -  Iaₓ:ₙ̅⁽ᵐ⁾
///
/// Present value of an decreasing life annuity-immediate: payments of n/m made m times per year for n years, with each annual payment decreasing by 1.
pub fn Da_x_n_(
    config: &MortTableConfig,
    entry_age: i32,
    x: i32,
    n: i32,
    m: i32,
) -> PolarsResult<f64> {
    let new_config = get_new_config_with_selected_table(config, entry_age)?;
    whole::Da_x_n(&new_config, x, n, m)
}

//-----------------Geometric increasing------------------

/// Geometric life annuity-immediate:
/// aₓ⁽ᵍ⁾ with adjusted interest rate i′ = (1+i)/(1+g) - 1
///
/// Present value of geometrically increasing payments paid m times per year for life, with payments at the end of each period.
/// The payment grows geometrically at rate g each year.
pub fn ga_x_(
    config: &MortTableConfig,
    entry_age: i32,
    x: i32,
    m: i32,
    g: f64,
) -> PolarsResult<f64> {
    let new_config = get_new_config_with_selected_table(config, entry_age)?;
    whole::ga_x(&new_config, x, m, g)
}

/// Geometric temporary annuity-immediate:
/// aₓ:ₙ̅⁽ᵍ⁾ with adjusted interest rate i′ = (1+i)/(1+g) - 1
///
/// Present value of geometrically increasing payments paid m times per year for n years, with payments at the end of each period.
/// The payment grows geometrically at rate g each year.
pub fn ga_x_n_(
    config: &MortTableConfig,
    entry_age: i32,
    x: i32,
    n: i32,
    m: i32,
    g: f64,
) -> PolarsResult<f64> {
    let new_config = get_new_config_with_selected_table(config, entry_age)?;
    whole::ga_x_n(&new_config, x, n, m, g)
}
