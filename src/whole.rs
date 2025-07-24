#![allow(non_snake_case)]

// Define module structure
pub mod annuities;
pub mod benefits;
pub mod survivals;

// Common imports for calculations
use crate::helpers::{
    get_new_config_geometric_functions, get_new_config_with_selected_table, get_value,
};
use crate::mt_config::MortTableConfig;
use polars::prelude::*;

// Re-export functions for easier access
pub use annuities::*;
pub use benefits::*;
pub use survivals::*;
