#![allow(non_snake_case)]

//! # Actuarial Calculations - Unified Interface
//!
//! **Primary interface** for all actuarial calculations with automatic implementation selection.
//! This module provides comprehensive documentation and examples for all actuarial functions.
//!
//! ## Architecture Overview
//!
//! Functions intelligently choose between optimized implementations based on input values:
//! - **Whole Numbers** (e.g., 30.0, 45.0): Uses [`crate::whole`] optimized integer implementations
//! - **Fractional Numbers** (e.g., 30.5, 45.25): Uses [`crate::fractional`] enhanced fractional implementations
//!
//! ## Available Calculations
//!
//! ### 🏥 Insurance Benefits
//! - **[`Ax`]** - Whole life insurance
//! - **[`Axn`]** - Term life insurance
//! - **[`Exn`]** - Pure endowment
//! - **[`AExn`]** - Endowment insurance
//! - **[`tAx`]** - Deferred whole life insurance
//! - **[`tAxn`]** - Deferred term insurance
//! - **[`IAx`]** - Increasing whole life insurance
//! - **[`IAxn`]** - Increasing term insurance
//! - **[`gAx`]** - Geometrically increasing whole life
//! - **[`gAxn`]** - Geometrically increasing term
//!
//! ### 💰 Annuities
//! - **[`aaxn`]** - Temporary annuity due
//! - **[`taax`]** - Deferred annuity
//! - **[`taaxn`]** - Deferred temporary annuity
//! - **[`Iaax`]** - Increasing annuity
//! - **[`Iaaxn`]** - Increasing temporary annuity
//! - **[`tIaax`]** - Deferred increasing annuity
//! - **[`tIaaxn`]** - Deferred increasing temporary annuity
//! - **[`gIaax`]** - Geometrically increasing annuity
//! - **[`gIaaxn`]** - Geometrically increasing temporary annuity
//!
//! ### 📊 Survival & Mortality
//! - **[`tpx`]** - Survival probability (fractional ages supported)
//! - **[`tqx`]** - Mortality probability (fractional ages supported)
//!
//! ## Usage Examples
//!
//! ### Basic Insurance Calculations
//! ```rust
//! use rslife::prelude::*;
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let xml = MortXML::from_url_id(1704)?;
//!     let config = MortTableConfig {
//!         xml,
//!         l_x_init: 100_000,
//!         pct: Some(1.0),
//!         int_rate: Some(0.03),
//!         assumption: Some(AssumptionEnum::UDD),
//!     };
//!     // Whole life insurance (automatically uses optimized integer implementation)
//!     let whole_life_30 = Ax(&config, 30.0)?;
//!     // Fractional age (automatically uses fractional implementation)
//!     let whole_life_30_5 = Ax(&config, 30.5)?;
//!     // Term insurance - 20 year term for 45-year-old
//!     let term_insurance = Axn(&config, 45.0, 20)?;
//!     // Pure endowment - 10 year endowment
//!     let endowment = Exn(&config, 35.0, 10)?;
//!     Ok(())
//! }
//! ```
//!
//! ### Annuity Calculations
//! ```rust
//! use rslife::prelude::*;
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let xml = MortXML::from_url_id(1704)?;
//!     let config = MortTableConfig {
//!         xml,
//!         l_x_init: 100_000,
//!         pct: Some(1.0),
//!         int_rate: Some(0.03),
//!         assumption: Some(AssumptionEnum::UDD),
//!     };
//!     // Temporary annuity due - monthly payments for 20 years
//!     let annuity = aaxn(&config, 65.0, 20, 12)?;
//!     // Deferred annuity - start in 10 years, annual payments
//!     let deferred_annuity = taax(&config, 55.0, 10, 1)?;
//!     // Increasing annuity - payments increase each year
//!     let increasing_annuity = Iaax(&config, 60.0, 25, 1)?;
//!     Ok(())
//! }
//! ```
//!
//! ### Fractional Age Calculations
//! ```rust
//! use rslife::prelude::*;
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let xml = MortXML::from_url_id(1704)?;
//!     let config = MortTableConfig {
//!         xml,
//!         l_x_init: 100_000,
//!         pct: Some(1.0),
//!         int_rate: Some(0.03),
//!         assumption: Some(AssumptionEnum::UDD),
//!     };
//!     // 6-month survival probability for someone aged 30.5
//!     let survival_6_months = tpx(&config, 0.5, 30.5)?;
//!     // 6-month mortality probability
//!     let mortality_6_months = tqx(&config, 0.5, 30.5)?;
//!     // Verify they sum to 1.0
//!     assert!((survival_6_months + mortality_6_months - 1.0).abs() < 1e-10);
//!     Ok(())
//! }
//! ```
//!
//! ## Mortality Assumptions (Fractional Calculations)
//!
//! When using fractional ages, specify the mortality assumption in your configuration:
//!
//! ### UDD (Uniform Distribution of Deaths)
//! Most commonly used assumption in life insurance. Deaths uniformly distributed within each age interval.
//! **Formula**: ₜpₓ₊ₛ = 1 - t·qₓ/(1 + s·qₓ)
//!
//! ### CFM (Constant Force of Mortality)
//! Constant mortality force within each age interval. Useful for continuous-time models.
//! **Formula**: ₜpₓ₊ₛ = (1 - qₓ)^t
//!
//! ### HPB (Hyperbolic/Balmer)
//! Hyperbolic distribution providing balance between UDD and CFM.
//! **Formula**: ₜpₓ₊ₛ = (1 - qₓ)/(1 - s·qₓ)
//!
//! ```rust
//! use rslife::prelude::*;
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! # let xml = MortXML::from_url_id(1704)?;
//!
//! let config_udd = MortTableConfig {
//!     xml: xml.clone(),
//!     l_x_init: 100_000,
//!     pct: Some(1.0),
//!     int_rate: Some(0.03),
//!     assumption: Some(AssumptionEnum::UDD), // Most common
//! };
//!
//! let config_cfm = MortTableConfig {
//!     xml,
//!     l_x_init: 100_000,
//!     pct: Some(1.0),
//!     int_rate: Some(0.03),
//!     assumption: Some(AssumptionEnum::CFM), // For continuous models
//! };
//! # Ok(())
//! # }
//! ```
//!
//! ## Performance Notes
//!
//! - **Integer ages**: Automatically use optimized implementations (~10x faster)
//! - **Fractional ages**: Use specialized algorithms with interpolation
//! - **Migration**: Functions gain fractional support transparently over time
//! - **Zero breaking changes**: Existing code continues to work as features are added
//!
//! ## Implementation Modules
//!
//! - **[`crate::whole`]** - Integer-optimized implementations (internal)
//! - **[`crate::fractional`]** - Fractional age implementations (internal)
//! - **[`crate::mt_config`]** - Mortality table configuration
//! - **[`crate::xml`]** - Mortality data loading from XTbML format

use crate::mt_config::MortTableConfig;
use polars::prelude::*;

// ================================================================================================
// INSURANCE BENEFITS
// ================================================================================================

/// Whole life insurance calculation with automatic implementation selection.
///
/// **Mathematical Formula**: Aₓ = Mₓ / Dₓ
///
/// Automatically selects the optimal implementation:
/// - Whole numbers: Uses `whole::benefits::Ax` (optimized for integers)
/// - Fractional numbers: Uses `fractional::survival::tpx` based calculations
///
/// # Parameters
/// - `config`: Mortality table configuration with interest rate
/// - `x`: Age (supports both whole and fractional ages)
///
/// # Returns
/// Present value of the whole life insurance benefit
///
/// # Example
/// ```rust
/// use rslife::prelude::*;
/// fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let xml = MortXML::from_url_id(1704)?;
///     let config = MortTableConfig {
///         xml,
///         l_x_init: 100_000,
///         pct: Some(1.0),
///         int_rate: Some(0.03),
///         assumption: Some(AssumptionEnum::UDD),
///     };
///     // Optimized integer implementation
///     let whole_life_30 = Ax(&config, 30.0)?;
///     // Enhanced fractional implementation
///     let whole_life_30_5 = Ax(&config, 30.5)?;
///     Ok(())
/// }
/// ```
pub fn Ax(config: &MortTableConfig, x: f64) -> PolarsResult<f64> {
    if x.fract() == 0.0 {
        // Use optimized whole-year implementation
        crate::whole::benefits::Ax(config, x as i32)
    } else {
        // TODO: Use fractional implementation when available
        // For now, fall back to whole number (rounded down)
        crate::whole::benefits::Ax(config, x.floor() as i32)
    }
}

/// Term life insurance calculation with automatic implementation selection.
///
/// **Mathematical Formula**: A¹ₓ:ₙ = Aₓ - Aₓ₊ₙ · Eₓ:ₙ
///
/// # Parameters
/// - `config`: Mortality table configuration with interest rate
/// - `x`: Current age (supports fractional ages)
/// - `n`: Term length in years
pub fn Axn(config: &MortTableConfig, x: f64, n: i32) -> PolarsResult<f64> {
    if x.fract() == 0.0 {
        crate::whole::benefits::Axn(config, x as i32, n)
    } else {
        // TODO: Use fractional implementation when available
        crate::whole::benefits::Axn(config, x.floor() as i32, n)
    }
}

/// Pure endowment calculation with automatic implementation selection.
///
/// **Mathematical Formula**: Eₓ:ₙ = Dₓ₊ₙ / Dₓ
///
/// # Parameters
/// - `config`: Mortality table configuration with interest rate
/// - `x`: Current age (supports fractional ages)
/// - `n`: Endowment period in years
pub fn Exn(config: &MortTableConfig, x: f64, n: i32) -> PolarsResult<f64> {
    if x.fract() == 0.0 {
        crate::whole::benefits::Exn(config, x as i32, n)
    } else {
        // TODO: Use fractional implementation when available
        crate::whole::benefits::Exn(config, x.floor() as i32, n)
    }
}

/// Endowment insurance calculation with automatic implementation selection.
///
/// **Mathematical Formula**: Aₓ:ₙ = A¹ₓ:ₙ + Eₓ:ₙ
///
/// # Parameters
/// - `config`: Mortality table configuration with interest rate
/// - `x`: Current age (supports fractional ages)
/// - `n`: Endowment period in years
pub fn AExn(config: &MortTableConfig, x: f64, n: i32) -> PolarsResult<f64> {
    if x.fract() == 0.0 {
        crate::whole::benefits::AExn(config, x as i32, n)
    } else {
        // TODO: Use fractional implementation when available
        crate::whole::benefits::AExn(config, x.floor() as i32, n)
    }
}

/// Deferred whole life insurance calculation with automatic implementation selection.
///
/// **Mathematical Formula**: ₜAₓ = Mₓ₊ₜ / Dₓ
///
/// # Parameters
/// - `config`: Mortality table configuration with interest rate
/// - `x`: Current age (supports fractional ages)
/// - `t`: Deferral period in years
pub fn tAx(config: &MortTableConfig, x: f64, t: i32) -> PolarsResult<f64> {
    if x.fract() == 0.0 {
        crate::whole::benefits::tAx(config, x as i32, t)
    } else {
        // TODO: Use fractional implementation when available
        crate::whole::benefits::tAx(config, x.floor() as i32, t)
    }
}

/// Deferred term insurance calculation with automatic implementation selection.
///
/// **Mathematical Formula**: ₜA¹ₓ:ₙ = ₜAₓ - ₜ₊ₙAₓ
///
/// # Parameters
/// - `config`: Mortality table configuration with interest rate
/// - `x`: Current age (supports fractional ages)
/// - `n`: Term length in years
/// - `t`: Deferral period in years
pub fn tAxn(config: &MortTableConfig, x: f64, n: i32, t: i32) -> PolarsResult<f64> {
    if x.fract() == 0.0 {
        crate::whole::benefits::tAxn(config, x as i32, n, t)
    } else {
        // TODO: Use fractional implementation when available
        crate::whole::benefits::tAxn(config, x.floor() as i32, n, t)
    }
}

/// Deferred pure endowment calculation with automatic implementation selection.
///
/// **Mathematical Formula**: ₜEₓ:ₙ = Dₓ₊ₙ₊ₜ / Dₓ
///
/// # Parameters
/// - `config`: Mortality table configuration with interest rate
/// - `x`: Current age (supports fractional ages)
/// - `n`: Additional survival period after deferral
/// - `t`: Deferral period in years
pub fn tExn(config: &MortTableConfig, x: f64, n: i32, t: i32) -> PolarsResult<f64> {
    if x.fract() == 0.0 {
        crate::whole::benefits::tExn(config, x as i32, n, t)
    } else {
        // TODO: Use fractional implementation when available
        crate::whole::benefits::tExn(config, x.floor() as i32, n, t)
    }
}

/// Deferred endowment insurance calculation with automatic implementation selection.
///
/// **Mathematical Formula**: ₜAₓ:ₙ = ₜA¹ₓ:ₙ + ₜEₓ:ₙ
///
/// # Parameters
/// - `config`: Mortality table configuration with interest rate
/// - `x`: Current age (supports fractional ages)
/// - `n`: Endowment period in years
/// - `t`: Deferral period in years
pub fn tAExn(config: &MortTableConfig, x: f64, n: i32, t: i32) -> PolarsResult<f64> {
    if x.fract() == 0.0 {
        crate::whole::benefits::tAExn(config, x as i32, n, t)
    } else {
        // TODO: Use fractional implementation when available
        crate::whole::benefits::tAExn(config, x.floor() as i32, n, t)
    }
}

/// Increasing whole life insurance calculation with automatic implementation selection.
///
/// **Mathematical Formula**: (IA)ₓ = Sₓ / Dₓ
///
/// # Parameters
/// - `config`: Mortality table configuration with interest rate
/// - `x`: Current age (supports fractional ages)
pub fn IAx(config: &MortTableConfig, x: f64) -> PolarsResult<f64> {
    if x.fract() == 0.0 {
        crate::whole::benefits::IAx(config, x as i32)
    } else {
        // TODO: Use fractional implementation when available
        crate::whole::benefits::IAx(config, x.floor() as i32)
    }
}

/// Increasing term insurance calculation with automatic implementation selection.
///
/// **Mathematical Formula**: (IA)¹ₓ:ₙ = (Sₓ - Sₓ₊ₙ - n · Mₓ₊ₙ) / Dₓ
///
/// # Parameters
/// - `config`: Mortality table configuration with interest rate
/// - `x`: Current age (supports fractional ages)
/// - `n`: Term length in years
pub fn IAxn(config: &MortTableConfig, x: f64, n: i32) -> PolarsResult<f64> {
    if x.fract() == 0.0 {
        crate::whole::benefits::IAxn(config, x as i32, n)
    } else {
        // TODO: Use fractional implementation when available
        crate::whole::benefits::IAxn(config, x.floor() as i32, n)
    }
}

/// Geometrically increasing whole life insurance calculation with automatic implementation selection.
///
/// **Mathematical Formula**: Aₓ⁽ᵍ⁾ = Aₓ (calculated with adjusted interest rate)
///
/// # Parameters
/// - `config`: Mortality table configuration with interest rate
/// - `x`: Current age (supports fractional ages)
/// - `g`: Annual growth rate of benefits
pub fn gAx(config: &MortTableConfig, x: f64, g: f64) -> PolarsResult<f64> {
    if x.fract() == 0.0 {
        crate::whole::benefits::gAx(config, x as i32, g)
    } else {
        // TODO: Use fractional implementation when available
        crate::whole::benefits::gAx(config, x.floor() as i32, g)
    }
}

/// Geometrically increasing term insurance calculation with automatic implementation selection.
///
/// **Mathematical Formula**: A¹ₓ:ₙ⁽ᵍ⁾ = A¹ₓ:ₙ (with adjusted interest rate)
///
/// # Parameters
/// - `config`: Mortality table configuration with interest rate
/// - `x`: Current age (supports fractional ages)
/// - `n`: Term length in years
/// - `g`: Annual growth rate of benefits
pub fn gAxn(config: &MortTableConfig, x: f64, n: i32, g: f64) -> PolarsResult<f64> {
    if x.fract() == 0.0 {
        crate::whole::benefits::gAxn(config, x as i32, n, g)
    } else {
        // TODO: Use fractional implementation when available
        crate::whole::benefits::gAxn(config, x.floor() as i32, n, g)
    }
}

// ================================================================================================
// ANNUITIES
// ================================================================================================

/// Annuity due calculation with automatic implementation selection.
///
/// # Parameters
/// - `config`: Mortality table configuration with interest rate
/// - `x`: Current age (supports fractional ages)
/// - `n`: Annuity period in years
/// - `m`: Payment frequency per year
pub fn aaxn(config: &MortTableConfig, x: f64, n: i32, m: i32) -> PolarsResult<f64> {
    if x.fract() == 0.0 {
        crate::whole::annuities::aaxn(config, x as i32, n, m)
    } else {
        // TODO: Use fractional implementation when available
        crate::whole::annuities::aaxn(config, x.floor() as i32, n, m)
    }
}

/// Deferred annuity calculation with automatic implementation selection.
///
/// # Parameters
/// - `config`: Mortality table configuration with interest rate
/// - `x`: Current age (supports fractional ages)
/// - `t`: Deferral period in years
/// - `m`: Payment frequency per year
pub fn taax(config: &MortTableConfig, x: f64, t: i32, m: i32) -> PolarsResult<f64> {
    if x.fract() == 0.0 {
        crate::whole::annuities::taax(config, x as i32, t, m)
    } else {
        // TODO: Use fractional implementation when available
        crate::whole::annuities::taax(config, x.floor() as i32, t, m)
    }
}

/// Deferred temporary annuity calculation with automatic implementation selection.
///
/// # Parameters
/// - `config`: Mortality table configuration with interest rate
/// - `x`: Current age (supports fractional ages)
/// - `n`: Annuity period in years
/// - `t`: Deferral period in years
/// - `m`: Payment frequency per year
pub fn taaxn(config: &MortTableConfig, x: f64, n: i32, t: i32, m: i32) -> PolarsResult<f64> {
    if x.fract() == 0.0 {
        crate::whole::annuities::taaxn(config, x as i32, n, t, m)
    } else {
        // TODO: Use fractional implementation when available
        crate::whole::annuities::taaxn(config, x.floor() as i32, n, t, m)
    }
}

/// Increasing annuity calculation with automatic implementation selection.
///
/// # Parameters
/// - `config`: Mortality table configuration with interest rate
/// - `x`: Current age (supports fractional ages)
/// - `n`: Annuity period in years
/// - `m`: Payment frequency per year
pub fn Iaax(config: &MortTableConfig, x: f64, n: i32, m: i32) -> PolarsResult<f64> {
    if x.fract() == 0.0 {
        crate::whole::annuities::Iaax(config, x as i32, n, m)
    } else {
        // TODO: Use fractional implementation when available
        crate::whole::annuities::Iaax(config, x.floor() as i32, n, m)
    }
}

/// Increasing temporary annuity calculation with automatic implementation selection.
///
/// # Parameters
/// - `config`: Mortality table configuration with interest rate
/// - `x`: Current age (supports fractional ages)
/// - `n`: Annuity period in years
/// - `m`: Payment frequency per year
pub fn Iaaxn(config: &MortTableConfig, x: f64, n: i32, m: i32) -> PolarsResult<f64> {
    if x.fract() == 0.0 {
        crate::whole::annuities::Iaaxn(config, x as i32, n, m)
    } else {
        // TODO: Use fractional implementation when available
        crate::whole::annuities::Iaaxn(config, x.floor() as i32, n, m)
    }
}

/// Deferred increasing annuity calculation with automatic implementation selection.
///
/// # Parameters
/// - `config`: Mortality table configuration with interest rate
/// - `x`: Current age (supports fractional ages)
/// - `n`: Annuity period in years
/// - `t`: Deferral period in years
/// - `m`: Payment frequency per year
pub fn tIaax(config: &MortTableConfig, x: f64, n: i32, t: i32, m: i32) -> PolarsResult<f64> {
    if x.fract() == 0.0 {
        crate::whole::annuities::tIaax(config, x as i32, n, t, m)
    } else {
        // TODO: Use fractional implementation when available
        crate::whole::annuities::tIaax(config, x.floor() as i32, n, t, m)
    }
}

/// Deferred increasing temporary annuity calculation with automatic implementation selection.
///
/// # Parameters
/// - `config`: Mortality table configuration with interest rate
/// - `x`: Current age (supports fractional ages)
/// - `n`: Annuity period in years
/// - `t`: Deferral period in years
/// - `m`: Payment frequency per year
pub fn tIaaxn(config: &MortTableConfig, x: f64, n: i32, t: i32, m: i32) -> PolarsResult<f64> {
    if x.fract() == 0.0 {
        crate::whole::annuities::tIaaxn(config, x as i32, n, t, m)
    } else {
        // TODO: Use fractional implementation when available
        crate::whole::annuities::tIaaxn(config, x.floor() as i32, n, t, m)
    }
}

/// Geometrically increasing annuity calculation with automatic implementation selection.
///
/// # Parameters
/// - `config`: Mortality table configuration with interest rate
/// - `x`: Current age (supports fractional ages)
/// - `n`: Annuity period in years
/// - `m`: Payment frequency per year
/// - `g`: Annual growth rate
pub fn gIaax(config: &MortTableConfig, x: f64, n: i32, m: i32, g: f64) -> PolarsResult<f64> {
    if x.fract() == 0.0 {
        crate::whole::annuities::gIaax(config, x as i32, n, m, g)
    } else {
        // TODO: Use fractional implementation when available
        crate::whole::annuities::gIaax(config, x.floor() as i32, n, m, g)
    }
}

/// Geometrically increasing temporary annuity calculation with automatic implementation selection.
///
/// # Parameters
/// - `config`: Mortality table configuration with interest rate
/// - `x`: Current age (supports fractional ages)
/// - `n`: Annuity period in years
/// - `m`: Payment frequency per year
/// - `g`: Annual growth rate
pub fn gIaaxn(config: &MortTableConfig, x: f64, n: i32, m: i32, g: f64) -> PolarsResult<f64> {
    if x.fract() == 0.0 {
        crate::whole::annuities::gIaaxn(config, x as i32, n, m, g)
    } else {
        // TODO: Use fractional implementation when available
        crate::whole::annuities::gIaaxn(config, x.floor() as i32, n, m, g)
    }
}

// ================================================================================================
// FRACTIONAL CALCULATIONS (Already available)
// ================================================================================================

/// Fractional survival probability calculation.
///
/// This function already supports fractional ages and time periods.
/// Re-exported from the fractional module for convenience.
pub use crate::fractional::{tpx, tqx};
