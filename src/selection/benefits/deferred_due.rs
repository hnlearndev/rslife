use super::*;

// Note:
// Deferred due benefits combine deferment period with due (beginning of year) payment.
// Due benefits are paid at the beginning of the year of death, not at the end.
// This is counterintuitive but mathematical convention in actuarial science.

//-----------------Basic------------------

/// Deferred due whole life insurance:
/// ₜ|Äₓ = Äₓ - Ä¹ₓ:ₜ̅
///
/// Present value of $1 payable at the beginning of the year of death, provided death occurs after a deferment period of t years.
/// This represents a deferred due whole life insurance benefit, where payment is made only if the insured survives the deferment period.
pub fn t_AA_x_(config: &MortTableConfig, entry_age: i32, t: i32, x: i32) -> PolarsResult<f64> {
    let new_config = get_new_config_with_selected_table(config, entry_age)?;
    whole::t_AA_x(&new_config, t, x)
}

/// Deferred due term life insurance:
/// ₜ|Ä¹ₓ:ₙ̅ = (Dₓ₊ₜ/Dₓ) · Ä¹ₓ₊ₜ:ₙ̅
///
/// Present value of $1 payable at the beginning of the year of death, provided death occurs within n years after a deferment period of t years.
/// This represents a deferred due term life insurance benefit, where payment is made only if the insured survives the deferment period and then dies within the following n years.
pub fn t_AA_x1_n_(
    config: &MortTableConfig,
    entry_age: i32,
    t: i32,
    x: i32,
    n: i32,
) -> PolarsResult<f64> {
    let new_config = get_new_config_with_selected_table(config, entry_age)?;
    whole::t_AA_x1_n(&new_config, t, x, n)
}

/// Deferred due pure endowment:
/// ₜ|Äₓ:ₙ̅¹ = Äₓ:ₜ₊ₙ̅¹ = (1+i) · Dₓ₊ₜ₊ₙ / Dₓ
///
/// Present value of $1 payable only if the insured survives the entire deferment period of t+n years.
/// This represents a deferred due pure endowment, where payment is made at the beginning of year t+n+1 provided the insured is still alive.
pub fn t_AA_x_n1_(
    config: &MortTableConfig,
    entry_age: i32,
    t: i32,
    x: i32,
    n: i32,
) -> PolarsResult<f64> {
    let new_config = get_new_config_with_selected_table(config, entry_age)?;
    whole::t_AA_x_n1(&new_config, t, x, n)
}

/// Deferred due endowment insurance:
/// ₜ|Äₓ:ₙ̅ = ₜ|Ä¹ₓ:ₙ̅ + ₜ|Äₓ:ₙ̅¹
///
/// Present value of $1 payable either at the beginning of the year of death (if it occurs within n years after a deferment period of t years), or at the beginning of year t+n+1 if the insured survives the entire period.
/// This represents a deferred due endowment insurance benefit, combining deferred due term insurance and deferred due pure endowment.
pub fn t_AA_x_n_(
    config: &MortTableConfig,
    entry_age: i32,
    t: i32,
    x: i32,
    n: i32,
) -> PolarsResult<f64> {
    let new_config = get_new_config_with_selected_table(config, entry_age)?;
    whole::t_AA_x_n(&new_config, t, x, n)
}

//-----------------Increasing------------------

/// Deferred due increasing whole life:
/// ₜ|IÄₓ = IÄₓ - IÄ¹ₓ:ₜ̅
///
/// Death benefit increases by 1 each year after deferment, with payment at the beginning of the year of death.
/// Represents a deferred due increasing whole life insurance where the benefit starts at 1 after the deferment period.
pub fn t_IAA_x_(config: &MortTableConfig, entry_age: i32, t: i32, x: i32) -> PolarsResult<f64> {
    let new_config = get_new_config_with_selected_table(config, entry_age)?;
    whole::t_IAA_x(&new_config, t, x)
}

/// Deferred due increasing term:
/// ₜ|IÄ¹ₓ:ₙ̅ = (Dₓ₊ₜ/Dₓ) · IÄ¹ₓ₊ₜ:ₙ̅
///
/// Death benefit increases by 1 each year after deferment, pays only if death occurs within n years after the deferment period.
/// This represents a deferred due increasing term life insurance with payment at the beginning of the year of death.
pub fn t_IAA_x1_n_(
    config: &MortTableConfig,
    entry_age: i32,
    t: i32,
    x: i32,
    n: i32,
) -> PolarsResult<f64> {
    let new_config = get_new_config_with_selected_table(config, entry_age)?;
    whole::t_IAA_x1_n(&new_config, t, x, n)
}

/// Deferred due increasing pure endowment:
/// ₜ|IÄₓ:ₙ̅¹ = IÄₓ:ₜ₊ₙ̅¹
///
/// Present value of an increasing pure endowment deferred for t years with due payment.
/// The benefit amount is based on the total period t+n.
pub fn t_IAA_x_n1_(
    config: &MortTableConfig,
    entry_age: i32,
    t: i32,
    x: i32,
    n: i32,
) -> PolarsResult<f64> {
    let new_config = get_new_config_with_selected_table(config, entry_age)?;
    whole::t_IAA_x_n1(&new_config, t, x, n)
}

/// Deferred due increasing endowment insurance:
/// ₜ|IÄₓ:ₙ̅ = ₜ|IÄ¹ₓ:ₙ̅ + ₜ|IÄₓ:ₙ̅¹
///
/// Present value of an increasing endowment insurance deferred for t years with due payment.
/// Combines deferred due increasing term insurance and deferred due increasing pure endowment.
pub fn t_IAA_x_n_(
    config: &MortTableConfig,
    entry_age: i32,
    t: i32,
    x: i32,
    n: i32,
) -> PolarsResult<f64> {
    let new_config = get_new_config_with_selected_table(config, entry_age)?;
    whole::t_IAA_x_n(&new_config, t, x, n)
}

//-----------------Decreasing------------------
// Note: There should be a starting amount hence t_DAA_x is not applicable

/// Deferred due decreasing term:
/// ₜ|DÄ¹ₓ:ₙ̅ = (Dₓ₊ₜ/Dₓ) · DÄ¹ₓ₊ₜ:ₙ̅
///
/// Death benefit decreases by 1 each year after deferment, pays only if death occurs within n years after the deferment period.
/// This represents a deferred due decreasing term life insurance with payment at the beginning of the year of death.
pub fn t_DAA_x1_n_(
    config: &MortTableConfig,
    entry_age: i32,
    t: i32,
    x: i32,
    n: i32,
) -> PolarsResult<f64> {
    let new_config = get_new_config_with_selected_table(config, entry_age)?;
    whole::t_DAA_x1_n(&new_config, t, x, n)
}

/// Deferred due decreasing pure endowment:
/// ₜ|DÄₓ:ₙ̅¹ = DÄₓ:ₜ₊ₙ̅¹ = Äₓ:ₜ₊ₙ̅¹
///
/// Present value of a decreasing pure endowment deferred for t years with due payment.
/// This is equivalent to the deferred due pure endowment, as the decreasing structure does not affect the benefit.
/// Rarely used in practice.
pub fn t_DAA_x_n1_(
    config: &MortTableConfig,
    entry_age: i32,
    t: i32,
    x: i32,
    n: i32,
) -> PolarsResult<f64> {
    let new_config = get_new_config_with_selected_table(config, entry_age)?;
    whole::t_DAA_x_n1(&new_config, t, x, n)
}

/// Deferred due decreasing endowment insurance:
/// ₜ|DÄₓ:ₙ̅ = ₜ|DÄ¹ₓ:ₙ̅ + ₜ|DÄₓ:ₙ̅¹
///
/// Present value of a decreasing endowment insurance deferred for t years with due payment.
/// Pays a decreasing benefit if death occurs within n years after deferment, or a pure endowment if survival occurs.
pub fn t_DAA_x_n_(
    config: &MortTableConfig,
    entry_age: i32,
    t: i32,
    x: i32,
    n: i32,
) -> PolarsResult<f64> {
    let new_config = get_new_config_with_selected_table(config, entry_age)?;
    whole::t_DAA_x_n(&new_config, t, x, n)
}

//-----------------Geometric increasing------------------

/// Deferred due geometric whole life:
/// ₜ|Äₓ⁽ᵍ⁾ = Äₓ⁽ᵍ⁾ - Ä¹ₓ:ₜ̅⁽ᵍ⁾
///
/// Death benefit grows geometrically at rate g each year after deferment with due payment.
/// Uses adjusted interest rate i′ = (1+i)/(1+g) - 1 for geometric growth calculations.
pub fn t_gAA_x_(
    config: &MortTableConfig,
    entry_age: i32,
    t: i32,
    x: i32,
    g: f64,
) -> PolarsResult<f64> {
    let new_config = get_new_config_with_selected_table(config, entry_age)?;
    whole::t_gAA_x(&new_config, t, x, g)
}

/// Deferred due geometric n-year term:
/// ₜ|Ä¹ₓ:ₙ̅⁽ᵍ⁾ = (Dₓ₊ₜ/Dₓ) · Ä¹ₓ₊ₜ:ₙ̅⁽ᵍ⁾
///
/// Death benefit grows geometrically at rate g for n years after deferment with due payment.
/// Uses adjusted interest rate i′ = (1+i)/(1+g) - 1 for geometric growth calculations.
pub fn t_gAA_x1_n_(
    config: &MortTableConfig,
    entry_age: i32,
    t: i32,
    x: i32,
    n: i32,
    g: f64,
) -> PolarsResult<f64> {
    let new_config = get_new_config_with_selected_table(config, entry_age)?;
    whole::t_gAA_x1_n(&new_config, t, x, n, g)
}

/// Deferred due geometric n-year pure endowment:
/// ₜ|Äₓ:ₙ̅¹⁽ᵍ⁾ = Äₓ:ₜ₊ₙ̅¹⁽ᵍ⁾
///
/// Death benefit grows geometrically at rate g for n years after deferment with due payment.
/// Uses adjusted interest rate i′ = (1+i)/(1+g) - 1 for geometric growth calculations.
pub fn t_gAA_x_n1_(
    config: &MortTableConfig,
    entry_age: i32,
    t: i32,
    x: i32,
    n: i32,
    g: f64,
) -> PolarsResult<f64> {
    let new_config = get_new_config_with_selected_table(config, entry_age)?;
    whole::t_gAA_x_n1(&new_config, t, x, n, g)
}

/// Deferred due geometric n-year endowment:
/// ₜ|Äₓ:ₙ̅⁽ᵍ⁾ = ₜ|Ä¹ₓ:ₙ̅⁽ᵍ⁾ + ₜ|Äₓ:ₙ̅¹⁽ᵍ⁾
///
/// Death benefit grows geometrically at rate g for n years after deferment with due payment.
/// Uses adjusted interest rate i′ = (1+i)/(1+g) - 1 for geometric growth calculations.
pub fn t_gAA_x_n_(
    config: &MortTableConfig,
    entry_age: i32,
    t: i32,
    x: i32,
    n: i32,
    g: f64,
) -> PolarsResult<f64> {
    let new_config = get_new_config_with_selected_table(config, entry_age)?;
    whole::t_gAA_x_n(&new_config, t, x, n, g)
}
