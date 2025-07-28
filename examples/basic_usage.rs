//! # RSLife Basic Usage Example
//!
//! This example demonstrates the basic usage of the rslife crate
//! for actuarial calculations using the new builder pattern API.

use rslife::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("RSLife Basic Usage Example");
    println!("==========================");
    println!();

    // Load mortality data
    println!("Loading mortality table...");
    let mort_data = MortData::from_soa_url_id(1704)?;

    // Method 1: Create mortality table configuration using struct literal
    println!("Creating MortTableConfig using struct literal...");
    let mt_config = MortTableConfig {
        data: mort_data.clone(),
        radix: Some(100_000),
        pct: Some(1.0),
        assumption: Some(AssumptionEnum::UDD),
    };

    // Method 2: Create another config with different settings using struct literal
    println!("Creating second MortTableConfig with different settings...");
    let mt_config_cfm = MortTableConfig {
        data: mort_data.clone(),
        radix: Some(100_000),
        pct: Some(0.75),                       // Reduced mortality
        assumption: Some(AssumptionEnum::CFM), // Different assumption
    };

    println!("✓ Mortality table configured with both UDD and CFM assumptions");
    println!();

    // Life insurance calculations using builder pattern
    println!("\n=== Life Insurance Calculations (UDD assumption, age 30) ===");
    let whole_life = Ax()
        .mt(&mt_config)
        .i(0.03)
        .x(30)
        .call()?;
    let term_20 = Ax1n()
        .mt(&mt_config)
        .i(0.03)
        .x(30)
        .n(20)
        .call()?;
    let endowment_20 = Axn()
        .mt(&mt_config)
        .i(0.03)
        .x(30)
        .n(20)
        .call()?;

    println!("Life Insurance:");
    println!("  Whole life (Ax): {whole_life:.6}");
    println!("  20-year term (Ax1n): {term_20:.6}");
    println!("  20-year endowment (Axn): {endowment_20:.6}");

    // Annuity calculations using CFM assumption
    println!("\n=== Annuity Calculations (CFM assumption, age 65) ===");
    let annuity_due = aaxn()
        .mt(&mt_config_cfm)
        .i(0.04)
        .x(65)
        .n(20)
        .call()?;
    let life_annuity = aax()
        .mt(&mt_config_cfm)
        .i(0.04)
        .x(65)
        .call()?;

    println!("Annuities:");
    println!("  20-year annuity due: {annuity_due:.6}");
    println!("  Life annuity due: {life_annuity:.6}");

    // Survival probability calculations
    println!("\n=== Survival Calculations (direct MortTableConfig calls) ===");
    let survival_10_years = tpx()
        .mt(&mt_config)
        .x(30.0)
        .t(10.0)
        .k(0.0)
        .call()?;
    let mortality_10_years = tqx()
        .mt(&mt_config)
        .x(30.0)
        .t(10.0)
        .k(0.0)
        .call()?;

    println!("Survival calculations (10 years, age 30):");
    println!("  Survival probability (tpx): {survival_10_years:.6}");
    println!("  Mortality probability (tqx): {mortality_10_years:.6}");
    println!(
        "  Sum (should be 1.0): {:.6}",
        survival_10_years + mortality_10_years
    );

    // Demonstrate fractional ages and times
    println!("\n=== Fractional Age and Time Calculations ===");
    let fractional_survival = tpx()
        .mt(&mt_config)
        .x(45.5)     // Fractional age
        .t(2.75)     // Fractional time
        .call()?;

    println!("Fractional calculations:");
    println!("  2.75-year survival from age 45.5: {fractional_survival:.6}");

    println!("\n✓ All calculations completed successfully!");
    println!("✓ Demonstrated both UDD and CFM mortality assumptions!");
    println!("✓ Showcased flexible builder pattern with optional parameters!");

    Ok(())
}
