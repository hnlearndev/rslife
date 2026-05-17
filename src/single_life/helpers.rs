//! Shared helpers for single_life submodules.
//!
//! Provides DataFrame lookup utilities and mortality table configuration
//! helpers used by `survivals`, `commutations`, `annuities`, and `benefits`.

use crate::RSLifeResult;
use crate::mt_config::MortTableConfig;
use polars::prelude::*;

// ================================================
// PUBLIC (within single_life) FUNCTIONS
// ================================================

/// Filter a column value from the mortality table DataFrame based on age and optional duration.
///
/// - If the DataFrame has no `duration` column, filters by `age` only.
/// - If it has a `duration` column and `duration` is `Some(d)`, filters by both `age` and `duration`.
/// - If it has a `duration` column and `duration` is `None`, filters by `age` and the max duration value.
pub(super) fn get_value(
    mt: &MortTableConfig,
    age: u32,
    duration: Option<u32>,
    col_name: &str,
) -> RSLifeResult<f64> {
    let df = &mt.data.dataframe;
    let has_duration = df.get_column_names().contains(&&"duration".into());

    let lazy = df.clone().lazy().filter(col("age").eq(lit(age)));

    let lazy = if has_duration {
        let dur = match duration {
            Some(d) => d,
            None => mt.max_duration()?,
        };
        lazy.filter(col("duration").eq(lit(dur)))
    } else {
        lazy
    };

    let value = lazy
        .select([col(col_name)])
        .collect()?
        .column(col_name)?
        .f64()?
        .get(0)
        .ok_or_else(|| {
            PolarsError::ComputeError(format!("{} not found for age {}", col_name, age).into())
        })?;
    Ok(value)
}

/// Single-pass lookup of `(lx, lx_next, qx)` at age x.
///
/// Filters the mortality table DataFrame once by `age ∈ {x, x+1}` and returns:
/// - `lx`   at age `x`
/// - `lx_next` at age `x + 1` (returns `0.0` when `x + 1` is beyond the table)
/// - `qx`   at age `x`
///
/// Duration handling matches `get_value` (uses `mt.max_duration()` when present).
pub(super) fn get_lx_and_qx(mt: &MortTableConfig, x: u32) -> RSLifeResult<(f64, f64, f64)> {
    let df = &mt.data.dataframe;
    let has_duration = df.get_column_names().contains(&&"duration".into());

    let mut lazy = df
        .clone()
        .lazy()
        .filter(col("age").gt_eq(lit(x)).and(col("age").lt_eq(lit(x + 1))));

    if has_duration {
        lazy = lazy.filter(col("duration").eq(lit(mt.max_duration()?)));
    }

    let filtered = lazy.select([col("age"), col("lx"), col("qx")]).collect()?;
    let age_ca = filtered.column("age")?.u32()?;
    let lx_ca = filtered.column("lx")?.f64()?;
    let qx_ca = filtered.column("qx")?.f64()?;

    let mut lx_x: Option<f64> = None;
    let mut qx_x: Option<f64> = None;
    let mut lx_next: f64 = 0.0;

    for row in 0..filtered.height() {
        match age_ca.get(row) {
            Some(a) if a == x => {
                lx_x = lx_ca.get(row);
                qx_x = qx_ca.get(row);
            }
            Some(a) if a == x + 1 => {
                lx_next = lx_ca.get(row).unwrap_or(0.0);
            }
            _ => {}
        }
    }

    let lx_x = lx_x
        .ok_or_else(|| PolarsError::ComputeError(format!("lx not found for age {}", x).into()))?;
    let qx_x = qx_x
        .ok_or_else(|| PolarsError::ComputeError(format!("qx not found for age {}", x).into()))?;

    Ok((lx_x, lx_next, qx_x))
}

/// Get a new MortTableConfig with selected or ultimate mortality table.
///
/// - If the table has no `duration` column, returns the config as-is.
/// - If `entry_age` is `Some`, builds a selected table from that entry age.
/// - If `entry_age` is `None`, uses the ultimate table (max duration row).
pub(super) fn get_new_config_with_selected_table(
    mt: &MortTableConfig,
    entry_age: Option<u32>,
) -> PolarsResult<MortTableConfig> {
    let df = &mt.data.dataframe;

    if !df.get_column_names().contains(&&"duration".into()) {
        return Ok(mt.clone());
    }

    let selected_df = if let Some(age) = entry_age {
        get_selected_mortality_table(mt, age)?
    } else {
        get_ultimate_mortality_table(mt)?
    };

    let mut new_mt = mt.clone();
    new_mt.data.dataframe = selected_df;

    Ok(new_mt)
}

// ================================================
// PRIVATE FUNCTIONS
// ================================================

fn get_ultimate_mortality_table(mt: &MortTableConfig) -> PolarsResult<DataFrame> {
    let df = &mt.data.dataframe;
    let max_duration = mt.max_duration()?;

    df.clone()
        .lazy()
        .filter(col("duration").eq(lit(max_duration)))
        .select([col("age"), col("qx"), col("lx")])
        .collect()
}

fn get_selected_mortality_table(mt: &MortTableConfig, entry_age: u32) -> PolarsResult<DataFrame> {
    let df = &mt.data.dataframe;

    let min_age = mt.min_age()?;
    let max_age = mt.max_age()?;
    let max_dur = mt.max_duration()?;

    // Entry age below this has NO meaning in calculation and interpretation
    let start_age = u32::max(entry_age, min_age.saturating_sub(max_dur));
    let age_vec: Vec<u32> = (start_age..=max_age).collect();
    let mut qx_vec: Vec<f64> = vec![0.0; age_vec.len()];
    let mut lx_vec: Vec<f64> = vec![0.0; age_vec.len()];

    for (i, &age) in age_vec.iter().enumerate() {
        let duration = u32::min(age - entry_age, max_dur); // age runs from start
        let qx = get_value_2d(df, age, duration, "qx")?;
        let lx = get_value_2d(df, age, duration, "lx")?;
        qx_vec[i] = qx;
        lx_vec[i] = lx;
    }

    let height = age_vec.len();
    let columns = vec![
        Series::new("age".into(), age_vec).into_column(),
        Series::new("qx".into(), qx_vec).into_column(),
        Series::new("lx".into(), lx_vec).into_column(),
    ];
    let result = DataFrame::new(height, columns)?;

    Ok(result)
}

/// Filter a column value from a 2D DataFrame by age and duration.
/// Returns 0.0 if data not found (e.g., age below table's min_age).
fn get_value_2d(df: &DataFrame, age: u32, duration: u32, col_name: &str) -> PolarsResult<f64> {
    let result = df
        .clone()
        .lazy()
        .filter(col("age").eq(lit(age)))
        .filter(col("duration").eq(lit(duration)))
        .select([col(col_name)])
        .collect()?;

    if result.height() == 0 {
        return Ok(0.0);
    }

    result.column(col_name)?.f64()?.get(0).ok_or_else(|| {
        PolarsError::ComputeError(
            format!(
                "{} not found for age {} duration {}",
                col_name, age, duration
            )
            .into(),
        )
    })
}

// ================================================
// UNIT TESTS
// ================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mt_config::MortTableConfig;
    use crate::mt_config::mt_data::MortData;

    #[test]
    fn test_get_new_config_with_selected_table() {
        let am92 = MortData::from_builtin("AM92").expect("Failed to load AM92");
        let mt = MortTableConfig::builder().data(am92).build().unwrap();

        // With entry_age = 11, min_age = 17, the table still starts at age 15 as value below that is irrelevant for caluclation
        // The age boundaries of tables are used for validation purpose.
        let selected = get_new_config_with_selected_table(&mt, Some(11)).unwrap();
        println!(
            "Selected table (entry_age=40):\n{}",
            selected.data.dataframe
        );

        // With entry_age = None (ultimate)
        let ultimate = get_new_config_with_selected_table(&mt, None).unwrap();
        println!("Ultimate table:\n{}", ultimate.data.dataframe);
    }
}
