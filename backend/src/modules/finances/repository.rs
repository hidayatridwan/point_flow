use serde_json::json;
use sqlx::PgPool;

use crate::{
    auth::AuthUser,
    error::{AppError, AppResult},
    models::{
        CancelRequest, CreateFinancialCategoryRequest, CreateFinancialTransactionRequest,
        FinancialCategoryResponse, FinancialTransactionQuery, FinancialTransactionResponse,
        ListQuery, UpdateFinancialCategoryRequest, UpdateFinancialTransactionRequest,
    },
    services::{audit, parse_status_filter, validate_positive, validate_required},
};

const FINANCIAL_TYPES: &[&str] = &["income", "expense"];
const FINANCIAL_STATUSES: &[&str] = &["completed", "cancelled"];

const FINANCIAL_CATEGORY_SELECT: &str = r#"
    SELECT
        id,
        name,
        type::text AS category_type,
        description,
        is_active,
        created_by,
        created_at,
        updated_at
    FROM financial_categories
"#;

const FINANCIAL_TRANSACTION_SELECT: &str = r#"
    SELECT
        ft.id,
        ft.transaction_no,
        ft.occurred_at,
        ft.type::text AS transaction_type,
        ft.financial_category_id,
        fc.name AS financial_category_name,
        ft.amount::float8 AS amount,
        ft.is_cash,
        ft.description,
        ft.status::text AS status,
        ft.cancelled_at,
        ft.cancelled_by,
        ft.cancel_reason,
        ft.recorded_by,
        ft.created_at,
        ft.updated_at
    FROM financial_transactions ft
    JOIN financial_categories fc ON fc.id = ft.financial_category_id
"#;

fn normalize_financial_type(value: &str) -> AppResult<String> {
    let normalized = value.trim().to_lowercase();
    if FINANCIAL_TYPES.contains(&normalized.as_str()) {
        Ok(normalized)
    } else {
        Err(AppError::Validation(
            "type must be 'income' or 'expense'".to_string(),
        ))
    }
}

fn normalize_optional_string(value: Option<String>) -> Option<String> {
    value.and_then(|value| {
        let value = value.trim().to_string();
        if value.is_empty() { None } else { Some(value) }
    })
}

fn map_unique_violation(error: sqlx::Error) -> AppError {
    match &error {
        sqlx::Error::Database(db_error) if db_error.code().as_deref() == Some("23505") => {
            AppError::Conflict("financial category already exists".to_string())
        }
        _ => AppError::Database(error),
    }
}

async fn category_type_by_id(pool: &PgPool, id: i64) -> AppResult<String> {
    sqlx::query_scalar::<_, String>("SELECT type::text FROM financial_categories WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or(AppError::NotFound)
}

pub async fn list_financial_categories(
    pool: &PgPool,
    query: ListQuery,
) -> AppResult<Vec<FinancialCategoryResponse>> {
    let search = query.search_pattern();
    let sql = format!(
        "{FINANCIAL_CATEGORY_SELECT}
        WHERE ($1 = '%%' OR lower(name) LIKE $1 OR lower(coalesce(description, '')) LIKE $1)
        ORDER BY name
        LIMIT $2 OFFSET $3"
    );

    sqlx::query_as::<_, FinancialCategoryResponse>(&sql)
        .bind(search)
        .bind(query.limit())
        .bind(query.offset())
        .fetch_all(pool)
        .await
        .map_err(Into::into)
}

pub async fn find_financial_category(
    pool: &PgPool,
    id: i64,
) -> AppResult<FinancialCategoryResponse> {
    let sql = format!("{FINANCIAL_CATEGORY_SELECT} WHERE id = $1");

    sqlx::query_as::<_, FinancialCategoryResponse>(&sql)
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or(AppError::NotFound)
}

pub async fn create_financial_category(
    pool: &PgPool,
    user: &AuthUser,
    request: CreateFinancialCategoryRequest,
) -> AppResult<FinancialCategoryResponse> {
    validate_required(&request.name, "name")?;
    let category_type = normalize_financial_type(&request.category_type)?;

    let name = request.name.trim().to_string();
    let duplicate: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM financial_categories WHERE name = $1 AND type = $2::financial_transaction_type)",
    )
    .bind(&name)
    .bind(&category_type)
    .fetch_one(pool)
    .await?;
    if duplicate {
        return Err(AppError::Conflict(
            "financial category already exists".to_string(),
        ));
    }

    let id: i64 = sqlx::query_scalar(
        r#"
        INSERT INTO financial_categories (name, type, description, is_active, created_by)
        VALUES ($1, $2::financial_transaction_type, $3, COALESCE($4, TRUE), $5)
        RETURNING id
        "#,
    )
    .bind(&name)
    .bind(&category_type)
    .bind(normalize_optional_string(request.description))
    .bind(request.is_active)
    .bind(user.id)
    .fetch_one(pool)
    .await
    .map_err(map_unique_violation)?;

    audit::record_audit_log_best_effort(
        pool,
        audit::AuditLogRecord::success(
            Some(user.id),
            "financial_category.create",
            "financial_category",
            Some(id),
        )
        .with_metadata(json!({ "name": name, "type": category_type })),
    )
    .await;

    find_financial_category(pool, id).await
}

pub async fn update_financial_category(
    pool: &PgPool,
    user: &AuthUser,
    id: i64,
    request: UpdateFinancialCategoryRequest,
) -> AppResult<FinancialCategoryResponse> {
    let current = find_financial_category(pool, id).await?;

    if let Some(name) = &request.name {
        validate_required(name, "name")?;
        let name = name.trim().to_string();
        let duplicate: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM financial_categories WHERE name = $1 AND type = $2::financial_transaction_type AND id <> $3)",
        )
        .bind(&name)
        .bind(&current.category_type)
        .bind(id)
        .fetch_one(pool)
        .await?;
        if duplicate {
            return Err(AppError::Conflict(
                "financial category already exists".to_string(),
            ));
        }
    }

    let updated_id: Option<i64> = sqlx::query_scalar(
        r#"
        UPDATE financial_categories
        SET
            name = COALESCE($2, name),
            description = COALESCE($3, description),
            is_active = COALESCE($4, is_active)
        WHERE id = $1
        RETURNING id
        "#,
    )
    .bind(id)
    .bind(request.name.map(|name| name.trim().to_string()))
    .bind(normalize_optional_string(request.description))
    .bind(request.is_active)
    .fetch_optional(pool)
    .await
    .map_err(map_unique_violation)?;

    match updated_id {
        Some(id) => {
            audit::record_audit_log_best_effort(
                pool,
                audit::AuditLogRecord::success(
                    Some(user.id),
                    "financial_category.update",
                    "financial_category",
                    Some(id),
                ),
            )
            .await;
            find_financial_category(pool, id).await
        }
        None => Err(AppError::NotFound),
    }
}

pub async fn delete_financial_category(pool: &PgPool, user: &AuthUser, id: i64) -> AppResult<()> {
    find_financial_category(pool, id).await?;

    let used: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM financial_transactions WHERE financial_category_id = $1)",
    )
    .bind(id)
    .fetch_one(pool)
    .await?;
    if used {
        return Err(AppError::Conflict(
            "financial category is used by transactions".to_string(),
        ));
    }

    let result = sqlx::query("DELETE FROM financial_categories WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }

    audit::record_audit_log_best_effort(
        pool,
        audit::AuditLogRecord::success(
            Some(user.id),
            "financial_category.delete",
            "financial_category",
            Some(id),
        ),
    )
    .await;

    Ok(())
}

pub async fn list_financial_transactions(
    pool: &PgPool,
    query: FinancialTransactionQuery,
) -> AppResult<Vec<FinancialTransactionResponse>> {
    let search = query.search_pattern();
    let statuses = parse_status_filter(
        query.status.as_deref(),
        FINANCIAL_STATUSES,
        "financial transaction",
    )?;
    let transaction_type = query
        .transaction_type
        .as_deref()
        .map(normalize_financial_type)
        .transpose()
        .map_err(|_| AppError::BadRequest("type must be 'income' or 'expense'".to_string()))?;
    let sql = format!(
        "{FINANCIAL_TRANSACTION_SELECT}
        WHERE ($1 = '%%'
            OR lower(ft.transaction_no) LIKE $1
            OR lower(coalesce(ft.description, '')) LIKE $1
            OR lower(fc.name) LIKE $1)
            AND ($2::date IS NULL OR ft.occurred_at::date >= $2)
            AND ($3::date IS NULL OR ft.occurred_at::date <= $3)
            AND ($4::text IS NULL OR ft.type::text = $4)
            AND ($5::bigint IS NULL OR ft.financial_category_id = $5)
            AND (COALESCE(array_length($6::text[], 1), 0) = 0 OR ft.status::text = ANY($6::text[]))
        ORDER BY ft.occurred_at DESC, ft.id DESC
        LIMIT $7 OFFSET $8"
    );

    sqlx::query_as::<_, FinancialTransactionResponse>(&sql)
        .bind(search)
        .bind(query.from)
        .bind(query.to)
        .bind(transaction_type)
        .bind(query.financial_category_id)
        .bind(statuses)
        .bind(query.limit())
        .bind(query.offset())
        .fetch_all(pool)
        .await
        .map_err(Into::into)
}

pub async fn find_financial_transaction(
    pool: &PgPool,
    id: i64,
) -> AppResult<FinancialTransactionResponse> {
    let sql = format!("{FINANCIAL_TRANSACTION_SELECT} WHERE ft.id = $1");

    sqlx::query_as::<_, FinancialTransactionResponse>(&sql)
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or(AppError::NotFound)
}

pub async fn create_financial_transaction(
    pool: &PgPool,
    user: &AuthUser,
    request: CreateFinancialTransactionRequest,
) -> AppResult<FinancialTransactionResponse> {
    let transaction_type = normalize_financial_type(&request.transaction_type)?;
    validate_positive(request.amount, "amount")?;

    let category_type = category_type_by_id(pool, request.financial_category_id).await?;
    if category_type != transaction_type {
        return Err(AppError::Validation(
            "financial category type must match transaction type".to_string(),
        ));
    }

    let id: i64 = sqlx::query_scalar(
        r#"
        INSERT INTO financial_transactions (
            occurred_at, type, financial_category_id, amount, is_cash, description, recorded_by
        )
        VALUES (
            COALESCE($1, now()), $2::financial_transaction_type, $3, $4::numeric,
            COALESCE($5, TRUE), $6, $7
        )
        RETURNING id
        "#,
    )
    .bind(request.occurred_at)
    .bind(&transaction_type)
    .bind(request.financial_category_id)
    .bind(request.amount)
    .bind(request.is_cash)
    .bind(normalize_optional_string(request.description))
    .bind(user.id)
    .fetch_one(pool)
    .await?;

    audit::record_audit_log_best_effort(
        pool,
        audit::AuditLogRecord::success(
            Some(user.id),
            "financial_transaction.create",
            "financial_transaction",
            Some(id),
        )
        .with_metadata(json!({
            "type": transaction_type,
            "financial_category_id": request.financial_category_id,
            "amount": request.amount,
        })),
    )
    .await;

    find_financial_transaction(pool, id).await
}

pub async fn update_financial_transaction(
    pool: &PgPool,
    user: &AuthUser,
    id: i64,
    request: UpdateFinancialTransactionRequest,
) -> AppResult<FinancialTransactionResponse> {
    let current = find_financial_transaction(pool, id).await?;
    if current.status != "completed" {
        return Err(AppError::Conflict(
            "financial transaction cannot be updated".to_string(),
        ));
    }

    if let Some(amount) = request.amount {
        validate_positive(amount, "amount")?;
    }
    if let Some(category_id) = request.financial_category_id {
        let category_type = category_type_by_id(pool, category_id).await?;
        if category_type != current.transaction_type {
            return Err(AppError::Validation(
                "financial category type must match transaction type".to_string(),
            ));
        }
    }

    let updated_id: Option<i64> = sqlx::query_scalar(
        r#"
        UPDATE financial_transactions
        SET
            occurred_at = COALESCE($2, occurred_at),
            financial_category_id = COALESCE($3, financial_category_id),
            amount = COALESCE($4::numeric, amount),
            is_cash = COALESCE($5, is_cash),
            description = COALESCE($6, description)
        WHERE id = $1 AND status = 'completed'
        RETURNING id
        "#,
    )
    .bind(id)
    .bind(request.occurred_at)
    .bind(request.financial_category_id)
    .bind(request.amount)
    .bind(request.is_cash)
    .bind(normalize_optional_string(request.description))
    .fetch_optional(pool)
    .await?;

    match updated_id {
        Some(id) => {
            audit::record_audit_log_best_effort(
                pool,
                audit::AuditLogRecord::success(
                    Some(user.id),
                    "financial_transaction.update",
                    "financial_transaction",
                    Some(id),
                ),
            )
            .await;
            find_financial_transaction(pool, id).await
        }
        None => Err(AppError::Conflict(
            "financial transaction cannot be updated".to_string(),
        )),
    }
}

pub async fn cancel_financial_transaction(
    pool: &PgPool,
    user: &AuthUser,
    id: i64,
    request: CancelRequest,
) -> AppResult<FinancialTransactionResponse> {
    let cancel_reason = normalize_optional_string(request.reason);
    let result = sqlx::query(
        r#"
        UPDATE financial_transactions
        SET status = 'cancelled', cancelled_at = now(), cancelled_by = $2, cancel_reason = $3
        WHERE id = $1 AND status = 'completed'
        "#,
    )
    .bind(id)
    .bind(user.id)
    .bind(cancel_reason.as_deref())
    .execute(pool)
    .await?;

    if result.rows_affected() == 0 {
        let current = find_financial_transaction(pool, id).await.ok();
        return match current {
            Some(transaction) if transaction.status == "cancelled" => Err(AppError::Conflict(
                "financial transaction is already cancelled".to_string(),
            )),
            Some(_) => Err(AppError::Conflict(
                "financial transaction cannot be cancelled".to_string(),
            )),
            None => Err(AppError::NotFound),
        };
    }

    audit::record_audit_log_best_effort(
        pool,
        audit::AuditLogRecord::success(
            Some(user.id),
            "financial_transaction.cancel",
            "financial_transaction",
            Some(id),
        )
        .with_metadata(json!({ "reason": cancel_reason })),
    )
    .await;

    find_financial_transaction(pool, id).await
}
