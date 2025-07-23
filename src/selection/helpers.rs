use super::*;

pub fn get_new_config_with_selected_table(
    config: &MortTableConfig,
    entry_age: i32,
) -> PolarsResult<MortTableConfig> {
    if !_is_table_layout_approved(config) {
        return Err(PolarsError::InvalidOperation(
            "Mortality table layout is not approved".into(),
        ));
    }

    let selected_df = _get_selected_mortality_table(config, entry_age)?;

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

fn _get_selected_mortality_table(
    config: &MortTableConfig,
    entry_age: i32,
) -> PolarsResult<DataFrame> {
    let df = &config.xml.tables[0].values;

    let min_age = df.column("age")?.i32()?.min().unwrap();
    let max_age = df.column("age")?.i32()?.max().unwrap();
    let min_duration = df.column("duration")?.i32()?.min().unwrap();
    let max_duration = df.column("duration")?.i32()?.max().unwrap();

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
    let mut age_vec = Vec::new();
    let mut qx_vec = Vec::new();

    let mut duration = min_duration;

    for age in entry_age..max_age {
        let qx_column = df
            .clone()
            .lazy()
            .filter(col("age").eq(lit(age)))
            .filter(col("duration").eq(lit(duration)))
            .select([col("value")])
            .collect()?;

        let qx = qx_column.column("value")?.f64()?.get(0).unwrap();

        age_vec.push(age);
        qx_vec.push(qx);

        duration = i32::min(duration + 1, max_duration); // Cap duration to max_duration
    }

    // Create a new DataFrame with the selected ages and qx values
    let result = DataFrame::new(vec![
        Series::new("age".into(), age_vec).into_column(),
        Series::new("value".into(), qx_vec).into_column(),
    ])?;

    Ok(result)
}
