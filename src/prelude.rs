//! # RSLife Prelude
//!
//! Conveniently re-exports the most common types and functions for actuarial calculations.
//! Import this module to access all primary RSLife features with a single `use` statement.
//!
//! ## Example
//!
//! ```rust
//! # use rslife::prelude::*;
//! use polars::prelude::*;
//! let df = df! {
//!     "age" => [30.0, 31.0],
//!     "qx" => [0.001, 0.002]
//! }?;
//! let data = MortData::from_df(df)?;
//! let mt_config = MortTableConfig::builder()
//!     .data(data)
//!     .radix(100_000)
//!     .pct(1.0)
//!     .assumption(AssumptionEnum::UDD)
//!     .build()
//!     .unwrap();
//! let result = Ax()
//!     .mt(&mt_config)
//!     .i(0.03)
//!     .x(30)
//!     .call()?;
//! println!("Whole life: {:.6}", result);
//! # RSLifeResult::Ok(())
//! ```

// Package Result type for RSLife functions
pub use crate::RSLifeResult;

// Interest rate conversion functions
pub use crate::int_rate_convert::*;

// Certain annuities
pub use crate::annuities_certain::*;

// Actuarial calculation functions
pub use crate::single_life::annuities::*;
pub use crate::single_life::benefits::*;
pub use crate::single_life::commutations::*;
pub use crate::single_life::survivals::*;

// Core mortality table types and configuration
pub use crate::mt_config::{AssumptionEnum, MortTableConfig};

// Mortality data type
pub use crate::mt_config::mt_data::MortData;

// Most commonly used Polars types for working with mortality tables
pub use polars::prelude::{DataFrame, LazyFrame, PolarsError, PolarsResult, Series};
