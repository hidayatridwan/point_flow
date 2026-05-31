use chrono::NaiveDate;
use serde::Deserialize;

pub use crate::models::ListQuery;

#[derive(Debug, Deserialize)]
pub struct SalesTrendQuery {
    pub days: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct TopProductsQuery {
    pub days: Option<i32>,
    pub limit: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct HourlySalesQuery {
    pub date: Option<NaiveDate>,
}

#[derive(Debug, Deserialize)]
pub struct LowStockQuery {
    pub limit: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct SalesByCategoryQuery {
    pub days: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct ProductScatterQuery {
    pub days: Option<i32>,
    pub limit: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct CashierPerformanceQuery {
    pub days: Option<i32>,
}
