use crate::actuarial::mort_tbl_config::MortTableConfig;
use polars::prelude::*;

// Helper function to check if a number is whole
pub fn is_whole_number(value: f64) -> bool {
    value.fract() == 0.0
}

// Helper function to get qx from the mortality table (with percentage already applied)
pub fn qx(config: &MortTableConfig, age: i32) -> PolarsResult<f64> {
    let df = &config.xml.tables[0].values;

    let df = df
        .clone()
        .lazy()
        .with_column(col("value") * lit(config.pct.unwrap_or(1.0)).alias("value"))
        .collect()?;

    let filtered = df
        .lazy()
        .filter(col("age").eq(lit(age)))
        .select([col("value")])
        .collect()?;

    // If there is exactly 1 value, return it
    match filtered.height() {
        1 => {
            // There will be a value, so unwrap safely
            let val = filtered
                .column("value")
                .unwrap()
                .f64()
                .unwrap()
                .get(0)
                .unwrap();
            Ok(val)
        }
        0 => {
            // If no value is found, return an error
            Err(PolarsError::ComputeError(
                format!("No qx value found for age {age}").into(),
            ))
        }
        _ => {
            // If multiple values are found, return an error
            Err(PolarsError::ComputeError(
                format!("Multiple qx values found for age {age}. Ambigous data.").into(),
            ))
        }
    }
}
