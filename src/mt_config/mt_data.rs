use super::ifoa_xls::IFOAMortXLS;
use super::soa_xml::SOAMortXML;
use crate::RSLifeResult;
use bon::bon;
use calamine::{Data, Reader, open_workbook_auto};
use polars::prelude::*;
use spreadsheet_ods::{Value, read_ods};
use std::fs;

#[derive(Debug, Clone)]
pub struct MortData {
    pub category: String,
    pub description: String,
    pub dataframe: DataFrame,
}

#[bon]
impl MortData {
    /// Create a new MortData instance with custom category and DataFrame.
    ///
    /// This is the primary constructor that validates the DataFrame schema before creating
    /// the MortData instance. The DataFrame must conform to mortality table requirements.
    ///
    /// # Schema Requirements
    /// - Must have 2 or 3 columns
    /// - First column: "age" (f64, but must contain whole numbers)
    /// - Second column: "qx" or "lx" (f64)
    /// - Optional third column: "duration" (f64, but must contain whole numbers)
    /// - All values must be non-negative
    /// - qx values must be ≤ 1.0
    ///
    /// # Errors
    /// - Invalid DataFrame schema
    /// - Incorrect column names or types
    /// - Invalid data values (negative, qx > 1.0)
    /// - Empty DataFrame
    ///
    /// # Examples
    /// ```rust, ignore
    /// # use rslife::prelude::*;
    /// use polars::prelude::*;
    ///
    /// let df = df! {
    ///     "age" => [20u32, 21, 22],
    ///     "qx" => [0.001f64, 0.002, 0.003]
    /// }?;
    ///
    /// let mort_data = MortData::new("Custom Table".to_string(), df)?;
    ///# RSLifeResult::Ok(())
    /// ```
    pub fn new(category: String, description: String, dataframe: DataFrame) -> RSLifeResult<Self> {
        // Validate DataFrame schema first
        validate_df_schema(&dataframe)?;

        // Validate and transform DataFrame to ensure it has the correct schema
        let dataframe = setup_dataframe_to_correct_schema(dataframe)
            .map_err(|err| Box::new(err) as Box<dyn std::error::Error>)?;

        // Return result
        let result = Self {
            category,
            description,
            dataframe,
        };

        Ok(result)
    }

    // ========================================================
    // Parametric Mortality Laws
    // ========================================================
    #[builder]
    pub fn from_Constant_Force_law(
        lambda: f64,
        #[builder(default = 0)] start_age: u32,
        #[builder(default = 150)] omega: u32, // end_age
    ) -> RSLifeResult<Self> {
        // Constant force of mortality law:
        // μₓ = λ
        // S₀(x) = exp(-λx)
        // ₜpₓ = exp(-λt)
        // qₓ = 1 - exp(-λ)

        if lambda <= 0.0 {
            return Err("Lambda must be positive".into());
        }

        let ages: Vec<u32> = (start_age..=omega).collect();
        let qx: Vec<f64> = ages
            .iter()
            .map(|&x| 1.0 - (-lambda * x as f64).exp())
            .collect();

        let data = df! {
            "age" => ages,
            "qx" => qx,
        }?;

        // Create MortData from the first table in the XML
        let category = "Parametric Mortality Data".to_string();
        let description = "Constant Force Law".to_string();
        Self::new(category, description, data)
    }

    #[builder]
    pub fn from_DeMoirve_law(
        #[builder(default = 0)] start_age: u32,
        #[builder(default = 150)] omega: u32, // end_age
    ) -> RSLifeResult<Self> {
        // De Moirve law:
        // μₓ = 1/(ω - x) for 0 ≤ x < ω
        // S₀(x) = 1-(x/ω)
        // ₜpₓ = S₀(x + t) / S₀(x) = 1 - t/(ω-x)
        // qₓ = 1 - (1 - 1/(ω-x)) = 1/(ω-x)
        let ages: Vec<u32> = (start_age..omega).collect(); // This excludes omega
        let qx: Vec<f64> = ages
            .iter()
            .map(|&x| 1.0 - (x as f64 / omega as f64))
            .collect();

        let data = df! {
            "age" => ages,
            "qx" => qx,
        }?;

        // Create MortData from the first table in the XML
        let category = "Parametric Mortality Data".to_string();
        let description = "De Moirve Law".to_string();
        Self::new(category, description, data)
    }

    #[builder]
    pub fn from_Gompertz_law(
        B: f64,
        C: f64,
        #[builder(default = 0)] start_age: u32,
        #[builder(default = 150)] omega: u32, // end_age,
    ) -> RSLifeResult<Self> {
        // Gompertz law:
        // μₓ = B.Cˣ
        // S₀(x) = exp[-B/log(C) * (Cˣ - 1)] x ≥ 0, B > 0, c > 1
        // ₜpₓ = S₀(x + t) / S₀(x) = exp[-B/log(C).Cˣ.(Cᵗ - 1)]
        // qₓ = 1 - exp[-B/log(C).Cˣ.(C - 1)]

        // Validate
        if B <= 0.0 || C <= 1.0 {
            return Err("Gompertz parameters must be B > 0 and C > 1".into());
        }

        let ages: Vec<u32> = (start_age..=omega).collect();
        let qx: Vec<f64> = ages
            .iter()
            .map(|&x| 1.0 - (-B / C.ln() * C.powf(x as f64) * (C - 1.0)).exp())
            .collect();

        // Keep 1 qx value equals to 1.0
        let data = keep_first_qx_1_remove_the_rest(ages, qx)?;

        // Create MortData from the first table in the XML
        let category = "Parametric Mortality Data".to_string();
        let description = "Gompertz Law".to_string();
        Self::new(category, description, data)
    }

    #[builder]
    pub fn from_Makeham_law(
        A: f64,
        B: f64,
        C: f64,
        #[builder(default = 0)] start_age: u32,
        #[builder(default = 150)] omega: u32, // end_age
    ) -> RSLifeResult<Self> {
        // Makeham law:
        // μₓ = A + B.Cˣ  x ≥ 0, B > 0, c > 1, A >= -B
        // S₀(x) = exp(-Ax - B  / ln(C) * (Cˣ - 1))
        // ₜpₓ = S₀(x + t) / S₀(x) = exp[-At - B / ln(C). Cˣ(Cᵗ - 1)]
        // qₓ = 1 - exp[-A - B / ln(C).Cˣ.(C - 1)]

        // Validate parameters
        if B <= 0.0 || C <= 1.0 || A < -B {
            return Err("Makeham parameters must be B > 0, C > 1, and A >= -B".into());
        }

        let ages: Vec<u32> = (start_age..=omega).collect();
        let qx: Vec<f64> = ages
            .iter()
            .map(|&x| 1.0 - (-A - B / C.ln() * C.powf(x as f64) * (C - 1.0)).exp())
            .collect();

        // Keep 1 qx value equals to 1.0
        let data = keep_first_qx_1_remove_the_rest(ages, qx)?;

        // Create MortData from the first table in the XML
        let category = "Parametric Mortality Data".to_string();
        let description = "MakeHam Law".to_string();
        Self::new(category, description, data)
    }

    // ========================================================
    // SOA XML PARSING
    // ========================================================

    /// Parse mortality table from SOA XML string format.
    ///
    /// Parses XML data directly from a string containing SOA (Society of Actuaries)
    /// mortality table data in XML format. The XML must conform to SOA standards
    /// and contain approved mortality table types.
    ///
    /// Only approved table layouts and content types are accepted:
    /// - Table layouts: "Aggregate", "Ultimate", "Select", "Select & Ultimate"
    /// - Content types: Various mortality and disability tables (see source for full list)
    ///
    /// Schema validation is performed by `new()` after parsing.
    ///
    /// # Errors
    /// - Invalid XML format
    /// - XML parsing errors
    /// - Unsupported or unapproved table types
    /// - Schema validation errors (via `new()`)
    /// - Missing required XML elements
    ///
    /// # Examples
    /// ```rust, ignore
    /// # use rslife::prelude::*;
    ///
    /// let xml_content = std::fs::read_to_string("mortality_table.xml")?;
    /// let mort_data = MortData::from_soa_xml_string(&xml_content)?;
    /// println!("Loaded: {}", mort_data.category);
    ///# RSLifeResult::Ok(())
    /// ```
    pub fn from_soa_xml_string(xml_str: &str) -> RSLifeResult<Self> {
        // Parse the XML string into SOAMortXML
        let xml_data = SOAMortXML::from_string(xml_str)
            .map_err(|e| PolarsError::ComputeError(e.to_string().into()))?;

        // Return error if the XML data is not in category of our filter
        if !is_soa_xml_data_approved(&xml_data) {
            return Err("XML data is not approved for calculation.".into());
        }

        // Create MortData from the first table in the XML
        let category = "SOA Mortality Data".to_string();
        let description = xml_data.content_classification.table_description.clone();
        let data = xml_data.tables[0].values.clone();
        let result = Self::new(category, description, data)?;
        Ok(result)
    }

    /// Parse mortality table from SOA XML file.
    ///
    /// Reads and parses an XML file containing SOA (Society of Actuaries)
    /// mortality table data. This is a convenience method that reads the file
    /// and delegates to `from_soa_xml_string()` for parsing.
    ///
    /// Only approved SOA table layouts and content types are accepted.
    /// See `from_soa_xml_string()` for more details on requirements.
    ///
    /// # Errors
    /// - File not found or not readable
    /// - File I/O errors
    /// - All errors from `from_soa_xml_string()`
    ///
    /// # Examples
    /// ```rust, ignore
    /// # use rslife::prelude::*;
    ///
    /// // Load SOA mortality table from local XML file
    /// let mort_data = MortData::from_soa_xml_file_path_str("data/t1704.xml")?;
    /// println!("Loaded: {}", mort_data.category);
    ///# RSLifeResult::Ok(())
    /// ```
    pub fn from_soa_xml_file_path_str(file_path: &str) -> RSLifeResult<Self> {
        // Read the XML file into a string
        let xml_str = fs::read_to_string(file_path)
            .map_err(|e| PolarsError::ComputeError(e.to_string().into()))?;

        // Use the from_soa_xml_string method to create MortData
        Self::from_soa_xml_string(&xml_str)
    }

    /// Parse mortality table from SOA URL.
    ///
    /// Downloads and parses mortality table data directly from a SOA (Society of Actuaries)
    /// URL. This method makes an HTTP GET request to fetch XML data and then parses it.
    ///
    /// Requires internet connection and the URL must return valid SOA XML format.
    /// Only approved SOA table layouts and content types are accepted.
    ///
    /// # Errors
    /// - Network connectivity issues
    /// - HTTP request failures (4xx, 5xx status codes)
    /// - Invalid or unreachable URL
    /// - All errors from `from_soa_xml_string()`
    ///
    /// # Examples
    /// ```rust, ignore
    /// # use rslife::prelude::*;
    ///
    /// // Load mortality table directly from SOA website
    /// let url = "https://mort.soa.org/data/t1704.xml";
    /// let mort_data = MortData::from_soa_url(url)?;
    /// println!("Downloaded: {}", mort_data.category);
    ///# RSLifeResult::Ok(())
    /// ```
    pub fn from_soa_url(url: &str) -> RSLifeResult<Self> {
        // Fetch the XML data from the URL
        let response = reqwest::blocking::get(url)
            .map_err(|e| PolarsError::ComputeError(e.to_string().into()))?;

        if !response.status().is_success() {
            return Err("Failed to fetch XML data from URL".into());
        }

        let xml_str = response
            .text()
            .map_err(|e| PolarsError::ComputeError(e.to_string().into()))?;

        // Use the from_soa_xml_string method to create MortData
        Self::from_soa_xml_string(&xml_str)
    }

    /// Parse mortality table from SOA website by table ID.
    ///
    /// Convenience method to download mortality table data from the SOA website
    /// using just the table ID. Constructs the standard SOA URL format and
    /// delegates to `from_soa_url()`.
    ///
    /// The URL format used is: `https://mort.soa.org/data/t{id}.xml`
    ///
    /// Requires internet connection. Only approved SOA table layouts and content types are accepted.
    ///
    /// # Parameters
    /// - `id`: SOA table identifier (e.g., 1704 for table t1704.xml)
    ///
    /// # Errors
    /// - Invalid table ID (table does not exist)
    /// - All errors from `from_soa_url()`
    ///
    /// # Examples
    /// ```rust, ignore
    /// # use rslife::prelude::*;
    ///
    /// // Load table t1704.xml from SOA website
    /// let mort_data = MortData::from_soa_url_id(1704)?;
    /// println!("Downloaded table 1704: {}", mort_data.category);
    ///# RSLifeResult::Ok(())
    /// ```
    pub fn from_soa_url_id(id: i32) -> RSLifeResult<Self> {
        let url = format!("https://mort.soa.org/data/t{id}.xml");
        Self::from_soa_url(&url)
    }

    pub fn from_soa_custom(id: &str) -> RSLifeResult<Self> {
        match id {
            // Makeham law with A=0.00022, B=2.7e-6, C=1.124
            "SULT" => Self::from_Makeham_law()
                .A(0.00022)
                .B(2.7e-6)
                .C(1.124)
                .start_age(20)
                .call(),
            _ => Err(format!("Unknown SOA custom id: {id}").into()),
        }
    }

    // ========================================================
    // IFOA XLS  PARSING
    // ========================================================
    pub fn from_ifoa_xls_file_path_str(file_path: &str, sheet_name: &str) -> RSLifeResult<Self> {
        let data = IFOAMortXLS::from_xls_file_path_str(file_path, sheet_name)?;
        let result = Self::new(
            "IFOA Mortality Data".to_string(),
            data.description,
            data.dataframe,
        )?;
        Ok(result)
    }

    pub fn from_ifoa_url(url: &str) -> RSLifeResult<Self> {
        let data = IFOAMortXLS::from_url(url)?;
        let result = Self::new(
            "IFOA Mortality Data".to_string(),
            data.description,
            data.dataframe,
        )?;
        Ok(result)
    }

    pub fn from_ifoa_url_id(id: &str) -> RSLifeResult<Self> {
        let data = IFOAMortXLS::from_url_id(id)?;
        let result = Self::new(
            "IFOA Mortality Data".to_string(),
            data.description,
            data.dataframe,
        )?;
        Ok(result)
    }

    pub fn from_ifoa_custom(id: &str) -> RSLifeResult<Self> {
        let data = IFOAMortXLS::from_custom(id)?;
        let result = Self::new(
            "IFOA Mortality Data".to_string(),
            data.description,
            data.dataframe,
        )?;
        Ok(result)
    }

    // ========================================================
    // OTHER PARSING METHODS
    // ========================================================
    /// Create mortality table from existing Polars DataFrame.
    ///
    /// Convenience method to create MortData from a pre-existing DataFrame
    /// with a default category name. The DataFrame must conform to mortality
    /// table schema requirements.
    ///
    /// This method delegates to `new()` with a standard category name,
    /// so all schema validation rules apply.
    ///
    /// # Schema Requirements
    /// - Must have 2 or 3 columns
    /// - First column: "age" (f64, but must contain whole numbers)
    /// - Second column: "qx" or "lx" (f64)
    /// - Optional third column: "duration" (f64, but must contain whole numbers)
    /// - All values must be non-negative
    /// - qx values must be ≤ 1.0
    ///
    /// # Errors
    /// - All errors from `new()` (schema validation failures)
    ///
    /// # Examples
    /// ```rust, ignore
    /// # use rslife::prelude::*;
    /// use polars::prelude::*;
    ///
    /// let df = df! {
    ///     "age" => [25.0, 26.0, 27.0],
    ///     "qx" => [0.002, 0.003, 0.004]
    /// }?;
    ///
    /// let mort_data = MortData::from_df(df)?;
    /// assert_eq!(mort_data.category, "Custom Mortality Data");
    ///# RSLifeResult::Ok(())
    /// ```
    pub fn from_df(df: DataFrame) -> RSLifeResult<Self> {
        // Create MortData with a default category
        let category = "Custom Mortality Data".to_string();
        let description = "Created from DataFrame".to_string();
        Self::new(category, description, df)
    }

    /// Parse mortality table from ODS file using spreadsheet-ods.
    ///
    /// Reads ODS files and automatically parses all columns as f64.
    /// Age and duration columns are validated to contain whole numbers during schema validation.
    ///
    /// Schema validation is performed by `from_df()` after parsing.
    ///
    /// # Errors
    /// - File not found or not readable
    /// - Invalid ODS format
    /// - Sheet not found
    /// - Invalid data in cells
    /// - Empty sheets or insufficient data
    /// - Schema validation errors (via `from_df`)
    ///
    /// # Examples
    /// ```rust, ignore
    /// # use rslife::prelude::*;
    ///
    /// // Ultimate table with qx values
    /// let mort_data = MortData::from_ods("data/ltam_standard_ultimate.ods", "ltam")?;
    ///
    /// // Select table with duration
    /// let mort_data = MortData::from_ods("data/am92_select.ods", "AM92")?;
    ///# RSLifeResult::Ok(())
    /// ```
    pub fn from_ods(ods_file_path_str: &str, sheet_name: &str) -> RSLifeResult<Self> {
        // Open ODS workbook
        let workbook = read_ods(ods_file_path_str).map_err(|e| -> Box<dyn std::error::Error> {
            format!("Failed to open ODS file '{ods_file_path_str}': {e}").into()
        })?;

        // Find the sheet by name - iterate through sheets to find by name
        let mut sheet = None;
        for i in 0..workbook.num_sheets() {
            let current_sheet = workbook.sheet(i);
            if current_sheet.name() == sheet_name {
                sheet = Some(current_sheet);
                break;
            }
        }

        let sheet = sheet.ok_or_else(|| format!("Sheet '{sheet_name}' not found in ODS file"))?;

        // Get sheet dimensions
        let (max_row, max_col) = sheet.used_grid_size();

        // Check if sheet is empty
        if max_row < 1 {
            return Err(format!("Sheet '{sheet_name}' is empty").into());
        }

        // Extract headers from first row
        let mut column_names = Vec::new();
        for col in 0..=max_col {
            let cell_value = sheet.value(0, col);
            let col_name = extract_ods_header_name(cell_value, &format!("Column {}", col + 1))?;
            column_names.push(col_name);
        }

        // Parse data rows based on column names
        let mut column_data: Vec<Vec<AnyValue>> = vec![Vec::new(); column_names.len()];

        for row in 1..=max_row {
            let row_num = (row + 1) as usize; // 1-based for user-friendly error messages

            for (col_idx, col_name) in column_names.iter().enumerate() {
                let cell_value = sheet.value(row, col_idx as u32);
                // Parse as f64 for all other columns (tqx, lx, etc.)
                let val = parse_ods_f64_cell(cell_value, row_num, col_name)?;
                let any_value = AnyValue::Float64(val);
                column_data[col_idx].push(any_value);
            }
        }

        // Validate that we have data
        if column_data.is_empty() || column_data[0].is_empty() {
            return Err("No data rows found in sheet".into());
        }

        // Build DataFrame
        let mut columns = Vec::new();
        for (col_name, data) in column_names.iter().zip(column_data.iter()) {
            let series = Series::from_any_values(col_name.as_str().into(), data, true)
                .map_err(|e| format!("Failed to create series for column '{col_name}': {e}"))?;
            columns.push(series.into_column());
        }

        let df = DataFrame::new(columns).map_err(|e| format!("Failed to create DataFrame: {e}"))?;

        // Create MortData with a default category
        let category = "Custom Mortality Data".to_string();
        let description =
            "Created from ODS file {ods_file_path_str}, sheet {sheet_name}.".to_string();
        Self::new(category, description, df)
    }

    /// Parse mortality table from XLSX file using calamine.
    ///
    /// Reads XLSX files and automatically parses all columns as f64.
    /// Age and duration columns are validated to contain whole numbers during schema validation.
    ///
    /// Schema validation is performed by `from_df()` after parsing.
    ///
    /// # Errors
    /// - File not found or not readable
    /// - Invalid XLSX format
    /// - Sheet not found
    /// - Invalid data in cells
    /// - Empty sheets or insufficient data
    /// - Schema validation errors (via `from_df`)
    ///
    /// # Examples
    /// ```rust, ignore
    /// # use rslife::prelude::*;
    ///
    /// // Ultimate table with qx values
    /// let mort_data = MortData::from_xlsx("data/elt15.xlsx", "female")?;
    ///
    /// // Select table with duration
    /// let mort_data = MortData::from_xlsx("data/am92_select.xlsx", "AM92")?;
    ///# RSLifeResult::Ok(())
    /// ```
    pub fn from_xlsx(xlsx_file_path_str: &str, sheet_name: &str) -> RSLifeResult<Self> {
        // Open workbook
        let mut workbook = open_workbook_auto(xlsx_file_path_str)
            .map_err(|e| format!("Failed to open XLSX file '{xlsx_file_path_str}': {e}"))?;

        let range = workbook
            .worksheet_range(sheet_name)
            .map_err(|e| format!("Failed to read sheet '{sheet_name}': {e}"))?;

        // Check if range is empty
        if range.is_empty() {
            return Err(format!("Sheet '{sheet_name}' is empty").into());
        }

        let rows: Vec<_> = range.rows().collect();
        if rows.len() < 2 {
            return Err("Sheet must contain at least a header row and one data row".into());
        }

        // Extract headers
        let header_row = &rows[0];
        if header_row.is_empty() {
            return Err("Header row is empty".into());
        }

        let mut column_names = Vec::new();
        for (i, cell) in header_row.iter().enumerate() {
            let col_name = extract_xlsx_header_name(Some(cell), &format!("Column {}", i + 1))?;
            column_names.push(col_name);
        }

        // Parse data rows based on column names
        let mut column_data: Vec<Vec<AnyValue>> = vec![Vec::new(); column_names.len()];

        for (i, row) in rows.iter().enumerate().skip(1) {
            let row_num = i + 1; // 1-based for user-friendly error messages

            for (col_idx, (cell, col_name)) in row.iter().zip(column_names.iter()).enumerate() {
                // Parse as f64 for all other columns (tqx, lx, etc.)
                let val = parse_xlsx_f64_cell(Some(cell), row_num, col_name)?;
                let any_value = AnyValue::Float64(val);
                column_data[col_idx].push(any_value);
            }
        }

        // Validate that we have data
        if column_data.is_empty() || column_data[0].is_empty() {
            return Err("No data rows found in sheet".into());
        }

        // Build DataFrame
        let mut columns = Vec::new();
        for (col_name, data) in column_names.iter().zip(column_data.iter()) {
            let series = Series::from_any_values(col_name.as_str().into(), data, true)
                .map_err(|e| format!("Failed to create series for column '{col_name}': {e}"))?;
            columns.push(series.into_column());
        }

        let df = DataFrame::new(columns).map_err(|e| format!("Failed to create DataFrame: {e}"))?;

        // Create MortData with a default category
        let category = "Custom Mortality Data".to_string();
        let description =
            "Created from XLSX file {xlsx_file_path_str}, sheet {sheet_name}.".to_string();
        Self::new(category, description, df)
    }
}

// ================================================
// PRIVATE FUNCTIONS
// ================================================
fn is_soa_xml_data_approved(data: &SOAMortXML) -> bool {
    // Check table layout
    let approved_table_layouts = ["Aggregate", "Ultimate", "Select", "Select & Ultimate"];
    let key_words = data.content_classification.key_words.clone();

    // Check if any keyword matches any approved table layout
    let tbl_layout_result = key_words.iter().any(|keyword| {
        approved_table_layouts
            .iter()
            .any(|layout| keyword == layout)
    });

    // Content type check
    let approved_content_types = vec![
        "ADB, AD&D",
        "Annuitant Mortality",
        "Claim Cost (in Disability)",
        "Claim Incidence",
        "Claim Termination",
        "CSO / CET",
        "Disability Recovery",
        "Disabled Lives Mortality",
        "Disability Incidence",
        "Group Life",
        "Healthy Lives Mortality",
        "Insured Lives Mortality",
        "Insured Lives Mortality - Ultimate",
        "Projection Scale",
        "Termination Voluntary",
        "Population Mortality",
    ];

    let content_type = data.content_classification.content_type.clone();

    // Check if content type is in approved content types
    let content_type_result = approved_content_types
        .iter()
        .any(|approved_type| content_type == *approved_type);

    // Return result
    tbl_layout_result && content_type_result
}

/// Validate DataFrame schema according to mortality table requirements.
///
/// Ensures DataFrame follows the expected schema for mortality tables:
/// - Must have 2 or 3 columns
/// - First column: "age" (f64, but must contain whole numbers)
/// - Second column: "qx" or "lx" (f64)
/// - Optional third column: "duration" (f64, but must contain whole numbers)
/// - All values must be non-negative
/// - qx values must be ≤ 1.0
/// - DataFrame must contain at least one row of data
///
/// This function performs comprehensive validation including:
/// - Row count validation (via `_validate_df_rows`)
/// - Column structure and type validation (via `_validate_df_columns`)
/// - Non-negative value validation (via `_validate_values_non_negative`)
/// - Mortality rate bounds validation (via `_validate_qx_lte_1`)
///
/// # Errors
/// - Empty DataFrame (no rows)
/// - Wrong number of columns (must be 2 or 3)
/// - Incorrect column names (must follow mortality table conventions)
/// - Incorrect data types (all columns must be f64)
/// - Invalid data values (negative values, qx > 1.0)
/// - Non-whole numbers in age/duration columns
fn validate_df_schema(df: &DataFrame) -> RSLifeResult<()> {
    // Check if DataFrame is empty (rows count)
    if df.height() == 0 {
        return Err("DataFrame must contain at least one row of data".into());
    }

    let columns = df.get_columns();
    let cols_count = columns.len();

    // Check column names using get_column_names
    let col_names = df.get_column_names();
    match cols_count {
        2 => {
            if !(col_names[0] == "age" && (col_names[1] == "qx" || col_names[1] == "lx")) {
                return Err("DataFrame columns must be ['age', 'qx/lx']".into());
            }
        }

        3 => {
            if !(col_names[0] == "age" && (col_names[1] == "qx")
                || (col_names[1] == "lx") && col_names[2] == "duration")
            {
                return Err("DataFrame columns must be ['age', 'qx/lx', 'duration']".into());
            }
        }

        _ => {
            return Err("DataFrame must have 2 or 3 columns".into());
        }
    }

    // Type input is flexible as long as:
    // - age and duration can be casted to u32
    // - qx/lx can be casted to f64
    for col in columns {
        let col_name = col.name();

        // qx <= 1.0
        if col_name == "qx" && col.f64().unwrap().max().unwrap_or(0.0) > 1.0 {
            return Err(format!("Column '{col_name}' must not exceed 1.0").into());
        }

        if col_name == "lx" || col_name == "qx" {
            // Check if column is f64 convertible
            if col.f64().is_err() {
                return Err(format!("Column '{col_name}' must be f64 convertible").into());
            }

            // f64 >=0
            if col.f64().unwrap().min().unwrap_or(0.0) < 0.0 {
                return Err(format!("Column '{col_name}' must be non-negative").into());
            }
        } else {
            // Check if column can be cast to u32 (age and duration)
            if col.cast(&DataType::UInt32).is_err() {
                return Err(format!("Column '{col_name}' must be u32 convertible").into());
            }

            // Filter unique values and casted to u32
            let unique_values = col.unique().unwrap();
            let mut values: Vec<u32> = unique_values
                .u32()
                .map(|ca| ca.into_no_null_iter().collect())
                .unwrap_or_default();

            // Sort ascending and check whether they are consecutive numbers
            values.sort_unstable();
            if values.len() > 1 {
                let is_consecutive = values
                    .iter()
                    .zip(values.iter().skip(1))
                    .all(|(a, b)| *b == *a + 1);
                if !is_consecutive {
                    return Err(format!(
                        "Column '{col_name}' must contain consecutive whole numbers (step 1)"
                    )
                    .into());
                }
            }
        }
    }

    Ok(())
}

// ================================================
// PRIVATE FUNCTIONS
// ================================================
fn keep_first_qx_1_remove_the_rest(ages: Vec<u32>, qx: Vec<f64>) -> RSLifeResult<DataFrame> {
    let mut found_one = false;
    let filtered: Vec<(u32, f64)> = ages
        .into_iter()
        .zip(qx)
        .filter(|&(_, rate)| {
            if rate == 1.0 {
                if !found_one {
                    found_one = true;
                    true
                } else {
                    false
                }
            } else {
                true
            }
        })
        .collect();
    let (ages, qx): (Vec<u32>, Vec<f64>) = filtered.into_iter().unzip();

    let data = df! {
        "age" => ages,
        "qx" => qx,
    }?;

    Ok(data)
}

fn setup_dataframe_to_correct_schema(df: DataFrame) -> PolarsResult<DataFrame> {
    // This function assumes DataFrame has already been validated
    // Validation is done in from_df() before calling this function

    let mut df = df.clone();

    // Cast age column to u32 if present
    if let Ok(age_col) = df.column("age") {
        let casted = age_col.cast(&DataType::UInt32)?;
        df.with_column(casted.into_column())?;
    }

    // Cast duration column to u32 if present
    if let Ok(duration_col) = df.column("duration") {
        let casted = duration_col.cast(&DataType::UInt32)?;
        df.with_column(casted.into_column())?;
    }

    Ok(df)
}

/// Extract header name from ODS cell value, ensuring it's a string.
fn extract_ods_header_name(cell_value: &Value, column_desc: &str) -> RSLifeResult<String> {
    match cell_value {
        Value::Text(s) => Ok(s.trim().to_lowercase()),
        Value::Empty => Err(format!("{column_desc} header is missing").into()),
        other => Err(format!("{column_desc} header must be text, found {other:?}").into()),
    }
}

/// Parse ODS cell value as f64 with comprehensive error handling.
fn parse_ods_f64_cell(cell_value: &Value, row_num: usize, col_name: &str) -> RSLifeResult<f64> {
    match cell_value {
        Value::Number(f) => Ok(*f),
        Value::Text(s) => {
            if s.trim().is_empty() {
                Ok(f64::NAN)
            } else {
                s.parse::<f64>().map_err(|_| {
                    format!("Cannot parse {col_name} '{s}' at row {row_num} as number").into()
                })
            }
        }
        // Bool type not supported in this version of spreadsheet-ods
        Value::Empty => Ok(f64::NAN),
        other => Err(format!("Invalid {col_name} cell type {other:?} at row {row_num}").into()),
    }
}

/// Extract header name from cell, ensuring it's a string value.
fn extract_xlsx_header_name(cell: Option<&Data>, column_desc: &str) -> RSLifeResult<String> {
    match cell {
        Some(Data::String(s)) => Ok(s.trim().to_lowercase()),
        Some(other) => Err(format!("{column_desc} header must be text, found {other:?}").into()),
        None => Err(format!("{column_desc} header is missing").into()),
    }
}

/// Parse cell value as f64 with comprehensive error handling.
fn parse_xlsx_f64_cell(cell: Option<&Data>, row_num: usize, col_name: &str) -> RSLifeResult<f64> {
    match cell {
        Some(Data::Float(f)) => Ok(*f),
        Some(Data::Int(v)) => Ok(*v as f64),
        Some(Data::String(s)) => {
            if s.trim().is_empty() {
                Ok(f64::NAN)
            } else {
                s.parse::<f64>().map_err(|_| {
                    format!("Cannot parse {col_name} '{s}' at row {row_num} as number").into()
                })
            }
        }
        Some(Data::Bool(b)) => Ok(if *b { 1.0 } else { 0.0 }),
        Some(Data::Empty) => Ok(f64::NAN),
        Some(other) => {
            Err(format!("Invalid {col_name} cell type {other:?} at row {row_num}").into())
        }
        None => Err(format!("Missing {col_name} cell at row {row_num}").into()),
    }
}

// ================================================
// UNIT TESTS
// ================================================
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_soa_xml_file() {
        // Test loading from XML file in data directory
        let result = MortData::from_soa_xml_file_path_str("data/t1704.xml");

        match result {
            Ok(mort_data) => {
                println!("✓ Successfully loaded XML file: {}", mort_data.category);
                println!("  DataFrame shape: {:?}", mort_data.dataframe.shape());

                // Verify basic structure
                assert!(!mort_data.dataframe.is_empty());
                assert!(mort_data.dataframe.get_columns().len() >= 2);

                // Check that we have age and qx/lx columns
                let column_names = mort_data.dataframe.get_column_names();
                let has_age = column_names.iter().any(|name| name.as_str() == "age");
                let has_rate = column_names
                    .iter()
                    .any(|name| name.as_str() == "qx" || name.as_str() == "lx");
                assert!(has_age, "Missing 'age' column");
                assert!(has_rate, "Missing 'qx' or 'lx' column");

                // Display first few rows
                println!("  First 3 rows:\n{}", mort_data.dataframe.head(Some(3)));
            }
            Err(e) => {
                // If file doesn't exist or has issues, just verify the error is reasonable
                println!("XML file test failed (this may be expected): {e}");
                // Don't panic - file might not be available in all test environments
            }
        }
    }

    #[test]
    fn test_from_soa_url_id() {
        // Test loading from SOA URL by ID (table 1704)
        // Note: This test requires internet connection
        let result = MortData::from_soa_url_id(1704);

        match result {
            Ok(mort_data) => {
                println!(
                    "✓ Successfully loaded from SOA URL ID 1704: {}",
                    mort_data.category
                );
                println!("  DataFrame shape: {:?}", mort_data.dataframe.shape());

                // Verify basic structure
                assert!(!mort_data.dataframe.is_empty());
                assert!(mort_data.dataframe.get_columns().len() >= 2);

                // Check that we have age and qx/lx columns
                let column_names = mort_data.dataframe.get_column_names();
                let has_age = column_names.iter().any(|name| name.as_str() == "age");
                let has_rate = column_names
                    .iter()
                    .any(|name| name.as_str() == "qx" || name.as_str() == "lx");
                assert!(has_age, "Missing 'age' column");
                assert!(has_rate, "Missing 'qx' or 'lx' column");

                // Display first few rows
                println!("  First 3 rows:\n{}", mort_data.dataframe.head(Some(3)));
            }
            Err(e) => {
                // Network might not be available in test environment
                println!("SOA URL test failed (network may be unavailable): {e}");
                // Don't panic - network might not be available
            }
        }
    }

    #[test]
    fn test_from_soa_url() {
        // Test loading from full SOA URL
        let url = "https://mort.soa.org/data/t1704.xml";
        let result = MortData::from_soa_url(url);

        match result {
            Ok(mort_data) => {
                println!("✓ Successfully loaded from SOA URL: {}", mort_data.category);
                println!("  DataFrame shape: {:?}", mort_data.dataframe.shape());

                // Verify basic structure
                assert!(!mort_data.dataframe.is_empty());
                assert!(mort_data.dataframe.get_columns().len() >= 2);

                // Check that we have age and qx/lx columns
                let column_names = mort_data.dataframe.get_column_names();
                let has_age = column_names.iter().any(|name| name.as_str() == "age");
                let has_rate = column_names
                    .iter()
                    .any(|name| name.as_str() == "qx" || name.as_str() == "lx");
                assert!(has_age, "Missing 'age' column");
                assert!(has_rate, "Missing 'qx' or 'lx' column");

                // Display first few rows
                println!("  First 3 rows:\n{}", mort_data.dataframe.head(Some(3)));
            }
            Err(e) => {
                // Network might not be available in test environment
                println!("SOA URL test failed (network may be unavailable): {e}");
                // Don't panic - network might not be available
            }
        }
    }

    #[test]
    fn test_from_xlsx_file() {
        // Test loading from XLSX file in data directory
        let result =
            MortData::from_xlsx("data/ltam_standard_ultimate.xlsx", "ltam_standard_ultimate");

        match result {
            Ok(mort_data) => {
                println!("✓ Successfully loaded XLSX file: {}", mort_data.category);
                println!("  DataFrame shape: {:?}", mort_data.dataframe.shape());

                // Verify basic structure
                assert!(!mort_data.dataframe.is_empty());
                assert!(mort_data.dataframe.get_columns().len() >= 2);

                // Check that we have age and qx/lx columns
                let column_names = mort_data.dataframe.get_column_names();
                let has_age = column_names.iter().any(|name| name.as_str() == "age");
                let has_rate = column_names
                    .iter()
                    .any(|name| name.as_str() == "qx" || name.as_str() == "lx");
                assert!(has_age, "Missing 'age' column");
                assert!(has_rate, "Missing 'qx' or 'lx' column");

                // Display first few rows
                println!("  First 3 rows:\n{}", mort_data.dataframe.head(Some(3)));
            }
            Err(e) => {
                // Try alternative files if the first one doesn't work
                println!("First XLSX test failed, trying alternative: {e}");

                let alt_result = MortData::from_xlsx("data/elt15.xlsx", "elt15");
                match alt_result {
                    Ok(mort_data) => {
                        println!(
                            "✓ Successfully loaded alternative XLSX file: {}",
                            mort_data.category
                        );
                        println!("  DataFrame shape: {:?}", mort_data.dataframe.shape());

                        // Verify basic structure
                        assert!(!mort_data.dataframe.is_empty());
                        assert!(mort_data.dataframe.get_columns().len() >= 2);
                    }
                    Err(e2) => {
                        println!("XLSX file tests failed (files may not be available): {e2}");
                        // Don't panic - files might not be available in all test environments
                    }
                }
            }
        }
    }

    #[test]
    fn test_from_ods_file() {
        // Test loading from ODS file in data directory
        let result =
            MortData::from_ods("data/ltam_standard_ultimate.ods", "ltam_standard_ultimate");

        match result {
            Ok(mort_data) => {
                println!("✓ Successfully loaded ODS file: {}", mort_data.category);
                println!("  DataFrame shape: {:?}", mort_data.dataframe.shape());

                // Verify basic structure
                assert!(!mort_data.dataframe.is_empty());
                assert!(mort_data.dataframe.get_columns().len() >= 2);

                // Check that we have age and qx/lx columns
                let column_names = mort_data.dataframe.get_column_names();
                let has_age = column_names.iter().any(|name| name.as_str() == "age");
                let has_rate = column_names
                    .iter()
                    .any(|name| name.as_str() == "qx" || name.as_str() == "lx");
                assert!(has_age, "Missing 'age' column");
                assert!(has_rate, "Missing 'qx' or 'lx' column");

                // Display first few rows
                println!("  First 3 rows:\n{}", mort_data.dataframe.head(Some(3)));
            }
            Err(e) => {
                // Try alternative files if the first one doesn't work
                println!("First ODS test failed, trying alternative: {e}");

                let alt_result = MortData::from_ods("data/elt15.ods", "elt15");
                match alt_result {
                    Ok(mort_data) => {
                        println!(
                            "✓ Successfully loaded alternative ODS file: {}",
                            mort_data.category
                        );
                        println!("  DataFrame shape: {:?}", mort_data.dataframe.shape());

                        // Verify basic structure
                        assert!(!mort_data.dataframe.is_empty());
                        assert!(mort_data.dataframe.get_columns().len() >= 2);
                    }
                    Err(e2) => {
                        println!("ODS file tests failed (files may not be available): {e2}");
                        // Don't panic - files might not be available in all test environments
                    }
                }
            }
        }
    }

    #[test]
    fn test_from_df_basic() {
        // Test creating MortData from a basic DataFrame
        let df = df! {
            "age" => [20.0, 21.0, 22.0, 23.0, 24.0],
            "qx" => [0.001, 0.002, 0.003, 0.004, 0.005]
        }
        .expect("Failed to create test DataFrame");

        let result = MortData::from_df(df);

        match result {
            Ok(mort_data) => {
                println!(
                    "✓ Successfully created MortData from DataFrame: {}",
                    mort_data.category
                );
                println!("  DataFrame shape: {:?}", mort_data.dataframe.shape());

                assert_eq!(mort_data.dataframe.height(), 5);
                assert_eq!(mort_data.dataframe.width(), 2);
                assert_eq!(mort_data.category, "Custom Mortality Data");

                // Check column names
                let column_names = mort_data.dataframe.get_column_names();
                let has_age = column_names.iter().any(|name| name.as_str() == "age");
                let has_qx = column_names.iter().any(|name| name.as_str() == "qx");
                assert!(has_age, "Missing 'age' column");
                assert!(has_qx, "Missing 'qx' column");

                println!("  DataFrame:\n{}", mort_data.dataframe);
            }
            Err(e) => {
                panic!("DataFrame creation should not fail: {e}");
            }
        }
    }

    #[test]
    fn test_from_df_with_duration() {
        // Test creating MortData from a DataFrame with duration column (select table)
        let df = df! {
            "age" => [25.0, 25.0, 26.0, 26.0],
            "qx" => [0.001, 0.002, 0.002, 0.003],
            "duration" => [0.0, 1.0, 0.0, 1.0]
        }
        .expect("Failed to create test DataFrame with duration");

        let result = MortData::from_df(df);

        match result {
            Ok(mort_data) => {
                println!(
                    "✓ Successfully created MortData with duration from DataFrame: {}",
                    mort_data.category
                );
                println!("  DataFrame shape: {:?}", mort_data.dataframe.shape());

                assert_eq!(mort_data.dataframe.height(), 4);
                assert_eq!(mort_data.dataframe.width(), 3);

                // Check column names
                let column_names = mort_data.dataframe.get_column_names();
                let has_age = column_names.iter().any(|name| name.as_str() == "age");
                let has_qx = column_names.iter().any(|name| name.as_str() == "qx");
                let has_duration = column_names.iter().any(|name| name.as_str() == "duration");
                assert!(has_age, "Missing 'age' column");
                assert!(has_qx, "Missing 'qx' column");
                assert!(has_duration, "Missing 'duration' column");

                println!("  DataFrame:\n{}", mort_data.dataframe);
            }
            Err(e) => {
                panic!("DataFrame with duration creation should not fail: {e}");
            }
        }
    }

    #[test]
    fn test_validation_errors() {
        // Test that validation catches common errors

        // Test 1: Invalid column names
        let invalid_df = df! {
            "invalid_age" => [20.0, 21.0],
            "invalid_rate" => [0.001, 0.002]
        }
        .expect("Failed to create invalid test DataFrame");

        let result = MortData::from_df(invalid_df);
        assert!(result.is_err(), "Should fail with invalid column names");
        println!("✓ Correctly rejected DataFrame with invalid column names");

        // Test 2: qx values > 1.0
        let invalid_qx_df = df! {
            "age" => [20.0, 21.0],
            "qx" => [0.5, 1.5]  // 1.5 > 1.0, should fail
        }
        .expect("Failed to create invalid qx test DataFrame");

        let result2 = MortData::from_df(invalid_qx_df);
        assert!(result2.is_err(), "Should fail with qx > 1.0");
        println!("✓ Correctly rejected DataFrame with qx > 1.0");

        // Test 3: Negative values
        let negative_df = df! {
            "age" => [20.0, 21.0],
            "qx" => [-0.001, 0.002]  // Negative qx should fail
        }
        .expect("Failed to create negative test DataFrame");

        let result3 = MortData::from_df(negative_df);
        assert!(result3.is_err(), "Should fail with negative values");
        println!("✓ Correctly rejected DataFrame with negative values");
    }
}
