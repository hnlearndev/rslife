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

    println!("Successfully imported AssumptionEnum: {:?}", assumption);
    println!("MortTableConfig type available: {}", _config_type_name);

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

    println!("DataFrame type available: {}", _df_type_name);
    println!("Series type available: {}", _series_type_name);
    println!("PolarsResult type available: {}", _result_type_name);

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

    println!("MortXML type available: {}", _xml_type_name);
    println!(
        "ContentClassification type available: {}",
        _content_class_type_name
    );

    // Verify type names are correct
    assert!(_xml_type_name.contains("MortXML"));
    assert!(_content_class_type_name.contains("ContentClassification"));
}

#[test]
fn test_prelude_function_signatures() {
    // Test that actuarial functions are accessible and have correct signatures
    // We verify they exist by getting their function pointers

    // Life insurance functions
    let _ax_fn = Ax as fn(&MortTableConfig, f64) -> PolarsResult<f64>;
    let _axn_fn = Axn as fn(&MortTableConfig, f64, i32) -> PolarsResult<f64>;
    let _aexn_fn = AExn as fn(&MortTableConfig, f64, i32) -> PolarsResult<f64>;

    // Deferred insurance functions
    let _tax_fn = tAx as fn(&MortTableConfig, f64, i32) -> PolarsResult<f64>;
    let _taxn_fn = tAxn as fn(&MortTableConfig, f64, i32, i32) -> PolarsResult<f64>;

    // Increasing insurance functions
    let _iax_fn = IAx as fn(&MortTableConfig, f64) -> PolarsResult<f64>;
    let _iaxn_fn = IAxn as fn(&MortTableConfig, f64, i32) -> PolarsResult<f64>;

    // Geometric insurance functions
    let _gax_fn = gAx as fn(&MortTableConfig, f64, f64) -> PolarsResult<f64>;
    let _gaxn_fn = gAxn as fn(&MortTableConfig, f64, i32, f64) -> PolarsResult<f64>;

    // Pure endowment functions
    let _exn_fn = Exn as fn(&MortTableConfig, f64, i32) -> PolarsResult<f64>;

    println!("✓ All life insurance functions accessible through prelude");
}

#[test]
fn test_prelude_annuity_functions() {
    // Test annuity function signatures

    // Basic annuity functions
    let _aaxn_fn = aaxn as fn(&MortTableConfig, f64, i32, i32) -> PolarsResult<f64>;

    // Deferred annuity functions
    let _taax_fn = taax as fn(&MortTableConfig, f64, i32, i32) -> PolarsResult<f64>;
    let _taaxn_fn = taaxn as fn(&MortTableConfig, f64, i32, i32, i32) -> PolarsResult<f64>;

    // Increasing annuity functions
    let _iaax_fn = Iaax as fn(&MortTableConfig, f64, i32, i32) -> PolarsResult<f64>;
    let _iaaxn_fn = Iaaxn as fn(&MortTableConfig, f64, i32, i32) -> PolarsResult<f64>;

    // Deferred increasing annuity functions
    let _tiaax_fn = tIaax as fn(&MortTableConfig, f64, i32, i32, i32) -> PolarsResult<f64>;
    let _tiaaxn_fn = tIaaxn as fn(&MortTableConfig, f64, i32, i32, i32) -> PolarsResult<f64>;

    // Geometric annuity functions
    let _giaax_fn = gIaax as fn(&MortTableConfig, f64, i32, i32, f64) -> PolarsResult<f64>;
    let _giaaxn_fn = gIaaxn as fn(&MortTableConfig, f64, i32, i32, f64) -> PolarsResult<f64>;

    println!("✓ All annuity functions accessible through prelude");
}

#[test]
fn test_prelude_fractional_age_functions() {
    // Test fractional age function signatures
    let _tpx_fn = tpx as fn(&MortTableConfig, f64, f64) -> PolarsResult<f64>;
    let _tqx_fn = tqx as fn(&MortTableConfig, f64, f64) -> PolarsResult<f64>;

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
        std::any::type_name_of_val(&(Ax as fn(&MortTableConfig, f64) -> PolarsResult<f64>)),
        std::any::type_name_of_val(&(Axn as fn(&MortTableConfig, f64, i32) -> PolarsResult<f64>)),
        std::any::type_name_of_val(&(AExn as fn(&MortTableConfig, f64, i32) -> PolarsResult<f64>)),
    ];

    let _annuity_functions = [
        std::any::type_name_of_val(
            &(aaxn as fn(&MortTableConfig, f64, i32, i32) -> PolarsResult<f64>),
        ),
        std::any::type_name_of_val(
            &(taax as fn(&MortTableConfig, f64, i32, i32) -> PolarsResult<f64>),
        ),
    ];

    let _fractional_functions = [
        std::any::type_name_of_val(&(tpx as fn(&MortTableConfig, f64, f64) -> PolarsResult<f64>)),
        std::any::type_name_of_val(&(tqx as fn(&MortTableConfig, f64, f64) -> PolarsResult<f64>)),
    ];

    println!("✓ All function categories successfully imported");
    println!("✓ AssumptionEnum working: {:?}", assumption);
    println!("✓ Comprehensive prelude functionality verified");
}

#[test]
fn test_prelude_with_real_data() {
    // Test that we can actually use prelude imports with real mortality data

    // Load a mortality table using the prelude imports
    let xml = MortXML::from_url_id(912).expect("Failed to load XML from prelude");

    let config = MortTableConfig {
        xml,
        l_x_init: 100_000,
        pct: Some(1.0),
        int_rate: Some(0.03),
        assumption: Some(AssumptionEnum::UDD),
    };

    // Test that we can call functions through the prelude
    let ax_result = Ax(&config, 30.0);
    assert!(ax_result.is_ok(), "Ax function should work through prelude");

    let axn_result = Axn(&config, 30.0, 20);
    assert!(
        axn_result.is_ok(),
        "Axn function should work through prelude"
    );

    let tpx_result = tpx(&config, 0.5, 30.0);
    assert!(
        tpx_result.is_ok(),
        "tpx function should work through prelude"
    );

    let tqx_result = tqx(&config, 0.5, 30.0);
    assert!(
        tqx_result.is_ok(),
        "tqx function should work through prelude"
    );

    // Verify the mathematical relationship: tpx + tqx = 1
    let survival = tpx_result.unwrap();
    let mortality = tqx_result.unwrap();
    assert!(
        (survival + mortality - 1.0).abs() < 1e-10,
        "tpx + tqx should equal 1.0, got: {} + {} = {}",
        survival,
        mortality,
        survival + mortality
    );

    println!("✓ Real calculations work through prelude");
    println!("✓ tpx(0.5, 30.0) = {:.6}", survival);
    println!("✓ tqx(0.5, 30.0) = {:.6}", mortality);
}
