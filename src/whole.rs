// Define module structure
pub mod annuities;
pub mod benefits;
mod helpers;
pub mod survivals;

// Common imports for calculations
use crate::helpers::get_value;
use crate::mt_config::MortTableConfig;
use polars::prelude::*;

// Re-export all insurance benefits functions (order matches benefits modules)
#[rustfmt::skip]
pub use benefits::{
    // Immediate basic insurance benefits
    immediate::{A_x, A_x1_n, A_x_n1, A_x_n},

    // Immediate increasing insurance benefits
    immediate::{IA_x, IA_x1_n, IA_x_n1, IA_x_n},

    // Immediate decreasing insurance benefits
    immediate::{DA_x1_n, DA_x_n1, DA_x_n},

    // Immediate geometric insurance benefits
    immediate::{gA_x, gA_x1_n, gA_x_n1, gA_x_n},

    // Due basic insurance benefits (Äₓ notation)
    due::{AA_x, AA_x1_n, AA_x_n1, AA_x_n},

    // Due increasing insurance benefits (IÄₓ notation)
    due::{IAA_x, IAA_x1_n, IAA_x_n1, IAA_x_n},

    // Due decreasing insurance benefits (DÄₓ notation)
    due::{DAA_x1_n, DAA_x_n1, DAA_x_n},

    // Due geometric insurance benefits (gÄₓ notation)
    due::{gAA_x, gAA_x1_n, gAA_x_n1, gAA_x_n},

    // Deferred immediate basic insurance benefits
    deferred_immediate::{t_A_x, t_A_x1_n, t_A_x_n1, t_A_x_n},

    // Deferred immediate increasing insurance benefits
    deferred_immediate::{t_IA_x, t_IA_x1_n, t_IA_x_n1, t_IA_x_n},

    // Deferred immediate decreasing insurance benefits
    deferred_immediate::{t_DA_x1_n, t_DA_x_n1, t_DA_x_n},

    // Deferred immediate geometric insurance benefits
    deferred_immediate::{t_gA_x, t_gA_x1_n, t_gA_x_n1, t_gA_x_n},

    // Deferred due basic insurance benefits
    deferred_due::{t_AA_x, t_AA_x1_n, t_AA_x_n1, t_AA_x_n},

    // Deferred due increasing insurance benefits
    deferred_due::{t_IAA_x, t_IAA_x1_n, t_IAA_x_n1, t_IAA_x_n},

    // Deferred due decreasing insurance benefits
    deferred_due::{t_DAA_x1_n, t_DAA_x_n1, t_DAA_x_n},

    // Deferred due geometric insurance benefits
    deferred_due::{t_gAA_x, t_gAA_x1_n, t_gAA_x_n1, t_gAA_x_n},
};

// Re-export all annuity functions (order matches annuity modules)
#[rustfmt::skip]
pub use annuities::{
    // Due annuities
    due::{aa_x, aa_x_n, Iaa_x, Iaa_x_n, Daa_x_n, gaa_x, gaa_x_n},

    // Immediate annuities
    immediate::{a_x, a_x_n, Ia_x, Ia_x_n, Da_x_n, ga_x, ga_x_n},

    // Deferred due annuities
    deferred_due::{t_aa_x, t_aa_x_n, t_Iaa_x, t_Iaa_x_n, t_Daa_x_n, t_gaa_x, t_gaa_x_n},

    // Deferred immediate annuities
    deferred_immediate::{t_a_x, t_a_x_n, t_Ia_x, t_Ia_x_n, t_Da_x_n, t_ga_x, t_ga_x_n},
};
