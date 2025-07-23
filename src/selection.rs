// Define module structure
pub mod annuities;
pub mod benefits;
mod helpers;
pub mod survivals;

// Common imports for fractional calculations
use crate::fractional;
use crate::mt_config::MortTableConfig;
use crate::whole;
use polars::prelude::*;

// Re-export functions from survival module for easier access
pub use survivals::{tpx_, tqx_};

// === Selection Annuities (all _-suffixed) ===
// [Basic] [Increasing] [Decreasing] [Geometric] [Deferred Immediate] [Deferred Due]
#[rustfmt::skip]
pub use annuities::{
    // --- Deferred Due annuities ---
    deferred_due::{t_Daa_x_n_, t_Iaa_x_, t_Iaa_x_n_, t_aa_x_, t_aa_x_n_, t_gaa_x_, t_gaa_x_n_},
    // --- Deferred Immediate annuities ---
    deferred_immediate::{t_Da_x_n_, t_Ia_x_, t_Ia_x_n_, t_a_x_, t_a_x_n_, t_ga_x_, t_ga_x_n_},
    // --- Due annuities ---
    due::{Daa_x_n_, Iaa_x_, Iaa_x_n_, aa_x_, aa_x_n_, gaa_x_, gaa_x_n_},
    // --- Immediate annuities ---
    immediate::{Da_x_n_, Ia_x_, Ia_x_n_, a_x_, a_x_n_, ga_x_, ga_x_n_},
};

// === Selection Benefits (all _-suffixed) ===
// [Basic] [Increasing] [Decreasing] [Geometric] [Deferred Immediate] [Deferred Due]
#[rustfmt::skip]
pub use benefits::{
    // --- Deferred Due benefits ---
    deferred_due::{
        t_AA_x_, t_AA_x_n_, t_AA_x_n1_, t_AA_x1_n_, t_DAA_x_n_, t_DAA_x_n1_, t_DAA_x1_n_, t_IAA_x_,
        t_IAA_x_n_, t_IAA_x_n1_, t_IAA_x1_n_, t_gAA_x_, t_gAA_x_n_, t_gAA_x_n1_, t_gAA_x1_n_,
    },
    // --- Deferred Immediate benefits ---
    deferred_immediate::{
        t_A_x_, t_A_x_n_, t_A_x_n1_, t_A_x1_n_, t_DA_x_n_, t_DA_x_n1_, t_DA_x1_n_, t_IA_x_,
        t_IA_x_n_, t_IA_x_n1_, t_IA_x1_n_, t_gA_x_, t_gA_x_n_, t_gA_x_n1_, t_gA_x1_n_,
    },
    // --- Due benefits ---
    due::{
        AA_x_, AA_x_n_, AA_x_n1_, AA_x1_n_, DAA_x_n_, DAA_x_n1_, DAA_x1_n_, IAA_x_, IAA_x_n_,
        IAA_x_n1_, IAA_x1_n_, gAA_x_, gAA_x_n_, gAA_x_n1_, gAA_x1_n_,
    },
    // --- Immediate benefits ---
    immediate::{
        A_x_, A_x_n_, A_x_n1_, A_x1_n_, DA_x_n_, DA_x_n1_, DA_x1_n_, IA_x_, IA_x_n_, IA_x_n1_,
        IA_x1_n_, gA_x_, gA_x_n_, gA_x_n1_, gA_x1_n_,
    },
};
