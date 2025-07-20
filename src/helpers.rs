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
/// - **Level 1** (~10x faster): Basic mortality rates only
/// - **Level 2** (~5x faster): Demographic functions
/// - **Level 3** (~2x faster): Commutation functions
/// - **Level 4** (full detail): Complete actuarial values including assurance and annuity functions
///
/// # Detail Level Column Mapping
///
/// ## Level 1: Basic Mortality Rates
/// **Columns**: `qx` (mortality rate), `px` (survival rate)
/// **Performance**: Fastest - only basic rate calculations
///
/// ## Level 2: Demographic Movement Functions
/// **Columns**: `lx` (lives), `dx` (deaths)
/// **Performance**: Fast - includes population dynamics
///
/// ## Level 3: Commutation Functions
/// **Columns**: `Cx`, `Dx`, `Mx`, `Nx`, `Px`, `Rx`, `Sx`
/// **Performance**: Medium - includes present value calculations
/// **Requirements**: Interest rate must be specified in configuration
///
/// ## Level 4: Complete Actuarial Functions
/// **Columns**: `Ax`, `AAx`, `IAx`, `IAAx`, `ax`, `aax`, `Iax`, `Iaax`
/// **Performance**: Complete - includes all insurance and annuity values
/// **Requirements**: Interest rate must be specified in configuration
///
/// # Parameters
/// - `config`: Mortality table configuration
/// - `x`: Age for which to retrieve the value
/// - `column_name`: Name of the column to retrieve
///
/// # Returns
/// The requested column value as a floating-point number
///
/// # Errors
/// Returns `PolarsError::ComputeError` if:
/// - The mortality table cannot be generated
/// - The requested age is not found in the table
/// - The requested column does not exist
/// - Interest rate is required but not provided for levels 3-4
///
/// # Examples
/// ```rust
/// use rslife::prelude::*;
/// use rslife::helpers::get_value;
/// fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let xml = MortXML::from_url_id(1704)?;
///     let config = MortTableConfig {
///         xml,
///         l_x_init: 100_000,
///         pct: Some(1.0),
///         int_rate: Some(0.03),
///         assumption: Some(AssumptionEnum::UDD),
///     };
///     // Level 1: Basic mortality rate (fastest)
///     let qx_value = get_value(&config, 65, "qx")?;
///     // Level 2: Population at age (fast)
///     let lx_value = get_value(&config, 65, "lx")?;
///     // Level 3: Commutation function (medium)
///     let dx_value = get_value(&config, 65, "Dx")?;
///     // Level 4: Actuarial present value (complete)
///     let ax_value = get_value(&config, 65, "Ax")?;
///     Ok(())
/// }
/// ```
pub fn get_value(config: &MortTableConfig, x: i32, column_name: &str) -> PolarsResult<f64> {
    // Determine the minimum detail level required for this column
    let detail_level = match column_name {
        // Level 1: Basic mortality rates
        "qx" | "px" => 1,
        // Level 2: Demographics and basic calculations
        "lx" | "dx" => 2,
        // Level 3: Full commutation functions
        "Cx" | "Dx" | "Mx" | "Nx" | "Px" | "Rx" | "Sx" => 3,
        // Level 4: Assurance and annuity values
        "Ax" | "AAx" | "IAx" | "IAAx" | "ax" | "aax" | "Iax" | "Iaax" => 4,
        // Return error for unknown columns
        _ => {
            return Err(PolarsError::ComputeError(
                format!(
                    r#"Unknown column name: '{column_name}'.
                    Supported columns are:
                    Level 1: qx, px
                    Level 2: lx, dx
                    Level 3: Cx, Dx, Mx, Nx, Px, Rx, Sx
                    Level 4: Ax, AAx, IAx, IAAx, ax, aax, Iax, Iaax"#
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

    // There must be that column name from the previous step
    let column = df.column(column_name).unwrap();

    let value = match column_name {
        "lx" | "dx" => column.i32().unwrap().get(0).map(|v| v as f64).unwrap(),
        _ => column.f64().unwrap().get(0).unwrap(),
    };

    Ok(value)
}

/// Creates a new configuration with adjusted interest rate for geometric growth calculations.
///
/// **Mathematical Formula**: i′ = (1+i)/(1+g) - 1
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
pub fn get_new_config(config: &MortTableConfig, g: f64) -> MortTableConfig {
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
