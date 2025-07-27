//! # RSLife Basic Usage Example
//!
//! This example demonstrates the basic usage of the rslife crate
//! for actuarial calculations using both struct construction methods.

use rslife::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("RSLife Basic Usage Example");
    println!("==========================");
    println!();

    // Load mortality data
    println!("Loading mortality table...");
    let xml = MortXML::from_url_id(1704)?;

    // Method 1: Create mortality table configuration using struct literal
    println!("Creating MortTableConfig using struct literal...");
    let mt_config = MortTableConfig {
        xml: xml.clone(),
        radix: Some(100_000),
        pct: Some(1.0),
        assumption: Some(AssumptionEnum::UDD),
    };

    // Method 2: Create another config with different settings using struct literal
    println!("Creating second MortTableConfig with different settings...");
    let mt_config_cfm = MortTableConfig {
        xml: xml.clone(),
        radix: Some(100_000),
        pct: Some(0.75),                       // Reduced mortality
        assumption: Some(AssumptionEnum::CFM), // Different assumption
    };

    println!("✓ Mortality table configured with both UDD and CFM assumptions");
    println!();

    // Create ParamConfig using both methods for actuarial calculations
    println!("Performing actuarial calculations with ParamConfig...");

    // Method 1: Struct literal for ParamConfig
    let params_struct = ParamConfig {
        mt: mt_config,
        i: 0.03,
        x: 30,
        n: Some(20),
        t: None,
        m: Some(1),
        moment: Some(1),
        entry_age: None,
    };

    // Method 2: Create ParamConfig with CFM assumption
    let params_cfm = ParamConfig {
        mt: mt_config_cfm,
        i: 0.04, // Different interest rate
        x: 65,
        n: Some(20),
        t: None,
        m: Some(1),
        moment: Some(1),
        entry_age: None,
    };

    // Life insurance calculations using struct literal params
    println!("\n=== Calculations with Struct Literal ParamConfig (age 30) ===");
    let whole_life = Ax(&params_struct)?;
    let term_20 = Ax1n(&params_struct)?;
    let endowment_20 = Axn(&params_struct)?;

    println!("Life Insurance:");
    println!("  Whole life (Ax): {whole_life:.6}");
    println!("  20-year term (Ax1n): {term_20:.6}");
    println!("  20-year endowment (Axn): {endowment_20:.6}");

    // Annuity calculations using CFM assumption params
    println!("\n=== Calculations with CFM Assumption ParamConfig (age 65) ===");
    let annuity_due = aaxn(&params_cfm)?;
    let life_annuity = aax(&params_cfm)?;

    println!("Annuities:");
    println!("  20-year annuity due: {annuity_due:.6}");
    println!("  Life annuity due: {life_annuity:.6}");

    // Survival probability calculations using direct MortTableConfig calls
    println!("\n=== Survival Calculations (direct MortTableConfig calls) ===");
    let survival_10_years = tpx(&params_struct.mt, 30.0, 10.0, 0.0, None)?;
    let mortality_10_years = tqx(&params_struct.mt, 30.0, 10.0, 0.0, None)?;

    println!("Survival calculations (10 years, age 30):");
    println!("  Survival probability (tpx): {survival_10_years:.6}");
    println!("  Mortality probability (tqx): {mortality_10_years:.6}");
    println!(
        "  Sum (should be 1.0): {:.6}",
        survival_10_years + mortality_10_years
    );

    println!("\n✓ All calculations completed successfully!");
    println!("✓ Demonstrated both UDD and CFM mortality assumptions!");

    Ok(())
}
