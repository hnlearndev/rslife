//! # RSLife Prelude
//!
//! Conveniently re-exports the most common types and functions for actuarial calculations.
//! Import this module to access all primary RSLife features with a single `use` statement.
//!
//! ## Example
//!
//! ```rust
//! use rslife::prelude::*;
//! // Now you can use MortTableConfig, MortXML, Ax, Axn, aaxn, tpx, etc.
//! ```

// Macros
pub use crate::mddf;

// Core mortality table types and configuration
pub use crate::mt_config::mt_data::MortData;
pub use crate::mt_config::{AssumptionEnum, MortTableConfig};

// All actuarial calculation functions (implementation functions from whole.rs)
pub use crate::int_rate_convert::*;

pub use crate::annuities_certain::{aan, an, Daan, Dan, Iaan, Ian};

pub use crate::single_life::survivals::{dx, lx, tpx, tqx};

pub use crate::single_life::commutations::{Cx, Dx, Mx, Nx, Rx, Sx};

pub use crate::single_life::benefits::{
    gAx, gAx1n, gAxn, Ax, Ax1n, Axn, Axn1, DAx1n, DAxn, Exn, IAx, IAx1n, IAxn,
};

pub use crate::single_life::annuities::{
    aax, aaxn, ax, axn, gaax, gaaxn, gax, gaxn, Daaxn, Daxn, Iaax, Iaaxn, Iax, Iaxn,
};

// Most commonly used Polars types for working with mortality tables
pub use polars::prelude::{DataFrame, LazyFrame, PolarsError, PolarsResult, Series};

// Package Result type for RSLife functions
pub use crate::RSLifeResult;
