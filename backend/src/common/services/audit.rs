use serde_json::Value;
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::{
    common::request_context,
    error::{AppError, AppResult},
    models::{AuditLogQuery, AuditLogResponse},
};

const AUDIT_LOG_SELECT: &str = r#"
    SELECT
        id,
        request_id,
        actor_id,
        actor_name,
        actor_email,
        actor_role,
        action,
        resource_type,
        resource_id,
        resource_label,
        outcome,
        before_data,
        after_data,
        metadata,
        ip_address::text AS ip_address,
        user_agent,
        occurred_at
    FROM audit_logs
"#;

#[derive(Debug, Clone)]
pub struct AuditLogRecord {
    pub actor_id: Option<i64>,
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
}

impl AuditLogRecord {
    pub fn success(
        actor_id: Option<i64>,
        action: impl Into<String>,
        resource_type: impl Into<String>,
        resource_id: Option<i64>,
    ) -> Self {
        Self {
            actor_id,
            action: action.into(),
            resource_type: resource_type.into(),
            resource_id,
            resource_label: None,
            outcome: "success".to_string(),
            before_data: None,
            after_data: None,
            metadata: None,
            ip_address: None,
            user_agent: None,
        }
    }

    pub fn with_resource_label(mut self, resource_label: impl Into<String>) -> Self {
        self.resource_label = Some(resource_label.into());
        self
    }

    pub fn with_before_data(mut self, before_data: Value) -> Self {
        self.before_data = Some(before_data);
        self
    }

    pub fn with_after_data(mut self, after_data: Value) -> Self {
        self.after_data = Some(after_data);
        self
    }

    pub fn with_metadata(mut self, metadata: Value) -> Self {
        self.metadata = Some(metadata);
        self
    }
}

pub async fn list_audit_logs(
    pool: &PgPool,
    query: AuditLogQuery,
) -> AppResult<Vec<AuditLogResponse>> {
    let search = query.search_pattern();
    let actions = normalize_csv_filter(query.action.as_deref());
    let resource_types = normalize_csv_filter(query.resource_type.as_deref());
    let outcomes = normalize_csv_filter(query.outcome.as_deref());
    validate_outcomes(&outcomes)?;
    let request_id = query.request_id.as_deref().unwrap_or("").trim().to_string();

    let sql = format!(
        "{AUDIT_LOG_SELECT}
        WHERE ($1 = '%%'
                OR lower(action) LIKE $1
                OR lower(resource_type) LIKE $1
                OR lower(COALESCE(resource_label, '')) LIKE $1
                OR lower(COALESCE(actor_name, '')) LIKE $1
                OR lower(COALESCE(actor_email, '')) LIKE $1)
            AND ($2::date IS NULL OR occurred_at::date >= $2)
            AND ($3::date IS NULL OR occurred_at::date <= $3)
            AND (COALESCE(array_length($4::text[], 1), 0) = 0 OR action = ANY($4::text[]))
            AND (COALESCE(array_length($5::text[], 1), 0) = 0 OR resource_type = ANY($5::text[]))
            AND (COALESCE(array_length($6::text[], 1), 0) = 0 OR outcome = ANY($6::text[]))
            AND ($7::bigint IS NULL OR actor_id = $7)
            AND ($8 = '' OR request_id::text = $8)
        ORDER BY occurred_at DESC, id DESC
        LIMIT $9 OFFSET $10"
    );

    sqlx::query_as::<_, AuditLogResponse>(&sql)
        .bind(search)
        .bind(query.from)
        .bind(query.to)
        .bind(actions)
        .bind(resource_types)
        .bind(outcomes)
        .bind(query.actor_id)
        .bind(request_id)
        .bind(query.limit())
        .bind(query.offset())
        .fetch_all(pool)
        .await
        .map_err(Into::into)
}

pub async fn find_audit_log(pool: &PgPool, id: i64) -> AppResult<AuditLogResponse> {
    let sql = format!("{AUDIT_LOG_SELECT} WHERE id = $1");

    sqlx::query_as::<_, AuditLogResponse>(&sql)
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or(AppError::NotFound)
}

pub async fn record_audit_log(pool: &PgPool, record: AuditLogRecord) -> AppResult<()> {
    validate_required(&record.action, "action")?;
    validate_required(&record.resource_type, "resource_type")?;
    validate_outcome(&record.outcome)?;
    validate_json_object(record.before_data.as_ref(), "before_data")?;
    validate_json_object(record.after_data.as_ref(), "after_data")?;
    validate_json_object(record.metadata.as_ref(), "metadata")?;

    let actor = match record.actor_id {
        Some(actor_id) => load_actor_snapshot(pool, actor_id).await?,
        None => None,
    };
    let request_id = Uuid::parse_str(&request_context::current_request_id()).ok();

    sqlx::query(
        r#"
        INSERT INTO audit_logs (
            request_id, actor_id, actor_name, actor_email, actor_role, action,
            resource_type, resource_id, resource_label, outcome, before_data,
            after_data, metadata, ip_address, user_agent
        )
        VALUES (
            $1, $2, $3, $4, $5, $6, $7, $8, $9, $10,
            $11, $12, $13, $14::inet, $15
        )
        "#,
    )
    .bind(request_id)
    .bind(record.actor_id)
    .bind(actor.as_ref().map(|actor| actor.name.as_str()))
    .bind(actor.as_ref().map(|actor| actor.email.as_str()))
    .bind(actor.as_ref().map(|actor| actor.role.as_str()))
    .bind(record.action.trim())
    .bind(record.resource_type.trim())
    .bind(record.resource_id)
    .bind(record.resource_label.as_deref())
    .bind(record.outcome.trim())
    .bind(record.before_data)
    .bind(record.after_data)
    .bind(record.metadata)
    .bind(record.ip_address)
    .bind(record.user_agent)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn record_audit_log_best_effort(pool: &PgPool, record: AuditLogRecord) {
    if let Err(error) = record_audit_log(pool, record).await {
        tracing::warn!(%error, "failed to record audit log");
    }
}

#[derive(Debug)]
struct ActorSnapshot {
    name: String,
    email: String,
    role: String,
}

async fn load_actor_snapshot(pool: &PgPool, actor_id: i64) -> AppResult<Option<ActorSnapshot>> {
    let row = sqlx::query(
        r#"
        SELECT u.name, u.email, r.code AS role
        FROM users u
        JOIN roles r ON r.id = u.role_id
        WHERE u.id = $1
        "#,
    )
    .bind(actor_id)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|row| ActorSnapshot {
        name: row.get("name"),
        email: row.get("email"),
        role: row.get("role"),
    }))
}

fn normalize_csv_filter(value: Option<&str>) -> Vec<String> {
    value
        .unwrap_or("")
        .split(',')
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .collect()
}

fn validate_outcomes(outcomes: &[String]) -> AppResult<()> {
    for outcome in outcomes {
        validate_outcome(outcome)?;
    }
    Ok(())
}

fn validate_outcome(outcome: &str) -> AppResult<()> {
    if matches!(outcome, "success" | "failure") {
        Ok(())
    } else {
        Err(AppError::Validation(
            "outcome contains an invalid status".to_string(),
        ))
    }
}

fn validate_required(value: &str, field: &str) -> AppResult<()> {
    if value.trim().is_empty() {
        Err(AppError::Validation(format!("{field} is required")))
    } else {
        Ok(())
    }
}

fn validate_json_object(value: Option<&Value>, field: &str) -> AppResult<()> {
    if value.is_some_and(|value| !value.is_object()) {
        Err(AppError::Validation(format!(
            "{field} must be a JSON object"
        )))
    } else {
        Ok(())
    }
}
