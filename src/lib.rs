//! # RSLife
//!
//! A comprehensive Rust library for actuarial mortality table calculations and life insurance mathematics.
//! Built with type safety, performance optimization, and automatic parameter validation.
//!
//! ## Features
//!
//! - **🚀 High Performance**: Optimized calculations with intelligent memory management
//! - **🔒 Type Safety**: Uses `u32` for age/duration columns (prevents negative values)
//! - **📊 Smart XML Parsing**: Loads mortality data from SOA XTbML standard with automatic `tqx`/`lx` detection
//! - **✅ Parameter Validation**: Automatic cross-field validation prevents calculation errors
//! - **🧮 Comprehensive Coverage**: Life insurance, annuities, and survival functions with standard actuarial notation
//! - **📈 Multiple Assumptions**: UDD, CFM, and HPB methods for fractional age calculations
//! - **🔄 Flexible Data Sources**: SOA XML, custom XLSX files, or programmatically created DataFrames
//!
//! ## Architecture Overview
//!
//! RSLife uses a two-tier configuration system:
//!
//! 1. **`MortTableConfig`**: Configures mortality table settings (data source, radix, percentage adjustments, assumptions)
//! 2. **`ParamConfig`**: Contains all calculation parameters including the mortality table config, interest rates, ages, terms, and validation
//!
//! This design ensures type safety, prevents parameter mismatches, and enables comprehensive validation before calculations
//!
//! ## Quick Start
//!
//! ```rust
//! use rslife::prelude::*;
//!
//! // Load SOA mortality table
//! let xml = MortXML::from_url_id(1704)?;
//! let mt_config = MortTableConfig {
//!     xml,
//!     radix: Some(100_000),
//!     pct: Some(1.0),
//!     assumption: Some(AssumptionEnum::UDD),
//! };
//!
//! let params = ParamConfig {
//!     mt: mt_config,
//!     i: 0.03,
//!     x: 35,
//!     n: None,
//!     t: Some(0),
//!     m: Some(1),
//!     moment: Some(1),
//!     entry_age: None,
//! };
//!
//! // Calculate actuarial values
//!
//! let whole_life = Ax(&params)?;
//! let annuity = aaxn(&params)?;
//! let survival = tpx(&params.mt, 30.0, 5.0, 0.0, None)?;
//!
//! println!("Whole life: {:.6}", whole_life);
//! println!("Annuity: {:.6}", annuity);
//! println!("5yr survival: {:.6}", survival);
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! ## Creating Custom Mortality Tables
//!
//! ```rust
//! use rslife::prelude::*;
//! use polars::prelude::*;
//!
//! // Create mortality table with u32 age columns (type-safe, no negative ages)
//! let df = df! {
//!     "age" => [25u32, 26, 27, 28, 29],
//!     "qx" => [0.001f64, 0.002, 0.003, 0.004, 0.005],
//! }?;
//!
//! let xml = MortXML::from_df(df)?;
//! let mt_config = MortTableConfig {
//!     xml,
//!     radix: Some(100_000),
//!     pct: Some(1.0),
//!     assumption: Some(AssumptionEnum::UDD),
//! };
//!
//! let params = ParamConfig {
//!     mt: mt_config,
//!     i: 0.05,
//!     x: 25,
//!     n: Some(10),
//!     t: None,
//!     m: Some(1),
//!     moment: Some(1),
//!     entry_age: None,
//! };
//!
//! // Custom table is ready for actuarial calculations
//! let term_insurance = Ax1n(&params)?;
//! println!("Custom table has {} rows", params.mt.xml.tables[0].values.height());
//! println!("10-year term insurance at age 25: {:.6}", term_insurance);
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! ## Data Structure
//!
//! - Age and duration columns use `u32` for type safety (prevents negative values)
//! - [`MortXML`] struct: parses and holds mortality table(s) from XML or DataFrame
//! - [`MortTableConfig`] struct: configures mortality table settings (radix, percentage, assumption)
//! - [`ParamConfig`] struct: contains all parameters for actuarial calculations including mortality table config, interest rate, ages, terms, and other calculation parameters
//!
//! ## Actuarial Functions
//!
//! Most functions use `ParamConfig` which contains all required parameters and performs automatic validation.
//! Functions support both regular mortality and select-ultimate tables via the optional `entry_age` parameter.
//!
//! ### Life Insurance Benefits
//! - `Ax(params: &ParamConfig) -> Result<f64>` - Whole life insurance
//! - `Ax1n(params: &ParamConfig) -> Result<f64>` - Term life insurance
//! - `Exn(params: &ParamConfig) -> Result<f64>` - Pure endowment
//! - `Axn(params: &ParamConfig) -> Result<f64>` - Endowment insurance
//!
//! ### Increasing Benefit Insurance
//! - `IAx(params: &ParamConfig) -> Result<f64>` - Increasing whole life insurance
//! - `IAx1n(params: &ParamConfig) -> Result<f64>` - Increasing term life insurance
//! - `IAxn(params: &ParamConfig) -> Result<f64>` - Increasing endowment insurance
//!
//! ### Decreasing Benefit Insurance
//! - `DAx1n(params: &ParamConfig) -> Result<f64>` - Decreasing term life insurance
//! - `DAxn(params: &ParamConfig) -> Result<f64>` - Decreasing endowment insurance
//!
//! ### Geometric Increasing Benefits
//! - `gAx(params: &ParamConfig) -> Result<f64>` - Geometric whole life insurance (requires growth rate)
//! - `gAx1n(params: &ParamConfig) -> Result<f64>` - Geometric term life insurance
//! - `gExn(params: &ParamConfig) -> Result<f64>` - Geometric pure endowment
//! - `gAxn(params: &ParamConfig) -> Result<f64>` - Geometric endowment insurance
//!
//! ### Annuities
//! - `aax(params: &ParamConfig) -> Result<f64>` - Life annuity-due
//! - `aaxn(params: &ParamConfig) -> Result<f64>` - Temporary annuity-due
//!
//! ### Increasing Annuities
//! - `Iaax(params: &ParamConfig) -> Result<f64>` - Increasing life annuity-due
//! - `Iaaxn(params: &ParamConfig) -> Result<f64>` - Increasing temporary annuity-due
//!
//! ### Decreasing Annuities
//! - `Daaxn(params: &ParamConfig) -> Result<f64>` - Decreasing temporary annuity-due
//!
//! ### Geometric Increasing Annuities
//! - `gaax(params: &ParamConfig) -> Result<f64>` - Geometric life annuity-due (requires growth rate)
//! - `gaaxn(params: &ParamConfig) -> Result<f64>` - Geometric temporary annuity-due
//!
//! ### Fractional Age Functions
//! - `tpx(config: &MortTableConfig, x: f64, t: f64, s: f64, entry_age: Option<u32>) -> Result<f64>` - Survival probability
//! - `tqx(config: &MortTableConfig, x: f64, t: f64, s: f64, entry_age: Option<u32>) -> Result<f64>` - Death probability
//!
//! **Notes:**
//! - All functions follow standard actuarial notation and use `ParamConfig` for parameters
//! - `ParamConfig` contains: mortality table config (`mt`), interest rate (`i`), age (`x`), term (`n`), deferral (`t`), payment frequency (`m`), moment, and entry age
//! - Set `entry_age = None` for regular mortality tables, or `Some(age)` for select-ultimate tables
//! - Fractional functions (`tpx`, `tqx`) take `MortTableConfig` directly and accept `f64` for age and time parameters
//! - All other functions use `ParamConfig` and perform automatic parameter validation

pub mod annuities;
pub mod annuities_certain;
pub mod benefits;
pub mod int_rate_convert;
pub mod mt_config;
pub mod param_config;
pub mod prelude;
pub mod survivals;
pub mod xml;
