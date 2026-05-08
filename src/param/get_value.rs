use crate::mt_config::MortTableConfig;
use bon::Builder;
use garde::Validate;

use super::validation::{
    collect_age_bounds_errors, validate_age_boundaries, validate_entry_age, ErrorVec,
};

#[derive(Debug, Clone, Validate, Builder)]
#[garde(allow_unvalidated)]
pub struct GetValueFunctionValidation {
    // Mortality table configuration
    // Validate when ParamConfig is validated
    #[garde(dive)]
    pub mt: MortTableConfig,

    // Age - Cannot exceed min and max from mort_table
    // Basic range validation (0-150 years old is reasonable)
    #[garde(range(min = 0.0, max = 130.0))]
    pub x: f64,

    // Entry age for select-ultimate tables,
    // Entry age cannot exceed age x
    #[garde(range(max = 130))]
    pub entry_age: Option<u32>,
}

impl GetValueFunctionValidation {
    /// Validate with cross-field validation using Result<(), garde::Report>
    pub fn validate_all(&self) -> Result<(), garde::Report> {
        // First run garde's built-in validations
        self.validate()?;

        // Then run our custom cross-field validations
        self.validate_custom_constraints()
    }

    /// Custom cross-field validations that garde can't handle with attributes
    fn validate_custom_constraints(&self) -> Result<(), garde::Report> {
        let mut report = garde::Report::new();
        let mut errors: ErrorVec = Vec::new();

        // Get age bounds and collect any errors
        let age_bounds = collect_age_bounds_errors(&self.mt, &mut errors);

        // If we can't get age bounds, return early
        let (min_age, max_age) = match age_bounds {
            Some(bounds) => bounds,
            None => {
                for (path, message) in errors {
                    report.append(garde::Path::new(path), garde::Error::new(message));
                }
                return Err(report);
            }
        };

        // Validate age boundaries
        validate_age_boundaries(self.x, min_age, max_age, &mut errors);

        // Validate entry age constraints
        validate_entry_age(self.entry_age, self.x, &mut errors);

        // Convert errors to report
        for (path, message) in errors {
            report.append(garde::Path::new(path), garde::Error::new(message));
        }

        // Return Ok if no errors, otherwise return the complete report
        if report.is_empty() {
            Ok(())
        } else {
            Err(report)
        }
    }
}

// =============================================================================
// UNIT TEST
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mt_config::mt_data::MortData;

    // ------------------------------- Age -------------------------------------

    #[test]
    fn test_negative_age_validation() {
        let mort_data = MortData::from_builtin("AM92").unwrap();
        let mt = MortTableConfig::builder().data(mort_data).build().unwrap();

        // Test with negative age (should fail)
        let params = GetValueFunctionValidation::builder()
            .mt(mt.clone())
            .x(-5.0)
            .build();

        let result = params.validate_all();
        assert!(result.is_err(), "Negative age should fail validation");
    }

    #[test]
    fn test_age_below_min_age() {
        let mort_data = MortData::from_builtin("AM92").unwrap();
        let mt = MortTableConfig::builder().data(mort_data).build().unwrap();

        let min_age = mt.min_age().unwrap();

        // Test with age below min_age (should fail)
        let below_min = (min_age as f64) - 1.0;
        let params = GetValueFunctionValidation::builder()
            .mt(mt.clone())
            .x(below_min)
            .build();

        let result = params.validate_all();
        assert!(
            result.is_err(),
            "Age {} should fail validation (min_age is {})",
            below_min,
            min_age
        );

        // Test with age equal to min_age (should pass)
        let params = GetValueFunctionValidation::builder()
            .mt(mt.clone())
            .x(min_age as f64)
            .build();

        let result = params.validate_all();
        assert!(
            result.is_ok(),
            "Age {} (min_age) should pass validation",
            min_age
        );
    }

    #[test]
    fn test_age_exceeds_table_max_age() {
        let mort_data = MortData::from_builtin("AM92").unwrap();
        let mt = MortTableConfig::builder().data(mort_data).build().unwrap();

        let max_age = mt.max_age().unwrap();

        // Test with age exceeding actual max_age from AM92 (should fail)
        let above_max = (max_age as f64) + 1.0;
        let params = GetValueFunctionValidation::builder()
            .mt(mt.clone())
            .x(above_max)
            .build();

        let result = params.validate_all();
        assert!(
            result.is_err(),
            "Age {} should fail validation (max_age is {})",
            above_max,
            max_age
        );

        // Test with age equal to max_age (should pass)
        let params = GetValueFunctionValidation::builder()
            .mt(mt.clone())
            .x(max_age as f64)
            .build();

        let result = params.validate_all();
        assert!(
            result.is_ok(),
            "Age {} (max_age) should pass validation",
            max_age
        );
    }

    // ------------------------------- Entry age -------------------------------

    #[test]
    fn test_entry_age_exceeds_age_x() {
        let mort_data = MortData::from_builtin("AM92").unwrap();
        let mt = MortTableConfig::builder().data(mort_data).build().unwrap();

        // Test with entry_age > x (should fail)
        let params = GetValueFunctionValidation::builder()
            .mt(mt.clone())
            .x(30.0)
            .entry_age(35)
            .build();

        let result = params.validate_all();
        assert!(result.is_err(), "entry_age 35 should fail when age x is 30");
    }

    #[test]
    fn test_entry_age_non_negative() {
        // entry_age is Option<u32>, so negative values are not possible at type level
        // This test verifies that entry_age = 0 is valid (minimum u32)
        let mort_data = MortData::from_builtin("AM92").unwrap();
        let mt = MortTableConfig::builder().data(mort_data).build().unwrap();

        // Test with entry_age = 0 (should pass - non-negative)
        let params = GetValueFunctionValidation::builder()
            .mt(mt.clone())
            .x(50.0)
            .entry_age(0)
            .build();

        let result = params.validate_all();
        assert!(
            result.is_ok(),
            "entry_age 0 should pass validation (non-negative)"
        );
    }

    #[test]
    fn test_entry_age_valid() {
        let mort_data = MortData::from_builtin("AM92").unwrap();
        let mt = MortTableConfig::builder().data(mort_data).build().unwrap();

        // Test with entry_age = x (should pass)
        let params = GetValueFunctionValidation::builder()
            .mt(mt.clone())
            .x(40.0)
            .entry_age(40)
            .build();

        let result = params.validate_all();
        assert!(
            result.is_ok(),
            "entry_age equal to x should pass validation"
        );

        // Test with entry_age = None (should pass)
        let params = GetValueFunctionValidation::builder()
            .mt(mt.clone())
            .x(40.0)
            .build();

        let result = params.validate_all();
        assert!(result.is_ok(), "entry_age = None should pass validation");
    }
}
