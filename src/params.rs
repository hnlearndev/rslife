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
    #[garde(range(min = 0.0, max = 150.0))]
    pub x: f64,

    // Term - x + t cannot exceed max age from mort_table
    // t can be 0 for some actuarial calculations
    #[garde(range(min = 0.0, max = 100.0))]
    pub t: Option<f64>,

    // Deferral period
    #[garde(range(min = 0.0, max = 100.0))]
    pub k: Option<f64>,

    // Entry age for select-ultimate tables,
    // Entry age cannot exceed age x
    #[garde(range(min = 0, max = 150))]
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
        let min_age = self.mt.min_age();
        let max_age = self.mt.max_age();
        let x = self.x;

        // Check boundary for x (age) vs mort table max min
        if x < min_age || x > max_age {
            let mut report = garde::Report::new();
            // Provide a path for the error. If unsure, use an empty string.
            report.append(
                garde::Path::new(""),
                garde::Error::new(format!(
                    "age {x} must be between {min_age} and {max_age} from mortality table"
                )),
            );
            return Err(report);
        }

        // x + t + k cannot exceed max age of mort table
        let t = self.t.unwrap_or(1.0); // This is actuarial convention, t defaults to 1 if not specified
        let k = self.k.unwrap_or(0.0);
        if x + t + k > max_age {
            let mut report = garde::Report::new();
            // Provide a path for the error. If unsure, use an empty string.
            report.append(garde::Path::new(""), garde::Error::new(format!(
                "age + deferral + term ({x} + {t} + {k}) cannot exceed max age {max_age} from mortality table"
            )));
            return Err(report);
        }

        // entry_age cannot exceed age x
        if let Some(entry_age) = self.entry_age {
            if (entry_age as f64) > x {
                let mut report = garde::Report::new();
                // Provide a path for the error. If unsure, use an empty string.
                report.append(
                    garde::Path::new(""),
                    garde::Error::new(format!("entry_age {entry_age} cannot exceed age {x}")),
                );
                return Err(report);
            }

            // entry age cannot be less than mortable min
            if (entry_age as f64) < min_age {
                let mut report = garde::Report::new();
                // Provide a path for the error. If unsure, use an empty string.
                report.append(garde::Path::new(""), garde::Error::new(format!(
                    "entry_age {entry_age} cannot be less than min age {min_age} from mortality table"
                )));
                return Err(report);
            }
        }

        Ok(())
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
    pub n: Option<u32>,

    // Deferral period
    #[garde(range(max = 150))]
    pub t: Option<u32>,

    // Payable m-thly
    // If not None must be greater than 0
    #[garde(range(min = 1))]
    pub m: Option<u32>,

    // Mathematical moment, 1 is the first moment (mean), 2 is the second moment (variance)
    // If not None, must be greater than 0
    #[garde(range(min = 1))]
    pub moment: Option<u32>,

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
        let min_age = self.mt.min_age() as u32;
        let max_age = self.mt.max_age() as u32;
        let x = self.x;

        // Check boundary for x (age) vs mort table max min
        if x < min_age || x > max_age {
            let mut report = garde::Report::new();
            // Provide a path for the error. If unsure, use an empty string.
            report.append(
                garde::Path::new(""),
                garde::Error::new(format!(
                    "age {x} must be between {min_age} and {max_age} from mortality table"
                )),
            );
            return Err(report);
        }

        // x + t + n cannot exceed max age of mort table
        let t = self.t.unwrap_or(0);
        let n = self.n.unwrap_or(0);
        if x + t + n > max_age {
            let mut report = garde::Report::new();
            // Provide a path for the error. If unsure, use an empty string.
            report.append(garde::Path::new(""), garde::Error::new(format!(
                "age + deferral + term ({x} + {t} + {n}) cannot exceed max age {max_age} from mortality table"
            )));
            return Err(report);
        }

        // entry_age cannot exceed age x
        if let Some(entry_age) = self.entry_age {
            if entry_age > x {
                let mut report = garde::Report::new();
                // Provide a path for the error. If unsure, use an empty string.
                report.append(
                    garde::Path::new(""),
                    garde::Error::new(format!("entry_age {entry_age} cannot exceed age {x}")),
                );
                return Err(report);
            }

            // entry age cannot be less than mortable min
            if entry_age < min_age {
                let mut report = garde::Report::new();
                // Provide a path for the error. If unsure, use an empty string.
                report.append(garde::Path::new(""), garde::Error::new(format!(
                "entry_age {entry_age} cannot be less than min age {min_age} from mortality table"
            )));
                return Err(report);
            }
        }

        Ok(())
    }
}
