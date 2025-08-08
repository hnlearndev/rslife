//! # XTbML Mortality Table Parser
//!
//! Parse Society of Actuaries (SOA) XTbML mortality tables into structured Rust types
//! with high-performance Polars DataFrames.
//!
//! ## Loading Methods
//!
//! - [`SOAMortXML::from_url_id()`] - Direct download from SOA (recommended)
//! - [`SOAMortXML::from_path()`] - Load from local file
//! - [`SOAMortXML::from_url()`] - Load from any URL
//! - [`SOAMortXML::from_string()`] - Parse XML string
//! - [`SOAMortXML::from_df()`] - Create from custom DataFrame (development/testing)
//!
//! ## Data Structure
//!
//! Each mortality table contains:
//! - **DataFrame**: Columnar data (age, value, optional duration)
//! - **Metadata**: Scaling factors, descriptions, axis definitions
//! - **Classification**: Table ID, provider, keywords for discovery

use crate::RSLifeResult;
use polars::prelude::*;

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
#[derive(Debug, Clone, Default)]
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
#[derive(Debug, Clone, Default)]
pub struct SOAMortXML {
    /// Table identification and classification metadata
    pub content_classification: ContentClassification,
    /// One or more mortality tables contained in this XML document
    pub tables: Vec<Table>,
}

impl SOAMortXML {
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
    /// use rslife::xml::SOAMortXML;
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
    /// let mort_xml = SOAMortXML::from_string(xml_content)?;
    /// assert_eq!(mort_xml.content_classification.table_identity, 1704);
    ///# RSLifeResult::Ok(())
    /// ```
    pub fn from_string(xml_str: &str) -> RSLifeResult<Self> {
        let doc = roxmltree::Document::parse(xml_str)?;
        let root = doc.root_element();
        let content_classification = create_content_classification(&root)?;
        let tables = create_tables(&root)?;

        if tables.len() != 1 {
            return Err("SOAMortXML must contain exactly one table".into());
        }

        let result = SOAMortXML {
            content_classification,
            tables,
        };

        Ok(result)
    }
}

//-----------------------------------------------------------------
// PRIVATE PARSING FUNCTIONS
//-----------------------------------------------------------------

/// Parse ContentClassification from XTbML root element.
/// Extracts table metadata including identity, provider, and descriptive fields.
fn create_content_classification(root: &roxmltree::Node) -> RSLifeResult<ContentClassification> {
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
fn create_tables(root: &roxmltree::Node) -> RSLifeResult<Vec<Table>> {
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
fn create_table(table_node: &roxmltree::Node) -> RSLifeResult<Table> {
    let meta_data = create_meta_data(table_node)?;
    let values = create_values(table_node)?;
    let result = Table { meta_data, values };
    Ok(result)
}

/// Parse MetaData element containing scaling factor, data type, and axis definitions.
fn create_meta_data(table_node: &roxmltree::Node) -> RSLifeResult<MetaData> {
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
fn create_axis_def(axis_def_node: &roxmltree::Node) -> RSLifeResult<AxisDef> {
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
fn create_values(table_node: &roxmltree::Node) -> RSLifeResult<DataFrame> {
    let mut ages: Vec<Option<f64>> = Vec::new();
    let mut durations: Vec<Option<f64>> = Vec::new();
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

    // ages vector - convert to f64 for consistency
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

    // durations vector (optional) - convert to f64 for consistency
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

type AxisValues = (Vec<Option<f64>>, Vec<Option<f64>>, Vec<f64>);

/// Extract mortality values from single Axis element.
/// Returns (ages, durations, values) where durations may be None for 1D tables.
fn get_axis_values(axis_node: &roxmltree::Node) -> RSLifeResult<AxisValues> {
    let mut ages: Vec<Option<f64>> = Vec::new();
    let mut durations: Vec<Option<f64>> = Vec::new();
    let mut values: Vec<f64> = Vec::new();

    let row_t = axis_node.attribute("t").and_then(|t| t.parse::<f64>().ok());

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

        let col_t = node.attribute("t").and_then(|t| t.parse::<f64>().ok());

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
mod tests {}
