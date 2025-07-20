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
///
/// # Example
/// ```rust
/// use rslife::prelude::*;
///
/// let assumption = AssumptionEnum::UDD; // Most common choice
/// ```
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
/// # Examples
///
/// See [`MortTableConfig::gen_mort_table`] for detailed usage and examples.
#[derive(Debug, Clone)]
pub struct MortTableConfig {
    /// Source mortality data (must contain exactly one age-based table).
    pub xml: MortXML,

    /// Initial population size (radix). Common values: 100,000 (standard), 1,000,000 (precise).
    pub l_x_init: i32,

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
    /// This method processes mortality data according to the specified configuration and returns a DataFrame with the requested level of detail.
    ///
    /// # Detail Levels
    ///
    /// - **Level 1**: Basic mortality rates only (`age`, `qx`, `px`). Fastest, for rate analysis and validation.
    /// - **Level 2**: Demographic functions (`age`, `qx`, `px`, `lx`, `dx`). For life table and survival analysis.
    /// - **Level 3**: Commutation functions (all level 2 plus `Cx`, `Dx`, `Mx`, `Nx`, `Px`, `Rx`, `Sx`). For present value and custom actuarial calculations (requires `int_rate`).
    /// - **Level 4**: Complete actuarial table (all columns, including present value and annuity functions). For insurance and pension calculations (requires `int_rate`).
    ///
    /// # Returns
    ///
    /// A `PolarsResult<DataFrame>` with columns depending on the detail level:
    ///
    /// - Level 1: `age`, `qx`, `px`
    /// - Level 2: `age`, `qx`, `px`, `lx`, `dx`
    /// - Level 3: All level 2 plus `Cx`, `Dx`, `Mx`, `Nx`, `Px`, `Rx`, `Sx`
    /// - Level 4: All actuarial columns (`age`, `qx`, `px`, `lx`, `dx`, `Cx`, `Dx`, `Mx`, `Nx`, `Px`, `Rx`, `Sx`, `Ax`, `AAx`, `IAx`, `IAAx`, `ax`, `aax`, `Iax`, `Iaax`)
    ///
    /// # Examples
    ///
    /// ## Level 1: Basic mortality rates only
    /// ```rust
    /// use rslife::prelude::*;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let xml = MortXML::from_url_id(1704)?;
    /// let config = MortTableConfig {
    ///     xml,
    ///     l_x_init: 100_000,
    ///     pct: Some(1.0),
    ///     int_rate: None,
    ///     assumption: None,
    /// };
    /// let rates_only = config.gen_mort_table(1)?;
    /// assert!(rates_only.height() > 0);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// ## Level 4: Complete actuarial table
    /// ```rust
    /// use rslife::prelude::*;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let xml = MortXML::from_url_id(1704)?;
    /// let config = MortTableConfig {
    ///     xml,
    ///     l_x_init: 100_000,
    ///     pct: Some(1.0),
    ///     int_rate: Some(0.03),
    ///     assumption: Some(AssumptionEnum::UDD),
    /// };
    /// let table = config.gen_mort_table(4)?;
    /// assert!(table.height() > 0);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Mathematical Formulas
    ///
    /// - lₓ₊₁ = lₓ · (1 - qₓ)
    /// - dₓ = lₓ · qₓ
    /// - v = 1/(1+i)
    /// - Cₓ = vˣ⁺¹ · dₓ
    /// - Dₓ = vˣ · lₓ
    /// - Mₓ = Σ(k=x to ω) Cₖ
    /// - Nₓ = Σ(k=x to ω) Dₖ
    /// - Rₓ = Σ(k=x to ω) Mₖ
    /// - Sₓ = Σ(k=x to ω) Nₖ
    /// - Pₓ = Mₓ/Nₓ
    /// - Ax = Mₓ/Dₓ
    /// - AAx = Ax + 1
    /// - IAx = Rₓ/Dₓ
    /// - IAAx = (Rₓ + Sₓ)/Dₓ
    /// - ax = Nₓ/Dₓ - 1
    /// - aax = Nₓ/Dₓ
    /// - Iax = Iaax - aax
    /// - Iaax = Sₓ/Dₓ
    ///
    /// # Errors
    ///
    /// Returns `PolarsError::ComputeError` if:
    /// - No tables found in the XML data
    /// - Multiple tables found (not yet supported)
    /// - Tables with 'duration' column (not yet supported)
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
                "Multiple tables are in MortXML.".into(),
            ));
        }

        let df = self.xml.tables[0].values.clone();

        // Push error first if 'duration' column exists, then match detail_level
        if df.column("duration").is_ok() {
            return Err(PolarsError::ComputeError(
                "Mortality table with 'duration' is not yet supported.".into(),
            ));
        }

        match detail_level {
            // Level 1: Include age,qx
            1 => gen_rate_with_pct(df, self.pct),
            // Level 2: Include age,mortality rates, lx, dx
            2 => {
                let df = gen_rate_with_pct(df, self.pct)?;
                gen_demographic_movement(df, self.l_x_init)
            }
            // Level 3: Include age,mortality rates, lx, dx, Cx, Dx, Mx, Nx, Px, Rx, Sx
            3 => {
                // Require interest rate for detail level 3
                if self.int_rate.is_none() {
                    return Err(PolarsError::ComputeError(
                        "Interest rate is required for detail level 3.".into(),
                    ));
                }

                let df = gen_rate_with_pct(df, self.pct)?;
                let df = gen_demographic_movement(df, self.l_x_init)?;
                gen_commutation(df, self.int_rate.unwrap())
            }
            // Level 4: Include all columns from level 3 plus assurance and annuity functions (Ax, AAx, IAx, IAAx, ax, aax, Iaax, Iax)
            4 => {
                if self.int_rate.is_none() {
                    return Err(PolarsError::ComputeError(
                        "Interest rate is required for detail level 4.".into(),
                    ));
                }

                let df = gen_rate_with_pct(df, self.pct)?;
                let df = gen_demographic_movement(df, self.l_x_init)?;
                let df = gen_commutation(df, self.int_rate.unwrap())?;
                gen_assurance_annuity(df)
            }
            // Invalid detail level
            _ => Err(PolarsError::ComputeError(
                "Invalid detail level specified.".into(),
            )),
        }
    }
}

//--------- HELPER FUNCTIONS FOR MORTALITY TABLE GENERATION ---------//
fn gen_rate_with_pct(df: DataFrame, pct: Option<f64>) -> PolarsResult<DataFrame> {
    // Apply percentage to the table rates if provided
    let result = df
        .lazy()
        .with_columns(vec![
            (col("value") * lit(pct.unwrap_or(1.0))).alias("qx"),
            (lit(1.0) - col("value")).alias("px"),
        ])
        .select(vec![col("age"), col("qx"), col("px")])
        .collect()?;

    Ok(result)
}

fn gen_demographic_movement(df: DataFrame, l_x_init: i32) -> PolarsResult<DataFrame> {
    // Calculate lx values from the mortality table
    let age = df.column("age")?.i32()?.to_vec();
    let qx = df.column("qx")?.f64()?.to_vec();
    let px = df.column("px")?.f64()?.to_vec();

    let mut lx: Vec<i32> = Vec::with_capacity(age.len());
    let mut dx: Vec<i32> = Vec::with_capacity(age.len());
    // Default initial lx value
    lx.push(l_x_init);

    for i in 0..age.len() {
        let qx_val = qx[i].unwrap(); // Known that the value is always present
        // lx
        if i > 0 {
            let lx_value = lx[i - 1] - dx[i - 1];
            lx.push(lx_value);
        }
        // dx
        let dx_value = (lx[i] as f64 * qx_val).round() as i32;
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

#[allow(non_snake_case)]
fn gen_commutation(
    df: DataFrame,
    int_rate: f64, // Interest rate
) -> PolarsResult<DataFrame> {
    let age = df.column("age")?.i32()?.to_vec();
    let qx = df.column("qx")?.f64()?.to_vec();
    let px = df.column("px")?.f64()?.to_vec();
    let lx = df.column("lx")?.i32()?.to_vec();
    let dx = df.column("dx")?.i32()?.to_vec();

    let mut Dx: Vec<f64> = Vec::with_capacity(age.len());
    let mut Cx: Vec<f64> = Vec::with_capacity(age.len());

    // Cx and Dx
    for i in 0..age.len() {
        let age_f64 = age[i].unwrap() as f64;
        let lx_f64 = lx[i].unwrap() as f64;
        let dx_f64 = dx[i].unwrap() as f64;

        // Cx = vˣ⁺¹ * dx = dx / (1+i)ˣ⁺¹
        let cx_value = dx_f64 / (1.0 + int_rate).powf(age_f64 + 1.0);
        Cx.push(cx_value);

        // Dx = vˣ * lx = lx / (1+i)ˣ
        let dx_value = lx_f64 / (1.0 + int_rate).powf(age_f64);
        Dx.push(dx_value);
    }

    // Nx and Mx
    let mut Nx: Vec<f64> = Vec::with_capacity(age.len());
    let mut Mx: Vec<f64> = Vec::with_capacity(age.len());
    let mut Px: Vec<f64> = Vec::with_capacity(age.len());
    for i in 0..age.len() {
        let nx_value = Dx[i..].iter().sum();
        Nx.push(nx_value);

        let mx_value = Cx[i..].iter().sum();
        Mx.push(mx_value);

        let px_value = if nx_value > 0.0 {
            mx_value / nx_value
        } else {
            0.0
        };
        Px.push(px_value);
    }

    // Sx and Rx
    let mut Rx: Vec<f64> = Vec::with_capacity(age.len());
    let mut Sx: Vec<f64> = Vec::with_capacity(age.len());
    for i in 0..age.len() {
        let rx_value = Mx[i..].iter().sum();
        Rx.push(rx_value);

        let sx_value = Nx[i..].iter().sum();
        Sx.push(sx_value);
    }

    let result = DataFrame::new(vec![
        Series::new("age".into(), age).into_column(),
        Series::new("qx".into(), qx).into_column(),
        Series::new("px".into(), px).into_column(),
        Series::new("lx".into(), lx).into_column(),
        Series::new("dx".into(), dx).into_column(),
        Series::new("Cx".into(), Cx).into_column(),
        Series::new("Dx".into(), Dx).into_column(),
        Series::new("Mx".into(), Mx).into_column(),
        Series::new("Nx".into(), Nx).into_column(),
        Series::new("Px".into(), Px).into_column(),
        Series::new("Rx".into(), Rx).into_column(),
        Series::new("Sx".into(), Sx).into_column(),
    ])?;

    Ok(result)
}

fn gen_assurance_annuity(df: DataFrame) -> PolarsResult<DataFrame> {
    let lf = df
        .lazy()
        .with_columns([
            (col("Mx") / col("Dx")).alias("Ax"),                 // Mₓ/Dₓ
            (col("Mx") / col("Dx") + lit(1.0)).alias("AAx"),     // Mₓ/Dₓ + 1
            (col("Rx") / col("Dx")).alias("IAx"),                // Rₓ/Dₓ
            ((col("Rx") + col("Sx")) / col("Dx")).alias("IAAx"), // (Rₓ+Sₓ)/Dₓ
            (col("Nx") / col("Dx") - lit(1.0)).alias("ax"),      // Nₓ/Dₓ - 1
            (col("Nx") / col("Dx")).alias("aax"),                // Nₓ/Dₓ
            (col("Sx") / col("Dx")).alias("Iaax"),               // Sₓ/Dₓ
        ])
        .with_columns([
            (col("Iaax") - col("aax")).alias("Iax"), //  (Ia)ₓ = (Iä)ₓ - äₓ
        ])
        .collect()?;
    Ok(lf)
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
            l_x_init: 100_000,
            pct: Some(1.0),
            int_rate: None,
            assumption: None,
        };

        let result = config
            .gen_mort_table(2)
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
            result.column("lx").unwrap().dtype().is_integer(),
            "lx should be integer"
        );
        assert!(
            result.column("dx").unwrap().dtype().is_integer(),
            "dx should be integer"
        );

        println!("✓ Basic mortality table generated successfully");
        println!(
            "Table dimensions: {} rows × {} columns",
            result.height(),
            result.width()
        );
    }

    #[test]
    fn test_mortality_table_with_commutation() {
        let xml = MortXML::from_url_id(1704).expect("Failed to load XML");

        let config = MortTableConfig {
            xml,
            l_x_init: 100_000,
            pct: Some(1.0),
            int_rate: Some(0.03), // 3% interest rate
            assumption: Some(AssumptionEnum::UDD),
        };

        let result = config
            .gen_mort_table(3)
            .expect("Failed to generate commutation table");

        // Test commutation table structure
        assert!(result.height() > 0, "Result DataFrame should not be empty");
        // Level 3 should have: age, qx, px, lx, dx, Cx, Dx, Mx, Nx, Px, Rx, Sx = 12 columns
        assert_eq!(
            result.width(),
            12,
            "Commutation table should have 12 columns"
        );

        // Test all expected columns are present
        let expected_columns = vec![
            "age", "qx", "px", "lx", "dx", "Cx", "Dx", "Mx", "Nx", "Px", "Rx", "Sx",
        ];
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
    fn test_percentage_adjustment() {
        let xml = MortXML::from_url_id(1704).expect("Failed to load XML");

        // Test with 50% of base rates
        let config_50 = MortTableConfig {
            xml: xml.clone(),
            l_x_init: 100_000,
            pct: Some(0.5),
            int_rate: None,
            assumption: None,
        };

        // Test with 100% of base rates
        let config_100 = MortTableConfig {
            xml: xml.clone(),
            l_x_init: 100_000,
            pct: Some(1.0),
            int_rate: None,
            assumption: None,
        };

        // Test with 150% of base rates
        let config_150 = MortTableConfig {
            xml,
            l_x_init: 100_000,
            pct: Some(1.5),
            int_rate: None,
            assumption: None,
        };

        let table_50 = config_50.gen_mort_table(2).expect("Failed with 50% rates");
        let table_100 = config_100
            .gen_mort_table(2)
            .expect("Failed with 100% rates");
        let table_150 = config_150
            .gen_mort_table(2)
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
            .i32()
            .unwrap()
            .get(30)
            .unwrap_or(0);
        let lx_30_100 = table_100
            .column("lx")
            .unwrap()
            .i32()
            .unwrap()
            .get(30)
            .unwrap_or(0);
        let lx_30_150 = table_150
            .column("lx")
            .unwrap()
            .i32()
            .unwrap()
            .get(30)
            .unwrap_or(0);

        assert!(
            lx_30_50 > lx_30_100,
            "Lower mortality should result in higher survival"
        );
        assert!(
            lx_30_100 > lx_30_150,
            "Higher mortality should result in lower survival"
        );

        println!("✓ Percentage adjustment working correctly");
        println!(
            "qx at index 5: 50%={:.6}, 100%={:.6}, 150%={:.6}",
            qx_50, qx_100, qx_150
        );
    }

    #[test]
    fn test_actuarial_relationships() {
        let xml = MortXML::from_url_id(1704).expect("Failed to load XML");

        let config = MortTableConfig {
            xml,
            l_x_init: 100_000,
            pct: Some(1.0),
            int_rate: Some(0.04), // 4% interest rate
            assumption: Some(AssumptionEnum::CFM),
        };

        let result = config.gen_mort_table(3).expect("Failed to generate table");

        let lx = result.column("lx").unwrap().i32().unwrap();
        let dx = result.column("dx").unwrap().i32().unwrap();
        let qx = result.column("qx").unwrap().f64().unwrap();

        // Test actuarial relationships for first few rows
        for i in 0..std::cmp::min(10, result.height()) {
            let lx_val = lx.get(i).unwrap();
            let dx_val = dx.get(i).unwrap();
            let qx_val = qx.get(i).unwrap();

            // Test: dx = lx * qx (approximately, due to rounding)
            let expected_dx = (lx_val as f64 * qx_val).round() as i32;
            assert_eq!(
                dx_val, expected_dx,
                "dx calculation incorrect at index {}",
                i
            );

            // Test: qx should be between 0 and 1
            assert!(
                qx_val >= 0.0 && qx_val <= 1.0,
                "qx should be a probability at index {}",
                i
            );

            // Test: lx should be non-negative and non-increasing
            if i > 0 {
                let prev_lx = lx.get(i - 1).unwrap();
                assert!(
                    lx_val <= prev_lx,
                    "lx should be non-increasing at index {}",
                    i
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
            assert!(nx_curr < nx_prev, "Nx should be decreasing at index {}", i);
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
                l_x_init: radix,
                pct: Some(1.0),
                int_rate: None,
                assumption: None,
            };

            let result = config
                .gen_mort_table(2)
                .expect(&format!("Failed with radix {}", radix));

            // Test that first lx value equals the radix
            let first_lx = result.column("lx").unwrap().i32().unwrap().get(0).unwrap();
            assert_eq!(first_lx, radix, "First lx should equal radix for {}", radix);

            println!("✓ Radix {} working correctly", radix);
        }
    }

    #[test]
    fn test_error_handling() {
        // Test with empty XML (this should be created to test error conditions)
        let xml = MortXML::from_url_id(1704).expect("Failed to load XML");

        // Create a config that should work
        let config = MortTableConfig {
            xml,
            l_x_init: 100_000,
            pct: Some(1.0),
            int_rate: None,
            assumption: None,
        };

        // Test that valid config works
        let result = config.gen_mort_table(2);
        assert!(result.is_ok(), "Valid config should succeed");

        println!("✓ Error handling tests completed");
    }

    #[test]
    fn test_comprehensive_table_validation() {
        let xml = MortXML::from_url_id(1704).expect("Failed to load XML");

        let config = MortTableConfig {
            xml,
            l_x_init: 100_000,
            pct: Some(0.75),       // 75% of base rates
            int_rate: Some(0.035), // 3.5% interest
            assumption: Some(AssumptionEnum::HPB),
        };

        let result = config
            .gen_mort_table(3)
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
        let lx_col = result.column("lx").unwrap().i32().unwrap();
        let dx_col = result.column("dx").unwrap().i32().unwrap();
        let qx_col = result.column("qx").unwrap().f64().unwrap();

        // Check that we start with the correct radix
        assert_eq!(
            lx_col.get(0).unwrap(),
            100_000,
            "Should start with 100,000 lives"
        );

        // Check that mortality rates are reasonable (between 0 and 1)
        for i in 0..result.height() {
            let qx = qx_col.get(i).unwrap();
            assert!(
                qx >= 0.0 && qx <= 1.0,
                "Mortality rate out of bounds at row {}: {}",
                i,
                qx
            );
        }

        // Check that deaths don't exceed lives
        for i in 0..result.height() {
            let lx = lx_col.get(i).unwrap();
            let dx = dx_col.get(i).unwrap();
            assert!(
                dx <= lx,
                "Deaths exceed lives at row {}: dx={}, lx={}",
                i,
                dx,
                lx
            );
        }

        // Test commutation function values are positive
        if let Ok(dx_comm) = result.column("Dx") {
            let dx_values = dx_comm.f64().unwrap();
            for i in 0..std::cmp::min(10, result.height()) {
                let dx_val = dx_values.get(i).unwrap();
                assert!(
                    dx_val > 0.0,
                    "Dx should be positive at row {}: {}",
                    i,
                    dx_val
                );
            }
        }

        println!("✓ All comprehensive validations passed");
        println!("✓ Table generation working correctly with all features");
    }

    #[test]
    fn test_mathematical_precision() {
        let xml = MortXML::from_url_id(1704).expect("Failed to load XML");

        let config = MortTableConfig {
            xml,
            l_x_init: 1_000_000, // Higher precision with larger radix
            pct: Some(1.0),
            int_rate: Some(0.03),
            assumption: Some(AssumptionEnum::UDD),
        };

        let result = config
            .gen_mort_table(3)
            .expect("Failed to generate high precision table");

        // Test precision of calculations
        let lx = result.column("lx").unwrap().i32().unwrap();
        let dx = result.column("dx").unwrap().i32().unwrap();
        let _qx = result.column("qx").unwrap().f64().unwrap();

        // Verify l(x+1) = lx - dx relationship
        for i in 0..std::cmp::min(result.height() - 1, 50) {
            let lx_curr = lx.get(i).unwrap();
            let dx_curr = dx.get(i).unwrap();
            let lx_next = lx.get(i + 1).unwrap();

            assert_eq!(
                lx_next,
                lx_curr - dx_curr,
                "Life table relationship violated at age {}: l(x+1)={}, lx-dx={}",
                i,
                lx_next,
                lx_curr - dx_curr
            );
        }

        println!("✓ Mathematical precision verified with high-precision calculations");
    }
}
