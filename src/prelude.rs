//! # RSLife Prelude
//!
//! The prelude module re-exports the most commonly used types and functions
//! from the rslife crate, allowing users to import everything they need
//! with a single `use` statement.
//!
//! ## Usage
//!
//! ```rust
//! use rslife::prelude::*;
//!
//! // The prelude gives you access to all the main types:
//! // - MortXML for loading mortality data
//! // - MortTableConfig for configuring mortality tables  
//! // - AssumptionEnum for mortality assumptions
//! // - All actuarial functions like Ax, Axn, axn_due, etc.
//! ```

// Core mortality table types and configuration
pub use crate::actuarial::mort_tbl_config::{AssumptionEnum, MortTableConfig};

// XML data loading and parsing
pub use crate::xml::{AxisDef, ContentClassification, MetaData, MortXML, Table};

// All actuarial calculation functions from wholes module
pub use crate::actuarial::wholes::{
    // Annuities due
    axn_due,
    // Geometric increasing benefits
    gAx,
    gAxn,

    gIax_due,
    gIaxn_due,
    tAExn,

    // Deferred benefits
    tAx,
    tAxn,
    tExn,
    tIax_due,
    tIaxn_due,
    tax_due,
    taxn_due,
    AExn,

    // Basic insurance benefits
    Ax,
    Axn,
    Exn,
    // Increasing benefits
    IAx,
    IAxn,

    Iax_due,
    Iaxn_due,
};

// Fractional age calculation functions
pub use crate::actuarial::fractionals::{conditional_tqx, tpx, tqx};

// Result and error types from Polars
pub use polars::prelude::{PolarsError, PolarsResult};

// Most commonly used Polars DataFrame types for working with mortality tables
pub use polars::prelude::{DataFrame, LazyFrame, Series};
