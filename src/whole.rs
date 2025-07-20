//! # Whole Numbers Actuarial Functions (Implementation Module)
//!
//! **Low-level implementation module** providing integer-optimized actuarial calculations.
//! For comprehensive documentation, examples, and user guidance, see [`crate::actuarial`].
//!
//! ## Module Purpose
//!
//! This module contains **optimized implementations** for whole ages and integer time periods.
//! Functions here are typically called through the unified [`crate::actuarial`] interface
//! which provides automatic implementation selection and comprehensive documentation.
//!
//! ## Direct Usage
//!
//! ```rust
//! use rslife::whole;
//! use rslife::mt_config::{MortTableConfig, AssumptionEnum};
//! use rslife::xml::MortXML;
//! // Set up a sample config (requires internet for MortXML::from_url_id)
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let xml = MortXML::from_url_id(1704)?;
//!     let config = MortTableConfig {
//!         xml,
//!         l_x_init: 100_000,
//!         pct: Some(1.0),
//!         int_rate: Some(0.03),
//!         assumption: Some(AssumptionEnum::UDD),
//!     };
//!     let value = whole::Ax(&config, 30)?; // Direct call (integer age only)
//!     Ok(())
//! }
//! ```
//!
//! ## See Also
//!
//! - **[`crate::actuarial`]** - Primary user interface with full documentation
//! - **[`benefits`]** - Insurance benefit implementations
//! - **[`annuities`]** - Annuity implementations

pub mod annuities;
pub mod benefits;
pub mod survivals;

use crate::helpers::{get_new_config, get_value};
use crate::mt_config::MortTableConfig;
use polars::prelude::*;

// Re-export all insurance benefits functions (order matches benefits.rs)
#[rustfmt::skip]
pub use benefits::{
    // Basic insurance benefits
    Ax, Axn, Exn, AExn,
    // Deferred insurance benefits
    tAx, tAxn, tExn, tAExn,
    // Increasing benefits
    IAx, IAxn,
    // Geometric increasing benefits
    gAx, gAxn,
};

// Re-export all annuity functions (order matches annuities.rs)
#[rustfmt::skip]
pub use annuities::{
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
