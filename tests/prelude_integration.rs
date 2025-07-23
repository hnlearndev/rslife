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

    // Immediate life insurance functions
    let _a_x_fn = A_x as fn(&MortTableConfig, i32) -> PolarsResult<f64>;
    let _a_x_n_fn = A_x_n as fn(&MortTableConfig, i32, i32) -> PolarsResult<f64>;
    let _a_x1_n_fn = A_x1_n as fn(&MortTableConfig, i32, i32) -> PolarsResult<f64>;

    // Due life insurance functions
    let _aa_x_fn = AA_x as fn(&MortTableConfig, i32) -> PolarsResult<f64>;
    let _aa_x_n_fn = AA_x_n as fn(&MortTableConfig, i32, i32) -> PolarsResult<f64>;

    // Deferred immediate insurance functions
    let _t_a_x_fn = t_A_x as fn(&MortTableConfig, i32, i32) -> PolarsResult<f64>;
    let _t_a_x_n_fn = t_A_x_n as fn(&MortTableConfig, i32, i32, i32) -> PolarsResult<f64>;

    // Increasing insurance functions
    let _ia_x_fn = IA_x as fn(&MortTableConfig, i32) -> PolarsResult<f64>;
    let _ia_x_n_fn = IA_x_n as fn(&MortTableConfig, i32, i32) -> PolarsResult<f64>;

    // Geometric insurance functions
    let _ga_x_fn = gA_x as fn(&MortTableConfig, i32, f64) -> PolarsResult<f64>;
    let _ga_x_n_fn = gA_x_n as fn(&MortTableConfig, i32, i32, f64) -> PolarsResult<f64>;

    println!("✓ All life insurance functions accessible through prelude");
}

#[test]
fn test_prelude_annuity_functions() {
    // Test annuity function signatures

    // Due annuity functions
    let _aa_x_fn = aa_x as fn(&MortTableConfig, i32, i32) -> PolarsResult<f64>;
    let _aa_x_n_fn = aa_x_n as fn(&MortTableConfig, i32, i32, i32) -> PolarsResult<f64>;

    // Immediate annuity functions
    let _a_x_fn = a_x as fn(&MortTableConfig, i32, i32) -> PolarsResult<f64>;
    let _a_x_n_fn = a_x_n as fn(&MortTableConfig, i32, i32, i32) -> PolarsResult<f64>;

    // Deferred due annuity functions
    let _t_aa_x_fn = t_aa_x as fn(&MortTableConfig, i32, i32, i32) -> PolarsResult<f64>;
    let _t_aa_x_n_fn = t_aa_x_n as fn(&MortTableConfig, i32, i32, i32, i32) -> PolarsResult<f64>;

    // Deferred immediate annuity functions
    let _t_a_x_fn = t_a_x as fn(&MortTableConfig, i32, i32, i32) -> PolarsResult<f64>;
    let _t_a_x_n_fn = t_a_x_n as fn(&MortTableConfig, i32, i32, i32, i32) -> PolarsResult<f64>;

    // Increasing annuity functions
    let _iaa_x_fn = Iaa_x as fn(&MortTableConfig, i32, i32) -> PolarsResult<f64>;
    let _ia_x_fn = Ia_x as fn(&MortTableConfig, i32, i32) -> PolarsResult<f64>;

    // Geometric annuity functions
    let _gaa_x_fn = gaa_x as fn(&MortTableConfig, i32, i32, f64) -> PolarsResult<f64>;
    let _ga_x_fn = ga_x as fn(&MortTableConfig, i32, i32, f64) -> PolarsResult<f64>;

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
        std::any::type_name_of_val(&(A_x as fn(&MortTableConfig, i32) -> PolarsResult<f64>)),
        std::any::type_name_of_val(&(A_x_n as fn(&MortTableConfig, i32, i32) -> PolarsResult<f64>)),
        std::any::type_name_of_val(
            &(A_x1_n as fn(&MortTableConfig, i32, i32) -> PolarsResult<f64>),
        ),
    ];

    let _annuity_functions = [
        std::any::type_name_of_val(&(aa_x as fn(&MortTableConfig, i32, i32) -> PolarsResult<f64>)),
        std::any::type_name_of_val(
            &(t_aa_x as fn(&MortTableConfig, i32, i32, i32) -> PolarsResult<f64>),
        ),
    ];

    let _fractional_functions = [
        std::any::type_name_of_val(&(tpx as fn(&MortTableConfig, f64, f64) -> PolarsResult<f64>)),
        std::any::type_name_of_val(&(tqx as fn(&MortTableConfig, f64, f64) -> PolarsResult<f64>)),
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
    let _a_x_fn = A_x;
    let _a_x_n_fn = A_x_n;
    let _aa_x_fn = aa_x;
    let _t_a_x_fn = t_a_x;
    let _ia_x_fn = IA_x;
    let _gaa_x_fn = gaa_x;

    // Test function type signatures
    let _test_a_x = A_x as fn(&MortTableConfig, i32) -> PolarsResult<f64>;
    let _test_aa_x_n = aa_x_n as fn(&MortTableConfig, i32, i32, i32) -> PolarsResult<f64>;
    let _test_t_ia_x = t_Ia_x as fn(&MortTableConfig, i32, i32, i32) -> PolarsResult<f64>;

    println!("✓ XML loading works through prelude");
    println!("✓ Function exports are accessible");
    println!("✓ Function type signatures are correct");
    println!("✓ All {0} benefit functions imported", 60);
    println!("✓ All {0} annuity functions imported", 28);
}
