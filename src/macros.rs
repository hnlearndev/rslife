/// Macro to create a MortData from age/qx or age/lx vectors.
/// Usage:
/// ```rust
/// # use rslife::prelude::*;
/// let data = mddf! {
///     "age" => [25_u32, 26, 27],
///     "qx" => [0.001_f64, 0.0012, 0.0015],
/// };
/// # RSLifeResult::Ok(())
/// ```
#[macro_export]
macro_rules! mddf {
    ($($name:expr => $val:expr),+ $(,)?) => {{
        use $crate::mt_config::mt_data::MortData;
        use polars::prelude::df;
        let df_result = df! { $($name => $val),+ };
        match df_result {
            Ok(df) => MortData::from_df(df),
            Err(e) => Err(e.into()),
        }
    }};
}

// ================================================
// UNIT TESTS
// ================================================
#[cfg(test)]
mod tests {

    #[test]
    fn test_mddf_macro_with_qx() {
        let data = mddf! {
            "age" => [25_u32, 26, 27],
            "qx" => [0.001_f64, 0.0012, 0.0015],
        }
        .expect("Failed to create MortData from macro");

        let df = &data.dataframe;
        assert_eq!(df.get_column_names(), vec!["age", "qx"]);
        assert_eq!(df.height(), 3);
        assert!(df.column("qx").is_ok());
    }

    #[test]
    fn test_mddf_macro_with_lx() {
        let data = mddf! {
            "age" => [30_u32, 31, 32],
            "lx" => [10000.0_f64, 9990.0, 9980.0],
        }
        .expect("Failed to create MortData from macro");

        let df = &data.dataframe;
        assert_eq!(df.get_column_names(), vec!["age", "lx"]);
        assert_eq!(df.height(), 3);
        assert!(df.column("lx").is_ok());
    }
}
