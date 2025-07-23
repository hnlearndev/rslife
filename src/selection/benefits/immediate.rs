use super::*;

//-----------------Basic------------------

/// Immediate whole life insurance:
/// Aₓ = Mₓ/Dₓ
///
/// Present value of $1 paid only if death occurs
pub fn A_x_(config: &MortTableConfig, entry_age: i32, x: i32) -> PolarsResult<f64> {
    let new_config = get_new_config_with_selected_table(config, entry_age)?;
    whole::A_x(&new_config, x)
}

/// Immediate term life insurance:
/// A¹ₓ:ₙ̅ = Aₓ - (Dₓ₊ₙ/Dₓ)·Aₓ₊ₙ = (Mₓ - Mₓ₊ₙ)/Dₓ
///
/// Present value of $1 paid only if death occurs within n years.
pub fn A_x1_n_(config: &MortTableConfig, entry_age: i32, x: i32, n: i32) -> PolarsResult<f64> {
    let new_config = get_new_config_with_selected_table(config, entry_age)?;
    whole::A_x1_n(&new_config, x, n)
}

/// Immediate pure endowment:
/// Aₓ:ₙ̅¹ = Dₓ₊ₙ/Dₓ
///
/// Present value of $1 paid if and only if the insured survives n years.
pub fn A_x_n1_(config: &MortTableConfig, entry_age: i32, x: i32, n: i32) -> PolarsResult<f64> {
    let new_config = get_new_config_with_selected_table(config, entry_age)?;
    whole::A_x_n1(&new_config, x, n)
}

/// Immediate Endowment insurance:
/// Aₓ:ₙ̅ = A¹ₓ:ₙ̅ + Aₓ:ₙ̅¹
///
/// $1 paid at death (if within n years) OR at survival to n years.
pub fn A_x_n_(config: &MortTableConfig, entry_age: i32, x: i32, n: i32) -> PolarsResult<f64> {
    let new_config = get_new_config_with_selected_table(config, entry_age)?;
    whole::A_x_n(&new_config, x, n)
}

//-----------------Increasing------------------

/// Immediate increasing whole life:
///  IAₓ = (Rₓ - Sₓ)/Dₓ
///
/// Death benefit increases by 1 each year: k paid if death in k-th year.
pub fn IA_x_(config: &MortTableConfig, entry_age: i32, x: i32) -> PolarsResult<f64> {
    let new_config = get_new_config_with_selected_table(config, entry_age)?;
    whole::IA_x(&new_config, x)
}

/// Immediate increasing term:
/// IA¹ₓ:ₙ̅ = IAₓ - (Dₓ₊ₙ/Dₓ) · (IAₓ₊ₙ + n · Aₓ₊ₙ)
///
/// Death benefit increases by 1 each year, pays only if death within n years.
pub fn IA_x1_n_(config: &MortTableConfig, entry_age: i32, x: i32, n: i32) -> PolarsResult<f64> {
    let new_config = get_new_config_with_selected_table(config, entry_age)?;
    whole::IA_x1_n(&new_config, x, n)
}

/// Immediate increasing pure endowment:
/// IAₓ:ₙ̅¹ = n · Dₓ₊ₙ/Dₓ = n . A_x_n1
///
/// Benefit of n paid if and only if the insured survives n years.
/// Note: This is actually just n times the pure endowment A_x_n1 and is of very little usage.
pub fn IA_x_n1_(config: &MortTableConfig, entry_age: i32, x: i32, n: i32) -> PolarsResult<f64> {
    let new_config = get_new_config_with_selected_table(config, entry_age)?;
    whole::IA_x_n1(&new_config, x, n)
}

/// Immediate endowment insurance:
/// IAₓ:ₙ̅ = IA¹ₓ:ₙ̅ + IA¹ₓ:ₙ̅
///
/// $1 paid at death (if within n years) OR at survival to n years.
pub fn IA_x_n_(config: &MortTableConfig, entry_age: i32, x: i32, n: i32) -> PolarsResult<f64> {
    let new_config = get_new_config_with_selected_table(config, entry_age)?;
    whole::IA_x_n(&new_config, x, n)
}

//-----------------Decreasing------------------
// Note: There should starting amount hence DAₓ is not applicable

/// Immediate decreasing term:
/// DA¹ₓ:ₙ̅ = (n+1) · A¹ₓ:ₙ̅ - IA¹ₓ:ₙ̅
///
/// Death benefit decreases by 1 each policy year (n in year 1, n-1 in year 2, ..., 1 in year n),
/// pays only if death occurs within n years.
pub fn DA_x1_n_(config: &MortTableConfig, entry_age: i32, x: i32, n: i32) -> PolarsResult<f64> {
    let new_config = get_new_config_with_selected_table(config, entry_age)?;
    whole::DA_x1_n(&new_config, x, n)
}

/// Immediate decreasing pure endowment:
/// DAₓ:ₙ̅¹ = Dₓ₊ₙ/Dₓ = Aₓ:ₙ̅¹
///
/// Death benefit increases by 1 each year, pays only if death within n years.
/// Note: This is actually just pure endowment A_x_n1 and is of very little usage.
pub fn DA_x_n1_(config: &MortTableConfig, entry_age: i32, x: i32, n: i32) -> PolarsResult<f64> {
    let new_config = get_new_config_with_selected_table(config, entry_age)?;
    whole::DA_x_n1(&new_config, x, n)
}

/// Immediate decreasing endowment insurance:
/// DAₓ:ₙ̅ = DA¹ₓ:ₙ̅ + DA¹ₓ:ₙ̅
///
/// $1 paid at death (if within n years) OR at survival to n years.
pub fn DA_x_n_(config: &MortTableConfig, entry_age: i32, x: i32, n: i32) -> PolarsResult<f64> {
    let new_config = get_new_config_with_selected_table(config, entry_age)?;
    whole::DA_x_n(&new_config, x, n)
}

//-----------------Geometric increasing------------------

/// Immediate geometric whole life:
/// Aₓ⁽ᵍ⁾ with adjusted interest rate i′ = (1+i)/(1+g) - 1
///
/// Death benefit grows geometrically at rate g each year.
pub fn gA_x_(config: &MortTableConfig, entry_age: i32, x: i32, g: f64) -> PolarsResult<f64> {
    let new_config = get_new_config_with_selected_table(config, entry_age)?;
    whole::gA_x(&new_config, x, g)
}

/// Immediate geometric n-year term:
/// A¹ₓ:ₙ̅⁽ᵍ⁾ with adjusted interest rate i′ = (1+i)/(1+g) - 1
///
/// Death benefit grows geometrically at rate g for n years.
pub fn gA_x1_n_(
    config: &MortTableConfig,
    entry_age: i32,
    x: i32,
    n: i32,
    g: f64,
) -> PolarsResult<f64> {
    let new_config = get_new_config_with_selected_table(config, entry_age)?;
    whole::gA_x1_n(&new_config, x, n, g)
}

/// Immediate geometric n-year pure endowment:
/// Aₓ:ₙ̅¹⁽ᵍ⁾ with adjusted interest rate i′ = (1+i)/(1+g) - 1
///
/// Death benefit grows geometrically at rate g for n years.
pub fn gA_x_n1_(
    config: &MortTableConfig,
    entry_age: i32,
    x: i32,
    n: i32,
    g: f64,
) -> PolarsResult<f64> {
    let new_config = get_new_config_with_selected_table(config, entry_age)?;
    whole::gA_x_n1(&new_config, x, n, g)
}

/// Immediate geometric n-year endowment:
/// Aₓ:ₙ̅⁽ᵍ⁾ with adjusted interest rate i′ = (1+i)/(1+g) - 1
///
/// Death benefit grows geometrically at rate g for n years.
pub fn gA_x_n_(
    config: &MortTableConfig,
    entry_age: i32,
    x: i32,
    n: i32,
    g: f64,
) -> PolarsResult<f64> {
    let new_config = get_new_config_with_selected_table(config, entry_age)?;
    whole::gA_x_n(&new_config, x, n, g)
}
