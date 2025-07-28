use crate::helpers::get_new_config_with_selected_table;
use crate::mt_config::{AssumptionEnum, MortTableConfig};
use crate::params::SurvivalFunctionParams;
use bon::builder;
use polars::prelude::*;

// =======================================
// PUBLIC FUNCTIONS
// =======================================
/// Calculate ₜpₓ: probability of surviving t years from age x (fractional ages supported).
/// ₖ|ₜp = ₖ₊ₜpₓ =  ∏ₖ₌₀^{t+k-1} (1 - qₓ₊ₖ₊ₜ) ✅
/// Uses UDD, CFM, or HPB formulas for fractional ages/times; delegates to whole ages if both are integers.

#[builder]
pub fn tpx(
    mt: &MortTableConfig,
    x: f64,
    t: Option<f64>,
    k: Option<f64>,
    entry_age: Option<u32>,
) -> Result<f64, Box<dyn std::error::Error>> {
    // Validate parameters
    let params = SurvivalFunctionParams {
        mt: mt.clone(),
        x,
        t,
        k,
        entry_age,
    };

    params
        .validate_all()
        .map_err(|err| Box::new(err) as Box<dyn std::error::Error>)?;

    // Decide if selected table is used
    let mt = get_new_config_with_selected_table(mt, entry_age)?;

    // Combine t and k
    let t = t.unwrap_or(1.0);
    let k = k.unwrap_or(0.0);
    let t = t + k;

    // Handle special case for whole numbers right at the start
    if x.fract() == 0.0 && t.fract() == 0.0 {
        return tpx_whole(&mt, x as u32, t as u32);
    }

    // If not start to handle fractional ages
    let x_whole = x.floor() as u32; // n
    let x_frac = x.fract(); // s
    let time_to_next_age = 1.0 - x_frac; // always between 0 and 1

    // Get mortality rate for age n (percentage already applied in tqx function)
    let tqx = get_qx(&mt, x_whole).unwrap_or(0.0);

    if t <= time_to_next_age {
        // Case 2a: when t ≤ (1-s) or t <= time_to_next_age
        // ------UDD------:
        // ₜqₓ₊ₛ = t · qₓ / (1 - s · qₓ)
        // ₜpₓ₊ₛ = 1 - t · qₓ / (1 - s · qₓ)
        // ------CFM------:
        // ₜpₓ₊ₛ = (1 - qₓ)ᵗ
        // ------HPB-------:
        // ₜqₓ₊ₛ = t · qₓ / (1 + s · qₓ)
        // ₜpₓ₊ₛ = 1 - t · qₓ / (1 + s · qₓ)
        let survival_rate = match mt.assumption {
            Some(AssumptionEnum::UDD) => 1.0 - t * tqx / (1.0 - x_frac * tqx),
            Some(AssumptionEnum::CFM) => (1.0 - tqx).powf(t),
            Some(AssumptionEnum::HPB) => 1.0 - t * tqx / (1.0 + x_frac * tqx),
            _ => {
                return Err("Unsupported assumption for fractional age".into());
            }
        };
        Ok(survival_rate)
    } else {
        // Case 2b:  when t > (1-s) or t > time_to_next_age
        // Calculate survival to next integer age using builder pattern, split into multiple lines
        let survival_to_next_age = tpx().mt(&mt).x(x).t(time_to_next_age).call()?;

        let remaining_time = t - time_to_next_age;

        let survival_after = tpx()
            .mt(&mt)
            .x((x_whole + 1) as f64)
            .t(remaining_time)
            .call()?;

        let result = survival_to_next_age * survival_after;
        Ok(result)
    }
}

/// Calculate ₜqₓ - probability of dying within t years starting at age x (fractional ages supported).
/// ₖ|ₜpₓ +  ₖ|ₜqₓ =  ₖpₓ ✅

#[builder]
pub fn tqx(
    mt: &MortTableConfig,
    x: f64,
    t: Option<f64>,
    k: Option<f64>,
    entry_age: Option<u32>,
) -> Result<f64, Box<dyn std::error::Error>> {
    // Default values for t and k
    let t = t.unwrap_or(1.0);
    let k = k.unwrap_or(0.0);

    // Decide if selected table is used
    let mt = get_new_config_with_selected_table(mt, entry_age)?;

    let kpx = tpx().mt(&mt).x(x).t(k).k(0.0).call()?; // ₖpₓ
    let ktpx = tpx().mt(&mt).x(x).t(t).k(k).call()?; // ₖ|ₜpₓ

    let result = kpx - ktpx;
    Ok(result)
}

// =======================================
// PRIVATE FUNCTIONS
// =======================================
/// Calculate ₜpₓ: probability of surviving t years from age x (whole ages only).
///
/// Formula: ₜpₓ = ∏(k=0 to t-1) (1 - qₓ₊ₖ)
fn tpx_whole(mt: &MortTableConfig, x: u32, t: u32) -> Result<f64, Box<dyn std::error::Error>> {
    let mut result = 1.0;
    for age in x..(x + t) {
        let tqx = get_qx(mt, age)?;
        let px = 1.0 - tqx;
        result *= px;
    }

    Ok(result)
}

// --------------------------------------------------------------------

// Apply for 1D table
fn get_qx(mt: &MortTableConfig, x: u32) -> PolarsResult<f64> {
    // Check if table contains column lx
    let df = &mt.data.dataframe;
    let has_lx = df.get_column_names().contains(&&"lx".into());
    let has_qx = df.get_column_names().contains(&&"qx".into());

    if has_lx {
        return _get_qx_from_lx(mt, x);
    } else if has_qx {
        return _get_qx(mt, x);
    } else {
        return Err(PolarsError::InvalidOperation(
            "Mortality table does not contain tqx or lx column".into(),
        ));
    }
}

fn _get_qx(mt: &MortTableConfig, x: u32) -> PolarsResult<f64> {
    // Check if table contains column tqx
    let df = &mt.data.dataframe;

    let filtered_df = df
        .clone()
        .lazy()
        .filter(col("age").eq(lit(x)))
        .select([col("qx")])
        .collect()?;

    // Check if any rows were found
    if filtered_df.height() == 0 {
        return Err(PolarsError::ComputeError(
            format!("Age {} not found in mortality table", x).into(),
        ));
    }

    let qx_value = filtered_df.column("qx")?.f64()?.get(0).unwrap();

    let result = qx_value * mt.pct.unwrap_or(1.0); // Apply pct multiplier
    Ok(result)
}

fn _get_qx_from_lx(mt: &MortTableConfig, x: u32) -> PolarsResult<f64> {
    let df = &mt.data.dataframe;

    let lx_filtered = df
        .clone()
        .lazy()
        .filter(col("age").eq(lit(x)))
        .select([col("lx")])
        .collect()?;

    if lx_filtered.height() == 0 {
        return Err(PolarsError::ComputeError(
            format!("Age {} not found in mortality table", x).into(),
        ));
    }

    let l_x = lx_filtered.column("lx")?.f64()?.get(0).unwrap();

    let lx1_filtered = df
        .clone()
        .lazy()
        .filter(col("age").eq(lit(x + 1)))
        .select([col("lx")])
        .collect()?;

    if lx1_filtered.height() == 0 {
        return Err(PolarsError::ComputeError(
            format!("Age {} not found in mortality table", x + 1).into(),
        ));
    }

    let l_x_1 = lx1_filtered.column("lx")?.f64()?.get(0).unwrap();

    let result = (l_x - l_x_1) / l_x * mt.pct.unwrap_or(1.0);
    Ok(result)
}

// ================================================
// UNIT TESTS
// ================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mt_config::{AssumptionEnum, MortTableConfig};
    use crate::mt_data::MortData;
    use approx::assert_abs_diff_eq;

    #[test]
    fn test_survival_cm1_01() {
        // This is obtain from CM1 study package 2019 Chapter 15 The Life Table
        // Load ELT15 Female
        let am92 = MortData::from_xlsx("data/elt15.xlsx", "female")
            .expect("Failed to load AM92 selected table");

        // Create MortTableConfig
        let mt = MortTableConfig::builder()
            .data(am92)
            .assumption(AssumptionEnum::UDD)
            .build();

        // Calculate  ₀.₅p₅₈
        let answer = tpx().mt(&mt).x(58.0).t(0.5).k(0.0).call().unwrap();
        let expected = 0.99670;
        assert_abs_diff_eq!(answer, expected, epsilon = 1e-6);
    }

    #[test]
    fn test_survival_cm1_02() {
        // This is obtain from CM1 study package 2019 Chapter 15 The Life Table
        // Load PFA92 table
        let pfa92 = MortData::from_xlsx("data/pfa92c20.xlsx", "pfa92c20")
            .expect("Failed to load PFA92C20 table");

        // Create MortTableConfig
        let mt = MortTableConfig::builder()
            .data(pfa92)
            .assumption(AssumptionEnum::CFM)
            .build();

        // Calculate  ₃p₆₂.₅
        let ans = tpx().mt(&mt).x(62.5).t(3.0).k(0.0).call().unwrap();
        let expected = 0.988861;
        assert_abs_diff_eq!(ans, expected, epsilon = 1e-6);
    }

    #[test]
    fn test_survival_cm1_03() {
        // This is obtain from CM1 study package 2019 Chapter 15 The Life Table
        // Load PFA92 table
        let pfa92 = MortData::from_xlsx("data/pfa92c20.xlsx", "pfa92c20")
            .expect("Failed to load PFA92C20 table");

        // Create MortTableConfig
        let mt = MortTableConfig::builder()
            .data(pfa92)
            .assumption(AssumptionEnum::UDD)
            .build();

        // Calculate  ₃p₆₂.₅
        let ans = tpx().mt(&mt).x(62.5).t(3.0).k(0.0).call().unwrap();
        let expected = 0.988863;
        assert_abs_diff_eq!(ans, expected, epsilon = 1e-6);
    }

    #[test]
    fn test_survival_cm1_04() {
        // This is obtain from CM1 study package 2019 Chapter 15 The Life Table
        // Load PFA92 table
        let am92 =
            MortData::from_xlsx("data/am92.xlsx", "am92").expect("Failed to load AM92 table");

        // Create MortTableConfig
        let config = MortTableConfig::builder()
            .data(am92)
            .assumption(AssumptionEnum::UDD)
            .build();

        // Calculate  ₃p₆₂.₅
        let ans = tpx()
            .mt(&config)
            .x(42.0)
            .t(2.0)
            .k(0.0)
            .entry_age(42)
            .call()
            .unwrap();
        let expected = 0.997929;
        assert_abs_diff_eq!(ans, expected, epsilon = 1e-6);
    }

    #[test]
    fn test_survival_cm1_05() {
        // This is obtain from CM1 study package 2019 Chapter 15 The Life Table
        // Load PFA92 table
        let am92_xml =
            MortData::from_xlsx("data/am92.xlsx", "am92").expect("Failed to load AM92 table");

        // Create MortTableConfig
        let mt = MortTableConfig::builder()
            .data(am92_xml)
            .assumption(AssumptionEnum::UDD)
            .build();

        // Calculate ₃q(₄₀)₊₁
        let ans = tqx()
            .mt(&mt)
            .x(41.0)
            .t(3.0)
            .k(0.0)
            .entry_age(40)
            .call()
            .unwrap();
        let expected = 0.003270;
        assert_abs_diff_eq!(ans, expected, epsilon = 1e-6);
    }

    #[test]
    fn test_survival_cm1_06() {
        // This is obtain from CM1 study package 2019 Chapter 15 The Life Table
        // Load PFA92 table
        let am92_xml =
            MortData::from_xlsx("data/am92.xlsx", "am92").expect("Failed to load AM92 table");

        // Create MortTableConfig
        let config = MortTableConfig::builder()
            .data(am92_xml)
            .assumption(AssumptionEnum::UDD)
            .build();

        // Calculate ₂|q(₄₁)₊₁
        let ans = tqx()
            .mt(&config)
            .x(42.0)
            .k(2.0)
            .entry_age(41)
            .call()
            .unwrap();
        let expected = 0.001324;
        assert_abs_diff_eq!(ans, expected, epsilon = 1e-6);
    }
}
