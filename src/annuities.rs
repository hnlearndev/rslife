use crate::benefits::nEx;
use crate::helpers::{
    get_new_config_geometric_functions, get_new_config_with_selected_table, get_value,
};
use crate::int_rate_convert::eff_i_to_nom_i;
use crate::mt_config::MortTableConfig;
use polars::prelude::*;

//-----------------Basic------------------

/// Life annuity-due payable m times per year:
/// äₓ⁽ᵐ⁾ = Nₓ/Dₓ . (i/i⁽ᵐ⁾)
/// ₜ|äₓ⁽ᵐ⁾ = ₜEₓ . (Nₓ₊ₜ/Dₓ) . (i/i⁽ᵐ⁾)
/// Present value of 1/m paid m times per year for life.
pub fn aax(
    config: &MortTableConfig,
    x: u32,
    m: u32,
    t: u32,
    entry_age: Option<u32>,
) -> PolarsResult<f64> {
    // Decide if selected table is used
    let new_config = get_new_config_with_selected_table(config, entry_age)?;

    let nEx = nEx(&new_config, x, t, 0, None)?;

    let nx_t = get_value(&new_config, x + t, "Nx")?;
    let dx = get_value(&new_config, x, "Dx")?;

    let i = new_config.int_rate.unwrap_or(0.0);
    let d_m = eff_i_to_nom_i(i, m);

    let result = nEx * (nx_t / dx) * (i / d_m);
    Ok(result)
}

/// Due temporary annuity-due payable m times per year:
/// äₓ:ₙ̅⁽ᵐ⁾ = äₓ⁽ᵐ⁾ - ₙEₓ · äₓ₊ₙ⁽ᵐ⁾
/// ₜ|äₓ:ₙ̅⁽ᵐ⁾ = ₜ|äₓ⁽ᵐ⁾ - ₜ₊ₙEₓ · äₓ₊ₜ₊ₙ⁽ᵐ⁾
/// Present value of 1/m paid m times per year for up to n years.
pub fn aaxn(
    config: &MortTableConfig,
    x: u32,
    n: u32,
    m: u32,
    t: u32,
    entry_age: Option<u32>,
) -> PolarsResult<f64> {
    // Decide if selected table is used
    let new_config = get_new_config_with_selected_table(config, entry_age)?;

    let aax_t = aax(&new_config, x, m, t, None)?;
    let nEx_tn = nEx(&new_config, x, n + t, 0, None)?;
    let aax_xtn = aax(&new_config, x + t + n, m, 0, None)?;
    let result = aax_t - nEx_tn * aax_xtn;
    Ok(result)
}

//-----------------Increasing------------------
/// Increasing life annuity-due payable m times per year:
/// There are no absolute formula but I propose decomposition like this:
/// 1/m, 2/m, 3/m, ..., n/m are each 1 year apart as (1/m) * Iaax. Each component then form the same present value at the start of its 1/m payment.
/// This is then annuity of m components: PV of component * aaxn
/// (Iä)ₓ = Sₓ / Dₓ
///
/// Present value of an increasing life annuity-due: payments of 1/m made m times per year for life, with each annual payment increasing by 1.
pub fn Iaax(
    config: &MortTableConfig,
    x: u32,
    m: u32,
    t: u32,
    entry_age: Option<u32>,
) -> PolarsResult<f64> {
    // Decide if selected table is used
    let new_config = get_new_config_with_selected_table(config, entry_age)?;

    let m_f64 = m as f64;
    let sx = get_value(&new_config, x, "Sx")?;
    let dx = get_value(&new_config, x, "Dx")?;
    let iaax = sx / dx;
    let component_iaa_x = (1.0 / m_f64) * iaax;

    // Adjust to get effective interest rate period m
    let i = new_config.int_rate.unwrap_or(0.0);
    let eff_i = (1.0 + i).powf(1.0 / m_f64) - 1.0;
    // Create a new config with the adjusted interest rate
    let mut eff_config = new_config.clone();
    eff_config.int_rate = Some(eff_i);

    // Calculate the annuity of components
    let result = component_iaa_x * aaxn(config, x, m, 1, t, None)?;

    Ok(result)
}

/// Due increasing temporary life annuity-due payable m times per year:
/// Iäₓ:ₙ̅⁽ᵐ⁾ = Iäₓ⁽ᵐ⁾ - ₙEₓ · (Iäₓ₊ₙ⁽ᵐ⁾ + n · äₓ₊ₙ⁽ᵐ⁾)
/// ₜ|(Iä)ₓ:ₙ̅⁽ᵐ⁾ = ₜ|(Iä)ₓ⁽ᵐ⁾ - ₜ₊ₙEₓ · ((Iä)ₓ₊ₜ₊ₙ⁽ᵐ⁾ + n · äₓ₊ₜ₊ₙ⁽ᵐ⁾)
///
/// Present value of an increasing life annuity-due: payments of 1/m made m times per year for n years, with each annual payment increasing by 1.
pub fn Iaaxn(
    config: &MortTableConfig,
    x: u32,
    n: u32,
    m: u32,
    t: u32,
    entry_age: Option<u32>,
) -> PolarsResult<f64> {
    // Decide if selected table is used
    let new_config = get_new_config_with_selected_table(config, entry_age)?;

    // ₜ|(Iä)ₓ:ₙ̅⁽ᵐ⁾ = ₜ|(Iä)ₓ⁽ᵐ⁾ - ₜ₊ₙEₓ · ((Iä)ₓ₊ₜ₊ₙ⁽ᵐ⁾ + n · äₓ₊ₜ₊ₙ⁽ᵐ⁾)
    let iaax_t = Iaax(&new_config, x, m, t, None)?; // ₜ|(Iä)ₓ⁽ᵐ⁾
    let nEx_tn = nEx(&new_config, x, n + t, 0, None)?; // ₜ₊ₙEₓ
    let iaax_xtn = Iaax(&new_config, x + t + n, m, 0, None)?; // (Iä)ₓ₊ₜ₊ₙ⁽ᵐ⁾
    let aax_xtn = aax(&new_config, x + t + n, m, 0, None)?; // äₓ₊ₜ₊ₙ⁽ᵐ⁾
    let result = iaax_t - nEx_tn * (iaax_xtn + (n as f64) * aax_xtn);
    Ok(result)
}

//-----------------Decreasing------------------
/// Decreasing temporary life annuity-due:
/// Däₓ:ₙ̅⁽ᵐ⁾ = (n+1) · äₓ:ₙ̅⁽ᵐ⁾ - Iäₓ:ₙ̅⁽ᵐ⁾
/// ₜ|(Dä)ₓ:ₙ̅⁽ᵐ⁾ = (n+1) · ₜ|äₓ:ₙ̅⁽ᵐ⁾ - ₜ|(Iä)ₓ:ₙ̅⁽ᵐ⁾
///
/// Present value of an decreasing life annuity-due: payments of n/m made m times per year for n years, with each annual payment decreasing by 1.
pub fn Daaxn(
    config: &MortTableConfig,
    x: u32,
    n: u32,
    m: u32,
    t: u32,
    entry_age: Option<u32>,
) -> PolarsResult<f64> {
    // Decide if selected table is used
    let new_config = get_new_config_with_selected_table(config, entry_age)?;

    // ₜ|(Dä)ₓ:ₙ̅⁽ᵐ⁾ = (n+1) · ₜ|äₓ:ₙ̅⁽ᵐ⁾ - ₜ|(Iä)ₓ:ₙ̅⁽ᵐ⁾
    let aaxn_val = aaxn(&new_config, x, n, m, t, None)?; // ₜ|äₓ:ₙ̅⁽ᵐ⁾
    let iaaxn_val = Iaaxn(&new_config, x, n, m, t, None)?; // ₜ|(Iä)ₓ:ₙ̅⁽ᵐ⁾
    let result = (n as f64 + 1.0) * aaxn_val - iaaxn_val;
    Ok(result)
}

//-----------------Geometric increasing------------------
/// Geometric  increasing life annuity-due:
/// äₓ⁽ᵍ⁾ with adjusted interest rate i′ = (1+i)/(1+g) - 1
///
/// Payments of 1/m are made m times per year for life, starting immediately,
/// with each annual payment increasing by a factor of (1+g) each year (i.e., geometric progression).
pub fn gaax(
    config: &MortTableConfig,
    x: u32,
    m: u32,
    g: f64,
    t: u32,
    entry_age: Option<u32>,
) -> PolarsResult<f64> {
    // Decide if selected table is used
    let new_config = get_new_config_with_selected_table(config, entry_age)?;
    let adjusted_config = get_new_config_geometric_functions(&new_config, g)?;
    let result = aax(&adjusted_config, x, m, t, None)?;
    Ok(result)
}

/// Geometric increasing temporary annuity-due:
/// äₓ:ₙ̅⁽ᵍ⁾ with adjusted interest rate i′ = (1+i)/(1+g) - 1
///
/// Payments of 1/m are made m times per year for n years, starting immediately,
/// with each annual payment increasing by a factor of (1+g) each year (i.e., geometric progression).
pub fn gaaxn(
    config: &MortTableConfig,
    x: u32,
    n: u32,
    m: u32,
    g: f64,
    t: u32,
    entry_age: Option<u32>,
) -> PolarsResult<f64> {
    // Decide if selected table is used
    let new_config = get_new_config_with_selected_table(config, entry_age)?;
    let adjusted_config = get_new_config_geometric_functions(&new_config, g)?;
    let result = aaxn(&adjusted_config, x, n, m, t, None)?;
    Ok(result)
}
