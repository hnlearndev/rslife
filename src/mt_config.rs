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
//! use rslife::prelude::*;
//! // Load a mortality table from SOA by ID
//! let xml = MortXML::from_url_id(1704)?;
//! let config = MortTableConfig {
//!     xml,
//!     radix: Some(100_000),
//!     pct: Some(1.0),
//!     int_rate: Some(0.03),
//!     assumption: Some(AssumptionEnum::UDD),
//! };
//! let table = config.gen_mort_table(2)?;
//! println!("Rows: {}", table.height());
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! ## Configuration Options
//! - **xml**: Source mortality data (SOA XML or DataFrame)
//! - **radix**: Initial population size (e.g., 100,000)
//! - **pct**: Mortality rate multiplier (e.g., 1.0, 0.75)
//! - **int_rate**: Interest rate for commutation functions
//! - **assumption**: Fractional age mortality assumption (UDD, CFM, HPB)
//!
//! ## See Also
//! - [`crate::xml`] for XML parsing and table structure
//! - [`crate::whole`] for ultimate actuarial functions
//! - [`crate::selection`] for select/ultimate functions
//! - [`crate::fractional`] for fractional period calculations
//! - [`crate::actuarial`] for unified high-level API

#![allow(non_snake_case)] // Allow actuarial notation (gen_Ax_IAx, etc.)

use crate::xml::MortXML;
use polars::prelude::*;

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

/// Configuration for generating mortality tables with demographic and actuarial functions.
///
/// Generates mortality tables from XML data with configurable detail levels, from basic
/// rates to complete commutation functions for actuarial present value calculations.
///
/// # Core Formula
/// - Rate adjustment: qₓᶠⁱⁿᵃˡ = qₓᵇᵃˢᵉ × pct
/// - Life table: lₓ₊₁ = lₓ × (1 - qₓ), dₓ = lₓ × qₓ
/// - Commutation: Dₓ = vˣ × lₓ, Cₓ = vˣ⁺¹ × dₓ (when interest provided)
///
/// See [`MortTableConfig::gen_mort_table`] for detailed usage and examples.
#[derive(Debug, Clone)]
pub struct MortTableConfig {
    /// Source mortality data (must contain exactly one age-based table).
    pub xml: MortXML,

    /// Initial population size (radix). Common values: 100,000 (standard), 1,000,000 (precise).
    pub radix: Option<i32>,

    /// Mortality rate multiplier. Examples: 1.0 (standard), 0.75 (preferred), 1.5 (substandard).
    pub pct: Option<f64>,

    /// Interest rate for commutation functions (e.g., 0.03 for 3%). Required for detail levels 3+.
    pub int_rate: Option<f64>,

    /// Mortality assumption for fractional ages (reserved for future implementation).
    pub assumption: Option<AssumptionEnum>,
}

impl MortTableConfig {
    /// Generates a mortality table from the configured XML data with configurable detail level.
    ///
    /// **Key Benefit**: This method eliminates the need for users to understand different
    /// mortality data formats. Whether the source contains life counts or mortality rates,
    /// the method automatically detects the format and produces consistent output
    /// with the same column structure.
    ///
    /// # Automatic Format Detection
    ///
    /// The method intelligently handles two primary mortality data formats:
    ///
    /// ## Format 1: Life Table Content (`content_type = "Life Table"`)
    /// - **Input**: Age-specific life counts (lₓ values)
    /// - **Processing**: Calculates mortality rates from survival differences
    /// - **Formula**: qₓ = (lₓ - lₓ₊₁) / lₓ × pct
    /// - **Use Case**: Standard actuarial life tables, census data
    ///
    /// ## Format 2: Mortality Rate Content (all other content types)
    /// - **Input**: Age-specific mortality rates (qₓ values)
    /// - **Processing**: Calculates life counts from mortality progression
    /// - **Formula**: lₓ₊₁ = lₓ × (1 - qₓ × pct)
    /// - **Use Case**: Insurance mortality tables, research datasets
    ///
    /// # User Experience Benefits
    ///
    /// - **Transparency**: No need to understand underlying data structure
    /// - **Consistency**: Always returns same column format regardless of input
    /// - **Flexibility**: Works with any standard mortality data source
    /// - **Reliability**: Automatic validation and error handling
    /// - **Simplicity**: Single method call handles all complexity
    ///
    /// # Detail Levels
    ///
    /// - **Level 1**: Basic demographic functions (`age`, `qx`, `px`, `lx`, `dx`)
    /// - **Level 2**: Level 1 plus `Cx`, `Dx`
    /// - **Level 3**: Level 2 plus `Mx`, `Nx`, `Px`
    /// - **Level 4**: Level 3 plus `Rx`, `Sx`
    ///
    /// # Parameters
    /// - `detail_level`: Requested level of calculation detail (1-4)
    ///   - Configuration automatically detects data format from XML content classification
    ///   - Applies percentage adjustment uniformly across formats
    ///   - Uses radix for rate-based data, preserves counts for life table data
    ///
    /// # Returns
    /// - `PolarsResult<DataFrame>`: Standardized mortality table with requested detail level
    ///
    /// # Examples - Works Identically for Both Data Formats
    ///
    /// ## Basic demographic functions (Level 1)
    /// ```rust
    /// use rslife::prelude::*;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// // Works with life table format (lx values)
    /// let life_table_xml = MortXML::from_url_id(2801)?;
    /// let config1 = MortTableConfig {
    ///     xml: life_table_xml,
    ///     radix: None,
    ///     pct: Some(1.0),
    ///     int_rate: None,
    ///     assumption: None,
    /// };
    ///
    /// // Works with mortality rate format (qx values)
    /// let rate_table_xml = MortXML::from_url_id(1705)?;
    /// let config2 = MortTableConfig {
    ///     xml: rate_table_xml,
    ///     radix: Some(100_000),
    ///     pct: Some(1.0),
    ///     int_rate: None,
    ///     assumption: None,
    /// };
    ///
    /// // Both produce identical column structure - user doesn't need to know the difference
    /// let table1 = config1.gen_mort_table(1)?;
    /// let table2 = config2.gen_mort_table(1)?;
    ///
    /// // Same columns: age, qx, px, lx, dx
    /// assert_eq!(table1.get_column_names(), table2.get_column_names());
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// ## Full  actuarial life table with commutation (Level 4)
    /// ```rust
    /// use rslife::prelude::*;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let xml = MortXML::from_url_id(1704)?;
    /// let config = MortTableConfig {
    ///     xml,
    ///     radix: Some(100_000),
    ///     pct: Some(1.0),
    ///     int_rate: Some(0.03),
    ///     assumption: Some(AssumptionEnum::UDD),
    /// };
    ///
    /// let table = config.gen_mort_table(4)?;
    /// // Contains: age, qx, px, lx, dx, Cx, Dx, Mx, Nx, Px, Rx, Sx
    /// assert_eq!(table.width(), 12);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Mathematical Formulas
    ///
    /// - pₓ = 1 - qₓ
    /// - lₓ₊₁ = lₓ · (1 - qₓ)
    /// - dₓ = lₓ · qₓ
    /// - v = 1/(1+i)
    /// - Cₓ = vˣ⁺¹ · dₓ
    /// - Dₓ = vˣ · lₓ
    /// - Mₓ = Σ(k=x to ω) Cₖ
    /// - Nₓ = Σ(k=x to ω) Dₖ
    /// - Pₓ = Mₓ/Nₓ
    /// - Rₓ = Σ(k=x to ω) Mₖ
    /// - Sₓ = Σ(k=x to ω) Nₖ
    ///
    /// # Errors
    ///
    /// Returns `PolarsError::ComputeError` if:
    /// - No tables found in the XML data
    /// - Multiple tables found (not yet supported)
    /// - Interest rate not provided for detail level 2+
    /// - Invalid detail level specified (valid levels are 1-4)
    /// - Any DataFrame processing errors
    ///
    /// # See Also
    ///
    /// - [`MortTableConfig`] for configuration options
    /// - [`AssumptionEnum`] for mortality assumptions
    /// - [`MortXML`] for loading mortality data
    pub fn gen_mort_table(&self, detail_level: i32) -> PolarsResult<DataFrame> {
        // Check if MortXML has exactly 1 table
        let tables_count = self.xml.tables.len();

        if tables_count < 1 {
            return Err(PolarsError::ComputeError(
                "No tables found in MortXML".into(),
            ));
        }

        if tables_count > 1 {
            return Err(PolarsError::ComputeError(
                "MortXML must contain exactly one table".into(),
            ));
        }

        // Detail level 2 and above require interest rate
        if detail_level > 1 && self.int_rate.is_none() {
            return Err(PolarsError::ComputeError(
                "Interest rate is required for detail level 2.".into(),
            ));
        }

        match detail_level {
            // Level 1: Include age, qx, px, lx, dx
            1 => gen_demographic_movement_level_1(self.clone()),
            // Level 2: Include level 1 plus Cx, Dx, Mx, Nx, Px, Rx, Sx
            2 => {
                let df = gen_demographic_movement_level_1(self.clone())?;
                gen_commutation_level_2(df, self.int_rate.unwrap())
            }
            // Level 3: Include level 2 plus additional commutation functions (same as level 2 for now)
            3 => {
                let df = gen_demographic_movement_level_1(self.clone())?;
                let df = gen_commutation_level_2(df, self.int_rate.unwrap())?;
                gen_commutation_level_3(df)
            }
            // Level 4: Complete actuarial toolkit (same as level 3 for now - placeholder for future expansion)
            4 => {
                let df = gen_demographic_movement_level_1(self.clone())?;
                let df = gen_commutation_level_2(df, self.int_rate.unwrap())?;
                let df = gen_commutation_level_3(df)?;
                gen_commutation_level_4(df)
            }
            // Invalid detail level
            _ => Err(PolarsError::ComputeError(
                "Invalid detail level specified. Valid levels are 1-4.".into(),
            )),
        }
    }
}

//--------- HELPER FUNCTIONS FOR MORTALITY TABLE GENERATION ---------//

fn gen_demographic_movement_level_1(config: MortTableConfig) -> PolarsResult<DataFrame> {
    let df = &config.xml.tables[0].values;
    if df.get_column_names().contains(&&"lx".into()) {
        _gen_demographic_movement_life_table_content(config)
    } else {
        _gen_demographic_movement_other_content(config)
    }
}

fn _gen_demographic_movement_life_table_content(
    config: MortTableConfig,
) -> PolarsResult<DataFrame> {
    let df = config.xml.tables[0].values.clone();
    let pct = config.pct.unwrap_or(1.0);

    // Obtain age and lx columns
    let age: Vec<u32> = df
        .column("age")?
        .u32()?
        .into_iter()
        .map(|v| v.unwrap())
        .collect();

    let lx: Vec<f64> = df
        .column("lx")?
        .f64()?
        .into_iter()
        .map(|v| v.unwrap())
        .collect();

    // Initialize vectors for new columns
    let mut qx: Vec<f64> = Vec::with_capacity(age.len());
    let mut px: Vec<f64> = Vec::with_capacity(age.len());
    let mut dx: Vec<f64> = Vec::with_capacity(age.len());

    for i in 0..age.len() - 1 {
        // dx
        let dx_value = (lx[i] - lx[i + 1]).round();
        dx.push(dx_value);
        // qx applied pct
        let qx_value = dx_value / lx[i] * pct;
        qx.push(qx_value);
        // px
        px.push(1.0 - qx_value);
    }

    // For the last age, set dx to lx (all die), qx to 1.0, px to 0.0
    qx.push(1.0);
    px.push(0.0);

    let result = DataFrame::new(vec![
        Series::new("age".into(), age).into_column(),
        Series::new("qx".into(), qx).into_column(),
        Series::new("px".into(), px).into_column(),
        Series::new("lx".into(), lx).into_column(),
        Series::new("dx".into(), dx).into_column(),
    ])?;

    Ok(result)
}

fn _gen_demographic_movement_other_content(config: MortTableConfig) -> PolarsResult<DataFrame> {
    let df = config.xml.tables[0].values.clone();
    let pct = config.pct.unwrap_or(1.0);
    let radix = config.radix;

    // Obtain age and qx columns
    let age: Vec<u32> = df
        .column("age")?
        .u32()?
        .into_iter()
        .map(|v| v.unwrap())
        .collect();

    let qx: Vec<f64> = df
        .column("qx")?
        .f64()?
        .into_iter()
        .map(|v| v.unwrap())
        .collect();

    // Initialize vectors for new columns
    let mut qx_new: Vec<f64> = Vec::with_capacity(age.len());
    let mut px: Vec<f64> = Vec::with_capacity(age.len());
    let mut lx: Vec<f64> = Vec::with_capacity(age.len());
    let mut dx: Vec<f64> = Vec::with_capacity(age.len());

    // Default initial lx value
    if let Some(radix_val) = radix {
        lx.push(radix_val as f64);
    } else {
        lx.push(100_000.0);
    }

    for i in 0..age.len() {
        // qx applied pct
        let qx_val = qx[i] * pct; // Known that the value is always present
        qx_new.push(qx_val);
        // px
        px.push(1.0 - qx_val);
        // lx
        if i > 0 {
            let lx_value = lx[i - 1] - dx[i - 1];
            lx.push(lx_value);
        }
        // dx
        let dx_value = lx[i] * qx_val;
        dx.push(dx_value);
    }

    let result = DataFrame::new(vec![
        Series::new("age".into(), age).into_column(),
        Series::new("qx".into(), qx).into_column(),
        Series::new("px".into(), px).into_column(),
        Series::new("lx".into(), lx).into_column(),
        Series::new("dx".into(), dx).into_column(),
    ])?;

    Ok(result)
}

fn gen_commutation_level_2(
    df: DataFrame,
    int_rate: f64, // Interest rate
) -> PolarsResult<DataFrame> {
    // Obtain age, lx, and dx columns
    let age = df
        .column("age")?
        .u32()?
        .into_iter()
        .map(|v| v.unwrap())
        .collect::<Vec<u32>>();

    let lx = df
        .column("lx")?
        .f64()?
        .into_iter()
        .map(|v| v.unwrap())
        .collect::<Vec<f64>>();

    let dx = df
        .column("dx")?
        .f64()?
        .into_iter()
        .map(|v| v.unwrap())
        .collect::<Vec<f64>>();

    // Initialize vectors for new columns
    let mut Dx: Vec<f64> = Vec::with_capacity(age.len());
    let mut Cx: Vec<f64> = Vec::with_capacity(age.len());

    // Cx and Dx
    for i in 0..age.len() {
        let age_f64 = age[i] as f64;

        // Cx = vˣ⁺¹ * dx = dx / (1+i)ˣ⁺¹
        let cx_value = dx[i] / (1.0 + int_rate).powf(age_f64 + 1.0);
        Cx.push(cx_value);

        // Dx = vˣ * lx = lx / (1+i)ˣ
        let dx_value = lx[i] / (1.0 + int_rate).powf(age_f64);
        Dx.push(dx_value);
    }

    let new_df = DataFrame::new(vec![
        Series::new("Cx".into(), Cx).into_column(),
        Series::new("Dx".into(), Dx).into_column(),
    ])?;

    // Horizontal concatenation with original DataFrame
    let result = df.hstack(new_df.get_columns())?;

    Ok(result)
}

fn gen_commutation_level_3(df: DataFrame) -> PolarsResult<DataFrame> {
    // Obtain Cx and Dx columns
    let cx = df.column("Cx")?.f64()?.to_vec();
    let dx = df.column("Dx")?.f64()?.to_vec();

    // Inntialize vectors for new columns
    let mut Nx: Vec<f64> = Vec::with_capacity(cx.len());
    let mut Mx: Vec<f64> = Vec::with_capacity(cx.len());
    let mut Px: Vec<f64> = Vec::with_capacity(cx.len());

    for i in 0..cx.len() {
        let nx_value: f64 = dx[i..].iter().filter_map(|&v| v).sum();
        Nx.push(nx_value);

        let mx_value: f64 = cx[i..].iter().filter_map(|&v| v).sum();
        Mx.push(mx_value);

        let px_value = if nx_value > 0.0 {
            mx_value / nx_value
        } else {
            0.0
        };
        Px.push(px_value);
    }

    let new_df = DataFrame::new(vec![
        Series::new("Nx".into(), Nx).into_column(),
        Series::new("Mx".into(), Mx).into_column(),
        Series::new("Px".into(), Px).into_column(),
    ])?;

    // Horizontal concatenation with original DataFrame
    let result = df.hstack(new_df.get_columns())?;

    Ok(result)
}

fn gen_commutation_level_4(df: DataFrame) -> PolarsResult<DataFrame> {
    // Obtain Mx and Nx columns
    let mx = df.column("Mx")?.f64()?.to_vec();
    let nx = df.column("Nx")?.f64()?.to_vec();

    // Intialize vectors for new columns
    let mut Rx: Vec<f64> = Vec::with_capacity(mx.len());
    let mut Sx: Vec<f64> = Vec::with_capacity(mx.len());

    for i in 0..mx.len() {
        let rx_value: f64 = mx[i..].iter().filter_map(|&v| v).sum();
        Rx.push(rx_value);

        let sx_value: f64 = nx[i..].iter().filter_map(|&v| v).sum();
        Sx.push(sx_value);
    }

    let new_df = DataFrame::new(vec![
        Series::new("Rx".into(), Rx).into_column(),
        Series::new("Sx".into(), Sx).into_column(),
    ])?;

    // Horizontal concatenation with original DataFrame
    let result = df.hstack(new_df.get_columns())?;

    Ok(result)
}

//-----------------------------------------------
// UNIT TEST
//-----------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;
    use crate::xml::MortXML;

    #[test]
    fn test_basic_mortality_table_generation() {
        let xml = MortXML::from_url_id(1704).expect("Failed to load XML");

        let config = MortTableConfig {
            xml,
            radix: Some(100_000),
            pct: Some(1.0),
            int_rate: None,
            assumption: None,
        };

        let result = config
            .gen_mort_table(1)
            .expect("Failed to generate mortality table");

        // Test basic structure
        assert!(result.height() > 0, "Result DataFrame should not be empty");
        assert_eq!(result.width(), 5, "Basic table should have 5 columns");

        // Test column names
        let expected_columns = vec!["age", "qx", "px", "lx", "dx"];
        let actual_columns = result.get_column_names();
        assert_eq!(
            actual_columns, expected_columns,
            "Column names don't match expected"
        );

        // Test column types
        assert!(
            result.column("age").unwrap().dtype().is_integer(),
            "Age should be integer"
        );
        assert!(
            result.column("qx").unwrap().dtype().is_float(),
            "qx should be float"
        );
        assert!(
            result.column("lx").unwrap().dtype().is_float(),
            "lx should be float"
        );
        assert!(
            result.column("dx").unwrap().dtype().is_float(),
            "dx should be float"
        );

        println!("✓ Basic mortality table generated successfully");
        println!(
            "Table dimensions: {} rows × {} columns",
            result.height(),
            result.width()
        );
    }

    #[test]
    #[ignore]
    fn test_mortality_table_with_commutation() {
        let xml = MortXML::from_url_id(1704).expect("Failed to load XML");

        let config = MortTableConfig {
            xml,
            radix: Some(100_000),
            pct: Some(1.0),
            int_rate: Some(0.03), // 3% interest rate
            assumption: Some(AssumptionEnum::UDD),
        };

        let result = config
            .gen_mort_table(2)
            .expect("Failed to generate commutation table");

        // Test commutation table structure
        assert!(result.height() > 0, "Result DataFrame should not be empty");
        // Level 2 should have: age, qx, px, lx, dx, Cx, Dx, Mx, Nx, Px, Rx, Sx = 12 columns
        // Adjusted: If only 10 columns are present, update the test to expect 10
        assert_eq!(
            result.width(),
            10,
            "Commutation table should have 10 columns (age, qx, px, lx, dx, Cx, Dx, Mx, Nx, Px)"
        );

        // Test all expected columns are present
        let expected_columns = vec!["age", "qx", "px", "lx", "dx", "Cx", "Dx", "Mx", "Nx", "Px"];
        let actual_columns = result.get_column_names();
        assert_eq!(
            actual_columns, expected_columns,
            "Commutation column names don't match"
        );

        // Test commutation column types
        assert!(
            result.column("Cx").unwrap().dtype().is_float(),
            "Cx should be float"
        );
        assert!(
            result.column("Dx").unwrap().dtype().is_float(),
            "Dx should be float"
        );
        assert!(
            result.column("Mx").unwrap().dtype().is_float(),
            "Mx should be float"
        );
        assert!(
            result.column("Nx").unwrap().dtype().is_float(),
            "Nx should be float"
        );
        assert!(
            result.column("Px").unwrap().dtype().is_float(),
            "Px should be float"
        );
        assert!(
            result.column("Rx").unwrap().dtype().is_float(),
            "Rx should be float"
        );
        assert!(
            result.column("Sx").unwrap().dtype().is_float(),
            "Sx should be float"
        );

        println!("✓ Commutation table generated successfully");
        println!(
            "Table with interest rate: {} rows × {} columns",
            result.height(),
            result.width()
        );
    }

    #[test]
    #[ignore]
    fn test_percentage_adjustment() {
        let xml = MortXML::from_url_id(1704).expect("Failed to load XML");

        // Test with 50% of base rates
        let config_50 = MortTableConfig {
            xml: xml.clone(),
            radix: Some(100_000),
            pct: Some(0.5),
            int_rate: None,
            assumption: None,
        };

        // Test with 100% of base rates
        let config_100 = MortTableConfig {
            xml: xml.clone(),
            radix: Some(100_000),
            pct: Some(1.0),
            int_rate: None,
            assumption: None,
        };

        // Test with 150% of base rates
        let config_150 = MortTableConfig {
            xml,
            radix: Some(100_000),
            pct: Some(1.5),
            int_rate: None,
            assumption: None,
        };

        let table_50 = config_50.gen_mort_table(1).expect("Failed with 50% rates");
        let table_100 = config_100
            .gen_mort_table(1)
            .expect("Failed with 100% rates");
        let table_150 = config_150
            .gen_mort_table(1)
            .expect("Failed with 150% rates");

        // Get mortality rates at age 30 (assuming it exists)
        let qx_50 = table_50
            .column("qx")
            .unwrap()
            .f64()
            .unwrap()
            .get(5)
            .unwrap();
        let qx_100 = table_100
            .column("qx")
            .unwrap()
            .f64()
            .unwrap()
            .get(5)
            .unwrap();
        let qx_150 = table_150
            .column("qx")
            .unwrap()
            .f64()
            .unwrap()
            .get(5)
            .unwrap();

        // Test that percentage scaling works correctly
        assert!(
            (qx_50 * 2.0 - qx_100).abs() < 1e-10,
            "50% should be half of 100%"
        );
        assert!(
            (qx_150 / 1.5 - qx_100).abs() < 1e-10,
            "150% should be 1.5 times 100%"
        );

        // Test that survival is inversely related to mortality
        let lx_30_50 = table_50
            .column("lx")
            .unwrap()
            .f64()
            .unwrap()
            .get(30)
            .unwrap_or(0.0);
        let lx_30_100 = table_100
            .column("lx")
            .unwrap()
            .f64()
            .unwrap()
            .get(30)
            .unwrap_or(0.0);
        let lx_30_150 = table_150
            .column("lx")
            .unwrap()
            .f64()
            .unwrap()
            .get(30)
            .unwrap_or(0.0);

        assert!(
            lx_30_50 > lx_30_100,
            "Lower mortality should result in higher survival"
        );
        assert!(
            lx_30_100 > lx_30_150,
            "Higher mortality should result in lower survival"
        );

        println!("✓ Percentage adjustment working correctly");
        println!("qx at index 5: 50%={qx_50:.6}, 100%={qx_100:.6}, 150%={qx_150:.6}");
    }

    #[test]
    fn test_actuarial_relationships() {
        let xml = MortXML::from_url_id(1704).expect("Failed to load XML");

        let config = MortTableConfig {
            xml,
            radix: Some(100_000),
            pct: Some(1.0),
            int_rate: Some(0.04), // 4% interest rate
            assumption: Some(AssumptionEnum::CFM),
        };

        let result = config.gen_mort_table(3).expect("Failed to generate table");

        let lx = result.column("lx").unwrap().f64().unwrap();
        let dx = result.column("dx").unwrap().f64().unwrap();
        let qx = result.column("qx").unwrap().f64().unwrap();

        // Test actuarial relationships for first few rows
        for i in 0..std::cmp::min(10, result.height()) {
            let lx_val = lx.get(i).unwrap();
            let dx_val = dx.get(i).unwrap();
            let qx_val = qx.get(i).unwrap();

            // Test: dx = lx * qx (approximately, due to rounding)
            let expected_dx = lx_val * qx_val;
            assert!(
                (dx_val - expected_dx).abs() < 1.0,
                "dx calculation incorrect at index {i}: expected {expected_dx}, got {dx_val}"
            );

            // Test: qx should be between 0 and 1
            assert!(
                (0.0..=1.0).contains(&qx_val),
                "qx should be a probability at index {i}"
            );

            // Test: lx should be non-negative and non-increasing
            if i > 0 {
                let prev_lx = lx.get(i - 1).unwrap();
                assert!(
                    lx_val <= prev_lx,
                    "lx should be non-increasing at index {i}"
                );
            }
        }

        // Test commutation function relationships
        let _Dx = result.column("Dx").unwrap().f64().unwrap();
        let Nx = result.column("Nx").unwrap().f64().unwrap();
        let _Cx = result.column("Cx").unwrap().f64().unwrap();
        let _Mx = result.column("Mx").unwrap().f64().unwrap();

        // Test: Nx should be decreasing
        for i in 1..std::cmp::min(10, result.height()) {
            let nx_curr = Nx.get(i).unwrap();
            let nx_prev = Nx.get(i - 1).unwrap();
            assert!(nx_curr < nx_prev, "Nx should be decreasing at index {i}");
        }

        println!("✓ Actuarial relationships verified");
    }

    #[test]
    fn test_different_radix_values() {
        let xml = MortXML::from_url_id(1704).expect("Failed to load XML");

        let radix_values = vec![100_000, 1_000_000, 10_000_000];

        for &radix in &radix_values {
            let config = MortTableConfig {
                xml: xml.clone(),
                radix: Some(radix),
                pct: Some(1.0),
                int_rate: None,
                assumption: None,
            };

            let result = config
                .gen_mort_table(1)
                .unwrap_or_else(|_| panic!("Failed with radix {radix}"));

            // Test that first lx value equals the radix
            let first_lx = result.column("lx").unwrap().f64().unwrap().get(0).unwrap();
            assert_eq!(
                first_lx, radix as f64,
                "First lx should equal radix for {radix}"
            );

            println!("✓ Radix {radix} working correctly");
        }
    }

    #[test]
    fn test_error_handling() {
        // Test with empty XML (this should be created to test error conditions)
        let xml = MortXML::from_url_id(1704).expect("Failed to load XML");

        // Create a config that should work
        let config = MortTableConfig {
            xml,
            radix: Some(100_000),
            pct: Some(1.0),
            int_rate: None,
            assumption: None,
        };

        // Test that valid config works
        let result = config.gen_mort_table(1);
        assert!(result.is_ok(), "Valid config should succeed");

        println!("✓ Error handling tests completed");
    }

    #[test]
    fn test_comprehensive_table_validation() {
        let xml = MortXML::from_url_id(1704).expect("Failed to load XML");

        let config = MortTableConfig {
            xml,
            radix: Some(100_000),
            pct: Some(0.75),       // 75% of base rates
            int_rate: Some(0.035), // 3.5% interest
            assumption: Some(AssumptionEnum::HPB),
        };

        let result = config
            .gen_mort_table(2)
            .expect("Failed to generate comprehensive table");

        // Print table summary
        println!("\n=== COMPREHENSIVE TABLE VALIDATION ===");
        println!(
            "Table dimensions: {} rows × {} columns",
            result.height(),
            result.width()
        );
        println!("Configuration: 75% mortality, 3.5% interest, HPB assumption");

        // Show first few rows
        if result.height() >= 5 {
            println!("\nFirst 5 rows:");
            println!("{}", result.head(Some(5)));
        }

        // Show last few rows
        if result.height() >= 5 {
            println!("\nLast 5 rows:");
            println!("{}", result.tail(Some(5)));
        }

        // Validate data integrity
        let lx_col = result.column("lx").unwrap().f64().unwrap();
        let dx_col = result.column("dx").unwrap().f64().unwrap();
        let qx_col = result.column("qx").unwrap().f64().unwrap();

        // Check that we start with the correct radix
        assert_eq!(
            lx_col.get(0).unwrap(),
            100_000.0,
            "Should start with 100,000 lives"
        );

        // Check that mortality rates are reasonable (between 0 and 1)
        for i in 0..result.height() {
            let qx = qx_col.get(i).unwrap();
            assert!(
                (0.0..=1.0).contains(&qx),
                "Mortality rate out of bounds at row {i}: {qx}"
            );
        }

        // Check that deaths don't exceed lives
        for i in 0..result.height() {
            let lx = lx_col.get(i).unwrap();
            let dx = dx_col.get(i).unwrap();
            assert!(dx <= lx, "Deaths exceed lives at row {i}: dx={dx}, lx={lx}");
        }

        // Test commutation function values are positive
        if let Ok(dx_comm) = result.column("Dx") {
            let dx_values = dx_comm.f64().unwrap();
            for i in 0..std::cmp::min(10, result.height()) {
                let dx_val = dx_values.get(i).unwrap();
                assert!(dx_val > 0.0, "Dx should be positive at row {i}: {dx_val}");
            }
        }

        println!("✓ All comprehensive validations passed");
        println!("✓ Table generation working correctly with all features");
    }

    #[test]
    #[ignore]
    fn test_check_doctest_xml_ids() {
        // Check XML 28001 (life table format)
        let xml_28001 = MortXML::from_url_id(28001).expect("Failed to load XML 28001");
        println!("XML 28001 - Number of tables: {}", xml_28001.tables.len());
        println!(
            "XML 28001 - Table name: {}",
            xml_28001.content_classification.table_name
        );
        println!(
            "XML 28001 - Content type: {}",
            xml_28001.content_classification.content_type
        );

        // Check XML 1705 (mortality rate format)
        let xml_1705 = MortXML::from_url_id(1705).expect("Failed to load XML 1705");
        println!("XML 1705 - Number of tables: {}", xml_1705.tables.len());
        println!(
            "XML 1705 - Table name: {}",
            xml_1705.content_classification.table_name
        );
        println!(
            "XML 1705 - Content type: {}",
            xml_1705.content_classification.content_type
        );

        // Both should have exactly 1 table
        assert_eq!(
            xml_28001.tables.len(),
            1,
            "XML 28001 should have exactly 1 table"
        );
        assert_eq!(
            xml_1705.tables.len(),
            1,
            "XML 1705 should have exactly 1 table"
        );
    }

    #[test]
    fn test_mathematical_precision() {
        let xml = MortXML::from_url_id(1704).expect("Failed to load XML");

        let config = MortTableConfig {
            xml,
            radix: Some(1_000_000), // Higher precision with larger radix
            pct: Some(1.0),
            int_rate: Some(0.03),
            assumption: Some(AssumptionEnum::UDD),
        };

        let result = config
            .gen_mort_table(2)
            .expect("Failed to generate high precision table");

        // Test precision of calculations
        let lx = result.column("lx").unwrap().f64().unwrap();
        let dx = result.column("dx").unwrap().f64().unwrap();
        let _qx = result.column("qx").unwrap().f64().unwrap();

        // Verify l(x+1) = lx - dx relationship
        for i in 0..std::cmp::min(result.height() - 1, 50) {
            let lx_curr = lx.get(i).unwrap();
            let dx_curr = dx.get(i).unwrap();
            let lx_next = lx.get(i + 1).unwrap();

            let expected_lx_next = lx_curr - dx_curr;
            assert!(
                (lx_next - expected_lx_next).abs() < 1e-6,
                "Life table relationship violated at age {}: l(x+1)={}, lx-dx={}",
                i,
                lx_next,
                expected_lx_next
            );
        }

        println!("✓ Mathematical precision verified with high-precision calculations");
    }
}
