use crate::mt_config::MortTableConfig;
use polars::prelude::*;

pub fn get_new_config_with_selected_table(
    mt: &MortTableConfig,
    entry_age: Option<u32>,
) -> Result<MortTableConfig, Box<dyn std::error::Error>> {
    // If entry age is Some, use selected table; otherwise, use ultimate table
    let selected_df = if let Some(age) = entry_age {
        _get_selected_mortality_table(mt, age)?
    } else {
        _get_ultimate_mortality_table(mt)?
    };

    // Create a new MortTableConfig with the modified DataFrame
    let mut new_mt = mt.clone();
    new_mt.data.dataframe = selected_df;

    Ok(new_mt)
}

fn _get_ultimate_mortality_table(mt: &MortTableConfig) -> PolarsResult<DataFrame> {
    // If entry age is None, we will use the highest duration as ultimate rate
    let df = &mt.data.dataframe;

    // Check if the table has already been processed (no duration column)
    if !df.get_column_names().contains(&&"duration".into()) {
        // If already processed, just return the table as-is
        return Ok(df.clone());
    }

    let max_duration = mt.max_duration();
    let value_column_name = df.get_column_names()[1].as_str();

    df.clone()
        .lazy()
        .filter(col("duration").eq(lit(max_duration)))
        .select([col("age"), col(value_column_name)])
        .collect()
}

fn _get_selected_mortality_table(mt: &MortTableConfig, entry_age: u32) -> PolarsResult<DataFrame> {
    // If entry age is Some, we will generate a new mortality table
    let df = &mt.data.dataframe;

    let max_age = mt.max_age() as u32;
    let min_duration = mt.min_duration() as u32;
    let max_duration = mt.max_duration() as u32;
    let value_column_name = df.get_column_names()[1].as_str();

    // Form a new mortality table with axis as
    // entry age at  duration 0,
    // entry age + 1 at duration 1 , ...
    // entry age + t - 1 at duration t-1
    // entry age + t ultimate
    // Note: if there is no  duration 0, the smallest duration will be used
    let mut age_vec: Vec<f64> = Vec::new();
    let mut value_vec: Vec<f64> = Vec::new();

    let mut duration = min_duration;

    // Iterate from entry_age to max_age, incrementing duration
    for age in entry_age..(max_age + 1) {
        // Filter the DataFrame to get the value for this age and duration
        let value = df
            .clone()
            .lazy()
            .filter(col("age").eq(lit(age as f64)))
            .filter(col("duration").eq(lit(duration as f64)))
            .select([col(value_column_name)])
            .collect()?
            .column(value_column_name)?
            .f64()?
            .get(0)
            .unwrap();

        age_vec.push(age as f64);
        value_vec.push(value);

        duration = u32::min(duration + 1, max_duration); // Cap duration to max_duration
    }

    // Create a new DataFrame with the selected ages and tqx values (both f64)
    let result = DataFrame::new(vec![
        Series::new("age".into(), age_vec).into_column(),
        Series::new(value_column_name.into(), value_vec).into_column(),
    ])?;

    Ok(result)
}
