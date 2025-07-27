use crate::mt_config::{AssumptionEnum, MortTableConfig};
use polars::prelude::*;

// =======================================
// PUBLIC FUNCTIONS
// =======================================
/// Calculate ₜpₓ: probability of surviving t years from age x (fractional ages supported).
/// ₖ|ₜp = ₖ₊ₜpₓ =  ∏ₖ₌₀^{t+k-1} (1 - qₓ₊ₖ₊ₜ) ✅
/// Uses UDD, CFM, or HPB formulas for fractional ages/times; delegates to whole ages if both are integers.
pub fn tpx(
    config: &MortTableConfig,
    x: f64,
    t: f64,
    k: f64,
    entry_age: Option<u32>,
) -> PolarsResult<f64> {
    // Decide if selected table is used
    let new_config = get_new_config_with_selected_table(config, entry_age)?;

    // Combine t and k
    let t = t + k;

    // Handle special case for whole numbers right at the start
    if x.fract() == 0.0 && t.fract() == 0.0 {
        return tpx_whole(&new_config, x as u32, t as u32);
    }

    // If not start to handle fractional ages
    let x_whole = x.floor() as u32; // n
    let x_frac = x.fract(); // s
    let time_to_next_age = 1.0 - x_frac; // always between 0 and 1

    // Get mortality rate for age n (percentage already applied in tqx function)
    let tqx = get_qx(&new_config, x_whole).unwrap_or(0.0);

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
        let survival_rate = match config.assumption {
            Some(AssumptionEnum::UDD) => 1.0 - t * tqx / (1.0 - x_frac * tqx),
            Some(AssumptionEnum::CFM) => (1.0 - tqx).powf(t),
            Some(AssumptionEnum::HPB) => 1.0 - t * tqx / (1.0 + x_frac * tqx),
            _ => {
                return Err(PolarsError::ComputeError(
                    "Unsupported assumption for fractional age".into(),
                ));
            }
        };
        Ok(survival_rate)
    } else {
        // Case 2b:  when t > (1-s) or t > time_to_next_age
        let survival_to_next_age = tpx(&new_config, x, time_to_next_age, 0.0, None)?;
        let remaining_time = t - time_to_next_age;
        let survival_after = tpx(&new_config, (x_whole + 1) as f64, remaining_time, 0.0, None)?;
        let result = survival_to_next_age * survival_after;
        Ok(result)
    }
}

/// Calculate ₜqₓ - probability of dying within t years starting at age x (fractional ages supported).
/// ₖ|ₜpₓ +  ₖ|ₜqₓ =  ₖpₓ ✅
pub fn tqx(
    config: &MortTableConfig,
    x: f64,
    t: f64,
    k: f64,
    entry_age: Option<u32>,
) -> PolarsResult<f64> {
    let kpx = tpx(config, x, k, 0.0, entry_age)?; // ₖpₓ
    let ktpx = tpx(config, x, t, k, entry_age)?; // ₖ|ₜpₓ
    let result = kpx - ktpx;
    Ok(result)
}

// =======================================
// PRIVATE FUNCTIONS
// =======================================
/// Calculate ₜpₓ: probability of surviving t years from age x (whole ages only).
///
/// Formula: ₜpₓ = ∏(k=0 to t-1) (1 - qₓ₊ₖ)
fn tpx_whole(config: &MortTableConfig, x: u32, t: u32) -> PolarsResult<f64> {
    let mut result = 1.0;
    for age in x..(x + t) {
        let tqx = get_qx(config, age)?;
        let px = 1.0 - tqx;
        result *= px;
    }

    Ok(result)
}

// --------------------------------------------------------------------

fn get_new_config_with_selected_table(
    config: &MortTableConfig,
    entry_age: Option<u32>,
) -> PolarsResult<MortTableConfig> {
    if !_is_table_layout_approved(config) {
        return Err(PolarsError::InvalidOperation(
            "Mortality table layout is not approved".into(),
        ));
    }

    // If entry age is Some, use selected table; otherwise, use ultimate table
    let selected_df = if let Some(age) = entry_age {
        _get_selected_mortality_table(config, age)?
    } else {
        _get_ultimate_mortality_table(config)?
    };

    // Create a new MortTableConfig with the modified DataFrame
    let mut new_config = config.clone();
    new_config.xml.tables[0].values = selected_df;

    Ok(new_config)
}

pub fn _is_table_layout_approved(config: &MortTableConfig) -> bool {
    // === Custom data ===
    let content_type = config.xml.content_classification.content_type.clone();
    if content_type == "Custom data" {
        return true;
    }

    // === SOA preset foramt===
    // Check table layout
    let approved_table_layouts = ["Aggregate", "Ultimate", "Select", "Select & Ultimate"];
    let key_words = config.xml.content_classification.key_words.clone();

    // Check if any keyword matches any approved table layout
    let tbl_layout_result = key_words.iter().any(|keyword| {
        approved_table_layouts
            .iter()
            .any(|layout| keyword == layout)
    });

    // Content type check
    let approved_content_types = vec![
        "ADB, AD&D",
        "Annuitant Mortality",
        "Claim Cost (in Disability)",
        "Claim Incidence",
        "Claim Termination",
        "CSO / CET",
        "Disability Recovery",
        "Disabled Lives Mortality",
        "Disability Incidence",
        "Group Life",
        "Healthy Lives Mortality",
        "Insured Lives Mortality",
        "Insured Lives Mortality - Ultimate",
        "Projection Scale",
        "Termination Voluntary",
        "Population Mortality",
    ];

    let content_type = config.xml.content_classification.content_type.clone();

    // Check if content type is in approved content types
    let content_type_result = approved_content_types
        .iter()
        .any(|approved_type| content_type == *approved_type);

    // Return result
    tbl_layout_result && content_type_result
}

fn _get_ultimate_mortality_table(config: &MortTableConfig) -> PolarsResult<DataFrame> {
    // If entry age is None, we will use the highest duration as ultimate rate
    let df = &config.xml.tables[0].values;

    // Check if the table has already been processed (no duration column)
    if !df.get_column_names().contains(&&"duration".into()) {
        // If already processed, just return the table as-is
        return Ok(df.clone());
    }

    let max_duration = df.column("duration")?.u32()?.max().unwrap();
    let value_column_name = df.get_column_names()[1].as_str();

    df.clone()
        .lazy()
        .filter(col("duration").eq(lit(max_duration)))
        .select([col("age"), col(value_column_name)])
        .collect()
}

fn _get_selected_mortality_table(
    config: &MortTableConfig,
    entry_age: u32,
) -> PolarsResult<DataFrame> {
    // If entry age is Some, we will generate a new mortality table
    let df = &config.xml.tables[0].values;

    let min_age = df.column("age")?.u32()?.min().unwrap();
    let max_age = df.column("age")?.u32()?.max().unwrap();
    let min_duration = df.column("duration")?.u32()?.min().unwrap();
    let max_duration = df.column("duration")?.u32()?.max().unwrap();
    let value_column_name = df.get_column_names()[1].as_str();

    // Entry age cannot be smaller than smallest age in table
    if entry_age < min_age {
        return Err(PolarsError::ComputeError(
            format!("Entry age {entry_age} cannot be less than minimum age {min_age}").into(),
        ));
    }

    // Form a new mortality table with axis as
    // entry age at  duration 0,
    // entry age + 1 at duration 1 , ...
    // entry age + t - 1 at duration t-1
    // entry age + t ultimate
    // Note: if there is no  duration 0, the smallest duration will be used
    let mut age_vec: Vec<u32> = Vec::new();
    let mut value_vec: Vec<f64> = Vec::new();

    let mut duration = min_duration;

    for age in entry_age..max_age {
        let value_column = df
            .clone()
            .lazy()
            .filter(col("age").eq(lit(age)))
            .filter(col("duration").eq(lit(duration)))
            .select([col(value_column_name)])
            .collect()?;

        let value = value_column
            .column(value_column_name)?
            .f64()?
            .get(0)
            .unwrap();

        age_vec.push(age);
        value_vec.push(value);

        duration = u32::min(duration + 1, max_duration); // Cap duration to max_duration
    }

    // Create a new DataFrame with the selected ages and tqx values
    let result = DataFrame::new(vec![
        Series::new("age".into(), age_vec).into_column(),
        Series::new(value_column_name.into(), value_vec).into_column(),
    ])?;

    Ok(result)
}

// --------------------------------------------------------------------

// Apply for 1D table
fn get_qx(config: &MortTableConfig, x: u32) -> PolarsResult<f64> {
    // Check if table contains column lx
    let df = &config.xml.tables[0].values;
    let has_lx = df.get_column_names().contains(&&"lx".into());
    let has_qx = df.get_column_names().contains(&&"qx".into());

    if has_lx {
        return _get_qx_from_lx(config, x);
    } else if has_qx {
        return _get_qx(config, x);
    } else {
        return Err(PolarsError::InvalidOperation(
            "Mortality table does not contain tqx or lx column".into(),
        ));
    }
}

fn _get_qx(config: &MortTableConfig, x: u32) -> PolarsResult<f64> {
    // Check if table contains column tqx
    let df = &config.xml.tables[0].values;

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

    let result = qx_value * config.pct.unwrap_or(1.0); // Apply pct multiplier
    Ok(result)
}

fn _get_qx_from_lx(config: &MortTableConfig, x: u32) -> PolarsResult<f64> {
    let df = &config.xml.tables[0].values;

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

    let result = (l_x - l_x_1) / l_x * config.pct.unwrap_or(1.0);
    Ok(result)
}

// ================================================
// UNIT TESTS
// ================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mt_config::{AssumptionEnum, MortTableConfig};
    use crate::xml::MortXML;
    use approx::assert_abs_diff_eq;

    #[test]
    fn test_survival_cm1_01() {
        // This is obtain from CM1 study package 2019 Chapter 15 The Life Table
        // Load ELT15 Female
        let am92_xml = MortXML::from_xlsx("data/elt15.xlsx", "female")
            .expect("Failed to load AM92 selected table");

        // Create MortTableConfig
        let config = MortTableConfig::builder()
            .xml(am92_xml)
            .assumption(AssumptionEnum::UDD)
            .build();

        // Calculate  ₀.₅p₅₈
        let answer = tpx(&config, 58.0, 0.5, 0.0, None).unwrap();
        let expected = 0.99670;
        assert_abs_diff_eq!(answer, expected, epsilon = 1e-6);
    }

    #[test]
    fn test_survival_cm1_02() {
        // This is obtain from CM1 study package 2019 Chapter 15 The Life Table
        // Load PFA92 table
        let pfa92_xml = MortXML::from_xlsx("data/pfa92c20.xlsx", "pfa92c20")
            .expect("Failed to load PFA92C20 table");

        // Create MortTableConfig
        let config = MortTableConfig::builder()
            .xml(pfa92_xml)
            .assumption(AssumptionEnum::CFM)
            .build();

        // Calculate  ₃p₆₂.₅
        let ans = tpx(&config, 62.5, 3.0, 0.0, None).unwrap();
        let expected = 0.988861;
        assert_abs_diff_eq!(ans, expected, epsilon = 1e-6);
    }

    #[test]
    fn test_survival_cm1_03() {
        // This is obtain from CM1 study package 2019 Chapter 15 The Life Table
        // Load PFA92 table
        let pfa92_xml = MortXML::from_xlsx("data/pfa92c20.xlsx", "pfa92c20")
            .expect("Failed to load PFA92C20 table");

        // Create MortTableConfig
        let config = MortTableConfig::builder()
            .xml(pfa92_xml)
            .assumption(AssumptionEnum::UDD)
            .build();

        // Calculate  ₃p₆₂.₅
        let ans = tpx(&config, 62.5, 3.0, 0.0, None).unwrap();
        let expected = 0.988863;
        assert_abs_diff_eq!(ans, expected, epsilon = 1e-6);
    }

    #[test]
    fn test_survival_cm1_04() {
        // This is obtain from CM1 study package 2019 Chapter 15 The Life Table
        // Load PFA92 table
        let am92_xml =
            MortXML::from_xlsx("data/am92.xlsx", "am92").expect("Failed to load AM92 table");

        // Create MortTableConfig
        let config = MortTableConfig::builder()
            .xml(am92_xml)
            .assumption(AssumptionEnum::UDD)
            .build();

        // Calculate  ₃p₆₂.₅
        let ans = tpx(&config, 42.0, 2.0, 0.0, Some(42)).unwrap();
        let expected = 0.997929;
        assert_abs_diff_eq!(ans, expected, epsilon = 1e-6);
    }

    #[test]
    fn test_survival_cm1_05() {
        // This is obtain from CM1 study package 2019 Chapter 15 The Life Table
        // Load PFA92 table
        let am92_xml =
            MortXML::from_xlsx("data/am92.xlsx", "am92").expect("Failed to load AM92 table");

        // Create MortTableConfig
        let config = MortTableConfig::builder()
            .xml(am92_xml)
            .assumption(AssumptionEnum::UDD)
            .build();

        // Calculate ₃q(₄₀)₊₁
        let ans = tqx(&config, 41.0, 3.0, 0.0, Some(40)).unwrap();
        let expected = 0.003270;
        assert_abs_diff_eq!(ans, expected, epsilon = 1e-6);
    }

    #[test]
    fn test_survival_cm1_06() {
        // This is obtain from CM1 study package 2019 Chapter 15 The Life Table
        // Load PFA92 table
        let am92_xml =
            MortXML::from_xlsx("data/am92.xlsx", "am92").expect("Failed to load AM92 table");

        // Create MortTableConfig
        let config = MortTableConfig::builder()
            .xml(am92_xml)
            .assumption(AssumptionEnum::UDD)
            .build();

        // Calculate ₂|q(₄₁)₊₁
        let ans = tqx(&config, 42.0, 1.0, 2.0, Some(41)).unwrap();
        let expected = 0.001324;
        assert_abs_diff_eq!(ans, expected, epsilon = 1e-6);
    }
}
