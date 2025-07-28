use rslife::prelude::*;

fn main() {
    // Load AM92 table
    let am92 = MortData::from_xlsx("data/am92.xlsx", "am92")
        .expect("Failed to load AM92 selected table");

    // Create MortTableConfig
    let mt = MortTableConfig::builder().data(am92).build();

    println!("Original table:");
    println!("min_age: {}", mt.min_age());
    println!("max_age: {}", mt.max_age());
    println!("min_duration: {}", mt.min_duration());
    println!("max_duration: {}", mt.max_duration());
    println!("Column names: {:?}", mt.data.dataframe.get_column_names());
    println!("Table shape: {:?}", mt.data.dataframe.shape());

    // Test what happens when we process the table with get_new_config_with_selected_table
    use rslife::helpers::get_new_config_with_selected_table;
    
    // Test with entry_age = None (ultimate table)
    let ultimate_mt = get_new_config_with_selected_table(&mt, None).unwrap();
    println!("\nUltimate table (entry_age = None):");
    println!("min_age: {}", ultimate_mt.min_age());
    println!("max_age: {}", ultimate_mt.max_age());
    println!("Column names: {:?}", ultimate_mt.data.dataframe.get_column_names());
    println!("Table shape: {:?}", ultimate_mt.data.dataframe.shape());
    
    // Test with entry_age = Some(50) (selected table)
    let selected_mt = get_new_config_with_selected_table(&mt, Some(50)).unwrap();
    println!("\nSelected table (entry_age = 50):");
    println!("min_age: {}", selected_mt.min_age());
    println!("max_age: {}", selected_mt.max_age());
    println!("Column names: {:?}", selected_mt.data.dataframe.get_column_names());
    println!("Table shape: {:?}", selected_mt.data.dataframe.shape());
    
    // Calculate the difference
    let age_range = selected_mt.max_age() - selected_mt.min_age();
    println!("Age range (max - min): {}", age_range);
}
