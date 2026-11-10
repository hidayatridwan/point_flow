use axum::{
    Json,
    extract::{Path, Query, State},
};

use crate::{app::AppState, auth::AuthUser, error::AppResult, models::ApiResponse};

use super::{
    dto::{
        CreateMidtransChargeRequest, GetPaymentQuery, MidtransChargeResponse,
        MidtransNotificationPayload, MidtransPaymentResponse,
    },
    service,
};

pub async fn create_charge(
    State(state): State<AppState>,
    user: AuthUser,
    Json(payload): Json<CreateMidtransChargeRequest>,
) -> AppResult<Json<ApiResponse<MidtransChargeResponse>>> {
    user.require_permission(&state.db, "point_flow").await?;
    let settings = service::require_settings(state.midtrans.as_ref())?;
    Ok(Json(ApiResponse::success(
        service::create_charge(&state.db, settings, &user, payload).await?,
    )))
}

pub async fn get_payment(
    State(state): State<AppState>,
    user: AuthUser,
    Path(order_id): Path<String>,
    Query(query): Query<GetPaymentQuery>,
) -> AppResult<Json<ApiResponse<MidtransPaymentResponse>>> {
    user.require_permission(&state.db, "point_flow").await?;
    Ok(Json(ApiResponse::success(
        service::get_payment(
            &state.db,
            state.midtrans.as_ref(),
            &order_id,
            query.refresh.unwrap_or(false),
        )
        .await?,
    )))
}

/// Webhook Midtrans: SENGAJA tanpa `AuthUser` (Midtrans tidak bisa mengirim
/// `Authorization` Bearer [redacted]). Keamanan berasal dari verifikasi
/// `signature_key` sebelum baris pembayaran disentuh.
pub async fn handle_notification(
    State(state): State<AppState>,
    Json(payload): Json<MidtransNotificationPayload>,
) -> AppResult<Json<ApiResponse<MidtransPaymentResponse>>> {
    let settings = service::require_settings(state.midtrans.as_ref())?;
    Ok(Json(ApiResponse::success(
        service::apply_notification(&state.db, settings, &payload).await?,
    )))
}
