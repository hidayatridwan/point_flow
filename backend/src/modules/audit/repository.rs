use sqlx::PgPool;

use crate::{
    error::AppResult,
    models::{AuditLogQuery, AuditLogResponse},
    services::audit,
};

pub async fn list_audit_logs(
    pool: &PgPool,
    query: AuditLogQuery,
) -> AppResult<Vec<AuditLogResponse>> {
    audit::list_audit_logs(pool, query).await
}

pub async fn find_audit_log(pool: &PgPool, id: i64) -> AppResult<AuditLogResponse> {
    audit::find_audit_log(pool, id).await
}
