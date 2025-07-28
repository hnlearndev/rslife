//! # Mortality Table Configuration (MortTableConfig)
//!
//! Configure, adjust, and generate actuarial mortality tables from XML or DataFrame sources.
//!
//! This module provides the `MortTableConfig` struct and related types for flexible, robust configuration of mortality tables, including:
//! - Data source selection (SOA XML, custom DataFrame)
//! - Population radix and rate scaling
//! - Interest rate and commutation function support
//! - Fractional age mortality assumptions (UDD, CFM, HPB)
//!
//! ## Quick Start
//! ```rust
//! use rslife::prelude::*;
//! // Load a mortality table from SOA by ID
//! let xml = MortXML::from_url_id(1704)?;
//! let config = MortTableConfig {
//!     xml,
//!     radix: Some(100_000),
//!     pct: Some(1.0),
//!     int_rate: Some(0.03),
//!     assumption: Some(AssumptionEnum::UDD),
//! };
//! // Config is ready for use with actuarial functions
//! println!("Config created with interest rate: {:?}", config.int_rate);
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! ## Configuration Options
//! - **xml**: Source mortality data (SOA XML or DataFrame)
//! - **radix**: Initial population size (e.g., 100,000)
//! - **pct**: Mortality rate multiplier (e.g., 1.0, 0.75)
//! - **int_rate**: Interest rate for commutation functions
//! - **assumption**: Fractional age mortality assumption (UDD, CFM, HPB)
//!
//! ## See Also
//! - [`crate::xml`] for XML parsing and table structure
//! - [`crate::whole`] for ultimate actuarial functions
//! - [`crate::selection`] for select/ultimate functions
//! - [`crate::fractional`] for fractional period calculations
//! - [`crate::actuarial`] for unified high-level API

#![allow(non_snake_case)] // Allow actuarial notation (gen_Ax_IAx, etc.)

use crate::mt_data::MortData;
use bon::Builder;
use garde::Validate;

// ===============================================
// MORTALITY ASSUMPTIONS
// ===============================================

/// Mortality assumptions for fractional age calculations.
///
/// Determines how mortality is distributed within age intervals, affecting
/// fractional survival probabilities ₜpₓ for time t at age x:
///
/// - **UDD**: ₜpₓ = 1 - t·qₓ (most common, conservative)
/// - **CFM**: ₜpₓ = (1-qₓ)ᵗ (constant force, mathematical convenience)
/// - **HPB**: ₜpₓ = (1-qₓ)/(1-(1-t)·qₓ) (hyperbolic, balanced approach)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssumptionEnum {
    /// Uniform Distribution of Deaths - most common assumption.
    UDD,

    /// Constant Force of Mortality - mathematical convenience.
    CFM,

    /// Hyperbolic (Balmer) - balanced between UDD and CFM.
    HPB,
}

// ===============================================
// MORTALITY ASSUMPTIONS
// ===============================================

/// Configuration for generating mortality tables with demographic and actuarial functions.
///
/// Generates mortality tables from XML data with configurable detail levels, from basic
/// rates to complete commutation functions for actuarial present value calculations.
///
/// See [`MortTableConfig::gen_mort_table`] for detailed usage and examples.
#[derive(Debug, Clone, Validate, Builder)]
#[garde(allow_unvalidated)]
pub struct MortTableConfig {
    /// Source mortality data (must contain exactly one age-based table).
    pub data: MortData,

    /// Initial population size (radix). Common values: 100,000 (standard), 1,000,000 (precise).
    #[garde(range(min = 1))]
    pub radix: Option<i32>,

    /// Mortality rate multiplier. Examples: 1.0 (standard), 0.75 (preferred), 0.5 (reduced).
    #[garde(custom(validate_pct))]
    pub pct: Option<f64>,

    /// Mortality assumption for fractional ages (reserved for future implementation).
    pub assumption: Option<AssumptionEnum>,
}

/// Custom validation function for pct field
fn validate_pct(value: &Option<f64>, _context: &()) -> garde::Result {
    if let Some(pct_val) = value {
        if *pct_val == 0.0 {
            return Err(garde::Error::new(
                "pct cannot be 0.0 as it would make mortality rate calculations meaningless",
            ));
        }
        // Note: negative values are allowed as they might represent special actuarial cases
    }
    Ok(())
}

impl MortTableConfig {
    pub fn min_age(&self) -> f64 {
        // Get the minimum age from the dataframe
        if let Ok(age_column) = self.data.dataframe.column("age") {
            if let Ok(age_series) = age_column.f64() {
                if let Some(min_val) = age_series.iter().flatten().min_by(|a, b| a.partial_cmp(b).unwrap()) {
                    return min_val;
                }
            }
        }
        // Default to 0 if no age data is available
        0.0
    }

    pub fn max_age(&self) -> f64 {
        // Get the maximum age from the dataframe
        if let Ok(age_column) = self.data.dataframe.column("age") {
            if let Ok(age_series) = age_column.f64() {
                if let Some(max_val) = age_series.iter().flatten().max_by(|a, b| a.partial_cmp(b).unwrap()) {
                    return max_val;
                }
            }
        }
        // Default to 0 if no age data is available
        0.0
    }

    pub fn min_duration(&self) -> f64 {
        // Get the minimum duration from the dataframe (if duration column exists)
        if let Ok(duration_column) = self.data.dataframe.column("duration") {
            if let Ok(duration_series) = duration_column.f64() {
                if let Some(min_val) = duration_series.iter().flatten().min_by(|a, b| a.partial_cmp(b).unwrap()) {
                    return min_val;
                }
            }
        }
        // Default to 0 if no duration data is available
        0.0
    }

    pub fn max_duration(&self) -> f64 {
        // Get the maximum duration from the dataframe (if duration column exists)
        if let Ok(duration_column) = self.data.dataframe.column("duration") {
            if let Ok(duration_series) = duration_column.f64() {
                if let Some(max_val) = duration_series.iter().flatten().max_by(|a, b| a.partial_cmp(b).unwrap()) {
                    return max_val;
                }
            }
        }
        // Default to 0 if no duration data is available
        0.0
    }
}

// Provide defaults for all fields except `data`, which must be set manually.
impl Default for MortTableConfig {
    fn default() -> Self {
        MortTableConfig {
            data: MortData::default(),
            radix: Some(100_000),
            pct: Some(1.0),
            assumption: Some(AssumptionEnum::UDD),
        }
    }
}

// ================================================
// UNIT TESTS
// ================================================
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pct_validation_valid_none() {
        let config = MortTableConfig {
            data: MortData::default(),
            radix: None,
            pct: None, // Valid: None is allowed
            assumption: None,
        };

        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_pct_validation_valid_positive() {
        let valid_pcts = vec![0.1, 0.5, 0.75, 1.0, 1.5, 2.0];

        for pct_val in valid_pcts {
            let config = MortTableConfig {
                data: MortData::default(),
                radix: None,
                pct: Some(pct_val), // Valid: > 0.0
                assumption: None,
            };

            assert!(config.validate().is_ok(), "pct {} should be valid", pct_val);
        }
    }

    #[test]
    fn test_pct_validation_invalid_zero() {
        let config = MortTableConfig {
            data: MortData::default(),
            radix: None,
            pct: Some(0.0), // Invalid: cannot be 0.0
            assumption: None,
        };

        // Validation fails due to custom validator
        assert!(config.validate().is_err());
        let error = config.validate().unwrap_err().to_string();
        assert!(error.contains("pct cannot be 0.0"));
        assert!(error.contains("meaningless"));
    }

    #[test]
    fn test_pct_validation_negative_values() {
        let negative_pcts = vec![-0.1, -0.5, -1.0];

        for pct_val in negative_pcts {
            let config = MortTableConfig {
                data: MortData::default(),
                radix: None,
                pct: Some(pct_val), // Negative values are allowed (might represent special cases)
                assumption: None,
            };

            // Validation passes (only 0.0 is forbidden)
            assert!(
                config.validate().is_ok(),
                "pct {} should pass validation",
                pct_val
            );
        }
    }

    #[test]
    fn test_radix_validation_with_pct() {
        // Test that radix validation still works alongside pct validation
        let config_invalid_radix = MortTableConfig {
            data: MortData::default(),
            radix: Some(0), // Invalid: < 1
            pct: Some(0.5), // Valid
            assumption: None,
        };

        // Validation should fail due to radix
        assert!(config_invalid_radix.validate().is_err());
    }

    #[test]
    fn test_multiple_validation_errors() {
        let config = MortTableConfig {
            data: MortData::default(),
            radix: Some(0), // Invalid: < 1 (built-in validation)
            pct: Some(0.0), // Invalid: cannot be 0.0 (custom validation)
            assumption: None,
        };

        // Validation should fail (both radix and pct are invalid)
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_default_configuration_validity() {
        let config = MortTableConfig::default();

        // Default should pass validation
        assert!(config.validate().is_ok());

        // Check default pct value
        assert_eq!(config.pct, Some(1.0));
    }

    #[test]
    fn test_realistic_mortality_scenarios() {
        // Scenario 1: Standard mortality table
        let standard = MortTableConfig {
            data: MortData::default(),
            radix: Some(100_000),
            pct: Some(1.0), // 100% of standard rates
            assumption: Some(AssumptionEnum::UDD),
        };
        assert!(standard.validate().is_ok());

        // Scenario 2: Preferred rates (reduced mortality)
        let preferred = MortTableConfig {
            data: MortData::default(),
            radix: Some(100_000),
            pct: Some(0.75), // 75% of standard rates
            assumption: Some(AssumptionEnum::CFM),
        };
        assert!(preferred.validate().is_ok());

        // Scenario 3: Substandard rates (increased mortality)
        let substandard = MortTableConfig {
            data: MortData::default(),
            radix: Some(50_000),
            pct: Some(1.5), // 150% of standard rates
            assumption: Some(AssumptionEnum::HPB),
        };
        assert!(substandard.validate().is_ok());
    }

    #[test]
    fn test_edge_case_very_small_pct() {
        // Very small but non-zero pct should be valid
        let config = MortTableConfig {
            data: MortData::default(),
            radix: None,
            pct: Some(0.001), // Very small but > 0.0
            assumption: None,
        };

        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_age_and_duration_methods() {
        use crate::mt_data::MortData;
        use polars::prelude::*;

        // Test with basic age-only data (no duration column)
        let df_age_only = df! {
            "age" => [20.0, 21.0, 22.0],
            "qx" => [0.001, 0.002, 0.003]
        }.expect("Failed to create age-only DataFrame");

        let data_age_only = MortData::from_df(df_age_only).expect("Failed to create MortData");
        let config_age_only = MortTableConfig::builder()
            .data(data_age_only)
            .build();

        // Test age methods
        assert_eq!(config_age_only.min_age(), 20.0);
        assert_eq!(config_age_only.max_age(), 22.0);
        
        // Test duration methods (should return 0.0 when no duration column)
        assert_eq!(config_age_only.min_duration(), 0.0);
        assert_eq!(config_age_only.max_duration(), 0.0);

        // Test with age + duration data (select table)
        let df_with_duration = df! {
            "age" => [25.0, 25.0, 26.0, 26.0],
            "qx" => [0.001, 0.002, 0.002, 0.003],
            "duration" => [0.0, 1.0, 0.0, 1.0]
        }.expect("Failed to create duration DataFrame");

        let data_with_duration = MortData::from_df(df_with_duration).expect("Failed to create MortData");
        let config_with_duration = MortTableConfig::builder()
            .data(data_with_duration)
            .build();

        // Test age methods
        assert_eq!(config_with_duration.min_age(), 25.0);
        assert_eq!(config_with_duration.max_age(), 26.0);
        
        // Test duration methods (should return actual min/max duration)
        assert_eq!(config_with_duration.min_duration(), 0.0);
        assert_eq!(config_with_duration.max_duration(), 1.0);
    }
}
