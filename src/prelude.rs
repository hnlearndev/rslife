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

// Core mortality table types and configuration
pub use crate::mt_config::mt_data::MortData;
pub use crate::mt_config::{AssumptionEnum, MortTableConfig};

// All actuarial calculation functions (implementation functions from whole.rs)
#[rustfmt::skip]
pub use crate::int_rate_convert::*;

pub use crate::annuities_certain::{Daan, Dan, Iaan, Ian, aan, an};

pub use crate::single_life::survivals::{tpx, tqx};

pub use crate::single_life::commutations::{Cx, Dx, Mx, Nx, Rx, Sx};

#[rustfmt::skip]
pub use crate::single_life::benefits::{
    Exn, Axn1, Ax1n, Ax, Axn,
    IAx1n, IAxn, IAx,
    DAx1n, DAxn,
    gAx1n, gAxn, gAx,
};

#[rustfmt::skip]
pub use crate::single_life::annuities::{
    aaxn,aax, Iaaxn, Iaax, Daaxn, gaax, gaaxn,
    axn,ax, Iaxn, Iax, Daxn, gax, gaxn,
};

// Most commonly used Polars types for working with mortality tables
pub use polars::prelude::{DataFrame, LazyFrame, PolarsError, PolarsResult, Series};

// Package Result type for RSLife functions
pub use crate::RSLifeResult;
