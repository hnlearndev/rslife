/// Convert nominal interest rate to effective interest rate.
///
/// # Formula
/// ```text
/// i = (1 + i⁽ᵐ⁾/m)^m - 1
/// ```
/// where:
/// - `nom_i` is the nominal interest rate convertible m times per year
/// - `m_i` is the number of conversion periods per year (m-payable for nominal i)
///
/// # Example
/// ```rust
/// # use rslife::prelude::nom_i_to_eff_i;
/// let eff = nom_i_to_eff_i(0.06, 2); // semi-annual nominal 6%
/// println!("Effective i: {:.6}", eff);
/// ```
pub fn nom_i_to_eff_i(nom_i: f64, m: u32) -> f64 {
    let m_f64 = m as f64;
    (1.0 + nom_i / m_f64).powf(m_f64) - 1.0
}

/// Convert effective interest rate to nominal interest rate (m-payable).
///
/// # Formula
/// ```text
/// i⁽ᵐ⁾ = m[(1 + i)¹⁄ᵐ - 1]
/// ```
/// where:
/// - `eff_i` is the effective annual interest rate
/// - `m` is the number of conversion periods per year (m-payable for nominal i)
///
/// # Example
/// ```rust
/// # use rslife::prelude::eff_i_to_nom_i;
/// let nom = eff_i_to_nom_i(0.06136, 2); // effective 6.136% to nominal semi-annual
/// println!("Nominal i: {:.6}", nom);
/// ```
pub fn eff_i_to_nom_i(eff_i: f64, m: u32) -> f64 {
    let m_f64 = m as f64;
    m_f64 * ((1.0 + eff_i).powf(1.0 / m_f64) - 1.0)
}

/// Convert effective discount rate to nominal discount rate (m-payable).
///
/// # Formula
/// ```text
/// d⁽ᵐ⁾ = m[1-(1-d)¹⁄ᵐ]
/// ```
/// where:
/// - `eff_d` is the effective annual discount rate
/// - `m_d` is the number of conversion periods per year (m-payable for nominal d)
///
/// # Example
/// ```rust
/// # use rslife::prelude::eff_d_to_nom_d;
/// let nom = eff_d_to_nom_d(0.05, 4); // effective 5% to nominal quarterly
/// println!("Nominal d: {:.6}", nom);
/// ```
pub fn eff_d_to_nom_d(eff_d: f64, m: u32) -> f64 {
    let m_f64 = m as f64;
    m_f64 * (1.0 - (1.0 - eff_d).powf(1.0 / m_f64))
}

/// Convert nominal discount rate (m-payable) to effective discount rate.
///
/// # Formula
/// ```text
/// d = 1 - [1 - d⁽ᵐ⁾/m]^m
/// ```
/// where:
/// - `nom_d` is the nominal discount rate convertible m times per year
/// - `m_d` is the number of conversion periods per year (m-payable for nominal d)
///
/// # Example
/// ```rust
/// # use rslife::prelude::nom_d_to_eff_d;
/// let eff = nom_d_to_eff_d(0.0488, 4); // nominal quarterly 4.88% to effective
/// println!("Effective d: {:.6}", eff);
/// ```
pub fn nom_d_to_eff_d(nom_d: f64, m: u32) -> f64 {
    let m_f64 = m as f64;
    1.0 - (1.0 - nom_d / m_f64).powf(m_f64)
}

/// Convert effective interest rate to effective discount rate.
///
/// # Formula
/// ```text
/// d = i / (1 + i)
/// ```
/// where:
/// - `eff_i` is the effective annual interest rate
///
/// # Example
/// ```rust
/// # use rslife::prelude::eff_i_to_eff_d;
/// let eff_d = eff_i_to_eff_d(0.06);
/// println!("Effective d: {:.6}", eff_d);
/// ```
pub fn eff_i_to_eff_d(eff_i: f64) -> f64 {
    eff_i / (1.0 + eff_i)
}

/// Convert effective interest rate to nominal discount rate (m_d-payable).
///
/// # Formula
/// ```text
/// d⁽ᵐ⁾ = m[1-(1-d)¹⁄ᵐ], where d = i/(1+i)
/// ```
/// where:
/// - `eff_i` is the effective annual interest rate
/// - `m_d` is the number of conversion periods per year for nominal d (m-payable for d)
///
/// # Example
/// ```rust
/// # use rslife::prelude::eff_i_to_nom_d;
/// let nom_d = eff_i_to_nom_d(0.06, 4);
/// println!("Nominal d: {:.6}", nom_d);
/// ```
pub fn eff_i_to_nom_d(eff_i: f64, m_d: u32) -> f64 {
    let eff_d = eff_i_to_eff_d(eff_i);
    eff_d_to_nom_d(eff_d, m_d)
}

/// Convert nominal interest rate (m_i-payable) to effective discount rate.
///
/// # Formula
/// ```text
/// d = i / (1 + i), where i = (1 + i⁽ᵐ⁾/m)^m - 1
/// ```
/// where:
/// - `nom_i` is the nominal interest rate convertible m_i times per year
/// - `m_i` is the number of conversion periods per year for nominal i (m-payable for i)
///
/// # Example
/// ```rust
/// # use rslife::prelude::nom_i_to_eff_d;
/// let eff_d = nom_i_to_eff_d(0.06, 2);
/// println!("Effective d: {:.6}", eff_d);
/// ```
pub fn nom_i_to_eff_d(nom_i: f64, m_i: u32) -> f64 {
    let eff_i = nom_i_to_eff_i(nom_i, m_i);
    eff_i_to_eff_d(eff_i)
}

/// Convert nominal interest rate (m_i-payable) to nominal discount rate (m_d-payable).
///
/// # Formula
/// ```text
/// d⁽ᵐ⁾ = m[1-(1-d)¹⁄ᵐ], where d = i/(1+i), i = (1 + i⁽ᵐ⁾/m)^m - 1
/// ```
/// where:
/// - `nom_i` is the nominal interest rate convertible m_i times per year
/// - `m_i` is the number of conversion periods per year for nominal i (m-payable for i)
/// - `m_d` is the number of conversion periods per year for nominal d (m-payable for d)
///
/// # Example
/// ```rust
/// # use rslife::prelude::nom_i_to_nom_d;
/// let nom_d = nom_i_to_nom_d(0.06, 2, 4);
/// println!("Nominal d: {:.6}", nom_d);
/// ```
pub fn nom_i_to_nom_d(nom_i: f64, m_i: u32, m_d: u32) -> f64 {
    let eff_d = nom_i_to_eff_d(nom_i, m_i);
    eff_d_to_nom_d(eff_d, m_d)
}

/// Convert effective discount rate to effective interest rate.
///
/// # Formula
/// ```text
/// i = d / (1 - d)
/// ```
/// where:
/// - `eff_d` is the effective annual discount rate
///
/// # Example
/// ```rust
/// # use rslife::prelude::eff_d_to_eff_i;
/// let eff_i = eff_d_to_eff_i(0.05);
/// println!("Effective i: {:.6}", eff_i);
/// ```
pub fn eff_d_to_eff_i(eff_d: f64) -> f64 {
    eff_d / (1.0 - eff_d)
}

/// Convert effective discount rate to nominal interest rate (m_i-payable).
///
/// # Formula
/// ```text
/// i⁽ᵐ⁾ = m[(1 + i)¹⁄ᵐ − 1], where i = d⁄(1−d)
/// ```
/// where:
/// - `eff_d` is the effective annual discount rate
/// - `m_i` is the number of conversion periods per year for nominal i (m-payable for i)
///
/// # Example
/// ```rust
/// # use rslife::prelude::eff_d_to_nom_i;
/// let nom_i = eff_d_to_nom_i(0.05, 2);
/// println!("Nominal i: {:.6}", nom_i);
/// ```
pub fn eff_d_to_nom_i(eff_d: f64, m_i: u32) -> f64 {
    let eff_i = eff_d_to_eff_i(eff_d);
    eff_i_to_nom_i(eff_i, m_i)
}

/// Convert nominal discount rate (m_d-payable) to effective interest rate.
///
/// # Formula
/// ```text
/// i = d / (1 - d), where d = 1 - [1 - d⁽ᵐ⁾/m]^m
/// ```
/// where:
/// - `nom_d` is the nominal discount rate convertible m_d times per year
/// - `m_d` is the number of conversion periods per year for nominal d (m-payable for d)
///
/// # Example
/// ```rust
/// # use rslife::prelude::nom_d_to_eff_i;
/// let eff_i = nom_d_to_eff_i(0.0488, 4);
/// println!("Effective i: {:.6}", eff_i);
/// ```
pub fn nom_d_to_eff_i(nom_d: f64, m_d: u32) -> f64 {
    let eff_d = nom_d_to_eff_d(nom_d, m_d);
    eff_d_to_eff_i(eff_d)
}

/// Convert nominal discount rate (m_d-payable) to nominal interest rate (m_i-payable).
///
/// # Formula
/// ```text
/// i⁽ᵐ⁾ = m[(1 + i)¹⁄ᵐ - 1], where i = d/(1-d), d = 1 - [1 - d⁽ᵐ⁾/m]^m
/// ```
/// where:
/// - `nom_d` is the nominal discount rate convertible m_d times per year
/// - `m_d` is the number of conversion periods per year for nominal d (m-payable for d)
/// - `m_i` is the number of conversion periods per year for nominal i (m-payable for i)
///
/// # Example
/// ```rust
/// # use rslife::prelude::nom_d_to_nom_i;
/// let nom_i = nom_d_to_nom_i(0.0488, 4, 2);
/// println!("Nominal i: {:.6}", nom_i);
/// ```
pub fn nom_d_to_nom_i(nom_d: f64, m_d: u32, m_i: u32) -> f64 {
    let eff_i = nom_d_to_eff_i(nom_d, m_d);
    eff_i_to_nom_i(eff_i, m_i)
}
