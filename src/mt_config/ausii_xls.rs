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
        let range = fetch_xls_range_from_url(url, id)?;
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

        let range = fetch_xls_range_from_url(&url, id)?;
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

        let range = fetch_xls_range_from_url(&url, sheet_name)?;
        data_process(structure, range)
    }
}

// ================================================
// PRIVATE FUNCTIONS
// ================================================
