//! # RSLife
//!
//! A comprehensive Rust library for actuarial mortality table calculations and life insurance mathematics.
//!
//! ## Features
//!
//! - **Performance Optimized**: 4-level detail system automatically optimizes calculations for 3x faster performance
//! - **Intelligent XML Parsing**: Loads mortality data from SOA XTbML standard with automatic `qx`/`lx` detection
//! - **Type-Safe Data**: Uses `u32` for age/duration columns (prevents negative values)
//! - **Multiple Mortality Assumptions**: UDD, CFM, and HPB methods for fractional age calculations
//! - **Comprehensive Functions**: Life insurance, annuity, and demographic calculations with standard actuarial notation
//! - **Smart Table Detection**: Automatically detects appropriate SOA mortality table usage for calculations
//!
//! ### Performance Optimization Details
//!
//! RSLife automatically optimizes performance with a **4-level detail system**:
//!
//! - **Level 1** (~3x faster): Demographics only - `age`, `qx`, `px`, `lx`, `dx`
//! - **Level 2** (standard): Level 1 + basic commutation - `Cx`, `Dx`
//! - **Level 3** (extended): Level 2 + additional commutation - `Mx`, `Nx`, `Px`
//! - **Level 4** (complete): Level 3 + additional - `Rx`, `Sx`
//!
//! Functions automatically select the minimum required level for optimal performance.
//!
//! ### Automatic Table Configuration
//!
//! - **Single Table Support**: Only XML files with exactly 1 table are supported
//! - **Automatic Data Detection**: Automatically detects whether `qx` or `lx` is provided and generates complete mortality table as needed
//! - **Smart Function Selection**: Selection functions automatically detect whether the appropriate SOA mortality table is used for calculation
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
//! let whole_life = Ax(&config, 35, 0, None)?;
//! let annuity = aaxn(&config, 35, 1, 1, 0, None)?;
//! let survival = tpx(&config, 30.0, 5.0, None)?;
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
//!     "age" => [25u32, 26u32, 27u32, 28u32, 29u32],
//!     "qx" => [0.001, 0.002, 0.003, 0.004, 0.005],
//! }?;
//!
//! let xml = MortXML::from_df(df)?;
//! let config = MortTableConfig {
//!     xml,
//!     radix: Some(100_000),
//!     int_rate: Some(0.03),
//!     pct: Some(1.0),
//!     assumption: Some(AssumptionEnum::UDD)
//! };
//!
//! // Use with actuarial calculations
//! let mortality_table = config.gen_mort_table(1)?;
//! println!("Table rows: {}", mortality_table.height());
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! ## Data Structure
//!
//! - Age and duration columns use `u32` for type safety (prevents negative values)
//! - [`MortXML`] struct: parses and holds mortality table(s) from XML or DataFrame
//! - [`MortTableConfig`] struct: configures actuarial calculations
//!
//! ## Actuarial Functions
//!
//! All functions require explicit `config` and argument parameters. Functions support both regular mortality and select-ultimate tables via the optional `entry_age` parameter.
//!
//! ### Life Insurance Benefits
//! - `Ax(config, x, t, entry_age)` - Whole life insurance (immediate or deferred by t years)
//! - `Ax1n(config, x, n, t, entry_age)` - Term life insurance (n-year term, deferred by t years)
//! - `nEx(config, x, n, t, entry_age)` - Pure endowment (deferred by t years)
//! - `Axn(config, x, n, t, entry_age)` - Endowment insurance (n-year endowment, deferred by t years)
//!
//! ### Increasing Benefit Insurance
//! - `IAx(config, x, t, entry_age)` - Increasing whole life (deferred by t years)
//! - `IAx1n(config, x, n, t, entry_age)` - Increasing term life (n-year term, deferred by t years)
//! - `IAxn(config, x, n, t, entry_age)` - Increasing endowment insurance
//!
//! ### Decreasing Benefit Insurance
//! - `DAx1n(config, x, n, t, entry_age)` - Decreasing term life (n-year term, deferred by t years)
//! - `DAxn(config, x, n, t, entry_age)` - Decreasing endowment insurance
//!
//! ### Geometric Increasing Benefits
//! - `gAx(config, x, g, t, entry_age)` - Geometric whole life (growth rate g, deferred by t years)
//! - `gAx1n(config, x, n, g, t, entry_age)` - Geometric term life
//! - `gnEx(config, x, n, g, t, entry_age)` - Geometric pure endowment
//! - `gAxn(config, x, n, g, t, entry_age)` - Geometric endowment insurance
//!
//! ### Annuities
//! - `aax(config, x, m, t, entry_age)` - Life annuity-due (m payments per year, deferred by t years)
//! - `aaxn(config, x, n, m, t, entry_age)` - Temporary annuity-due (n years, m payments per year)
//!
//! ### Increasing Annuities
//! - `Iaax(config, x, m, t, entry_age)` - Increasing life annuity-due
//! - `Iaaxn(config, x, n, m, t, entry_age)` - Increasing temporary annuity-due
//!
//! ### Decreasing Annuities
//! - `Daaxn(config, x, n, m, t, entry_age)` - Decreasing temporary annuity-due
//!
//! ### Geometric Increasing Annuities
//! - `gaax(config, x, m, g, t, entry_age)` - Geometric life annuity-due (growth rate g)
//! - `gaaxn(config, x, n, m, g, t, entry_age)` - Geometric temporary annuity-due
//!
//! ### Fractional Age Functions
//! - `tpx(config, x, t, entry_age)` - Survival probability for fractional time/age (uses UDD, CFM, or HPB)
//! - `tqx(config, x, t, entry_age)` - Death probability for fractional time/age
//!
//! **Notes:**
//! - All functions follow standard actuarial notation
//! - `x` = age, `n` = term length, `t` = deferral period, `m` = payment frequency, `g` = growth rate
//! - Set `entry_age = None` for regular mortality tables, or `Some(age)` for select-ultimate tables
//! - Fractional functions accept `f64` for `x` and `t`; whole functions use `u32`

pub mod fractional;
pub mod helpers;
pub mod int_rate_convert;
pub mod mt_config;
pub mod prelude;
pub mod whole;
pub mod xml;
