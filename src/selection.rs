//! # Select and Ultimate Survival Functions (Implementation Module)
//!
//! **Low-level implementation module** providing select and ultimate mortality calculations.
//! For comprehensive documentation, examples, and user guidance, see [`crate::actuarial`].
//!
//! ## Module Purpose
//!
//! This module contains **specialized implementations** for select and ultimate mortality tables
//! where mortality rates depend on both attained age and duration since policy issue.
//! Functions here are typically called through the unified [`crate::actuarial`] interface
//! which provides automatic implementation selection and comprehensive documentation.
//!
//! ## Direct Usage
//!
//! ```rust
//! use rslife::selection;
//! use rslife::mt_config::{MortTableConfig, AssumptionEnum};
//! use rslife::xml::MortXML;
//! // Set up a sample config (requires internet for MortXML::from_url_id)
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let xml = MortXML::from_url_id(3604)?;
//!     let config = MortTableConfig {
//!         xml,
//!         radix: Some(100_000),
//!         pct: Some(1.0),
//!         int_rate: None,
//!         assumption: None,
//!     };
//!     let survival = selection::tpx_(&config, 5, 30, 26)?; // Direct call with entry age
//!     Ok(())
//! }
//! ```
//!
//! ## See Also
//!
//! - **[`crate::actuarial`]** - Primary user interface with full documentation
//! - **[`survivals`]** - Select and ultimate survival calculations

// Define module structure
pub mod annuities;
pub mod benefits;
mod helpers;
pub mod survivals;

// Common imports for fractional calculations
use crate::fractional;
use crate::mt_config::MortTableConfig;
use polars::prelude::*;

// Re-export functions from survival module for easier access
pub use survivals::{tpx_, tqx_};
