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

    // Test that we can reference the MortTableConfig and ParamConfig structs
    let _config_type_name = std::any::type_name::<MortTableConfig>();
    let _param_type_name = std::any::type_name::<ParamConfig>();

    println!("Successfully imported AssumptionEnum: {assumption:?}");
    println!("MortTableConfig type available: {_config_type_name}");
    println!("ParamConfig type available: {_param_type_name}");

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
fn test_prelude_function_accessibility() {
    // Test that actuarial functions are accessible through prelude
    // We test function accessibility without complex type casting

    // Verify that our function exports are accessible
    let _ax_fn = Ax;
    let _axn_fn = Axn;
    let _ax1n_fn = Ax1n;
    let _Exn_fn = Exn;
    let _iax_fn = IAx;
    let _aax_fn = aax;
    let _aaxn_fn = aaxn;
    let _tpx_fn = tpx;
    let _tqx_fn = tqx;
    let _gaax_fn = gaax;

    println!("✓ All actuarial functions accessible through prelude");
}

#[test]
fn test_prelude_with_real_data() {
    // Test that we can load XML and create config through the prelude
    let xml = MortXML::from_url_id(1704).expect("Failed to load XML from prelude");

    let mt_config = MortTableConfig {
        xml,
        radix: Some(100_000),
        pct: Some(1.0),
        assumption: Some(AssumptionEnum::UDD),
    };

    let _params = ParamConfig {
        mt: mt_config,
        i: 0.03,
        x: 35,
        n: Some(10),
        t: None,
        m: Some(1),
        moment: Some(1),
        entry_age: None,
    };

    println!("✓ XML loading works through prelude");
    println!("✓ MortTableConfig creation works through prelude");
    println!("✓ ParamConfig creation works through prelude");
    println!("✓ All types and functions successfully imported through prelude");
}
