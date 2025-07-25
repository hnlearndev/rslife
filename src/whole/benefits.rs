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

/// Present value of $1 paid only if death occurs within n years.
pub fn Ax1n(
    config: &MortTableConfig,
    x: u32,
    n: u32,
    t: u32,
    entry_age: Option<u32>,
) -> PolarsResult<f64> {
    // Decide if selected table is used
    let new_config = get_new_config_with_selected_table(config, entry_age)?;
    let mxt = get_value(&new_config, x + t, "Mx")?;
    let mxtn = get_value(&new_config, x + t + n, "Mx")?;
    let dx = get_value(&new_config, x, "Dx")?;
    let result = (mxt - mxtn) / dx;
    Ok(result)
}

/// Immediate pure endowment:
/// ₙEₓ = Dₓ₊ₙ/Dₓ
/// ₜ|ₙEₓ = Dₓ₊ₜ₊ₙ / Dₓ
///
/// Present value of $1 paid if and only if the insured survives n years.
pub fn nEx(
    config: &MortTableConfig,
    x: u32,
    n: u32,
    t: u32,
    entry_age: Option<u32>,
) -> PolarsResult<f64> {
    // Decide if selected table is used
    let new_config = get_new_config_with_selected_table(config, entry_age)?;
    let result = get_value(&new_config, x + t + n, "Dx")? / get_value(&new_config, x, "Dx")?;
    Ok(result)
}

/// Immediate Endowment insurance:
/// Aₓ:ₙ̅ = A¹ₓ:ₙ̅ + Aₓ:ₙ̅¹
///
/// $1 paid at death (if within n years) OR at survival to n years.
pub fn Axn(
    config: &MortTableConfig,
    x: u32,
    n: u32,
    t: u32,
    entry_age: Option<u32>,
) -> PolarsResult<f64> {
    // Decide if selected table is used
    let new_config = get_new_config_with_selected_table(config, entry_age)?;
    let result = Ax1n(&new_config, x, n, t, None)? + nEx(&new_config, x, n, t, None)?;
    Ok(result)
}

//-----------------Increasing------------------

/// Immediate increasing whole life:
///  (IA)ₓ = Rₓ / Dₓ
/// ₜ|(IA)ₓ = Rₓ₊ₜ / Dₓ
///
/// Death benefit increases by 1 each year: k paid if death in k-th year.
pub fn IAx(config: &MortTableConfig, x: u32, t: u32, entry_age: Option<u32>) -> PolarsResult<f64> {
    // Decide if selected table is used
    let new_config = get_new_config_with_selected_table(config, entry_age)?;
    let rxt = get_value(&new_config, x + t, "Rx")?;
    let dx = get_value(&new_config, x, "Dx")?;
    Ok(rxt / dx)
}

/// Immediate increasing term:
/// (IA)¹ₓ:ₙ̅ = = (Rₓ - Rₓ₊ₙ) / Dₓ
/// ₜ|(IA)¹ₓ:ₙ̅ = (Rₓ₊ₜ - Rₓ₊ₜ₊ₙ) / Dₓ
/// Death benefit increases by 1 each year, pays only if death within n years.
pub fn IAx1n(
    config: &MortTableConfig,
    x: u32,
    n: u32,
    t: u32,
    entry_age: Option<u32>,
) -> PolarsResult<f64> {
    // Decide if selected table is used
    let new_config = get_new_config_with_selected_table(config, entry_age)?;
    let rxt = get_value(&new_config, x + t, "Rx")?;
    let rxtn = get_value(&new_config, x + t + n, "Rx")?;
    let dx = get_value(&new_config, x, "Dx")?;
    let result = (rxt - rxtn) / dx;
    Ok(result)
}

/// Immediate endowment insurance:
/// IAₓ:ₙ̅ = IA¹ₓ:ₙ̅ + IA¹ₓ:ₙ̅
/// ₜ|IAₓ:ₙ̅ = ₜ|IA¹ₓ:ₙ̅ + ₜ|IA¹ₓ:ₙ̅
/// $1 paid at death (if within n years) OR at survival to n years.
pub fn IAxn(
    config: &MortTableConfig,
    x: u32,
    n: u32,
    t: u32,
    entry_age: Option<u32>,
) -> PolarsResult<f64> {
    // Decide if selected table is used
    let new_config = get_new_config_with_selected_table(config, entry_age)?;
    let term = IAx1n(&new_config, x, n, t, None)?;
    let pure_endowment = (n as f64) * nEx(&new_config, x, n, t, None)?;
    let result = term + pure_endowment;
    Ok(result)
}

//-----------------Decreasing------------------
// Note: There should starting amount hence DAₓ is not applicable

/// Immediate decreasing term:
/// (DA)¹ₓ:ₙ̅ = (n+1) · A¹ₓ:ₙ̅ - (IA)¹ₓ:ₙ̅
/// ₜ|(DA¹)ₓ:ₙ̅ = n · ₜ|A¹ₓ:ₙ̅ - ₜ|(IA¹)ₓ:ₙ̅
/// Death benefit decreases by 1 each policy year (n in year 1, n-1 in year 2, ..., 1 in year n),
/// pays only if death occurs within n years.
pub fn DAx1n(
    config: &MortTableConfig,
    x: u32,
    n: u32,
    t: u32,
    entry_age: Option<u32>,
) -> PolarsResult<f64> {
    // Decide if selected table is used
    let new_config = get_new_config_with_selected_table(config, entry_age)?;
    let n_a_x1_n = (n + 1) as f64 * Ax1n(&new_config, x, n, t, None)?;
    let ia_x1_n = IAx1n(&new_config, x, n, t, None)?;
    let result = n_a_x1_n - ia_x1_n;
    Ok(result)
}

/// Immediate decreasing endowment insurance:
/// (DA)ₓ:ₙ̅ = (DA)¹ₓ:ₙ̅ + ₜ|ₙEₓ
/// ₜ|(DA)ₓ:ₙ̅ = ₜ|(DA)¹ₓ:ₙ̅ + ₜ|ₙEₓ
/// $1 paid at death (if within n years) OR at survival to n years.
pub fn DAxn(
    config: &MortTableConfig,
    x: u32,
    n: u32,
    t: u32,
    entry_age: Option<u32>,
) -> PolarsResult<f64> {
    // Decide if selected table is used
    let new_config = get_new_config_with_selected_table(config, entry_age)?;
    let result = DAx1n(&new_config, x, n, t, None)? + nEx(&new_config, x, n, t, None)?;
    Ok(result)
}

//-----------------Geometric increasing------------------

/// Immediate geometric whole life:
/// Aₓ⁽ᵍ⁾ with adjusted interest rate i′ = (1+i)/(1+g) - 1
///
/// Death benefit grows geometrically at rate g each year.
pub fn gAx(
    config: &MortTableConfig,
    x: u32,
    g: f64,
    t: u32,
    entry_age: Option<u32>,
) -> PolarsResult<f64> {
    // Decide if selected table is used
    let new_config = get_new_config_with_selected_table(config, entry_age)?;
    let adjusted_config = get_new_config_geometric_functions(&new_config, g)?;
    let result = Ax(&adjusted_config, x, t, None)?;
    Ok(result)
}

/// Immediate geometric n-year term:
/// A¹ₓ:ₙ̅⁽ᵍ⁾ with adjusted interest rate i′ = (1+i)/(1+g) - 1
///
/// Death benefit grows geometrically at rate g for n years.
pub fn gAx1n(
    config: &MortTableConfig,
    x: u32,
    n: u32,
    g: f64,
    t: u32,
    entry_age: Option<u32>,
) -> PolarsResult<f64> {
    // Decide if selected table is used
    let new_config = get_new_config_with_selected_table(config, entry_age)?;
    let adjusted_config = get_new_config_geometric_functions(&new_config, g)?;
    let result = Ax1n(&adjusted_config, x, n, t, None)?;
    Ok(result)
}

/// Immediate geometric n-year pure endowment:
/// Aₓ:ₙ̅¹⁽ᵍ⁾ with adjusted interest rate i′ = (1+i)/(1+g) - 1
///
/// Death benefit grows geometrically at rate g for n years.
pub fn gnEx(
    config: &MortTableConfig,
    x: u32,
    n: u32,
    g: f64,
    t: u32,
    entry_age: Option<u32>,
) -> PolarsResult<f64> {
    // Decide if selected table is used
    let new_config = get_new_config_with_selected_table(config, entry_age)?;
    let adjusted_config = get_new_config_geometric_functions(&new_config, g)?;
    let result = nEx(&adjusted_config, x, n, t, None)?;
    Ok(result)
}

/// Immediate geometric n-year endowment:
/// Aₓ:ₙ̅⁽ᵍ⁾ = A¹ₓ:ₙ̅⁽ᵍ⁾ + Aₓ:ₙ̅¹⁽ᵍ⁾
///
/// Death benefit grows geometrically at rate g for n years.
pub fn gAxn(
    config: &MortTableConfig,
    x: u32,
    n: u32,
    g: f64,
    t: u32,
    entry_age: Option<u32>,
) -> PolarsResult<f64> {
    // Decide if selected table is used
    let new_config = get_new_config_with_selected_table(config, entry_age)?;
    let term = gAx1n(&new_config, x, n, g, t, None)?;
    let pure_endowment = gnEx(&new_config, x, n, g, t, None)?;
    let result = term + pure_endowment;
    Ok(result)
}

// ================================================
// UNIT TESTS
// ================================================
#[cfg(test)]
mod tests {
    use super::*;
    use crate::prelude::*;

    #[test]
    fn test_axn_am92_selected() {
        // ======Question 1======:
        // Load AM92 selected table
        let am92_xml = MortXML::from_xlsx("data/am92.xlsx", "am92")
            .expect("Failed to load AM92 selected table");

        // Create MortTableConfig
        let config = MortTableConfig {
            xml: am92_xml.clone(),
            radix: Some(100_000),
            int_rate: Some(0.05),
            pct: Some(1.0),
            assumption: Some(AssumptionEnum::UDD),
        };

        // Print out the content type for verification
        println!("Content Type: {}", config.xml.content_classification.content_type);

        // Calculate  A₍₇₀₎:₃
        let ans = Axn(&config, 70, 3, 0, Some(70)).expect("Axn calculation failed");

        // Expected value: 0.8663440 (as per notebook comment)
        let expected = 0.8663440;
        let tol = 1e-6;
        assert!(
            (ans - expected).abs() < tol,
            "Axn(70,3,0,Some(70)) = {ans}, expected {expected}"
        );
    }
}
