use crate::mt_config::MortTableConfig;

pub type ErrorVec = Vec<(&'static str, String)>;

pub fn collect_age_bounds_errors(
    mt: &MortTableConfig,
    errors: &mut ErrorVec,
) -> Option<(f64, f64)> {
    let min_age_result = mt.min_age();
    let max_age_result = mt.max_age();
    match (&min_age_result, &max_age_result) {
        (Ok(min), Ok(max)) => Some((*min as f64, *max as f64)),
        _ => {
            if min_age_result.is_err() {
                errors.push(("mt", "Failed to get min_age from mortality table".into()));
            }
            if max_age_result.is_err() {
                errors.push(("mt", "Failed to get max_age from mortality table".into()));
            }
            None
        }
    }
}

pub fn validate_age_boundaries(x: f64, min_age: f64, max_age: f64, errors: &mut ErrorVec) {
    if x < min_age || x > max_age {
        errors.push((
            "x",
            format!("age {x} must be between {min_age} and {max_age} from mortality table"),
        ));
    }
}

pub fn validate_entry_age(entry_age: Option<u32>, x: f64, errors: &mut ErrorVec) {
    if let Some(entry_age) = entry_age {
        let entry_age_f = entry_age as f64;
        if entry_age_f > x {
            errors.push((
                "entry_age",
                format!("entry_age {entry_age} cannot exceed age {x}"),
            ));
        }
    }
}
