// Define module structure
pub mod annuities;
pub mod benefits;
mod helpers;
pub mod survivals;

// Common imports for fractional calculations
use crate::helpers::get_value;
use crate::mt_config::{AssumptionEnum, MortTableConfig};
use crate::whole;
use polars::prelude::*;

// Re-export functions from survival module for easier access
pub use survivals::{tpx, tqx};
