//! # Select and Ultimate Survival Functions
//!
//! This module provides functions for calculating survival probabilities using select and ultimate
//! mortality tables. These functions handle mortality rates that depend on both attained age
//! and duration since policy issue or entry.
//!
//! ## Functions
//!
//! - [`tpx_`] - Calculate t-year survival probability from age x with entry age selection
//! - [`tqx_`] - Calculate t-year mortality probability from age x with entry age selection
//!
//! ## Mathematical Foundation
//!
//! **Select and Ultimate Tables** use different mortality rates based on time since policy issue:
//!
//! - **Select Period**: Uses mortality rates qₓ⁽ᵏ⁾ where k is duration since entry
//! - **Ultimate Period**: Uses ultimate mortality rates qₓ after select period ends
//! - **Selection Effect**: Lower mortality rates in early policy years due to underwriting
//!
//! **Survival Probability**: ₜpₓ = ∏(k=0 to t-1) pₓ₊ₖ⁽ᵈ⁺ᵏ⁾
//!
//! Where:
//! - `x` = starting age
//! - `t` = time period in years
//! - `d` = duration since entry at starting age
//! - `qₓ⁽ᵈ⁾` = select mortality rate at age x, duration d
//!
//! ## Usage Example
//!
//! ```rust
//! use rslife::prelude::*;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let xml = MortXML::from_url_id(3604)?;
//! let config = MortTableConfig {
//!     xml,
//!     radix: 100_000,
//!     pct: Some(1.0),
//!     int_rate: None,
//!     assumption: None,
//! };
//!
//! // Calculate 5-year survival from age 30, with entry at age 26
//! let survival = rslife::selection::survivals::tpx_(&config, 5, 30, 26)?;
//! let mortality = rslife::selection::survivals::tqx_(&config, 5, 30, 26)?;
//!
//! assert!((survival + mortality - 1.0).abs() < 1e-10);
//! assert!(survival > 0.0 && survival <= 1.0);
//! # Ok(())
//! # }
//! ```

use self::helpers::get_selected_mortality_table;
use super::*;

/// Calculate ₜpₓ_ - probability of surviving t years starting at age x with entry age selection (whole ages only).
///
/// This function computes the probability that a person aged x will survive for t years
/// using select and ultimate mortality tables. For select tables, the mortality rates
/// depend on both attained age and duration since entry/policy issue.
///
/// # Arguments
///
/// * `config` - Mortality table configuration containing the select/ultimate mortality data
/// * `t` - Time period in years (must be a positive integer)
/// * `x` - Starting age (must be a positive integer)
/// * `entry_age` - Age at entry/policy issue (used for select period calculations)
///
/// # Returns
///
/// Returns `PolarsResult<f64>` containing the survival probability (between 0.0 and 1.0).
///
/// # Mathematical Foundation
///
/// For select and ultimate tables:
/// - **Select Period**: Uses mortality rates qₓ⁽ᵏ⁾ where k is duration since entry
/// - **Ultimate Period**: Uses ultimate mortality rates qₓ after select period ends
/// - **Selection Effect**: Lower mortality rates in early policy years due to underwriting
///
/// # Table Structure
///
/// The function handles two table types:
/// 1. **Ultimate Tables**: No duration column, uses standard calculation
/// 2. **Select Tables**: Has duration column, uses entry age to determine rates
///
/// # Examples
///
/// ```rust
/// use rslife::prelude::*;
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// // Load a select and ultimate table
/// let xml = MortXML::from_url_id(3604)?;
/// let config = MortTableConfig {
///     xml,
///     radix: 100_000,
///     pct: Some(1.0),
///     int_rate: None,
///     assumption: None,
/// };
///
/// // 5-year survival from age 30, with entry at age 26
/// let prob = rslife::selection::survivals::tpx_(&config, 5, 30, 26)?;
/// assert!(prob > 0.0 && prob <= 1.0);
/// # Ok(())
/// # }
/// ```
///
/// # Errors
///
/// Returns `PolarsError` if:
/// - Entry age is less than minimum age in table
/// - Entry age is greater than starting age
/// - The mortality table cannot be accessed
/// - No mortality data found for required ages
pub fn tpx_(config: &MortTableConfig, t: f64, x: f64, entry_age: i32) -> PolarsResult<f64> {
    let df = &config.xml.tables[0].values;

    // Entry age cannot be greater than x
    if (entry_age as f64) > x {
        return Err(PolarsError::ComputeError(
            format!("Entry age {entry_age} cannot be greater than starting age {x}",).into(),
        ));
    }

    // If mortality table does not have column "duration" - it is ultimate table then we can use whole::survivals::tpx directly
    if !df.get_column_names().contains(&&"duration".into()) {
        return fractional::survivals::tpx(config, t, x);
    }

    let selected_df = get_selected_mortality_table(config, entry_age)?;

    // Create a new MortTableConfig with the modified DataFrame
    let mut new_config = config.clone();
    new_config.xml.tables[0].values = selected_df;

    // Calculate the survival probability using the new configuration
    fractional::survivals::tpx(&new_config, t, x)
}

/// Calculate ₜqₓ - probability of dying within t years starting at age x with entry age selection (whole ages only).
///
/// This is the complement of [`tpx_`]: ₜqₓ = 1 - ₜpₓ.
pub fn tqx_(config: &MortTableConfig, t: f64, x: f64, entry_age: i32) -> PolarsResult<f64> {
    let tpx = tpx_(config, t, x, entry_age)?;
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
    fn test_tpx_selection() {
        let xml = MortXML::from_url_id(3604).expect("Failed to load XML");
        let config = MortTableConfig {
            xml,
            radix: 100_000,
            pct: Some(1.0),
            int_rate: None,
            assumption: None,
        };

        let prob = tpx_(&config, 5.0, 30.0, 26).expect("tpx_ failed");
        assert!(prob > 0.0 && prob <= 1.0, "tpx_ should be a probability");
    }

    #[test]
    fn test_tqx_selection() {
        let xml = MortXML::from_url_id(3604).expect("Failed to load XML");
        let config = MortTableConfig {
            xml,
            radix: 100_000,
            pct: Some(1.0),
            int_rate: None,
            assumption: None,
        };

        let tqx_val = tqx_(&config, 5.0, 30.0, 26).expect("tqx_ failed");
        let tpx_val = tpx_(&config, 5.0, 30.0, 26).expect("tpx_ failed");
        assert!(
            tqx_val >= 0.0 && tqx_val <= 1.0,
            "tqx_ should be a probability"
        );
        assert!(
            (tqx_val + tpx_val - 1.0).abs() < 1e-10,
            "tqx_ + tpx_ should equal 1"
        );
    }

    #[test]
    fn test_fractional_time_selection() {
        let xml = MortXML::from_url_id(3604).expect("Failed to load XML");
        let config = MortTableConfig {
            xml,
            radix: 100_000,
            pct: Some(1.0),
            int_rate: None,
            assumption: Some(crate::mt_config::AssumptionEnum::UDD),
        };

        // Test fractional time periods
        let prob_half_year = tpx_(&config, 0.5, 30.0, 26).expect("tpx_ failed");
        let prob_one_year = tpx_(&config, 1.0, 30.0, 26).expect("tpx_ failed");

        assert!(
            prob_half_year > 0.0 && prob_half_year <= 1.0,
            "Half-year survival should be a probability"
        );
        assert!(
            prob_one_year > 0.0 && prob_one_year <= 1.0,
            "One-year survival should be a probability"
        );
        assert!(
            prob_half_year >= prob_one_year,
            "Shorter time period should have higher survival probability"
        );
    }

    #[test]
    fn test_fractional_age_selection() {
        let xml = MortXML::from_url_id(3604).expect("Failed to load XML");
        let config = MortTableConfig {
            xml,
            radix: 100_000,
            pct: Some(1.0),
            int_rate: None,
            assumption: Some(crate::mt_config::AssumptionEnum::UDD),
        };

        // Test fractional ages
        let prob_whole_age = tpx_(&config, 1.0, 30.0, 26).expect("tpx_ failed");
        let prob_fractional_age = tpx_(&config, 1.0, 30.25, 26).expect("tpx_ failed");

        assert!(
            prob_whole_age > 0.0 && prob_whole_age <= 1.0,
            "Whole age survival should be a probability"
        );
        assert!(
            prob_fractional_age > 0.0 && prob_fractional_age <= 1.0,
            "Fractional age survival should be a probability"
        );
    }

    #[test]
    fn test_error_conditions() {
        let xml = MortXML::from_url_id(3604).expect("Failed to load XML");
        let config = MortTableConfig {
            xml,
            radix: 100_000,
            pct: Some(1.0),
            int_rate: None,
            assumption: None,
        };

        // Test entry age greater than starting age
        let result = tpx_(&config, 5.0, 30.0, 35);
        assert!(
            result.is_err(),
            "Should return error when entry age > starting age"
        );

        // Test the error message
        if let Err(e) = result {
            assert!(
                e.to_string()
                    .contains("Entry age 35 cannot be greater than starting age 30")
            );
        }
    }
}
