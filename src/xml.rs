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
//! // Load 2017 CSO mortality table from SOA
//! let xml = MortXML::from_url_id(1704)?;
//! let table = &xml.tables[0];
//!
//! println!("Table: {}", xml.content_classification.table_name);
//! println!("Rows: {}", table.values.height());
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! ## Loading Methods
//!
//! - [`MortXML::from_url_id()`] - Direct download from SOA (recommended)
//! - [`MortXML::from_path()`] - Load from local file
//! - [`MortXML::from_url()`] - Load from any URL
//! - [`MortXML::from_string()`] - Parse XML string
//!
//! ## Data Structure
//!
//! Each mortality table contains:
//! - **DataFrame**: Columnar data (age, value, optional duration)
//! - **Metadata**: Scaling factors, descriptions, axis definitions
//! - **Classification**: Table ID, provider, keywords for discovery

use polars::prelude::*;
use std::fs;
use std::path::Path;

/// XTbML axis definition for table dimensions.
///
/// Defines the structure of table axes (typically age and duration).
///
/// # Example
/// ```rust
/// use rslife::xml::AxisDef;
///
/// let age_axis = AxisDef {
///     scale_type: "Age".to_string(),
///     axis_name: "Age".to_string(),
///     min_scale_value: 0,
///     max_scale_value: 120,
///     increment: 1,
/// };
/// ```
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AxisDef {
    /// The type of scale (e.g., "Age", "Duration", "AttainedAge")
    pub scale_type: String,
    /// Human-readable name for this axis
    pub axis_name: String,
    /// Minimum value on this axis (inclusive)
    pub min_scale_value: i32,
    /// Maximum value on this axis (inclusive)
    pub max_scale_value: i32,
    /// Step size between consecutive values
    pub increment: i32,
}

/// Mortality table metadata container.
///
/// Contains scaling factors, data type information, and axis definitions
/// required for proper interpretation of table values.
///
/// # Example
/// ```rust
/// use rslife::xml::MetaData;
///
/// // Metadata indicates values are scaled by 1 million
/// let metadata = MetaData {
///     scaling_factor: 1000000.0,
///     data_type: "Mortality Rate".to_string(),
///     nation: "United States".to_string(),
///     table_description: "2017 CSO Mortality Table".to_string(),
///     axis_defs: vec![],
/// };
/// ```
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
///
/// # Example
/// ```rust
/// use rslife::prelude::*;
/// use polars::prelude::{col, IntoLazy};
///
/// let xml = MortXML::from_url_id(1704)?;
/// let table = &xml.tables[0];
///
/// // Query mortality rates for age 65
/// let rates_65 = table.values.clone()
///     .lazy()
///     .filter(col("age").eq(65))
///     .collect()?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
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
///
/// # Example
/// ```rust
/// use rslife::prelude::*;
///
/// let xml = MortXML::from_url_id(1704)?;
/// let classification = &xml.content_classification;
///
/// println!("Table ID: {}", classification.table_identity);
/// println!("Name: {}", classification.table_name);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
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
///
/// # Table IDs
/// Find table IDs at [mort.soa.org](https://mort.soa.org/Default.aspx).
/// Popular tables: 1704 (2017 CSO), 912 (1980 CSO), 1076 (2001 VBT).
///
/// # Example
/// ```rust
/// use rslife::prelude::*;
///
/// // Load 2017 CSO mortality table
/// let xml = MortXML::from_url_id(1704)?;
/// let table = &xml.tables[0];
///
/// println!("Table: {}", xml.content_classification.table_name);
/// println!("Rows: {}", table.values.height());
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
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
    /// ```rust
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
        let result = MortXML {
            content_classification,
            tables,
        };
        Ok(result)
    }

    /// Load mortality table from local filesystem by table ID.
    ///
    /// Loads from `src/table_xml/t{id}.xml`. Used for development/testing.
    ///
    /// # Errors
    /// - File not found
    /// - Permission errors
    /// - Invalid XML content
    ///
    /// # Example
    /// ```rust,ignore
    /// // Requires src/table_xml/t1704.xml to exist
    /// use std::path::Path;
    /// let mort_xml = MortXML::from_path(Path::new("table_xml/t1704.xml"))?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn from_id(id: i32) -> Result<Self, Box<dyn std::error::Error>> {
        let filename = format!("src/table_xml/t{id}.xml");
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
    /// ```rust,ignore
    /// use std::path::Path;
    ///
    /// let mort_xml = MortXML::from_path(Path::new("table_xml/t1704.xml"))?;
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
    /// ```rust
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
    /// ```rust
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
        .and_then(|t| t.parse::<i32>().ok())
        .unwrap_or(0);

    let max_scale_value = axis_def_node
        .descendants()
        .find(|n| n.tag_name().name() == "MaxScaleValue")
        .and_then(|n| n.text())
        .and_then(|t| t.parse::<i32>().ok())
        .unwrap_or(0);

    let increment = axis_def_node
        .descendants()
        .find(|n| n.tag_name().name() == "Increment")
        .and_then(|n| n.text())
        .and_then(|t| t.parse::<i32>().ok())
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
    let mut ages: Vec<Option<i32>> = Vec::new();
    let mut durations: Vec<Option<i32>> = Vec::new();
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

    if ages.iter().any(|age| age.is_some()) {
        columns_vec.push(Series::new("age".into(), ages.clone()).into_column());
    }

    columns_vec.push(Series::new("value".into(), values.clone()).into_column());

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
        df.lazy().filter(col("duration").is_not_null()).collect()?
    } else {
        df
    };

    Ok(result)
}

type AxisValues = (Vec<Option<i32>>, Vec<Option<i32>>, Vec<f64>);

/// Extract mortality values from single Axis element.
/// Returns (ages, durations, values) where durations may be None for 1D tables.
fn get_axis_values(axis_node: &roxmltree::Node) -> Result<AxisValues, Box<dyn std::error::Error>> {
    let mut ages: Vec<Option<i32>> = Vec::new();
    let mut durations: Vec<Option<i32>> = Vec::new();
    let mut values: Vec<f64> = Vec::new();

    let row_t = axis_node.attribute("t").and_then(|t| t.parse::<i32>().ok());

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

        let col_t = node.attribute("t").and_then(|t| t.parse::<i32>().ok());

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
    fn test_xml_from_url_id() {
        // This will call the MortXML::from_url method as well
        let result = MortXML::from_url_id(912);
        assert!(result.is_ok(), "Failed to load MortXML from URL ID");

        let mort_xml = result.unwrap();
        assert!(mort_xml.tables.len() > 0, "No tables loaded from URL ID");

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
    #[ignore]
    fn test_xml_from_id() {
        // This will call the MortXML::from_string method as well
        let result = MortXML::from_id(1704);
        assert!(result.is_ok(), "Failed to load MortXML from id 1704");

        let mort_xml = result.unwrap();
        assert!(mort_xml.tables.len() > 0, "No tables loaded from ID");

        let df = &mort_xml.tables[0].values;
        assert!(df.height() > 0, "DataFrame is empty");
        assert!(df.column("value").is_ok(), "No 'value' column");

        // Check a random value (first row)
        let values = df.column("value").unwrap();
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
        println!("First value: {:?}", first_value);
    }

    #[test]
    #[ignore]
    fn test_xml_from_path() {
        use std::path::Path;

        let path = Path::new("src/table_xml/t1704.xml");
        let result = MortXML::from_path(path);
        assert!(result.is_ok(), "Failed to load MortXML from path");

        let mort_xml = result.unwrap();
        assert!(mort_xml.tables.len() > 0, "No tables loaded from path");

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
}
