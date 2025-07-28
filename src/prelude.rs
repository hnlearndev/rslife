//! # RSLife Prelude
//!
//! Conveniently re-exports the most common types and functions for actuarial calculations.
//! Import this module to access all primary RSLife features with a single `use` statement.
//!
//! ## Example
//!
//! ```rust
//! use rslife::prelude::*;
//! // Now you can use MortTableConfig, MortData, Ax, Axn, aaxn, tpx, etc.
//! ```

// Actuarial calculation functions
pub use crate::single::annuities::*;
pub use crate::single::benefits::*;
pub use crate::single::survivals::*;

// Interest rate conversion functions
pub use crate::int_rate_convert::*;

// Certain annuities
pub use crate::annuities_certain::*;

// Core mortality table types and configuration
pub use crate::mt_config::*;

// Parameter structs are internal - not exposed to users

// Mortality data types
pub use crate::mt_data::MortData;

// Most commonly used Polars types for working with mortality tables
pub use polars::prelude::{DataFrame, LazyFrame, PolarsError, PolarsResult, Series};
