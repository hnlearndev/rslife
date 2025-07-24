// Example demonstrating the XML documentation and functionality
use rslife::xml::MortXML;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🦀 rslife XML Documentation Demo");
    println!("=================================");

    // Example from documentation - load 2017 CSO mortality table
    println!("\n📡 Loading Mortality Table from SOA...");
    let xml = MortXML::from_url_id(1704)?;
    let table = &xml.tables[0];

    // Display basic information
    println!("✅ Successfully loaded mortality table!");
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

    println!("\n🎉 Documentation example completed successfully!");

    Ok(())
}
