pub mod fractionals;
pub mod mort_tbl_config;
pub mod wholes;

// Re-export types and enums from mort_tbl_config for easier imports
pub use mort_tbl_config::{AssumptionEnum, MortTableConfig};

// Re-export all actuarial functions from wholes for easier imports
pub use wholes::{
    // Annuities
    axn_due,
    gAx,
    gAxn,

    gIax_due,
    gIaxn_due,
    tAExn,
    tAx,
    tAxn,
    tExn,
    tIax_due,
    tIaxn_due,
    tax_due,
    taxn_due,
    AExn,
    // Life Insurance Benefits
    Ax,
    Axn,
    Exn,
    IAx,
    IAxn,
    Iax_due,
    Iaxn_due,
};

// Re-export fractional age functions
pub use fractionals::{conditional_tqx, tpx, tqx};
