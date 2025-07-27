//! # RSLife Custom Data Demo
//!
//! This example demonstrates loading mortality tables from custom data sources
//! including XLSX files and programmatically created DataFrames.
//! Shows both struct literal and builder pattern usage.

use polars::df;
use rslife::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🦀 RSLife Custom XLSX Demo with Real Data");
    println!("===========================================");

    // Example 1: Load ELT15 Female Mortality Table
    println!("\n📂 Loading ELT15 Female Mortality Table...");
    demo_elt15_female()?;

    // Example 2: Load AM92 Select Mortality Table
    println!("\n📂 Loading AM92 Select Mortality Table...");
    demo_am92_select()?;

    // Example 3: Load LTAM Standard Ultimate Table
    println!("\n📂 Loading LTAM Standard Ultimate Table...");
    demo_ltam_ultimate()?;

    println!("\n🎉 All XLSX demos completed successfully!");

    Ok(())
}

/// Demonstrate loading and using ELT15 female mortality data
fn demo_elt15_female() -> Result<(), Box<dyn std::error::Error>> {
    let xlsx_result = MortXML::from_xlsx("data/elt15.xlsx", "female");

    match xlsx_result {
        Ok(xml) => {
            let table = &xml.tables[0];

            println!("✅ Successfully loaded ELT15 Female table!");
            println!("📊 Table: {}", xml.content_classification.table_name);
            println!("📏 Rows: {}", table.values.height());

            // Show table structure
            let columns = table.values.get_column_names();
            println!("📋 Columns: {:?}", columns);

            // Display metadata
            println!("\n📝 Metadata:");
            println!("   Data Type: {}", table.meta_data.data_type);
            println!("   Nation: {}", table.meta_data.nation);
            println!("   Scaling Factor: {}", table.meta_data.scaling_factor);

            // Show first few rows
            println!("\n📈 First 5 rows of ELT15 Female data:");
            if table.values.height() > 0 {
                let sample = table.values.head(Some(5));
                println!("{}", sample);
            }

            // Show last few rows to see the age range
            println!("\n📈 Last 5 rows to show age range:");
            if table.values.height() > 5 {
                let sample = table.values.tail(Some(5));
                println!("{}", sample);
            }

            // Perform actuarial calculations with ELT15 data using struct literals
            println!("\n🧮 Actuarial calculations with ELT15 Female data (using struct literals):");
            let mt_config = MortTableConfig {
                xml,
                radix: Some(100_000),
                pct: Some(1.0),
                assumption: Some(AssumptionEnum::UDD),
            };

            let params = ParamConfig {
                mt: mt_config,
                i: 0.03,
                x: 35,
                n: Some(20),
                t: None,
                m: Some(1),
                moment: Some(1),
                entry_age: None,
            };

            // Calculate various actuarial values using new API
            let whole_life_35 = Ax(&params)?;
            let term_life_35_20 = Ax1n(&params)?;
            let pure_endowment_35_30 = Exn(&params)?;
            let annuity_due_35_20 = aaxn(&params)?;
            let survival_10p35 = tpx(&params.mt, 35.0, 10.0, 0.0, None)?;
            let survival_30p35 = tpx(&params.mt, 35.0, 30.0, 0.0, None)?;

            println!("   Life Insurance Values:");
            println!("     Whole life (age 35): {:.6}", whole_life_35);
            println!("     20-year term (age 35): {:.6}", term_life_35_20);
            println!(
                "     30-year pure endowment (age 35): {:.6}",
                pure_endowment_35_30
            );
            println!("\n   Annuity Values:");
            println!(
                "     20-year annuity due (age 35): {:.6}",
                annuity_due_35_20
            );
            println!("\n   Survival Probabilities:");
            println!("     10-year survival from age 35: {:.6}", survival_10p35);
            println!("     30-year survival from age 35: {:.6}", survival_30p35);
        }
        Err(e) => {
            println!("⚠️  Could not load ELT15 file: {}", e);
            println!("   This is likely a data type validation issue in the XLSX parser.");
            println!("   📋 Let's demonstrate the expected format and show a workaround:");

            // Workaround: create sample data manually to show what should work
            demonstrate_elt15_format()?;
        }
    }

    Ok(())
}

/// Demonstrate loading and using AM92 select mortality data
fn demo_am92_select() -> Result<(), Box<dyn std::error::Error>> {
    let xlsx_result = MortXML::from_xlsx("data/am92.xlsx", "am92");

    match xlsx_result {
        Ok(xml) => {
            let table = &xml.tables[0];

            println!("✅ Successfully loaded AM92 Select table!");
            println!("📊 Table: {}", xml.content_classification.table_name);
            println!("📏 Rows: {}", table.values.height());

            let columns = table.values.get_column_names();
            println!("📋 Columns: {:?}", columns);

            // Check if this has duration column (select table)
            if columns.iter().any(|col| col.as_str() == "duration") {
                println!("📊 This is a SELECT mortality table with duration periods");
            } else {
                println!("📊 This appears to be an ULTIMATE mortality table");
            }

            // Show sample data
            println!("\n📈 First 10 rows of AM92 Select data:");
            if table.values.height() > 0 {
                let sample = table.values.head(Some(10));
                println!("{}", sample);
            }

            // Perform calculations if it's a proper mortality table using struct literal
            if columns.iter().any(|col| col.as_str() == "qx")
                || columns.iter().any(|col| col.as_str() == "lx")
            {
                let mt_config = MortTableConfig {
                    xml,
                    radix: Some(100_000),
                    pct: Some(1.0),
                    assumption: Some(AssumptionEnum::UDD),
                };

                let params = ParamConfig {
                    mt: mt_config,
                    i: 0.04,
                    x: 40,
                    n: None,
                    t: None,
                    m: Some(1),
                    moment: Some(1),
                    entry_age: Some(40), // Demo select table usage
                };

                println!("\n🧮 Sample calculations with AM92 data (struct literal):");
                let whole_life_40 = Ax(&params)?;
                let survival_20p40 = tpx(&params.mt, 40.0, 20.0, 0.0, params.entry_age)?;

                println!("   Whole life insurance (age 40): {:.6}", whole_life_40);
                println!("   20-year survival from age 40: {:.6}", survival_20p40);
            }
        }
        Err(e) => {
            println!("⚠️  Could not load AM92 Select file: {}", e);
            println!("   This is likely a data type validation issue in the XLSX parser.");
            demonstrate_am92_format()?;
        }
    }

    Ok(())
}

/// Demonstrate loading and using LTAM standard ultimate table
fn demo_ltam_ultimate() -> Result<(), Box<dyn std::error::Error>> {
    let xlsx_result = MortXML::from_xlsx("data/ltam_standard_ultimate.xlsx", "ltam");

    match xlsx_result {
        Ok(xml) => {
            let table = &xml.tables[0];

            println!("✅ Successfully loaded LTAM Standard Ultimate table!");
            println!("📊 Table: {}", xml.content_classification.table_name);
            println!("📏 Rows: {}", table.values.height());

            let columns = table.values.get_column_names();
            println!("📋 Columns: {:?}", columns);

            // Show sample data
            println!("\n📈 First 8 rows of LTAM Ultimate data:");
            if table.values.height() > 0 {
                let sample = table.values.head(Some(8));
                println!("{}", sample);
            }

            // Perform comprehensive calculations using both construction methods
            if columns.iter().any(|col| col.as_str() == "qx")
                || columns.iter().any(|col| col.as_str() == "lx")
            {
                let mt_config = MortTableConfig {
                    xml,
                    radix: Some(100_000),
                    pct: Some(1.0),
                    assumption: Some(AssumptionEnum::UDD),
                };

                println!("\n🧮 Comprehensive calculations with LTAM data:");

                // Create different ParamConfigs for different ages
                let params_30 = ParamConfig {
                    mt: mt_config.clone(),
                    i: 0.05,
                    x: 30,
                    n: Some(10),
                    t: None,
                    m: Some(1),
                    moment: Some(1),
                    entry_age: None,
                };

                let params_50 = ParamConfig {
                    mt: mt_config.clone(),
                    i: 0.05,
                    x: 50,
                    n: None,
                    t: None,
                    m: Some(1),
                    moment: Some(1),
                    entry_age: None,
                };

                let params_30_annuity = ParamConfig {
                    mt: mt_config,
                    i: 0.05,
                    x: 30,
                    n: Some(20),
                    t: None,
                    m: Some(1),
                    moment: Some(1),
                    entry_age: None,
                };

                // Life insurance calculations
                let whole_life_30 = Ax(&params_30)?;
                let whole_life_50 = Ax(&params_50)?;
                let term_10_age_30 = Ax1n(&params_30)?;

                // Annuity calculations
                let life_annuity_30 = aax(&params_30)?;
                let temp_annuity_30_20 = aaxn(&params_30_annuity)?;

                // Survival probabilities
                let surv_1p30 = tpx(&params_30.mt, 30.0, 1.0, 0.0, None)?;
                let surv_10p30 = tpx(&params_30.mt, 30.0, 10.0, 0.0, None)?;
                let surv_40p30 = tpx(&params_30.mt, 30.0, 40.0, 0.0, None)?;

                println!("\n   🏥 Life Insurance Values:");
                println!("     Whole life at age 30: {:.6}", whole_life_30);
                println!("     Whole life at age 50: {:.6}", whole_life_50);
                println!("     10-year term at age 30: {:.6}", term_10_age_30);

                println!("\n   💰 Annuity Values:");
                println!("     Life annuity at age 30: {:.6}", life_annuity_30);
                println!(
                    "     20-year temp annuity at age 30: {:.6}",
                    temp_annuity_30_20
                );

                println!("\n   📊 Survival Probabilities from Age 30:");
                println!("     1-year: {:.6} ({:.2}%)", surv_1p30, surv_1p30 * 100.0);
                println!(
                    "     10-year: {:.6} ({:.2}%)",
                    surv_10p30,
                    surv_10p30 * 100.0
                );
                println!(
                    "     40-year: {:.6} ({:.2}%)",
                    surv_40p30,
                    surv_40p30 * 100.0
                );
            }
        }
        Err(e) => {
            println!("⚠️  Could not load LTAM Ultimate file: {}", e);
            println!("   This is likely a data type validation issue in the XLSX parser.");
            demonstrate_ltam_format()?;
        }
    }

    Ok(())
}

/// Demonstrate ELT15 format with sample data when XLSX loading fails
fn demonstrate_elt15_format() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n💡 Creating sample ELT15-style data to demonstrate functionality:");

    // Create sample female mortality data similar to ELT15
    let ages = (0..=110).collect::<Vec<i32>>();
    let qx_values: Vec<f64> = ages
        .iter()
        .map(|&age| {
            // Simplified female mortality pattern
            if age < 1 {
                0.00632 // Infant mortality
            } else if age < 15 {
                0.0003 + (age as f64 * 0.00001) // Low childhood mortality
            } else if age < 25 {
                0.0005 + (age as f64 * 0.00002) // Slightly increasing
            } else if age < 65 {
                0.001 + ((age as f64 - 25.0) * 0.00003) // Gradual increase
            } else {
                // Exponential increase after 65
                let base = 0.01;
                base * (1.08_f64).powf(age as f64 - 65.0).min(1.0)
            }
        })
        .collect();

    let df = df! {
        "age" => ages,
        "qx" => qx_values,
    }?;

    println!("📈 Sample ELT15-style Female mortality data (first 10 rows):");
    println!("{}", df.head(Some(10)));

    // Create MortXML and perform calculations using struct literals
    let mort_xml = MortXML::from_df(df)?;
    let mt_config = MortTableConfig {
        xml: mort_xml,
        radix: Some(100_000),
        pct: Some(1.0),
        assumption: Some(AssumptionEnum::UDD),
    };

    let params = ParamConfig {
        mt: mt_config,
        i: 0.03,
        x: 35,
        n: None,
        t: None,
        m: Some(1),
        moment: Some(1),
        entry_age: None,
    };

    println!("\n🧮 Actuarial calculations with sample ELT15-style data:");
    let whole_life_35 = Ax(&params)?;
    let survival_30p35 = tpx(&params.mt, 35.0, 30.0, 0.0, None)?;

    println!("   Whole life insurance (age 35): {:.6}", whole_life_35);
    println!(
        "   30-year survival from age 35: {:.2}%",
        survival_30p35 * 100.0
    );

    Ok(())
}

/// Demonstrate AM92 format with sample data when XLSX loading fails
fn demonstrate_am92_format() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n💡 Creating sample AM92-style select data to demonstrate functionality:");

    // Create sample select mortality data
    let mut ages = Vec::new();
    let mut qx_values = Vec::new();
    let mut durations = Vec::new();

    // Select period data (first 5 years after issue)
    for dur in 1..=5 {
        for age in 25..=70 {
            ages.push(age);
            durations.push(dur);
            // Select mortality is lower than ultimate, with selection effect wearing off
            let base_qx = 0.001 + (age as f64 * 0.0001);
            let selection_factor = 0.7 + (dur as f64 * 0.06); // Selection effect wears off
            qx_values.push(base_qx * selection_factor);
        }
    }

    let df = df! {
        "age" => ages,
        "qx" => qx_values,
        "duration" => durations,
    }?;

    println!("📈 Sample AM92-style Select mortality data (first 10 rows):");
    println!("{}", df.head(Some(10)));

    println!("   📊 This shows a SELECT table structure with duration column");
    println!("   📊 Duration 1 = first year after policy issue (lowest mortality)");
    println!("   📊 Duration 5 = fifth year after issue (approaching ultimate)");

    Ok(())
}

/// Demonstrate LTAM format with sample data when XLSX loading fails
fn demonstrate_ltam_format() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n💡 Creating sample LTAM-style ultimate data to demonstrate functionality:");

    // Create sample LTAM-style data (ages 20-100)
    let ages = (20..=100).collect::<Vec<i32>>();
    let qx_values: Vec<f64> = ages
        .iter()
        .map(|&age| {
            // Standard ultimate mortality pattern
            if age < 30 {
                0.0003 + (age as f64 * 0.00001)
            } else if age < 50 {
                0.0008 + ((age as f64 - 30.0) * 0.00005)
            } else if age < 70 {
                0.002 + ((age as f64 - 50.0) * 0.0002)
            } else {
                // Exponential increase after 70
                let base = 0.006;
                base * (1.09_f64).powf(age as f64 - 70.0).min(1.0)
            }
        })
        .collect();

    let df = df! {
        "age" => ages,
        "qx" => qx_values,
    }?;

    println!("📈 Sample LTAM-style Ultimate mortality data (first 8 rows):");
    println!("{}", df.head(Some(8)));

    // Perform comprehensive calculations using struct literal
    let mort_xml = MortXML::from_df(df)?;
    let mt_config = MortTableConfig {
        xml: mort_xml,
        radix: Some(100_000),
        pct: Some(1.0),
        assumption: Some(AssumptionEnum::UDD),
    };

    let params_30 = ParamConfig {
        mt: mt_config.clone(),
        i: 0.05,
        x: 30,
        n: None,
        t: None,
        m: Some(1),
        moment: Some(1),
        entry_age: None,
    };

    let params_50 = ParamConfig {
        mt: mt_config,
        i: 0.05,
        x: 50,
        n: None,
        t: None,
        m: Some(1),
        moment: Some(1),
        entry_age: None,
    };

    println!("\n🧮 Comprehensive calculations with sample LTAM-style data:");

    let whole_life_30 = Ax(&params_30)?;
    let whole_life_50 = Ax(&params_50)?;
    let annuity_30 = aax(&params_30)?;
    let survival_40p30 = tpx(&params_30.mt, 30.0, 40.0, 0.0, None)?;

    println!("   🏥 Life Insurance Values:");
    println!("     Whole life at age 30: {:.6}", whole_life_30);
    println!("     Whole life at age 50: {:.6}", whole_life_50);
    println!("   💰 Life annuity at age 30: {:.6}", annuity_30);
    println!(
        "   📊 40-year survival from age 30: {:.2}%",
        survival_40p30 * 100.0
    );

    Ok(())
}
