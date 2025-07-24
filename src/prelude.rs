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
pub use crate::mt_config::{AssumptionEnum, MortTableConfig};

// XML data loading and parsing
pub use crate::xml::{AxisDef, ContentClassification, MetaData, MortXML, Table};

// All actuarial calculation functions (implementation functions from whole.rs)

#[rustfmt::skip]
pub use crate::whole::{
    // Annuities
    aax, aaxn,
    Iaax, Iaaxn,
    Daaxn,
    gaax, gaaxn,

    // Benefits and Life Insurance
    Ax, Ax1n, nEx, Axn,
    IAx, IAx1n, IAxn,
    DAx1n, DAxn,
    gAx, gAx1n, gnEx, gAxn,

    // Survival Probabilities
    tpx, tqx,
};

// Result and error types from Polars
pub use polars::prelude::{PolarsError, PolarsResult};

// Most commonly used Polars DataFrame types for working with mortality tables
pub use polars::prelude::{DataFrame, LazyFrame, Series};
