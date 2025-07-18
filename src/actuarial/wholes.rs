//! # Actuarial Functions for Life Insurance and Annuity Calculations
//!
//! This module provides a complete set of actuarial functions for calculating life insurance
//! benefits and annuity values using standard commutation functions. All calculations are
//! based on mortality tables with optional interest rate considerations.
//!
//! ## Mathematical Foundation
//!
//! The functions use standard actuarial notation and commutation functions:
//! - $D_x = v^x \cdot l_x$ (present value of lives at age x)
//! - $C_x = v^{x+1} \cdot d_x$ (present value of deaths at age x)
//! - $M_x = \sum_{k=x}^{\omega} C_k$ (sum of C values from age x)
//! - $N_x = \sum_{k=x}^{\omega} D_k$ (sum of D values from age x)
//! - $S_x = \sum_{k=x}^{\omega} N_k$ (sum of N values from age x)
//!
//! Where:
//! - $v = \frac{1}{1+i}$ is the present value discount factor
//! - $i$ is the interest rate
//! - $l_x$ is the number of lives at age x
//! - $d_x$ is the number of deaths between age x and x+1
//! - $\omega$ is the terminal age
//!
//! ## Function Categories
//!
//! ### Insurance Benefits
//! - **Whole Life**: $A_x$ - Present value of $1 paid at death
//! - **Term**: $A_{x:\overline{n}|}$ - Present value for n-year term
//! - **Endowment**: $A_{x:\overline{n}}$ - Combined term insurance and pure endowment
//! - **Deferred**: $_t|A_x$ - Benefits starting after t years
//! - **Increasing**: $(IA)_x$ - Benefits that increase each year
//! - **Geometric**: $A_x^{(g)}$ - Benefits growing at rate g
//!
//! ### Annuities
//! - **Life Annuities**: $\ddot{a}_x^{(m)}$ - Payments while alive
//! - **Temporary**: $\ddot{a}_{x:\overline{n}|}^{(m)}$ - Limited-time payments
//! - **Deferred**: $_t|\ddot{a}_x^{(m)}$ - Payments starting after delay
//! - **Increasing**: $(I\ddot{a})_x^{(m)}$ - Payments that increase
//! - **Geometric**: $(g\ddot{a})_x^{(m)}$ - Payments growing at rate g
//!
//! ## Examples
//!
//! ```rust
//! use rslife::prelude::*;
//!
//! // All actuarial functions are available through the prelude:
//! // Life insurance: Ax, Axn, AExn, IAx, IAxn, gAx, gAxn
//! // Deferred: tAx, tAxn, tExn, tAExn  
//! // Annuities: axn_due, tax_due, taxn_due, Iax_due, Iaxn_due
//! // And many more...
//! ```

#![allow(non_snake_case)] // Allow actuarial notation (Ax, Axn, etc.)

use crate::actuarial::mort_tbl_config::MortTableConfig;
use polars::prelude::*;

//------------------- Insurance benefit -------------------

/// Calculates the actuarial present value of a whole life insurance of $1.
///
/// **Mathematical Formula**: $A_x = \frac{M_x}{D_x}$
///
/// This represents the expected present value of $1 paid immediately upon
/// the death of a life currently aged x.
///
/// # Parameters
/// - `config`: Mortality table configuration with interest rate
/// - `x`: Current age of the insured
///
/// # Returns
/// Present value of the whole life insurance benefit
///
/// # Example
/// ```rust
/// use rslife::prelude::*;
/// 
/// // Ax(&config, 30) calculates whole life insurance for age 30
/// // Returns the present value of $1 paid at death
/// ```
///
/// # Mathematical Background
/// The whole life insurance is the most fundamental life insurance product.
/// It provides a death benefit regardless of when death occurs, with the
/// present value calculated using the mortality table and interest rate.
pub fn Ax(config: &MortTableConfig, x: i32) -> PolarsResult<f64> {
    get_value(config, x, "Ax")
}

/// Calculates the actuarial present value of an n-year term life insurance.
///
/// **Mathematical Formula**: $A_{x:\overline{n}|} = A_x - A_{x+n} \cdot E_{x:n}$
///
/// This represents the present value of $1 paid only if death occurs
/// within n years of issue.
///
/// # Parameters
/// - `config`: Mortality table configuration with interest rate
/// - `x`: Current age of the insured
/// - `n`: Term length in years
///
/// # Returns
/// Present value of the n-year term insurance benefit
///
/// # Example
/// ```rust
/// let term_insurance = Axn(&config, 30, 20)?;
/// // 20-year term insurance for a 30-year-old
/// ```
///
/// # Use Cases
/// - Temporary insurance needs (mortgage protection, income replacement)
/// - Lower-cost alternative to whole life insurance
/// - Coverage during working years only
pub fn Axn(config: &MortTableConfig, x: i32, n: i32) -> PolarsResult<f64> {
    let ax = Ax(config, x)?;
    let axn = Ax(config, x + n)?;
    let exn = Exn(config, x, n)?;
    let result = ax - axn * exn;
    Ok(result)
}

/// Calculates the actuarial present value of an n-year pure endowment.
///
/// **Mathematical Formula**: $E_{x:n} = \frac{D_{x+n}}{D_x}$
///
/// This represents the present value of $1 paid if and only if the
/// insured survives n years.
///
/// # Parameters
/// - `config`: Mortality table configuration with interest rate
/// - `x`: Current age of the insured
/// - `n`: Endowment period in years
///
/// # Returns
/// Present value of the pure endowment benefit
///
/// # Example
/// ```rust
/// let pure_endowment = Exn(&config, 30, 20)?;
/// // $1 paid if the 30-year-old survives to age 50
/// ```
///
/// # Mathematical Background
/// Pure endowments are the building blocks for many insurance products.
/// They represent the survival probability adjusted for interest.
pub fn Exn(config: &MortTableConfig, x: i32, n: i32) -> PolarsResult<f64> {
    let dx = get_value(config, x, "Dx")?;
    let dxn = get_value(config, x + n, "Dx")?;
    let result = dxn / dx;
    Ok(result)
}

/// Calculates the actuarial present value of an n-year endowment insurance.
///
/// **Mathematical Formula**: $A_{x:\overline{n}} = A_{x:\overline{n}|} + E_{x:n}$
///
/// This combines term insurance and pure endowment: $1 is paid either
/// at death (if within n years) or at survival to n years.
///
/// # Parameters
/// - `config`: Mortality table configuration with interest rate
/// - `x`: Current age of the insured
/// - `n`: Endowment period in years
///
/// # Returns
/// Present value of the endowment insurance benefit
///
/// # Example
/// ```rust
/// let endowment = AExn(&config, 30, 20)?;
/// // $1 paid at death within 20 years OR at survival to 20 years
/// ```
///
/// # Use Cases
/// - Savings with insurance protection
/// - Education funding
/// - Retirement planning with death benefit protection
pub fn AExn(config: &MortTableConfig, x: i32, n: i32) -> PolarsResult<f64> {
    let axn = Axn(config, x, n)?;
    let exn = Exn(config, x, n)?;
    let result = axn + exn;
    Ok(result)
}

/// Calculates the actuarial present value of a t-year deferred whole life insurance.
///
/// **Mathematical Formula**: $_{t|}A_x = \frac{M_{x+t}}{D_x}$
///
/// This represents the present value of $1 paid at death, but only if
/// death occurs after t years have elapsed.
///
/// # Parameters
/// - `config`: Mortality table configuration with interest rate
/// - `x`: Current age of the insured
/// - `t`: Deferral period in years
///
/// # Returns
/// Present value of the deferred whole life insurance benefit
///
/// # Example
/// ```rust
/// let deferred_life = tAx(&config, 30, 10)?;
/// // Whole life insurance starting after 10 years for a 30-year-old
/// ```
///
/// # Use Cases
/// - Reduced premium insurance (no coverage during deferral period)
/// - Second-to-die insurance planning
/// - Estate planning with delayed coverage needs
pub fn tAx(config: &MortTableConfig, x: i32, t: i32) -> PolarsResult<f64> {
    let mx_t = get_value(config, x + t, "Mx")?;
    let dx = get_value(config, x, "Dx")?;
    let result = mx_t / dx;
    Ok(result)
}

/// Calculates the actuarial present value of a t-year deferred, n-year term insurance.
///
/// **Mathematical Formula**: $_{t|}A_{x:\overline{n}|} = _{t|}A_x - _{t+n|}A_x$
///
/// This represents $1 paid at death, but only if death occurs between
/// t and t+n years after issue.
///
/// # Parameters
/// - `config`: Mortality table configuration with interest rate
/// - `x`: Current age of the insured
/// - `n`: Term length in years
/// - `t`: Deferral period in years
///
/// # Returns
/// Present value of the deferred term insurance benefit
///
/// # Example
/// ```rust
/// let deferred_term = tAxn(&config, 30, 20, 10)?;
/// // 20-year term starting after 10 years for a 30-year-old
/// ```
pub fn tAxn(config: &MortTableConfig, x: i32, n: i32, t: i32) -> PolarsResult<f64> {
    let tax = tAx(config, x, t)?;
    let taxn = tAx(config, x, t + n)?;
    let result = tax - taxn;
    Ok(result)
}

/// Calculates the actuarial present value of a t-year deferred, n-year pure endowment.
///
/// **Mathematical Formula**: $_{t|}E_{x:n} = \frac{D_{x+n+t}}{D_x}$
///
/// This represents $1 paid if and only if the insured survives both
/// the deferral period t and the additional period n.
///
/// # Parameters
/// - `config`: Mortality table configuration with interest rate
/// - `x`: Current age of the insured
/// - `n`: Additional survival period after deferral
/// - `t`: Deferral period in years
///
/// # Returns
/// Present value of the deferred pure endowment benefit
pub fn tExn(config: &MortTableConfig, x: i32, n: i32, t: i32) -> PolarsResult<f64> {
    let dx = get_value(config, x, "Dx")?;
    let dxnt = get_value(config, x + n + t, "Dx")?;
    let result = dxnt / dx;
    Ok(result)
}

/// Calculates the actuarial present value of a t-year deferred, n-year endowment insurance.
///
/// **Mathematical Formula**: $_{t|}A_{x:\overline{n}} = _{t|}A_{x:\overline{n}|} + _{t|}E_{x:n}$
///
/// # Parameters
/// - `config`: Mortality table configuration with interest rate
/// - `x`: Current age of the insured
/// - `n`: Endowment period in years
/// - `t`: Deferral period in years
///
/// # Returns
/// Present value of the deferred endowment insurance benefit
pub fn tAExn(config: &MortTableConfig, x: i32, n: i32, t: i32) -> PolarsResult<f64> {
    let taxn = tAxn(config, x, n, t)?;
    let texn = tExn(config, x, n, t)?;
    let result = taxn + texn;
    Ok(result)
}

/// Calculates the actuarial present value of an increasing whole life insurance.
///
/// **Mathematical Formula**: $(IA)_x = \frac{S_x}{D_x}$
///
/// The death benefit increases by $1 each year: $k is paid if death
/// occurs in the k-th year after issue.
///
/// # Parameters
/// - `config`: Mortality table configuration with interest rate
/// - `x`: Current age of the insured
///
/// # Returns
/// Present value of the increasing whole life insurance benefit
///
/// # Example
/// ```rust
/// let increasing_life = IAx(&config, 30)?;
/// // Whole life with benefits of $1, $2, $3, ... by year of death
/// ```
///
/// # Use Cases
/// - Inflation protection for life insurance
/// - Coverage that grows with income needs
/// - Estate planning with increasing wealth transfer
pub fn IAx(config: &MortTableConfig, x: i32) -> PolarsResult<f64> {
    get_value(config, x, "IAx")
}

/// Calculates the actuarial present value of an increasing n-year term insurance.
///
/// **Mathematical Formula**: $(IA)_{x:\overline{n}|} = (S_x - S_{x+n} - n \cdot M_{x+n})/D_x$
///
/// The death benefit increases by $1 each year, but only pays if death
/// occurs within n years.
///
/// # Parameters
/// - `config`: Mortality table configuration with interest rate
/// - `x`: Current age of the insured
/// - `n`: Term length in years
///
/// # Returns
/// Present value of the increasing term insurance benefit
///
/// # Example
/// ```rust
/// let increasing_term = IAxn(&config, 30, 20)?;
/// // 20-year term with benefits of $1, $2, ..., $20 by year of death
/// ```
pub fn IAxn(config: &MortTableConfig, x: i32, n: i32) -> PolarsResult<f64> {
    let sx = get_value(config, x, "Sx")?;
    let sxn = get_value(config, x + n, "Sx")?;
    let mxn = get_value(config, x + n, "Mx")?;
    let dx = get_value(config, x, "Dx")?;
    let result = (sx - sxn - n as f64 * mxn) / dx;
    Ok(result)
}

/// Calculates the actuarial present value of a geometrically increasing whole life insurance.
///
/// **Mathematical Formula**: $A_x^{(g)} = A_x$ (calculated with adjusted interest rate)
///
/// The death benefit grows geometrically at rate g each year. This is
/// calculated by adjusting the interest rate to $i' = \frac{1+i}{1+g} - 1$.
///
/// # Parameters
/// - `config`: Mortality table configuration with interest rate
/// - `x`: Current age of the insured
/// - `g`: Annual growth rate of benefits (as decimal, e.g., 0.03 for 3%)
///
/// # Returns
/// Present value of the geometrically increasing whole life insurance
///
/// # Example
/// ```rust
/// let geometric_life = gAx(&config, 30, 0.03)?;
/// // Whole life with benefits growing at 3% annually
/// ```
///
/// # Use Cases
/// - Inflation-adjusted life insurance
/// - Coverage tied to investment returns
/// - Long-term wealth transfer planning
pub fn gAx(config: &MortTableConfig, x: i32, g: f64) -> PolarsResult<f64> {
    let new_config = get_new_config(config, g);
    let result = Ax(&new_config, x)?;
    Ok(result)
}

/// Calculates the actuarial present value of a geometrically increasing n-year term insurance.
///
/// **Mathematical Formula**: $A_{x:\overline{n}|}^{(g)} = A_{x:\overline{n}|}$ (with adjusted interest rate)
///
/// # Parameters
/// - `config`: Mortality table configuration with interest rate
/// - `x`: Current age of the insured
/// - `n`: Term length in years
/// - `g`: Annual growth rate of benefits
///
/// # Returns
/// Present value of the geometrically increasing term insurance
pub fn gAxn(config: &MortTableConfig, x: i32, n: i32, g: f64) -> PolarsResult<f64> {
    let new_config = get_new_config(config, g);
    let result = Axn(&new_config, x, n)?;
    Ok(result)
}

//------------------- Annuities -------------------

/// Calculates the present value of a life annuity-due with m payments per year.
///
/// **Mathematical Formula**: $\ddot{a}_x^{(m)} = \frac{1}{m} \left[\frac{N_x}{D_x} - \frac{m-1}{2m}\right]$
///
/// This is a private function used internally for annuity calculations.
///
/// # Parameters
/// - `config`: Mortality table configuration with interest rate
/// - `x`: Current age of the annuitant
/// - `m`: Number of payments per year (1=annual, 12=monthly, etc.)
///
/// # Mathematical Background
/// The correction factor $\frac{m-1}{2m}$ adjusts for the timing difference
/// between continuous and discrete payments within each year.
fn ax_due(config: &MortTableConfig, x: i32, m: i32) -> PolarsResult<f64> {
    let nx = get_value(config, x, "Nx")?;
    let dx = get_value(config, x, "Dx")?;
    let ax = nx / dx;
    let correction = (m as f64 - 1.0) / (2.0 * m as f64);
    Ok((1.0 / m as f64) * (ax - correction))
}

/// Calculates the present value of an n-year temporary annuity-due with m payments per year.
///
/// **Mathematical Formula**:
/// $$\ddot{a}_{x:\overline{n}|}^{(m)} = \frac{1}{m} \left[\frac{N_x - N_{x+n}}{D_x} - \frac{m-1}{2m}\left(1 - \frac{D_{x+n}}{D_x}\right)\right]$$
///
/// This represents the present value of periodic payments of $\frac{1}{m}$ made
/// m times per year while the annuitant is alive, but for at most n years.
///
/// # Parameters
/// - `config`: Mortality table configuration with interest rate
/// - `x`: Current age of the annuitant
/// - `n`: Maximum number of years of payments
/// - `m`: Number of payments per year
///
/// # Returns
/// Present value of the temporary annuity-due
///
/// # Example
/// ```rust
/// // Monthly annuity for 20 years starting at age 65
/// let monthly_annuity = axn_due(&config, 65, 20, 12)?;
/// // This gives the present value of $1/12 paid monthly for 20 years or until death
///
/// // Annual annuity for life starting at age 65
/// let life_annuity = axn_due(&config, 65, 100, 1)?; // n=100 approximates lifetime
/// ```
///
/// # Use Cases
/// - Pension payments with guaranteed period
/// - Income annuities with term certain
/// - Retirement income planning
/// - Social security benefit valuations
pub fn axn_due(config: &MortTableConfig, x: i32, n: i32, m: i32) -> PolarsResult<f64> {
    let dx = get_value(config, x, "Dx")?;
    let dxn = get_value(config, x + n, "Dx")?;
    let nx = get_value(config, x, "Nx")?;
    let nxn = get_value(config, x + n, "Nx")?;
    let annuity = (nx - nxn) / dx;
    let correction = ((m as f64 - 1.0) / (2.0 * m as f64)) * (1.0 - dxn / dx);
    Ok((1.0 / m as f64) * (annuity - correction))
}

//------------------- Deferred Annuities -------------------

/// Calculates the present value of a t-year deferred life annuity-due.
///
/// **Mathematical Formula**: $_{t|}\ddot{a}_x^{(m)} = \frac{D_{x+t}}{D_x} \times \ddot{a}_{x+t}^{(m)}$
///
/// This represents an annuity that begins payments after a deferral period of t years.
///
/// # Parameters
/// - `config`: Mortality table configuration with interest rate
/// - `x`: Current age of the annuitant
/// - `t`: Deferral period in years
/// - `m`: Number of payments per year
///
/// # Returns
/// Present value of the deferred life annuity
///
/// # Example
/// ```rust
/// // Annuity starting 10 years from now for a 55-year-old
/// let deferred_annuity = tax_due(&config, 55, 10, 12)?;
/// // Monthly payments starting at age 65
/// ```
///
/// # Use Cases
/// - Deferred retirement annuities
/// - Social Security delayed benefits
/// - Pension plans with vesting periods
pub fn tax_due(config: &MortTableConfig, x: i32, t: i32, m: i32) -> PolarsResult<f64> {
    let dx = get_value(config, x, "Dx")?;
    let dxh = get_value(config, x + t, "Dx")?;
    let ax_due_h = ax_due(config, x + t, m)?;
    Ok((dxh / dx) * ax_due_h)
}

/// Calculates the present value of a t-year deferred, n-year temporary annuity-due.
///
/// **Mathematical Formula**: $_{t|}\ddot{a}_{x:\overline{n}|}^{(m)} = _{t|}\ddot{a}_x^{(m)} - _{t+n|}\ddot{a}_x^{(m)}$
///
/// This represents an annuity with both a deferral period and a limited payment period.
///
/// # Parameters
/// - `config`: Mortality table configuration with interest rate
/// - `x`: Current age of the annuitant
/// - `n`: Number of years of payments
/// - `t`: Deferral period in years
/// - `m`: Number of payments per year
///
/// # Returns
/// Present value of the deferred temporary annuity
///
/// # Example
/// ```rust
/// // Annuity paying for 20 years, starting after 10-year deferral
/// let deferred_temp = taxn_due(&config, 55, 20, 10, 1)?;
/// // Annual payments from age 65 to 85
/// ```
pub fn taxn_due(config: &MortTableConfig, x: i32, n: i32, t: i32, m: i32) -> PolarsResult<f64> {
    let tax_due_h = tax_due(config, x, t, m)?;
    let tax_due_hn = tax_due(config, x, t + n, m)?;
    Ok(tax_due_h - tax_due_hn)
}

//------------------- Increasing Annuities -------------------

/// Calculates the present value of an increasing life annuity-due.
///
/// **Mathematical Formula**:
/// $$(I\ddot{a})_x^{(m)} = \frac{1}{m} \times \frac{(3-m)(S_x + N_x) - (m-1)D_x}{2D_x}$$
///
/// The annuity payments increase by $\frac{1}{m}$ each year.
///
/// # Parameters
/// - `config`: Mortality table configuration with interest rate
/// - `x`: Current age of the annuitant
/// - `n`: Parameter for compatibility (not used in calculation)
/// - `m`: Number of payments per year
///
/// # Returns
/// Present value of the increasing life annuity
///
/// # Example
/// ```rust
/// // Annual increasing annuity starting at age 65
/// let increasing_annuity = Iax_due(&config, 65, 0, 1)?;
/// // Payments of $1, $2, $3, ... each year
/// ```
///
/// # Use Cases
/// - Inflation-protected pension payments
/// - Income that grows with cost of living
/// - Long-term care insurance with increasing benefits
pub fn Iax_due(config: &MortTableConfig, x: i32, _n: i32, m: i32) -> PolarsResult<f64> {
    let sx = get_value(config, x, "Sx")?;
    let nx = get_value(config, x, "Nx")?;
    let dx = get_value(config, x, "Dx")?;
    let numerator = (3.0 - m as f64) * (sx + nx) - (m as f64 - 1.0) * dx;
    let denominator = 2.0 * dx;
    Ok((1.0 / m as f64) * (numerator / denominator))
}

/// Calculates the present value of an increasing n-year temporary annuity-due.
///
/// **Mathematical Formula**:
/// $$(I\ddot{a})_{x:\overline{n}|}^{(m)} = (I\ddot{a})_x^{(m)} - \frac{D_{x+n}}{D_x} \times (I\ddot{a})_{x+n}^{(m)}$$
///
/// # Parameters
/// - `config`: Mortality table configuration with interest rate
/// - `x`: Current age of the annuitant
/// - `n`: Number of years of payments
/// - `m`: Number of payments per year
///
/// # Returns
/// Present value of the increasing temporary annuity
pub fn Iaxn_due(config: &MortTableConfig, x: i32, n: i32, m: i32) -> PolarsResult<f64> {
    let iax = Iax_due(config, x, n, m)?;
    let dx = get_value(config, x, "Dx")?;
    let dxn = get_value(config, x + n, "Dx")?;
    let iax_n = Iax_due(config, x + n, n, m)?;
    Ok(iax - (dxn / dx) * iax_n)
}

/// Calculates the present value of a t-year deferred increasing life annuity-due.
///
/// **Mathematical Formula**:
/// $$_{t|}(I\ddot{a})_x^{(m)} = \frac{D_{x+t}}{D_x} \times (I\ddot{a})_{x+t}^{(m)}$$
///
/// # Parameters
/// - `config`: Mortality table configuration with interest rate
/// - `x`: Current age of the annuitant
/// - `n`: Parameter for compatibility
/// - `t`: Deferral period in years
/// - `m`: Number of payments per year
///
/// # Returns
/// Present value of the deferred increasing life annuity
pub fn tIax_due(config: &MortTableConfig, x: i32, n: i32, t: i32, m: i32) -> PolarsResult<f64> {
    let dx = get_value(config, x, "Dx")?;
    let dx_t = get_value(config, x + t, "Dx")?;
    let iax_due_t = Iax_due(config, x + t, n, m)?;
    Ok((dx_t / dx) * iax_due_t)
}

/// Calculates the present value of a t-year deferred, n-year increasing temporary annuity-due.
///
/// **Mathematical Formula**:
/// $$_{t|}(I\ddot{a})_{x:\overline{n}|}^{(m)} = _{t|}(I\ddot{a})_x^{(m)} - _{t+n|}(I\ddot{a})_x^{(m)}$$
///
/// # Parameters
/// - `config`: Mortality table configuration with interest rate
/// - `x`: Current age of the annuitant
/// - `n`: Number of years of payments
/// - `t`: Deferral period in years
/// - `m`: Number of payments per year
///
/// # Returns
/// Present value of the deferred increasing temporary annuity
pub fn tIaxn_due(config: &MortTableConfig, x: i32, n: i32, t: i32, m: i32) -> PolarsResult<f64> {
    let t_iax_due = tIax_due(config, x, n, t, m)?;
    let t_iax_due_n = tIax_due(config, x, n, t + n, m)?;
    Ok(t_iax_due - t_iax_due_n)
}

/// Calculates the present value of a geometrically increasing life annuity-due.
///
/// **Mathematical Formula**: Calculated using adjusted interest rate $i' = \frac{1+i}{1+g} - 1$
///
/// # Parameters
/// - `config`: Mortality table configuration with interest rate
/// - `x`: Current age of the annuitant
/// - `n`: Parameter for compatibility
/// - `m`: Number of payments per year
/// - `g`: Annual growth rate of payments
///
/// # Returns
/// Present value of the geometrically increasing life annuity
///
/// # Example
/// ```rust
/// // Annuity growing at 3% annually
/// let geometric_annuity = gIax_due(&config, 65, 0, 1, 0.03)?;
/// ```
pub fn gIax_due(config: &MortTableConfig, x: i32, n: i32, m: i32, g: f64) -> PolarsResult<f64> {
    let new_config = get_new_config(config, g);
    let result = Iax_due(&new_config, x, n, m)?;
    Ok(result)
}

/// Calculates the present value of a geometrically increasing temporary annuity-due.
///
/// # Parameters
/// - `config`: Mortality table configuration with interest rate
/// - `x`: Current age of the annuitant
/// - `n`: Number of years of payments
/// - `m`: Number of payments per year
/// - `g`: Annual growth rate of payments
///
/// # Returns
/// Present value of the geometrically increasing temporary annuity
pub fn gIaxn_due(config: &MortTableConfig, x: i32, n: i32, m: i32, g: f64) -> PolarsResult<f64> {
    let new_config = get_new_config(config, g);
    let result = Iaxn_due(&new_config, x, n, m)?;
    Ok(result)
}

//---------------------------------------------------------
// PRIVATE FUNCTIONS
//---------------------------------------------------------

/// Creates a new configuration with adjusted interest rate for geometric growth calculations.
///
/// **Mathematical Formula**: $i' = \frac{1+i}{1+g} - 1$
///
/// This adjustment allows geometric growth calculations to be performed using
/// standard actuarial functions with the modified interest rate.
///
/// # Parameters
/// - `config`: Original mortality table configuration
/// - `g`: Growth rate for geometric calculations
///
/// # Returns
/// New configuration with adjusted interest rate
fn get_new_config(config: &MortTableConfig, g: f64) -> MortTableConfig {
    let i = config.int_rate.unwrap();
    let int_rate = (1.0 + i) / (1.0 + g) - 1.0;
    MortTableConfig {
        int_rate: Some(int_rate),
        xml: config.xml.clone(),
        l_x_init: config.l_x_init,
        pct: config.pct,
        assumption: config.assumption,
    }
}

/// Retrieves a specific commutation function value from the mortality table.
///
/// This function generates the mortality table and extracts the requested
/// commutation function value for the specified age.
///
/// # Parameters
/// - `config`: Mortality table configuration
/// - `x`: Age for which to retrieve the value
/// - `column_name`: Name of the commutation function column
///
/// # Returns
/// The requested commutation function value
///
/// # Errors
/// Returns `PolarsError::ComputeError` if:
/// - The mortality table cannot be generated
/// - The requested age is not found in the table
/// - The requested column does not exist
fn get_value(config: &MortTableConfig, x: i32, column_name: &str) -> PolarsResult<f64> {
    let df = config
        .gen_mort_table()?
        .lazy()
        .filter(col("age").eq(lit(x)))
        .select([col(column_name)])
        .collect()?;

    let result = df
        .column(column_name)
        .unwrap()
        .f64()
        .unwrap()
        .get(0)
        .ok_or_else(|| {
            PolarsError::ComputeError(format!("No {column_name} value found for age {x}").into())
        })?;

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::actuarial::mort_tbl_config::MortTableConfig;
    use crate::xml::MortXML;

    #[test]
    fn test_whole_life_insurance() {
        let xml = MortXML::from_url_id(1704).expect("Failed to load XML");
        let config = MortTableConfig {
            xml,
            l_x_init: 100_000,
            pct: Some(1.0),
            int_rate: Some(0.03),
            assumption: None,
        };

        let ax_30 = Ax(&config, 30).expect("Failed to calculate Ax");

        // Whole life insurance should be between 0 and 1
        assert!(
            ax_30 > 0.0 && ax_30 < 1.0,
            "Ax should be a probability-like value"
        );

        // Older ages should have higher insurance values
        let ax_60 = Ax(&config, 60).expect("Failed to calculate Ax for age 60");
        assert!(ax_60 > ax_30, "Insurance value should increase with age");

        println!("✓ Whole life insurance A_30 = {:.6}", ax_30);
        println!("✓ Whole life insurance A_60 = {:.6}", ax_60);
    }

    #[test]
    fn test_term_insurance() {
        let xml = MortXML::from_url_id(1704).expect("Failed to load XML");
        let config = MortTableConfig {
            xml,
            l_x_init: 100_000,
            pct: Some(1.0),
            int_rate: Some(0.03),
            assumption: None,
        };

        let axn_20 = Axn(&config, 30, 20).expect("Failed to calculate 20-year term");
        let axn_10 = Axn(&config, 30, 10).expect("Failed to calculate 10-year term");
        let ax_whole = Ax(&config, 30).expect("Failed to calculate whole life");

        // Term insurance should be less than whole life
        assert!(
            axn_20 < ax_whole,
            "Term insurance should be less than whole life"
        );
        assert!(axn_10 < axn_20, "Shorter term should have lower value");

        println!("✓ 10-year term A_30:10 = {:.6}", axn_10);
        println!("✓ 20-year term A_30:20 = {:.6}", axn_20);
        println!("✓ Whole life A_30 = {:.6}", ax_whole);
    }

    #[test]
    fn test_annuities() {
        let xml = MortXML::from_url_id(1704).expect("Failed to load XML");
        let config = MortTableConfig {
            xml,
            l_x_init: 100_000,
            pct: Some(1.0),
            int_rate: Some(0.03),
            assumption: None,
        };

        // Test annual annuity
        let annuity_annual =
            axn_due(&config, 65, 30, 1).expect("Failed to calculate annual annuity");

        // Test monthly annuity
        let annuity_monthly =
            axn_due(&config, 65, 30, 12).expect("Failed to calculate monthly annuity");

        // Monthly should be lower due to interest discounting between payments
        assert!(
            annuity_monthly < annuity_annual,
            "Monthly annuity should be lower than annual due to discounting"
        );

        // Values should be reasonable for retirement planning
        assert!(
            annuity_annual > 10.0 && annuity_annual < 25.0,
            "Annual annuity value seems unreasonable"
        );

        println!("✓ Annual annuity ä_65:30 = {:.6}", annuity_annual);
        println!("✓ Monthly annuity ä_65:30^(12) = {:.6}", annuity_monthly);
    }

    #[test]
    fn test_endowment_insurance() {
        let xml = MortXML::from_url_id(1704).expect("Failed to load XML");
        let config = MortTableConfig {
            xml,
            l_x_init: 100_000,
            pct: Some(1.0),
            int_rate: Some(0.03),
            assumption: None,
        };

        let endowment = AExn(&config, 30, 20).expect("Failed to calculate endowment");
        let term = Axn(&config, 30, 20).expect("Failed to calculate term");
        let pure_endowment = Exn(&config, 30, 20).expect("Failed to calculate pure endowment");

        // Endowment should equal term plus pure endowment
        let sum = term + pure_endowment;
        assert!(
            (endowment - sum).abs() < 1e-10,
            "Endowment should equal term + pure endowment"
        );

        println!("✓ 20-year endowment A_30:20 = {:.6}", endowment);
        println!("✓ Term component = {:.6}", term);
        println!("✓ Pure endowment component = {:.6}", pure_endowment);
    }

    #[test]
    fn test_geometric_growth() {
        let xml = MortXML::from_url_id(1704).expect("Failed to load XML");
        let config = MortTableConfig {
            xml,
            l_x_init: 100_000,
            pct: Some(1.0),
            int_rate: Some(0.03),
            assumption: None,
        };

        let standard = Ax(&config, 30).expect("Failed to calculate standard whole life");
        let geometric = gAx(&config, 30, 0.02).expect("Failed to calculate geometric whole life");

        // Geometric growth should increase the insurance value
        assert!(
            geometric > standard,
            "Geometric growth should increase insurance value"
        );

        println!("✓ Standard whole life A_30 = {:.6}", standard);
        println!("✓ Geometric whole life (2% growth) = {:.6}", geometric);
    }

    #[test]
    fn test_deferred_insurance() {
        let xml = MortXML::from_url_id(1704).expect("Failed to load XML");
        let config = MortTableConfig {
            xml,
            l_x_init: 100_000,
            pct: Some(1.0),
            int_rate: Some(0.03),
            assumption: None,
        };

        let immediate = Ax(&config, 30).expect("Failed to calculate immediate");
        let deferred_10 = tAx(&config, 30, 10).expect("Failed to calculate 10-year deferred");

        // Deferred should be less than immediate
        assert!(
            deferred_10 < immediate,
            "Deferred insurance should be less than immediate"
        );

        println!("✓ Immediate whole life A_30 = {:.6}", immediate);
        println!("✓ 10-year deferred 10|A_30 = {:.6}", deferred_10);
    }
}
