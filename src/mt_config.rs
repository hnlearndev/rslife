//! # Mortality Table Configuration (MortTableConfig)
//!
//! Configure, adjust, and generate actuarial mortality tables from XML or DataFrame sources.
//!
//! This module provides the `MortTableConfig` struct and related types for flexible, robust configuration of mortality tables, including:
//! - Data source selection (SOA XML, custom DataFrame)
//! - Population radix and rate scaling
//! - Interest rate and commutation function support
//! - Fractional age mortality assumptions (UDD, CFM, HPB)
//!
//! ## Quick Start
//! ```rust
//! # use rslife::prelude::*;
//! // Load a mortality table from SOA by ID
//! let data = MortData::from_soa_url_id(1704)?;
//! let config = MortTableConfig::builder().data(data).build()?;
//! // Config is ready for use with actuarial functions
//! println!("Config created with radix: {:?}", config.radix);
//! # RSLifeResult::Ok(())
//! ```
//!
//! ## Configuration Options
//! - **data**: Mortality data under struct [`MortData`]
//! - **radix**: Initial population size (e.g., 100,000)
//! - **pct**: Mortality rate multiplier (e.g., 1.0, 0.75)
//! - **int_rate**: Interest rate for commutation functions
//! - **assumption**: Fractional age mortality assumption (UDD, CFM, HPB)
//!
//! ## See Also
//! - [`crate::mt_config::soa_xml`] for XML parsing and table structure
//! - [`crate::single_life::benefits`] for insurance functions
//! - [`crate::single_life::annuities`] for annuity functions
//! - [`crate::fractional`] for fractional period calculations
//! - [`crate::annuities_certain`] for certain annuities

#![allow(non_snake_case)]

// Create a structure for the module
mod aga_xls;
mod ifoa_xls;
pub mod mt_data;
mod soa_xml;
mod spreadsheet_helpers;

// Declare the module for MortData
use self::mt_data::MortData;
use crate::RSLifeResult;
use crate::helpers::get_new_config_with_selected_table;
use crate::params::GetValueFunctionValidation;
use bon::bon;
use garde::Validate;
use polars::prelude::*;

// ===============================================
// MORTALITY ASSUMPTIONS
// ===============================================

/// Mortality assumptions for fractional age calculations.
///
/// Determines how mortality is distributed within age intervals, affecting
/// fractional survival probabilities ₜpₓ for time t at age x:
///
/// - **UDD**: ₜpₓ = 1 - t·qₓ (most common, conservative)
/// - **CFM**: ₜpₓ = (1-qₓ)ᵗ (constant force, mathematical convenience)
/// - **HPB**: ₜpₓ = (1-qₓ)/(1-(1-t)·qₓ) (hyperbolic, balanced approach)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssumptionEnum {
    /// Uniform Distribution of Deaths - most common assumption.
    UDD,

    /// Constant Force of Mortality - mathematical convenience.
    CFM,

    /// Hyperbolic (Balmer) - balanced between UDD and CFM.
    HPB,
}

// ===============================================
// MORTALITY ASSUMPTIONS
// ===============================================

/// Configuration for generating mortality tables with demographic and actuarial functions.
///
/// Generates mortality tables from XML data with configurable detail levels, from basic
/// rates to complete commutation functions for actuarial present value calculations.
///
/// See the documentation for MortTableConfig for detailed usage and examples.

#[derive(Debug, Clone, Validate)]
#[garde(allow_unvalidated)]
pub struct MortTableConfig {
    /// Source mortality data (must contain exactly one age-based table).
    pub data: MortData,

    /// Initial population size (radix). Common values: 100,000 (standard), 1,000,000 (precise).
    #[garde(range(min = 1))]
    pub radix: u32,

    /// Mortality rate multiplier. Examples: 1.0 (standard), 0.75 (preferred), 0.5 (reduced).
    #[garde(custom(validate_pct))]
    pub pct: f64,

    /// Mortality assumption for fractional ages (reserved for future implementation).
    pub assumption: AssumptionEnum,
}

/// Custom validation function for pct field
fn validate_pct(value: &f64, _context: &()) -> garde::Result {
    if *value <= 0.0 {
        return Err(garde::Error::new(
            "pct cannot be less than or equal to 0.0 as it would make mortality rate calculations meaningless",
        ));
    }
    Ok(())
}

#[bon]
impl MortTableConfig {
    #[builder]
    pub fn new(
        data: MortData,
        #[builder(default = 100_000)] radix: u32,
        #[builder(default = 1.0)] pct: f64,
        #[builder(default = AssumptionEnum::UDD)] assumption: AssumptionEnum,
    ) -> RSLifeResult<Self> {
        // Temporarily allow unvalidated data
        let config = MortTableConfig {
            data,
            radix,
            pct,
            assumption,
        };

        // Validate the configuration
        config
            .validate()
            .map_err(|err| Box::new(err) as Box<dyn std::error::Error>)?;

        // MortData contains raw data which is usually contains only lx or qx.
        // Convert data to include both lx and qx for future calculations
        let config = config.get_qx_lx_data_config()?;
        Ok(config)
    }

    pub fn min_age(&self) -> PolarsResult<u32> {
        // Get the minimum age from the dataframe
        let age_column = self.data.dataframe.column("age")?;
        let age_series = age_column.u32()?;
        age_series
            .iter()
            .flatten()
            .min()
            .ok_or_else(|| PolarsError::ComputeError("No age data available".into()))
    }

    pub fn max_age(&self) -> PolarsResult<u32> {
        // Get the maximum age from the dataframe
        let age_column = self.data.dataframe.column("age")?;
        let age_series = age_column.u32()?;
        age_series
            .iter()
            .flatten()
            .max()
            .ok_or_else(|| PolarsError::ComputeError("No age data available".into()))
    }

    // Alias for max_age
    pub fn omega(&self) -> PolarsResult<u32> {
        self.max_age()
    }

    pub fn min_duration(&self) -> PolarsResult<u32> {
        // Return error if duration column does not exist
        let has_duration = self
            .data
            .dataframe
            .get_column_names()
            .contains(&&"duration".into());

        if !has_duration {
            return Err(PolarsError::ColumnNotFound(
                "duration column not found".into(),
            ));
        }

        let duration_column = self.data.dataframe.column("duration")?;
        let duration_series = duration_column.u32()?;
        duration_series
            .iter()
            .flatten()
            .min()
            .ok_or_else(|| PolarsError::ComputeError("No duration data available".into()))
    }

    pub fn max_duration(&self) -> PolarsResult<u32> {
        // Return error if duration column does not exist
        let has_duration = self
            .data
            .dataframe
            .get_column_names()
            .contains(&&"duration".into());

        if !has_duration {
            return Err(PolarsError::ColumnNotFound(
                "duration column not found".into(),
            ));
        }

        let duration_column = self.data.dataframe.column("duration")?;
        let duration_series = duration_column.u32()?;
        duration_series
            .iter()
            .flatten()
            .max()
            .ok_or_else(|| PolarsError::ComputeError("No duration data available".into()))
    }

    #[builder]
    pub fn lx(
        &self,
        x: u32,
        entry_age: Option<u32>,
        #[builder(default = true)] validate: bool,
    ) -> RSLifeResult<f64> {
        get_value(self, "lx", x, entry_age, validate)
    }

    #[builder]
    pub fn qx(
        &self,
        x: u32,
        entry_age: Option<u32>,
        #[builder(default = true)] validate: bool,
    ) -> RSLifeResult<f64> {
        get_value(self, "qx", x, entry_age, validate)
    }

    #[builder]
    pub fn dx(
        &self,
        x: u32,
        entry_age: Option<u32>,
        #[builder(default = true)] validate: bool,
    ) -> RSLifeResult<f64> {
        let lx = get_value(self, "lx", x, entry_age, validate)?;

        // if x is max age in table, return lx (assume 100% mortality)
        let max_age = self.max_age()?;
        if x == max_age {
            return Ok(lx); // No deaths at max age
        }
        // dx = lx - lx₊₁
        let lx = get_value(self, "lx", x, entry_age, validate)?;

        // Either use entry_age to deduce duration (x+1), or use duration+1
        let lx_next = get_value(self, "lx", x + 1, entry_age, validate)?;

        Ok(lx - lx_next)
    }

    #[builder]
    pub fn px(
        &self,
        x: u32,
        entry_age: Option<u32>,
        #[builder(default = true)] validate: bool,
    ) -> RSLifeResult<f64> {
        // px = 1 - qx
        let qx = get_value(self, "qx", x, entry_age, validate)?;
        Ok(1.0 - qx)
    }

    fn get_qx_lx_data_config(&self) -> RSLifeResult<Self> {
        let df = self.data.dataframe.clone();
        let has_lx = df.get_column_names().contains(&&"lx".into());
        let has_qx = df.get_column_names().contains(&&"qx".into());
        let is_2d = df.get_column_names().contains(&&"duration".into());
        let radix = self.radix;
        let (min_dur, max_dur) = if is_2d {
            (self.min_duration()?, self.max_duration()?)
        } else {
            (0, 0) // dummy values, won't be used for 1D
        };

        let new_df = match (is_2d, has_lx, has_qx) {
            // 1D
            (false, true, false) => get_qx_from_lx_1D(df), // When lx is present, compute qx from lx
            (false, false, true) => get_lx_from_qx_1D(df, radix), // When qx is present, compute lx from qx
            // 2D
            (true, true, false) => get_qx_from_lx_2D(df, min_dur, max_dur), // When lx is present, compute qx from lx
            (true, false, true) => get_lx_from_qx_2D(df, min_dur, max_dur, radix), // When qx is present, compute lx from qx
            // If both lx and qx are present, return the DataFrame as-is
            (_, true, true) => Ok(df.clone()),
            _ => Err(PolarsError::ComputeError(
                "Mortality table format not recognized".into(),
            )),
        }?;

        let mut config = self.clone();
        config.data.dataframe = new_df;

        // Return the configured MortTableConfig
        Ok(config)
    }
}

// ================================================
// PRIVATE FUNCTIONS
// ================================================

fn get_value(
    mt: &MortTableConfig,
    column_name: &str,
    x: u32,
    entry_age: Option<u32>,
    validate: bool,
) -> RSLifeResult<f64> {
    // Validate the parameters
    if validate {
        let params = GetValueFunctionValidation {
            mt: mt.clone(),
            x,
            entry_age,
        };

        params
            .validate_all()
            .map_err(|err| Box::new(err) as Box<dyn std::error::Error>)?;
    }

    // Decide if selected table is used
    let mt = get_new_config_with_selected_table(mt, entry_age)?;
    let df = mt.data.dataframe;

    // Filter the DataFrame for the specified age
    let filtered_df = df.clone().lazy().filter(col("age").eq(lit(x))).collect()?;

    // Get the value from the specified column
    let series = filtered_df.column(column_name)?;
    let value = series
        .f64()?
        .get(0)
        .ok_or_else(|| PolarsError::ComputeError(format!("No value found for age {x}").into()))?;

    Ok(value)
}

// --------1D data----------

fn get_qx_from_lx_1D(df: DataFrame) -> PolarsResult<DataFrame> {
    let age_vec = df
        .column("age")?
        .u32()?
        .into_iter()
        .map(|opt| opt.unwrap())
        .collect::<Vec<u32>>();

    let lx_vec = df
        .column("lx")?
        .f64()?
        .into_iter()
        .map(|opt| opt.unwrap())
        .collect::<Vec<f64>>();

    let mut qx_vec: Vec<f64> = Vec::with_capacity(lx_vec.len());

    for i in 0..lx_vec.len() - 1 {
        let qx = (lx_vec[i] - lx_vec[i + 1]) / lx_vec[i];
        qx_vec.push(qx);
    }

    // At the last age, qx = 1.0 (certainty of death)
    qx_vec.push(1.0);

    let result = DataFrame::new(vec![
        Series::new("age".into(), age_vec).into_column(),
        Series::new("qx".into(), qx_vec).into_column(),
        Series::new("lx".into(), lx_vec).into_column(),
    ])?;

    Ok(result)
}

fn get_lx_from_qx_1D(df: DataFrame, radix: u32) -> PolarsResult<DataFrame> {
    // Convert to f64 for calculations - Keep u32 for input consistency
    let radix = f64::from(radix);

    let age_vec = df
        .column("age")?
        .u32()?
        .into_iter()
        .map(|opt| opt.unwrap())
        .collect::<Vec<u32>>();

    let qx_vec = df
        .column("qx")?
        .f64()?
        .into_iter()
        .map(|opt| opt.unwrap())
        .collect::<Vec<f64>>();

    let mut lx_vec: Vec<f64> = Vec::with_capacity(qx_vec.len());

    // Set l₀ = radix
    lx_vec.push(radix);

    // Compute lx for each age using the qx values and the previous lx
    for i in 1..qx_vec.len() {
        let lx = lx_vec[i - 1] * (1.0 - qx_vec[i - 1]);
        lx_vec.push(lx);
    }

    let result = DataFrame::new(vec![
        Series::new("age".into(), age_vec).into_column(),
        Series::new("qx".into(), qx_vec).into_column(),
        Series::new("lx".into(), lx_vec).into_column(),
    ])?;

    Ok(result)
}

// --------2D data----------

// --------
// Get qx from lx for 2D data
fn get_qx_from_lx_2D(df: DataFrame, min_dur: u32, max_dur: u32) -> PolarsResult<DataFrame> {
    let df = _pivot_2D_data(df, "lx")?;
    let df = _get_all_qx_columns_2D(df, min_dur, max_dur)?;
    let df = _unpivot_data_2D(df, min_dur, max_dur)?;
    Ok(df)
}

fn _get_all_qx_columns_2D(df: DataFrame, min_dur: u32, max_dur: u32) -> PolarsResult<DataFrame> {
    let mut df = df;

    // We already have lx for max_duration, so fill from max_duration-1 down to min_duration (inclusive)
    for duration in (min_dur..max_dur + 1).rev() {
        let qx_current_col_name = format!("qx_{duration}");
        let lx_current_col_name = format!("lx_{duration}");
        let lx_next_col_name = format!("lx_{}", u32::min(duration + 1, max_dur));

        let lx_current_series = df.column(&lx_current_col_name)?.f64()?;
        let lx_next_series = df.column(&lx_next_col_name)?.f64()?;

        // Calculate the current lx values
        let lx_current_vec: Vec<Option<f64>> = lx_current_series.into_iter().collect();
        let lx_next_vec: Vec<Option<f64>> = lx_next_series.into_iter().collect();
        let mut qx_current_values: Vec<Option<f64>> = vec![None; lx_next_vec.len()];

        // For each i, compute qx_current_vec[i] from  lx_current[i] from lx_next_vec[i+1] and
        // We can only compute from start to second-to-last element
        for i in 0..(lx_next_vec.len() - 1) {
            if let (Some(lx_next), Some(lx_current)) = (lx_next_vec[i + 1], lx_current_vec[i]) {
                let computed = 1.0 - lx_next / lx_current;
                qx_current_values[i] = Some(computed);
            }
        }

        // Add the new column to the DataFrame
        df.with_column(Series::new(qx_current_col_name.into(), qx_current_values))?;
    }

    Ok(df)
}

// --------
// Get lx from qx for 2D data
fn get_lx_from_qx_2D(
    df: DataFrame,
    min_dur: u32,
    max_dur: u32,
    radix: u32,
) -> PolarsResult<DataFrame> {
    let mut df = _pivot_2D_data(df, "qx")?;
    df = _get_lx_ultimate_2D(df, max_dur, radix)?;
    df = _get_all_lx_columns_2D(df, min_dur, max_dur)?;
    df = _unpivot_data_2D(df, min_dur, max_dur)?;
    Ok(df)
}

fn _get_lx_ultimate_2D(df: DataFrame, max_dur: u32, radix: u32) -> PolarsResult<DataFrame> {
    // Convert to f64 for calculations - Keep u32 for input consistency
    let radix = f64::from(radix);

    let qx_col_name = format!("qx_{max_dur}");
    let lx_col_name = format!("lx_{max_dur}");

    // Get the ultimate qx series and create a new series for ultimate lx
    let qx_ultimate_series = df.column(&qx_col_name)?;
    let qx_iter = qx_ultimate_series.f64()?.into_iter();
    let mut lx_values = Vec::with_capacity(qx_ultimate_series.len());
    let mut prev_lx = radix;
    for qx_opt in qx_iter {
        lx_values.push(prev_lx);
        if let Some(qx) = qx_opt {
            prev_lx *= 1.0 - qx;
        }
    }
    // Now lx_values.len() == qx_ultimate_series.len()
    let mut df = df;
    df.with_column(Series::new(lx_col_name.into(), lx_values))?;

    Ok(df)
}

fn _get_all_lx_columns_2D(df: DataFrame, min_dur: u32, max_dur: u32) -> PolarsResult<DataFrame> {
    let mut df = df;
    // We already have lx for max_duration, so fill from max_duration-1 down to min_duration (inclusive)
    for duration in (min_dur..max_dur).rev() {
        let qx_current_col_name = format!("qx_{duration}");
        let lx_current_col_name = format!("lx_{duration}");
        let lx_next_col_name = format!("lx_{}", duration + 1);

        let qx_current_series = df.column(&qx_current_col_name)?.f64()?;
        let lx_next_series = df.column(&lx_next_col_name)?.f64()?;

        // Calculate the current lx values
        let qx_vec: Vec<Option<f64>> = qx_current_series.into_iter().collect();
        let lx_next_vec: Vec<Option<f64>> = lx_next_series.into_iter().collect();
        let mut lx_current_values: Vec<Option<f64>> = vec![None; lx_next_vec.len()];
        // For each i, compute lx_current[i] from lx_next_vec[i+1] and qx_vec[i]
        // We can only compute from start to second-to-last element
        for i in 0..(lx_next_vec.len() - 1) {
            if let (Some(lx_next), Some(qx)) = (lx_next_vec[i + 1], qx_vec[i]) {
                let computed = lx_next / (1.0 - qx);
                lx_current_values[i] = Some(computed);
            }
        }

        // Add the new column to the DataFrame
        df.with_column(Series::new(
            lx_current_col_name.clone().into(),
            lx_current_values,
        ))?;
    }

    Ok(df)
}

fn _unpivot_data_2D(df: DataFrame, min_dur: u32, max_dur: u32) -> PolarsResult<DataFrame> {
    let mut long_lfs = Vec::new();

    let age_series = df
        .column("age")?
        .u32()?
        .into_no_null_iter()
        .collect::<Vec<_>>();

    // Loop over all durations
    for duration in min_dur..=max_dur {
        let qx_col = format!("qx_{duration}");
        let lx_col = format!("lx_{duration}");

        // Get qx and lx as Vec<Option<f64>>
        let qx_vec = df.column(&qx_col)?.f64()?.into_iter().collect::<Vec<_>>();

        let lx_vec = df.column(&lx_col)?.f64()?.into_iter().collect::<Vec<_>>();
        let duration_vec = vec![duration; age_series.len()];

        // Build new DataFrame for this duration
        let lf = DataFrame::new(vec![
            Series::new("age".into(), age_series.clone()).into_column(),
            Series::new("qx".into(), qx_vec).into_column(),
            Series::new("lx".into(), lx_vec).into_column(),
            Series::new("duration".into(), duration_vec).into_column(),
        ])?
        .lazy();

        long_lfs.push(lf);
    }

    // Concatenate all long DataFrames
    let unpivoted_df = concat(&long_lfs, Default::default())?.collect()?;

    // Unpitvot data
    Ok(unpivoted_df)
}

fn _pivot_2D_data(df: DataFrame, value_column: &str) -> PolarsResult<DataFrame> {
    // Get unique durations to create pivot columns dynamically
    let unique_durations: Vec<u32> = df
        .column("duration")?
        .u32()?
        .unique()?
        .into_iter()
        .flatten()
        .collect();

    // Create a list of aggregation expressions for each duration
    let mut agg_exprs = Vec::with_capacity(unique_durations.len());
    for duration in &unique_durations {
        let expr = col(value_column)
            .filter(col("duration").eq(lit(*duration)))
            .first()
            .alias(format!("{value_column}_{duration}"));
        agg_exprs.push(expr);
    }

    // Use group_by and agg to pivot the data
    df.clone()
        .lazy()
        .group_by([col("age")])
        .agg(agg_exprs)
        .sort(["age"], Default::default())
        .collect()
}

// ================================================
// UNIT TESTS
// ================================================
#[cfg(test)]
mod tests {
    use super::*;
    use crate::mt_config::MortTableConfig;
    use approx::assert_abs_diff_eq;

    #[test]
    fn test_fn_qx_01() {
        // Construct a MortTableConfig with Standard Ultimate Life Table (SULT) data
        let sult = MortData::from_soa_custom("SULT").expect("Failed to load SULT");
        let mt = MortTableConfig::builder()
            .data(sult)
            .radix(100_000)
            .build()
            .unwrap();

        let age = [20, 49, 100];
        let expected_qx = [0.000250, 0.001100, 0.289584]; // This is obtained from exam materails
        for (i, &a) in age.iter().enumerate() {
            let qx = mt.qx().x(a).call().unwrap();
            assert_abs_diff_eq!(qx, expected_qx[i], epsilon = 1e-6);
        }
    }

    #[test]
    fn test_fn_lx_01() {
        // Construct a MortTableConfig with Standard Ultimate Life Table (SULT) data
        let sult = MortData::from_soa_custom("SULT").expect("Failed to load SULT");
        let mt = MortTableConfig::builder()
            .data(sult)
            .radix(100_000)
            .build()
            .unwrap();

        let age = [20, 49, 100];
        let expected_lx = [100_000.0, 98_684.9, 6_248.2];
        for (i, &a) in age.iter().enumerate() {
            let lx = mt.lx().x(a).call().unwrap();
            assert_abs_diff_eq!(lx, expected_lx[i], epsilon = 1e-1);
        }
    }

    #[test]
    fn test_fn_lx_02() {
        // Construct a MortTableConfig with AM92 data
        let am92 = MortData::from_ifoa_url_id("AM92").expect("Failed to load AM92 selected table");
        let mt = MortTableConfig::builder()
            .data(am92)
            .radix(10_000)
            .build()
            .unwrap();
        // lx, age 62, entry age 60 = ultimate lx
        let ans = mt.lx().x(62).entry_age(60).call().unwrap();
        let expected = 9129.717;
        assert_abs_diff_eq!(ans, expected, epsilon = 1e-3);
    }

    #[test]
    fn test_fn_dx() {
        // Construct a MortTableConfig with AM92 data
        let am92 = MortData::from_ifoa_url_id("AM92").expect("Failed to load AM92 selected table");
        let mt = MortTableConfig::builder()
            .data(am92)
            .radix(10_000)
            .build()
            .unwrap();
        // dx, age 47, entry age 46
        let ans = mt.dx().x(47).entry_age(46).call().unwrap();
        let expected = 16.9517;
        assert_abs_diff_eq!(ans, expected, epsilon = 1e-4);
    }

    #[test]
    fn test_fn_get_value_2D_dat_01() {
        // Construct a MortTableConfig with AM92 data
        let am92 = MortData::from_ifoa_url_id("AM92").expect("Failed to load AM92 selected table");
        let mt = MortTableConfig::builder()
            .data(am92)
            .radix(10_000)
            .build()
            .unwrap();

        let ans = mt.lx().x(30).entry_age(29).call().unwrap();
        let expected = 9924.8916; // Expected lx value for age 30, duration 1
        assert_abs_diff_eq!(ans, expected, epsilon = 1e-4);
    }

    #[test]
    fn test_fn_get_value_2D_data_02() {
        // Construct a MortTableConfig with AM92 data
        let am92 = MortData::from_ifoa_url_id("AM92").expect("Failed to load AM92 selected table");
        let mt = MortTableConfig::builder()
            .data(am92)
            .radix(10_000)
            .build()
            .unwrap();

        let ans = mt.dx().x(44).entry_age(44).call().unwrap();
        let expected = 10.7533; // Expected lx value for age 30, duration 1
        assert_abs_diff_eq!(ans, expected, epsilon = 1e-4);
    }

    #[test]
    fn test_fn_get_value_2D_data_03() {
        // Construct a MortTableConfig with AM92 data
        let am92 = MortData::from_ifoa_url_id("AM92").expect("Failed to load AM92 selected table");
        let mt = MortTableConfig::builder()
            .data(am92)
            .radix(10_000)
            .build()
            .unwrap();

        // let ans = mt.qx().x(59).call().unwrap();
        let ans = mt.qx().x(59).entry_age(40).call().unwrap(); // This will use ultimate qx
        let expected = 0.007140; // Expected lx value for age 30, duration 1
        // assert_abs_diff_eq!(ans, ans_2, epsilon = 1e-4);
        assert_abs_diff_eq!(ans, expected, epsilon = 1e-4);
    }

    // -----------------------------------------------------
    // Test data print out
    #[test]
    fn test_fn_get_qx_lx_data() {
        let am92 = MortData::from_ifoa_url_id("AM92").expect("Failed to load AM92 selected table");
        let mt = MortTableConfig::builder()
            .data(am92)
            .radix(10_000)
            .build()
            .unwrap();
        let df = mt.data.dataframe.clone();
        println!("Sample qx/lx data: {:?}", df.head(Some(5)));
    }

    #[test]
    fn test_lx_from_qx_2d_demo() {
        let am92 = MortData::from_ifoa_url_id("AM92").expect("Failed to load AM92 selected table");
        let mt = MortTableConfig::builder()
            .data(am92)
            .radix(10_000)
            .build()
            .unwrap();
        let df = mt.data.dataframe.clone();
        let min_dur = mt.min_duration().unwrap();
        let max_dur = mt.max_duration().unwrap();
        let df_pivot = _pivot_2D_data(df.clone(), "qx").unwrap();
        println!("Step 1: _pivot_2D_data\n{}", df_pivot.head(Some(10)));
        let df1 = _get_lx_ultimate_2D(df_pivot.clone(), max_dur, mt.radix).unwrap();
        println!("\nStep 2: _get_lx_ultimate_2D\n{}", df1.head(Some(10)));
        let df2 = _get_all_lx_columns_2D(df1.clone(), min_dur, max_dur).unwrap();
        println!("\nStep 3: _get_all_lx_columns_2D\n{}", df2.head(Some(10)));
        let df3 = _unpivot_data_2D(df2.clone(), min_dur, max_dur).unwrap();
        println!("\nStep 4: _unpivot_data_2D\n{}", df3.head(Some(10)));
    }

    #[test]
    fn test_qx_from_lx_2d_demo() {
        let am92 = MortData::from_ifoa_url_id("AM92").expect("Failed to load AM92 selected table");
        let mt = MortTableConfig::builder()
            .data(am92)
            .radix(10_000)
            .build()
            .unwrap();
        let df = mt.data.dataframe.clone();
        let min_dur = mt.min_duration().unwrap();
        let max_dur = mt.max_duration().unwrap();
        let df_pivot = _pivot_2D_data(df.clone(), "lx").unwrap();
        println!("Step 1: _pivot_2D_data\n{}", df_pivot.head(Some(10)));
        let df1 = _get_all_qx_columns_2D(df_pivot.clone(), min_dur, max_dur).unwrap();
        println!("\nStep 2: _get_all_qx_columns_2D\n{}", df1.head(Some(10)));
        let df2 = _unpivot_data_2D(df1.clone(), min_dur, max_dur).unwrap();
        println!("\nStep 3: _unpivot_data_2D\n{}", df2.head(Some(10)));
    }
}
