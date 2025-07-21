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
    // Immediate basic insurance benefits
    A_x, A_x1_n, A_x_n1, A_x_n,

    // Immediate increasing insurance benefits
    IA_x, IA_x1_n, IA_x_n1, IA_x_n,

    // Immediate geometric insurance benefits
    gA_x, gA_x1_n, gA_x_n1, gA_x_n,

    // Due basic insurance benefits (Äₓ notation)
    AA_x, AA_x1_n, AA_x_n1, AA_x_n,

    // Due increasing insurance benefits (IÄₓ notation)
    IAA_x, IAA_x1_n, IAA_x_n1, IAA_x_n,

    // Due geometric insurance benefits (gÄₓ notation)
    gAA_x, gAA_x1_n, gAA_x_n1, gAA_x_n,

    // Deferred immediate basic insurance benefits
    t_A_x, t_A_x1_n, t_A_x_n1, t_A_x_n,

    // Deferred immediate increasing insurance benefits
    t_IA_x, t_IA_x1_n, t_IA_x_n1, t_IA_x_n,

    // Deferred immediate geometric insurance benefits
    t_gA_x, t_gA_x1_n, t_gA_x_n1, t_gA_x_n,

    // Deferred due basic insurance benefits
    t_AA_x, t_AA_x1_n, t_AA_x_n1, t_AA_x_n,

    // Deferred due increasing insurance benefits
    t_IAA_x, t_IAA_x1_n, t_IAA_x_n1, t_IAA_x_n,

    // Deferred due geometric insurance benefits
    t_gAA_x, t_gAA_x1_n, t_gAA_x_n1, t_gAA_x_n,

    // Basic annuities
    aaxn,
    // Deferred annuities
    taax, taaxn,
    // Increasing annuities
    Iaax, Iaaxn,
    // Deferred increasing annuities
    tIaax, tIaaxn,
    // Geometric increasing annuities
    gIaax, gIaaxn,
};

// Fractional calculations (already available)
pub use crate::fractional::{tpx, tqx};

// Select and ultimate calculations
pub use crate::selection::{tpx_, tqx_};

// Result and error types from Polars
pub use polars::prelude::{PolarsError, PolarsResult};

// Most commonly used Polars DataFrame types for working with mortality tables
pub use polars::prelude::{DataFrame, LazyFrame, Series};
