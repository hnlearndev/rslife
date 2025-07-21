#![allow(non_snake_case)]
use self::helpers::get_new_config;
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
///     radix: 100_000,
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
///     xml, radix: 100_000, pct: Some(1.0),
///     int_rate: Some(0.03), assumption: Some(AssumptionEnum::UDD),
/// };
/// let value = rslife::whole::benefits::Ax(&config, 30)?; // ~$0.40 for age 30
/// # Ok(())
/// # }
/// ```

//--------------------------Immediate-------------------------------
//-----------------Basic------------------

/// Immediate whole life insurance:
/// Aₓ = Mₓ/Dₓ
///
/// Present value of $1 paid only if death occurs
pub fn A_x(config: &MortTableConfig, x: i32) -> PolarsResult<f64> {
    let mx = get_value(config, x, "Mx")?;
    let dx = get_value(config, x, "Dx")?;
    Ok(mx / dx)
}

/// Immediate term life insurance:
/// A¹ₓ:ₙ̅ =  (Mₓ - Mₓ₊ₙ)/Dₓ
///
/// Present value of $1 paid only if death occurs within n years.
pub fn A_x1_n(config: &MortTableConfig, x: i32, n: i32) -> PolarsResult<f64> {
    let mx = get_value(config, x, "Mx")?;
    let mxn = get_value(config, x + n, "Mx")?;
    let dx = get_value(config, x, "Dx")?;
    let result = (mx - mxn) / dx;
    Ok(result)
}

/// Immediate pure endowment:
/// Aₓ:ₙ̅¹ = Dₓ₊ₙ/Dₓ
///
/// Present value of $1 paid if and only if the insured survives n years.
pub fn A_x_n1(config: &MortTableConfig, x: i32, n: i32) -> PolarsResult<f64> {
    let result = get_value(config, x, "Dx")? / get_value(config, x + n, "Dx")?;
    Ok(result)
}

/// Immediate Endowment insurance:
/// Aₓ:ₙ̅ = A¹ₓ:ₙ̅ + Aₓ:ₙ̅¹
///
/// $1 paid at death (if within n years) OR at survival to n years.
pub fn A_x_n(config: &MortTableConfig, x: i32, n: i32) -> PolarsResult<f64> {
    let result = A_x1_n(config, x, n)? + A_x_n1(config, x, n)?;
    Ok(result)
}

//-----------------Increasing------------------

/// Immediate increasing whole life:
/// IAₓ = Sₓ/Dₓ
///
/// Death benefit increases by 1 each year: k paid if death in k-th year.
pub fn IA_x(config: &MortTableConfig, x: i32) -> PolarsResult<f64> {
    let sx = get_value(config, x, "Sx")?;
    let dx = get_value(config, x, "Dx")?;
    Ok(sx / dx)
}

/// Immediate increasing term:
/// IA¹ₓ:ₙ̅ = IAₓ - (Dₓ₊ₙ/Dₓ) · (IAₓ₊ₙ - n  · Aₓ₊ₙ)
///
/// Death benefit increases by 1 each year, pays only if death within n years.
pub fn IA_x1_n(config: &MortTableConfig, x: i32, n: i32) -> PolarsResult<f64> {
    let iax = IA_x(config, x)?;
    let dxn = get_value(config, x + n, "Dx")?;
    let dx = get_value(config, x, "Dx")?;
    let iax_n = IA_x(config, x + n)?;
    let ax_n = A_x(config, x + n)?;
    let result = iax - (dxn / dx) * (iax_n - n as f64 * ax_n);
    Ok(result)
}

/// Immediate increasing pure endowment:
/// IAₓ:ₙ̅¹ = n · Dₓ₊ₙ/Dₓ = n . A_x_n1
///
/// Death benefit increases by 1 each year, pays only if death within n years.
/// This is actually just pure endowment n . A_x_n1 and is of very little usage.
pub fn IA_x_n1(config: &MortTableConfig, x: i32, n: i32) -> PolarsResult<f64> {
    let result = (n as f64) * A_x_n1(config, x, n)?;
    Ok(result)
}

/// Immediate endowment insurance:
/// IAₓ:ₙ̅ = IA¹ₓ:ₙ̅ + IA¹ₓ:ₙ̅
///
/// $1 paid at death (if within n years) OR at survival to n years.
pub fn IA_x_n(config: &MortTableConfig, x: i32, n: i32) -> PolarsResult<f64> {
    let result = IA_x1_n(config, x, n)? + IA_x_n1(config, x, n)?;
    Ok(result)
}

//-----------------Decreasing------------------
// Note: There should starting amount hence DAₓ is not applicable

/// Immediate decreasing term:
/// DA¹ₓ:ₙ̅ = (n+1) · A¹ₓ:ₙ̅ - IA¹ₓ:ₙ̅
///
/// Death benefit decreases by 1 each year, pays only if death occurs within n years.
pub fn DA_x1_n(config: &MortTableConfig, x: i32, n: i32) -> PolarsResult<f64> {
    let n_a_x1_n = n as f64 * A_x1_n(config, x, n)?;
    let ia_x1_n = IA_x1_n(config, x, n)?;
    let result = n_a_x1_n - ia_x1_n;
    Ok(result)
}

/// Immediate decreasing pure endowment:
/// DAₓ:ₙ̅¹ = Dₓ₊ₙ/Dₓ = Aₓ:ₙ̅¹
///
/// Death benefit increases by 1 each year, pays only if death within n years.
/// This is actually just pure endowment A_x_n1 and is of very little usage.
pub fn DA_x_n1(config: &MortTableConfig, x: i32, n: i32) -> PolarsResult<f64> {
    let result = A_x_n1(config, x, n)?;
    Ok(result)
}

/// Immediate endowment insurance:
/// DAₓ:ₙ̅ = IA¹ₓ:ₙ̅ + IA¹ₓ:ₙ̅
///
/// $1 paid at death (if within n years) OR at survival to n years.
pub fn IA_x_n(config: &MortTableConfig, x: i32, n: i32) -> PolarsResult<f64> {
    let result = IA_x1_n(config, x, n)? + IA_x_n1(config, x, n)?;
    Ok(result)
}

//-----------------Geometric increasing------------------

/// Immediate geometric whole life:
/// Aₓ⁽ᵍ⁾ with adjusted interest rate i′ = (1+i)/(1+g) - 1
///
/// Death benefit grows geometrically at rate g each year.
pub fn gA_x(config: &MortTableConfig, x: i32, g: f64) -> PolarsResult<f64> {
    let new_config = get_new_config(config, g);
    let result = A_x(&new_config, x)?;
    Ok(result)
}

/// Immediate geometric n-year term:
/// A¹ₓ:ₙ̅⁽ᵍ⁾ with adjusted interest rate i′ = (1+i)/(1+g) - 1
///
/// Death benefit grows geometrically at rate g for n years.
pub fn gA_x1_n(config: &MortTableConfig, x: i32, n: i32, g: f64) -> PolarsResult<f64> {
    let new_config = get_new_config(config, g);
    let result = A_x1_n(&new_config, x, n)?;
    Ok(result)
}

/// Immediate geometric n-year pure endowment:
/// Aₓ:ₙ̅¹⁽ᵍ⁾ with adjusted interest rate i′ = (1+i)/(1+g) - 1
///
/// Death benefit grows geometrically at rate g for n years.
pub fn gA_x_n1(config: &MortTableConfig, x: i32, n: i32, g: f64) -> PolarsResult<f64> {
    let new_config = get_new_config(config, g);
    let result = A_x_n1(&new_config, x, n)?;
    Ok(result)
}

/// Immediate geometric n-year endowment:
/// Aₓ:ₙ̅⁽ᵍ⁾ with adjusted interest rate i′ = (1+i)/(1+g) - 1
///
/// Death benefit grows geometrically at rate g for n years.
pub fn gA_x_n(config: &MortTableConfig, x: i32, n: i32, g: f64) -> PolarsResult<f64> {
    let new_config = get_new_config(config, g);
    let result = A_x_n(&new_config, x, n)?;
    Ok(result)
}

//-------------------------------------------------------------
// Due
//-------------------------------------------------------------
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

//-------------------------------------------------------------
// Defered
//-------------------------------------------------------------

//------------------------Immediate----------------------------------
//-----------------Basic------------------
/// Deferred whole life:
/// ₜAₓ = Aₓ - Aₓ:ₜ̄
///
/// $1 paid at death, but only if death occurs after t years.
pub fn t_A_x(config: &MortTableConfig, t: i32, x: i32) -> PolarsResult<f64> {
    let result = A_x(config, x)? - A_x_n(config, x, t)?;
    Ok(result)
}

/// Deferred term:
/// ₜA¹ₓ:ₙ̅ = ₜAₓ - ₜ₊ₙAₓ
///
/// $1 paid at death between t and t+n years after issue.
pub fn t_A_x1_n(config: &MortTableConfig, t: i32, x: i32, n: i32) -> PolarsResult<f64> {
    let result = t_A_x(config, t, x)? - t_A_x(config, t + n, x)?;
    Ok(result)
}

/// Deferred pure endowment:
/// ₜAₓ:ₙ̅¹ = Dₓ₊ₜ₊ₙ/Dₓ
///
/// $1 paid if insured survives both deferral period t and additional period n.
pub fn t_A_x_n1(config: &MortTableConfig, t: i32, x: i32, n: i32) -> PolarsResult<f64> {
    let dx = get_value(config, x, "Dx")?;
    let dxnt = get_value(config, x + t + n, "Dx")?;
    let result = dxnt / dx;
    Ok(result)
}

/// Deferred endowment:
/// ₜAₓ:ₙ = ₜA¹ₓ:ₙ̅ + ₜAₓ:ₙ̅¹
///
/// $1 paid at death if it occurs between t and t+n years after issue, or at survival to t+n years.
pub fn t_A_x_n(config: &MortTableConfig, t: i32, x: i32, n: i32) -> PolarsResult<f64> {
    let result = t_A_x1_n(config, t, x, n)? + t_A_x_n1(config, t, x, n)?;
    Ok(result)
}

//-----------------Increasing------------------
/// Deferred increasing whole life:
/// ₜIAₓ = ₜpₓ . IAₓ₊ₜ = Dₓ₊ₜ/Dₓ . IAₓ₊ₜ
///
/// $1 paid at death, with benefit increasing by 1 each year, but only if death occurs after a deferral period of t years.
pub fn t_IA_x(config: &MortTableConfig, t: i32, x: i32) -> PolarsResult<f64> {
    let ia_xt = IA_x(config, x + t)?;
    let dx_xt = get_value(config, x + t, "Dx")?;
    let dx_x = get_value(config, x, "Dx")?;
    let result = ia_xt * dx_xt / dx_x;
    Ok(result)
}

/// Deferred increasing term:
/// ₜIA¹ₓ:ₙ̅ = ₜ|IAₓ - ₜ₊ₙ|IAₓ - (Sₓ₊ₜ - Sₓ₊ₜ₊ₙ)/Dₓ
///
/// Death benefit increases by 1 each year, pays only if death occurs between t and t+n years.
pub fn t_IA_x1_n(config: &MortTableConfig, t: i32, x: i32, n: i32) -> PolarsResult<f64> {
    let t_ia_x = t_IA_x(config, t, x)?;
    let tpn_ia_x = t_IA_x(config, t + n, x)?;
    let sx_t = get_value(config, x + t, "Sx")?;
    let sx_tn = get_value(config, x + t + n, "Sx")?;
    let dx = get_value(config, x, "Dx")?;
    let result = t_ia_x - tpn_ia_x - (sx_t - sx_tn) / dx;
    Ok(result)
}

/// Deferred increasing pure endowment:
/// ₜIAₓ:ₙ̅¹ = n·Dₓ₊ₜ₊ₙ/Dₓ
///
/// Benefit of n paid if insured survives both deferral period t and additional period n.
pub fn t_IA_x_n1(config: &MortTableConfig, t: i32, x: i32, n: i32) -> PolarsResult<f64> {
    let dx = get_value(config, x, "Dx")?;
    let dxnt = get_value(config, x + t + n, "Dx")?;
    let result = (n as f64 * dxnt) / dx;
    Ok(result)
}

/// Deferred increasing endowment:
/// ₜIAₓ:ₙ̅ = ₜIA¹ₓ:ₙ̅ + ₜIAₓ:ₙ̅¹
///
/// Death benefit increases by 1 each year if death occurs between t and t+n years, or benefit of n if survives to t+n years.
pub fn t_IA_x_n(config: &MortTableConfig, t: i32, x: i32, n: i32) -> PolarsResult<f64> {
    let result = t_IA_x1_n(config, t, x, n)? + t_IA_x_n1(config, t, x, n)?;
    Ok(result)
}

//-----------------Geometric increasing------------------
/// Deferred geometric whole life:
/// ₜAₓ⁽ᵍ⁾ with adjusted interest rate i′ = (1+i)/(1+g) - 1
///
/// Death benefit grows geometrically at rate g each year, but only if death occurs after t years.
pub fn t_gA_x(config: &MortTableConfig, t: i32, x: i32, g: f64) -> PolarsResult<f64> {
    let new_config = get_new_config(config, g);
    let result = t_A_x(&new_config, t, x)?;
    Ok(result)
}

/// Deferred geometric term:
/// ₜA¹ₓ:ₙ̅⁽ᵍ⁾ with adjusted interest rate i′ = (1+i)/(1+g) - 1
///
/// Death benefit grows geometrically at rate g for n years, but only if death occurs between t and t+n years.
pub fn t_gA_x1_n(config: &MortTableConfig, t: i32, x: i32, n: i32, g: f64) -> PolarsResult<f64> {
    let new_config = get_new_config(config, g);
    let result = t_A_x1_n(&new_config, t, x, n)?;
    Ok(result)
}

/// Deferred geometric pure endowment:
/// ₜAₓ:ₙ̅¹⁽ᵍ⁾ with adjusted interest rate i′ = (1+i)/(1+g) - 1
///
/// Geometric benefit paid if insured survives both deferral period t and additional period n.
pub fn t_gA_x_n1(config: &MortTableConfig, t: i32, x: i32, n: i32, g: f64) -> PolarsResult<f64> {
    let new_config = get_new_config(config, g);
    let result = t_A_x_n1(&new_config, t, x, n)?;
    Ok(result)
}

/// Deferred geometric endowment:
/// ₜAₓ:ₙ̅⁽ᵍ⁾ with adjusted interest rate i′ = (1+i)/(1+g) - 1
///
/// Death benefit grows geometrically at rate g if death occurs between t and t+n years, or at survival to t+n years.
pub fn t_gA_x_n(config: &MortTableConfig, t: i32, x: i32, n: i32, g: f64) -> PolarsResult<f64> {
    let new_config = get_new_config(config, g);
    let result = t_A_x_n(&new_config, t, x, n)?;
    Ok(result)
}

//------------------------Due----------------------------------
//-----------------Basic------------------
/// Deferred due whole life:
/// ₜÄₓ = Äₓ - Äₓ:ₜ̄
///
/// $1 paid at death, but only if death occurs after t years (due basis).
pub fn t_AA_x(config: &MortTableConfig, t: i32, x: i32) -> PolarsResult<f64> {
    let result = AA_x(config, x)? - AA_x_n(config, x, t)?;
    Ok(result)
}

/// Deferred due term:
/// ₜÄ¹ₓ:ₙ̅ = ₜÄₓ - ₜ₊ₙÄₓ
///
/// $1 paid at death between t and t+n years after issue (due basis).
pub fn t_AA_x1_n(config: &MortTableConfig, t: i32, x: i32, n: i32) -> PolarsResult<f64> {
    let result = t_AA_x(config, t, x)? - t_AA_x(config, t + n, x)?;
    Ok(result)
}

/// Deferred due pure endowment:
/// ₜÄₓ:ₙ̅¹ = (1+i) × ₜAₓ:ₙ̅¹
///
/// $1 paid if insured survives both deferral period t and additional period n (due basis).
pub fn t_AA_x_n1(config: &MortTableConfig, t: i32, x: i32, n: i32) -> PolarsResult<f64> {
    let i = config.int_rate.unwrap_or(0.0);
    let result = t_A_x_n1(config, t, x, n)? * (1.0 + i);
    Ok(result)
}

/// Deferred due endowment:
/// ₜÄₓ:ₙ̅ = ₜÄ¹ₓ:ₙ̅ + ₜÄₓ:ₙ̅
///
/// $1 paid at death if it occurs between t and t+n years after issue, or at survival to t+n years (due basis).
pub fn t_AA_x_n(config: &MortTableConfig, t: i32, x: i32, n: i32) -> PolarsResult<f64> {
    let result = t_AA_x1_n(config, t, x, n)? + t_AA_x_n1(config, t, x, n)?;
    Ok(result)
}

//-----------------Increasing------------------
/// Deferred due increasing whole life:
/// ₜIÄₓ = (1+i)(ₜIAₓ + ₜäₓ)
///
/// Death benefit increases by 1 each year, but only if death occurs after a deferral period of t years (due basis).
pub fn t_IAA_x(config: &MortTableConfig, t: i32, x: i32) -> PolarsResult<f64> {
    let dx_xt = get_value(config, x + t, "Dx")?;
    let dx_x = get_value(config, x, "Dx")?;
    let result = IAA_x(config, x + t)? * dx_xt / dx_x;
    Ok(result)
}

/// Deferred due increasing term:
/// ₜIÄ¹ₓ:ₙ̅ = ₜIÄₓ - ₜ₊ₙIÄₓ
///
/// Death benefit increases by 1 each year, pays only if death occurs between t and t+n years (due basis).
pub fn t_IAA_x1_n(config: &MortTableConfig, t: i32, x: i32, n: i32) -> PolarsResult<f64> {
    let result = t_IAA_x(config, t, x)? - t_IAA_x(config, t + n, x)?;
    Ok(result)
}

/// Deferred due increasing pure endowment:
/// ₜIÄₓ:ₙ̅¹ = ₜIAₓ:ₙ̅¹ · (1+i)
///
/// Benefit of n paid if insured survives both deferral period t and additional period n (due basis).
pub fn t_IAA_x_n1(config: &MortTableConfig, t: i32, x: i32, n: i32) -> PolarsResult<f64> {
    let i = config.int_rate.unwrap_or(0.0);
    let result = t_IA_x_n1(config, t, x, n)? * (1.0 + i);
    Ok(result)
}

/// Deferred due increasing endowment:
/// ₜIÄₓ:ₙ̅ = ₜIÄ¹ₓ:ₙ̅ + ₜIÄₓ:ₙ̅¹
///
/// Death benefit increases by 1 each year if death occurs between t and t+n years, or benefit of n if survives to t+n years (due basis).
pub fn t_IAA_x_n(config: &MortTableConfig, t: i32, x: i32, n: i32) -> PolarsResult<f64> {
    let result = t_IAA_x1_n(config, t, x, n)? + t_IAA_x_n1(config, t, x, n)?;
    Ok(result)
}

//-----------------Geometric increasing------------------
/// Deferred due geometric whole life:
/// ₜÄₓ⁽ᵍ⁾ with adjusted interest rate i′ = (1+i)/(1+g) - 1
///
/// Death benefit grows geometrically at rate g each year, but only if death occurs after t years (due basis).
pub fn t_gAA_x(config: &MortTableConfig, t: i32, x: i32, g: f64) -> PolarsResult<f64> {
    let new_config = get_new_config(config, g);
    let result = t_AA_x(&new_config, t, x)?;
    Ok(result)
}

/// Deferred due geometric term:
/// ₜÄ¹ₓ:ₙ̅⁽ᵍ⁾ with adjusted interest rate i′ = (1+i)/(1+g) - 1
///
/// Death benefit grows geometrically at rate g for n years, but only if death occurs between t and t+n years (due basis).
pub fn t_gAA_x1_n(config: &MortTableConfig, t: i32, x: i32, n: i32, g: f64) -> PolarsResult<f64> {
    let new_config = get_new_config(config, g);
    let result = t_AA_x1_n(&new_config, t, x, n)?;
    Ok(result)
}

/// Deferred due geometric pure endowment:
/// ₜÄₓ:ₙ̅¹⁽ᵍ⁾ with adjusted interest rate i′ = (1+i)/(1+g) - 1
///
/// Geometric benefit paid if insured survives both deferral period t and additional period n (due basis).
pub fn t_gAA_x_n1(config: &MortTableConfig, t: i32, x: i32, n: i32, g: f64) -> PolarsResult<f64> {
    let new_config = get_new_config(config, g);
    let result = t_AA_x_n1(&new_config, t, x, n)?;
    Ok(result)
}

/// Deferred due geometric endowment:
/// ₜÄₓ:ₙ̅⁽ᵍ⁾ with adjusted interest rate i′ = (1+i)/(1+g) - 1
///
/// Death benefit grows geometrically at rate g if death occurs between t and t+n years, or at survival to t+n years (due basis).
pub fn t_gAA_x_n(config: &MortTableConfig, t: i32, x: i32, n: i32, g: f64) -> PolarsResult<f64> {
    let new_config = get_new_config(config, g);
    let result = t_AA_x_n(&new_config, t, x, n)?;
    Ok(result)
}
