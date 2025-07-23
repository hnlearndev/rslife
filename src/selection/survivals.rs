use self::helpers::get_new_config_with_selected_table;
use super::*;

/// Calculate ₜpₓ_ : probability of surviving t years from age x with entry age selection (whole ages only).
///
/// Uses select rates for duration since entry, then ultimate rates after select period.
pub fn tpx_(config: &MortTableConfig, entry_age: i32, t: f64, x: f64) -> PolarsResult<f64> {
    // Entry age cannot be greater than x
    if (entry_age as f64) > x {
        return Err(PolarsError::ComputeError(
            format!("Entry age {entry_age} cannot be greater than age {x}").into(),
        ));
    }

    let new_config = get_new_config_with_selected_table(config, entry_age)?;

    fractional::survivals::tpx(&new_config, t, x)
}

/// Calculate ₜqₓ - probability of dying within t years starting at age x with entry age selection (whole ages only).
///
/// This is the complement of [`tpx_`]: ₜqₓ = 1 - ₜpₓ.
pub fn tqx_(config: &MortTableConfig, entry_age: i32, t: f64, x: f64) -> PolarsResult<f64> {
    let result = 1.0 - tpx_(config, entry_age, t, x)?;
    Ok(result)
}

//-----------------------------------------------------------
// UNIT TESTS
//-----------------------------------------------------------
#[cfg(test)]
mod tests {
    // use super::*;
}
