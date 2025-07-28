//! # RSLife
//!
//! A comprehensive Rust library for actuarial mortality table calculations and life insurance mathematics.
//! Built with type safety, performance optimization, and automatic parameter validation.
//!
//! ## Features
//!
//! - **🚀 High Performance**: Optimized calculations with intelligent memory management
//! - **🔒 Type Safety**: Uses `f64` for age/duration columns (supports fractional values)
//! - **📊 Smart Data Loading**: Loads mortality data from SOA XTbML standard, XLSX files, or DataFrames with automatic `tqx`/`lx` detection
//! - **✅ Parameter Validation**: Automatic cross-field validation prevents calculation errors
//! - **🧮 Comprehensive Coverage**: Life insurance, annuities, and survival functions with standard actuarial notation
//! - **📈 Multiple Assumptions**: UDD, CFM, and HPB methods for fractional age calculations
//! - **🔄 Flexible Data Sources**: SOA XML, custom XLSX files, or programmatically created DataFrames
//!
//! ## Architecture Overview
//!
//! RSLife provides direct function calls with automatic parameter validation:
//!
//! 1. **`MortTableConfig`**: Configures mortality table settings using builder pattern with `data` field
//! 2. **Direct Function Calls**: All actuarial functions like `Ax()`, `aaxn()`, `tpx()` are called directly with parameters
//! 3. **Internal Validation**: Parameter validation happens automatically within each function
//!
//! This design ensures type safety, prevents parameter mismatches, and provides a clean API for calculations
//!
//! ## Quick Start
//!
//! ```rust
//! use rslife::prelude::*;
//!
//! // Load SOA mortality table using MortData
//! let data = MortData::from_soa_url(1704)?;
//! let mt_config = MortTableConfig::builder()
//!     .data(data)
//!     .radix(100_000)
//!     .pct(1.0)
//!     .assumption(AssumptionEnum::UDD)
//!     .build()?;
//!
//! // Calculate actuarial values using direct function calls
//! let whole_life = Ax()
//!     .mt(&mt_config)
//!     .i(0.03)
//!     .x(35)
//!     .call()?;
//!
//! let annuity = aaxn()
//!     .mt(&mt_config)
//!     .i(0.03)
//!     .x(35)
//!     .n(10)
//!     .call()?;
//!
//! let survival = tpx()
//!     .mt(&mt_config)
//!     .x(30.0)
//!     .t(5.0)
//!     .call()?;
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
//! // Create mortality table with f64 age columns (supports fractional ages)
//! let df = df! {
//!     "age" => [25.0f64, 26.0, 27.0, 28.0, 29.0],
//!     "qx" => [0.001f64, 0.002, 0.003, 0.004, 0.005],
//! }?;
//!
//! let data = MortData::from_dataframe(df)?;
//! let mt_config = MortTableConfig::builder()
//!     .data(data)
//!     .radix(100_000)
//!     .pct(1.0)
//!     .assumption(AssumptionEnum::UDD)
//!     .build()?;
//!
//! // Custom table is ready for actuarial calculations
//! let term_insurance = Ax1n()
//!     .mt(&mt_config)
//!     .i(0.05)
//!     .x(25)
//!     .n(10)
//!     .call()?;
//!
//! println!("Custom table has {} rows", mt_config.data.tables[0].values.height());
//! println!("10-year term insurance at age 25: {:.6}", term_insurance);
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! ## Data Structure
//!
//! - Age and duration columns use `f64` for flexibility (supports fractional values)
//! - [`MortData`] struct: loads and holds mortality table(s) from SOA URL, XLSX files, or DataFrames
//! - [`MortTableConfig`] struct: configures mortality table settings using builder pattern with `data` field
//! - Direct function calls: `Ax()`, `aaxn()`, `tpx()`, etc. with automatic parameter validation
//!
//! ## Actuarial Functions
//!
//! All functions use direct function calls with automatic parameter validation.
//! Functions support both regular mortality and select-ultimate tables via the optional `entry_age` parameter.
//!
//! ### Function Call Pattern
//!
//! Each function uses a builder pattern for parameters, then `.call()` to execute:
//!
//! ```rust
//! let result = Ax()  // function name
//!     .mt(&mt_config)  // mortality table config
//!     .i(0.03)         // interest rate
//!     .x(35)           // age
//!     .t(Some(0))      // optional deferral
//!     .call()?;        // execute with validation
//! ```
//!
//! ### Life Insurance Benefits
//! - `Ax().call()` - Whole life insurance
//! - `Ax1n().call()` - Term life insurance (requires `.n()`)
//! - `Exn().call()` - Pure endowment (requires `.n()`)
//! - `Axn().call()` - Endowment insurance (requires `.n()`)
//!
//! ### Increasing Benefit Insurance
//! - `IAx().call()` - Increasing whole life insurance
//! - `IAx1n().call()` - Increasing term life insurance
//! - `IAxn().call()` - Increasing endowment insurance
//!
//! ### Decreasing Benefit Insurance
//! - `DAx1n().call()` - Decreasing term life insurance
//! - `DAxn().call()` - Decreasing endowment insurance
//!
//! ### Geometric Increasing Benefits
//! - `gAx().call()` - Geometric whole life insurance
//! - `gAx1n().call()` - Geometric term life insurance
//! - `gExn().call()` - Geometric pure endowment
//! - `gAxn().call()` - Geometric endowment insurance
//!
//! ### Annuities
//! - `aax().call()` - Life annuity-due
//! - `aaxn().call()` - Temporary annuity-due (requires `.n()`)
//!
//! ### Increasing Annuities
//! - `Iaax().call()` - Increasing life annuity-due
//! - `Iaaxn().call()` - Increasing temporary annuity-due
//!
//! ### Decreasing Annuities
//! - `Daaxn().call()` - Decreasing temporary annuity-due
//!
//! ### Geometric Increasing Annuities
//! - `gaax().call()` - Geometric life annuity-due
//! - `gaaxn().call()` - Geometric temporary annuity-due
//!
//! ### Survival Functions
//!
//! ```rust
//! let survival = tpx()
//!     .mt(&mt_config)
//!     .x(30.0)  // age (f64 for fractional)
//!     .t(5.0)   // time (f64 for fractional)
//!     .call()?;
//! ```
//!
//! - `tpx().call()` - Survival probability
//! - `tqx().call()` - Death probability
//!
//! **Notes:**
//! - All functions follow standard actuarial notation
//! - Common builder methods: `.mt()`, `.i()`, `.x()`, `.n()`, `.t()`, `.m()`, `.moment()`, `.entry_age()`
//! - Survival functions accept `f64` for age and time parameters for fractional calculations
//! - All calculations include automatic parameter validation before execution

pub mod annuities_certain;
pub mod helpers;
pub mod int_rate_convert;
pub mod mt_config;
pub mod mt_data;
pub mod params;
pub mod prelude;
pub mod single;
pub mod xml;
