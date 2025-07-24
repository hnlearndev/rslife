//! # RSLife
//!
//! A comprehensive Rust library for actuarial mortality table calculations and life insurance mathematics.
//!
//! ## Features
//!
//! - Loads and parses mortality data from XML sources (SOA XTbML standard)
//! - Supports UDD, CFM, and HPB fractional age assumptions
//! - Provides life insurance, annuity, and demographic functions with standard actuarial notation
//!
//! ## Quick Start
//!
//! ```rust
//! use rslife::prelude::*;
//!
//! // Load SOA mortality table
//! let xml = MortXML::from_url_id(1704)?;
//! let config = MortTableConfig {
//!     xml,
//!     radix: Some(100_000),
//!     int_rate: Some(0.03),
//!     pct: Some(1.0),
//!     assumption: Some(AssumptionEnum::UDD)
//! };
//!
//! // Calculate actuarial values
//! 
//! let whole_life = A_x(&config, 35)?;
//! let annuity = aa_x_n(&config, 35, 1, 1)?;
//! let survival = tpx(&config, 5.0, 30.0)?;
//!
//! println!("Whole life: {:.6}", whole_life);
//! println!("Annuity: {:.6}", annuity);
//! println!("5yr survival: {:.6}", survival);
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! ## Data Structure
//!
//! - [`TableValues`] struct: columnar vectors for age, value, and optional duration
//! - [`MortXML`] struct: parses and holds mortality table(s) from XML or DataFrame
//! - [`MortTableConfig`] struct: configures actuarial calculations
//!
//! ## Actuarial Functions
//!
//! All functions require explicit `config` and argument parameters, following the signature conventions: `function(&config, ...)`.
//!
//! ### Life Insurance Benefits
//! - `A_x(&config, x)` - Whole life insurance
//! - `A_x_n(&config, x, n)` - Term life insurance
//! - `A_x_n1(&config, x, n)` - Pure endowment
//! - `A_x1_n(&config, x, n)` - Endowment insurance
//! - `IA_x(&config, x)`, `IA_x_n(&config, x, n)` - Increasing benefit insurance
//! - `gA_x(&config, x)`, `gA_x_n(&config, x, n)` - Geometric increasing insurance
//!
//! ### Deferred Life Insurance
//! - `t_A_x(&config, x, t)` - Deferred whole life insurance
//! - `t_A_x_n(&config, x, t, n)` - Deferred term life insurance
//!
//! ### Annuities
//! - `aa_x_n(&config, x, n)` - Life annuity due
//! - `Iaa_x(&config, x, n)`, `Iaaxn(&config, x, n)` - Increasing annuities
//! - `gaa_x(&config, x, n)`, `gaa_x_n(&config, x, n)` - Geometric increasing annuities
//!
//! ### Deffered annuities
//! - `t_aa_x(&config, x, t)`, `taaxn(&config, x, t, n)` - Deferred annuities-due
//! - `t_Iaa_x(&config, x, t, n)`, `tIaaxn(&config, x, t, n)` - Deferred increasing annuities-due
//!
//! //! ### Fractional Age Functions
//! - `tpx(&config, t, x)` - Survival probability for fractional time/age (uses UDD, CFM, or HPB)
//! - `tqx(&config, t, x)` - Death probability for fractional time/age
//!
//! ### Select and Ultimate Functions
//! - `A_x_n_(&config, entry_age, x, n)`` - Life insurance with select/ultimate
//! - `t_Iaa_x(&config,entry_age, x, t, n)`` - Deferred increasing annuities-due with select/ultimate
//! - `tpx_(&config, entry_age, t, x)` - Survival probability with entry age selection (select/ultimate)
//! - `tqx_(&config, entry_age, t, x)` - Death probability with entry age selection
//!
//! All functions follow standard actuarial notation and support the three mortality assumptions. Fractional and select/ultimate functions require explicit arguments for config, time, age, and (for selection) entry age.

pub mod fractional;
pub mod helpers;
pub mod mt_config;
pub mod prelude;
pub mod selection;
pub mod whole;
pub mod xml;
