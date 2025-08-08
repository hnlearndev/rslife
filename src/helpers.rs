use crate::RSLifeResult;
use crate::mt_config::MortTableConfig;
use polars::prelude::*;

// ================================================
// PUBLIC FUNCTIONS
// ================================================

pub fn get_new_config_with_selected_table(
    mt: &MortTableConfig,
    entry_age: Option<u32>,
) -> RSLifeResult<MortTableConfig> {
    // If mortality table does not have a duration column, we can use it as-is
    let df = &mt.data.dataframe;

    if !df.get_column_names().contains(&&"duration".into()) {
        return Ok(mt.clone());
    }

    // If entry age is Some, use selected table; otherwise, use ultimate table
    let selected_df = if let Some(age) = entry_age {
        get_selected_mortality_table(mt, age)?
    } else {
        get_ultimate_mortality_table(mt)?
    };

    // Create a new MortTableConfig with the modified DataFrame
    let mut new_mt = mt.clone();
    new_mt.data.dataframe = selected_df;

    Ok(new_mt)
}

// ================================================
// PRIVATE FUNCTIONS
// ================================================

fn get_ultimate_mortality_table(mt: &MortTableConfig) -> RSLifeResult<DataFrame> {
    // If entry age is None, we will use the highest duration as ultimate rate
    // Also obtain lx along the way so the computation requires the values can be performed
    let df = &mt.data.dataframe;
    let max_duration = mt.max_duration()?;

    df.clone()
        .lazy()
        .filter(col("duration").eq(lit(max_duration)))
        .select([col("age"), col("qx"), col("lx")])
        .collect()
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
}

fn get_selected_mortality_table(mt: &MortTableConfig, entry_age: u32) -> RSLifeResult<DataFrame> {
    let df = &mt.data.dataframe;

    // If entry age is Some, we will generate a new mortality table
    let min_age = mt.min_age()?;
    let max_age = mt.max_age()?;
    let max_dur = mt.max_duration()?;

    // Form a new mortality table with axis as
    // min_age<= age < entry_age: set all rates to 0.0
    // entry age at  duration 0,
    // entry age + 1 at duration 1 , ...
    // entry age + t - 1 at duration t-1
    // entry age + t ultimate
    // Note: if there is no  duration 0, the smallest duration will be used
    let age_vec: Vec<u32> = (min_age..=max_age).collect();
    let mut qx_vec: Vec<f64> = vec![0.0; age_vec.len()];
    let mut lx_vec: Vec<f64> = vec![0.0; age_vec.len()];

    // Iterate from entry_age to max_age, incrementing duration
    for (i, age) in age_vec.iter().enumerate() {
        // Skip ages before entry_age, left as 0.0
        if *age < entry_age {
            continue;
        }

        let duration = u32::min(age - entry_age, max_dur);

        let get_value = |col_name: &str| -> f64 {
            df.clone()
                .lazy()
                .filter(col("age").eq(lit(*age)))
                .filter(col("duration").eq(lit(duration)))
                .select([col(col_name)])
                .collect()
                .unwrap()
                .column(col_name)
                .unwrap()
                .f64()
                .unwrap()
                .get(0)
                .unwrap()
        };

        let qx = get_value("qx");
        let lx = get_value("lx");

        qx_vec[i] = qx;
        lx_vec[i] = lx;
    }

    // Create a new DataFrame with the selected ages and tqx values (age as u32, values as f64)
    let result = DataFrame::new(vec![
        Series::new("age".into(), age_vec).into_column(),
        Series::new("qx".into(), qx_vec).into_column(),
        Series::new("lx".into(), lx_vec).into_column(),
    ])?;

    Ok(result)
}

// ================================================
// UNIT TESTS
// ================================================
