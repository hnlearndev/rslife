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

    // Immediate decreasing insurance benefits
    DA_x1_n, DA_x_n1, DA_x_n,

    // Immediate geometric insurance benefits
    gA_x, gA_x1_n, gA_x_n1, gA_x_n,

    // Due basic insurance benefits (Äₓ notation)
    AA_x, AA_x1_n, AA_x_n1, AA_x_n,

    // Due increasing insurance benefits (IÄₓ notation)
    IAA_x, IAA_x1_n, IAA_x_n1, IAA_x_n,

    // Due decreasing insurance benefits (DÄₓ notation)
    DAA_x1_n, DAA_x_n1, DAA_x_n,

    // Due geometric insurance benefits (gÄₓ notation)
    gAA_x, gAA_x1_n, gAA_x_n1, gAA_x_n,

    // Deferred immediate basic insurance benefits
    t_A_x, t_A_x1_n, t_A_x_n1, t_A_x_n,

    // Deferred immediate increasing insurance benefits
    t_IA_x, t_IA_x1_n, t_IA_x_n1, t_IA_x_n,

    // Deferred immediate decreasing insurance benefits
    t_DA_x1_n, t_DA_x_n1, t_DA_x_n,

    // Deferred immediate geometric insurance benefits
    t_gA_x, t_gA_x1_n, t_gA_x_n1, t_gA_x_n,

    // Deferred due basic insurance benefits
    t_AA_x, t_AA_x1_n, t_AA_x_n1, t_AA_x_n,

    // Deferred due increasing insurance benefits
    t_IAA_x, t_IAA_x1_n, t_IAA_x_n1, t_IAA_x_n,

    // Deferred due decreasing insurance benefits
    t_DAA_x1_n, t_DAA_x_n1, t_DAA_x_n,

    // Deferred due geometric insurance benefits
    t_gAA_x, t_gAA_x1_n, t_gAA_x_n1, t_gAA_x_n,

    // Due annuities
    aa_x, aa_x_n, Iaa_x, Iaa_x_n, Daa_x_n, gaa_x, gaa_x_n,

    // Immediate annuities
    a_x, a_x_n, Ia_x, Ia_x_n, Da_x_n, ga_x, ga_x_n,

    // Deferred due annuities
    t_aa_x, t_aa_x_n, t_Iaa_x, t_Iaa_x_n, t_Daa_x_n, t_gaa_x, t_gaa_x_n,

    // Deferred immediate annuities
    t_a_x, t_a_x_n, t_Ia_x, t_Ia_x_n, t_Da_x_n, t_ga_x, t_ga_x_n,
};

// Fractional calculations (already available)
pub use crate::fractional::{tpx, tqx};

// Select and ultimate calculations
pub use crate::selection::{
    // Benefits
    A_x_,
    A_x_n_,
    A_x_n1_,
    A_x1_n_,
    AA_x_,
    AA_x_n_,
    AA_x_n1_,
    AA_x1_n_,
    DA_x_n_,
    DA_x_n1_,
    DA_x1_n_,
    DAA_x_n_,
    DAA_x_n1_,
    DAA_x1_n_,
    Da_x_n_,
    Daa_x_n_,
    IA_x_,
    IA_x_n_,
    IA_x_n1_,
    IA_x1_n_,
    IAA_x_,
    IAA_x_n_,
    IAA_x_n1_,
    IAA_x1_n_,
    Ia_x_,
    Ia_x_n_,
    Iaa_x_,
    Iaa_x_n_,
    // Annuities
    a_x_,
    a_x_n_,
    aa_x_,
    aa_x_n_,
    gA_x_,
    gA_x_n_,
    gA_x_n1_,
    gA_x1_n_,
    gAA_x_,
    gAA_x_n_,
    gAA_x_n1_,
    gAA_x1_n_,
    ga_x_,
    ga_x_n_,
    gaa_x_,
    gaa_x_n_,
    t_A_x_,
    t_A_x_n_,
    t_A_x_n1_,
    t_A_x1_n_,
    t_AA_x_,
    t_AA_x_n_,
    t_AA_x_n1_,
    t_AA_x1_n_,
    t_DA_x_n_,
    t_DA_x_n1_,
    t_DA_x1_n_,
    t_DAA_x_n_,
    t_DAA_x_n1_,
    t_DAA_x1_n_,
    t_Da_x_n_,
    t_Daa_x_n_,
    t_IA_x_,
    t_IA_x_n_,
    t_IA_x_n1_,
    t_IA_x1_n_,
    t_IAA_x_,
    t_IAA_x_n_,
    t_IAA_x_n1_,
    t_IAA_x1_n_,
    t_Ia_x_,
    t_Ia_x_n_,
    t_Iaa_x_,
    t_Iaa_x_n_,
    t_a_x_,
    t_a_x_n_,
    t_aa_x_,
    t_aa_x_n_,
    t_gA_x_,
    t_gA_x_n_,
    t_gA_x_n1_,
    t_gA_x1_n_,
    t_gAA_x_,
    t_gAA_x_n_,
    t_gAA_x_n1_,
    t_gAA_x1_n_,
    t_ga_x_,
    t_ga_x_n_,
    t_gaa_x_,
    t_gaa_x_n_,
    tpx_,
    tqx_,
};

// Result and error types from Polars
pub use polars::prelude::{PolarsError, PolarsResult};

// Most commonly used Polars DataFrame types for working with mortality tables
pub use polars::prelude::{DataFrame, LazyFrame, Series};
