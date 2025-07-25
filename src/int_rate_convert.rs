// ====== Same i======
pub fn nom_i_to_eff_i(nom_i: f64, m: u32) -> f64 {
    //  i = (1 + i⁽ᵐ⁾/m)ᵐ - 1 ✅
    let m_f64 = m as f64;
    let eff_i = (1.0 + nom_i / m_f64).powf(m_f64) - 1.0;
    eff_i
}

pub fn eff_i_to_nom_i(eff_i: f64, m: u32) -> f64 {
    // i⁽ᵐ⁾ = (1 + i)⁽¹/ᵐ⁾ - 1 ✅
    let m_f64 = m as f64;
    let nom_i = (1.0 + eff_i).powf(1.0 / m_f64) - 1.0;
    nom_i
}

// ======Same d======
pub fn eff_d_to_nom_d(eff_d: f64, m: u32) -> f64 {
    // d⁽ᵐ⁾ = m[1-(1-d)⁽¹/ᵐ⁾] ✅
    let m_f64 = m as f64;
    let nom_d = m_f64 * (1.0 - (1.0 - eff_d).powf(1.0 / m_f64));
    nom_d
}

pub fn nom_d_to_eff_d(nom_d: f64, m: u32) -> f64 {
    // d = 1 - [1 - d⁽ᵐ⁾/m]ᵐ ✅
    let m_f64 = m as f64;
    let eff_d = 1.0 - (1.0 - nom_d / m_f64).powf(m_f64);
    eff_d
}

// ======From i to d======
pub fn eff_i_to_eff_d(eff_i: f64) -> f64 {
    // d = i / (1 + i) ✅
    let eff_d = eff_i / (1.0 + eff_i);
    eff_d
}

pub fn eff_i_to_nom_d(eff_i: f64, d_m: u32) -> f64 {
    let eff_d = eff_i_to_eff_d(eff_i);
    eff_d_to_nom_d(eff_d, d_m)
}

pub fn nom_i_to_eff_d(nom_i: f64, i_m: u32) -> f64 {
    let eff_i = nom_i_to_eff_i(nom_i, i_m);
    eff_i_to_eff_d(eff_i)
}

pub fn nom_i_to_nom_d(nom_i: f64, i_m: u32, d_m: u32) -> f64 {
    let eff_d = nom_i_to_eff_d(nom_i, i_m);
    eff_d_to_nom_d(eff_d, d_m)
}

// ====== From d to i======
pub fn eff_d_to_eff_i(eff_d: f64) -> f64 {
    // i = d / (1 - d)
    let eff_i = eff_d / (1.0 - eff_d);
    eff_i
}

pub fn eff_d_to_nom_i(eff_d: f64, i_m: u32) -> f64 {
    let eff_i = eff_d_to_eff_i(eff_d);
    eff_i_to_nom_i(eff_i, i_m)
}

pub fn nom_d_to_eff_i(nom_d: f64, d_m: u32) -> f64 {
    let eff_d = nom_d_to_eff_d(nom_d, d_m);
    eff_d_to_eff_i(eff_d)
}

pub fn nom_d_to_nom_i(nom_d: f64, d_m: u32, i_m: u32) -> f64 {
    let eff_i = nom_d_to_eff_i(nom_d, d_m);
    eff_i_to_nom_i(eff_i, i_m)
}
