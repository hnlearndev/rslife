//! # Single Life Actuarial Functions
//!
//! This module provides the core actuarial calculations for single-life contracts, including:
//!
//! - Present value and expected value calculations for life insurance and annuities
//! - Commutation functions for efficient actuarial computations
//! - Survival probabilities and related functions
//! - Modular submodules for annuities, benefits, commutations, and survivals
//!
//! ## Submodules
//! - [`annuities`] — Present value and expected value of life annuities (temporary, whole, deferred, etc.)
//! - [`benefits`] — Present value and expected value of life insurance benefits (whole life, term, pure endowment, etc.)
//! - [`commutations`] — Commutation functions (Dx, Nx, Sx, Cx, Mx, Rx) for efficient actuarial calculations
//! - [`survivals`] — Survival probabilities (tpx, tqx, etc.) and related functions
//!
//! ## Usage Example
//! ```rust
//! # use rslife::prelude::*;
//! // Load a mortality table (AM92)
//! let mort_data = MortData::from_ifoa_url_id("AM92")?;
//! let mt = MortTableConfig::builder()
//!     .data(mort_data)
//!     .radix(10_000)
//!     .build()?;
//!
//! // Calculate the present value of a whole life annuity (a_x)
//! let a_x = aax().mt(&mt).i(0.04).x(65).call()?;
//!
//! // Calculate the present value of a term insurance (A_xn)
//! let A_xn = Axn().mt(&mt).i(0.04).x(40).n(20).call()?;
//!
//! // Compute commutation function Dx
//! let D_x = Dx().mt(&mt).i(0.04).x(50).call()?;
//!
//! // Compute survival probability tpx
//! let t_p_x = tpx().mt(&mt).x(30.0).t(10.0).call()?;
//!
//! # RSLifeResult::Ok(())
//! ```
//!
//! ## Notes
//! - Ensure that the mortality table (`mort_table`) is properly defined and passed to the functions as needed.
//! - The interest rate used in the calculations should be in decimal form (e.g., 0.04 for 4%).
//! - This module assumes the use of Rust's standard error handling with `Result` and `Option` types.
//! - For more advanced usage, consider exploring the individual submodules for specialized functions and calculations.

// Module structure
pub mod annuities;
pub mod benefits;
pub mod commutations;
pub mod survivals;
