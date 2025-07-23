//! # RSLife
//!
//! A comprehensive Rust library for actuarial mortality table calculations and life insurance mathematics.
//!
//! This crate provides functionality for:
//! - Loading and parsing mortality data from XML sources
//! - Generating mortality tables with various assumption methods (UDD, CFM, HPB)
//! - Calculating life insurance and annuity values using commutation functions
//! - Working with demographic and actuarial functions
//!
//! ## Quick Start
//!
//! The easiest way to get started is to use the prelude module which re-exports
//! all the commonly used types and functions:
//!
//! ```rust
//! use rslife::prelude::*;
//!
//! // Load mortality data from XML (using a test URL - replace with actual data)
//! // let mort_xml = MortXML::from_url("https://mort.soa.org/data/t1704.xml")?;
//!
//! // Configure mortality table with your XML data
//! // let config = MortTableConfig {
//! //     xml: mort_xml,
//! //     radix: Some(100_000),
//! //     pct: Some(1.0),
//! //     int_rate: Some(0.03),
//! //     assumption: Some(AssumptionEnum::UDD),
//! // };
//!
//! // Calculate whole life insurance for age 35
//! // let ax_35 = Ax(&config, 35)?;
//! ```
//!
//! ## Module Structure
//!
//! - [`xml`] - XML parsing for mortality data from SOA and other sources
//! - [`actuarial`] - Core actuarial calculations including life insurance and annuity functions
//! - [`prelude`] - Common imports for easy use
//!
//! ## Available Mortality Assumptions
//!
//! The library supports three standard mortality assumptions for fractional age calculations:
//!
//! - **UDD (Uniform Distribution of Deaths)**: Linear interpolation of mortality rates
//! - **CFM (Constant Force of Mortality)**: Exponential survival model
//! - **HPB (Hyperbolic/Balmer)**: Hyperbolic interpolation method
//!
//! Each assumption affects how survival probabilities are calculated for fractional ages
//! and time periods, following standard actuarial practice.
//!
//! ## Actuarial Functions
//!
//! The library provides comprehensive actuarial functions including:
//!
//! ### Life Insurance Benefits
//! - `Ax(config, x)` - Whole life insurance
//! - `Axn(config, x, n)` - Term life insurance
//! - `AExn(config, x, n)` - Endowment insurance
//! - `Exn(config, x, n)` - Pure endowment
//! - `IAx(config, x)`, `IAxn(config, x, n)` - Increasing benefit insurance
//! - `gAx(config, x)`, `gAxn(config, x, n)` - Geometric increasing insurance
//!
//! ### Deferred Life Insurance
//! - `tAx(config, x, t)` - Deferred whole life insurance
//! - `tAxn(config, x, t, n)` - Deferred term life insurance
//! - `tExn(config, x, t, n)` - Deferred pure endowment
//! - `tAExn(config, x, t, n)` - Deferred endowment insurance
//!
//! ### Annuities
//! - `aaxn(config, x, n)` - Life annuity due
//! - `taax(config, x, t)`, `taaxn(config, x, t, n)` - Deferred annuities
//! - `Iaax(config, x, n)`, `Iaaxn(config, x, n)` - Increasing annuities
//! - `tIaax(config, x, t, n)`, `tIaaxn(config, x, t, n)` - Deferred increasing annuities
//! - `gIaax(config, x, n)`, `gIaaxn(config, x, n)` - Geometric increasing annuities
//!
//! **Note:** All functions require explicit `config` and argument parameters, following the signature conventions: `function(config, ...)`.
//!
//! ### Fractional Age Functions
//! - `tpx(config, t, x)` - Survival probability for fractional time/age (uses UDD, CFM, or HPB)
//! - `tqx(config, t, x)` - Death probability for fractional time/age
//!
//! ### Select and Ultimate Functions
//! - `tpx_(config, entry_age, t, x)` - Survival probability with entry age selection (select/ultimate)
//! - `tqx_(config, entry_age, t, x)` - Death probability with entry age selection
//!
//! All functions follow standard actuarial notation and support the three mortality assumptions. Fractional and select/ultimate functions require explicit arguments for config, time, age, and (for selection) entry age.

pub mod fractional;
pub mod helpers;
pub mod mt_config;
pub mod prelude;
pub mod selection;
pub mod whole;
pub mod xml;
