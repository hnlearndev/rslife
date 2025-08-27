use crate::RSLifeResult;
use crate::mt_config::spreadsheet_helpers::{parse_excel_data, parse_excel_headers};
use calamine::{Data, Reader, Xlsx};
use polars::prelude::*;
use reqwest::blocking::get;
use std::io::Cursor;

pub struct AusGovActMortXLS {
    pub description: String,
    pub dataframe: DataFrame,
}

impl AusGovActMortXLS {
    /// Load an  Australian  mortality table from a direct URL
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
    pub fn from_url(gender: &str, period: &str) -> RSLifeResult<Self> {
        let response = get(
            "https://aga.gov.au/sites/aga.gov.au/files/2024-12/historical-mortality-rates-life-expectancies_0.xlsx",
        )?;

        let bytes = response.bytes()?;
        let mut workbook = Xlsx::new(Cursor::new(bytes))?;

        // Depend on gender input to determine which sheet to parse
        let sheet_name = match gender {
            "M" | "m" | "Male" | "male" => "Historical Male qx",
            "F" | "f" | "Female" | "female" => "Historical Female qx",
            _ => return Err(format!("Unknown gender: {}", gender).into()),
        };

        // Check if the expected sheet is present
        let sheet_names = workbook.sheet_names().to_owned();
        if !sheet_names.iter().any(|n| n == sheet_name) {
            return Err(format!("Sheet '{sheet_name}' not found in workbook").into());
        }

        // Obtain the sheet range
        let range = workbook.worksheet_range(sheet_name)?;

        // Obtain data
        let data = parse_data(&range, period)?;

        // Construct DataFrame
        let df = df! {
            "age" => &data[0],
            "qx" => &data[1],
        }?;

        // Return the IFOAMortXLS instance
        let gender_description = match gender {
            "m" | "M" | "male" | "Male" => "Male",
            "f" | "F" | "female" | "Female" => "Female",
            _ => "Unknown",
        };

        let descrription = format!(
            "Australian Goverment Actuary Mortality Data - {gender_description} - {period}"
        );

        let result = AusGovActMortXLS {
            description: descrription,
            dataframe: df,
        };

        Ok(result)
    }
}

// ================================================
// PRIVATE FUNCTIONS
// ================================================
fn parse_data(range: &calamine::Range<Data>, period: &str) -> RSLifeResult<Vec<Vec<f64>>> {
    let headers = parse_excel_headers(range, 1)?; // Header row is row 2 (0-based index 1)

    // Column index matching period
    let period_col_index = headers
        .iter()
        .position(|h| h.trim() == period)
        .ok_or_else(|| format!("Period '{period}' not found in headers"))?;

    // This will contain data age column to column of interest
    let data = parse_excel_data(range, 2, period_col_index + 1)?; // Age from row 3 to 121 (0-based index 2 to 120), column 0

    // However, we are keeping only the first and the last one
    let mut selected_data: Vec<Vec<f64>> = vec![Vec::new(); 2];
    if let Some(first_row) = data.first() {
        selected_data[0].push(first_row[0]); // Age from first row
    }

    if let Some(last_row) = data.last() {
        selected_data[1].push(last_row[period_col_index]); // qx from last row
    }

    // You may want to return selected_data or handle it as needed
    Ok(selected_data)
}

// ================================================
// UNIT TESTS
// ================================================
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_url_male_with_available_period() {
        // Load the file to get an available period first
        let response = get("https://aga.gov.au/sites/aga.gov.au/files/2024-12/historical-mortality-rates-life-expectancies_0.xlsx").unwrap();
        let bytes = response.bytes().unwrap();
        let mut workbook = Xlsx::new(Cursor::new(bytes)).unwrap();

        let range = workbook.worksheet_range("Historical Male qx").unwrap();
        let headers = parse_excel_headers(&range, 1).unwrap();

        // Use the first available period after age column
        let test_period = &headers[1]; // Skip age column (index 0)

        // Test the from_url method with Male gender and an available period
        let result = AusGovActMortXLS::from_url("Male", test_period);

        assert!(result.is_ok(), "Loading AGA mortality data should succeed");

        let aus_mort = result.unwrap();

        // Verify the basic structure
        assert!(aus_mort.description.contains("Australian"));
        assert!(aus_mort.description.contains("Male"));
        assert!(aus_mort.description.contains(test_period));

        // Verify DataFrame structure
        assert!(
            !aus_mort.dataframe.is_empty(),
            "DataFrame should not be empty"
        );
        assert_eq!(
            aus_mort.dataframe.width(),
            2,
            "Should have 2 columns: age and qx"
        );
        assert!(
            aus_mort.dataframe.height() > 0,
            "Should have at least one row of data"
        );

        // Verify column names
        let column_names = aus_mort.dataframe.get_column_names();
        assert!(
            column_names.iter().any(|name| name.as_str() == "age"),
            "Should contain 'age' column"
        );
        assert!(
            column_names.iter().any(|name| name.as_str() == "qx"),
            "Should contain 'qx' column"
        );

        println!("✓ Test passed! Successfully loaded Australian mortality data for males.");
        println!("  Description: {}", aus_mort.description);
        println!(
            "  Data shape: {} rows x {} columns",
            aus_mort.dataframe.height(),
            aus_mort.dataframe.width()
        );
    }
}
