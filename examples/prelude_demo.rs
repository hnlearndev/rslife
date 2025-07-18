//! # Example using the prelude
//!
//! This example demonstrates how to use the rslife prelude for easy access
//! to all commonly used types and functions.

use rslife::prelude::*;

fn main() {
    println!("RSLife Prelude Demo");
    println!("==================");
    println!();
    println!("This example demonstrates that all types and functions");
    println!("are accessible through the prelude module.");
    println!();
    println!("Run with: cargo test --example prelude_demo");
    println!("To see the actual tests that verify prelude functionality.");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prelude_imports_basic_types() {
        // Test that we can create the basic configuration enum
        let assumption = AssumptionEnum::UDD;

        // Test that we can reference the MortTableConfig struct
        // (we won't instantiate it here since we'd need actual XML data)
        let _config_type_name = std::any::type_name::<MortTableConfig>();

        println!("Successfully imported AssumptionEnum: {:?}", assumption);
        println!("MortTableConfig type available: {}", _config_type_name);
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
    }

    #[test]
    fn test_prelude_function_availability() {
        // Test that actuarial functions are accessible (we can't call them without data)
        // But we can verify they exist by getting their function pointers

        // Life insurance functions
        let _ax_fn = Ax as fn(&MortTableConfig, i32) -> PolarsResult<f64>;
        let _axn_fn = Axn as fn(&MortTableConfig, i32, i32) -> PolarsResult<f64>;
        let _aexn_fn = AExn as fn(&MortTableConfig, i32, i32) -> PolarsResult<f64>;

        // Annuity functions
        let _axn_due_fn = axn_due as fn(&MortTableConfig, i32, i32, i32) -> PolarsResult<f64>;
        let _tax_due_fn = tax_due as fn(&MortTableConfig, i32, i32, i32) -> PolarsResult<f64>;

        // Fractional age functions - corrected signature
        let _tpx_fn = tpx as fn(&MortTableConfig, f64, f64) -> PolarsResult<f64>;
        let _tqx_fn = tqx as fn(&MortTableConfig, f64, f64) -> PolarsResult<f64>;

        println!("Actuarial functions successfully imported from prelude");
    }
}
