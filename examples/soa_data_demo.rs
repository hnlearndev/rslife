//! # RSLife SOA Data Demo
//!
//! This example demonstrates loading mortality tables directly from the
//! Society of Actuaries (SOA) website and performing actuarial calculations.
//! Shows both MortTableConfig construction methods and basic table exploration.

use rslife::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🦀 RSLife SOA Data Demo");
    println!("========================");

    // Example 1: Load 2017 CSO mortality table
    println!("\n📡 Loading 2017 CSO Mortality Table from SOA...");
    demo_2017_cso()?;

    // Example 2: Load a different table to show variety
    println!("\n📡 Loading 1980 CSO Basic Table from SOA...");
    demo_1980_cso()?;

    println!("\n🎉 All SOA data demos completed successfully!");

    Ok(())
}

/// Demonstrate loading and using 2017 CSO mortality table
fn demo_2017_cso() -> Result<(), Box<dyn std::error::Error>> {
    let xml = MortXML::from_url_id(1704)?;
    let table = &xml.tables[0];

    // Display basic information
    println!("✅ Successfully loaded 2017 CSO mortality table!");
    println!("📊 Table: {}", xml.content_classification.table_name);
    println!("📏 Rows: {}", table.values.height());
    println!(
        "🏷️  Table ID: {}",
        xml.content_classification.table_identity
    );

    // Show table structure
    let columns = table.values.get_column_names();
    println!("📋 Columns: {columns:?}");

    // Display metadata
    println!("\n📝 Metadata:");
    println!("   Scaling Factor: {}", table.meta_data.scaling_factor);
    println!("   Data Type: {}", table.meta_data.data_type);
    println!("   Nation: {}", table.meta_data.nation);

    // Show first few rows
    println!("\n📈 First 5 rows:");
    if table.values.height() > 0 {
        let sample = table.values.head(Some(5));
        println!("{sample}");
    }

    // Show last few rows to see age range
    println!("\n📈 Last 5 rows (age range):");
    if table.values.height() > 5 {
        let sample = table.values.tail(Some(5));
        println!("{sample}");
    }

    // Perform actuarial calculations using struct literal
    println!("\n🧮 Actuarial calculations with 2017 CSO (struct literal):");
    let mt_config = MortTableConfig {
        xml,
        radix: Some(100_000),
        pct: Some(1.0),
        assumption: Some(AssumptionEnum::UDD),
    };

    let params = ParamConfig {
        mt: mt_config,
        i: 0.03,
        x: 40,
        n: Some(25),
        t: None,
        m: Some(1),
        moment: Some(1),
        entry_age: None,
    };

    let whole_life_40 = Ax(&params)?;
    let term_25_40 = Ax1n(&params)?;
    let endowment_25_40 = Axn(&params)?;
    let survival_25p40 = tpx(&params.mt, 40.0, 25.0, 0.0, None)?;

    println!("   Life Insurance (age 40):");
    println!("     Whole life: {:.6}", whole_life_40);
    println!("     25-year term: {:.6}", term_25_40);
    println!("     25-year endowment: {:.6}", endowment_25_40);
    println!("   Survival:");
    println!("     25-year survival: {:.2}%", survival_25p40 * 100.0);

    Ok(())
}

/// Demonstrate loading and using 1980 CSO mortality table
fn demo_1980_cso() -> Result<(), Box<dyn std::error::Error>> {
    let xml = MortXML::from_url_id(912)?;
    let table = &xml.tables[0];

    println!("✅ Successfully loaded 1980 CSO Basic Table!");
    println!("📊 Table: {}", xml.content_classification.table_name);
    println!("📏 Rows: {}", table.values.height());
    println!(
        "🏷️  Table ID: {}",
        xml.content_classification.table_identity
    );

    // Show sample data
    println!("\n📈 Sample data (rows 20-25):");
    if table.values.height() > 25 {
        let sample = table.values.slice(20, 5);
        println!("{sample}");
    }

    // Perform actuarial calculations using struct literals
    println!("\n🧮 Actuarial calculations with 1980 CSO (struct literals):");
    let mt_config = MortTableConfig {
        xml,
        radix: Some(100_000),
        pct: Some(1.0),
        assumption: Some(AssumptionEnum::CFM),
    };

    let params_35 = ParamConfig {
        mt: mt_config.clone(),
        i: 0.05,
        x: 35,
        n: Some(30),
        t: None,
        m: Some(1),
        moment: Some(1),
        entry_age: None,
    };

    let params_60 = ParamConfig {
        mt: mt_config,
        i: 0.05,
        x: 60,
        n: Some(15),
        t: None,
        m: Some(12), // Monthly payments
        moment: Some(1),
        entry_age: None,
    };

    // Life insurance calculations
    let whole_life_35 = Ax(&params_35)?;
    let term_30_35 = Ax1n(&params_35)?;

    // Annuity calculations
    let annuity_15_60 = aaxn(&params_60)?;
    let life_annuity_60 = aax(&params_60)?;

    // Survival calculations
    let survival_30p35 = tpx(&params_35.mt, 35.0, 30.0, 0.0, None)?;
    let survival_15p60 = tpx(&params_60.mt, 60.0, 15.0, 0.0, None)?;

    println!("   Life Insurance Values:");
    println!("     Whole life (age 35): {:.6}", whole_life_35);
    println!("     30-year term (age 35): {:.6}", term_30_35);

    println!("\n   Annuity Values:");
    println!(
        "     15-year monthly annuity (age 60): {:.6}",
        annuity_15_60
    );
    println!("     Life monthly annuity (age 60): {:.6}", life_annuity_60);

    println!("\n   Survival Probabilities:");
    println!(
        "     30-year survival from age 35: {:.2}%",
        survival_30p35 * 100.0
    );
    println!(
        "     15-year survival from age 60: {:.2}%",
        survival_15p60 * 100.0
    );

    Ok(())
}
