pub mod auth;
pub mod documents;
pub mod master;
pub mod reports;

use crate::error::{AppError, AppResult};

pub const PURCHASE_ORDER_STATUSES: &[&str] = &[
    "draft",
    "ordered",
    "partially_received",
    "received",
    "cancelled",
];
pub const GOODS_RECEIPT_STATUSES: &[&str] = &["draft", "posted", "cancelled"];
pub const STOCK_ADJUSTMENT_STATUSES: &[&str] = &["draft", "posted", "cancelled"];
pub const STOCK_CONVERSION_STATUSES: &[&str] = &["draft", "posted", "cancelled"];
pub const SALE_STATUSES: &[&str] = &["completed", "cancelled"];
pub const STOCK_REPORT_STATUSES: &[&str] = &["in_stock", "low_stock", "inactive"];
pub const STOCK_MOVEMENT_TYPES: &[&str] = &[
    "stock_in",
    "stock_out",
    "sale",
    "purchase_receipt",
    "adjustment",
    "conversion_in",
    "conversion_out",
    "correction",
];

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

pub fn parse_status_filter(
    status: Option<&str>,
    allowed: &[&str],
    resource: &str,
) -> AppResult<Vec<String>> {
    status
        .unwrap_or("")
        .split(',')
        .map(str::trim)
        .filter(|status| !status.is_empty())
        .map(|status| {
            if allowed.contains(&status) {
                Ok(status.to_string())
            } else {
                Err(AppError::BadRequest(format!(
                    "status contains invalid {resource} status: {status}"
                )))
            }
        })
        .collect()
}
