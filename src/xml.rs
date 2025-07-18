use polars::prelude::*;
use std::fs;
use std::path::Path;

// The following structs represent the xml elements in the XTbML file https://mort.soa.org/About.aspx
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AxisDef {
    // Corresponds to the XML element having the same name.
    pub scale_type: String,
    pub axis_name: String,
    pub min_scale_value: i32,
    pub max_scale_value: i32,
    pub increment: i32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MetaData {
    // Corresponds to the XML element having the same name.
    pub scaling_factor: f64,
    pub data_type: String,
    pub nation: String,
    pub table_description: String,
    pub axis_defs: Vec<AxisDef>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Table {
    // Corresponds to the XML element having the same name.
    pub meta_data: MetaData,
    pub values: DataFrame,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ContentClassification {
    // Corresponds to the XML element having the same name.
    pub table_identity: i32,
    pub provider_domain: String,
    pub provider_name: String,
    pub table_reference: String,
    pub content_type: String,
    pub table_name: String,
    pub table_description: String,
    pub comments: String,
    pub key_words: Vec<String>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MortXML {
    // A Rust wrapper for XML mortality tables.
    // Corresponds to the `XTbML` root element in the XML file.
    pub content_classification: ContentClassification,
    pub tables: Vec<Table>,
}

impl MortXML {
    pub fn from_string(xml_str: &str) -> Result<Self, Box<dyn std::error::Error>> {
        // Parse XML string and create MortXML object
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

    pub fn from_id(id: i32) -> Result<Self, Box<dyn std::error::Error>> {
        // Load XML file by table ID
        let filename = format!("src/table_xml/t{id}.xml");
        let xml_str = fs::read_to_string(filename)?;
        Self::from_string(&xml_str)
    }

    pub fn from_path(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        // Load XML file from path
        let xml_str = fs::read_to_string(path)?;
        Self::from_string(&xml_str)
    }

    pub fn from_url(url: &str) -> Result<Self, Box<dyn std::error::Error>> {
        // Load XML file from URL
        let xml_str = reqwest::blocking::get(url)?.text()?;
        Self::from_string(&xml_str)
    }

    pub fn from_url_id(id: i32) -> Result<Self, Box<dyn std::error::Error>> {
        // Load XML file from https://mort.soa.org/data/t<id>.xml
        let url = format!("https://mort.soa.org/data/t{id}.xml");
        Self::from_url(&url)
    }
}

// ---------------- PRIVATE HELPER FUNCTIONS ----------------
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

fn create_table(table_node: &roxmltree::Node) -> Result<Table, Box<dyn std::error::Error>> {
    let meta_data = create_meta_data(table_node)?;
    let values = create_values(table_node)?;
    let result = Table { meta_data, values };
    Ok(result)
}

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

    // As long as there is one valid age, we add the age column
    if ages.iter().any(|age| age.is_some()) {
        columns_vec.push(Series::new("age".into(), ages.clone()).into_column());
    }

    columns_vec.push(Series::new("value".into(), values.clone()).into_column());

    if durations.iter().any(|duration| duration.is_some()) {
        columns_vec.push(Series::new("duration".into(), durations.clone()).into_column());
    }

    let columns: Vec<Column> = columns_vec.into_iter().map(|s| s.into_column()).collect();
    let df = DataFrame::new(columns)?;

    // Filter out rows where duration is null
    // Applicable only if duration column exists
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

// ---------------- UNIT TEST ----------------
#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_xml_from_id() {
        // This will call the MortXML::from_string method as well
        let result = MortXML::from_id(912);
        assert!(result.is_ok(), "Failed to load MortXML from id 912");

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
    fn test_xml_from_path() {
        use std::path::Path;

        let path = Path::new("src/table_xml/t912.xml");
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
