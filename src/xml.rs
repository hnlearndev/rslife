//! # XTbML Mortality Table Parser
//!
//! Parse Society of Actuaries (SOA) XTbML mortality tables into structured Rust types
//! with high-performance Polars DataFrames.
//!
//! ## Quick Start
//!
//! ```rust
//! use rslife::prelude::*;
//!
//! // Load ELT 15 Female mortality table from SOA
//! let xml = MortXML::from_url_id(1704)?;
//! let table = &xml.tables[0];
//!
//! println!("Table: {}", xml.content_classification.table_name);
//! println!("Rows: {}", table.values.height());
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! ## Custom DataFrame Example
//!
//! ```rust
//! use rslife::prelude::*;
//! use polars::prelude::*;
//!
//! // Create custom mortality data
//! let df = df! {
//!     "age" => [25, 26],
//!     "qx" => [0.0015, 0.0018],
//! }?;
//!
//! // Convert to mortality table
//! let xml = MortXML::from_df(df)?;
//! let config = MortTableConfig {
//!     xml,
//!     radix: Some(100_000),
//!     pct: Some(1.0),
//!     int_rate: None,
//!     assumption: None,
//! };
//!
//! // Generate life table
//! let table = config.gen_mort_table(1)?;
//! println!("Custom table rows: {}", table.height());
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! ## Loading Methods
//!
//! - [`MortXML::from_url_id()`] - Direct download from SOA (recommended)
//! - [`MortXML::from_path()`] - Load from local file
//! - [`MortXML::from_url()`] - Load from any URL
//! - [`MortXML::from_string()`] - Parse XML string
//! - [`MortXML::from_df()`] - Create from custom DataFrame (development/testing)
//!
//! ## Data Structure
//!
//! Each mortality table contains:
//! - **DataFrame**: Columnar data (age, value, optional duration)
//! - **Metadata**: Scaling factors, descriptions, axis definitions
//! - **Classification**: Table ID, provider, keywords for discovery

use calamine::{Data, Reader, open_workbook_auto};
use polars::prelude::*;
use spreadsheet_ods::{Value, read_ods};
use std::fs;
use std::path::Path;

/// XTbML axis definition for table dimensions.
///
/// Defines the structure of table axes (typically age and duration).
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AxisDef {
    /// The type of scale (e.g., "Age", "Duration", "AttainedAge")
    pub scale_type: String,
    /// Human-readable name for this axis
    pub axis_name: String,
    /// Minimum value on this axis (inclusive)
    pub min_scale_value: u32,
    /// Maximum value on this axis (inclusive)
    pub max_scale_value: u32,
    /// Step size between consecutive values
    pub increment: u32,
}

/// Mortality table metadata container.
///
/// Contains scaling factors, data type information, and axis definitions
/// required for proper interpretation of table values.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MetaData {
    /// Scaling factor applied to all table values (typically 1.0, 1000.0, or 1000000.0)
    pub scaling_factor: f64,
    /// Type of data contained in the table (e.g., "Mortality Rate", "Disability Rate")
    pub data_type: String,
    /// Country or region for which the table applies
    pub nation: String,
    /// Human-readable description of the table contents and purpose
    pub table_description: String,
    /// Definitions of all axes/dimensions in the table
    pub axis_defs: Vec<AxisDef>,
}

/// Mortality table with metadata and data.
///
/// Contains table metadata and mortality rates in a high-performance DataFrame.
/// DataFrame columns: `age`, `value`, and optionally `duration`.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Table {
    /// Descriptive metadata for this table
    pub meta_data: MetaData,
    /// Mortality rate data in columnar format for efficient access
    pub values: DataFrame,
}

/// Table identification and classification information.
///
/// Contains table metadata for discovery and regulatory compliance.
/// The `table_identity` field is used with loading functions.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ContentClassification {
    /// Unique numeric identifier for this table (used in SOA URLs and references)
    pub table_identity: i32,
    /// Domain or organization responsible for table maintenance
    pub provider_domain: String,
    /// Name of the organization that created/maintains this table
    pub provider_name: String,
    /// Official reference or citation for this table
    pub table_reference: String,
    /// Category of actuarial data (e.g., "Mortality", "Disability", "Morbidity")
    pub content_type: String,
    /// Official table name (e.g., "2017 CSO Mortality Table")
    pub table_name: String,
    /// Detailed description of table purpose, scope, and methodology
    pub table_description: String,
    /// Additional explanatory notes or usage guidance
    pub comments: String,
    /// Searchable keywords for table discovery and categorization
    pub key_words: Vec<String>,
}

/// XTbML mortality table parser and container.
///
/// Primary interface for loading SOA mortality tables from various sources.
/// Each instance contains table identification and one or more mortality tables.
///
/// # Loading Methods
/// - [`from_url_id()`](Self::from_url_id) - Download from SOA by table ID (recommended)
/// - [`from_path()`](Self::from_path) - Load from local file
/// - [`from_url()`](Self::from_url) - Load from any URL
/// - [`from_string()`](Self::from_string) - Parse XML string
/// - [`from_df()`](Self::from_df) - Create from custom DataFrame (development/testing)
///
/// # Table IDs
/// Find table IDs at [mort.soa.org](https://mort.soa.org/Default.aspx).
/// Popular tables: 1704 (2017 CSO), 912 (1980 CSO), 1076 (2001 VBT).
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MortXML {
    /// Table identification and classification metadata
    pub content_classification: ContentClassification,
    /// One or more mortality tables contained in this XML document
    pub tables: Vec<Table>,
}

impl MortXML {
    /// Parse XTbML XML string into structured mortality table data.
    ///
    /// Core parsing method used by all other loading functions.
    ///
    /// # Errors
    /// - Invalid XML format
    /// - Missing required XTbML elements
    /// - Malformed numeric data
    ///
    /// # Example
    /// ```rust, ignore
    /// use rslife::xml::MortXML;
    ///
    /// let xml_content = r#"<?xml version="1.0" encoding="utf-8"?>
    /// <XTbML>
    ///   <ContentClassification>
    ///     <TableIdentity>1704</TableIdentity>
    ///     <ProviderDomain>soa.org</ProviderDomain>
    ///     <ProviderName>Society of Actuaries</ProviderName>
    ///     <TableReference>2017 CSO Mortality Table</TableReference>
    ///     <ContentType>Mortality</ContentType>
    ///     <TableName>2017 CSO Mortality Table</TableName>
    ///     <TableDescription>Test table</TableDescription>
    ///     <Comments>Test comments</Comments>
    ///   </ContentClassification>
    ///   <Table>
    ///     <MetaData>
    ///       <ScalingFactor>1000000</ScalingFactor>
    ///       <DataType>Mortality Rate</DataType>
    ///       <Nation>United States</Nation>
    ///       <TableDescription>Test mortality rates</TableDescription>
    ///     </MetaData>
    ///     <Axis>
    ///       <Y t="0">0.006271</Y>
    ///       <Y t="1">0.000430</Y>
    ///     </Axis>
    ///   </Table>
    /// </XTbML>"#;
    /// let mort_xml = MortXML::from_string(xml_content)?;
    /// assert_eq!(mort_xml.content_classification.table_identity, 1704);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn from_string(xml_str: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let doc = roxmltree::Document::parse(xml_str)?;
        let root = doc.root_element();
        let content_classification = create_content_classification(&root)?;
        let tables = create_tables(&root)?;

        if tables.len() != 1 {
            return Err("MortXML must contain exactly one table".into());
        }

        let result = MortXML {
            content_classification,
            tables,
        };
        Ok(result)
    }

    /// Load mortality table from local filesystem by table ID.
    ///
    /// Loads from `data/t{id}.xml`. Used for development/testing.
    ///
    /// # Errors
    /// - File not found
    /// - Permission errors
    /// - Invalid XML content
    ///
    /// # Example
    /// ```rust, ignore
    /// use rslife::xml::MortXML;
    ///
    /// // Requires data/t1704.xml to exist
    /// let mort_xml = MortXML::from_id(1704)?;
    /// println!("Table: {}", mort_xml.content_classification.table_name);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn from_id(id: i32) -> Result<Self, Box<dyn std::error::Error>> {
        let filename = format!("data/t{id}.xml");
        let xml_str = fs::read_to_string(filename)?;
        Self::from_string(&xml_str)
    }

    /// Load mortality table from filesystem path.
    ///
    /// # Errors
    /// - Path does not exist
    /// - Permission errors
    /// - Invalid XML content
    ///
    /// # Example
    /// ```rust, ignore
    /// use std::path::Path;
    /// use rslife::xml::MortXML;
    ///
    /// let mort_xml = MortXML::from_path(Path::new("data/t1704.xml"))?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn from_path(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let xml_str = fs::read_to_string(path)?;
        Self::from_string(&xml_str)
    }

    /// Load mortality table from URL.
    ///
    /// Downloads and parses XTbML files from web URLs using blocking I/O.
    ///
    /// # Errors
    /// - Network connectivity issues
    /// - HTTP error responses (404, 500, etc.)
    /// - Invalid XML content
    ///
    /// # Example
    /// ```rust, ignore
    /// use rslife::xml::MortXML;
    ///
    /// let url = "https://mort.soa.org/data/t1704.xml";
    /// let mort_xml = MortXML::from_url(url)?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn from_url(url: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let xml_str = reqwest::blocking::get(url)?.text()?;
        Self::from_string(&xml_str)
    }

    /// Load mortality table from SOA website by table ID.
    ///
    /// **Recommended method** for accessing official SOA mortality tables.
    /// Downloads from `https://mort.soa.org/data/t{id}.xml`.
    ///
    /// # Popular Table IDs
    /// - `1704`: 2017 CSO Mortality Table
    /// - `912`: 1980 CSO Basic Table
    /// - `1076`: 2001 VBT Mortality Table
    ///
    /// Find more IDs at [mort.soa.org](https://mort.soa.org/Default.aspx).
    ///
    /// # Errors
    /// - Network connectivity issues
    /// - Invalid table ID (404 response)
    /// - SOA server errors
    ///
    /// # Example
    /// ```rust, ignore
    /// use rslife::xml::MortXML;
    ///
    /// // Load mortality table
    /// let mort_xml = MortXML::from_url_id(1704)?;
    /// println!("Table: {}", mort_xml.content_classification.table_name);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn from_url_id(id: i32) -> Result<Self, Box<dyn std::error::Error>> {
        let url = format!("https://mort.soa.org/data/t{id}.xml");
        Self::from_url(&url)
    }

    /// Create mortality table from existing Polars DataFrame.
    ///
    /// **Development method** for creating mortality tables from custom data sources
    /// or computed values. Wraps a DataFrame in the standard XTbML structure with
    /// default metadata for compatibility with actuarial functions.
    ///
    /// ## Supported DataFrame Schemas
    ///
    /// **Ultimate Rate Tables:**
    /// - `age: i32, qx: f64` - Mortality rates by age
    /// - `age: i32, lx: f64` - Life counts by age (note: must be f64, not i32)
    ///
    /// **Select Rate Tables:**
    /// - `age: i32, qx: f64, duration: i32` - Mortality rates by age and duration since entry
    /// - `age: i32, lx: f64, duration: i32` - Life counts by age and duration since entry
    ///
    /// # Generated Metadata
    /// - **Table ID**: 0 (local table indicator)
    /// - **Scaling Factor**: 1.0 (no scaling applied)
    /// - **Provider**: "Local DataFrame"
    /// - **Data Type**: "Mortality Rate"
    ///
    /// # Use Cases
    /// - Testing with synthetic mortality data
    /// - Custom table construction from external sources
    /// - Academic research with modified rate structures
    /// - Integration with non-SOA data providers
    ///
    /// # Errors
    /// - Invalid DataFrame structure
    /// - Memory allocation failures for large tables
    ///
    /// # Example
    /// ```rust, ignore
    /// use rslife::prelude::*;
    /// use polars::prelude::*;
    ///
    /// // Create synthetic mortality table
    /// let ages = (0..121).collect::<Vec<i32>>();
    /// let values = (0..121).map(|age| {
    ///     // Simple mortality model: q_x = 0.001 * e^(age/80)
    ///     0.001 * (age as f64 / 80.0).exp()
    /// }).collect::<Vec<f64>>();
    ///
    /// let df = df! {
    ///     "age" => ages,
    ///     "qx" => values,
    /// }?;
    ///
    /// let mort_xml = MortXML::from_df(df)?;
    ///
    /// // Use with actuarial functions
    /// let config = MortTableConfig {
    ///     xml: mort_xml,
    ///     radix: Some(100_000),
    ///     pct: Some(1.0),
    ///     int_rate: Some(0.03),
    ///     assumption: Some(AssumptionEnum::UDD),
    /// };
    ///
    /// println!("Table: {}", config.xml.content_classification.table_name);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn from_df(df: DataFrame) -> Result<Self, Box<dyn std::error::Error>> {
        // Check dataframe schema first
        // - First column must be i32, named "age",
        // - Second column must be f64, named "qx" or "lx"
        // - Third column optional, must be i32 if present and named "duration"
        Self::_validate_df_schema(&df)?;

        // Create a dummy ContentClassification
        let content_classification = ContentClassification {
            table_identity: 0,
            provider_domain: "local".to_string(),
            provider_name: "Local DataFrame".to_string(),
            table_reference: "DataFrame Table".to_string(),
            content_type: "Mortality/Life table".to_string(),
            table_name: "DataFrame Table".to_string(),
            table_description: "Table created from DataFrame".to_string(),
            comments: "No comments".to_string(),
            key_words: vec![],
        };

        // Create a dummy Table with empty metadata
        let meta_data = MetaData {
            scaling_factor: 1.0,
            data_type: "Mortality Rate".to_string(),
            nation: "Local".to_string(),
            table_description: "Table created from DataFrame".to_string(),
            axis_defs: vec![],
        };

        let table = Table {
            meta_data,
            values: df,
        };

        let result = MortXML {
            content_classification,
            tables: vec![table],
        };

        Ok(result)
    }

    /// Parse mortality table from XLSX file using calamine.
    ///
    /// Reads XLSX files and automatically parses columns based on their names:
    /// - "age" and "duration" columns are parsed as i32
    /// - All other columns ("qx", "lx", etc.) are parsed as f64
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
    /// use rslife::xml::MortXML;
    ///
    /// // Ultimate table with qx values
    /// let mort_xml = MortXML::from_xlsx("data/elt15.xlsx", "female")?;
    ///
    /// // Select table with duration
    /// let mort_xml = MortXML::from_xlsx("data/am92_select.xlsx", "AM92")?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn from_xlsx(
        xlsx_file: &str,
        sheet_name: &str,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        // Open workbook
        let mut workbook = open_workbook_auto(xlsx_file)
            .map_err(|e| format!("Failed to open XLSX file '{xlsx_file}': {e}"))?;
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
            let col_name =
                Self::extract_xlsx_header_name(Some(cell), &format!("Column {}", i + 1))?;
            column_names.push(col_name);
        }

        // Parse data rows based on column names
        let mut column_data: Vec<Vec<AnyValue>> = vec![Vec::new(); column_names.len()];

        for (i, row) in rows.iter().enumerate().skip(1) {
            let row_num = i + 1; // 1-based for user-friendly error messages

            for (col_idx, (cell, col_name)) in row.iter().zip(column_names.iter()).enumerate() {
                let any_value = if col_name == "age" || col_name == "duration" {
                    // Parse as i32
                    let val = Self::_parse_xlsx_i32_cell(Some(cell), row_num, col_name)?;
                    AnyValue::Int32(val)
                } else {
                    // Parse as f64 for all other columns (qx, lx, etc.)
                    let val = Self::_parse_xlsx_f64_cell(Some(cell), row_num, col_name)?;
                    AnyValue::Float64(val)
                };
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

        // Delegate to from_df for validation and MortXML construction
        MortXML::from_df(df)
    }

    /// Parse mortality table from ODS file using spreadsheet-ods.
    ///
    /// Reads ODS files and automatically parses columns based on their names:
    /// - "age" and "duration" columns are parsed as i32
    /// - All other columns ("qx", "lx", etc.) are parsed as f64
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
    /// use rslife::xml::MortXML;
    ///
    /// // Ultimate table with qx values
    /// let mort_xml = MortXML::from_ods("data/ltam_standard_ultimate.ods", "ltam")?;
    ///
    /// // Select table with duration
    /// let mort_xml = MortXML::from_ods("data/am92_select.ods", "AM92")?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn from_ods(ods_file: &str, sheet_name: &str) -> Result<Self, Box<dyn std::error::Error>> {
        // Open ODS workbook
        let workbook =
            read_ods(ods_file).map_err(|e| format!("Failed to open ODS file '{ods_file}': {e}"))?;

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
            let col_name =
                Self::_extract_ods_header_name(cell_value, &format!("Column {}", col + 1))?;
            column_names.push(col_name);
        }

        // Parse data rows based on column names
        let mut column_data: Vec<Vec<AnyValue>> = vec![Vec::new(); column_names.len()];

        for row in 1..=max_row {
            let row_num = (row + 1) as usize; // 1-based for user-friendly error messages

            for (col_idx, col_name) in column_names.iter().enumerate() {
                let cell_value = sheet.value(row, col_idx as u32);
                let any_value = if col_name == "age" || col_name == "duration" {
                    // Parse as i32
                    let val = Self::_parse_ods_i32_cell(cell_value, row_num, col_name)?;
                    AnyValue::Int32(val)
                } else {
                    // Parse as f64 for all other columns (qx, lx, etc.)
                    let val = Self::_parse_ods_f64_cell(cell_value, row_num, col_name)?;
                    AnyValue::Float64(val)
                };
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

        // Delegate to from_df for validation and MortXML construction
        MortXML::from_df(df)
    }

    //==========================================================================
    // Private helper methods for schema validation and parsing
    //==========================================================================
    /// Validate DataFrame schema according to mortality table requirements.
    ///
    /// Ensures DataFrame follows the expected schema:
    /// - First column must be i32, named "age"
    /// - Second column must be f64, named "qx" or "lx"
    /// - Third column optional, must be i32 if present, named "duration"
    ///
    /// # Errors
    /// - Wrong number of columns (must be 2 or 3)
    /// - Incorrect column names
    /// - Incorrect data types
    /// - Empty DataFrame
    fn _validate_df_schema(df: &DataFrame) -> Result<(), Box<dyn std::error::Error>> {
        // Check if DataFrame is empty
        if df.height() == 0 {
            return Err("DataFrame must contain at least one row of data".into());
        }

        let columns = df.get_columns();
        let num_cols = columns.len();

        // Check number of columns (must be 2 or 3)
        if num_cols < 2 {
            return Err("DataFrame must have at least 2 columns (age and qx/lx)".into());
        }

        if num_cols > 3 {
            return Err(
                "DataFrame must have at most 3 columns (age, qx/lx, optional duration)".into(),
            );
        }

        // Validate first column: "age" (must be u32 for strict typing)
        let age_col = &columns[0];
        if age_col.name() != "age" {
            return Err(format!(
                "First column must be named 'age', found '{}'",
                age_col.name()
            )
            .into());
        }
        if !matches!(age_col.dtype(), DataType::UInt32) {
            return Err(format!(
                "First column 'age' must be u32 type, found {:?}",
                age_col.dtype()
            )
            .into());
        }

        // Validate second column: "qx" or "lx" (f64)
        let value_col = &columns[1];
        let value_col_name = value_col.name();
        if value_col_name != "qx" && value_col_name != "lx" {
            return Err(format!(
                "Second column must be named 'qx' or 'lx', found '{value_col_name}'"
            )
            .into());
        }
        if !matches!(value_col.dtype(), DataType::Float64) {
            return Err(format!(
                "Second column '{}' must be f64 type, found {:?}",
                value_col_name,
                value_col.dtype()
            )
            .into());
        }

        // Validate third column if present: "duration" (must be u32 for strict typing)
        if num_cols == 3 {
            let duration_col = &columns[2];
            if duration_col.name() != "duration" {
                return Err(format!(
                    "Third column must be named 'duration', found '{}'",
                    duration_col.name()
                )
                .into());
            }
            if !matches!(duration_col.dtype(), DataType::UInt32) {
                return Err(format!(
                    "Third column 'duration' must be u32 type, found {:?}",
                    duration_col.dtype()
                )
                .into());
            }
        }

        // Additional data validation
        // Check for negative ages
        let age_column = df.column("age")?;
        let age_series = age_column.as_materialized_series();
        if let Ok(Some(min_age)) = age_series.min::<i32>() {
            if min_age < 0 {
                return Err(
                    format!("Age values must be non-negative, found minimum: {min_age}").into(),
                );
            }
        }

        // Check for valid mortality/life values (should be non-negative for most cases)
        let value_column = df.column(value_col_name)?;
        let value_series = value_column.as_materialized_series();
        if value_col_name == "lx" {
            // Life counts should be non-negative
            if let Ok(Some(min_val)) = value_series.min::<f64>() {
                if min_val < 0.0 {
                    return Err(format!(
                        "Life count values (lx) must be non-negative, found minimum: {min_val}"
                    )
                    .into());
                }
            }
        } else if value_col_name == "qx" {
            // Mortality rates should be between 0 and 1
            if let Ok(Some(min_val)) = value_series.min::<f64>() {
                if min_val < 0.0 {
                    return Err(format!(
                        "Mortality rate values (qx) must be non-negative, found minimum: {min_val}"
                    )
                    .into());
                }
            }
            if let Ok(Some(max_val)) = value_series.max::<f64>() {
                if max_val > 1.0 {
                    return Err(format!(
                        "Mortality rate values (qx) must be ≤ 1.0, found maximum: {max_val}"
                    )
                    .into());
                }
            }
        }

        // Check duration values if present
        if num_cols == 3 {
            let duration_column = df.column("duration")?;
            let duration_series = duration_column.as_materialized_series();
            if let Ok(Some(min_dur)) = duration_series.min::<i32>() {
                if min_dur < 0 {
                    return Err(format!(
                        "Duration values must be non-negative, found minimum: {min_dur}"
                    )
                    .into());
                }
            }
        }

        Ok(())
    }

    /// Extract header name from ODS cell value, ensuring it's a string.
    fn _extract_ods_header_name(
        cell_value: &Value,
        column_desc: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        match cell_value {
            Value::Text(s) => Ok(s.trim().to_lowercase()),
            Value::Empty => Err(format!("{column_desc} header is missing").into()),
            other => Err(format!("{column_desc} header must be text, found {other:?}").into()),
        }
    }

    /// Parse ODS cell value as i32 with comprehensive error handling.
    fn _parse_ods_i32_cell(
        cell_value: &Value,
        row_num: usize,
        col_name: &str,
    ) -> Result<i32, Box<dyn std::error::Error>> {
        match cell_value {
            Value::Number(f) => {
                if f.is_nan() || f.is_infinite() || *f < 0.0 || *f > i32::MAX as f64 {
                    Err(
                        format!("{col_name} value {f} at row {row_num} is invalid or out of range")
                            .into(),
                    )
                } else {
                    Ok(*f as i32)
                }
            }
            Value::Text(s) => s.parse::<i32>().map_err(|_| {
                format!("Cannot parse {col_name} '{s}' at row {row_num} as integer").into()
            }),
            // Bool type not supported in this version of spreadsheet-ods
            Value::Empty => Err(format!("Missing {col_name} value at row {row_num}").into()),
            other => Err(format!("Invalid {col_name} cell type {other:?} at row {row_num}").into()),
        }
    }

    /// Parse ODS cell value as f64 with comprehensive error handling.
    fn _parse_ods_f64_cell(
        cell_value: &Value,
        row_num: usize,
        col_name: &str,
    ) -> Result<f64, Box<dyn std::error::Error>> {
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
    fn extract_xlsx_header_name(
        cell: Option<&Data>,
        column_desc: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        match cell {
            Some(Data::String(s)) => Ok(s.trim().to_lowercase()),
            Some(other) => {
                Err(format!("{column_desc} header must be text, found {other:?}").into())
            }
            None => Err(format!("{column_desc} header is missing").into()),
        }
    }

    /// Parse cell value as i32 with comprehensive error handling.
    fn _parse_xlsx_i32_cell(
        cell: Option<&Data>,
        row_num: usize,
        col_name: &str,
    ) -> Result<i32, Box<dyn std::error::Error>> {
        match cell {
            Some(Data::Int(v)) => {
                if *v < 0 || *v > i32::MAX as i64 {
                    Err(
                        format!("{col_name} value {v} at row {row_num} is out of valid range")
                            .into(),
                    )
                } else {
                    Ok(*v as i32)
                }
            }
            Some(Data::Float(f)) => {
                if f.is_nan() || f.is_infinite() || *f < 0.0 || *f > i32::MAX as f64 {
                    Err(
                        format!("{col_name} value {f} at row {row_num} is invalid or out of range")
                            .into(),
                    )
                } else {
                    Ok(*f as i32)
                }
            }
            Some(Data::String(s)) => s.parse::<i32>().map_err(|_| {
                format!("Cannot parse {col_name} '{s}' at row {row_num} as integer").into()
            }),
            Some(Data::Bool(b)) => Ok(if *b { 1 } else { 0 }),
            Some(Data::Empty) => Err(format!("Missing {col_name} value at row {row_num}").into()),
            Some(other) => {
                Err(format!("Invalid {col_name} cell type {other:?} at row {row_num}").into())
            }
            None => Err(format!("Missing {col_name} cell at row {row_num}").into()),
        }
    }

    /// Parse cell value as f64 with comprehensive error handling.
    fn _parse_xlsx_f64_cell(
        cell: Option<&Data>,
        row_num: usize,
        col_name: &str,
    ) -> Result<f64, Box<dyn std::error::Error>> {
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
}

//-----------------------------------------------------------------
// PRIVATE PARSING FUNCTIONS
//-----------------------------------------------------------------

/// Parse ContentClassification from XTbML root element.
/// Extracts table metadata including identity, provider, and descriptive fields.
fn create_content_classification(
    root: &roxmltree::Node,
) -> Result<ContentClassification, Box<dyn std::error::Error>> {
    let cc = root
        .descendants()
        .find(|n| n.tag_name().name() == "ContentClassification")
        .ok_or("ContentClassification element not found")?;

    let table_identity = cc
        .descendants()
        .find(|n| n.tag_name().name() == "TableIdentity")
        .and_then(|n| n.text())
        .and_then(|t| t.parse::<i32>().ok())
        .ok_or("TableIdentity not found or invalid")?;

    let provider_domain = cc
        .descendants()
        .find(|n| n.tag_name().name() == "ProviderDomain")
        .and_then(|n| n.text())
        .unwrap_or("")
        .to_string();

    let provider_name = cc
        .descendants()
        .find(|n| n.tag_name().name() == "ProviderName")
        .and_then(|n| n.text())
        .unwrap_or("")
        .to_string();

    let table_reference = cc
        .descendants()
        .find(|n| n.tag_name().name() == "TableReference")
        .and_then(|n| n.text())
        .unwrap_or("")
        .to_string();

    let content_type = cc
        .descendants()
        .find(|n| n.tag_name().name() == "ContentType")
        .and_then(|n| n.text())
        .unwrap_or("")
        .to_string();

    let table_name = cc
        .descendants()
        .find(|n| n.tag_name().name() == "TableName")
        .and_then(|n| n.text())
        .unwrap_or("")
        .to_string();

    let table_description = cc
        .descendants()
        .find(|n| n.tag_name().name() == "TableDescription")
        .and_then(|n| n.text())
        .unwrap_or("")
        .to_string();

    let comments = cc
        .descendants()
        .find(|n| n.tag_name().name() == "Comments")
        .and_then(|n| n.text())
        .unwrap_or("")
        .to_string();

    let key_words = cc
        .descendants()
        .filter(|n| n.tag_name().name() == "KeyWord")
        .filter_map(|n| n.text())
        .map(|s| s.to_string())
        .collect();

    let result = ContentClassification {
        table_identity,
        provider_domain,
        provider_name,
        table_reference,
        content_type,
        table_name,
        table_description,
        comments,
        key_words,
    };

    Ok(result)
}

/// Parse all Table elements from XTbML root.
/// Returns vector of tables, each containing metadata and DataFrame values.
fn create_tables(root: &roxmltree::Node) -> Result<Vec<Table>, Box<dyn std::error::Error>> {
    let mut tables = Vec::new();

    let table_nodes = root
        .descendants()
        .filter(|n| n.tag_name().name() == "Table");

    for node in table_nodes {
        let table = create_table(&node)?;
        tables.push(table);
    }

    Ok(tables)
}

/// Parse single Table element combining metadata and values into Table struct.
fn create_table(table_node: &roxmltree::Node) -> Result<Table, Box<dyn std::error::Error>> {
    let meta_data = create_meta_data(table_node)?;
    let values = create_values(table_node)?;
    let result = Table { meta_data, values };
    Ok(result)
}

/// Parse MetaData element containing scaling factor, data type, and axis definitions.
fn create_meta_data(table_node: &roxmltree::Node) -> Result<MetaData, Box<dyn std::error::Error>> {
    let metadata_node = table_node
        .descendants()
        .find(|n| n.tag_name().name() == "MetaData")
        .ok_or("MetaData element not found")?;

    let scaling_factor = metadata_node
        .descendants()
        .find(|n| n.tag_name().name() == "ScalingFactor")
        .and_then(|n| n.text())
        .and_then(|t| t.parse::<f64>().ok())
        .unwrap_or(1.0);

    let data_type = metadata_node
        .descendants()
        .find(|n| n.tag_name().name() == "DataType")
        .and_then(|n| n.text())
        .unwrap_or("")
        .to_string();

    let nation = metadata_node
        .descendants()
        .find(|n| n.tag_name().name() == "Nation")
        .and_then(|n| n.text())
        .unwrap_or("")
        .to_string();

    let table_description = metadata_node
        .descendants()
        .find(|n| n.tag_name().name() == "TableDescription")
        .and_then(|n| n.text())
        .unwrap_or("")
        .to_string();

    let mut axis_defs = Vec::new();

    let axis_def_nodes = metadata_node
        .descendants()
        .filter(|n| n.tag_name().name() == "AxisDef");

    for node in axis_def_nodes {
        let axis_def = create_axis_def(&node)?;
        axis_defs.push(axis_def);
    }

    let result = MetaData {
        scaling_factor,
        data_type,
        nation,
        table_description,
        axis_defs,
    };

    Ok(result)
}

/// Parse AxisDef element defining table dimensions (age, duration ranges).
fn create_axis_def(axis_def_node: &roxmltree::Node) -> Result<AxisDef, Box<dyn std::error::Error>> {
    let scale_type = axis_def_node
        .descendants()
        .find(|n| n.tag_name().name() == "ScaleType")
        .and_then(|n| n.text())
        .unwrap_or("")
        .to_string();

    let axis_name = axis_def_node
        .descendants()
        .find(|n| n.tag_name().name() == "AxisName")
        .and_then(|n| n.text())
        .unwrap_or("")
        .to_string();

    let min_scale_value = axis_def_node
        .descendants()
        .find(|n| n.tag_name().name() == "MinScaleValue")
        .and_then(|n| n.text())
        .and_then(|t| t.parse::<u32>().ok())
        .unwrap_or(0);

    let max_scale_value = axis_def_node
        .descendants()
        .find(|n| n.tag_name().name() == "MaxScaleValue")
        .and_then(|n| n.text())
        .and_then(|t| t.parse::<u32>().ok())
        .unwrap_or(0);

    let increment = axis_def_node
        .descendants()
        .find(|n| n.tag_name().name() == "Increment")
        .and_then(|n| n.text())
        .and_then(|t| t.parse::<u32>().ok())
        .unwrap_or(1);

    let result = AxisDef {
        scale_type,
        axis_name,
        min_scale_value,
        max_scale_value,
        increment,
    };

    Ok(result)
}

/// Parse table values into Polars DataFrame with age, duration, and value columns.
/// Handles both 1D (age only) and 2D (age+duration) table formats.
fn create_values(table_node: &roxmltree::Node) -> Result<DataFrame, Box<dyn std::error::Error>> {
    let mut ages: Vec<Option<u32>> = Vec::new();
    let mut durations: Vec<Option<u32>> = Vec::new();
    let mut values: Vec<f64> = Vec::new();

    let axis_nodes = table_node
        .descendants()
        .filter(|n| n.tag_name().name() == "Axis");

    for node in axis_nodes {
        let (axis_ages, axis_durations, axis_values) = get_axis_values(&node)?;
        ages.extend(axis_ages);
        durations.extend(axis_durations);
        values.extend(axis_values);
    }

    let mut columns_vec: Vec<Column> = Vec::new();

    // ages vector - keep as u32 for type safety
    if ages.iter().any(|age| age.is_some()) {
        columns_vec.push(Series::new("age".into(), ages.clone()).into_column());
    }

    // value vector
    let content_type = table_node
        .descendants()
        .find(|n| n.tag_name().name() == "ContentType")
        .and_then(|n| n.text())
        .unwrap_or("Mortality/Life table");

    let value_column_name = if content_type == "Life Table" {
        "lx"
    } else {
        "qx"
    };

    columns_vec.push(Series::new(value_column_name.into(), values.clone()).into_column());

    // durations vector (optional) - keep as u32 for type safety
    if durations.iter().any(|duration| duration.is_some()) {
        columns_vec.push(Series::new("duration".into(), durations.clone()).into_column());
    }

    let columns: Vec<Column> = columns_vec.into_iter().map(|s| s.into_column()).collect();
    let df = DataFrame::new(columns)?;

    let result = if df
        .get_column_names()
        .iter()
        .any(|name| name.as_str() == "duration")
    {
        df.lazy().filter(col("duration").is_not_null()).collect()? // This is due to algorithm, null values will be present
    } else {
        df
    };

    Ok(result)
}

type AxisValues = (Vec<Option<u32>>, Vec<Option<u32>>, Vec<f64>);

/// Extract mortality values from single Axis element.
/// Returns (ages, durations, values) where durations may be None for 1D tables.
fn get_axis_values(axis_node: &roxmltree::Node) -> Result<AxisValues, Box<dyn std::error::Error>> {
    let mut ages: Vec<Option<u32>> = Vec::new();
    let mut durations: Vec<Option<u32>> = Vec::new();
    let mut values: Vec<f64> = Vec::new();

    let row_t = axis_node.attribute("t").and_then(|t| t.parse::<u32>().ok());

    let y_nodes = axis_node
        .descendants()
        .filter(|n| n.tag_name().name() == "Y");

    for node in y_nodes {
        let text = node.text();

        let value = text.and_then(|t| t.parse::<f64>().ok());

        if value.is_none() {
            return Err("Invalid value in Y node".into());
        }

        let value = value.unwrap();

        let col_t = node.attribute("t").and_then(|t| t.parse::<u32>().ok());

        match row_t {
            Some(age) => {
                // Two-dimensional table
                ages.push(Some(age));
                durations.push(col_t);
                values.push(value);
            }
            None => {
                // One-dimensional table
                ages.push(col_t);
                durations.push(None);
                values.push(value);
            }
        }
    }

    Ok((ages, durations, values))
}

//-----------------------------------------------------------------
// UNIT TEST
//-----------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xml_selection_table() {
        // Show the structure of table with selection effect
        let mort_xml = MortXML::from_url_id(47).unwrap();
        let df = &mort_xml.tables[0].values;
        println!("DataFrame: {:?}", df.head(Some(10)));
        assert!(df.height() > 0, "DataFrame is empty");
    }

    #[test]
    #[ignore]
    fn test_xml_from_url_id() {
        // This will call the MortXML::from_url method as well
        let result = MortXML::from_url_id(912);
        assert!(result.is_ok(), "Failed to load MortXML from URL ID");

        let mort_xml = result.unwrap();
        assert!(!mort_xml.tables.is_empty(), "No tables loaded from URL ID");

        let df = &mort_xml.tables[0].values;
        assert!(df.height() > 0, "DataFrame is empty");
        assert!(df.column("value").is_ok(), "No 'value' column");

        // Verify content classification
        assert!(
            mort_xml.content_classification.table_identity > 0,
            "Invalid table identity"
        );

        assert!(
            !mort_xml.content_classification.table_name.is_empty(),
            "Table name is empty"
        );

        println!("Table name: {}", mort_xml.content_classification.table_name);
        println!("Number of rows: {}", df.height());
    }

    #[test]
    fn test_xml_from_id() {
        // This will call the MortXML::from_string method as well
        let result = MortXML::from_id(1704);
        assert!(result.is_ok(), "Failed to load MortXML from id 1704");

        let mort_xml = result.unwrap();
        assert!(!mort_xml.tables.is_empty(), "No tables loaded from ID");

        let df = &mort_xml.tables[0].values;
        assert!(df.height() > 0, "DataFrame is empty");
        assert!(df.column("qx").is_ok(), "No 'qx' column");

        // Check a random value (first row)
        let values = df.column("qx").unwrap();
        let first_value = values.get(0).unwrap();
        assert!(!first_value.is_null(), "First value is missing or null");

        // Verify content classification
        assert!(
            mort_xml.content_classification.table_identity > 0,
            "Invalid table identity"
        );
        assert!(
            !mort_xml.content_classification.table_name.is_empty(),
            "Table name is empty"
        );

        println!("Table name: {}", mort_xml.content_classification.table_name);
        println!("Number of rows: {}", df.height());
        println!("First value: {first_value:?}");
    }

    #[test]
    #[ignore]
    fn test_xml_from_path() {
        use std::path::Path;

        let path = Path::new("src/table_xml/t1704.xml");
        let result = MortXML::from_path(path);
        assert!(result.is_ok(), "Failed to load MortXML from path");

        let mort_xml = result.unwrap();
        assert!(!mort_xml.tables.is_empty(), "No tables loaded from path");

        let df = &mort_xml.tables[0].values;
        assert!(df.height() > 0, "DataFrame is empty");
        assert!(df.column("qx").is_ok(), "No 'qx' column");

        // Verify content classification
        assert!(
            mort_xml.content_classification.table_identity > 0,
            "Invalid table identity"
        );
        assert!(
            !mort_xml.content_classification.table_name.is_empty(),
            "Table name is empty"
        );

        println!("Table name: {}", mort_xml.content_classification.table_name);
        println!("Number of rows: {}", df.height());
    }

    #[test]
    #[ignore] // Requires xlsx file to exist
    fn test_xml_from_xlsx() {
        // Test loading from existing XLSX file with proper column names
        let result = MortXML::from_xlsx("data/elt15.xlsx", "female");

        match result {
            Ok(mort_xml) => {
                println!("✓ Successfully loaded XLSX file");
                println!(
                    "  Table name: {}",
                    mort_xml.content_classification.table_name
                );
                println!("  Rows: {}", mort_xml.tables[0].values.height());
                println!(
                    "  Columns: {:?}",
                    mort_xml.tables[0].values.get_column_names()
                );

                // Basic validation
                assert!(!mort_xml.tables.is_empty(), "No tables loaded from XLSX");
                assert!(mort_xml.tables[0].values.height() > 0, "DataFrame is empty");

                // Verify DataFrame structure
                let df = &mort_xml.tables[0].values;
                assert!(df.column("age").is_ok(), "Should have 'age' column");
                assert!(df.column("qx").is_ok(), "Should have 'qx' column");

                // Check some sample values
                let age_col = df.column("age").unwrap();
                let qx_col = df.column("qx").unwrap();

                // First age should be 0
                assert_eq!(age_col.get(0).unwrap().try_extract::<i32>().unwrap(), 0);

                // First qx should be around 0.00632 (from examine output)
                let first_qx = qx_col.get(0).unwrap().try_extract::<f64>().unwrap();
                assert!(
                    (first_qx - 0.00632).abs() < 0.0001,
                    "First qx should be approximately 0.00632"
                );

                println!(
                    "  First age: {}",
                    age_col.get(0).unwrap().try_extract::<i32>().unwrap()
                );
                println!("  First qx: {:.5}", first_qx);
            }
            Err(e) => {
                panic!("Failed to load XLSX file: {}", e);
            }
        }
    }

    #[test]
    fn test_xml_from_df() {
        use polars::prelude::*;

        // Create synthetic mortality table data
        let ages = (0..121).collect::<Vec<i32>>();
        let values = (0..121)
            .map(|age| {
                // Simple mortality model: q_x = 0.001 * e^(age/80)
                0.001 * (age as f64 / 80.0).exp()
            })
            .collect::<Vec<f64>>();

        // Create DataFrame
        let df = df! {
            "age" => ages.clone(),
            "qx" => values.clone(),
        }
        .expect("Failed to create DataFrame");

        // Test from_df method
        let result = MortXML::from_df(df);
        assert!(result.is_ok(), "Failed to create MortXML from DataFrame");

        let mort_xml = result.unwrap();

        // Verify structure
        assert_eq!(mort_xml.tables.len(), 1, "Should have exactly one table");

        let table = &mort_xml.tables[0];
        assert_eq!(
            table.values.height(),
            121,
            "Should have 121 rows (ages 0-120)"
        );
        assert!(
            table.values.column("age").is_ok(),
            "Should have 'age' column"
        );
        assert!(table.values.column("qx").is_ok(), "Should have 'qx' column");

        // Verify content classification defaults
        let classification = &mort_xml.content_classification;
        assert_eq!(
            classification.table_identity, 0,
            "Should have ID 0 for local table"
        );
        assert_eq!(classification.provider_name, "Local DataFrame");
        assert_eq!(classification.table_name, "DataFrame Table");
        assert_eq!(classification.content_type, "Mortality/Life table");

        // Verify metadata defaults
        let metadata = &table.meta_data;
        assert_eq!(
            metadata.scaling_factor, 1.0,
            "Should have scaling factor 1.0"
        );
        assert_eq!(metadata.data_type, "Mortality Rate");
        assert_eq!(metadata.nation, "Local");

        // Verify some data values
        let age_column = table.values.column("age").unwrap();
        let value_column = table.values.column("qx").unwrap();

        // Check first row (age 0)
        let first_age = age_column.get(0).unwrap();
        let first_value = value_column.get(0).unwrap();

        assert_eq!(first_age.try_extract::<i32>().unwrap(), 0);
        // First value should be 0.001 * e^(0/80) = 0.001 * 1 = 0.001
        assert!((first_value.try_extract::<f64>().unwrap() - 0.001).abs() < 1e-10);

        // Check last row (age 120)
        let last_age = age_column.get(120).unwrap();
        let last_value = value_column.get(120).unwrap();

        assert_eq!(last_age.try_extract::<i32>().unwrap(), 120);
        // Last value should be 0.001 * e^(120/80) = 0.001 * e^1.5 ≈ 0.004481
        let expected_last_value = 0.001 * (120.0_f64 / 80.0).exp();
        assert!((last_value.try_extract::<f64>().unwrap() - expected_last_value).abs() < 1e-6);

        println!("✓ Successfully created MortXML from DataFrame");
        println!("  Table name: {}", classification.table_name);
        println!("  Rows: {}", table.values.height());
        println!(
            "  First mortality rate (age 0): {:.6}",
            first_value.try_extract::<f64>().unwrap()
        );
        println!(
            "  Last mortality rate (age 120): {:.6}",
            last_value.try_extract::<f64>().unwrap()
        );
    }

    #[test]
    fn test_dataframe_schema_validation_valid_qx() {
        use polars::prelude::*;

        // Test valid DataFrame with qx column
        let df = df! {
            "age" => [25i32, 26i32, 27i32],
            "qx" => [0.0015f64, 0.0018f64, 0.0020f64],
        }
        .expect("Failed to create DataFrame");

        let result = MortXML::from_df(df);
        assert!(result.is_ok(), "Valid qx DataFrame should pass validation");
    }

    #[test]
    fn test_dataframe_schema_validation_valid_lx() {
        use polars::prelude::*;

        // Test valid DataFrame with lx column
        let df = df! {
            "age" => [25i32, 26i32, 27i32],
            "lx" => [100000.0f64, 99850.0f64, 99680.0f64],
        }
        .expect("Failed to create DataFrame");

        let result = MortXML::from_df(df);
        assert!(result.is_ok(), "Valid lx DataFrame should pass validation");
    }

    #[test]
    fn test_dataframe_schema_validation_with_duration() {
        use polars::prelude::*;

        // Test valid DataFrame with duration column
        let df = df! {
            "age" => [25i32, 26i32, 27i32],
            "qx" => [0.0015f64, 0.0018f64, 0.0020f64],
            "duration" => [0i32, 1i32, 2i32],
        }
        .expect("Failed to create DataFrame");

        let result = MortXML::from_df(df);
        assert!(
            result.is_ok(),
            "Valid DataFrame with duration should pass validation"
        );
    }

    #[test]
    fn test_dataframe_schema_validation_wrong_column_name() {
        use polars::prelude::*;

        // Test DataFrame with wrong column name
        let df = df! {
            "age" => [25i32, 26i32, 27i32],
            "value" => [0.0015f64, 0.0018f64, 0.0020f64],  // Wrong column name
        }
        .expect("Failed to create DataFrame");

        let result = MortXML::from_df(df);
        assert!(
            result.is_err(),
            "DataFrame with wrong column name should fail validation"
        );
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Second column must be named 'qx' or 'lx'"));
    }

    #[test]
    fn test_dataframe_schema_validation_wrong_age_column_name() {
        use polars::prelude::*;

        // Test DataFrame with wrong age column name
        let df = df! {
            "years" => [25i32, 26i32, 27i32],  // Wrong column name
            "qx" => [0.0015f64, 0.0018f64, 0.0020f64],
        }
        .expect("Failed to create DataFrame");

        let result = MortXML::from_df(df);
        assert!(
            result.is_err(),
            "DataFrame with wrong age column name should fail validation"
        );
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("First column must be named 'age'"));
    }

    #[test]
    fn test_dataframe_schema_validation_wrong_data_types() {
        use polars::prelude::*;

        // Test DataFrame with wrong data types
        let df = df! {
            "age" => [25.0f64, 26.0f64, 27.0f64],  // Should be i32
            "qx" => [0.0015f64, 0.0018f64, 0.0020f64],
        }
        .expect("Failed to create DataFrame");

        let result = MortXML::from_df(df);
        assert!(
            result.is_err(),
            "DataFrame with wrong age data type should fail validation"
        );
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("First column 'age' must be i32 type"));
    }

    #[test]
    fn test_dataframe_schema_validation_empty_dataframe() {
        use polars::prelude::*;

        // Test empty DataFrame
        let df = df! {
            "age" => Vec::<i32>::new(),
            "qx" => Vec::<f64>::new(),
        }
        .expect("Failed to create DataFrame");

        let result = MortXML::from_df(df);
        assert!(result.is_err(), "Empty DataFrame should fail validation");
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("DataFrame must contain at least one row of data"));
    }

    #[test]
    fn test_dataframe_schema_validation_negative_ages() {
        use polars::prelude::*;

        // Test DataFrame with negative ages
        let df = df! {
            "age" => [-1i32, 26i32, 27i32],  // Negative age
            "qx" => [0.0015f64, 0.0018f64, 0.0020f64],
        }
        .expect("Failed to create DataFrame");

        let result = MortXML::from_df(df);
        assert!(
            result.is_err(),
            "DataFrame with negative ages should fail validation"
        );
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Age values must be non-negative"));
    }

    #[test]
    fn test_dataframe_schema_validation_invalid_qx_values() {
        use polars::prelude::*;

        // Test DataFrame with qx values > 1.0
        let df = df! {
            "age" => [25i32, 26i32, 27i32],
            "qx" => [0.5f64, 1.5f64, 0.8f64],  // 1.5 > 1.0
        }
        .expect("Failed to create DataFrame");

        let result = MortXML::from_df(df);
        assert!(
            result.is_err(),
            "DataFrame with qx > 1.0 should fail validation"
        );
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Mortality rate values (qx) must be ≤ 1.0"));
    }

    #[test]
    fn test_dataframe_schema_validation_too_many_columns() {
        use polars::prelude::*;

        // Test DataFrame with too many columns
        let df = df! {
            "age" => [25i32, 26i32, 27i32],
            "qx" => [0.0015f64, 0.0018f64, 0.0020f64],
            "duration" => [0i32, 1i32, 2i32],
            "extra" => [1.0f64, 2.0f64, 3.0f64],  // Too many columns
        }
        .expect("Failed to create DataFrame");

        let result = MortXML::from_df(df);
        assert!(
            result.is_err(),
            "DataFrame with too many columns should fail validation"
        );
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("DataFrame must have at most 3 columns"));
    }
}
