use sqlx::{FromRow, PgPool, Postgres, Transaction};

use crate::{
    auth::AuthUser,
    error::{AppError, AppResult},
    models::{
        CreateProfitSharingPeriodRequest, DistributionQuery, ListQuery, PaginationMeta,
        ProfitSharingDistributionResponse, ProfitSharingPeriodResponse,
    },
    services::{audit, parse_status_filter},
};

const PERIOD_SELECT: &str = r#"
    SELECT
        id,
        period_name,
        start_date,
        end_date,
        status::text AS status,
        total_pool::float8 AS total_pool,
        auto_net_profit::float8 AS auto_net_profit,
        member_pct::float8 AS member_pct,
        reserve_pct::float8 AS reserve_pct,
        management_pct::float8 AS management_pct,
        supervisor_pct::float8 AS supervisor_pct,
        social_pct::float8 AS social_pct,
        education_pct::float8 AS education_pct,
        notes,
        calculated_at,
        finalized_at,
        finalized_by,
        cancelled_at,
        cancelled_by,
        cancel_reason,
        created_at,
        updated_at
    FROM profit_sharing_periods
"#;

const PERIOD_STATUSES: &[&str] = &["draft", "calculated", "finalized", "cancelled"];

const DISTRIBUTION_SELECT: &str = r#"
    SELECT
        d.id,
        d.profit_sharing_period_id,
        d.member_id,
        m.member_no,
        m.full_name,
        m.phone,
        d.transaction_amount::float8 AS transaction_amount,
        d.share_pct::float8 AS share_pct,
        d.share_amount::float8 AS share_amount,
        d.calculated_at
    FROM profit_sharing_distributions d
    JOIN members m ON m.id = d.member_id
"#;

/// Net per anggota aktif: penjualan completed (sold_at) − retur completed
/// (returned_at) selama periode. Retur teratribusi via sale pemilik.
#[derive(Debug, FromRow)]
pub struct MemberNetRow {
    pub member_id: i64,
    pub gross_sales: f64,
    pub returns_amount: f64,
}

pub async fn list_periods(
    pool: &PgPool,
    query: &ListQuery,
) -> AppResult<(Vec<ProfitSharingPeriodResponse>, PaginationMeta)> {
    let search = query.search_pattern();
    let limit = query.limit();
    let offset = query.offset();
    let statuses = parse_status_filter(
        query.status.as_deref(),
        PERIOD_STATUSES,
        "profit sharing period",
    )?;

    let sql = format!(
        r#"{PERIOD_SELECT}
        WHERE ($1 = '%%' OR lower(period_name) LIKE $1)
            AND (COALESCE(array_length($4::text[], 1), 0) = 0 OR status::text = ANY($4::text[]))
        ORDER BY start_date DESC, id DESC
        LIMIT $2 OFFSET $3"#
    );
    let rows = sqlx::query_as::<_, ProfitSharingPeriodResponse>(&sql)
        .bind(&search)
        .bind(limit)
        .bind(offset)
        .bind(&statuses)
        .fetch_all(pool)
        .await?;

    let total: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*)
        FROM profit_sharing_periods
        WHERE ($1 = '%%' OR lower(period_name) LIKE $1)
            AND (COALESCE(array_length($2::text[], 1), 0) = 0 OR status::text = ANY($2::text[]))
        "#,
    )
    .bind(&search)
    .bind(&statuses)
    .fetch_one(pool)
    .await?;

    Ok((rows, PaginationMeta::new(limit, offset, total)))
}

pub async fn find_period(pool: &PgPool, id: i64) -> AppResult<Option<ProfitSharingPeriodResponse>> {
    let row =
        sqlx::query_as::<_, ProfitSharingPeriodResponse>(&format!("{PERIOD_SELECT} WHERE id = $1"))
            .bind(id)
            .fetch_optional(pool)
            .await?;

    Ok(row)
}

pub async fn create_period(
    pool: &PgPool,
    user: &AuthUser,
    request: CreateProfitSharingPeriodRequest,
) -> AppResult<ProfitSharingPeriodResponse> {
    let id: i64 = sqlx::query_scalar(
        r#"
        INSERT INTO profit_sharing_periods (
            period_name, start_date, end_date, total_pool,
            member_pct, reserve_pct, management_pct, supervisor_pct, social_pct, education_pct,
            notes
        )
        VALUES ($1, $2, $3, $4::numeric, $5::numeric, $6::numeric, $7::numeric, $8::numeric, $9::numeric, $10::numeric, $11)
        RETURNING id
        "#,
    )
    .bind(request.period_name.trim())
    .bind(request.start_date)
    .bind(request.end_date)
    .bind(request.total_pool)
    .bind(request.member_pct)
    .bind(request.reserve_pct)
    .bind(request.management_pct)
    .bind(request.supervisor_pct)
    .bind(request.social_pct)
    .bind(request.education_pct)
    .bind(request.notes.map(|value| value.trim().to_string()).filter(|value| !value.is_empty()))
    .fetch_one(pool)
    .await?;

    audit::record_audit_log_best_effort(
        pool,
        audit::AuditLogRecord::success(
            Some(user.id),
            "profit_sharing.create",
            "profit_sharing_period",
            Some(id),
        ),
    )
    .await;

    find_period(pool, id).await?.ok_or(AppError::NotFound)
}

#[allow(clippy::too_many_arguments)]
pub async fn update_period(
    pool: &PgPool,
    user: &AuthUser,
    id: i64,
    period_name: Option<String>,
    start_date: Option<chrono::NaiveDate>,
    end_date: Option<chrono::NaiveDate>,
    clear_pool: bool,
    total_pool: Option<f64>,
    member_pct: Option<f64>,
    reserve_pct: Option<f64>,
    management_pct: Option<f64>,
    supervisor_pct: Option<f64>,
    social_pct: Option<f64>,
    education_pct: Option<f64>,
    notes: Option<String>,
) -> AppResult<ProfitSharingPeriodResponse> {
    let updated: Option<i64> = sqlx::query_scalar(
        r#"
        UPDATE profit_sharing_periods
        SET
            period_name = COALESCE($2, period_name),
            start_date = COALESCE($3, start_date),
            end_date = COALESCE($4, end_date),
            total_pool = CASE WHEN $5 THEN NULL ELSE COALESCE($6::numeric, total_pool) END,
            member_pct = COALESCE($7::numeric, member_pct),
            reserve_pct = COALESCE($8::numeric, reserve_pct),
            management_pct = COALESCE($9::numeric, management_pct),
            supervisor_pct = COALESCE($10::numeric, supervisor_pct),
            social_pct = COALESCE($11::numeric, social_pct),
            education_pct = COALESCE($12::numeric, education_pct),
            notes = COALESCE($13, notes)
        WHERE id = $1
        RETURNING id
        "#,
    )
    .bind(id)
    .bind(period_name.map(|value| value.trim().to_string()))
    .bind(start_date)
    .bind(end_date)
    .bind(clear_pool)
    .bind(total_pool)
    .bind(member_pct)
    .bind(reserve_pct)
    .bind(management_pct)
    .bind(supervisor_pct)
    .bind(social_pct)
    .bind(education_pct)
    .bind(
        notes
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty()),
    )
    .fetch_optional(pool)
    .await?;

    match updated {
        Some(id) => {
            audit::record_audit_log_best_effort(
                pool,
                audit::AuditLogRecord::success(
                    Some(user.id),
                    "profit_sharing.update",
                    "profit_sharing_period",
                    Some(id),
                ),
            )
            .await;
            find_period(pool, id).await?.ok_or(AppError::NotFound)
        }
        None => Err(AppError::NotFound),
    }
}

pub async fn lock_period(
    tx: &mut Transaction<'_, Postgres>,
    id: i64,
) -> AppResult<Option<ProfitSharingPeriodResponse>> {
    let row = sqlx::query_as::<_, ProfitSharingPeriodResponse>(&format!(
        "{PERIOD_SELECT} WHERE id = $1 FOR UPDATE"
    ))
    .bind(id)
    .fetch_optional(&mut **tx)
    .await?;

    Ok(row)
}

pub async fn member_nets(
    pool: &PgPool,
    start: chrono::NaiveDate,
    end: chrono::NaiveDate,
) -> AppResult<Vec<MemberNetRow>> {
    let rows = sqlx::query_as::<_, MemberNetRow>(
        r#"
        SELECT
            m.id AS member_id,
            COALESCE(s.gross, 0)::float8 AS gross_sales,
            COALESCE(r.ret, 0)::float8 AS returns_amount
        FROM members m
        LEFT JOIN (
            SELECT member_id, SUM(total_amount) AS gross
            FROM sales
            WHERE status = 'completed'
                AND member_id IS NOT NULL
                AND sold_at::date >= $1
                AND sold_at::date <= $2
            GROUP BY member_id
        ) s ON s.member_id = m.id
        LEFT JOIN (
            SELECT s.member_id, SUM(sr.subtotal_amount) AS ret
            FROM sale_returns sr
            JOIN sales s ON s.id = sr.sale_id
            WHERE sr.status = 'completed'
                AND s.member_id IS NOT NULL
                AND sr.returned_at::date >= $1
                AND sr.returned_at::date <= $2
            GROUP BY s.member_id
        ) r ON r.member_id = m.id
        WHERE m.is_active = TRUE
        "#,
    )
    .bind(start)
    .bind(end)
    .fetch_all(pool)
    .await?;

    Ok(rows)
}

pub async fn replace_distributions(
    tx: &mut Transaction<'_, Postgres>,
    period_id: i64,
    shares: &[NewDistribution],
) -> AppResult<()> {
    sqlx::query("DELETE FROM profit_sharing_distributions WHERE profit_sharing_period_id = $1")
        .bind(period_id)
        .execute(&mut **tx)
        .await?;

    for share in shares {
        sqlx::query(
            r#"
            INSERT INTO profit_sharing_distributions (
                profit_sharing_period_id, member_id,
                transaction_amount, share_pct, share_amount
            )
            VALUES ($1, $2, $3::numeric, $4::numeric, $5::numeric)
            "#,
        )
        .bind(period_id)
        .bind(share.member_id)
        .bind(share.transaction_amount)
        .bind(share.share_pct)
        .bind(share.share_amount)
        .execute(&mut **tx)
        .await?;
    }

    Ok(())
}

pub struct NewDistribution {
    pub member_id: i64,
    pub transaction_amount: f64,
    pub share_pct: f64,
    pub share_amount: f64,
}

pub async fn mark_calculated(
    tx: &mut Transaction<'_, Postgres>,
    id: i64,
    auto_net_profit: f64,
    from_draft: bool,
) -> AppResult<()> {
    let affected = if from_draft {
        sqlx::query(
            r#"
            UPDATE profit_sharing_periods
            SET status = 'calculated', auto_net_profit = $2::numeric, calculated_at = now()
            WHERE id = $1
            "#,
        )
        .bind(id)
        .bind(auto_net_profit)
        .execute(&mut **tx)
        .await?
    } else {
        sqlx::query(
            r#"
            UPDATE profit_sharing_periods
            SET auto_net_profit = $2::numeric, calculated_at = now()
            WHERE id = $1
            "#,
        )
        .bind(id)
        .bind(auto_net_profit)
        .execute(&mut **tx)
        .await?
    };

    if affected.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }

    Ok(())
}

pub async fn mark_finalized(
    tx: &mut Transaction<'_, Postgres>,
    id: i64,
    user_id: i64,
) -> AppResult<()> {
    let affected = sqlx::query(
        r#"
        UPDATE profit_sharing_periods
        SET status = 'finalized', finalized_at = now(), finalized_by = $2
        WHERE id = $1
        "#,
    )
    .bind(id)
    .bind(user_id)
    .execute(&mut **tx)
    .await?;

    if affected.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }

    Ok(())
}

pub async fn mark_cancelled(
    tx: &mut Transaction<'_, Postgres>,
    id: i64,
    user_id: i64,
    reason: &str,
) -> AppResult<()> {
    let affected = sqlx::query(
        r#"
        UPDATE profit_sharing_periods
        SET status = 'cancelled', cancelled_at = now(),
            cancelled_by = $2, cancel_reason = $3
        WHERE id = $1
        "#,
    )
    .bind(id)
    .bind(user_id)
    .bind(reason)
    .execute(&mut **tx)
    .await?;

    if affected.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }

    Ok(())
}

pub async fn distribution_summary(pool: &PgPool, period_id: i64) -> AppResult<(i64, f64)> {
    let row: (i64, Option<f64>) = sqlx::query_as(
        r#"
        SELECT COUNT(*), SUM(transaction_amount)::float8
        FROM profit_sharing_distributions
        WHERE profit_sharing_period_id = $1
        "#,
    )
    .bind(period_id)
    .fetch_one(pool)
    .await?;

    Ok((row.0, row.1.unwrap_or(0.0)))
}

pub async fn list_distributions(
    pool: &PgPool,
    period_id: i64,
    query: &DistributionQuery,
) -> AppResult<(Vec<ProfitSharingDistributionResponse>, PaginationMeta)> {
    let search = query.search_pattern();
    let limit = query.limit();
    let offset = query.offset();

    let sql = format!(
        r#"{DISTRIBUTION_SELECT}
        WHERE d.profit_sharing_period_id = $1
            AND ($2 = '%%'
                OR lower(m.full_name) LIKE $2
                OR lower(m.phone) LIKE $2
                OR lower(m.member_no) LIKE $2)
        ORDER BY d.share_amount DESC, m.full_name, m.id
        LIMIT $3 OFFSET $4"#
    );
    let rows = sqlx::query_as::<_, ProfitSharingDistributionResponse>(&sql)
        .bind(period_id)
        .bind(&search)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?;

    let total: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*)
        FROM profit_sharing_distributions d
        JOIN members m ON m.id = d.member_id
        WHERE d.profit_sharing_period_id = $1
            AND ($2 = '%%'
                OR lower(m.full_name) LIKE $2
                OR lower(m.phone) LIKE $2
                OR lower(m.member_no) LIKE $2)
        "#,
    )
    .bind(period_id)
    .bind(&search)
    .fetch_one(pool)
    .await?;

    Ok((rows, PaginationMeta::new(limit, offset, total)))
}

pub async fn export_distributions(
    pool: &PgPool,
    period_id: i64,
) -> AppResult<Vec<ProfitSharingDistributionResponse>> {
    let sql = format!(
        r#"{DISTRIBUTION_SELECT}
        WHERE d.profit_sharing_period_id = $1
        ORDER BY d.share_amount DESC, m.full_name, m.id"#
    );

    sqlx::query_as::<_, ProfitSharingDistributionResponse>(&sql)
        .bind(period_id)
        .fetch_all(pool)
        .await
        .map_err(Into::into)
}

pub async fn record_action_audit(
    pool: &PgPool,
    user: &AuthUser,
    action: &str,
    id: i64,
    metadata: serde_json::Value,
) {
    audit::record_audit_log_best_effort(
        pool,
        audit::AuditLogRecord::success(Some(user.id), action, "profit_sharing_period", Some(id))
            .with_metadata(metadata),
    )
    .await;
}
