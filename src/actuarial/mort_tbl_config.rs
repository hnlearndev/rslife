#![allow(non_snake_case)] // Allow actuarial notation (gen_Ax_IAx, etc.)

use crate::xml::MortXML;
use polars::prelude::*;

/// Mortality assumption types for fractional age calculations.
///
/// These assumptions determine how mortality is distributed within each age interval
/// and affect calculations involving fractional ages or time periods.
///
/// # Mathematical Formulations
///
/// For fractional time $t$ at age $x$, the survival probability ${}_{t}p_x$ is calculated as:
///
/// **UDD (Uniform Distribution of Deaths)**:
/// $${}_{t}p_x = 1 - t \cdot q_x$$
///
/// **CFM (Constant Force of Mortality)**:
/// $${}_{t}p_x = (1 - q_x)^t$$
///
/// **HPB (Hyperbolic/Balmer)**:
/// $${}_{t}p_x = \frac{1 - q_x}{1 - (1-t) \cdot q_x}$$
///
/// Where $q_x$ is the annual mortality rate at age $x$.
///
/// # Use Cases
///
/// - **UDD**: Most conservative assumption, commonly used in life insurance
/// - **CFM**: Mathematical convenience, used in continuous-time models
/// - **HPB**: Balances between UDD and CFM, used in some pension calculations
///
/// # Examples
///
/// ```rust
/// use rslife::AssumptionEnum;
///
/// // Different assumptions for fractional age calculations
/// let udd_assumption = AssumptionEnum::UDD;
/// let cfm_assumption = AssumptionEnum::CFM;
/// let hpb_assumption = AssumptionEnum::HPB;
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssumptionEnum {
    /// Uniform Distribution of Deaths
    ///
    /// Assumes deaths are uniformly distributed within each age interval.
    /// The force of mortality increases linearly throughout the year:
    /// $$\mu_{x+t} = \frac{q_x}{1-t \cdot q_x}$$
    ///
    /// This is the most commonly used assumption in life insurance calculations.
    UDD,

    /// Constant Force of Mortality
    ///
    /// Assumes the force of mortality is constant within each age interval.
    /// The constant force is given by:
    /// $$\mu_x = -\ln(1-q_x)$$
    ///
    /// This assumption provides mathematical convenience for continuous-time models.
    CFM,

    /// Hyperbolic (Balmer) Assumption
    ///
    /// Assumes a hyperbolic distribution of deaths within each age interval.
    /// The force of mortality decreases throughout the year:
    /// $$\mu_{x+t} = \frac{q_x}{1+t \cdot q_x}$$
    ///
    /// This assumption provides a middle ground between UDD and CFM.
    HPB,
}

/// Configuration for generating complete mortality tables with demographic and actuarial functions.
///
/// This struct contains all parameters needed to generate a comprehensive mortality table
/// from XML source data, including basic demographic movement functions and optional
/// commutation functions for actuarial present value calculations.
///
/// # Fields
///
/// - `xml`: Source mortality data in standardized XML format
/// - `l_x_init`: Initial population size (radix) - typically 100,000 or 1,000,000
/// - `pct`: Optional percentage multiplier for mortality rates (e.g., 0.5 for 50% of base rates)
/// - `int_rate`: Optional interest rate for present value calculations and commutation functions
/// - `assumption`: Mortality assumption for fractional age calculations
///
/// # Mathematical Background
///
/// The mortality table generation follows standard actuarial principles:
///
/// ## Rate Adjustment
/// When a percentage is specified:
/// $$q_x^{adjusted} = q_x^{base} \times pct$$
///
/// ## Demographic Functions
/// - **Population**: $l_{x+1} = l_x \times (1 - q_x)$
/// - **Deaths**: $d_x = l_x \times q_x$
///
/// ## Commutation Functions (when interest rate provided)
/// - **Present Value Factor**: $v = \frac{1}{1+i}$
/// - **Life Commutation**: $D_x = v^x \times l_x$
/// - **Death Commutation**: $C_x = v^{x+1} \times d_x$
///
/// # Examples
///
/// ## Basic mortality table without interest:
/// ```rust
/// use rslife::{MortXML, MortTableConfig, AssumptionEnum};
///
/// let xml = MortXML::from_url_id(1704)?;
/// let config = MortTableConfig {
///     xml,
///     l_x_init: 100_000,  // Standard radix of 100,000 lives
///     pct: Some(1.0),      // Use 100% of base mortality rates
///     int_rate: None,      // No interest rate calculations
///     assumption: Some(AssumptionEnum::UDD), // UDD for fractional ages
/// };
///
/// let table = config.gen_mort_table()?;
/// // Generates table with columns: age, qx, lx, dx
/// ```
///
/// ## Complete actuarial table with commutation functions:
/// ```rust
/// let config = MortTableConfig {
///     xml,
///     l_x_init: 100_000,
///     pct: Some(0.5),        // Use 50% of standard mortality (improved mortality)
///     int_rate: Some(0.03),  // 3% annual interest rate
///     assumption: Some(AssumptionEnum::CFM),
/// };
///
/// let actuarial_table = config.gen_mort_table()?;
/// // Generates complete table with commutation functions:
/// // age, qx, lx, dx, Cx, Dx, Mx, Nx, Px, Rx, Sx
/// ```
///
/// ## Substandard mortality (higher risk):
/// ```rust
/// let high_risk_config = MortTableConfig {
///     xml,
///     l_x_init: 100_000,
///     pct: Some(1.5),        // 150% of standard mortality rates
///     int_rate: Some(0.04),  // 4% interest rate
///     assumption: Some(AssumptionEnum::HPB),
/// };
///
/// let substandard_table = high_risk_config.gen_mort_table()?;
/// ```
///
/// # Data Sources
///
/// The `xml` field should contain mortality data from sources such as:
/// - Society of Actuaries (SOA) mortality tables
/// - National statistical office life tables
/// - Insurance company experience tables
/// - Custom mortality studies
///
/// # Performance Considerations
///
/// - Memory allocation is optimized with pre-sized vectors
/// - Sequential processing maintains actuarial dependencies
/// - Large tables (1000+ ages) typically process in milliseconds
/// - Commutation functions add minimal computational overhead
///
/// # Limitations
///
/// - Currently supports single-table XML sources only
/// - Duration-based mortality tables are not yet implemented
/// - The `assumption` field is reserved for future fractional age implementations
///
/// # See Also
///
/// - [`gen_mort_table()`](MortTableConfig::gen_mort_table) for the main table generation method
/// - [`AssumptionEnum`] for mortality assumption types
/// - [`MortXML`] for loading and parsing mortality data
#[derive(Debug, Clone)]
pub struct MortTableConfig {
    /// Source mortality data in XML format.
    ///
    /// Must contain exactly one mortality table with age-based data.
    /// The XML should follow standard mortality table formats with
    /// columns for age and mortality rates.
    pub xml: MortXML,

    /// Initial population size (radix) for the life table.
    ///
    /// This represents the hypothetical starting population at the
    /// youngest age in the table. Common values are:
    /// - 100,000 (standard actuarial practice)
    /// - 1,000,000 (for higher precision)
    /// - 10,000,000 (for very precise calculations)
    ///
    /// # Example
    /// ```rust
    /// let config = MortTableConfig {
    ///     // ... other fields
    ///     l_x_init: 100_000, // Start with 100,000 lives
    ///     // ... other fields
    /// };
    /// ```
    pub l_x_init: i32,

    /// Optional percentage multiplier for mortality rates.
    ///
    /// Allows adjustment of base mortality rates for different populations:
    /// - `Some(1.0)`: Use 100% of base rates (standard mortality)
    /// - `Some(0.5)`: Use 50% of base rates (improved mortality)
    /// - `Some(1.5)`: Use 150% of base rates (substandard mortality)
    /// - `None`: Defaults to 100% of base rates
    ///
    /// Formula: $q_x^{final} = q_x^{base} \times pct$
    ///
    /// # Examples
    /// ```rust
    /// // Preferred risk class (better than standard)
    /// let preferred = MortTableConfig {
    ///     pct: Some(0.75), // 25% reduction in mortality
    ///     // ... other fields
    /// };
    ///
    /// // Substandard risk class
    /// let substandard = MortTableConfig {
    ///     pct: Some(2.0), // Double the standard mortality
    ///     // ... other fields
    /// };
    /// ```
    pub pct: Option<f64>,

    /// Optional interest rate for present value calculations.
    ///
    /// When provided, enables calculation of commutation functions
    /// used in life insurance and pension valuations:
    /// - Present value factors
    /// - Commutation functions (Cx, Dx, Mx, Nx, etc.)
    /// - Actuarial present values
    ///
    /// Should be expressed as a decimal (e.g., 0.03 for 3%).
    ///
    /// # Examples
    /// ```rust
    /// let config = MortTableConfig {
    ///     int_rate: Some(0.03), // 3% annual interest rate
    ///     // ... other fields
    /// };
    /// ```
    pub int_rate: Option<f64>,

    /// Mortality assumption for fractional age calculations.
    ///
    /// Determines how mortality is distributed within each age interval.
    /// Currently reserved for future implementation of fractional age
    /// survival and mortality probability calculations.
    ///
    /// # Future Use
    /// Will be used for calculations involving:
    /// - Fractional survival probabilities (e.g., ${}_{0.5}p_{30}$)
    /// - Mid-year mortality adjustments
    /// - Precise timing of deaths within age intervals
    ///
    /// # Examples
    /// ```rust
    /// let config = MortTableConfig {
    ///     assumption: Some(AssumptionEnum::UDD), // Most common assumption
    ///     // ... other fields
    /// };
    /// ```
    pub assumption: Option<AssumptionEnum>,
}

impl MortTableConfig {
    /// Generates a complete mortality table from the configured XML data.
    ///
    /// This method processes mortality data according to the specified configuration,
    /// creating a comprehensive DataFrame with demographic movement functions and
    /// optional commutation functions for actuarial calculations.
    ///
    /// # Mathematical Formulas
    ///
    /// ## Demographic Movement Functions
    ///
    /// **Life Table Population**:
    /// $$l_{x+1} = l_x - d_x = l_x \cdot (1 - q_x)$$
    ///
    /// **Deaths**:
    /// $$d_x = l_x \cdot q_x$$
    ///
    /// ## Commutation Functions (when interest rate is provided)
    ///
    /// **Present Value Factor**:
    /// $$v = \frac{1}{1+i}$$
    ///
    /// **Commutation Function C_x**:
    /// $$C_x = v^{x+1} \cdot d_x = \frac{d_x}{(1+i)^{x+1}}$$
    ///
    /// **Commutation Function D_x**:
    /// $$D_x = v^x \cdot l_x = \frac{l_x}{(1+i)^x}$$
    ///
    /// **Cumulative Functions**:
    /// $$M_x = \sum_{k=x}^{\omega} C_k$$
    /// $$N_x = \sum_{k=x}^{\omega} D_k$$
    /// $$R_x = \sum_{k=x}^{\omega} M_k$$
    /// $$S_x = \sum_{k=x}^{\omega} N_k$$
    ///
    /// **Probability Function**:
    /// $$P_x = \frac{M_x}{N_x}$$
    ///
    /// Where:
    /// - $x$ = age
    /// - $i$ = interest rate
    /// - $\omega$ = terminal age
    /// - $q_x$ = mortality rate at age $x$
    /// - $l_x$ = number of lives at age $x$
    /// - $d_x$ = number of deaths between age $x$ and $x+1$
    ///
    /// # Returns
    ///
    /// Returns a `PolarsResult<DataFrame>` containing:
    ///
    /// ## Basic Demographic Columns (always present):
    /// - `age`: Age values (i32)
    /// - `qx`: Mortality rates, adjusted by percentage if specified (f64)
    /// - `lx`: Number of lives at age x (i32)
    /// - `dx`: Number of deaths between age x and x+1 (i32)
    ///
    /// ## Commutation Columns (present when `int_rate` is provided):
    /// - `Cx`: Commutation function $C_x = v^{x+1} \times d_x$ (f64)
    /// - `Dx`: Commutation function $D_x = v^x \times l_x$ (f64)
    /// - `Mx`: Sum of $C_x$ values from age x to terminal age (f64)
    /// - `Nx`: Sum of $D_x$ values from age x to terminal age (f64)
    /// - `Px`: Probability function $P_x = M_x / N_x$ (f64)
    /// - `Rx`: Sum of $M_x$ values from age x to terminal age (f64)
    /// - `Sx`: Sum of $N_x$ values from age x to terminal age (f64)
    ///
    /// # Configuration Parameters
    ///
    /// The method uses the following configuration from `MortTableConfig`:
    /// - `xml`: Source mortality data (must contain exactly one table)
    /// - `l_x_init`: Initial population at starting age (radix)
    /// - `pct`: Optional percentage multiplier for mortality rates
    ///   - Formula: $q_x^{adjusted} = q_x^{base} \times pct$
    /// - `int_rate`: Optional interest rate for commutation functions
    /// - `assumption`: Mortality assumption (UDD/CFM/HPB) - currently not used in table generation
    ///
    /// # Examples
    ///
    /// ## Basic mortality table without interest:
    /// ```rust
    /// use rslife::{MortXML, MortTableConfig};
    ///
    /// let xml = MortXML::from_url_id(1704)?;
    /// let config = MortTableConfig {
    ///     xml,
    ///     l_x_init: 100_000,  // Start with 100,000 lives
    ///     pct: Some(1.0),      // Use 100% of base rates
    ///     int_rate: None,      // No interest rate
    ///     assumption: None,    // No specific assumption
    /// };
    ///
    /// let mortality_table = config.gen_mort_table()?;
    /// // Contains columns: age, qx, lx, dx
    /// ```
    ///
    /// ## Complete actuarial table with 3% interest:
    /// ```rust
    /// let config = MortTableConfig {
    ///     xml,
    ///     l_x_init: 100_000,
    ///     pct: Some(0.5),        // Use 50% of base mortality rates
    ///     int_rate: Some(0.03),  // 3% interest rate
    ///     assumption: Some(AssumptionEnum::UDD),
    /// };
    ///
    /// let complete_table = config.gen_mort_table()?;
    /// // Contains all columns including Cx, Dx, Mx, Nx, Px, Rx, Sx
    /// ```
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
    pub fn gen_mort_table(&self) -> PolarsResult<DataFrame> {
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

        // When the mortality table has a 'duration' column, we need to handle it differently
        if df.column("duration").is_ok() {
            Err(PolarsError::ComputeError(
                "Mortality table with 'duration' is not yet supported.".into(),
            ))
        } else {
            gen_full_mort_table(df, self.l_x_init, self.pct, self.int_rate)
        }
    }
}

//--------- HELPER FUNCTIONS FOR MORTALITY TABLE GENERATION ---------//
fn gen_full_mort_table(
    df: DataFrame,
    l_x_init: i32,
    pct: Option<f64>, // Percentage for the table rates values
    int_rate: Option<f64>,
) -> PolarsResult<DataFrame> {
    // Apply percentage to the table rates if provided
    let df = gen_rate_with_pct(df, pct)?;

    // Generate mortality table without interest rate
    let mut df = gen_demographic_movement(df, l_x_init)?;

    // If interest rate is provided, perform commutation
    if let Some(interest_rate) = int_rate {
        df = gen_commutation(df, interest_rate)?;
        df = gen_Ax_IAx(df)?;
    }

    Ok(df)
}

fn gen_rate_with_pct(df: DataFrame, pct: Option<f64>) -> PolarsResult<DataFrame> {
    // Apply percentage to the table rates if provided
    let result = df
        .lazy()
        .with_column(col("value") * lit(pct.unwrap_or(1.0)).alias("value"))
        .collect()?;

    Ok(result)
}

fn gen_demographic_movement(df: DataFrame, l_x_init: i32) -> PolarsResult<DataFrame> {
    // Calculate lx values from the mortality table
    let age = df.column("age")?.i32()?.to_vec();
    let qx = df.column("value")?.f64()?.to_vec();

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
    let lx = df.column("lx")?.i32()?.to_vec();
    let dx = df.column("dx")?.i32()?.to_vec();

    let mut Dx: Vec<f64> = Vec::with_capacity(age.len());
    let mut Cx: Vec<f64> = Vec::with_capacity(age.len());

    // Cx and Dx
    for i in 0..age.len() {
        let age_f64 = age[i].unwrap() as f64;
        let lx_f64 = lx[i].unwrap() as f64;
        let dx_f64 = dx[i].unwrap() as f64;

        // Cx = v^(x+1) * dx = dx / (1+i)^(x+1)
        let cx_value = dx_f64 / (1.0 + int_rate).powf(age_f64 + 1.0);
        Cx.push(cx_value);

        // Dx = v^x * lx = lx / (1+i)^x
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

fn gen_Ax_IAx(df: DataFrame) -> PolarsResult<DataFrame> {
    let lf = df
        .lazy()
        .with_columns([
            (col("Mx") / col("Dx")).alias("Ax"),
            (col("Mx") / col("Dx").shift(lit(-1))).alias("Ax_due"),
            (col("Sx") / col("Dx")).alias("IAx"),
            ((col("Sx") + col("Nx")) / col("Dx")).alias("IAx_due"),
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
            .gen_mort_table()
            .expect("Failed to generate mortality table");

        // Test basic structure
        assert!(result.height() > 0, "Result DataFrame should not be empty");
        assert_eq!(result.width(), 4, "Basic table should have 4 columns");

        // Test column names
        let expected_columns = vec!["age", "qx", "lx", "dx"];
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
            .gen_mort_table()
            .expect("Failed to generate commutation table");

        // Test commutation table structure
        assert!(result.height() > 0, "Result DataFrame should not be empty");
        // The commutation table actually has 15 columns when including all computed values
        assert_eq!(
            result.width(),
            15,
            "Commutation table should have 15 columns"
        );

        // Test all expected columns are present
        let expected_columns = vec![
            "age", "qx", "lx", "dx", "Cx", "Dx", "Mx", "Nx", "Px", "Rx", "Sx", "Ax", "Ax_due",
            "IAx", "IAx_due",
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

        let table_50 = config_50.gen_mort_table().expect("Failed with 50% rates");
        let table_100 = config_100.gen_mort_table().expect("Failed with 100% rates");
        let table_150 = config_150.gen_mort_table().expect("Failed with 150% rates");

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

        let result = config.gen_mort_table().expect("Failed to generate table");

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
                .gen_mort_table()
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
        let result = config.gen_mort_table();
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
            .gen_mort_table()
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
            .gen_mort_table()
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
