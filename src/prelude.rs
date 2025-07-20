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

// All actuarial calculation functions (unified interface with automatic selection)
#[rustfmt::skip]
pub use crate::actuarial::{
    // Insurance Benefits (automatically optimized)
    Ax, Axn, Exn, AExn,
    tAx, tAxn, tExn, tAExn,
    IAx, IAxn,
    gAx, gAxn,
    // Annuities (automatically optimized)
    aaxn,
    taax, taaxn,
    Iaax, Iaaxn,
    tIaax, tIaaxn,
    gIaax, gIaaxn,
    // Fractional calculations (already available)
    tpx, tqx,
};

// Result and error types from Polars
pub use polars::prelude::{PolarsError, PolarsResult};

// Most commonly used Polars DataFrame types for working with mortality tables
pub use polars::prelude::{DataFrame, LazyFrame, Series};
