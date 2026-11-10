use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    pub search: Option<String>,
    pub from: Option<NaiveDate>,
    pub to: Option<NaiveDate>,
    pub status: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

impl ListQuery {
    pub fn limit(&self) -> i64 {
        self.limit.unwrap_or(50).clamp(1, 100)
    }

    pub fn offset(&self) -> i64 {
        self.offset.unwrap_or(0).max(0)
    }

    pub fn search_pattern(&self) -> String {
        format!("%{}%", self.search.as_deref().unwrap_or("").to_lowercase())
    }
}

#[derive(Debug, Deserialize)]
pub struct CashDrawerMovementQuery {
    pub search: Option<String>,
    pub from: Option<NaiveDate>,
    pub to: Option<NaiveDate>,
    pub status: Option<String>,
    #[serde(rename = "type")]
    pub movement_type: Option<String>,
    pub cash_register_session_id: Option<i64>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

impl CashDrawerMovementQuery {
    pub fn limit(&self) -> i64 {
        self.limit.unwrap_or(50).clamp(1, 100)
    }

    pub fn offset(&self) -> i64 {
        self.offset.unwrap_or(0).max(0)
    }

    pub fn search_pattern(&self) -> String {
        format!("%{}%", self.search.as_deref().unwrap_or("").to_lowercase())
    }
}

#[derive(Debug, Deserialize)]
pub struct AuditLogQuery {
    pub search: Option<String>,
    pub from: Option<NaiveDate>,
    pub to: Option<NaiveDate>,
    pub action: Option<String>,
    pub resource_type: Option<String>,
    pub outcome: Option<String>,
    pub actor_id: Option<i64>,
    pub request_id: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

impl AuditLogQuery {
    pub fn limit(&self) -> i64 {
        self.limit.unwrap_or(50).clamp(1, 100)
    }

    pub fn offset(&self) -> i64 {
        self.offset.unwrap_or(0).max(0)
    }

    pub fn search_pattern(&self) -> String {
        format!("%{}%", self.search.as_deref().unwrap_or("").to_lowercase())
    }
}

#[derive(Debug, Deserialize)]
pub struct StockMovementReportQuery {
    pub search: Option<String>,
    pub from: Option<NaiveDate>,
    pub to: Option<NaiveDate>,
    #[serde(rename = "type")]
    pub movement_type: Option<String>,
    pub product_sku_id: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

impl StockMovementReportQuery {
    pub fn limit(&self) -> i64 {
        self.limit.unwrap_or(50).clamp(1, 100)
    }

    pub fn offset(&self) -> i64 {
        self.offset.unwrap_or(0).max(0)
    }

    pub fn search_pattern(&self) -> String {
        format!("%{}%", self.search.as_deref().unwrap_or("").to_lowercase())
    }
}

#[derive(Debug, Serialize)]
pub struct ApiResponse<T, M = ()> {
    pub data: Option<T>,
    pub meta: Option<M>,
    pub error: Option<ApiErrorBody>,
    pub request_id: String,
}

impl<T> ApiResponse<T, ()> {
    pub fn success(data: T) -> Self {
        Self {
            data: Some(data),
            meta: None,
            error: None,
            request_id: crate::common::request_context::current_request_id(),
        }
    }
}

impl<T, M> ApiResponse<T, M> {
    pub fn success_with_meta(data: T, meta: M) -> Self {
        Self {
            data: Some(data),
            meta: Some(meta),
            error: None,
            request_id: crate::common::request_context::current_request_id(),
        }
    }
}

impl ApiResponse<(), ()> {
    pub fn error(code: &'static str, message: String) -> Self {
        Self {
            data: None,
            meta: None,
            error: Some(ApiErrorBody { code, message }),
            request_id: crate::common::request_context::current_request_id(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ApiErrorBody {
    pub code: &'static str,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct PaginationMeta {
    pub page: i64,
    pub limit: i64,
    pub total_records: i64,
    pub total_pages: i64,
}

impl PaginationMeta {
    pub fn new(limit: i64, offset: i64, total_records: i64) -> Self {
        let page = (offset / limit) + 1;
        let total_pages = if total_records == 0 {
            0
        } else {
            (total_records + limit - 1) / limit
        };

        Self {
            page,
            limit,
            total_records,
            total_pages,
        }
    }
}

#[derive(Debug)]
pub struct Paginated<T> {
    pub data: Vec<T>,
    pub meta: PaginationMeta,
}

impl<T> Paginated<T> {
    pub fn new(data: Vec<T>, limit: i64, offset: i64, total_records: i64) -> Self {
        Self {
            data,
            meta: PaginationMeta::new(limit, offset, total_records),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct MessageResponse {
    pub message: &'static str,
}

/// Hasil upload file ke object storage: object key + URL publik.
#[derive(Debug, Serialize)]
pub struct UploadResponse {
    pub key: String,
    pub url: String,
}

#[derive(Debug, Serialize, FromRow)]
pub struct AuditLogResponse {
    pub id: i64,
    pub request_id: Option<Uuid>,
    pub actor_id: Option<i64>,
    pub actor_name: Option<String>,
    pub actor_email: Option<String>,
    pub actor_role: Option<String>,
    pub action: String,
    pub resource_type: String,
    pub resource_id: Option<i64>,
    pub resource_label: Option<String>,
    pub outcome: String,
    pub before_data: Option<Value>,
    pub after_data: Option<Value>,
    pub metadata: Option<Value>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub occurred_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CancelRequest {
    pub reason: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct ChangePasswordRequest {
    pub current_password: String,
    pub new_password: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub access_token: String,
    pub token_type: &'static str,
    pub expires_in: i64,
    pub user: UserResponse,
}

#[derive(Debug, Serialize, FromRow)]
pub struct UserResponse {
    pub id: i64,
    pub name: String,
    pub email: String,
    pub role_id: i64,
    pub role_code: String,
    pub role_name: String,
    pub role: String,
    pub permissions: Vec<String>,
    pub is_active: bool,
    pub last_login_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
pub struct CreateUserRequest {
    pub name: String,
    pub email: String,
    pub password: String,
    pub role_id: Option<i64>,
    pub role: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateUserRequest {
    pub name: Option<String>,
    pub email: Option<String>,
    pub password: Option<String>,
    pub role_id: Option<i64>,
    pub role: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct RoleResponse {
    pub id: i64,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub is_system: bool,
    pub permissions: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct PermissionResponse {
    pub id: i64,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateRoleRequest {
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub permission_codes: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateRoleRequest {
    pub code: Option<String>,
    pub name: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateRolePermissionsRequest {
    pub permission_codes: Vec<String>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct ProductCategoryResponse {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
pub struct CreateProductCategoryRequest {
    pub name: String,
    pub description: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateProductCategoryRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct ProductResponse {
    pub id: i64,
    pub category_id: Option<i64>,
    pub category_name: Option<String>,
    pub name: String,
    pub is_active: bool,
    pub image_url: Option<String>,
    pub photo_url: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
pub struct CreateProductRequest {
    pub category_id: Option<i64>,
    pub name: String,
    pub is_active: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateProductRequest {
    pub category_id: Option<i64>,
    pub name: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct UnitResponse {
    pub id: i64,
    pub code: String,
    pub name: String,
    pub allow_decimal: bool,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
pub struct CreateUnitRequest {
    pub code: String,
    pub name: String,
    pub allow_decimal: Option<bool>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateUnitRequest {
    pub code: Option<String>,
    pub name: Option<String>,
    pub allow_decimal: Option<bool>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct ProductSkuResponse {
    pub id: i64,
    pub product_id: i64,
    pub product_name: String,
    pub category_id: Option<i64>,
    pub category_name: Option<String>,
    pub sku: String,
    pub barcode: Option<String>,
    pub unit_id: i64,
    pub unit_code: String,
    pub unit_name: String,
    pub contents_qty: f64,
    pub purchase_price: f64,
    pub selling_price: f64,
    pub stock: f64,
    pub minimum_stock: f64,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
pub struct CreateProductSkuRequest {
    pub product_id: i64,
    pub sku: String,
    pub barcode: Option<String>,
    pub unit_id: i64,
    pub contents_qty: Option<f64>,
    pub purchase_price: f64,
    pub selling_price: f64,
    pub stock: Option<f64>,
    pub minimum_stock: Option<f64>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateProductSkuRequest {
    pub sku: Option<String>,
    pub barcode: Option<String>,
    pub unit_id: Option<i64>,
    pub contents_qty: Option<f64>,
    pub purchase_price: Option<f64>,
    pub selling_price: Option<f64>,
    pub stock: Option<f64>,
    pub minimum_stock: Option<f64>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct SupplierResponse {
    pub id: i64,
    pub name: String,
    pub contact_name: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub address: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
pub struct CreateSupplierRequest {
    pub name: String,
    pub contact_name: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub address: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateSupplierRequest {
    pub name: Option<String>,
    pub contact_name: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub address: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct CashRegisterResponse {
    pub id: i64,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
pub struct CreateCashRegisterRequest {
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateCashRegisterRequest {
    pub code: Option<String>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub status: Option<String>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct CashRegisterSessionResponse {
    pub id: i64,
    pub session_no: String,
    pub cash_register_id: i64,
    pub cash_register_code: String,
    pub cash_register_name: String,
    pub opened_at: DateTime<Utc>,
    pub opened_by: i64,
    pub opened_by_name: String,
    pub opening_cash: f64,
    pub opening_notes: Option<String>,
    pub closed_at: Option<DateTime<Utc>>,
    pub closed_by: Option<i64>,
    pub closed_by_name: Option<String>,
    pub closing_cash_counted: Option<f64>,
    pub expected_cash: Option<f64>,
    pub cash_difference: Option<f64>,
    pub closing_notes: Option<String>,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct OpenCashRegisterSessionRequest {
    pub cash_register_id: i64,
    pub opening_cash: f64,
    pub opening_notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CloseCashRegisterSessionRequest {
    pub closing_cash_counted: f64,
    pub closing_notes: Option<String>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct CashDrawerMovementResponse {
    pub id: i64,
    pub movement_no: String,
    pub cash_register_session_id: i64,
    pub session_no: String,
    pub cash_register_id: i64,
    pub cash_register_code: String,
    pub cash_register_name: String,
    pub occurred_at: DateTime<Utc>,
    pub created_by: i64,
    pub created_by_name: String,
    pub movement_type: String,
    pub direction: String,
    pub amount: f64,
    pub reason: Option<String>,
    pub notes: Option<String>,
    pub status: String,
    pub cancelled_at: Option<DateTime<Utc>>,
    pub cancelled_by: Option<i64>,
    pub cancelled_by_name: Option<String>,
    pub cancel_reason: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateCashDrawerMovementRequest {
    pub cash_register_session_id: i64,
    pub occurred_at: Option<DateTime<Utc>>,
    pub movement_type: String,
    pub amount: f64,
    pub reason: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct EmployeeResponse {
    pub id: i64,
    pub name: String,
    pub position: String,
    pub phone: Option<String>,
    pub address: Option<String>,
    pub base_salary: f64,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
pub struct CreateEmployeeRequest {
    pub name: String,
    pub position: String,
    pub phone: Option<String>,
    pub address: Option<String>,
    pub base_salary: f64,
    pub is_active: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateEmployeeRequest {
    pub name: Option<String>,
    pub position: Option<String>,
    pub phone: Option<String>,
    pub address: Option<String>,
    pub base_salary: Option<f64>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct PurchaseOrderResponse {
    pub id: i64,
    pub po_no: String,
    pub supplier_id: i64,
    pub supplier_name: String,
    pub order_date: NaiveDate,
    pub expected_date: Option<NaiveDate>,
    pub status: String,
    pub subtotal_amount: f64,
    pub notes: Option<String>,
    pub ordered_by: Option<i64>,
    pub ordered_at: Option<DateTime<Utc>>,
    pub cancelled_at: Option<DateTime<Utc>>,
    pub cancelled_by: Option<i64>,
    pub cancel_reason: Option<String>,
    pub created_by: Option<i64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct PurchaseOrderItemResponse {
    pub id: i64,
    pub purchase_order_id: i64,
    pub product_id: i64,
    pub product_sku_id: i64,
    pub product_name: String,
    pub product_sku: String,
    pub unit: String,
    pub quantity_ordered: f64,
    pub quantity_received: f64,
    pub unit_cost: f64,
    pub line_total: f64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct PurchaseOrderDetailResponse {
    #[serde(flatten)]
    pub purchase_order: PurchaseOrderResponse,
    pub items: Vec<PurchaseOrderItemResponse>,
}

#[derive(Debug, Deserialize)]
pub struct CreatePurchaseOrderRequest {
    pub supplier_id: i64,
    pub order_date: Option<NaiveDate>,
    pub expected_date: Option<NaiveDate>,
    pub notes: Option<String>,
    pub items: Vec<PurchaseOrderItemInput>,
}

#[derive(Debug, Deserialize)]
pub struct PurchaseOrderItemInput {
    pub product_sku_id: i64,
    pub quantity_ordered: f64,
    pub unit_cost: f64,
}

#[derive(Debug, Serialize, FromRow)]
pub struct GoodsReceiptResponse {
    pub id: i64,
    pub gr_no: String,
    pub purchase_order_id: i64,
    pub po_no: String,
    pub received_at: DateTime<Utc>,
    pub status: String,
    pub received_by: Option<i64>,
    pub posted_at: Option<DateTime<Utc>>,
    pub posted_by: Option<i64>,
    pub cancelled_at: Option<DateTime<Utc>>,
    pub cancelled_by: Option<i64>,
    pub cancel_reason: Option<String>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct GoodsReceiptItemResponse {
    pub id: i64,
    pub goods_receipt_id: i64,
    pub purchase_order_item_id: i64,
    pub product_id: i64,
    pub product_sku_id: i64,
    pub product_name: String,
    pub product_sku: String,
    pub unit: String,
    pub quantity_received: f64,
    pub unit_cost: f64,
    pub margin_percent: Option<f64>,
    pub line_total: f64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct GoodsReceiptDetailResponse {
    #[serde(flatten)]
    pub goods_receipt: GoodsReceiptResponse,
    pub items: Vec<GoodsReceiptItemResponse>,
}

#[derive(Debug, Deserialize)]
pub struct CreateGoodsReceiptRequest {
    pub purchase_order_id: i64,
    pub received_at: Option<DateTime<Utc>>,
    pub notes: Option<String>,
    pub items: Vec<GoodsReceiptItemInput>,
}

#[derive(Debug, Deserialize)]
pub struct GoodsReceiptItemInput {
    pub purchase_order_item_id: i64,
    pub quantity_received: f64,
    pub unit_cost: Option<f64>,
    pub margin_percent: Option<f64>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct StockAdjustmentResponse {
    pub id: i64,
    pub adjustment_no: String,
    pub adjusted_at: DateTime<Utc>,
    pub status: String,
    pub reason: String,
    pub notes: Option<String>,
    pub created_by: Option<i64>,
    pub posted_at: Option<DateTime<Utc>>,
    pub posted_by: Option<i64>,
    pub cancelled_at: Option<DateTime<Utc>>,
    pub cancelled_by: Option<i64>,
    pub cancel_reason: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct StockAdjustmentItemResponse {
    pub id: i64,
    pub stock_adjustment_id: i64,
    pub product_id: i64,
    pub product_sku_id: i64,
    pub product_name: String,
    pub product_sku: String,
    pub unit: String,
    pub stock_before: f64,
    pub stock_after: f64,
    pub quantity_delta: f64,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct StockAdjustmentDetailResponse {
    #[serde(flatten)]
    pub stock_adjustment: StockAdjustmentResponse,
    pub items: Vec<StockAdjustmentItemResponse>,
}

#[derive(Debug, Deserialize)]
pub struct CreateStockAdjustmentRequest {
    pub adjusted_at: Option<DateTime<Utc>>,
    pub reason: String,
    pub notes: Option<String>,
    pub items: Vec<StockAdjustmentItemInput>,
}

#[derive(Debug, Deserialize)]
pub struct StockAdjustmentItemInput {
    pub product_sku_id: i64,
    pub stock_after: f64,
    pub notes: Option<String>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct StockConversionResponse {
    pub id: i64,
    pub conversion_no: String,
    pub converted_at: DateTime<Utc>,
    pub status: String,
    pub notes: Option<String>,
    pub created_by: Option<i64>,
    pub posted_at: Option<DateTime<Utc>>,
    pub posted_by: Option<i64>,
    pub cancelled_at: Option<DateTime<Utc>>,
    pub cancelled_by: Option<i64>,
    pub cancel_reason: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct StockConversionItemResponse {
    pub id: i64,
    pub stock_conversion_id: i64,
    pub from_product_id: i64,
    pub from_product_sku_id: i64,
    pub from_product_name: String,
    pub from_product_sku: String,
    pub from_unit: String,
    pub from_stock_before: f64,
    pub from_quantity: f64,
    pub from_stock_after: f64,
    pub to_product_id: i64,
    pub to_product_sku_id: i64,
    pub to_product_name: String,
    pub to_product_sku: String,
    pub to_unit: String,
    pub to_stock_before: f64,
    pub to_quantity: f64,
    pub to_stock_after: f64,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct StockConversionDetailResponse {
    #[serde(flatten)]
    pub stock_conversion: StockConversionResponse,
    pub items: Vec<StockConversionItemResponse>,
}

#[derive(Debug, Deserialize)]
pub struct CreateStockConversionRequest {
    pub converted_at: Option<DateTime<Utc>>,
    pub notes: Option<String>,
    pub items: Vec<StockConversionItemInput>,
}

#[derive(Debug, Deserialize)]
pub struct StockConversionItemInput {
    pub from_product_sku_id: i64,
    pub from_quantity: f64,
    pub to_product_sku_id: i64,
    pub to_quantity: f64,
    pub notes: Option<String>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct SaleResponse {
    pub id: i64,
    pub transaction_no: String,
    pub sold_at: DateTime<Utc>,
    pub cashier_id: i64,
    pub cashier_name: String,
    pub member_id: Option<i64>,
    pub cash_register_session_id: Option<i64>,
    pub cash_register_id: Option<i64>,
    pub cash_register_code: Option<String>,
    pub cash_register_name: Option<String>,
    pub subtotal_amount: f64,
    pub total_amount: f64,
    pub paid_amount: f64,
    pub change_amount: f64,
    pub status: String,
    pub notes: Option<String>,
    pub cancelled_at: Option<DateTime<Utc>>,
    pub cancelled_by: Option<i64>,
    pub cancel_reason: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct SaleItemResponse {
    pub id: i64,
    pub sale_id: i64,
    pub product_id: i64,
    pub product_sku_id: i64,
    pub product_name: String,
    pub product_sku: String,
    pub unit: String,
    pub quantity: f64,
    pub unit_price: f64,
    pub unit_cost: f64,
    pub line_total: f64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct SaleDetailResponse {
    #[serde(flatten)]
    pub sale: SaleResponse,
    pub items: Vec<SaleItemResponse>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct SaleReturnResponse {
    pub id: i64,
    pub return_no: String,
    pub sale_id: i64,
    pub sale_transaction_no: String,
    pub returned_at: DateTime<Utc>,
    pub cashier_id: i64,
    pub cashier_name: String,
    pub cash_register_session_id: i64,
    pub cash_register_id: i64,
    pub cash_register_code: String,
    pub cash_register_name: String,
    pub subtotal_amount: f64,
    pub refund_amount: f64,
    pub status: String,
    pub reason: Option<String>,
    pub notes: Option<String>,
    pub cancelled_at: Option<DateTime<Utc>>,
    pub cancelled_by: Option<i64>,
    pub cancel_reason: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct SaleReturnItemResponse {
    pub id: i64,
    pub sale_return_id: i64,
    pub sale_item_id: i64,
    pub product_id: i64,
    pub product_sku_id: i64,
    pub product_name: String,
    pub product_sku: String,
    pub unit: String,
    pub quantity: f64,
    pub unit_price: f64,
    pub line_total: f64,
    pub restock: bool,
    pub reason: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct SaleReturnDetailResponse {
    #[serde(flatten)]
    pub sale_return: SaleReturnResponse,
    pub items: Vec<SaleReturnItemResponse>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct ReturnableSaleItemResponse {
    pub id: i64,
    pub sale_id: i64,
    pub product_id: i64,
    pub product_sku_id: i64,
    pub product_name: String,
    pub product_sku: String,
    pub unit: String,
    pub quantity: f64,
    pub returned_quantity: f64,
    pub returnable_quantity: f64,
    pub unit_price: f64,
    pub line_total: f64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateSaleRequest {
    pub sold_at: Option<DateTime<Utc>>,
    pub cash_register_session_id: Option<i64>,
    pub member_id: Option<i64>,
    pub paid_amount: f64,
    pub notes: Option<String>,
    pub items: Vec<SaleItemInput>,
}

#[derive(Debug, Deserialize)]
pub struct SaleItemInput {
    pub product_sku_id: i64,
    pub quantity: f64,
}

#[derive(Debug, Deserialize)]
pub struct CreateSaleReturnRequest {
    pub sale_id: i64,
    pub returned_at: Option<DateTime<Utc>>,
    pub cash_register_session_id: i64,
    pub reason: Option<String>,
    pub notes: Option<String>,
    pub items: Vec<SaleReturnItemInput>,
}

#[derive(Debug, Deserialize)]
pub struct SaleReturnItemInput {
    pub sale_item_id: i64,
    pub quantity: f64,
    pub restock: bool,
    pub reason: Option<String>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct PayrollSlipResponse {
    pub id: i64,
    pub slip_no: String,
    pub employee_id: i64,
    pub employee_name: String,
    pub employee_position: String,
    pub period_start: NaiveDate,
    pub period_end: NaiveDate,
    pub base_salary: f64,
    pub bonus: f64,
    pub deduction: f64,
    pub net_salary: f64,
    pub created_by: Option<i64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreatePayrollSlipRequest {
    pub employee_id: i64,
    pub period_start: NaiveDate,
    pub period_end: NaiveDate,
    pub bonus: Option<f64>,
    pub deduction: Option<f64>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct DashboardResponse {
    pub transaction_count_today: i64,
    pub revenue_today: f64,
    pub low_stock_product_count: i64,
    pub pending_purchase_order_count: i64,
    pub draft_stock_adjustment_count: i64,
    pub active_employee_count: i64,
}

#[derive(Debug, Serialize, FromRow)]
pub struct DailySalesReport {
    pub sales_date: NaiveDate,
    pub transaction_count: i64,
    pub subtotal_amount: f64,
    pub total_amount: f64,
}

#[derive(Debug, Serialize, FromRow)]
pub struct MonthlySalesReport {
    pub sales_month: NaiveDate,
    pub transaction_count: i64,
    pub subtotal_amount: f64,
    pub total_amount: f64,
}

#[derive(Debug, Serialize, FromRow)]
pub struct StockReport {
    pub product_sku_id: i64,
    pub product_id: i64,
    pub product_name: String,
    pub sku: String,
    pub barcode: Option<String>,
    pub category_name: Option<String>,
    pub purchase_price: f64,
    pub selling_price: f64,
    pub stock: f64,
    pub minimum_stock: f64,
    pub unit: String,
    pub quantity_on_order: f64,
    pub is_low_stock: bool,
    pub is_active: bool,
}

#[derive(Debug, Serialize, FromRow)]
pub struct StockMovementReport {
    pub id: i64,
    pub created_at: DateTime<Utc>,
    pub product_id: i64,
    pub product_sku_id: i64,
    pub product_name: String,
    pub sku: String,
    pub unit: String,
    pub movement_type: String,
    pub direction: String,
    pub quantity: f64,
    pub stock_before: f64,
    pub stock_after: f64,
    pub quantity_delta: f64,
    pub reference_type: String,
    pub reference_id: Option<i64>,
    pub reference_no: Option<String>,
    pub notes: Option<String>,
    pub created_by: Option<i64>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct PurchaseOrderReport {
    pub id: i64,
    pub po_no: String,
    pub order_date: NaiveDate,
    pub expected_date: Option<NaiveDate>,
    pub supplier_name: String,
    pub status: String,
    pub item_count: i64,
    pub total_quantity_ordered: f64,
    pub total_quantity_received: f64,
    pub total_quantity_remaining: f64,
    pub subtotal_amount: f64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct GoodsReceiptReport {
    pub id: i64,
    pub gr_no: String,
    pub received_at: DateTime<Utc>,
    pub status: String,
    pub po_no: String,
    pub supplier_name: String,
    pub item_count: i64,
    pub total_quantity_received: f64,
    pub total_amount: f64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct StockAdjustmentReport {
    pub id: i64,
    pub adjustment_no: String,
    pub adjusted_at: DateTime<Utc>,
    pub status: String,
    pub reason: String,
    pub item_count: i64,
    pub total_quantity_increased: f64,
    pub total_quantity_decreased: f64,
    pub created_by: Option<i64>,
    pub posted_at: Option<DateTime<Utc>>,
    pub posted_by: Option<i64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct PayrollReport {
    pub period_start: NaiveDate,
    pub period_end: NaiveDate,
    pub slip_count: i64,
    pub total_base_salary: f64,
    pub total_bonus: f64,
    pub total_deduction: f64,
    pub total_net_salary: f64,
}

#[derive(Debug, Serialize, FromRow)]
pub struct SalesTrendPoint {
    pub date: NaiveDate,
    pub revenue: f64,
    pub transaction_count: i64,
}

#[derive(Debug, Serialize, FromRow)]
pub struct TopProductItem {
    pub product_sku_id: i64,
    pub product_name: String,
    pub product_sku: String,
    pub unit: String,
    pub revenue: f64,
    pub quantity_sold: f64,
}

#[derive(Debug, Serialize, FromRow)]
pub struct HourlySalesPoint {
    pub hour: i32,
    pub revenue: f64,
    pub transaction_count: i64,
}

#[derive(Debug, Serialize, FromRow)]
pub struct SalesByCategoryItem {
    pub category_name: String,
    pub revenue: f64,
    pub transaction_count: i64,
    pub percentage: f64,
}

#[derive(Debug, Serialize, FromRow)]
pub struct ProductScatterItem {
    pub product_sku_id: i64,
    pub product_name: String,
    pub product_sku: String,
    pub unit: String,
    pub selling_price: f64,
    pub quantity_sold: f64,
    pub revenue: f64,
}

#[derive(Debug, Serialize, FromRow)]
pub struct CashierPerformance {
    pub cashier_id: i64,
    pub cashier_name: String,
    pub transaction_count: i64,
    pub revenue: f64,
    pub avg_basket_size: f64,
    pub return_count: i64,
}

#[derive(Debug, Serialize, FromRow)]
pub struct ProcurementFunnel {
    pub po_draft: i64,
    pub po_ordered: i64,
    pub po_partially_received: i64,
    pub po_received: i64,
    pub po_cancelled: i64,
    pub gr_draft: i64,
    pub gr_posted: i64,
    pub gr_cancelled: i64,
    pub sa_draft: i64,
    pub sa_posted: i64,
    pub sa_cancelled: i64,
}

/// Filter list transaksi keuangan manual (desain 2.2).
#[derive(Debug, Deserialize)]
pub struct FinancialTransactionQuery {
    pub search: Option<String>,
    pub from: Option<NaiveDate>,
    pub to: Option<NaiveDate>,
    pub status: Option<String>,
    #[serde(rename = "type")]
    pub transaction_type: Option<String>,
    pub financial_category_id: Option<i64>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

impl FinancialTransactionQuery {
    pub fn limit(&self) -> i64 {
        self.limit.unwrap_or(50).clamp(1, 100)
    }

    pub fn offset(&self) -> i64 {
        self.offset.unwrap_or(0).max(0)
    }

    pub fn search_pattern(&self) -> String {
        format!("%{}%", self.search.as_deref().unwrap_or("").to_lowercase())
    }
}

#[derive(Debug, Serialize, FromRow)]
pub struct FinancialCategoryResponse {
    pub id: i64,
    pub name: String,
    pub category_type: String,
    pub description: Option<String>,
    pub is_active: bool,
    pub created_by: Option<i64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateFinancialCategoryRequest {
    pub name: String,
    pub category_type: String,
    pub description: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateFinancialCategoryRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct FinancialTransactionResponse {
    pub id: i64,
    pub transaction_no: String,
    pub occurred_at: DateTime<Utc>,
    pub transaction_type: String,
    pub financial_category_id: i64,
    pub financial_category_name: String,
    pub amount: f64,
    pub is_cash: bool,
    pub description: Option<String>,
    pub status: String,
    pub cancelled_at: Option<DateTime<Utc>>,
    pub cancelled_by: Option<i64>,
    pub cancel_reason: Option<String>,
    pub recorded_by: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateFinancialTransactionRequest {
    pub occurred_at: Option<DateTime<Utc>>,
    pub transaction_type: String,
    pub financial_category_id: i64,
    pub amount: f64,
    pub is_cash: Option<bool>,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateFinancialTransactionRequest {
    pub occurred_at: Option<DateTime<Utc>>,
    pub financial_category_id: Option<i64>,
    pub amount: Option<f64>,
    pub is_cash: Option<bool>,
    pub description: Option<String>,
}

/// Filter list anggota: pencarian nama/telepon/nomor + status aktif.
#[derive(Debug, Deserialize)]
pub struct MemberQuery {
    pub search: Option<String>,
    pub is_active: Option<bool>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

impl MemberQuery {
    pub fn limit(&self) -> i64 {
        self.limit.unwrap_or(50).clamp(1, 100)
    }

    pub fn offset(&self) -> i64 {
        self.offset.unwrap_or(0).max(0)
    }

    pub fn search_pattern(&self) -> String {
        format!("%{}%", self.search.as_deref().unwrap_or("").to_lowercase())
    }
}

#[derive(Debug, Serialize, FromRow)]
pub struct MemberResponse {
    pub id: i64,
    pub member_no: String,
    pub full_name: String,
    pub phone: String,
    pub email: Option<String>,
    pub address: Option<String>,
    pub joined_at: DateTime<Utc>,
    pub is_active: bool,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateMemberRequest {
    pub full_name: String,
    pub phone: String,
    pub email: Option<String>,
    pub password: String,
    pub address: Option<String>,
    pub is_active: Option<bool>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateMemberRequest {
    pub full_name: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub address: Option<String>,
    pub is_active: Option<bool>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ResetMemberPasswordRequest {
    pub new_password: String,
}

/// Login storefront: identitas member adalah telepon (unique) + password.
/// `ChangePasswordRequest` yang sudah ada dipakai ulang untuk ganti password.
#[derive(Debug, Deserialize)]
pub struct MemberLoginRequest {
    pub phone: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct MemberLoginResponse {
    pub access_token: String,
    pub token_type: &'static str,
    pub expires_in: i64,
    pub member: MemberResponse,
}

/// Laba rugi derivatif (desain 2.2): pendapatan net retur, HPP snapshot
/// `unit_cost` (fallback harga beli berjalan untuk penjualan pra-migrasi),
/// beban gaji + beban manual, laba kotor & bersih.
#[derive(Debug, Serialize, FromRow)]
pub struct IncomeStatementResponse {
    pub gross_revenue: f64,
    pub return_amount: f64,
    pub net_revenue: f64,
    pub cogs: f64,
    pub gross_profit: f64,
    pub payroll_expense: f64,
    pub other_expense: f64,
    pub net_profit: f64,
}

/// Arus kas derivatif (desain 2.2): transaksi non-tunai (`is_cash = false`)
/// tidak masuk kas; pergerakan drawer `cash_in` bukan kas keluar.
#[derive(Debug, Serialize, FromRow)]
pub struct CashFlowResponse {
    pub cash_in_sales: f64,
    pub cash_in_manual: f64,
    pub total_cash_in: f64,
    pub cash_out_drawer: f64,
    pub cash_out_manual: f64,
    pub total_cash_out: f64,
    pub net_cash_flow: f64,
}

/// Posisi keuangan sederhana (desain 2.2): kas kumulatif + persediaan
/// (stok × harga beli saat ini) + laba kumulatif.
#[derive(Debug, Serialize, FromRow)]
pub struct FinancialPositionResponse {
    pub as_of: NaiveDate,
    pub cash: f64,
    pub inventory_value: f64,
    pub total_assets: f64,
    pub cumulative_net_profit: f64,
}

/// Profit sharing (SHU) — desain Bagian 4.1. Teknis full English; istilah "SHU"
/// hanya tampil di UI via i18n (locale id), backend selalu English.
#[derive(Debug, Deserialize)]
pub struct CreateProfitSharingPeriodRequest {
    pub period_name: String,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub total_pool: Option<f64>,
    pub member_pct: f64,
    pub reserve_pct: f64,
    pub management_pct: f64,
    pub supervisor_pct: f64,
    pub social_pct: f64,
    pub education_pct: f64,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateProfitSharingPeriodRequest {
    pub period_name: Option<String>,
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
    /// Absent = tidak diubah; null = kembali ke auto (hapus pool manual).
    #[serde(default)]
    pub total_pool: Option<Option<f64>>,
    pub member_pct: Option<f64>,
    pub reserve_pct: Option<f64>,
    pub management_pct: Option<f64>,
    pub supervisor_pct: Option<f64>,
    pub social_pct: Option<f64>,
    pub education_pct: Option<f64>,
    pub notes: Option<String>,
}

/// Filter distribusi: pencarian nama/telepon/nomor anggota.
#[derive(Debug, Deserialize)]
pub struct DistributionQuery {
    pub search: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

impl DistributionQuery {
    pub fn limit(&self) -> i64 {
        self.limit.unwrap_or(50).clamp(1, 100)
    }

    pub fn offset(&self) -> i64 {
        self.offset.unwrap_or(0).max(0)
    }

    pub fn search_pattern(&self) -> String {
        format!("%{}%", self.search.as_deref().unwrap_or("").to_lowercase())
    }
}

#[derive(Debug, Serialize, FromRow)]
pub struct ProfitSharingPeriodResponse {
    pub id: i64,
    pub period_name: String,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub status: String,
    pub total_pool: Option<f64>,
    pub auto_net_profit: Option<f64>,
    pub member_pct: f64,
    pub reserve_pct: f64,
    pub management_pct: f64,
    pub supervisor_pct: f64,
    pub social_pct: f64,
    pub education_pct: f64,
    pub notes: Option<String>,
    pub calculated_at: Option<DateTime<Utc>>,
    pub finalized_at: Option<DateTime<Utc>>,
    pub finalized_by: Option<i64>,
    pub cancelled_at: Option<DateTime<Utc>>,
    pub cancelled_by: Option<i64>,
    pub cancel_reason: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Satu bucket alokasi: kunci English stabil untuk i18n frontend (Fase 11).
#[derive(Debug, Serialize)]
pub struct BucketAllocation {
    pub key: &'static str,
    pub percentage: f64,
    pub amount: f64,
}

#[derive(Debug, Serialize)]
pub struct ProfitSharingPeriodDetailResponse {
    #[serde(flatten)]
    pub period: ProfitSharingPeriodResponse,
    /// Pool yang dipakai kalkulasi terakhir (`total_pool` ?? `auto_net_profit`).
    pub pool_used: f64,
    pub member_pool: f64,
    pub member_count: i64,
    pub total_member_transaction: f64,
    pub allocations: Vec<BucketAllocation>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct ProfitSharingDistributionResponse {
    pub id: i64,
    pub profit_sharing_period_id: i64,
    pub member_id: i64,
    pub member_no: String,
    pub full_name: String,
    pub phone: String,
    pub transaction_amount: f64,
    pub share_pct: f64,
    pub share_amount: f64,
    pub calculated_at: DateTime<Utc>,
}
