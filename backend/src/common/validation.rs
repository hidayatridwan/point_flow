use crate::common::error::{AppError, AppResult};

pub fn validate_required(value: &str, field: &str) -> AppResult<()> {
    if value.trim().is_empty() {
        Err(AppError::Validation(format!("{field} is required")))
    } else {
        Ok(())
    }
}

pub fn validate_non_negative(value: f64, field: &str) -> AppResult<()> {
    if value < 0.0 {
        Err(AppError::Validation(format!(
            "{field} must be greater than or equal to 0"
        )))
    } else {
        Ok(())
    }
}

pub fn validate_positive(value: f64, field: &str) -> AppResult<()> {
    if value <= 0.0 {
        Err(AppError::Validation(format!(
            "{field} must be greater than 0"
        )))
    } else {
        Ok(())
    }
}

pub fn validate_status(value: &str, allowed: &[&str], field: &str) -> AppResult<()> {
    if allowed.contains(&value) {
        Ok(())
    } else {
        Err(AppError::Validation(format!(
            "{field} contains an invalid status"
        )))
    }
}
