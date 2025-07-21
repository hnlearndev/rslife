use super::*;

pub fn get_new_config(config: &MortTableConfig, g: f64) -> MortTableConfig {
    // Replace the effective interest rate with the adjusted one
    let i = config.int_rate.unwrap();
    let int_rate = (1.0 + i) / (1.0 + g) - 1.0;
    let mut new_config = config.clone();
    new_config.int_rate = Some(int_rate);
    new_config
}
