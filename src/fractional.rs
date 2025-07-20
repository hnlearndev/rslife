//! # Fractional Age Actuarial Functions (Implementation Module)
//!
//! **Low-level implementation module** providing fractional age and time period calculations.
//! For comprehensive documentation, examples, and user guidance, see [`crate::actuarial`].
//!
//! ## Module Purpose
//!
//! This module contains **specialized implementations** for fractional ages with UDD/CFM/HPB
//! mortality assumptions. Functions here are typically called through the unified
//! [`crate::actuarial`] interface which provides automatic implementation selection.
//!
//! ## Direct Usage
//!
//! ```rust
//! use rslife::fractional;
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
//!     let survival = fractional::tpx(&config, 0.5, 30.5)?; // Direct call (fractional age)
//!     Ok(())
//! }
//! ```
//!
//! ## See Also
//!
//! - **[`crate::actuarial`]** - Primary user interface with full documentation
//! - **[`survivals`]** - Fractional survival and mortality calculations

pub mod annuities;
pub mod benefits;
pub mod survivals;

use crate::helpers::get_value;
use crate::mt_config::{AssumptionEnum, MortTableConfig};
use crate::whole;
use polars::prelude::*;

// Re-export functions from survival module for easier access
pub use survivals::{tpx, tqx};
