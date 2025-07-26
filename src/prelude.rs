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
pub use crate::int_rate_convert::{
    // From i to i
    nom_i_to_eff_i,
    eff_i_to_nom_i,
    // From d to d,
    eff_d_to_nom_d,
    nom_d_to_eff_d,
    // From i to d
    eff_i_to_eff_d,
    eff_i_to_nom_d,
    nom_i_to_eff_d,
    nom_i_to_nom_d,
    // From d to i
    eff_d_to_eff_i,
    eff_d_to_nom_i,
    nom_d_to_eff_i,
    nom_d_to_nom_i,
};

#[rustfmt::skip]
pub use crate::annuities::{
    // Certain Annuities
    an, aan,
    // Annuities
    aax, aaxn,
    Iaax, Iaaxn,
    Daaxn,
    gaax, gaaxn,
};

#[rustfmt::skip]
pub use crate::benefits::{
    // Benefits and Life Insurance
    Ax, Ax1n, nEx, Axn,
    IAx, IAx1n, IAxn,
    DAx1n, DAxn,
    gAx, gAx1n, gnEx, gAxn,
};

pub use crate::survivals::{tpx, tqx};

// Most commonly used Polars types for working with mortality tables
pub use polars::prelude::{DataFrame, LazyFrame, PolarsError, PolarsResult, Series};
