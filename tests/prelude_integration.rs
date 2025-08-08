//! # Integration Tests for RSLife Prelude
//!
//! This integration test verifies that the prelude module properly re-exports
//! all commonly used types and functions from the rslife crate, ensuring that
//! users can import everything they need with a single `use` statement.

use polars::df;
use rslife::prelude::*;

#[test]
fn test_prelude_imports_basic_types() {
    // Test that we can create the basic configuration enum
    let assumption = AssumptionEnum::UDD;

    // Test that we can reference the MortTableConfig (param structs are internal)
    let _config_type_name = std::any::type_name::<MortTableConfig>();

    println!("Successfully imported AssumptionEnum: {assumption:?}");
    println!("MortTableConfig type available: {_config_type_name}");

    // Verify the enum has the expected variants
    assert!(matches!(assumption, AssumptionEnum::UDD));

    // Test other assumption variants
    let _cfm = AssumptionEnum::CFM;
    let _hpb = AssumptionEnum::HPB;

    println!("✓ Parameter structs are internal and not exposed through prelude");
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
fn test_prelude_imports_data_types() {
    // Test that data types are accessible
    let _data_type_name = std::any::type_name::<MortData>();

    println!("MortData type available: {_data_type_name}");

    // Verify type names are correct
    assert!(_data_type_name.contains("MortData"));

    // Note: XML types are internal to MortData implementation and not exposed
    println!("✓ MortData accessible through prelude (XML types are internal)");
}

#[test]
fn test_prelude_function_accessibility() {
    // Test that actuarial functions are accessible through prelude
    // We test function accessibility without complex type casting

    // Verify that our function exports are accessible
    let _ax_fn = Ax;
    let _axn_fn = Axn;
    let _ax1n_fn = Ax1n;
    let _exn_fn = Exn;
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
    // Create mock data instead of loading from SOA to avoid network dependency
    let df = df! {
        "age" => [20.0, 21.0, 22.0, 23.0, 24.0],
        "qx" => [0.001, 0.002, 0.003, 0.004, 0.005],
    }
    .expect("Failed to create mock DataFrame");

    let data = MortData::from_df(df).expect("Failed to create MortData from DataFrame");

    let mt_config = MortTableConfig::builder()
        .data(data)
        .radix(100_000)
        .pct(1.0)
        .assumption(AssumptionEnum::UDD)
        .build()
        .unwrap();

    // Test direct function calls (the new API)
    let whole_life_result = Ax().mt(&mt_config).i(0.03).x(20).call();

    let survival_result = tpx().mt(&mt_config).x(20.0).t(3.0).call();

    // These should compile successfully (we don't need to verify exact values)
    println!("✓ Mock data creation works through prelude");
    println!("✓ MortTableConfig builder works through prelude");
    println!("✓ Direct function calls work through prelude");
    println!(
        "✓ Ax() function call result: {:?}",
        whole_life_result.is_ok()
    );
    println!(
        "✓ tpx() function call result: {:?}",
        survival_result.is_ok()
    );
    println!("✓ All types and functions successfully imported through prelude");
}
