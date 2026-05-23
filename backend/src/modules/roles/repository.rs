use sqlx::PgPool;

use crate::{
    error::AppResult,
    models::{
        CreateRoleRequest, ListQuery, PermissionResponse, RoleResponse,
        UpdateRolePermissionsRequest, UpdateRoleRequest,
    },
    services::rbac,
};

pub async fn list_roles(pool: &PgPool, query: ListQuery) -> AppResult<Vec<RoleResponse>> {
    rbac::list_roles(pool, query).await
}

pub async fn find_role(pool: &PgPool, id: i64) -> AppResult<RoleResponse> {
    rbac::find_role(pool, id).await
}

pub async fn create_role(pool: &PgPool, request: CreateRoleRequest) -> AppResult<RoleResponse> {
    rbac::create_role(pool, request).await
}

pub async fn update_role(
    pool: &PgPool,
    id: i64,
    request: UpdateRoleRequest,
) -> AppResult<RoleResponse> {
    rbac::update_role(pool, id, request).await
}

pub async fn delete_role(pool: &PgPool, id: i64) -> AppResult<()> {
    rbac::delete_role(pool, id).await
}

pub async fn list_permissions(
    pool: &PgPool,
    query: ListQuery,
) -> AppResult<Vec<PermissionResponse>> {
    rbac::list_permissions(pool, query).await
}

pub async fn replace_role_permissions(
    pool: &PgPool,
    role_id: i64,
    request: UpdateRolePermissionsRequest,
) -> AppResult<RoleResponse> {
    rbac::replace_role_permissions(pool, role_id, request).await
}
