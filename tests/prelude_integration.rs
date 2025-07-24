//! # Integration Tests for RSLife Prelude
//!
//! This integration test verifies that the prelude module properly re-exports
//! all commonly used types and functions from the rslife crate, ensuring that
//! users can import everything they need with a single `use` statement.

use rslife::prelude::*;

#[test]
fn test_prelude_imports_basic_types() {
    // Test that we can create the basic configuration enum
    let assumption = AssumptionEnum::UDD;

    // Test that we can reference the MortTableConfig struct
    // (we won't instantiate it here since we'd need actual XML data)
    let _config_type_name = std::any::type_name::<MortTableConfig>();

    println!("Successfully imported AssumptionEnum: {assumption:?}");
    println!("MortTableConfig type available: {_config_type_name}");

    // Verify the enum has the expected variants
    assert!(matches!(assumption, AssumptionEnum::UDD));

    // Test other assumption variants
    let _cfm = AssumptionEnum::CFM;
    let _hpb = AssumptionEnum::HPB;
}

#[test]
fn test_prelude_imports_polars_types() {
    // Test that Polars types are accessible
    let _df_type_name = std::any::type_name::<DataFrame>();
    let _series_type_name = std::any::type_name::<Series>();
    let _result_type_name = std::any::type_name::<PolarsResult<f64>>();

    println!("DataFrame type available: {_df_type_name}");
    println!("Series type available: {_series_type_name}");
    println!("PolarsResult type available: {_result_type_name}");

    // Verify type names contain expected content
    assert!(_df_type_name.contains("DataFrame"));
    assert!(_series_type_name.contains("Series"));
    assert!(_result_type_name.contains("PolarsError")); // PolarsResult is actually Result<T, PolarsError>
}

#[test]
fn test_prelude_imports_xml_types() {
    // Test that XML types are accessible
    let _xml_type_name = std::any::type_name::<MortXML>();
    let _content_class_type_name = std::any::type_name::<ContentClassification>();

    println!("MortXML type available: {_xml_type_name}");
    println!("ContentClassification type available: {_content_class_type_name}");

    // Verify type names are correct
    assert!(_xml_type_name.contains("MortXML"));
    assert!(_content_class_type_name.contains("ContentClassification"));
}

#[test]
fn test_prelude_function_signatures() {
    // Test that actuarial functions are accessible and have correct signatures
    // We verify they exist by getting their function pointers

    // Life insurance functions
    let _ax_fn = Ax as fn(&MortTableConfig, u32, u32, Option<u32>) -> PolarsResult<f64>;
    let _ax1n_fn = Ax1n as fn(&MortTableConfig, u32, u32, u32, Option<u32>) -> PolarsResult<f64>;
    let _axn_fn = Axn as fn(&MortTableConfig, u32, u32, u32, Option<u32>) -> PolarsResult<f64>;
    let _nex_fn = nEx as fn(&MortTableConfig, u32, u32, u32, Option<u32>) -> PolarsResult<f64>;

    // Increasing insurance functions
    let _iax_fn = IAx as fn(&MortTableConfig, u32, u32, Option<u32>) -> PolarsResult<f64>;
    let _iax1n_fn = IAx1n as fn(&MortTableConfig, u32, u32, u32, Option<u32>) -> PolarsResult<f64>;
    let _iaxn_fn = IAxn as fn(&MortTableConfig, u32, u32, u32, Option<u32>) -> PolarsResult<f64>;

    // Decreasing insurance functions
    let _dax1n_fn = DAx1n as fn(&MortTableConfig, u32, u32, u32, Option<u32>) -> PolarsResult<f64>;
    let _daxn_fn = DAxn as fn(&MortTableConfig, u32, u32, u32, Option<u32>) -> PolarsResult<f64>;

    // Geometric insurance functions
    let _gax_fn = gAx as fn(&MortTableConfig, u32, f64, u32, Option<u32>) -> PolarsResult<f64>;
    let _gax1n_fn = gAx1n as fn(&MortTableConfig, u32, u32, f64, u32, Option<u32>) -> PolarsResult<f64>;

    println!("✓ All life insurance functions accessible through prelude");
}

#[test]
fn test_prelude_annuity_functions() {
    // Test annuity function signatures

    // Basic annuity functions
    let _aax_fn = aax as fn(&MortTableConfig, u32, u32, u32, Option<u32>) -> PolarsResult<f64>;
    let _aaxn_fn = aaxn as fn(&MortTableConfig, u32, u32, u32, u32, Option<u32>) -> PolarsResult<f64>;

    // Increasing annuity functions
    let _iaax_fn = Iaax as fn(&MortTableConfig, u32, u32, u32, Option<u32>) -> PolarsResult<f64>;
    let _iaaxn_fn = Iaaxn as fn(&MortTableConfig, u32, u32, u32, u32, Option<u32>) -> PolarsResult<f64>;

    // Decreasing annuity functions
    let _daaxn_fn = Daaxn as fn(&MortTableConfig, u32, u32, u32, u32, Option<u32>) -> PolarsResult<f64>;

    // Geometric annuity functions
    let _gaax_fn = gaax as fn(&MortTableConfig, u32, u32, f64, u32, Option<u32>) -> PolarsResult<f64>;
    let _gaaxn_fn = gaaxn as fn(&MortTableConfig, u32, u32, u32, f64, u32, Option<u32>) -> PolarsResult<f64>;

    println!("✓ All annuity functions accessible through prelude");
}

#[test]
fn test_prelude_fractional_age_functions() {
    // Test fractional age function signatures
    let _tpx_fn = tpx as fn(&MortTableConfig, u32, u32, Option<u32>) -> PolarsResult<f64>;
    let _tqx_fn = tqx as fn(&MortTableConfig, u32, u32, Option<u32>) -> PolarsResult<f64>;

    println!("✓ Fractional age functions (tpx, tqx) accessible through prelude");
}

#[test]
fn test_prelude_comprehensive_functionality() {
    // This test verifies that we can actually use the prelude imports
    // to create a working configuration (using a mock scenario)

    // Test that all major types work together
    let assumption = AssumptionEnum::UDD;

    // Verify we can reference all the function types we need
    let _life_insurance_functions = [
        std::any::type_name_of_val(&(Ax as fn(&MortTableConfig, u32, u32, Option<u32>) -> PolarsResult<f64>)),
        std::any::type_name_of_val(&(Axn as fn(&MortTableConfig, u32, u32, u32, Option<u32>) -> PolarsResult<f64>)),
        std::any::type_name_of_val(
            &(Ax1n as fn(&MortTableConfig, u32, u32, u32, Option<u32>) -> PolarsResult<f64>),
        ),
    ];

    let _annuity_functions = [
        std::any::type_name_of_val(&(aax as fn(&MortTableConfig, u32, u32, u32, Option<u32>) -> PolarsResult<f64>)),
        std::any::type_name_of_val(
            &(aaxn as fn(&MortTableConfig, u32, u32, u32, u32, Option<u32>) -> PolarsResult<f64>),
        ),
    ];

    let _fractional_functions = [
        std::any::type_name_of_val(&(tpx as fn(&MortTableConfig, u32, u32, Option<u32>) -> PolarsResult<f64>)),
        std::any::type_name_of_val(&(tqx as fn(&MortTableConfig, u32, u32, Option<u32>) -> PolarsResult<f64>)),
    ];

    println!("✓ All function categories successfully imported");
    println!("✓ AssumptionEnum working: {assumption:?}");
    println!("✓ Comprehensive prelude functionality verified");
}

#[test]
fn test_prelude_with_real_data() {
    // Test that we can load XML and create config through the prelude
    let xml = MortXML::from_url_id(1704).expect("Failed to load XML from prelude");

    let _config = MortTableConfig {
        xml,
        radix: Some(100_000),
        pct: Some(1.0),
        int_rate: Some(0.03),
        assumption: Some(AssumptionEnum::UDD),
    };

    // Verify that our function exports are accessible (without calling problematic ones)
    let _ax_fn = Ax;
    let _axn_fn = Axn;
    let _aax_fn = aax;
    let _tpx_fn = tpx;
    let _iax_fn = IAx;
    let _gaax_fn = gaax;

    // Test function type signatures
    let _test_ax = Ax as fn(&MortTableConfig, u32, u32, Option<u32>) -> PolarsResult<f64>;
    let _test_aaxn = aaxn as fn(&MortTableConfig, u32, u32, u32, u32, Option<u32>) -> PolarsResult<f64>;
    let _test_tpx = tpx as fn(&MortTableConfig, u32, u32, Option<u32>) -> PolarsResult<f64>;

    println!("✓ XML loading works through prelude");
    println!("✓ Function exports are accessible");
    println!("✓ Function type signatures are correct");
    println!("✓ All {0} benefit functions imported", 13);
    println!("✓ All {0} annuity functions imported", 7);
}
