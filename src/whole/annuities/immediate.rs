use super::*;

// Note: due.rs and immediate.rs are kept independent.
// Their relationship will be used for testing purposes.

//-----------------Basic------------------

/// Life annuity-immediate:
/// aₓ⁽ᵐ⁾ = (1/m) × (v¹ᐟᵐ - vˣ)/(1 - v¹ᐟᵐ)
///
/// Present value of 1/m paid m times per year for life, with payments at the end of each period.
pub fn a_x(config: &MortTableConfig, x: i32, m: i32) -> PolarsResult<f64> {
    let v = config.int_rate.unwrap_or(0.0);
    let v_1_m = v.powf(1.0 / m as f64);
    let v_x = v.powf(x as f64);
    let numerator = v_1_m - v_x;
    let denominator = 1.0 - v_1_m;
    let result = (1.0 / m as f64) * (numerator / denominator);
    Ok(result)
}

/// Temporary annuity-immediate:
/// aₓ:ₙ̅⁽ᵐ⁾ = aₓ⁽ᵐ⁾ - (Dₓ₊ₜ/Dₓ) · aₓ₊ₙ⁽ᵐ⁾
///
/// Present value of 1/m paid m times per year for n years, with payments at the end of each period.
pub fn a_x_n(config: &MortTableConfig, x: i32, n: i32, m: i32) -> PolarsResult<f64> {
    let a_x_val = a_x(config, x, m)?;
    let D_x_n = get_value(config, x + n, "Dx")?;
    let D_x_val = get_value(config, x, "Dx")?;
    let a_x_n_val = a_x(config, x + n, m)?;
    let result = a_x_val - (D_x_n / D_x_val) * a_x_n_val;
    Ok(result)
}

//-----------------Increasing------------------

/// Increasing life annuity-immediate:
/// Iaₓ⁽ᵐ⁾ = (Rₓ - Nₓ · i⁽ᵐ⁾/(m·i))/(Dₓ · d⁽ᵐ⁾)
/// where d⁽ᵐ⁾ = m[1 - (1+i)⁻¹/ᵐ]
/// and i⁽ᵐ⁾ = m[(1+i)¹/ᵐ - 1]
///
/// Present value of an increasing life annuity-immediate: payments of 1/m made m times per year for life, with each annual payment increasing by 1.
pub fn Ia_x(config: &MortTableConfig, x: i32, m: i32) -> PolarsResult<f64> {
    let i = config.int_rate.unwrap_or(0.0);
    let m_f64 = m as f64;
    let d_m = m_f64 * (1.0 - (1.0 + i).powf(-1.0 / m_f64));
    let i_m = m_f64 * ((1.0 + i).powf(1.0 / m_f64) - 1.0);

    let R_x = get_value(config, x, "Rx")?;
    let N_x = get_value(config, x, "Nx")?;
    let D_x = get_value(config, x, "Dx")?;

    let numerator = R_x - N_x * (i_m / (m_f64 * i));
    let denominator = D_x * d_m;

    let result = numerator / denominator;
    Ok(result)
}

///  Increasing temporary annuity-immediate:
/// (Ia)ₓ:ₙ̅⁽ᵐ⁾ = (Ia)ₓ⁽ᵐ⁾ - (Dₓ₊ₙ/Dₓ) · [(Ia)ₓ₊ₙ⁽ᵐ⁾ + (n/m) · aₓ₊ₙ⁽ᵐ⁾]
///
/// Present value of an increasing life annuity-immediate: payments of 1/m made m times per year for n years, with each annual payment increasing by 1.
pub fn Ia_x_n(config: &MortTableConfig, x: i32, n: i32, m: i32) -> PolarsResult<f64> {
    let Ia_x_val = Ia_x(config, x, m)?;
    let D_x_n = get_value(config, x + n, "Dx")?;
    let D_x_val = get_value(config, x, "Dx")?;
    let Ia_x_n_val = Ia_x(config, x + n, m)?;
    let a_x_n_val = a_x(config, x + n, m)?;
    let n_m = n as f64 / m as f64;
    let bracket = Ia_x_n_val + n_m * a_x_n_val;
    let result = Ia_x_val - (D_x_n / D_x_val) * bracket;
    Ok(result)
}

//-----------------Decreasing------------------

///  Decreasing temporary annuity-immediate:
/// Daₓ:ₙ̅⁽ᵐ⁾  = (n+1) · aₓ:ₙ̅⁽ᵐ⁾ -  Iaₓ:ₙ̅⁽ᵐ⁾
///
/// Present value of an decreasing life annuity-immediate: payments of n/m made m times per year for n years, with each annual payment decreasing by 1.
pub fn Da_x_n(config: &MortTableConfig, x: i32, n: i32, m: i32) -> PolarsResult<f64> {
    let a_x_n_val = a_x_n(config, x, n, m)?;
    let Ia_x_n_val = Ia_x_n(config, x, n, m)?;
    let result = (n as f64 + 1.0) * a_x_n_val - Ia_x_n_val;
    Ok(result)
}

//-----------------Geometric increasing------------------

/// Geometric life annuity-immediate:
/// aₓ⁽ᵍ⁾ with adjusted interest rate i′ = (1+i)/(1+g) - 1
///
/// Present value of geometrically increasing payments paid m times per year for life, with payments at the end of each period.
/// The payment grows geometrically at rate g each year.
pub fn ga_x(config: &MortTableConfig, x: i32, m: i32, g: f64) -> PolarsResult<f64> {
    let new_config = get_new_config(config, g);
    let result = a_x(&new_config, x, m)?;
    Ok(result)
}

/// Geometric temporary annuity-immediate:
/// aₓ:ₙ̅⁽ᵍ⁾ with adjusted interest rate i′ = (1+i)/(1+g) - 1
///
/// Present value of geometrically increasing payments paid m times per year for n years, with payments at the end of each period.
/// The payment grows geometrically at rate g each year.
pub fn ga_x_n(config: &MortTableConfig, x: i32, n: i32, m: i32, g: f64) -> PolarsResult<f64> {
    let new_config = get_new_config(config, g);
    let result = a_x_n(&new_config, x, n, m)?;
    Ok(result)
}
