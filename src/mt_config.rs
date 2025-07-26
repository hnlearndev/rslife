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

use crate::xml::MortXML;
use garde::Validate;

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

/// Configuration for generating mortality tables with demographic and actuarial functions.
///
/// Generates mortality tables from XML data with configurable detail levels, from basic
/// rates to complete commutation functions for actuarial present value calculations.
///
/// See [`MortTableConfig::gen_mort_table`] for detailed usage and examples.
#[derive(Debug, Clone, Validate)]
#[garde(allow_unvalidated)]
pub struct MortTableConfig {
    /// Source mortality data (must contain exactly one age-based table).
    pub xml: MortXML,

    /// Initial population size (radix). Common values: 100,000 (standard), 1,000,000 (precise).
    #[garde(range(min = 1))]
    pub radix: Option<i32>,

    /// Mortality rate multiplier. Examples: 1.0 (standard), 0.75 (preferred), 0.5 (reduced).
    #[garde(custom(validate_pct))]
    pub pct: Option<f64>,

    /// Interest rate for commutation functions (e.g., 0.03 for 3%). Required for detail levels 3+.
    pub int_rate: Option<f64>,

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
    /// Validate with cross-field validation
    pub fn validate_all(&self) -> Result<(), String> {
        // First run garde's built-in validations
        if let Err(e) = self.validate() {
            return Err(e.to_string());
        }

        // Dummy validation for consistency
        // Then check cross-field validation: entry_age cannot exceed age_x

        Ok(())
    }
}

// Default apply no interest rate, no radix, 100% mortality rates UDD assumption.
impl Default for MortTableConfig {
    fn default() -> Self {
        MortTableConfig {
            xml: MortXML::default(),
            radix: None,
            pct: Some(1.0),
            int_rate: None,
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
            xml: MortXML::default(),
            radix: None,
            pct: None, // Valid: None is allowed
            int_rate: None,
            assumption: None,
        };

        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_pct_validation_valid_positive() {
        let valid_pcts = vec![0.1, 0.5, 0.75, 1.0, 1.5, 2.0];

        for pct_val in valid_pcts {
            let config = MortTableConfig {
                xml: MortXML::default(),
                radix: None,
                pct: Some(pct_val), // Valid: > 0.0
                int_rate: None,
                assumption: None,
            };

            assert!(config.validate().is_ok(), "pct {} should be valid", pct_val);
        }
    }

    #[test]
    fn test_pct_validation_invalid_zero() {
        let config = MortTableConfig {
            xml: MortXML::default(),
            radix: None,
            pct: Some(0.0), // Invalid: cannot be 0.0
            int_rate: None,
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
                xml: MortXML::default(),
                radix: None,
                pct: Some(pct_val), // Negative values are allowed (might represent special cases)
                int_rate: None,
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
            xml: MortXML::default(),
            radix: Some(0), // Invalid: < 1
            pct: Some(0.5), // Valid
            int_rate: None,
            assumption: None,
        };

        // Validation should fail due to radix
        assert!(config_invalid_radix.validate().is_err());
    }

    #[test]
    fn test_multiple_validation_errors() {
        let config = MortTableConfig {
            xml: MortXML::default(),
            radix: Some(0), // Invalid: < 1 (built-in validation)
            pct: Some(0.0), // Invalid: cannot be 0.0 (custom validation)
            int_rate: None,
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
            xml: MortXML::default(),
            radix: Some(100_000),
            pct: Some(1.0), // 100% of standard rates
            int_rate: Some(0.03),
            assumption: Some(AssumptionEnum::UDD),
        };
        assert!(standard.validate().is_ok());

        // Scenario 2: Preferred rates (reduced mortality)
        let preferred = MortTableConfig {
            xml: MortXML::default(),
            radix: Some(100_000),
            pct: Some(0.75), // 75% of standard rates
            int_rate: Some(0.03),
            assumption: Some(AssumptionEnum::CFM),
        };
        assert!(preferred.validate().is_ok());

        // Scenario 3: Substandard rates (increased mortality)
        let substandard = MortTableConfig {
            xml: MortXML::default(),
            radix: Some(50_000),
            pct: Some(1.5), // 150% of standard rates
            int_rate: Some(0.04),
            assumption: Some(AssumptionEnum::HPB),
        };
        assert!(substandard.validate().is_ok());
    }

    #[test]
    fn test_edge_case_very_small_pct() {
        // Very small but non-zero pct should be valid
        let config = MortTableConfig {
            xml: MortXML::default(),
            radix: None,
            pct: Some(0.001), // Very small but > 0.0
            int_rate: None,
            assumption: None,
        };

        assert!(config.validate().is_ok());
    }
}
