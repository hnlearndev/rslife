use crate::mt_config::MortTableConfig;
use bon::Builder;
use garde::Validate;

// =======================================
// SURVICAL FUNCTION PARAMETER STRUCT
// =======================================
#[derive(Debug, Clone, Validate, Builder)]
#[garde(allow_unvalidated)]
pub struct SurvivalFunctionParams {
    // Mortality table configuration
    // Validate when ParamConfig is validated
    #[garde(dive)]
    pub mt: MortTableConfig,

    // Age - Cannot exceed min and max from mort_table
    // Basic range validation (0-150 years old is reasonable)
    #[garde(range(max = 150.0))]
    pub x: f64,

    // Term - x + t cannot exceed max age from mort_table
    // t can be 0 for some actuarial calculations
    #[garde(range(max = 150.0))]
    pub t: f64,

    // Deferral period
    #[garde(range(max = 150.0))]
    pub k: f64,

    // Entry age for select-ultimate tables,
    // Entry age cannot exceed age x
    #[garde(range(max = 150))]
    pub entry_age: Option<u32>,
}

impl SurvivalFunctionParams {
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

        // Validate term constraints (custom for SurvivalFunctionParams)
        let x = self.x;
        let t = self.t;
        let k = self.k;
        if x + t + k > max_age {
            errors.push(("", format!(
                "age + deferral + term ({x} + {t} + {k}) cannot exceed max age {max_age} from mortality table"
            )));
        }

        // Validate entry age constraints
        validate_entry_age(self.entry_age, self.x, min_age, &mut errors);

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

// =======================================
// SINGLE LIFE PARAMETER STRUCT
// =======================================
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
    #[garde(range(max = 150))]
    pub x: u32,

    // Term - x + n cannot exceed max age from mort_table
    // n can be 0 for some actuarial calculations
    #[garde(range(max = 150))]
    pub n: u32,

    // Deferral period
    #[garde(range(max = 150))]
    pub t: u32,

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
    #[garde(range(max = 150))]
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
        validate_age_boundaries(self.x as f64, min_age, max_age, &mut errors);

        // Validate term constraints (custom for SingleLifeParams)
        let x = self.x as f64;
        let t = self.t as f64;
        let n = self.n as f64;
        if x + t + n > max_age {
            errors.push(("", format!(
                "age + deferral + term ({x} + {t} + {n}) cannot exceed max age {max_age} from mortality table"
            )));
        }

        // Validate entry age constraints
        validate_entry_age(self.entry_age, x, min_age, &mut errors);

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

// =======================================
// CUSTOM GET VALUE FUNCTION VALIDATION STRUCT
// =======================================
#[derive(Debug, Clone, Validate, Builder)]
#[garde(allow_unvalidated)]
pub struct GetValueFunctionValidation {
    // Mortality table configuration
    // Validate when ParamConfig is validated
    #[garde(dive)]
    pub mt: MortTableConfig,

    // Age - Cannot exceed min and max from mort_table
    // Basic range validation (0-150 years old is reasonable)
    #[garde(range(max = 150))]
    pub x: u32,

    // Entry age for select-ultimate tables,
    // Entry age cannot exceed age x
    #[garde(range(max = 150))]
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
        validate_age_boundaries(self.x as f64, min_age, max_age, &mut errors);

        let x = self.x as f64;

        // Validate entry age constraints
        validate_entry_age(self.entry_age, x, min_age, &mut errors);

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

// =======================================
// PRIVATE FUNCTIONS
// =======================================

type ErrorVec = Vec<(&'static str, String)>;

fn collect_age_bounds_errors(mt: &MortTableConfig, errors: &mut ErrorVec) -> Option<(f64, f64)> {
    let min_age_result = mt.min_age();
    let max_age_result = mt.max_age();
    match (&min_age_result, &max_age_result) {
        (Ok(min), Ok(max)) => Some((*min as f64, *max as f64)),
        _ => {
            if min_age_result.is_err() {
                errors.push(("mt", "Failed to get min_age from mortality table".into()));
            }
            if max_age_result.is_err() {
                errors.push(("mt", "Failed to get max_age from mortality table".into()));
            }
            None
        }
    }
}

fn validate_age_boundaries(x: f64, min_age: f64, max_age: f64, errors: &mut ErrorVec) {
    if x < min_age || x > max_age {
        errors.push((
            "x",
            format!("age {x} must be between {min_age} and {max_age} from mortality table"),
        ));
    }
}

fn validate_entry_age(entry_age: Option<u32>, x: f64, min_age: f64, errors: &mut ErrorVec) {
    if let Some(entry_age) = entry_age {
        if (entry_age as f64) > x {
            errors.push((
                "entry_age",
                format!("entry_age {entry_age} cannot exceed age {x}"),
            ));
        }

        if (entry_age as f64) < min_age {
            errors.push(("entry_age", format!(
                "entry_age {entry_age} cannot be less than min age {min_age} from mortality table"
            )));
        }
    }
}

// =======================================
// UNIT TESTS
// =======================================
