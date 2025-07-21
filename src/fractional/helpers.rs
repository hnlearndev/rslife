use super::*;

pub fn is_table_layout_approved(config: &MortTableConfig) -> bool {
    // Check table layout
    let approved_table_layouts = vec!["Aggregate", "Ultimate"];
    let key_words = config.xml.content_classification.key_words.clone();

    // Check if any keyword matches any approved table layout
    let tbl_layout_result = key_words.iter().any(|keyword| {
        approved_table_layouts
            .iter()
            .any(|layout| keyword == layout)
    });

    // Content type check
    let approved_content_types = vec![
        "ADB, AD&D",
        "Annuitant Mortality",
        "Claim Cost (in Disability)",
        "Claim Incidence",
        "Claim Termination",
        "CSO / CET",
        "Disability Recovery",
        "Disabled Lives Mortality",
        "Disability Incidence",
        "Group Life",
        "Healthy Lives Mortality",
        "Insured Lives Mortality",
        "Insured Lives Mortality - Ultimate",
        "Projection Scale",
        "Termination Voluntary",
    ];

    let content_type = config.xml.content_classification.content_type.clone();

    // Check if content type is in approved content types
    let content_type_result = approved_content_types
        .iter()
        .any(|approved_type| content_type == *approved_type);

    // Return result
    tbl_layout_result && content_type_result
}
