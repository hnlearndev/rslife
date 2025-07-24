//! # RSLife Basic Usage Example
//!
//! This example demonstrates the basic usage of the rslife crate
//! for actuarial calculations using the prelude.

use rslife::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("RSLife Basic Usage Example");
    println!("==========================");
    println!();

    // Load mortality data
    println!("Loading mortality table...");
    let xml = MortXML::from_url_id(1705)?;

    // Create mortality table configuration
    let config = MortTableConfig {
        xml,
        radix: Some(100_000),
        pct: Some(1.0),
        int_rate: Some(0.03),
        assumption: Some(AssumptionEnum::UDD),
    };

    println!("✓ Mortality table configured");
    println!();

    // Example actuarial calculations
    println!("Performing actuarial calculations...");

    // Life insurance calculations
    let whole_life = Ax(&config, 30, 0, None)?;
    let term_20 = Ax1n(&config, 30, 20, 0, None)?;
    let endowment_20 = Axn(&config, 30, 20, 0, None)?;

    println!("Life Insurance (age 30):");
    println!("  Whole life (Ax): {whole_life:.6}");
    println!("  20-year term (Ax1n): {term_20:.6}");
    println!("  20-year endowment (Axn): {endowment_20:.6}");
    println!();

    // Annuity calculations
    let annuity_annual_due = aaxn(&config, 65, 20, 1, 0, None)?;
    let annuity_monthly_due = aaxn(&config, 65, 20, 12, 0, None)?;
    let annuity_life_due = aax(&config, 65, 1, 0, None)?;

    println!("Annuities (age 65):");
    println!("  20-year annual due payments: {annuity_annual_due:.6}");
    println!("  20-year monthly due payments: {annuity_monthly_due:.6}");
    println!("  Life annual due payments: {annuity_life_due:.6}");
    println!();

    // Survival probability calculations
    let survival_10_years = tpx(&config, 30.0, 10.0, None)?;
    let mortality_10_years = tqx(&config, 30.0, 10.0, None)?;

    println!("Survival calculations (10 years, age 30):");
    println!("  Survival probability (tpx): {survival_10_years:.6}");
    println!("  Mortality probability (tqx): {mortality_10_years:.6}");
    println!(
        "  Sum (should be 1.0): {:.6}",
        survival_10_years + mortality_10_years
    );
    println!();

    println!("✓ All calculations completed successfully!");

    Ok(())
}
