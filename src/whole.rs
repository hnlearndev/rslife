//! # Whole Numbers Actuarial Functions (Implementation Module)
//!
//! **Low-level implementation module** providing integer-optimized actuarial calculations.
//! For comprehensive documentation, examples, and user guidance, see [`crate::actuarial`].
//!
//! ## Module Purpose
//!
//! This module contains **optimized implementations** for whole ages and integer time periods.
//! Functions here are typically called through the unified [`crate::actuarial`] interface
//! which provides automatic implementation selection and comprehensive documentation.
//!
//! ## Direct Usage
//!
//! ```rust
//! use rslife::whole;
//! use rslife::mt_config::{MortTableConfig, AssumptionEnum};
//! use rslife::xml::MortXML;
//! // Set up a sample config (requires internet for MortXML::from_url_id)
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let xml = MortXML::from_url_id(1704)?;
//!     let config = MortTableConfig {
//!         xml,
//!         radix: 100_000,
//!         pct: Some(1.0),
//!         int_rate: Some(0.03),
//!         assumption: Some(AssumptionEnum::UDD),
//!     };
//!     let value = whole::Ax(&config, 30)?; // Direct call (integer age only)
//!     Ok(())
//! }
//! ```
//!
//! ## See Also
//!
//! - **[`crate::actuarial`]** - Primary user interface with full documentation
//! - **[`benefits`]** - Insurance benefit implementations
//! - **[`annuities`]** - Annuity implementations

pub mod annuities;
pub mod benefits;
mod helpers;
pub mod survivals;

use crate::helpers::get_value;
use crate::mt_config::MortTableConfig;
use polars::prelude::*;

// Re-export all insurance benefits functions (order matches benefits.rs)
#[rustfmt::skip]
pub use benefits::{
    // Immediate basic insurance benefits
    A_x, A_x1_n, A_x_n1, A_x_n,

    // Immediate increasing insurance benefits
    IA_x, IA_x1_n, IA_x_n1, IA_x_n,

    // Immediate geometric insurance benefits
    gA_x, gA_x1_n, gA_x_n1, gA_x_n,

    // Due basic insurance benefits (Äₓ notation)
    AA_x, AA_x1_n, AA_x_n1, AA_x_n,

    // Due increasing insurance benefits (IÄₓ notation)
    IAA_x, IAA_x1_n, IAA_x_n1, IAA_x_n,

    // Due geometric insurance benefits (gÄₓ notation)
    gAA_x, gAA_x1_n, gAA_x_n1, gAA_x_n,

    // Deferred immediate basic insurance benefits
    t_A_x, t_A_x1_n, t_A_x_n1, t_A_x_n,

    // Deferred immediate increasing insurance benefits
    t_IA_x, t_IA_x1_n, t_IA_x_n1, t_IA_x_n,

    // Deferred immediate geometric insurance benefits
    t_gA_x, t_gA_x1_n, t_gA_x_n1, t_gA_x_n,

    // Deferred due basic insurance benefits
    t_AA_x, t_AA_x1_n, t_AA_x_n1, t_AA_x_n,

    // Deferred due increasing insurance benefits
    t_IAA_x, t_IAA_x1_n, t_IAA_x_n1, t_IAA_x_n,

    // Deferred due geometric insurance benefits
    t_gAA_x, t_gAA_x1_n, t_gAA_x_n1, t_gAA_x_n,
};

// Traditional actuarial function names (for unified interface compatibility)
// These wrapper functions provide the standard actuarial naming convention
// that matches the fractional module, enabling seamless function calls
// from the actuarial unified interface.

/// Whole life insurance: Aₓ = A_x
#[allow(non_snake_case)]
pub fn Ax(config: &MortTableConfig, x: i32) -> PolarsResult<f64> {
    A_x(config, x)
}

/// Term life insurance: A¹ₓ:ₙ̅ = A_x1_n
#[allow(non_snake_case)]
pub fn Axn(config: &MortTableConfig, x: i32, n: i32) -> PolarsResult<f64> {
    A_x1_n(config, x, n)
}

/// Pure endowment: Eₓ:ₙ = A_x_n1
#[allow(non_snake_case)]
pub fn Exn(config: &MortTableConfig, x: i32, n: i32) -> PolarsResult<f64> {
    A_x_n1(config, x, n)
}

/// Endowment insurance: Aₓ:ₙ = A_x_n
#[allow(non_snake_case)]
pub fn AExn(config: &MortTableConfig, x: i32, n: i32) -> PolarsResult<f64> {
    A_x_n(config, x, n)
}

/// Deferred whole life: ₜAₓ = t_A_x (with parameter reordering)
#[allow(non_snake_case)]
pub fn tAx(config: &MortTableConfig, x: i32, t: i32) -> PolarsResult<f64> {
    t_A_x(config, t, x)
}

/// Deferred term: ₜA¹ₓ:ₙ̅ = t_A_x1_n (with parameter reordering)
#[allow(non_snake_case)]
pub fn tAxn(config: &MortTableConfig, x: i32, n: i32, t: i32) -> PolarsResult<f64> {
    t_A_x1_n(config, t, x, n)
}

/// Deferred pure endowment: ₜEₓ:ₙ = t_A_x_n1 (with parameter reordering)
#[allow(non_snake_case)]
pub fn tExn(config: &MortTableConfig, x: i32, n: i32, t: i32) -> PolarsResult<f64> {
    t_A_x_n1(config, t, x, n)
}

/// Deferred endowment: ₜAₓ:ₙ = t_A_x_n (with parameter reordering)
#[allow(non_snake_case)]
pub fn tAExn(config: &MortTableConfig, x: i32, n: i32, t: i32) -> PolarsResult<f64> {
    t_A_x_n(config, t, x, n)
}

/// Increasing whole life: (IA)ₓ = IA_x
#[allow(non_snake_case)]
pub fn IAx(config: &MortTableConfig, x: i32) -> PolarsResult<f64> {
    IA_x(config, x)
}

/// Increasing term: (IA)¹ₓ:ₙ̅ = IA_x1_n
#[allow(non_snake_case)]
pub fn IAxn(config: &MortTableConfig, x: i32, n: i32) -> PolarsResult<f64> {
    IA_x1_n(config, x, n)
}

/// Geometric whole life: Aₓ⁽ᵍ⁾ = gA_x
#[allow(non_snake_case)]
pub fn gAx(config: &MortTableConfig, x: i32, g: f64) -> PolarsResult<f64> {
    gA_x(config, x, g)
}

/// Geometric term: A¹ₓ:ₙ̅⁽ᵍ⁾ = gA_x1_n
#[allow(non_snake_case)]
pub fn gAxn(config: &MortTableConfig, x: i32, n: i32, g: f64) -> PolarsResult<f64> {
    gA_x1_n(config, x, n, g)
}

/// Due whole life: Äₓ = AA_x
#[allow(non_snake_case)]
pub fn AAx(config: &MortTableConfig, x: i32) -> PolarsResult<f64> {
    AA_x(config, x)
}

/// Due term: Ä¹ₓ:ₙ̅ = AA_x1_n
#[allow(non_snake_case)]
pub fn AAxn(config: &MortTableConfig, x: i32, n: i32) -> PolarsResult<f64> {
    AA_x1_n(config, x, n)
}

/// Due increasing whole life: IÄₓ = IAA_x
#[allow(non_snake_case)]
pub fn IAAx(config: &MortTableConfig, x: i32) -> PolarsResult<f64> {
    IAA_x(config, x)
}

// Re-export all annuity functions (order matches annuities.rs)
#[rustfmt::skip]
pub use annuities::{
    // Basic annuities
    aaxn,
    // Deferred annuities
    taax, taaxn,
    // Increasing annuities
    Iaax, Iaaxn,
    // Deferred increasing annuities
    tIaax, tIaaxn,
    // Geometric increasing annuities
    gIaax, gIaaxn,
};
