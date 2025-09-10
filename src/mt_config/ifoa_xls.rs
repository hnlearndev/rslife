use crate::RSLifeResult;
use crate::mt_config::spreadsheet_helpers::{parse_excel_data, parse_excel_headers};
use calamine::{Data, Reader, Xls, open_workbook_auto};
use polars::prelude::*;
use reqwest::blocking::get;
use std::io::Cursor;

/// IFOAMortXLS represents a parsed IFOA mortality table from an XLS file or URL.
///
/// This struct encapsulates the description and data for a mortality table published by the Institute and Faculty of Actuaries (IFOA).
/// It is used for actuarial calculations, including life insurance, annuities, and commutation functions.
///
/// # Fields
/// - `description`: A textual description of the table, typically extracted from the XLS file (e.g., cell A1).
/// - `dataframe`: A Polars DataFrame containing the parsed mortality data. Columns typically include:
///     - `age`: Age of the insured (integer, required)
///     - `qx`: Mortality rate (probability of death, f64, required)
///     - `duration`: Duration since entry (optional, integer)
///
/// # Data Sources
/// - IFOA official XLS files (downloaded from actuaries.org.uk)
/// - IFOA mortality table URLs (e.g., <https://www.actuaries.org.uk/documents/am92-base-mortality-table>)
/// - [`crate::mt_config::MortData`] for schema validation and actuarial usage
///
/// # Usage
/// The struct is typically constructed via one of the following methods:
/// - [`IFOAMortXLS::from_xls_file_path_str`] — Load from a local XLS file and sheet name
/// - [`IFOAMortXLS::from_url`] — Load from a direct URL to an XLS file
/// - [`IFOAMortXLS::from_url_id`] — Load from a table ID (e.g., "AM92")
///
/// # Schema Requirements
/// - DataFrame must contain at least `age` and `qx` columns
/// - All values must be non-negative
/// - `qx` values must be ≤ 1.0
/// - Age and duration columns must contain whole numbers
///
///
/// # Errors
/// - File not found or not readable
/// - Invalid XLS format or unsupported structure
/// - Sheet not found
/// - Invalid data in cells
/// - Schema validation errors (see MortData)
///
/// # See Also
/// - [`MortData`] for schema validation and actuarial usage (if imported)
/// - [`IFOAMortXLS::from_xls_file_path_str`], [`IFOAMortXLS::from_url`], [`IFOAMortXLS::from_url_id`]
pub struct IFOAMortXLS {
    pub description: String,
    pub dataframe: DataFrame,
}

impl IFOAMortXLS {
    /// Load an IFOA mortality table from a local XLS file and sheet name.
    ///
    /// This method parses the specified sheet in the given XLS file and constructs an `IFOAMortXLS` instance.
    /// The sheet name is used as the table ID to determine the parsing structure.
    ///
    /// # Parameters
    /// - `file_path`: Path to the local XLS file.
    /// - `sheet_name`: Name of the sheet to parse (also used as table ID).
    ///
    /// # Errors
    /// - File not found or not readable
    /// - Sheet not found in workbook
    /// - Invalid data or unsupported structure
    pub fn from_xls_file_path_str(file_path: &str, sheet_name: &str) -> RSLifeResult<Self> {
        // Obtain the sheet range
        // The sheet name is also the ID
        let mut workbook = open_workbook_auto(file_path)?;
        let sheet_names = workbook.sheet_names().to_owned();
        if !sheet_names.iter().any(|n| n == sheet_name) {
            return Err(format!("Sheet '{sheet_name}' not found in workbook").into());
        }
        let range = workbook.worksheet_range(sheet_name)?;
        // Obtain structure to identify correct parsing process
        let info_from_id = get_info_from_id(sheet_name)?;
        let structure = info_from_id.0;
        data_process(structure, range)
    }

    /// Load an IFOA mortality table from a direct URL to an XLS file.
    ///
    /// This method downloads the XLS file from the given URL, extracts the sheet name from the URL, and parses the data.
    ///
    /// # Parameters
    /// - `url`: Direct URL to the XLS file on the IFOA website.
    ///
    /// # Errors
    /// - Network errors or invalid URL
    /// - Sheet not found in workbook
    /// - Invalid data or unsupported structure
    pub fn from_url(url: &str) -> RSLifeResult<Self> {
        // Eg: https://www.actuaries.org.uk/documents/tm92-temporary-assurances-males
        // Extract last part of url . Eg "tm92-temporary-assurances-males"
        let full_name = url
            .split('/')
            .next_back()
            .ok_or("Invalid URL format, no sheet name found")?;
        // Extract the first part of full name which is sheet name/id Eg: "TM92"
        let id_owned = full_name
            .split('-')
            .next()
            .ok_or("Invalid URL format, no sheet name found")?
            .to_uppercase();
        let id = id_owned.as_str();
        let range = fetch_range_from_url(url, id)?;
        data_process(1, range)
    }

    /// Load an IFOA mortality table by table ID (e.g., "AM92").
    ///
    /// This method constructs the appropriate IFOA URL and parsing structure based on the table ID, downloads the XLS file, and parses the data.
    ///
    /// # Parameters
    /// - `id`: Table identifier (e.g., "AM92", "PFA92C20").
    ///
    /// # Errors
    /// - Unknown or unsupported table ID
    /// - Network errors or invalid URL
    /// - Sheet not found in workbook
    /// - Invalid data or unsupported structure
    pub fn from_url_id(id: &str) -> RSLifeResult<Self> {
        // Obtain range from the IFOA URL
        let info_from_id = get_info_from_id(id)?;
        let structure = info_from_id.0;
        let url_suffix = info_from_id.1;

        // TODO: url now also get from ONS not just IFOA
        let url = format!("https://www.actuaries.org.uk/documents/{url_suffix}");

        let id = match structure {
            1 => id,
            101 => {
                return Err(format!(
                    "{id} is not supported. Use method from_ifoa_builtin instead."
                )
                .into());
            }
            _ => return Err(format!("{id} is not supported").into()),
        };

        let range = fetch_range_from_url(&url, id)?;
        data_process(structure, range)
    }

    pub fn from_custom(id: &str) -> RSLifeResult<Self> {
        // Obtain range from the IFOA URL
        let info_from_id = get_info_from_id(id)?;
        let structure = info_from_id.0;
        let url_suffix = info_from_id.1;

        let url = format!("https://www.actuaries.org.uk/documents/{url_suffix}");

        let sheet_name = match structure {
            // PFA92C20 and PMA92C20 is using PFA92 and PMA92 sheet. ID and Sheet name are not the same.
            101 => id.strip_suffix("C20").unwrap(),
            _ => return Err(format!("{id} is not supported").into()),
        };

        let range = fetch_range_from_url(&url, sheet_name)?;
        data_process(structure, range)
    }
}

// ================================================
// PRIVATE FUNCTIONS
// ================================================

fn data_process(structure: u32, range: calamine::Range<Data>) -> RSLifeResult<IFOAMortXLS> {
    let (description, df) = match structure {
        1 => data_process_01(range),
        101 => data_process_101(range), // Custom series, same as 01 with C20 projection
        _ => Err(format!("Unsupported structure {structure}.").into()),
    }?;

    // Return the IFOAMortXLS instance
    let result = IFOAMortXLS {
        description,
        dataframe: df,
    };

    Ok(result)
}

fn data_process_01(range: calamine::Range<Data>) -> RSLifeResult<(String, DataFrame)> {
    // Extract description and headers
    let description = extract_description(&range).unwrap_or_default();
    let headers = extract_headers(&range);

    let ncols = headers.len();
    let columns = parse_excel_data(&range, 4, ncols)?;

    // The first column is age, the rest are durations
    let age_col = &columns[0];
    let mut lfs = Vec::new();
    for (i, header) in headers.iter().enumerate().skip(1) {
        let duration: u32 = header.parse().unwrap_or(0);
        let value_col = &columns[i];
        let age_col_u32: Vec<u32> = age_col.iter().map(|v| *v as u32).collect();
        let df = DataFrame::new(vec![
            Series::new("age".into(), age_col_u32).into_column(),
            Series::new("qx".into(), value_col.clone()).into_column(),
            Series::new("duration".into(), vec![duration; age_col.len()]).into_column(),
        ])?;
        lfs.push(df.lazy());
    }
    let stacked = concat(&lfs, Default::default())?.collect()?;
    let dataframe = if headers.len() == 2 {
        stacked.drop("duration")?
    } else {
        stacked
    };

    // Return result
    Ok((description, dataframe))
}

fn data_process_101(range: calamine::Range<Data>) -> RSLifeResult<(String, DataFrame)> {
    // Use process from data_process_01
    let (description, dataframe) = data_process_01(range)?;

    // Modify description
    let new_description = format!(
        "{description}\nThis is a custom series based on the 92-series base mortality tables with C20 projection."
    );

    // Project the orginal data
    let dataframe = dataframe
        .lazy()
        .with_columns(vec![
            // Add 'alpha' column using the specified piecewise logic
            when(col("age").lt(lit(60)))
                .then(lit(0.13))
                .when(col("age").gt_eq(lit(60)).and(col("age").lt_eq(lit(110))))
                .then(lit(1.0) - lit(0.87) * (lit(110.0) - col("age")) / lit(50.0))
                .when(col("age").gt_eq(lit(110)))
                .then(lit(1.0))
                .otherwise(lit(f64::NAN))
                .alias("alpha"),
            // Add column 'f' with specified piecewise logic
            when(col("age").lt(lit(60)))
                .then(lit(0.55))
                .when(col("age").gt_eq(lit(60)).and(col("age").lt_eq(lit(110))))
                .then(
                    lit(0.55) * (lit(110.0) - col("age")) / lit(50.0)
                        + lit(0.29) * (col("age") - lit(60.0)) / lit(50.0),
                )
                .when(col("age").gt_eq(lit(110)))
                .then(lit(0.29))
                .otherwise(lit(f64::NAN))
                .alias("f"),
        ])
        .with_column(
            (col("alpha")
                + (lit(1.0) - col("alpha"))
                    * (lit(1.0) - col("f")).pow(lit((2020.0 - 1992.0) / 20.0)))
            .alias("reduction_factor"),
        )
        .with_column((col("qx") * col("reduction_factor")).alias("qx_reduced"))
        .select(&[col("age"), col("qx_reduced").alias("qx")])
        .collect()?;

    // Return result
    Ok((new_description, dataframe))
}
//---------------------------------------------------------------------

fn get_info_from_id(id: &str) -> RSLifeResult<(u32, &str)> {
    // These are updated manually from the IFOA website
    match id {
        // 80-series
        "AM80" | "AF80" | "AF80(5)" | "TM80" | "PML80" | "PFL80" | "PMA80" | "PFA80" | "IM80"
        | "IF80" | "WL80" | "WA80" => Ok((1, "80-series-base-mortality-tables-complete-set")),

        // 92-series
        "AM92" | "AF92" | "TM92" | "TF92" | "IML92" | "IFL92" | "IMA92" | "IFA92" | "PML92"
        | "PFL92" | "PFA92" | "PMA92" | "WL92" | "WA92" | "RMV92" | "RFV92" => {
            Ok((1, "92-series-base-mortality-tables-complete-set"))
        }

        // 00-series
        "AMC00" | "AMS00" | "AMN00" | "AFC00" | "AFS00" | "AFN00" | "TMC00" | "TMS00" | "TMN00"
        | "TFC00" | "TFS00" | "TFN00" | "IML00" | "IFL00" | "PNML00" | "PNMA00" | "PEML00"
        | "PEMA00" | "PCML00" | "PCMA00" | "PNFL00" | "PNFA00" | "PEFL00" | "PEFA00" | "PCFL00"
        | "PCFA00" | "WL00" | "WA00" | "RMD00" | "RMV00" | "RMC00" | "RFD00" | "RFV00"
        | "RFC00" | "PPMD00" | "PPMV00" | "PPMC00" | "PPFD00" | "PPFV00" => {
            Ok((1, "00-series-base-mortality-tables-complete-set"))
        }

        // Custom series
        "PMA92C20" | "PFA92C20" => Ok((101, "92-series-base-mortality-tables-complete-set")),

        // Unsupported
        _ => Err(format!("Unknown id: {id}").into()),
    }
}

//---------------------------------------------------------------------

///// 0. Get the number of sheets in url provided
// fn get_number_of_sheets(url: &str) -> Result<usize, Box<dyn Error>> {
//     let response = get(url)?;
//     let bytes = response.bytes()?;
//     let workbook = Xls::new(Cursor::new(bytes))?;
//     let sheet_count = workbook.sheet_names().len();
//     Ok(sheet_count)
// }

/// 1. Retrieve the data from a URL and return the calamine::Range<Data> for the first sheet
fn fetch_range_from_url(url: &str, sheet_name: &str) -> RSLifeResult<calamine::Range<Data>> {
    let response = get(url)?;
    let bytes = response.bytes()?;
    let mut workbook = Xls::new(Cursor::new(bytes))?;
    let sheet_names = workbook.sheet_names().to_owned();
    if !sheet_names.iter().any(|n| n == sheet_name) {
        return Err(format!("Sheet '{sheet_name}' not found in workbook").into());
    }
    let range = workbook.worksheet_range(sheet_name)?;
    Ok(range)
}

/// 2. Process the description (cell A1)
fn extract_description(range: &calamine::Range<Data>) -> Option<String> {
    range.get((0, 0)).and_then(|cell| match cell {
        Data::String(s) => Some(s.trim().to_string()),
        Data::Empty => None,
        other => Some(other.to_string()),
    })
}

/// 3. Process the header (row 3, parse until first blank)
fn extract_headers(range: &calamine::Range<Data>) -> Vec<String> {
    // Extract raw headers first
    let headers = parse_excel_headers(range, 2).unwrap_or_default();

    // Process headers: convert to canonical form
    headers
        .into_iter()
        .enumerate()
        .map(|(i, h)| {
            if i == 0 {
                // First column: Age x -> x
                "x".to_string()
            } else {
                // Duration columns: "Duration 0", "Duration 1", ..., "Durations 2+"
                let h = h.to_lowercase();
                if let Some(num) = h.strip_prefix("duration ") {
                    // e.g. "Duration 0" -> "0"
                    num.trim_end_matches('+').trim().to_string()
                } else if let Some(num) = h.strip_prefix("durations ") {
                    // e.g. "Durations 2+" -> "2"
                    num.trim_end_matches('+').trim().to_string()
                } else {
                    h
                }
            }
        })
        .collect()
}

// ================================================
// UNIT TESTS
// ================================================
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ifoa_mort_xls_am92() {
        let result = IFOAMortXLS::from_url_id("AM92");
        match result {
            Ok(xls) => {
                println!("Description: {}", xls.description);
                println!("DataFrame:\n{:?}", xls.dataframe);
            }
            Err(e) => panic!("Failed to load IFOA XLS: {e}"),
        }
    }
}
