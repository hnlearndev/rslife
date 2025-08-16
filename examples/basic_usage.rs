//! # RSLife Basic Usage Example
//!
//! This example demonstrates the basic usage of the rslife crate
//! for actuarial calculations using the new builder pattern API.

use rslife::prelude::*;

fn main() -> RSLifeResult<()> {
    println!("RSLife Basic Usage Example");
    println!("==========================");
    println!();

    // Load mortality data
    println!("Loading mortality table...");
    let mort_data = MortData::from_soa_url_id(1704)?;

    // Method 1: Create mortality table configuration using builder pattern
    println!("Creating MortTableConfig using builder pattern...");
    let mt_config = MortTableConfig::builder()
        .data(mort_data.clone())
        .radix(10_000) // Default radix - Might even not needed to be declared
        .pct(1.0) // Default mortality percentage - Might even not needed to be declared
        .assumption(AssumptionEnum::UDD) // Default assumption - Might even not needed to be declared
        .build()
        .unwrap();

    // Method 2: Create another config with different settings using builder pattern
    println!("Creating second MortTableConfig with different settings (CFM)...");
    let mt_config_cfm = MortTableConfig::builder()
        .data(mort_data.clone())
        .radix(100_000)
        .pct(0.75) // Reduced mortality
        .assumption(AssumptionEnum::CFM) // Different assumption
        .build()
        .unwrap();

    println!("✓ Mortality table configured with both UDD and CFM assumptions");
    println!();

    // Life insurance calculations using builder pattern
    println!("\n=== Life Insurance Calculations (UDD assumption, age 30) ===");
    let term_10 = Ax1n().mt(&mt_config).i(0.03).x(30).n(10).call()?;
    let term_20 = Ax1n().mt(&mt_config).i(0.03).x(30).n(20).call()?;
    let endowment_15 = Axn().mt(&mt_config).i(0.03).x(30).n(15).call()?;

    println!("Life Insurance:");
    println!("  10-year term (Ax1n): {term_10:.6}");
    println!("  20-year term (Ax1n): {term_20:.6}");
    println!("  15-year endowment (Axn): {endowment_15:.6}");

    // Annuity calculations using CFM assumption
    println!("\n=== Annuity Calculations (CFM assumption, age 40) ===");
    let annuity_due = aaxn().mt(&mt_config_cfm).i(0.04).x(40).n(15).call()?;
    let life_annuity = aax().mt(&mt_config_cfm).i(0.04).x(40).call()?;

    println!("Annuities:");
    println!("  15-year annuity due: {annuity_due:.6}");
    println!("  Life annuity due: {life_annuity:.6}");

    // Survival probability calculations
    println!("\n=== Survival Calculations (direct MortTableConfig calls) ===");
    let survival_10_years = tpx().mt(&mt_config).x(30.0).t(10.0).k(0.0).call()?;
    let mortality_10_years = tqx().mt(&mt_config).x(30.0).t(10.0).k(0.0).call()?;

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
        .x(45.5) // Fractional age
        .t(2.75) // Fractional time
        .call()?;

    println!("Fractional calculations:");
    println!("  2.75-year survival from age 45.5: {fractional_survival:.6}");

    println!("\n✓ All calculations completed successfully!");
    println!("✓ Demonstrated both UDD and CFM mortality assumptions!");
    println!("✓ Showcased flexible builder pattern with optional parameters!");

    Ok(())
}
