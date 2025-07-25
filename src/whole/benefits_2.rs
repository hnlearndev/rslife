use super::*;

//-----------------Basic------------------

/// Immediate whole life insurance:
/// Aₓ = Mₓ/Dₓ
/// ₜ|Aₓ = Aₓ₊ₜ · ₜEₓ = Mₓ₊ₜ / Dₓ
/// Present value of $1 paid only if death occurs
pub fn Ax(config: &MortTableConfig, x: u32, t: u32, entry_age: Option<u32>) -> PolarsResult<f64> {
    // Decide if selected table is used
    let new_config = get_new_config_with_selected_table(config, entry_age)?;
    let mx = get_value(&new_config, x + t, "Mx")?;
    let dx = get_value(&new_config, x, "Dx")?;
    Ok(mx / dx)
}
