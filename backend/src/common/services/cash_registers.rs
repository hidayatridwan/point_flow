use serde_json::json;
use sqlx::{PgPool, Row};

use crate::{
    auth::AuthUser,
    error::{AppError, AppResult},
    models::{
        CancelRequest, CashDrawerMovementQuery, CashDrawerMovementResponse, CashRegisterResponse,
        CashRegisterSessionResponse, CloseCashRegisterSessionRequest,
        CreateCashDrawerMovementRequest, CreateCashRegisterRequest, ListQuery,
        OpenCashRegisterSessionRequest, UpdateCashRegisterRequest,
    },
};

use super::{
    CASH_DRAWER_MOVEMENT_STATUSES, CASH_DRAWER_MOVEMENT_TYPES, CASH_REGISTER_SESSION_STATUSES,
    CASH_REGISTER_STATUSES, audit, parse_status_filter, validate_non_negative, validate_positive,
    validate_required, validate_status,
};

const CASH_REGISTER_SELECT: &str = r#"
    SELECT
        id,
        code,
        name,
        description,
        status::text AS status,
        created_at,
        updated_at,
        deleted_at
    FROM cash_registers
"#;

const CASH_REGISTER_SESSION_SELECT: &str = r#"
    SELECT
        crs.id,
        crs.session_no,
        crs.cash_register_id,
        cr.code AS cash_register_code,
        cr.name AS cash_register_name,
        crs.opened_at,
        crs.opened_by,
        opened_by_user.name AS opened_by_name,
        crs.opening_cash::float8 AS opening_cash,
        crs.opening_notes,
        crs.closed_at,
        crs.closed_by,
        closed_by_user.name AS closed_by_name,
        crs.closing_cash_counted::float8 AS closing_cash_counted,
        crs.expected_cash::float8 AS expected_cash,
        crs.cash_difference::float8 AS cash_difference,
        crs.closing_notes,
        crs.status::text AS status,
        crs.created_at,
        crs.updated_at
    FROM cash_register_sessions crs
    JOIN cash_registers cr ON cr.id = crs.cash_register_id
    JOIN users opened_by_user ON opened_by_user.id = crs.opened_by
    LEFT JOIN users closed_by_user ON closed_by_user.id = crs.closed_by
"#;

const CASH_DRAWER_MOVEMENT_SELECT: &str = r#"
    SELECT
        cdm.id,
        cdm.movement_no,
        cdm.cash_register_session_id,
        crs.session_no,
        crs.cash_register_id,
        cr.code AS cash_register_code,
        cr.name AS cash_register_name,
        cdm.occurred_at,
        cdm.created_by,
        created_by_user.name AS created_by_name,
        cdm.movement_type::text AS movement_type,
        cdm.direction::text AS direction,
        cdm.amount::float8 AS amount,
        cdm.reason,
        cdm.notes,
        cdm.status::text AS status,
        cdm.cancelled_at,
        cdm.cancelled_by,
        cancelled_by_user.name AS cancelled_by_name,
        cdm.cancel_reason,
        cdm.created_at,
        cdm.updated_at
    FROM cash_drawer_movements cdm
    JOIN cash_register_sessions crs ON crs.id = cdm.cash_register_session_id
    JOIN cash_registers cr ON cr.id = crs.cash_register_id
    JOIN users created_by_user ON created_by_user.id = cdm.created_by
    LEFT JOIN users cancelled_by_user ON cancelled_by_user.id = cdm.cancelled_by
"#;

pub async fn list_cash_registers(
    pool: &PgPool,
    query: ListQuery,
) -> AppResult<Vec<CashRegisterResponse>> {
    let search = query.search_pattern();
    let statuses = parse_status_filter(
        query.status.as_deref(),
        CASH_REGISTER_STATUSES,
        "cash register",
    )?;
    let sql = format!(
        "{CASH_REGISTER_SELECT}
        WHERE deleted_at IS NULL
            AND ($1 = '%%' OR lower(code) LIKE $1 OR lower(name) LIKE $1)
            AND (COALESCE(array_length($2::text[], 1), 0) = 0 OR status::text = ANY($2::text[]))
        ORDER BY id DESC
        LIMIT $3 OFFSET $4"
    );

    sqlx::query_as::<_, CashRegisterResponse>(&sql)
        .bind(search)
        .bind(statuses)
        .bind(query.limit())
        .bind(query.offset())
        .fetch_all(pool)
        .await
        .map_err(Into::into)
}

pub async fn find_cash_register(pool: &PgPool, id: i64) -> AppResult<CashRegisterResponse> {
    let sql = format!("{CASH_REGISTER_SELECT} WHERE id = $1 AND deleted_at IS NULL");

    sqlx::query_as::<_, CashRegisterResponse>(&sql)
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or(AppError::NotFound)
}

pub async fn create_cash_register(
    pool: &PgPool,
    request: CreateCashRegisterRequest,
) -> AppResult<CashRegisterResponse> {
    validate_required(&request.code, "code")?;
    validate_required(&request.name, "name")?;
    let status = request.status.unwrap_or_else(|| "active".to_string());
    validate_status(&status, CASH_REGISTER_STATUSES, "status")?;

    let id: i64 = sqlx::query_scalar(
        r#"
        INSERT INTO cash_registers (code, name, description, status)
        VALUES ($1, $2, $3, $4::cash_register_status)
        RETURNING id
        "#,
    )
    .bind(request.code.trim())
    .bind(request.name.trim())
    .bind(normalize_optional_string(request.description))
    .bind(status)
    .fetch_one(pool)
    .await?;

    find_cash_register(pool, id).await
}

pub async fn update_cash_register(
    pool: &PgPool,
    id: i64,
    request: UpdateCashRegisterRequest,
) -> AppResult<CashRegisterResponse> {
    if let Some(code) = &request.code {
        validate_required(code, "code")?;
    }
    if let Some(name) = &request.name {
        validate_required(name, "name")?;
    }
    if let Some(status) = &request.status {
        validate_status(status, CASH_REGISTER_STATUSES, "status")?;
    }

    let updated_id: Option<i64> = sqlx::query_scalar(
        r#"
        UPDATE cash_registers
        SET
            code = COALESCE($2, code),
            name = COALESCE($3, name),
            description = COALESCE($4, description),
            status = COALESCE($5::cash_register_status, status)
        WHERE id = $1 AND deleted_at IS NULL
        RETURNING id
        "#,
    )
    .bind(id)
    .bind(request.code.map(|code| code.trim().to_string()))
    .bind(request.name.map(|name| name.trim().to_string()))
    .bind(normalize_optional_string(request.description))
    .bind(request.status)
    .fetch_optional(pool)
    .await?;

    match updated_id {
        Some(id) => find_cash_register(pool, id).await,
        None => Err(AppError::NotFound),
    }
}

pub async fn delete_cash_register(pool: &PgPool, id: i64) -> AppResult<()> {
    let result = sqlx::query(
        r#"
        UPDATE cash_registers
        SET deleted_at = now()
        WHERE id = $1
            AND deleted_at IS NULL
        "#,
    )
    .bind(id)
    .execute(pool)
    .await?;

    if result.rows_affected() == 0 {
        Err(AppError::NotFound)
    } else {
        Ok(())
    }
}

pub async fn list_cash_register_sessions(
    pool: &PgPool,
    query: ListQuery,
) -> AppResult<Vec<CashRegisterSessionResponse>> {
    let search = query.search_pattern();
    let statuses = parse_status_filter(
        query.status.as_deref(),
        CASH_REGISTER_SESSION_STATUSES,
        "cash register session",
    )?;
    let sql = format!(
        "{CASH_REGISTER_SESSION_SELECT}
        WHERE ($1 = '%%'
                OR lower(crs.session_no) LIKE $1
                OR lower(cr.code) LIKE $1
                OR lower(cr.name) LIKE $1
                OR lower(opened_by_user.name) LIKE $1
                OR lower(COALESCE(closed_by_user.name, '')) LIKE $1)
            AND ($2::date IS NULL OR crs.opened_at::date >= $2)
            AND ($3::date IS NULL OR crs.opened_at::date <= $3)
            AND (COALESCE(array_length($4::text[], 1), 0) = 0 OR crs.status::text = ANY($4::text[]))
        ORDER BY crs.id DESC
        LIMIT $5 OFFSET $6"
    );

    sqlx::query_as::<_, CashRegisterSessionResponse>(&sql)
        .bind(search)
        .bind(query.from)
        .bind(query.to)
        .bind(statuses)
        .bind(query.limit())
        .bind(query.offset())
        .fetch_all(pool)
        .await
        .map_err(Into::into)
}

pub async fn find_cash_register_session(
    pool: &PgPool,
    id: i64,
) -> AppResult<CashRegisterSessionResponse> {
    let sql = format!("{CASH_REGISTER_SESSION_SELECT} WHERE crs.id = $1");

    sqlx::query_as::<_, CashRegisterSessionResponse>(&sql)
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or(AppError::NotFound)
}

pub async fn open_cash_register_session(
    pool: &PgPool,
    user: &AuthUser,
    request: OpenCashRegisterSessionRequest,
) -> AppResult<CashRegisterSessionResponse> {
    validate_non_negative(request.opening_cash, "opening_cash")?;

    let mut tx = pool.begin().await?;
    let register_exists = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT id
        FROM cash_registers
        WHERE id = $1
            AND status = 'active'
            AND deleted_at IS NULL
        FOR UPDATE
        "#,
    )
    .bind(request.cash_register_id)
    .fetch_optional(&mut *tx)
    .await?;

    if register_exists.is_none() {
        return Err(AppError::NotFound);
    }

    let open_session_id = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT id
        FROM cash_register_sessions
        WHERE cash_register_id = $1
            AND status = 'open'
        "#,
    )
    .bind(request.cash_register_id)
    .fetch_optional(&mut *tx)
    .await?;

    if open_session_id.is_some() {
        return Err(AppError::Conflict(
            "cash register already has an open session".to_string(),
        ));
    }

    let session_id: i64 = sqlx::query_scalar(
        r#"
        INSERT INTO cash_register_sessions (
            cash_register_id, opened_by, opening_cash, opening_notes
        )
        VALUES ($1, $2, $3::numeric, $4)
        RETURNING id
        "#,
    )
    .bind(request.cash_register_id)
    .bind(user.id)
    .bind(request.opening_cash)
    .bind(normalize_optional_string(request.opening_notes))
    .fetch_one(&mut *tx)
    .await?;

    tx.commit().await?;

    audit::record_audit_log_best_effort(
        pool,
        audit::AuditLogRecord::success(
            Some(user.id),
            "cash_register_session.open",
            "cash_register_session",
            Some(session_id),
        )
        .with_metadata(json!({
            "cash_register_id": request.cash_register_id,
            "opening_cash": request.opening_cash
        })),
    )
    .await;

    find_cash_register_session(pool, session_id).await
}

pub async fn close_cash_register_session(
    pool: &PgPool,
    user: &AuthUser,
    id: i64,
    request: CloseCashRegisterSessionRequest,
) -> AppResult<CashRegisterSessionResponse> {
    validate_non_negative(request.closing_cash_counted, "closing_cash_counted")?;

    let mut tx = pool.begin().await?;
    let row = sqlx::query(
        r#"
        SELECT status::text AS status
        FROM cash_register_sessions
        WHERE id = $1
        FOR UPDATE
        "#,
    )
    .bind(id)
    .fetch_optional(&mut *tx)
    .await?
    .ok_or(AppError::NotFound)?;

    let status: String = row.get("status");
    if status != "open" {
        return Err(AppError::Conflict(
            "cash register session is already closed".to_string(),
        ));
    }

    let expected_cash: f64 = sqlx::query_scalar(
        r#"
        SELECT (
            crs.opening_cash
            + COALESCE((
                SELECT SUM(s.total_amount)
                FROM sales s
                WHERE s.cash_register_session_id = crs.id
                    AND s.status = 'completed'
            ), 0)
            - COALESCE((
                SELECT SUM(sr.refund_amount)
                FROM sale_returns sr
                WHERE sr.cash_register_session_id = crs.id
                    AND sr.status = 'completed'
            ), 0)
            + COALESCE((
                SELECT SUM(cdm.amount)
                FROM cash_drawer_movements cdm
                WHERE cdm.cash_register_session_id = crs.id
                    AND cdm.status = 'completed'
                    AND cdm.direction = 'in'
            ), 0)
            - COALESCE((
                SELECT SUM(cdm.amount)
                FROM cash_drawer_movements cdm
                WHERE cdm.cash_register_session_id = crs.id
                    AND cdm.status = 'completed'
                    AND cdm.direction = 'out'
            ), 0)
        )::float8 AS expected_cash
        FROM cash_register_sessions crs
        WHERE crs.id = $1
        "#,
    )
    .bind(id)
    .fetch_one(&mut *tx)
    .await?;

    sqlx::query(
        r#"
        UPDATE cash_register_sessions
        SET
            closed_at = now(),
            closed_by = $2,
            closing_cash_counted = $3::numeric,
            expected_cash = $4::numeric,
            cash_difference = ($3::numeric - $4::numeric),
            closing_notes = $5,
            status = 'closed'
        WHERE id = $1
        "#,
    )
    .bind(id)
    .bind(user.id)
    .bind(request.closing_cash_counted)
    .bind(expected_cash)
    .bind(normalize_optional_string(request.closing_notes))
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    audit::record_audit_log_best_effort(
        pool,
        audit::AuditLogRecord::success(
            Some(user.id),
            "cash_register_session.close",
            "cash_register_session",
            Some(id),
        )
        .with_metadata(json!({
            "closing_cash_counted": request.closing_cash_counted,
            "expected_cash": expected_cash
        })),
    )
    .await;

    find_cash_register_session(pool, id).await
}

pub async fn list_cash_drawer_movements(
    pool: &PgPool,
    query: CashDrawerMovementQuery,
) -> AppResult<Vec<CashDrawerMovementResponse>> {
    let search = query.search_pattern();
    let statuses = parse_status_filter(
        query.status.as_deref(),
        CASH_DRAWER_MOVEMENT_STATUSES,
        "cash drawer movement",
    )?;
    let movement_types = parse_status_filter(
        query.movement_type.as_deref(),
        CASH_DRAWER_MOVEMENT_TYPES,
        "cash drawer movement type",
    )?;
    let sql = format!(
        "{CASH_DRAWER_MOVEMENT_SELECT}
        WHERE ($1 = '%%'
                OR lower(cdm.movement_no) LIKE $1
                OR lower(crs.session_no) LIKE $1
                OR lower(cr.code) LIKE $1
                OR lower(cr.name) LIKE $1
                OR lower(created_by_user.name) LIKE $1
                OR lower(COALESCE(cdm.reason, '')) LIKE $1
                OR lower(COALESCE(cdm.notes, '')) LIKE $1)
            AND ($2::date IS NULL OR cdm.occurred_at::date >= $2)
            AND ($3::date IS NULL OR cdm.occurred_at::date <= $3)
            AND ($4::bigint IS NULL OR cdm.cash_register_session_id = $4)
            AND (COALESCE(array_length($5::text[], 1), 0) = 0 OR cdm.status::text = ANY($5::text[]))
            AND (COALESCE(array_length($6::text[], 1), 0) = 0 OR cdm.movement_type::text = ANY($6::text[]))
        ORDER BY cdm.id DESC
        LIMIT $7 OFFSET $8"
    );

    sqlx::query_as::<_, CashDrawerMovementResponse>(&sql)
        .bind(search)
        .bind(query.from)
        .bind(query.to)
        .bind(query.cash_register_session_id)
        .bind(statuses)
        .bind(movement_types)
        .bind(query.limit())
        .bind(query.offset())
        .fetch_all(pool)
        .await
        .map_err(Into::into)
}

pub async fn find_cash_drawer_movement(
    pool: &PgPool,
    id: i64,
) -> AppResult<CashDrawerMovementResponse> {
    let sql = format!("{CASH_DRAWER_MOVEMENT_SELECT} WHERE cdm.id = $1");

    sqlx::query_as::<_, CashDrawerMovementResponse>(&sql)
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or(AppError::NotFound)
}

pub async fn create_cash_drawer_movement(
    pool: &PgPool,
    user: &AuthUser,
    request: CreateCashDrawerMovementRequest,
) -> AppResult<CashDrawerMovementResponse> {
    validate_status(
        &request.movement_type,
        CASH_DRAWER_MOVEMENT_TYPES,
        "movement_type",
    )?;
    validate_positive(request.amount, "amount")?;
    let direction = movement_direction(&request.movement_type);

    let mut tx = pool.begin().await?;
    ensure_open_cash_register_session(&mut tx, request.cash_register_session_id).await?;

    let id: i64 = sqlx::query_scalar(
        r#"
        INSERT INTO cash_drawer_movements (
            cash_register_session_id, occurred_at, created_by, movement_type, direction,
            amount, reason, notes
        )
        VALUES (
            $1, COALESCE($2, now()), $3, $4::cash_drawer_movement_type,
            $5::cash_drawer_movement_direction, $6::numeric, $7, $8
        )
        RETURNING id
        "#,
    )
    .bind(request.cash_register_session_id)
    .bind(request.occurred_at)
    .bind(user.id)
    .bind(request.movement_type.as_str())
    .bind(direction)
    .bind(request.amount)
    .bind(normalize_optional_string(request.reason))
    .bind(normalize_optional_string(request.notes))
    .fetch_one(&mut *tx)
    .await?;

    tx.commit().await?;

    audit::record_audit_log_best_effort(
        pool,
        audit::AuditLogRecord::success(
            Some(user.id),
            "cash_drawer_movement.create",
            "cash_drawer_movement",
            Some(id),
        )
        .with_metadata(json!({
            "cash_register_session_id": request.cash_register_session_id,
            "movement_type": request.movement_type,
            "direction": direction,
            "amount": request.amount
        })),
    )
    .await;

    find_cash_drawer_movement(pool, id).await
}

pub async fn cancel_cash_drawer_movement(
    pool: &PgPool,
    user: &AuthUser,
    id: i64,
    request: CancelRequest,
) -> AppResult<CashDrawerMovementResponse> {
    let cancel_reason = normalize_optional_string(request.reason);
    let mut tx = pool.begin().await?;
    let row = sqlx::query(
        r#"
        SELECT
            cdm.status::text AS status,
            crs.status::text AS cash_register_session_status
        FROM cash_drawer_movements cdm
        JOIN cash_register_sessions crs ON crs.id = cdm.cash_register_session_id
        WHERE cdm.id = $1
        FOR UPDATE
        "#,
    )
    .bind(id)
    .fetch_optional(&mut *tx)
    .await?
    .ok_or(AppError::NotFound)?;

    let status: String = row.get("status");
    if status != "completed" {
        return Err(AppError::Conflict(
            "cash drawer movement cannot be cancelled".to_string(),
        ));
    }

    let cash_register_session_status: String = row.get("cash_register_session_status");
    if cash_register_session_status != "open" {
        return Err(AppError::Conflict(
            "cash drawer movement cannot be cancelled after cash register session is closed"
                .to_string(),
        ));
    }

    sqlx::query(
        r#"
        UPDATE cash_drawer_movements
        SET status = 'cancelled',
            cancelled_at = now(),
            cancelled_by = $2,
            cancel_reason = $3
        WHERE id = $1
        "#,
    )
    .bind(id)
    .bind(user.id)
    .bind(cancel_reason.as_deref())
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    audit::record_audit_log_best_effort(
        pool,
        audit::AuditLogRecord::success(
            Some(user.id),
            "cash_drawer_movement.cancel",
            "cash_drawer_movement",
            Some(id),
        )
        .with_metadata(json!({ "reason": cancel_reason })),
    )
    .await;

    find_cash_drawer_movement(pool, id).await
}

pub async fn ensure_open_cash_register_session(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    session_id: i64,
) -> AppResult<()> {
    let exists = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT crs.id
        FROM cash_register_sessions crs
        JOIN cash_registers cr ON cr.id = crs.cash_register_id
        WHERE crs.id = $1
            AND crs.status = 'open'
            AND cr.status = 'active'
            AND cr.deleted_at IS NULL
        "#,
    )
    .bind(session_id)
    .fetch_optional(&mut **tx)
    .await?;

    if exists.is_none() {
        Err(AppError::Conflict(
            "cash_register_session_id must reference an open cash register session".to_string(),
        ))
    } else {
        Ok(())
    }
}

fn movement_direction(movement_type: &str) -> &'static str {
    if movement_type == "cash_in" {
        "in"
    } else {
        "out"
    }
}

fn normalize_optional_string(value: Option<String>) -> Option<String> {
    value.and_then(|value| {
        let value = value.trim().to_string();
        if value.is_empty() { None } else { Some(value) }
    })
}
