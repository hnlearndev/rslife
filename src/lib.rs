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
//! //     radix: 100_000,
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
//! - `Ax` - Whole life insurance
//! - `Axn` - Term life insurance
//! - `AExn` - Endowment insurance
//! - `Exn` - Pure endowment
//! - `IAx`, `IAxn` - Increasing benefit insurance
//! - `gAx`, `gAxn` - Geometric increasing insurance
//!
//! ### Deferred Life Insurance
//! - `tAx` - Deferred whole life insurance
//! - `tAxn` - Deferred term life insurance
//! - `tExn` - Deferred pure endowment
//! - `tAExn` - Deferred endowment insurance
//!
//! ### Annuities
//! - `aaxn` - Life annuity due
//! - `taax`, `taaxn` - Deferred annuities
//! - `Iaax`, `Iaaxn` - Increasing annuities
//! - `tIaax`, `tIaaxn` - Deferred increasing annuities
//! - `gIaax`, `gIaaxn` - Geometric increasing annuities
//!
//! ### Fractional Age Functions
//! - `tpx` - Survival probability for fractional time
//! - `tqx` - Death probability for fractional time
//!
//! ### Select and Ultimate Functions
//! - `tpx_` - Survival probability with entry age selection
//! - `tqx_` - Death probability with entry age selection
//!
//! All functions follow standard actuarial notation and support the three mortality assumptions.

pub mod fractional;
pub mod helpers;
pub mod mt_config;
pub mod prelude;
pub mod selection;
pub mod whole;
pub mod xml;
