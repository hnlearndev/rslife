#![allow(non_snake_case)]
use self::helpers::get_new_config;
use super::*;

/// # Annuities Module
///
/// Actuarial functions for calculating present values of annuity products.
/// Uses commutation functions from mortality tables with interest rates.
///
/// ## Core Functions
///
/// - [`aaxn`] - Temporary annuity: äₓ:ₙ⁽ᵐ⁾
/// - [`taax`] - Deferred life annuity: ₜäₓ⁽ᵐ⁾
/// - [`Iaax`] - Increasing life annuity: (Iä)ₓ⁽ᵐ⁾
/// - [`gIaax`] - Geometric growth annuity
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
/// let temp_annuity = rslife::whole::annuities::aaxn(&config, 65, 20, 12)?;  // Monthly for 20 years
/// let deferred = rslife::whole::annuities::taax(&config, 55, 10, 1)?;       // Start in 10 years
/// # Ok(())
/// # }
/// ```

//--------------------------Immediate-------------------------------
//-----------------Basic------------------

/// Due life annuity-due:
/// äₓ⁽ᵐ⁾ = (1/m) × [(1 - vˣ)/(1 - v¹/ᵐ)]
///
/// Present value of 1/m paid m times per year for life, starting immediately.
fn aa_x(config: &MortTableConfig, x: i32, m: i32) -> PolarsResult<f64> {
    let i = config.int_rate.unwrap_or(0.0);
    let v = 1.0 / (1.0 + i);
    let numerator = 1.0 - v.powi(x);
    let denominator = 1.0 - v.powf(1.0 / m as f64);
    let result = (1.0 / m as f64) * (numerator / denominator);
    Ok(result)
}

/// Due temporary annuity-due:
/// äₓ:ₙ̅⁽ᵐ⁾ = äₓ₊ₙ̅⁽ᵐ⁾ - äₓ⁽ᵐ⁾
///
/// Present value of 1/m paid m times per year for at most n years.
pub fn aa_x_n(config: &MortTableConfig, x: i32, n: i32, m: i32) -> PolarsResult<f64> {
    let result = aa_x(config, x, m)? - aa_x(config, x + n, m)?;
    Ok(result)
}

//-----------------Increasing------------------
/// Due life annuity-due:
/// Iäₓ⁽ᵐ⁾ = (äₓ⁽ᵐ⁾ + Aₓ) / (d⁽ᵐ⁾)² where d⁽ᵐ⁾ = m[1 - (1+i)⁻¹/ᵐ]
///
/// Present value of 1/m paid m times per year for life, starting immediately.
fn Iaa_x(config: &MortTableConfig, x: i32, m: i32) -> PolarsResult<f64> {
    let i = config.int_rate.unwrap_or(0.0);
    let m_f = m as f64;
    let d_m = m_f * (1.0 - (1.0 + i).powf(-1.0 / m_f));
    let aax_m = aa_x(config, x, m)?; // äₓ⁽ᵐ⁾
    let ax = get_value(config, x, "Ax")?; // Aₓ
    let result = (aax_m + ax) / (d_m * d_m);
    Ok(result)
}

/// Due temporary annuity-due:
/// äₓ:ₙ̅⁽ᵐ⁾ = äₓ₊ₙ̅⁽ᵐ⁾ - äₓ⁽ᵐ⁾
///
/// Present value of 1/m paid m times per year for at most n years.
pub fn Iaa_x_n(config: &MortTableConfig, x: i32, n: i32, m: i32) -> PolarsResult<f64> {
    let result = aa_x(config, x, m)? - aa_x(config, x + n, m)?;
    Ok(result)
}

//------------------- Deferred Annuities -------------------

/// Immediate deferred life annuity-due: ₜäₓ⁽ᵐ⁾ = (Dₓ₊ₜ⁽ⁱ⁾/Dₓ⁽ⁱ⁾) × äₓ₊ₜ⁽ᵐ⁾
///
/// Annuity beginning after t-year deferral period.
pub fn taax(config: &MortTableConfig, x: i32, t: i32, m: i32) -> PolarsResult<f64> {
    let dx = get_value(config, x, "Dx")?;
    let dxh = get_value(config, x + t, "Dx")?;
    let ax_due_h = aax(config, x + t, m)?;
    Ok((dxh / dx) * ax_due_h)
}

/// Immediate deferred temporary annuity: ₜäₓ:ₙ⁽ᵐ⁾ = ₜäₓ⁽ᵐ⁾ - ₜ₊ₙäₓ⁽ᵐ⁾
///
/// Annuity with both deferral period t and payment period n.
pub fn taaxn(config: &MortTableConfig, x: i32, n: i32, t: i32, m: i32) -> PolarsResult<f64> {
    let tax_due_h = taax(config, x, t, m)?;
    let tax_due_hn = taax(config, x, t + n, m)?;
    Ok(tax_due_h - tax_due_hn)
}

//------------------- Increasing Annuities -------------------

/// Immediate increasing life annuity: (Iä)ₓ⁽ᵐ⁾ = (1/m) × [(3-m)(Sₓ⁽ⁱ⁾ + Nₓ⁽ⁱ⁾) - (m-1)Dₓ⁽ⁱ⁾] / (2Dₓ⁽ⁱ⁾)
///
/// Payments increase by 1/m each year: 1/m, 2/m, 3/m, ...
pub fn Iaax(config: &MortTableConfig, x: i32, _n: i32, m: i32) -> PolarsResult<f64> {
    let sx = get_value(config, x, "Sx")?;
    let nx = get_value(config, x, "Nx")?;
    let dx = get_value(config, x, "Dx")?;
    let numerator = (3.0 - m as f64) * (sx + nx) - (m as f64 - 1.0) * dx;
    let denominator = 2.0 * dx;
    Ok((1.0 / m as f64) * (numerator / denominator))
}

/// Immediate increasing temporary annuity: (Iä)ₓ:ₙ⁽ᵐ⁾ = (Iä)ₓ⁽ᵐ⁾ - (Dₓ₊ₙ⁽ⁱ⁾/Dₓ⁽ⁱ⁾) × (Iä)ₓ₊ₙ⁽ᵐ⁾
///
/// Increasing payments for at most n years.
pub fn Iaaxn(config: &MortTableConfig, x: i32, n: i32, m: i32) -> PolarsResult<f64> {
    let iax = Iaax(config, x, n, m)?;
    let dx = get_value(config, x, "Dx")?;
    let dxn = get_value(config, x + n, "Dx")?;
    let iax_n = Iaax(config, x + n, n, m)?;
    Ok(iax - (dxn / dx) * iax_n)
}

/// Immediate deferred increasing annuity: ₜ|(Iä)ₓ⁽ᵐ⁾ = (Dₓ₊ₜ⁽ⁱ⁾/Dₓ⁽ⁱ⁾) × (Iä)ₓ₊ₜ⁽ᵐ⁾
///
/// Increasing annuity beginning after t-year deferral period.
pub fn tIaax(config: &MortTableConfig, x: i32, _n: i32, t: i32, m: i32) -> PolarsResult<f64> {
    let dx = get_value(config, x, "Dx")?;
    let dx_t = get_value(config, x + t, "Dx")?;
    let iax_due_t = Iaax(config, x + t, _n, m)?;
    Ok((dx_t / dx) * iax_due_t)
}

/// Immediate deferred increasing temporary annuity: ₜ|(Iä)ₓ:ₙ⁽ᵐ⁾ = ₜ|(Iä)ₓ⁽ᵐ⁾ - ₜ₊ₙ|(Iä)ₓ⁽ᵐ⁾
///
/// Increasing annuity with both deferral period t and payment period n.
pub fn tIaaxn(config: &MortTableConfig, x: i32, n: i32, t: i32, m: i32) -> PolarsResult<f64> {
    let t_iax_due = tIaax(config, x, n, t, m)?;
    let t_iax_due_n = tIaax(config, x, n, t + n, m)?;
    Ok(t_iax_due - t_iax_due_n)
}

/// Calculates the present value of a geometrically increasing life annuity-due.
///
/// **Mathematical Formula**: Calculated using adjusted interest rate i′ = (1+i)/(1+g) - 1
///
/// # Parameters
/// - `config`: Mortality table configuration with interest rate
/// - `x`: Current age of the annuitant
/// - `n`: Parameter for compatibility
/// - `m`: Number of payments per year
/// - `g`: Annual growth rate of payments
///
/// # Returns
/// Geometric increasing annuity: ä̈ₓ⁽ᵐ⁾ calculated with adjusted interest rate i′ = (1+i)/(1+g) - 1
pub fn gIaax(config: &MortTableConfig, x: i32, n: i32, m: i32, g: f64) -> PolarsResult<f64> {
    let new_config = get_new_config(config, g);
    let result = Iaax(&new_config, x, n, m)?;
    Ok(result)
}

/// Immediate geometric increasing temporary annuity: (gIä)ₓ:ₙ⁽ᵐ⁾ with adjusted interest rate i′ = (1+i)/(1+g) - 1
///
/// Increasing temporary annuity with geometric growth rate g.
pub fn gIaaxn(config: &MortTableConfig, x: i32, n: i32, m: i32, g: f64) -> PolarsResult<f64> {
    let new_config = get_new_config(config, g);
    let result = Iaaxn(&new_config, x, n, m)?;
    Ok(result)
}

//---------------------------------------------------------
// PRIVATE FUNCTIONS
//---------------------------------------------------------
