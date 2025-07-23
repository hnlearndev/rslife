use super::*;

/// Calculate ₜpₓ: probability of surviving t years from age x (whole ages only).
///
/// Formula: ₜpₓ = ∏(k=0 to t-1) (1 - qₓ₊ₖ), where qₓ₊ₖ is the one-year mortality rate at age x+k.
pub fn tpx(config: &MortTableConfig, t: i32, x: i32) -> PolarsResult<f64> {
    let mut result = 1.0;

    for age in x..(x + t) {
        let qx = get_value(config, age, "qx")?;
        let px = 1.0 - qx;
        result *= px;
    }

    Ok(result)
}

/// Calculate ₜqₓ - probability of dying within t years starting at age x (fractional ages supported).
///
/// This is the complement of [`tpx`]: ₜqₓ = 1 - ₜpₓ.
pub fn tqx(config: &MortTableConfig, t: i32, x: i32) -> PolarsResult<f64> {
    let result = 1.0 - tpx(config, t, x)?;
    Ok(result)
}

//-----------------------------------------------------------
// UNIT TESTS
//-----------------------------------------------------------
#[cfg(test)]
mod tests {
    // use super::*;
}
