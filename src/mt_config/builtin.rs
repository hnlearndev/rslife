use super::mt_data::MortData;
use once_cell::sync::Lazy;
use rustc_hash::FxHashMap;

// SOA XML tables embedded at compile time.
// Paths are relative to this source file.
const ELT15_F_XML: &str = include_str!("../../data/elt15_f.xml");
const ELT15_M_XML: &str = include_str!("../../data/elt15_m.xml");

/// Preloaded commonly-used mortality tables cached in an `FxHashMap`.
///
/// Loaded once at first access. Subsequent lookups are O(1) hash access
/// with no I/O overhead. All underlying data is embedded into the binary
/// via `include_bytes!` / `include_str!`, so the cache works offline and
/// does not depend on the `data/` folder at runtime.
///
/// # Loaded tables
/// - IFOA: `AM92`, `PFA92`, `PMA92` (via `from_ifoa_url_id`)
/// - IFOA: `PFA92C10`, `PMA92C10`, `PFA92C20`, `PMA92C20` (via `from_ifoa_custom`)
/// - SOA: `ELT15_F`, `ELT15_M` (embedded XML via `from_soa_xml_string`)
/// - SOA: `SULT` (via `from_soa_custom`)
pub static BUILTIN_MORT_DATA: Lazy<FxHashMap<&'static str, MortData>> = Lazy::new(|| {
    let mut map = FxHashMap::default();

    // Standard IFOA tables (embedded 92series.xls etc via ifoa_xls module)
    for id in ["AM92", "AF92", "PFA92", "PMA92"] {
        match MortData::from_ifoa_url_id(id) {
            Ok(data) => {
                map.insert(id, data);
            }
            Err(e) => panic!("Failed to preload builtin table {id}: {e}"),
        }
    }

    // Custom C10 / C20 projection tables
    for id in ["PFA92C10", "PMA92C10", "PFA92C20", "PMA92C20"] {
        match MortData::from_ifoa_custom(id) {
            Ok(data) => {
                map.insert(id, data);
            }
            Err(e) => panic!("Failed to preload builtin table {id}: {e}"),
        }
    }

    // SOA tables parsed from embedded XML
    for (key, xml_str) in [("ELT15_F", ELT15_F_XML), ("ELT15_M", ELT15_M_XML)] {
        match MortData::from_soa_xml_string(xml_str) {
            Ok(data) => {
                map.insert(key, data);
            }
            Err(e) => panic!("Failed to preload builtin table {key}: {e}"),
        }
    }

    // SOA custom tables
    for id in ["SULT"] {
        match MortData::from_soa_custom(id) {
            Ok(data) => {
                map.insert(id, data);
            }
            Err(e) => panic!("Failed to preload builtin table {id}: {e}"),
        }
    }

    map
});
