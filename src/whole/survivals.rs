//! # Whole Life Survival Functions
//!
//! This module provides functions for calculating survival probabilities for whole ages
//! using mortality tables. These functions work with integer ages and time periods.
//!
//! ## Functions
//!
//! - [`tpx`] - Calculate t-year survival probability from age x
//! - [`tqx`] - Calculate t-year mortality probability from age x
//!
//! ## Mathematical Foundation
//!
//! For whole ages, survival probabilities are calculated using the fundamental relationship:
//!
//! **Survival Probability**: ₜpₓ = ∏(k=0 to t-1) pₓ₊ₖ = ∏(k=0 to t-1) (1 - qₓ₊ₖ)
//!
//! **Mortality Probability**: ₜqₓ = 1 - ₜpₓ
//!
//! Where:
//! - `x` = starting age (integer)
//! - `t` = time period in years (integer)
//! - `qₓ` = probability of death within one year at age x
//! - `pₓ` = probability of survival for one year at age x = (1 - qₓ)
//!
//! ## Usage Example
//!
//! ```rust
//! use rslife::prelude::*;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let xml = MortXML::from_url_id(1704)?;
//! let config = MortTableConfig {
//!     xml,
//!     l_x_init: 100_000,
//!     pct: Some(1.0),
//!     int_rate: None,
//!     assumption: None,
//! };
//!
//! // Calculate 5-year survival probability from age 30
//! let survival = rslife::whole::survivals::tpx(&config, 5, 30)?;
//! let mortality = rslife::whole::survivals::tqx(&config, 5, 30)?;
//!
//! assert!((survival + mortality - 1.0).abs() < 1e-10);
//! # Ok(())
//! # }
//! ```

use super::*;

/// Calculate ₜpₓ - probability of surviving t years starting at age x (whole ages only).
///
/// This function computes the probability that a person aged x will survive for t years
/// by multiplying the one-year survival probabilities for each year from age x to x+t-1.
///
/// # Arguments
///
/// * `config` - Mortality table configuration containing the mortality data
/// * `t` - Time period in years (must be a positive integer)
/// * `x` - Starting age (must be a positive integer)
///
/// # Returns
///
/// Returns `PolarsResult<f64>` containing the survival probability (between 0.0 and 1.0).
///
/// # Mathematical Formula
///
/// ₜpₓ = ∏(k=0 to t-1) (1 - qₓ₊ₖ)
///
/// Where qₓ₊ₖ is the one-year mortality rate at age x+k.
///
/// # Examples
///
/// ```rust
/// use rslife::prelude::*;
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let xml = MortXML::from_url_id(1704)?;
/// let config = MortTableConfig {
///     xml,
///     l_x_init: 100_000,
///     pct: Some(1.0),
///     int_rate: None,
///     assumption: None,
/// };
///
/// // 10-year survival probability from age 25
/// let prob = rslife::whole::survivals::tpx(&config, 10, 25)?;
/// assert!(prob > 0.0 && prob <= 1.0);
/// # Ok(())
/// # }
/// ```
///
/// # Errors
///
/// Returns `PolarsError` if:
/// - The mortality table cannot be generated
/// - The specified age is not found in the mortality table
/// - Any underlying calculation fails
pub fn tpx(config: &MortTableConfig, t: i32, x: i32) -> PolarsResult<f64> {
    let mut tpx = 1.0;

    for age in x..(x + t) {
        let qx = get_value(config, age, "qx")?;
        let px = 1.0 - qx;
        tpx *= px;
    }

    Ok(tpx)
}

/// Calculate ₜqₓ - probability of dying within t years starting at age x (whole ages only).
///
/// This function computes the probability that a person aged x will die within t years.
/// It uses the complement relationship: ₜqₓ = 1 - ₜpₓ.
///
/// # Arguments
///
/// * `config` - Mortality table configuration containing the mortality data
/// * `t` - Time period in years (must be a positive integer)
/// * `x` - Starting age (must be a positive integer)
///
/// # Returns
///
/// Returns `PolarsResult<f64>` containing the mortality probability (between 0.0 and 1.0).
///
/// # Mathematical Formula
///
/// ₜqₓ = 1 - ₜpₓ = 1 - ∏(k=0 to t-1) (1 - qₓ₊ₖ)
///
/// # Examples
///
/// ```rust
/// use rslife::prelude::*;
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let xml = MortXML::from_url_id(1704)?;
/// let config = MortTableConfig {
///     xml,
///     l_x_init: 100_000,
///     pct: Some(1.0),
///     int_rate: None,
///     assumption: None,
/// };
///
/// // 5-year mortality probability from age 60
/// let mortality = rslife::whole::survivals::tqx(&config, 5, 60)?;
/// let survival = rslife::whole::survivals::tpx(&config, 5, 60)?;
///
/// // Verify they sum to 1
/// assert!((mortality + survival - 1.0).abs() < 1e-10);
/// # Ok(())
/// # }
/// ```
///
/// # Errors
///
/// Returns `PolarsError` if the underlying `tpx` calculation fails.
pub fn tqx(config: &MortTableConfig, t: i32, x: i32) -> PolarsResult<f64> {
    let tpx = tpx(config, t, x)?;
    Ok(1.0 - tpx)
}

//-----------------------------------------------------------
// UNIT TESTS
//-----------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;
    use crate::mt_config::MortTableConfig;
    use crate::xml::MortXML;

    #[test]
    fn test_tpx_basic() {
        let xml = MortXML::from_url_id(1704).expect("Failed to load XML");
        let config = MortTableConfig {
            xml,
            l_x_init: 100_000,
            pct: Some(1.0),
            int_rate: None,
            assumption: None,
        };
        let prob = tpx(&config, 5, 30).expect("tpx failed");
        assert!(prob > 0.0 && prob <= 1.0, "tpx should be a probability");
    }

    #[test]
    fn test_tqx_basic() {
        let xml = MortXML::from_url_id(1704).expect("Failed to load XML");
        let config = MortTableConfig {
            xml,
            l_x_init: 100_000,
            pct: Some(1.0),
            int_rate: None,
            assumption: None,
        };
        let tqx_val = tqx(&config, 5, 30).expect("tqx failed");
        let tpx_val = tpx(&config, 5, 30).expect("tpx failed");
        assert!(
            tqx_val >= 0.0 && tqx_val <= 1.0,
            "tqx should be a probability"
        );
        assert!(
            (tqx_val + tpx_val - 1.0).abs() < 1e-10,
            "tqx + tpx should equal 1"
        );
    }
}
