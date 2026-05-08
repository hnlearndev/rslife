use crate::mt_config::MortTableConfig;
use bon::Builder;
use garde::Validate;

use super::validation::{
    collect_age_bounds_errors, validate_age_boundaries, validate_entry_age, ErrorVec,
};

#[derive(Debug, Clone, Validate, Builder)]
#[garde(allow_unvalidated)]
pub struct SingleLifeParams {
    // Mortality table configuration
    // Validate when ParamConfig is validated
    #[garde(dive)]
    pub mt: MortTableConfig,

    // Interest rate for actuarial calculations
    // Basic range validation (reasonable interest rate range)
    pub i: f64,

    // Age - Cannot exceed min and max from mort_table
    // Basic range validation (0-150 years old is reasonable)
    #[garde(range(min = 0.0, max = 130.0))]
    pub x: f64,

    // Term - x + n cannot exceed max age from mort_table
    // n can be 0 for some actuarial calculations
    #[garde(range(min = 0.0, max = 130.0))]
    pub n: f64,

    // Deferral period
    #[garde(range(min = 0.0, max = 130.0))]
    pub t: f64,

    // Payable m-thly
    // If not None must be greater than 0
    #[garde(range(min = 1))]
    pub m: u32,

    // Mathematical moment, 1 is the first moment (mean), 2 is the second moment (variance)
    // If not None, must be greater than 0
    #[garde(range(min = 1))]
    pub moment: u32,

    // Entry age for select-ultimate tables,
    // Entry age cannot exceed age x
    #[garde(range(min = 0, max = 130))]
    pub entry_age: Option<u32>,
}

impl SingleLifeParams {
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

        // Validate term constraints (custom for SingleLifeParams)
        let x = self.x;
        let t = self.t;
        let n = self.n;
        if x + t + n > max_age {
            errors.push(("", format!(
                "age + deferral + term ({x} + {t} + {n}) cannot exceed max age {max_age} from mortality table"
            )));
        }

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
        let params = SingleLifeParams::builder()
            .mt(mt.clone())
            .i(0.04)
            .x(-5.0)
            .n(10.0)
            .t(0.0)
            .m(1)
            .moment(1)
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
        let params = SingleLifeParams::builder()
            .mt(mt.clone())
            .i(0.04)
            .x(below_min)
            .n(10.0)
            .t(0.0)
            .m(1)
            .moment(1)
            .build();

        let result = params.validate_all();
        assert!(
            result.is_err(),
            "Age {} should fail validation (min_age is {})",
            below_min,
            min_age
        );

        // Test with age equal to min_age (should pass)
        let params = SingleLifeParams::builder()
            .mt(mt.clone())
            .i(0.04)
            .x(min_age as f64)
            .n(10.0)
            .t(0.0)
            .m(1)
            .moment(1)
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
        let params = SingleLifeParams::builder()
            .mt(mt.clone())
            .i(0.04)
            .x(above_max)
            .n(0.0)
            .t(0.0)
            .m(1)
            .moment(1)
            .build();

        let result = params.validate_all();
        assert!(
            result.is_err(),
            "Age {} should fail validation (max_age is {})",
            above_max,
            max_age
        );

        // Test with age equal to max_age (should pass)
        let params = SingleLifeParams::builder()
            .mt(mt.clone())
            .i(0.04)
            .x(max_age as f64)
            .n(0.0)
            .t(0.0)
            .m(1)
            .moment(1)
            .build();

        let result = params.validate_all();
        assert!(
            result.is_ok(),
            "Age {} (max_age) should pass validation",
            max_age
        );
    }

    // ------------------------------- Term (n) --------------------------------

    #[test]
    fn test_negative_term_validation() {
        let mort_data = MortData::from_builtin("AM92").unwrap();
        let mt = MortTableConfig::builder().data(mort_data).build().unwrap();

        // Test with negative term (should fail)
        let params = SingleLifeParams::builder()
            .mt(mt.clone())
            .i(0.04)
            .x(30.0)
            .n(-5.0)
            .t(0.0)
            .m(1)
            .moment(1)
            .build();

        let result = params.validate_all();
        assert!(result.is_err(), "Negative term should fail validation");
    }

    #[test]
    fn test_term_exceeds_hardcoded_limit() {
        let mort_data = MortData::from_builtin("AM92").unwrap();
        let mt = MortTableConfig::builder().data(mort_data).build().unwrap();

        let min_age = mt.min_age().unwrap();

        // Test with term 131 (exceeds hard-coded limit of 130)
        let params = SingleLifeParams::builder()
            .mt(mt.clone())
            .i(0.04)
            .x(min_age as f64)
            .n(131.0)
            .t(0.0)
            .m(1)
            .moment(1)
            .build();

        let result = params.validate_all();
        assert!(
            result.is_err(),
            "Term 131 should fail validation (exceeds hard-coded max of 130)"
        );
    }

    #[test]
    fn test_term_exceeds_table_max_age() {
        let mort_data = MortData::from_builtin("AM92").unwrap();
        let mt = MortTableConfig::builder().data(mort_data).build().unwrap();

        let max_age = mt.max_age().unwrap();

        // Test with term that causes x + t + n to exceed max_age (should fail)
        let params = SingleLifeParams::builder()
            .mt(mt.clone())
            .i(0.04)
            .x(50.0)
            .n((max_age as f64) - 50.0 + 1.0)
            .t(0.0)
            .m(1)
            .moment(1)
            .build();

        let result = params.validate_all();
        assert!(
            result.is_err(),
            "Term causing x + t + n > max_age should fail validation"
        );

        // Test with term where x + t + n equals max_age (should pass)
        let params = SingleLifeParams::builder()
            .mt(mt.clone())
            .i(0.04)
            .x(50.0)
            .n((max_age as f64) - 50.0)
            .t(0.0)
            .m(1)
            .moment(1)
            .build();

        let result = params.validate_all();
        assert!(
            result.is_ok(),
            "Term where x + t + n = max_age should pass validation"
        );
    }

    // ------------------------------- Deferral period (t) ---------------------

    #[test]
    fn test_negative_deferral_period() {
        let mort_data = MortData::from_builtin("AM92").unwrap();
        let mt = MortTableConfig::builder().data(mort_data).build().unwrap();

        // Test with negative deferral period (should fail)
        let params = SingleLifeParams::builder()
            .mt(mt.clone())
            .i(0.04)
            .x(30.0)
            .n(10.0)
            .t(-5.0)
            .m(1)
            .moment(1)
            .build();

        let result = params.validate_all();
        assert!(
            result.is_err(),
            "Negative deferral period should fail validation"
        );
    }

    #[test]
    fn test_deferral_period_exceeds_hardcoded_limit() {
        let mort_data = MortData::from_builtin("AM92").unwrap();
        let mt = MortTableConfig::builder().data(mort_data).build().unwrap();

        let min_age = mt.min_age().unwrap();

        // Test with deferral period 131 (exceeds hard-coded limit of 130)
        let params = SingleLifeParams::builder()
            .mt(mt.clone())
            .i(0.04)
            .x(min_age as f64)
            .n(0.0)
            .t(131.0)
            .m(1)
            .moment(1)
            .build();

        let result = params.validate_all();
        assert!(
            result.is_err(),
            "Deferral period 131 should fail validation (exceeds hard-coded max of 130)"
        );
    }

    #[test]
    fn test_deferral_period_exceeds_table_max_age() {
        let mort_data = MortData::from_builtin("AM92").unwrap();
        let mt = MortTableConfig::builder().data(mort_data).build().unwrap();

        let max_age = mt.max_age().unwrap();

        // Test with deferral period that causes x + t + n to exceed max_age (should fail)
        let params = SingleLifeParams::builder()
            .mt(mt.clone())
            .i(0.04)
            .x(30.0)
            .n(10.0)
            .t((max_age as f64) - 30.0 - 10.0 + 1.0)
            .m(1)
            .moment(1)
            .build();

        let result = params.validate_all();
        assert!(
            result.is_err(),
            "Deferral period causing x + t + n > max_age should fail validation"
        );

        // Test with deferral period where x + t + n equals max_age (should pass)
        let params = SingleLifeParams::builder()
            .mt(mt.clone())
            .i(0.04)
            .x(30.0)
            .n(10.0)
            .t((max_age as f64) - 30.0 - 10.0)
            .m(1)
            .moment(1)
            .build();

        let result = params.validate_all();
        assert!(
            result.is_ok(),
            "Deferral period where x + t + n = max_age should pass validation"
        );
    }

    // ------------------------------- Entry age -------------------------------

    #[test]
    fn test_entry_age_exceeds_age_x() {
        let mort_data = MortData::from_builtin("AM92").unwrap();
        let mt = MortTableConfig::builder().data(mort_data).build().unwrap();

        // Test with entry_age > x (should fail)
        let params = SingleLifeParams::builder()
            .mt(mt.clone())
            .i(0.04)
            .x(30.0)
            .n(10.0)
            .t(0.0)
            .entry_age(35)
            .m(1)
            .moment(1)
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
        let params = SingleLifeParams::builder()
            .mt(mt.clone())
            .i(0.04)
            .x(50.0)
            .n(10.0)
            .t(0.0)
            .entry_age(0)
            .m(1)
            .moment(1)
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
        let params = SingleLifeParams::builder()
            .mt(mt.clone())
            .i(0.04)
            .x(40.0)
            .n(10.0)
            .t(0.0)
            .entry_age(40)
            .m(1)
            .moment(1)
            .build();

        let result = params.validate_all();
        assert!(
            result.is_ok(),
            "entry_age equal to x should pass validation"
        );

        // Test with entry_age = None (should pass)
        let params = SingleLifeParams::builder()
            .mt(mt.clone())
            .i(0.04)
            .x(40.0)
            .n(10.0)
            .t(0.0)
            .m(1)
            .moment(1)
            .build();

        let result = params.validate_all();
        assert!(result.is_ok(), "entry_age = None should pass validation");
    }
}
