pub mod cfm;
pub mod hbp;
pub mod helpers;
pub mod udd;

use crate::actuarial::mort_tbl_config::{AssumptionEnum, MortTableConfig};
use polars::prelude::*;

// If 'helpers' is a sibling module, use the correct relative path:
use crate::actuarial::fractionals::helpers::{is_whole_number, qx};

pub fn tpx(config: &MortTableConfig, t: f64, x: f64) -> PolarsResult<f64> {
    if let Some(assumption) = &config.assumption {
        match assumption {
            AssumptionEnum::UDD => udd::tpx_udd(config, t, x),
            AssumptionEnum::CFM => cfm::tpx_cfm(config, t, x),
            AssumptionEnum::HPB => hbp::tpx_hpb(config, t, x),
        }
    } else {
        // Error handling if assumption is not set
        Err(PolarsError::ComputeError("Assumption not set".into()))
    }
}

pub fn tqx(config: &MortTableConfig, t: f64, x: f64) -> PolarsResult<f64> {
    if let Some(assumption) = &config.assumption {
        match assumption {
            AssumptionEnum::UDD => udd::tqx_udd(config, t, x),
            AssumptionEnum::CFM => cfm::tqx_cfm(config, t, x),
            AssumptionEnum::HPB => hbp::tqx_hpb(config, t, x),
        }
    } else {
        // Error handling if assumption is not set
        Err(PolarsError::ComputeError("Assumption not set".into()))
    }
}

pub fn conditional_tqx(config: &MortTableConfig, t: f64, x: f64, s: f64) -> PolarsResult<f64> {
    if let Some(assumption) = &config.assumption {
        match assumption {
            AssumptionEnum::UDD => udd::conditional_tqx_udd(config, t, x, s),
            AssumptionEnum::CFM => cfm::conditional_tqx_cfm(config, t, x, s),
            AssumptionEnum::HPB => hbp::conditional_tqx_hpb(config, t, x, s),
        }
    } else {
        // Error handling if assumption is not set
        Err(PolarsError::ComputeError("Assumption not set".into()))
    }
}
