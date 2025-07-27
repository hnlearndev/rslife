use crate::mt_config::MortTableConfig;
use bon::Builder;
use garde::Validate;
use polars::prelude::*;

#[derive(Debug, Clone, Validate, Builder)]
#[garde(allow_unvalidated)]
pub struct ParamConfig {
    // Mortality table configuration
    // Validate when ParamConfig is validated
    #[garde(dive)]
    pub mt: MortTableConfig,

    // Interest rate for actuarial calculations
    pub i: f64,

    // Age - Cannot exceed min and max from mort_table
    pub x: u32,

    // Term - x + n cannot exceed max age from mort_table
    // n can be 0 for some actuarial calculations
    pub n: Option<u32>,

    // Deferral period
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
    pub entry_age: Option<u32>,
}

impl Default for ParamConfig {
    fn default() -> Self {
        ParamConfig {
            mt: MortTableConfig::default(),
            i: 0.0,          // Default interest rate
            x: 0,            // Default age
            n: None,         // Default term length
            t: None,         // Default deferral period
            m: Some(1),      // Default payment frequency as Annual
            moment: Some(1), // Default moment
            entry_age: None, // Default entry age
        }
    }
}

impl ParamConfig {
    /// Validate with cross-field validation
    pub fn validate_all(&self) -> PolarsResult<()> {
        // First run garde's built-in validations
        if let Err(e) = self.validate() {
            return Err(PolarsError::ComputeError(
                format!("Validation error: {}", e.to_string()).into(),
            ));
        }

        let min_age = self.mt.min_age();
        let max_age = self.mt.max_age();

        // Check boundary for x (age) vs mort table max min
        if self.x < min_age || self.x > max_age {
            return Err(PolarsError::ComputeError(
                format!(
                    "age {} must be between {} and {} from mortality table",
                    self.x, min_age, max_age
                )
                .into(),
            ));
        }

        // x+n cannot exceed max age of mort table
        if let Some(n) = self.n {
            if self.x + n > max_age {
                return Err(PolarsError::ComputeError(
                    format!(
                        "age + term ({} + {}) cannot exceed max age {} from mortality table",
                        self.x, n, max_age
                    )
                    .into(),
                ));
            }
        }

        // entry_age cannot exceed age x
        if let Some(entry_age) = self.entry_age {
            if entry_age > self.x {
                return Err(PolarsError::ComputeError(
                    format!("entry_age {} cannot exceed age {}", entry_age, self.x).into(),
                ));
            }

            // entry age cannot be less than mortable min
            if entry_age < min_age {
                return Err(PolarsError::ComputeError(
                    format!(
                        "entry_age {} cannot be less than min age {} from mortality table",
                        entry_age, min_age
                    )
                    .into(),
                ));
            }
        }

        Ok(())
    }
}
