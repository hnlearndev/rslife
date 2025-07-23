use super::*;

// Note: due.rs and immediate.rs are kept independent.
// Their relationship will be used for testing purposes.

//-----------------Basic------------------

/// Due life annuity-due:
/// äₓ⁽ᵐ⁾ = (1/m) × [(1 - vˣ)/(1 - v¹/ᵐ)]
///
/// Present value of 1/m paid m times per year for life.
pub fn aa_x_(config: &MortTableConfig, entry_age: i32, x: i32, m: i32) -> PolarsResult<f64> {
    let new_config = get_new_config_with_selected_table(config, entry_age)?;
    whole::aa_x(&new_config, x, m)
}

/// Due temporary annuity-due:
/// äₓ:ₙ̅⁽ᵐ⁾ = äₓ⁽ᵐ⁾ - (Dₓ₊ₙ/Dₓ) · äₓ₊ₙ⁽ᵐ⁾
///
/// Present value of 1/m paid m times per year for up to n years.
pub fn aa_x_n_(
    config: &MortTableConfig,
    entry_age: i32,
    x: i32,
    n: i32,
    m: i32,
) -> PolarsResult<f64> {
    let new_config = get_new_config_with_selected_table(config, entry_age)?;
    whole::aa_x_n(&new_config, x, n, m)
}

//-----------------Increasing------------------
/// Increasing life annuity-due:
/// Iäₓ⁽ᵐ⁾ = (Rₓ - Nₓ)/(Dₓ · d⁽ᵐ⁾)
/// where d⁽ᵐ⁾ = m[1 - (1+i)⁻¹/ᵐ]
///
/// Present value of an increasing life annuity-due: payments of 1/m made m times per year for life, with each annual payment increasing by 1.
pub fn Iaa_x_(config: &MortTableConfig, entry_age: i32, x: i32, m: i32) -> PolarsResult<f64> {
    let new_config = get_new_config_with_selected_table(config, entry_age)?;
    whole::Iaa_x(&new_config, x, m)
}

/// Due increasing temporary life annuity-due:
/// Iäₓ:ₙ̅⁽ᵐ⁾ = Iäₓ⁽ᵐ⁾ - (Dₓ₊ₙ/Dₓ) · (Iäₓ₊ₙ⁽ᵐ⁾ + n · äₓ₊ₙ⁽ᵐ⁾)
///
/// Present value of an increasing life annuity-due: payments of 1/m made m times per year for n years, with each annual payment increasing by 1.
pub fn Iaa_x_n_(
    config: &MortTableConfig,
    entry_age: i32,
    x: i32,
    m: i32,
    n: i32,
) -> PolarsResult<f64> {
    let new_config = get_new_config_with_selected_table(config, entry_age)?;
    whole::Iaa_x_n(&new_config, x, m, n)
}

//-----------------Decreasing------------------
/// Decreasing temporary life annuity-due:
/// Däₓ:ₙ̅⁽ᵐ⁾ = (n+1) · äₓ:ₙ̅⁽ᵐ⁾ - Iäₓ:ₙ̅⁽ᵐ⁾
///
/// Present value of an decreasing life annuity-due: payments of n/m made m times per year for n years, with each annual payment decreasing by 1.
pub fn Daa_x_n_(
    config: &MortTableConfig,
    entry_age: i32,
    x: i32,
    m: i32,
    n: i32,
) -> PolarsResult<f64> {
    let new_config = get_new_config_with_selected_table(config, entry_age)?;
    whole::Daa_x_n(&new_config, x, m, n)
}

//-----------------Geometric increasing------------------
/// Geometric  increasing life annuity-due:
/// äₓ⁽ᵍ⁾ with adjusted interest rate i′ = (1+i)/(1+g) - 1
///
/// Payments of 1/m are made m times per year for life, starting immediately,
/// with each annual payment increasing by a factor of (1+g) each year (i.e., geometric progression).
pub fn gaa_x_(
    config: &MortTableConfig,
    entry_age: i32,
    x: i32,
    m: i32,
    g: f64,
) -> PolarsResult<f64> {
    let new_config = get_new_config_with_selected_table(config, entry_age)?;
    whole::gaa_x(&new_config, x, m, g)
}

/// Geometric increasing temporary annuity-due:
/// äₓ:ₙ̅⁽ᵍ⁾ with adjusted interest rate i′ = (1+i)/(1+g) - 1
///
/// Payments of 1/m are made m times per year for n years, starting immediately,
/// with each annual payment increasing by a factor of (1+g) each year (i.e., geometric progression).
pub fn gaa_x_n_(
    config: &MortTableConfig,
    entry_age: i32,
    x: i32,
    n: i32,
    m: i32,
    g: f64,
) -> PolarsResult<f64> {
    let new_config = get_new_config_with_selected_table(config, entry_age)?;
    whole::gaa_x_n(&new_config, x, n, m, g)
}
