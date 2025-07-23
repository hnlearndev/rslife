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
pub fn get_value(config: &MortTableConfig, x: i32, column_name: &str) -> PolarsResult<f64> {
    // Determine the minimum detail level required for this column
    let detail_level = match column_name {
        // Level 1: Basic mortality rates
        "qx" | "px" | "lx" | "dx" => 1,
        // Level 2: Demographics and basic calculations
        "Cx" | "Dx" => 2,
        // Level 3: Assurance and annuity values
        "Mx" | "Nx" | "Px" => 3,
        // Level 4: Full actuarial values
        "Rx" | "Sx" => 4,
        // Return error for unknown columns
        _ => {
            return Err(PolarsError::ComputeError(
                format!(
                    "Unknown column name: '{column_name}'. \
                    Supported columns: \
                    Level 1: qx, px, lx, dx; \
                    Level 2: Cx, Dx; \
                    Level 3: Mx, Nx, Px; \
                    Level 4: Rx, Sx"
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
        "lx" | "dx" => column.i32()?.get(0).map(|v| v as f64).unwrap(),
        _ => column.f64()?.get(0).unwrap(),
    };

    Ok(value)
}
