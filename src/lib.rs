//! # RSLife
//!
//! A high-performance, type-safe Rust library for actuarial mortality table calculations and life insurance mathematics.
//!
//! ## Features
//! - **Fast & Safe**: Optimized for speed and reliability
//! - **Flexible Data**: Load directly from SOA or IFOA mortality and morbidity database, Speadsheets (ODS/XLSX), or DataFrames
//! - **Comprehensive Coverage**: Life insurance, annuities, survival functions,commutatons and more
//! - **Fractional Ages**: Supports both integer and fractional ages/durations
//! - **Multiple Assumptions**: Uniform Distribution of  Death (UDD), Constant Force of Mortality (CFM), Hyperbolic (HPB) for fractional age calculations
//! - **Builder Pattern**: All functions use builder pattern with automatic parameter validation
//!
//! ## Quick Start
//!
//! ```rust
//! use rslife::prelude::*;
//!
//! // Load AM92 mortality data from IFOA databse
//! let data = MortData::from_ifoa_url_id("AM92")?;
//!
//! // Default every other parameters: 10,000 radix, 100% mortality, UDD assumption
//! let mt_config = MortTableConfig::builder().data(data).build().unwrap();
//!
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
//!     .x(35.0)
//!     .t(5.0)
//!     .call()?;
//!
//! println!("Whole life: {:.6}", whole_life);
//! println!("Annuity: {:.6}", annuity);
//! println!("5yr survival: {:.6}", survival);
//! # RSLifeResult::Ok(())
//! ```
//!
//!
//! ## Supported Functions
//!
//! - **Life Insurance**: `Ax`, `Ax1n`, `Axn`, `Exn`
//! - **Increasing/Decreasing/Geometric Insurance**: `IAx`, `IAx1n`, `IAxn`, `DAx1n`, `DAxn`, `gAx`, `gAx1n`, `gExn`, `gAxn`
//! - **Annuities**: `aax`, `aaxn`, `Iaax`, `Iaaxn`, `Daaxn`, `gaax`, `gaaxn`
//! - **Survival Functions**: `tpx`, `tqx` (fractional ages supported)
//! - **Commutation Functions**: `Cx`,`Dx`,`Mx`,`Nx`,`Sx`,`Rx`
//! - **Annuities Certain**: `an`, `aan`
//! - **Interest Rate Conversions**: between nominial/effective interest rates and discount factors
//!
//!
//! ## Notes
//! - All functions follow standard actuarial notation
//! - Common builder methods: `.mt()`, `.i()`, `.x()`, `.n()`, `.t()`, `.m()`, `.moment()`, `.entry_age()`
//! - Survival functions accept `f64` for age and time parameters to fractional value calculation
//! - All calculations include automatic parameter validation

pub type RSLifeResult<T> = Result<T, Box<dyn std::error::Error>>;
pub mod annuities_certain;
pub mod helpers;
pub mod int_rate_convert;
pub mod mt_config;
pub mod params;
pub mod prelude;
pub mod single_life;
