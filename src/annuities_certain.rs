use crate::int_rate_convert::eff_i_to_nom_i;
use bon::builder;

// Annuity-certain in arrears:
// ₜ| aₙ⁽ᵐ⁾ =  vᵗ . (1 - vⁿ) / i⁽ᵐ⁾  ✅
#[builder]
pub fn an(
    i: f64,
    n: u32,
    t: Option<u32>,
    m: Option<u32>,
) -> Result<f64, Box<dyn std::error::Error>> {
    if n == 0 {
        return Ok(0.0);
    }

    if m == Some(0) {
        return Err("Number of payments per year must be greater than zero".into());
    }

    let v = 1.0 / (1.0 + i);
    let n = n as f64;
    let t = t.unwrap_or(0) as f64; // Default to no deferral if not specified
    let m = m.unwrap_or(1); // Default to annual payments if not specified
    let i_m = eff_i_to_nom_i(i, m);

    let result = v.powf(t) * (1.0 - v.powf(n)) / i_m;
    Ok(result)
}

// Annuity-certain due:
// ₜ| äₙ⁽ᵐ⁾ = vᵗ · (1 - vⁿ) / d⁽ᵐ⁾ = ₜ| aₙ⁽ᵐ⁾ * i⁽ᵐ⁾ / d⁽ᵐ⁾  ✅
#[builder]
pub fn aan(
    i: f64,
    n: u32,
    t: Option<u32>,
    m: Option<u32>,
) -> Result<f64, Box<dyn std::error::Error>> {
    // Default values
    let t = t.unwrap_or(0); // Default to no deferral if not specified
    let m = m.unwrap_or(1); // Default to annual payments if not specified

    let d_m = eff_i_to_nom_i(i, m);
    let i_m = eff_i_to_nom_i(i, m);
    let an = an().i(i).n(n).t(t).m(m).call()?;
    let result = an * i_m / d_m;
    Ok(result)
}
