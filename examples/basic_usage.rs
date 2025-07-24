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
    let xml = MortXML::from_url_id(1704)?;

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
    let whole_life = A_x(&config, 30)?;
    let term_20 = A_x1_n(&config, 30, 20)?;
    let endowment_20 = A_x_n(&config, 30, 20)?;

    println!("Life Insurance (age 30):");
    println!("  Whole life (A_x): {whole_life:.6}");
    println!("  20-year term (A_x1_n): {term_20:.6}");
    println!("  20-year endowment (A_x_n): {endowment_20:.6}");
    println!();

    // Annuity calculations
    let annuity_annual_due = aa_x_n(&config, 65, 20, 1)?;
    let annuity_monthly_due = aa_x_n(&config, 65, 20, 12)?;
    let annuity_annual_immediate = a_x_n(&config, 65, 20, 1)?;

    println!("Annuities (age 65, 20 years):");
    println!("  Annual due payments: {annuity_annual_due:.6}");
    println!("  Monthly due payments: {annuity_monthly_due:.6}");
    println!("  Annual immediate payments: {annuity_annual_immediate:.6}");
    println!();

    // Fractional age calculations
    let survival_half_year = tpx(&config, 0.5, 30.0)?;
    let mortality_half_year = tqx(&config, 0.5, 30.0)?;

    println!("Fractional age calculations (6 months, age 30):");
    println!("  Survival probability (tpx): {survival_half_year:.6}");
    println!("  Mortality probability (tqx): {mortality_half_year:.6}");
    println!(
        "  Sum (should be 1.0): {:.6}",
        survival_half_year + mortality_half_year
    );
    println!();

    println!("✓ All calculations completed successfully!");

    Ok(())
}
