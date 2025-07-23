use super::*;

//-----------------Basic------------------

/// Deferred immediate  whole life insurance:
/// ₜ|Aₓ = Aₓ - A¹ₓ:ₜ̅
///
/// Present value of $1 payable at the moment of death, provided death occurs after a deferment period of t years.
/// This represents a deferred whole life insurance benefit, where payment is made only if the insured survives the deferment period.
pub fn t_A_x_(config: &MortTableConfig, entry_age: i32, t: i32, x: i32) -> PolarsResult<f64> {
    let new_config = get_new_config_with_selected_table(config, entry_age)?;
    whole::t_A_x(&new_config, t, x)
}

/// Deferred immediate term life insurance:
/// ₜ|A¹ₓ:ₙ̅ = (Dₓ₊ₜ/Dₓ) · A¹ₓ₊ₜ:ₙ̅
///
/// Present value of $1 payable at the moment of death, provided death occurs within n years after a deferment period of t years.
/// This represents a deferred term life insurance benefit, where payment is made only if the insured survives the deferment period and then dies within the following n years.
pub fn t_A_x1_n_(
    config: &MortTableConfig,
    entry_age: i32,
    t: i32,
    x: i32,
    n: i32,
) -> PolarsResult<f64> {
    let new_config = get_new_config_with_selected_table(config, entry_age)?;
    whole::t_A_x1_n(&new_config, t, x, n)
}

/// Deferred immediate pure endowment:
///ₜ|Aₓ:ₙ̅¹ = Aₓ:ₜ₊ₙ̅¹ = Dₓ₊ₜ₊ₙ / Dₓ
///
/// Present value of $1 payable only if the insured survives the entire deferment period of n years.
/// This represents a deferred pure endowment, where payment is made at the end of n years provided the insured is still alive.
pub fn t_A_x_n1_(
    config: &MortTableConfig,
    entry_age: i32,
    t: i32,
    x: i32,
    n: i32,
) -> PolarsResult<f64> {
    let new_config = get_new_config_with_selected_table(config, entry_age)?;
    whole::t_A_x_n1(&new_config, t, x, n)
}

/// Deferred immediate Endowment insurance:
/// ₜ|Aₓ:ₙ̅ = ₜ|A¹ₓ:ₙ̅ + ₜ|Aₓ:ₙ̅¹
///
/// Present value of $1 payable either at the moment of death (if it occurs within n years after a deferment period of t years), or at the end of n years if the insured survives the entire period.
/// This represents a deferred endowment insurance benefit, combining deferred term insurance and deferred pure endowment.
pub fn t_A_x_n_(
    config: &MortTableConfig,
    entry_age: i32,
    t: i32,
    x: i32,
    n: i32,
) -> PolarsResult<f64> {
    let new_config = get_new_config_with_selected_table(config, entry_age)?;
    whole::t_A_x_n(&new_config, t, x, n)
}

//-----------------Increasing------------------

/// Deferred immediate increasing whole life:
/// ₜ|IAₓ = (Dₓ₊ₜ/Dₓ) · IAₓ₊ₜ
///
/// Death benefit increases by 1 each year: if death occurs in the k-th policy year after the deferment period, the benefit paid is k.
pub fn t_IA_x_(config: &MortTableConfig, entry_age: i32, t: i32, x: i32) -> PolarsResult<f64> {
    let new_config = get_new_config_with_selected_table(config, entry_age)?;
    whole::t_IA_x(&new_config, t, x)
}

/// Deferred immediate increasing term:
/// ₜ|IA¹ₓ:ₙ̅ = (Dₓ₊ₜ/Dₓ) · IA¹ₓ₊ₜ:ₙ̅
///
/// Present value of an increasing term insurance where the death benefit increases by 1 each year, payable if death occurs within n years after a deferment period of t years.
pub fn t_IA_x1_n_(
    config: &MortTableConfig,
    entry_age: i32,
    t: i32,
    x: i32,
    n: i32,
) -> PolarsResult<f64> {
    let new_config = get_new_config_with_selected_table(config, entry_age)?;
    whole::t_IA_x1_n(&new_config, t, x, n)
}

/// Deferred immediate increasing pure endowment:
/// ₜ|IAₓ:ₙ̅¹ =  (Dₓ₊ₜ/Dₓ) · IAₓ₊ₜ:ₙ̅¹
///
/// Present value of an increasing pure endowment: pays a benefit of n if the insured survives n years after a deferment period of t years.
/// This is equivalent to n times the deferred pure endowment, and is rarely used in practice.
pub fn t_IA_x_n1_(
    config: &MortTableConfig,
    entry_age: i32,
    t: i32,
    x: i32,
    n: i32,
) -> PolarsResult<f64> {
    let new_config = get_new_config_with_selected_table(config, entry_age)?;
    whole::t_IA_x_n1(&new_config, t, x, n)
}

/// Deferred immediate endowment insurance:
/// ₜ|IAₓ:ₙ̅ = IA¹ₓ:ₙ̅ + IA¹ₓ:ₙ̅
///
/// Present value of an increasing endowment insurance: pays a benefit increasing by 1 each year if death occurs within n years after a deferment period of t years, or a benefit of n if the insured survives the entire period.
pub fn t_IA_x_n_(
    config: &MortTableConfig,
    entry_age: i32,
    t: i32,
    x: i32,
    n: i32,
) -> PolarsResult<f64> {
    let new_config = get_new_config_with_selected_table(config, entry_age)?;
    whole::t_IA_x_n(&new_config, t, x, n)
}

//-----------------Decreasing------------------
// Note: There should starting amount hence DAₓ is not applicable

/// Immediate decreasing term:
/// ₜ|DA¹ₓ:ₙ̅ = (Dₓ₊ₜ/Dₓ) · DA¹ₓ₊ₜ:ₙ̅
///
/// Death benefit decreases by 1 each policy year after the deferment period: pays n if death occurs in the first year after deferment, n-1 in the second year, ..., down to 1 in the nth year, provided death occurs within n years after the deferment period.
pub fn t_DA_x1_n_(
    config: &MortTableConfig,
    entry_age: i32,
    t: i32,
    x: i32,
    n: i32,
) -> PolarsResult<f64> {
    let new_config = get_new_config_with_selected_table(config, entry_age)?;
    whole::t_DA_x1_n(&new_config, t, x, n)
}

/// Deferred immediate decreasing pure endowment:
/// ₜ|DAₓ:ₙ̅¹ = (Dₓ₊ₜ/Dₓ) · DAₓ₊ₜ:ₙ̅¹
///
/// Deferred immediate decreasing pure endowment:
/// ₜ|DAₓ:ₙ̅¹ = (Dₓ₊ₜ/Dₓ) · DAₓ₊ₜ:ₙ̅¹
///
/// Present value of a decreasing pure endowment: pays a benefit that decreases by 1 each year, provided the insured survives n years after a deferment period of t years.
/// This is rarely used in practice and is mathematically equivalent to a deferred pure endowment.
pub fn t_DA_x_n1_(
    config: &MortTableConfig,
    entry_age: i32,
    t: i32,
    x: i32,
    n: i32,
) -> PolarsResult<f64> {
    let new_config = get_new_config_with_selected_table(config, entry_age)?;
    whole::t_DA_x_n1(&new_config, t, x, n)
}

/// Deferred immediate decreasing endowment insurance:
/// ₜ|DAₓ:ₙ̅ = ₜ|DA¹ₓ:ₙ̅ + ₜ|DA¹ₓ:ₙ̅
///
/// Present value of $1 payable at the moment of death if it occurs within n years, or $1 payable at the end of n years if the insured survives the entire period.
pub fn t_DA_x_n_(
    config: &MortTableConfig,
    entry_age: i32,
    t: i32,
    x: i32,
    n: i32,
) -> PolarsResult<f64> {
    let new_config = get_new_config_with_selected_table(config, entry_age)?;
    whole::t_DA_x_n(&new_config, t, x, n)
}

//-----------------Geometric increasing------------------

/// Deferred immediate geometric whole life:
/// ₜ|Aₓ⁽ᵍ⁾ with adjusted interest rate i′ = (1+i)/(1+g) - 1
///
/// Present value of a whole life insurance where the death benefit increases geometrically at rate g each year after a deferment period of t years.
pub fn t_gA_x_(
    config: &MortTableConfig,
    entry_age: i32,
    t: i32,
    x: i32,
    g: f64,
) -> PolarsResult<f64> {
    let new_config = get_new_config_with_selected_table(config, entry_age)?;
    whole::t_gA_x(&new_config, t, x, g)
}

/// Immediate geometric n-year term:
/// ₜ|A¹ₓ:ₙ̅⁽ᵍ⁾ with adjusted interest rate i′ = (1+i)/(1+g) - 1
///
/// Death benefit grows geometrically at rate g for n years.
pub fn t_gA_x1_n_(
    config: &MortTableConfig,
    entry_age: i32,
    t: i32,
    x: i32,
    n: i32,
    g: f64,
) -> PolarsResult<f64> {
    let new_config = get_new_config_with_selected_table(config, entry_age)?;
    whole::t_gA_x1_n(&new_config, t, x, n, g)
}

/// Immediate geometric n-year pure endowment:
/// ₜ|Aₓ:ₙ̅¹⁽ᵍ⁾ with adjusted interest rate i′ = (1+i)/(1+g) - 1
///
/// Death benefit grows geometrically at rate g for n years.
pub fn t_gA_x_n1_(
    config: &MortTableConfig,
    entry_age: i32,
    t: i32,
    x: i32,
    n: i32,
    g: f64,
) -> PolarsResult<f64> {
    let new_config = get_new_config_with_selected_table(config, entry_age)?;
    whole::t_gA_x_n1(&new_config, t, x, n, g)
}

/// Immediate geometric n-year endowment:
/// ₜ|Aₓ:ₙ̅⁽ᵍ⁾ with adjusted interest rate i′ = (1+i)/(1+g) - 1
///
/// Present value of an endowment insurance where the benefit increases geometrically at rate g each year for n years after a deferment period of t years.
/// Pays a geometrically increasing benefit if death occurs within n years after deferment, or the full geometric benefit at the end of n years if the insured survives.
pub fn t_gA_x_n_(
    config: &MortTableConfig,
    entry_age: i32,
    t: i32,
    x: i32,
    n: i32,
    g: f64,
) -> PolarsResult<f64> {
    let new_config = get_new_config_with_selected_table(config, entry_age)?;
    whole::t_gA_x_n(&new_config, t, x, n, g)
}
