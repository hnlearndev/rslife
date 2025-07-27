use crate::int_rate_convert::eff_i_to_nom_i;
use polars::prelude::*;

// Annuity-certain in arrears:
// ₜ| aₙ⁽ᵐ⁾ =  vᵗ . (1 - vⁿ) / i⁽ᵐ⁾  ✅
pub fn an(i: f64, n: u32, t: u32, m: u32) -> PolarsResult<f64> {
    if n == 0 {
        return Ok(0.0);
    }

    if m == 0 {
        return Err(PolarsError::ComputeError(
            "Number of payments per year must be greater than zero".into(),
        ));
    }

    let i_m = eff_i_to_nom_i(i, m);
    let v = 1.0 / (1.0 + i);
    let n_f64 = n as f64;
    let t_f64 = t as f64;

    let result = v.powf(t_f64) * (1.0 - v.powf(n_f64)) / i_m;
    Ok(result)
}

// Annuity-certain due:
// ₜ| äₙ⁽ᵐ⁾ = vᵗ · (1 - vⁿ) / d⁽ᵐ⁾ = ₜ| aₙ⁽ᵐ⁾ * i⁽ᵐ⁾ / d⁽ᵐ⁾  ✅
pub fn aan(i: f64, n: u32, t: u32, m: u32) -> PolarsResult<f64> {
    let d_m = eff_i_to_nom_i(i, m);
    let i_m = eff_i_to_nom_i(i, m);
    let an_value = an(i, n, t, m)?;
    let result = an_value * i_m / d_m;
    Ok(result)
}
