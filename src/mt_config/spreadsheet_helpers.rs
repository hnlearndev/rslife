use crate::RSLifeResult;
use calamine::{Data, Range};
use spreadsheet_ods::Value;

// ========= ODS Using spreadsheet_ods =========

pub fn parse_ods_headers(
    sheet: &spreadsheet_ods::Sheet,
    header_row: u32, // Base 0 - identify which row is header
) -> RSLifeResult<Vec<String>> {
    let mut column_names = Vec::new();
    let mut col = 0;

    loop {
        let cell_value = sheet.value(header_row, col);
        let col_name = match cell_value {
            // Trim and convert to lowercase for consistency
            Value::Text(s) if !s.trim().is_empty() => s.trim().to_lowercase(),
            Value::Empty => break,
            // Convert every other type to string
            Value::Number(f) => f.to_string(),
            Value::DateTime(dt) => format!("{dt:?}"),
            Value::Boolean(b) => b.to_string(),
            _ => String::new(),
        };
        column_names.push(col_name);
        col += 1;
    }

    Ok(column_names)
}

pub fn parse_ods_data(
    sheet: &spreadsheet_ods::Sheet,
    start_row: usize,
    ncols: usize,
) -> RSLifeResult<Vec<Vec<f64>>> {
    // Initialize columns as Vec<Vec<f64>> for ncols
    let mut columns: Vec<Vec<f64>> = vec![Vec::new(); ncols];
    let mut row_num = start_row;

    loop {
        let mut row_vals = Vec::with_capacity(ncols);
        let mut has_data = false;

        for col in 0..ncols {
            let cell_value = sheet.value(row_num as u32, col as u32);
            let val = match parse_ods_f64_cell(cell_value, row_num + 1, &format!("col{col}")) {
                Ok(v) => v,
                Err(_) => f64::NAN,
            };
            if !val.is_nan() {
                has_data = true;
            }
            row_vals.push(val);
        }

        // If the whole row is empty, break
        if !has_data {
            break;
        }

        // Convert from row data to column data
        for (col, column) in columns.iter_mut().enumerate().take(ncols) {
            column.push(row_vals[col]);
        }

        row_num += 1;
    }

    Ok(columns)
}

/// Parse ODS cell value as f64 with comprehensive error handling.
pub fn parse_ods_f64_cell(cell_value: &Value, row_num: usize, col_name: &str) -> RSLifeResult<f64> {
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
        Value::Empty => Ok(f64::NAN),
        // Convert boolean to f64: true -> 1.0, false -> 0.0
        Value::Boolean(b) => Ok(if *b { 1.0 } else { 0.0 }),
        other => Err(format!("Invalid {col_name} cell type {other:?} at row {row_num}").into()),
    }
}

// ========= XLSX - Using Calamine=========

pub fn parse_excel_headers(
    range: &calamine::Range<Data>,
    start_row: usize, // Base 0
) -> RSLifeResult<Vec<String>> {
    // If the first cell in the header row is None, return error
    if range.get((start_row, 0)).is_none() {
        return Err("Header row is empty".into());
    }

    // Initialize
    let mut headers = Vec::new();
    let mut col = 0;

    loop {
        let cell = range.get((start_row, col));
        match cell {
            // Trim and convert to lowercase for consistency
            Some(Data::String(s)) if !s.trim().is_empty() => headers.push(s.trim().to_lowercase()),
            Some(Data::Empty) | None => return Ok(headers),
            Some(other) => headers.push(other.to_string()),
        }
        col += 1;
    }
}

pub fn parse_excel_data(
    range: &Range<Data>,
    start_row: usize,
    ncols: usize,
) -> RSLifeResult<Vec<Vec<f64>>> {
    // Initialize
    let mut columns: Vec<Vec<f64>> = vec![Vec::new(); ncols];
    let mut row_num = start_row; // Base 0

    // Loop until reaching a row where all cells are empty or NaN
    loop {
        let mut row_vals = Vec::with_capacity(ncols);
        let mut has_data = false; // Initialize as has no data

        for col in 0..ncols {
            let cell = range.get((row_num, col));
            let val = match parse_excel_f64_cell(cell, row_num + 1, &format!("col{col}")) {
                Ok(v) => v,
                Err(_) => f64::NAN,
            };

            // There might be columns empty but other are not - turn to true once there is a value
            if !val.is_nan() {
                has_data = true;
            }

            // Push the data row by row
            row_vals.push(val);
        }

        // This will occurs when a whole row is empty
        if !has_data {
            break;
        }

        // Convert from row data to column data
        for (col, column) in columns.iter_mut().enumerate().take(ncols) {
            column.push(row_vals[col]);
        }

        row_num += 1;
    }

    // Return the columns
    Ok(columns)
}

/// Parse calamine::Data cell value as f64 with comprehensive error handling.
/// Used for both XLS and XLSX formats.
fn parse_excel_f64_cell(
    cell: Option<&calamine::Data>,
    row_num: usize,
    col_name: &str,
) -> RSLifeResult<f64> {
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
