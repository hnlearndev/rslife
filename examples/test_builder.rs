use rslife::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load SOA mortality table
    let xml = MortXML::from_url_id(1704)?;
    let mt_config = MortTableConfig {
        xml,
        radix: Some(100_000),
        pct: Some(1.0),
        assumption: Some(AssumptionEnum::UDD),
    };

    // Test struct literal first
    println!("=== STRUCT LITERAL (all fields) ===");
    let params_struct = ParamConfig {
        mt: mt_config.clone(),
        i: 0.03,
        x: 35,
        n: Some(20),  // All optional fields explicitly set
        t: None,
        m: Some(1),
        moment: Some(1),
        entry_age: None,
    };
    
    let whole_life_struct = Ax(&params_struct)?;
    println!("Whole life (struct literal): {:.6}", whole_life_struct);

    // Test builder pattern - only set what we need
    println!("\n=== BUILDER PATTERN (minimal fields) ===");
    
    // For Ax function: only needs mt, i, x 
    let params_builder_minimal = ParamConfig::builder()
        .mt(mt_config.clone())
        .i(0.03)
        .x(35)
        .build();
        
    let whole_life_builder = Ax(&params_builder_minimal)?;
    println!("Whole life (builder minimal): {:.6}", whole_life_builder);

    // Comparison: should be same result
    println!("Results match: {}", (whole_life_struct - whole_life_builder).abs() < 1e-10);

    Ok(())
}
