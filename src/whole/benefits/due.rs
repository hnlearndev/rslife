use super::*;
use self::immediate::*;

// Note:
// Due benefits means paid at begining of year of death, not end.
// This is counterintuitive but mathematical convention in actuarial science.

//-----------------Basic------------------

/// Due whole life insurance:
/// Äₓ = Aₓ + 1
///
/// Present value of $1 paid only if death occurs
pub fn AA_x(config: &MortTableConfig, x: i32) -> PolarsResult<f64> {
    let a_x = A_x(config, x)?;
    let i = config.int_rate.unwrap_or(0.0);
    Ok(a_x * (1.0 + i))
}

/// Due term life insurance:
/// Ä¹ₓ:ₙ̅ = (1 + i)A¹ₓ:ₙ̅ - i . Aₓ:ₙ̅¹
///
/// Present value of $1 paid only if death occurs within n years.
pub fn AA_x1_n(config: &MortTableConfig, x: i32, n: i32) -> PolarsResult<f64> {
    let i = config.int_rate.unwrap_or(0.0);
    let a_x1_n = A_x1_n(config, x, n)?;
    let a_x_n1 = A_x_n1(config, x, n)?;
    let result = (1.0 + i) * a_x1_n - i * a_x_n1;
    Ok(result)
}

/// Due pure endowment:
/// Äₓ:ₙ̅¹ = Aₓ:ₙ̅¹.(1 + i)
///
/// Present value of $1 paid if and only if the insured survives n years.
pub fn AA_x_n1(config: &MortTableConfig, x: i32, n: i32) -> PolarsResult<f64> {
    let i = config.int_rate.unwrap_or(0.0);
    let result = A_x_n1(config, x, n)? * (1.0 + i);
    Ok(result)
}

/// Due endowment insurance:
/// Äₓ:ₙ̅ = Ä¹ₓ:ₙ̅ + Äₓ:ₙ̅¹
///
/// $1 paid at death (if within n years) OR at survival to n years.
pub fn AA_x_n(config: &MortTableConfig, x: i32, n: i32) -> PolarsResult<f64> {
    let result = AA_x1_n(config, x, n)? + AA_x_n1(config, x, n)?;
    Ok(result)
}

//-----------------Increasing------------------

/// Due increasing whole life:
/// IÄₓ = Äₓ + (1+i)·IAₓ = (Rₓ + Sₓ)/Dₓ
///
/// Death benefit increases by 1 each year: k paid if death in k-th year.
pub fn IAA_x(config: &MortTableConfig, x: i32) -> PolarsResult<f64> {
    let i = config.int_rate.unwrap_or(0.0);
    let aax = AA_x(config, x)?;
    let iax = IA_x(config, x)?;
    let result = aax + (1.0 + i) * iax;
    Ok(result)
}

/// Due increasing term:
/// IÄ¹ₓ:ₙ̅ = Ä¹ₓ:ₙ̅ + IA¹ₓ:ₙ̅
///
/// Death benefit increases by 1 each year, pays only if death within n years.
pub fn IAA_x1_n(config: &MortTableConfig, x: i32, n: i32) -> PolarsResult<f64> {
    let i = config.int_rate.unwrap_or(0.0);
    let ia_x1_n = IA_x1_n(config, x, n)?;
    let sxm1 = get_value(config, x - 1, "Sx")?;
    let sxnp1 = get_value(config, x + n - 1, "Sx")?;
    let dxnp1 = get_value(config, x + n - 1, "Dx")?;
    let dx = get_value(config, x, "Dx")?;
    let result = (1.0 + i) * ia_x1_n + (sxm1 - sxnp1 - n as f64 * dxnp1) / dx;
    Ok(result)
}

/// Due increasing pure endowment:
/// IÄₓ:ₙ̅¹ = (1+i) × IAₓ:ₙ̅¹
///
/// Death benefit increases by 1 each year, pays only if death within n years.
pub fn IAA_x_n1(config: &MortTableConfig, x: i32, n: i32) -> PolarsResult<f64> {
    let i = config.int_rate.unwrap_or(0.0);
    let result = IA_x_n1(config, x, n)? * (1.0 + i);
    Ok(result)
}

/// Due increasing endowment insurance:
/// IÄₓ:ₙ̅ = IÄ¹ₓ:ₙ̅ + IÄₓ:ₙ̅¹
///
/// $1 paid at death (if within n years) OR at survival to n years.
pub fn IAA_x_n(config: &MortTableConfig, x: i32, n: i32) -> PolarsResult<f64> {
    let result = IAA_x1_n(config, x, n)? + IAA_x_n1(config, x, n)?;
    Ok(result)
}

//-----------------Decreasing------------------
// Note: There should starting amount hence DAₓ is not applicable

/// Due decreasing term:
// DÄ¹ₓ:ₙ̅ = (1+i) · DA¹ₓ:ₙ̅ - i · A¹ₓ:ₙ̅
///
/// Death benefit decreases by 1 each year, pays only if death occurs within n years.
pub fn DAA_x1_n(config: &MortTableConfig, x: i32, n: i32) -> PolarsResult<f64> {
    let i = config.int_rate.unwrap_or(0.0);
    let da_x1_n = DA_x1_n(config, x, n)?;
    let a_x1_n = A_x1_n(config, x, n)?;
    let result = (1.0 + i) * da_x1_n - i * a_x1_n;
    Ok(result)
}

/// Immediate decreasing pure endowment:
/// DÄₓ:ₙ̅¹ = Äₓ:ₙ̅¹
///
/// Due decreasing pure endowment:
/// DÄₓ:ₙ̅¹ = Äₓ:ₙ̅¹
///
/// Present value of a decreasing pure endowment.
/// This is equivalent to the due pure endowment (Äₓ:ₙ̅¹), as the decreasing structure does not affect the benefit.
/// Rarely used in practice.
pub fn DAA_x_n1(config: &MortTableConfig, x: i32, n: i32) -> PolarsResult<f64> {
    let result = AA_x_n1(config, x, n)?;
    Ok(result)
}

/// Due decreasing endowment insurance:
/// DAₓ:ₙ̅ = DA¹ₓ:ₙ̅ + DAₓ:ₙ̅¹
///
/// Pays a benefit that decreases each year if death occurs within n years,
/// or pays a pure endowment if the insured survives n years.
pub fn DAA_x_n(config: &MortTableConfig, x: i32, n: i32) -> PolarsResult<f64> {
    let result = DAA_x1_n(config, x, n)? + DAA_x_n1(config, x, n)?;
    Ok(result)
}

//-----------------Geometric increasing------------------

/// Due geometric whole life:
/// Äₓ⁽ᵍ⁾ with adjusted interest rate i′ = (1+i)/(1+g) - 1
///
/// Death benefit grows geometrically at rate g each year.
pub fn gAA_x(config: &MortTableConfig, x: i32, g: f64) -> PolarsResult<f64> {
    let new_config = get_new_config(config, g);
    let result = AA_x(&new_config, x)?;
    Ok(result)
}

/// Due geometric n-year term:
/// Ä¹ₓ:ₙ̅⁽ᵍ⁾ with adjusted interest rate i′ = (1+i)/(1+g) - 1
///
/// Death benefit grows geometrically at rate g for n years.
pub fn gAA_x1_n(config: &MortTableConfig, x: i32, n: i32, g: f64) -> PolarsResult<f64> {
    let new_config = get_new_config(config, g);
    let result = AA_x1_n(&new_config, x, n)?;
    Ok(result)
}

/// Due geometric n-year pure endowment:
/// Äₓ:ₙ̅¹⁽ᵍ⁾ with adjusted interest rate i′ = (1+i)/(1+g) - 1
///
/// Death benefit grows geometrically at rate g for n years.
pub fn gAA_x_n1(config: &MortTableConfig, x: i32, n: i32, g: f64) -> PolarsResult<f64> {
    let new_config = get_new_config(config, g);
    let result = AA_x_n1(&new_config, x, n)?;
    Ok(result)
}

/// Due geometric n-year endowment:
/// Äₓ:ₙ̅⁽ᵍ⁾ with adjusted interest rate i′ = (1+i)/(1+g) - 1
///
/// Death benefit grows geometrically at rate g for n years.
pub fn gAA_x_n(config: &MortTableConfig, x: i32, n: i32, g: f64) -> PolarsResult<f64> {
    let new_config = get_new_config(config, g);
    let result = AA_x_n(&new_config, x, n)?;
    Ok(result)
}

