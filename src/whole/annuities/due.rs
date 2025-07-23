use super::*;

// Note: due.rs and immediate.rs are kept independent.
// Their relationship will be used for testing purposes.

//-----------------Basic------------------

/// Due life annuity-due:
/// äₓ⁽ᵐ⁾ = (1/m) × [(1 - vˣ)/(1 - v¹/ᵐ)]
///
/// Present value of 1/m paid m times per year for life.
pub fn aa_x(config: &MortTableConfig, x: i32, m: i32) -> PolarsResult<f64> {
    let i = config.int_rate.unwrap_or(0.0);
    let v = 1.0 / (1.0 + i);
    let numerator = 1.0 - v.powi(x);
    let denominator = 1.0 - v.powf(1.0 / m as f64);
    let result = (1.0 / m as f64) * (numerator / denominator);
    Ok(result)
}

/// Due temporary annuity-due:
/// äₓ:ₙ̅⁽ᵐ⁾ = äₓ⁽ᵐ⁾ - (Dₓ₊ₙ/Dₓ) · äₓ₊ₙ⁽ᵐ⁾
///
/// Present value of 1/m paid m times per year for up to n years.
pub fn aa_x_n(config: &MortTableConfig, x: i32, n: i32, m: i32) -> PolarsResult<f64> {
    let aax_m = aa_x(config, x, m)?; // äₓ⁽ᵐ⁾
    let dx = get_value(config, x, "Dx")?;
    let dxn = get_value(config, x + n, "Dx")?;
    let aaxn_m = aa_x(config, x + n, m)?; // äₓ₊ₙ⁽ᵐ⁾
    let result = aax_m - (dxn / dx) * aaxn_m;
    Ok(result)
}

//-----------------Increasing------------------
/// Increasing life annuity-due:
/// Iäₓ⁽ᵐ⁾ = (Rₓ - Nₓ)/(Dₓ · d⁽ᵐ⁾)
/// where d⁽ᵐ⁾ = m[1 - (1+i)⁻¹/ᵐ]
///
/// Present value of an increasing life annuity-due: payments of 1/m made m times per year for life, with each annual payment increasing by 1.
pub fn Iaa_x(config: &MortTableConfig, x: i32, m: i32) -> PolarsResult<f64> {
    let i = config.int_rate.unwrap_or(0.0);
    let m_f = m as f64;
    let d_m = m_f * (1.0 - (1.0 + i).powf(-1.0 / m_f));
    let rx = get_value(config, x, "Rx")?;
    let nx = get_value(config, x, "Nx")?;
    let dx = get_value(config, x, "Dx")?;
    let result = (rx - nx) / (dx * d_m);
    Ok(result)
}

/// Due increasing temporary life annuity-due:
/// Iäₓ:ₙ̅⁽ᵐ⁾ = Iäₓ⁽ᵐ⁾ - (Dₓ₊ₙ/Dₓ) · (Iäₓ₊ₙ⁽ᵐ⁾ + n · äₓ₊ₙ⁽ᵐ⁾)
///
/// Present value of an increasing life annuity-due: payments of 1/m made m times per year for n years, with each annual payment increasing by 1.
pub fn Iaa_x_n(config: &MortTableConfig, x: i32, m: i32, n: i32) -> PolarsResult<f64> {
    let iaa_x = Iaa_x(config, x, m)?;
    let dx = get_value(config, x, "Dx")?;
    let dxn = get_value(config, x + n, "Dx")?;
    let iaa_xn = Iaa_x(config, x + n, m)?;
    let aa_xn = aa_x(config, x + n, m)?;
    let result = iaa_x - (dxn / dx) * (iaa_xn + n as f64 * aa_xn);
    Ok(result)
}

//-----------------Decreasing------------------
/// Decreasing temporary life annuity-due:
/// Däₓ:ₙ̅⁽ᵐ⁾ = (n+1) · äₓ:ₙ̅⁽ᵐ⁾ - Iäₓ:ₙ̅⁽ᵐ⁾
///
/// Present value of an decreasing life annuity-due: payments of n/m made m times per year for n years, with each annual payment decreasing by 1.
pub fn Daa_x_n(config: &MortTableConfig, x: i32, m: i32, n: i32) -> PolarsResult<f64> {
    let aa_xn = aa_x_n(config, x, n, m)?;
    let iaa_xn = Iaa_x_n(config, x, m, n)?;
    let result = (n as f64 + 1.0) * aa_xn - iaa_xn;
    Ok(result)
}

//-----------------Geometric increasing------------------
/// Geometric  increasing life annuity-due:
/// äₓ⁽ᵍ⁾ with adjusted interest rate i′ = (1+i)/(1+g) - 1
///
/// Payments of 1/m are made m times per year for life, starting immediately,
/// with each annual payment increasing by a factor of (1+g) each year (i.e., geometric progression).
pub fn gaa_x(config: &MortTableConfig, x: i32, m: i32, g: f64) -> PolarsResult<f64> {
    let new_config = get_new_config(config, g);
    let result = aa_x(&new_config, x, m)?;
    Ok(result)
}

/// Geometric increasing temporary annuity-due:
/// äₓ:ₙ̅⁽ᵍ⁾ with adjusted interest rate i′ = (1+i)/(1+g) - 1
///
/// Payments of 1/m are made m times per year for n years, starting immediately,
/// with each annual payment increasing by a factor of (1+g) each year (i.e., geometric progression).
pub fn gaa_x_n(config: &MortTableConfig, x: i32, n: i32, m: i32, g: f64) -> PolarsResult<f64> {
    let new_config = get_new_config(config, g);
    let result = aa_x_n(&new_config, x, n, m)?;
    Ok(result)
}
