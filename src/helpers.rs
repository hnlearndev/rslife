use crate::mt_config::MortTableConfig;
use polars::prelude::*;

/// Retrieves a specific column value from the mortality table for a given age.
///
/// This function generates the mortality table and extracts the requested
/// column value for the specified age. It automatically determines the minimum
/// detail level required based on the column name to optimize performance.
///
/// # Performance Optimization
///
/// The function automatically selects the optimal detail level based on the column requested,
/// providing significant performance improvements by generating only the necessary calculations:
///
/// - **Level 1** (~10x faster): Basic demographic functions only
/// - **Level 2** (~5x faster): Basic commutation functions
/// - **Level 3** (~2x faster): Extended commutation functions
/// - **Level 4** (full detail): Complete commutation functions
///
/// # Detail Level Column Mapping
///
/// ## Level 1: Basic Demographic Functions
/// **Columns**: `qx` (mortality rate), `px` (survival rate), `lx` (lives), `dx` (deaths)
/// **Performance**: Fastest - basic life table calculations
/// **Requirements**: None
///
/// ## Level 2: Basic Commutation Functions
/// **Columns**: Level 1 plus `Cx`, `Dx`
/// **Performance**: Fast - includes present value of deaths and lives
/// **Requirements**: Interest rate must be specified in configuration
/// **Formulas**: Cₓ = vˣ⁺¹ · dₓ, Dₓ = vˣ · lₓ
///
/// ## Level 3: Extended Commutation Functions
/// **Columns**: Level 2 plus `Mx`, `Nx`, `Px`
/// **Performance**: Medium - includes cumulative commutation functions
/// **Requirements**: Interest rate must be specified in configuration
/// **Formulas**: Mₓ = Σ(k=x to ω) Cₖ, Nₓ = Σ(k=x to ω) Dₖ, Pₓ = Mₓ/Nₓ
///
/// ## Level 4: Complete Commutation Functions
/// **Columns**: Level 3 plus `Rx`, `Sx`
/// **Performance**: Complete - includes all standard commutation functions
/// **Requirements**: Interest rate must be specified in configuration
/// **Formulas**: Rₓ = Σ(k=x to ω) Mₖ, Sₓ = Σ(k=x to ω) Nₖ
///
/// # Parameters
/// - `config`: Mortality table configuration
/// - `x`: Age for which to retrieve the value
/// - `column_name`: Name of the column to retrieve
///
/// # Returns
/// The requested column value as a floating-point number
///
/// # Example
/// ```rust
/// use rslife::prelude::*;
/// use rslife::helpers::get_value;
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let xml = MortXML::from_url_id(1704)?;
/// let config = MortTableConfig {
///     xml,
///     radix: Some(100_000),
///     pct: Some(1.0),
///     int_rate: Some(0.03),
///     assumption: None,
/// };
///
/// // Get mortality rate for age 30 (Level 1 - fastest)
/// let qx_30 = get_value(&config, 30, "qx")?;
/// println!("Mortality rate at age 30: {:.6}", qx_30);
///
/// // Get commutation function for age 30 (Level 3 - requires interest rate)
/// let mx_30 = get_value(&config, 30, "Mx")?;
/// println!("Mx at age 30: {:.2}", mx_30);
/// # Ok(())
/// # }
/// ```
///
/// # Errors
/// Returns `PolarsError::ComputeError` if:
/// - The mortality table cannot be generated
/// - The requested age is not found in the table
/// - The requested column does not exist
/// - Interest rate is required but not provided for levels 2-4
pub fn get_value(config: &MortTableConfig, x: u32, column_name: &str) -> PolarsResult<f64> {
    // Determine the minimum detail level required for this column
    let detail_level = match column_name {
        // Level 1: Basic demographic functions
        "qx" | "px" | "lx" | "dx" => 1,
        // Level 2: Basic commutation functions
        "Cx" | "Dx" => 2,
        // Level 3: Extended commutation functions
        "Mx" | "Nx" | "Px" => 3,
        // Level 4: Complete commutation functions
        "Rx" | "Sx" => 4,
        // Return error for unknown columns
        _ => {
            return Err(PolarsError::ComputeError(
                format!(
                    r#"Unknown column name: '{column_name}'.
                        Supported columns:
                        Level 1: qx, px, lx, dx
                        Level 2: Cx, Dx
                        Level 3: Mx, Nx, Px
                        Level 4: Rx, Sx"#
                )
                .into(),
            ));
        }
    };

    let df = config
        .gen_mort_table(detail_level)?
        .lazy()
        .filter(col("age").eq(lit(x)))
        .select([col(column_name)])
        .collect()?;

    // Check if the age exists in the table
    if df.height() == 0 {
        return Err(PolarsError::ComputeError(
            format!("Age {x} not found in mortality table").into(),
        ));
    }

    // There must be that column name from the previous step
    let column = df.column(column_name).unwrap();

    // All columns except age are f64
    let value = column.f64()?.get(0).ok_or_else(|| {
        PolarsError::ComputeError(
            format!("No value found for column '{column_name}' at age {x}").into(),
        )
    })?;

    Ok(value)
}

pub fn get_new_config_geometric_functions(
    config: &MortTableConfig,
    g: f64,
) -> PolarsResult<MortTableConfig> {
    // Replace the effective interest rate with the adjusted one
    let i = config.int_rate.unwrap();
    let int_rate = (1.0 + i) / (1.0 + g) - 1.0;
    let mut new_config = config.clone();
    new_config.int_rate = Some(int_rate);
    Ok(new_config)
}

pub fn get_new_config_with_selected_table(
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

fn _is_table_layout_approved(config: &MortTableConfig) -> bool {
    // Check table layout
    let approved_table_layouts = ["Select", "Select & Ultimate"];
    let key_words = config.xml.content_classification.key_words.clone();

    // Check if any keyword matches any approved table layout
    key_words.iter().any(|keyword| {
        approved_table_layouts
            .iter()
            .any(|layout| keyword == layout)
    })
}

fn _get_ultimate_mortality_table(config: &MortTableConfig) -> PolarsResult<DataFrame> {
    // If entry age is None, we will use the highest duration as ultimate rate
    let df = &config.xml.tables[0].values;
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

    // Create a new DataFrame with the selected ages and qx values
    let result = DataFrame::new(vec![
        Series::new("age".into(), age_vec).into_column(),
        Series::new(value_column_name.into(), value_vec).into_column(),
    ])?;

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::xml::MortXML;

    #[test]
    fn test_get_value_basic_columns() {
        let xml = MortXML::from_url_id(1704).expect("Failed to load XML");
        let config = MortTableConfig {
            xml,
            radix: Some(100_000),
            pct: Some(1.0),
            int_rate: Some(0.03),
            assumption: None,
        };

        // Test Level 1 columns

        let qx_val = get_value(&config, 25, "qx").expect("Failed to get qx");
        assert!(qx_val > 0.0 && qx_val < 1.0, "qx should be a probability");

        let lx_val = get_value(&config, 25, "lx").expect("Failed to get lx");
        assert!(lx_val > 0.0, "lx should be positive");

        // Test Level 2 columns (requires interest rate)
        let cx_val = get_value(&config, 25, "Cx").expect("Failed to get Cx");
        assert!(cx_val > 0.0, "Cx should be positive");

        // Test Level 3 columns
        let mx_val = get_value(&config, 25, "Mx").expect("Failed to get Mx");
        assert!(mx_val > 0.0, "Mx should be positive");

        // Test Level 4 columns
        let rx_val = get_value(&config, 25, "Rx").expect("Failed to get Rx");
        assert!(rx_val > 0.0, "Rx should be positive");

        println!("✓ All get_value tests passed");
    }

    #[test]
    fn test_get_value_edge_cases() {
        let xml = MortXML::from_url_id(1704).expect("Failed to load XML");
        let config = MortTableConfig {
            xml,
            radix: Some(100_000),
            pct: Some(1.0),
            int_rate: Some(0.03),
            assumption: None,
        };

        // Test with an age that doesn't exist in the table
        let result = get_value(&config, 999, "qx");
        assert!(result.is_err(), "Should return error for non-existent age");
        let error_msg = result.unwrap_err().to_string();
        assert!(
            error_msg.contains("Age 999 not found"),
            "Error should mention age not found"
        );

        // Test level 2 without interest rate
        let config_no_interest = MortTableConfig {
            xml: MortXML::from_url_id(1704).expect("Failed to load XML"),
            radix: Some(100_000),
            pct: Some(1.0),
            int_rate: None, // No interest rate
            assumption: None,
        };

        let result = get_value(&config_no_interest, 30, "Cx");
        assert!(
            result.is_err(),
            "Should return error when interest rate is required but not provided"
        );
        let error_msg = result.unwrap_err().to_string();
        assert!(
            error_msg.contains("Interest rate is required"),
            "Error should mention interest rate requirement"
        );

        println!("✓ Edge case handling working correctly");
    }

    #[test]
    fn test_get_value_unknown_column() {
        let xml = MortXML::from_url_id(1704).expect("Failed to load XML");
        let config = MortTableConfig {
            xml,
            radix: Some(100_000),
            pct: Some(1.0),
            int_rate: Some(0.03),
            assumption: None,
        };

        let result = get_value(&config, 25, "unknown_column");
        assert!(result.is_err(), "Should return error for unknown column");

        let error_msg = result.unwrap_err().to_string();
        assert!(
            error_msg.contains("Unknown column name"),
            "Error should mention unknown column"
        );
        assert!(
            error_msg.contains("Level 1: qx, px, lx, dx"),
            "Error should list Level 1 columns"
        );
    }
}
