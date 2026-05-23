use sqlx::{PgPool, Row};

use crate::{
    error::{AppError, AppResult},
    models::{
        CreateRoleRequest, ListQuery, PermissionResponse, RoleResponse,
        UpdateRolePermissionsRequest, UpdateRoleRequest,
    },
};

use super::validate_required;

const ROLE_SELECT: &str = r#"
    SELECT
        r.id,
        r.code,
        r.name,
        r.description,
        r.is_system,
        ARRAY(
            SELECT p.code
            FROM role_permissions rp
            JOIN permissions p ON p.id = rp.permission_id
            WHERE rp.role_id = r.id
            ORDER BY p.code
        ) AS permissions,
        r.created_at,
        r.updated_at,
        r.deleted_at
    FROM roles r
"#;

const PERMISSION_SELECT: &str = r#"
    SELECT id, code, name, description, created_at, updated_at
    FROM permissions
"#;

pub async fn user_has_any_permission(
    pool: &PgPool,
    user_id: i64,
    permission_codes: &[&str],
) -> AppResult<bool> {
    if permission_codes.is_empty() {
        return Ok(false);
    }

    let has_permission = sqlx::query_scalar::<_, bool>(
        r#"
        SELECT EXISTS (
            SELECT 1
            FROM users u
            JOIN roles r ON r.id = u.role_id
            LEFT JOIN role_permissions rp ON rp.role_id = r.id
            LEFT JOIN permissions p ON p.id = rp.permission_id
            WHERE u.id = $1
                AND u.deleted_at IS NULL
                AND u.is_active = TRUE
                AND r.deleted_at IS NULL
                AND (r.code = 'admin' OR p.code = ANY($2::text[]))
        )
        "#,
    )
    .bind(user_id)
    .bind(permission_codes)
    .fetch_one(pool)
    .await?;

    Ok(has_permission)
}

pub async fn list_roles(pool: &PgPool, query: ListQuery) -> AppResult<Vec<RoleResponse>> {
    let search = query.search_pattern();
    let sql = format!(
        "{ROLE_SELECT}
        WHERE r.deleted_at IS NULL
            AND ($1 = '%%' OR lower(r.code) LIKE $1 OR lower(r.name) LIKE $1)
        ORDER BY r.is_system DESC, r.name
        LIMIT $2 OFFSET $3"
    );

    sqlx::query_as::<_, RoleResponse>(&sql)
        .bind(search)
        .bind(query.limit())
        .bind(query.offset())
        .fetch_all(pool)
        .await
        .map_err(Into::into)
}

pub async fn find_role(pool: &PgPool, id: i64) -> AppResult<RoleResponse> {
    let sql = format!("{ROLE_SELECT} WHERE r.id = $1 AND r.deleted_at IS NULL");

    sqlx::query_as::<_, RoleResponse>(&sql)
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or(AppError::NotFound)
}

pub async fn create_role(pool: &PgPool, request: CreateRoleRequest) -> AppResult<RoleResponse> {
    validate_required(&request.name, "name")?;
    let code = normalize_code(&request.code, "code")?;
    let permission_codes =
        normalize_permission_codes(request.permission_codes.unwrap_or_default())?;

    let mut tx = pool.begin().await?;
    let id: i64 = sqlx::query_scalar(
        r#"
        INSERT INTO roles (code, name, description, is_system)
        VALUES ($1, $2, $3, FALSE)
        RETURNING id
        "#,
    )
    .bind(&code)
    .bind(request.name.trim())
    .bind(empty_to_none(request.description))
    .fetch_one(&mut *tx)
    .await?;

    replace_role_permissions_in_tx(&mut tx, id, &permission_codes).await?;
    tx.commit().await?;

    find_role(pool, id).await
}

pub async fn update_role(
    pool: &PgPool,
    id: i64,
    request: UpdateRoleRequest,
) -> AppResult<RoleResponse> {
    if let Some(name) = &request.name {
        validate_required(name, "name")?;
    }

    let current = find_role(pool, id).await?;
    let code = match request.code {
        Some(code) => {
            if current.is_system && code != current.code {
                return Err(AppError::Validation(
                    "system role code cannot be changed".to_string(),
                ));
            }
            Some(normalize_code(&code, "code")?)
        }
        None => None,
    };

    let updated_id: Option<i64> = sqlx::query_scalar(
        r#"
        UPDATE roles
        SET
            code = COALESCE($2, code),
            name = COALESCE($3, name),
            description = COALESCE($4, description)
        WHERE id = $1 AND deleted_at IS NULL
        RETURNING id
        "#,
    )
    .bind(id)
    .bind(code)
    .bind(request.name.map(|value| value.trim().to_string()))
    .bind(empty_to_none(request.description))
    .fetch_optional(pool)
    .await?;

    match updated_id {
        Some(id) => find_role(pool, id).await,
        None => Err(AppError::NotFound),
    }
}

pub async fn delete_role(pool: &PgPool, id: i64) -> AppResult<()> {
    let role = find_role(pool, id).await?;
    if role.is_system {
        return Err(AppError::Validation(
            "system roles cannot be deleted".to_string(),
        ));
    }

    let assigned_user_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM users WHERE role_id = $1 AND deleted_at IS NULL")
            .bind(id)
            .fetch_one(pool)
            .await?;

    if assigned_user_count > 0 {
        return Err(AppError::Conflict(
            "role cannot be deleted while assigned to users".to_string(),
        ));
    }

    sqlx::query("UPDATE roles SET deleted_at = now() WHERE id = $1 AND deleted_at IS NULL")
        .bind(id)
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn list_permissions(
    pool: &PgPool,
    query: ListQuery,
) -> AppResult<Vec<PermissionResponse>> {
    let search = query.search_pattern();
    let sql = format!(
        "{PERMISSION_SELECT}
        WHERE $1 = '%%' OR lower(code) LIKE $1 OR lower(name) LIKE $1
        ORDER BY code
        LIMIT $2 OFFSET $3"
    );

    sqlx::query_as::<_, PermissionResponse>(&sql)
        .bind(search)
        .bind(query.limit())
        .bind(query.offset())
        .fetch_all(pool)
        .await
        .map_err(Into::into)
}

pub async fn replace_role_permissions(
    pool: &PgPool,
    role_id: i64,
    request: UpdateRolePermissionsRequest,
) -> AppResult<RoleResponse> {
    let permission_codes = normalize_permission_codes(request.permission_codes)?;
    find_role(pool, role_id).await?;

    let mut tx = pool.begin().await?;
    replace_role_permissions_in_tx(&mut tx, role_id, &permission_codes).await?;
    tx.commit().await?;

    find_role(pool, role_id).await
}

pub async fn resolve_role(
    pool: &PgPool,
    role_id: Option<i64>,
    role_code: Option<String>,
    default_role_code: &str,
) -> AppResult<(i64, String)> {
    if let Some(role_id) = role_id {
        let role = find_role(pool, role_id).await?;
        return Ok((role.id, role.code));
    }

    let code = role_code.unwrap_or_else(|| default_role_code.to_string());
    let normalized_code = normalize_code(&code, "role")?;
    let row = sqlx::query(
        r#"
        SELECT id, code
        FROM roles
        WHERE lower(code) = lower($1) AND deleted_at IS NULL
        "#,
    )
    .bind(normalized_code)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::Validation("role must be selected".to_string()))?;

    Ok((row.get("id"), row.get("code")))
}

async fn replace_role_permissions_in_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    role_id: i64,
    permission_codes: &[String],
) -> AppResult<()> {
    let permission_ids = load_permission_ids(tx, permission_codes).await?;

    sqlx::query("DELETE FROM role_permissions WHERE role_id = $1")
        .bind(role_id)
        .execute(&mut **tx)
        .await?;

    for permission_id in permission_ids {
        sqlx::query(
            r#"
            INSERT INTO role_permissions (role_id, permission_id)
            VALUES ($1, $2)
            ON CONFLICT DO NOTHING
            "#,
        )
        .bind(role_id)
        .bind(permission_id)
        .execute(&mut **tx)
        .await?;
    }

    Ok(())
}

async fn load_permission_ids(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    permission_codes: &[String],
) -> AppResult<Vec<i64>> {
    if permission_codes.is_empty() {
        return Ok(Vec::new());
    }

    let rows = sqlx::query(
        r#"
        SELECT id, code
        FROM permissions
        WHERE code = ANY($1::text[])
        ORDER BY code
        "#,
    )
    .bind(permission_codes)
    .fetch_all(&mut **tx)
    .await?;

    if rows.len() != permission_codes.len() {
        return Err(AppError::Validation(
            "permission_codes contains unknown permission".to_string(),
        ));
    }

    Ok(rows.into_iter().map(|row| row.get("id")).collect())
}

fn normalize_permission_codes(values: Vec<String>) -> AppResult<Vec<String>> {
    let mut codes = Vec::new();

    for value in values {
        let code = normalize_code(&value, "permission")?;
        if !codes.contains(&code) {
            codes.push(code);
        }
    }

    Ok(codes)
}

fn normalize_code(value: &str, label: &str) -> AppResult<String> {
    let code = value.trim().to_lowercase();
    validate_required(&code, label)?;

    let is_valid = code.chars().all(|character| {
        character.is_ascii_lowercase() || character.is_ascii_digit() || character == '_'
    });
    if !is_valid {
        return Err(AppError::Validation(format!(
            "{label} can only contain lowercase letters, numbers, and underscores"
        )));
    }

    Ok(code)
}

fn empty_to_none(value: Option<String>) -> Option<String> {
    value.and_then(|value| {
        let trimmed = value.trim().to_string();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed)
        }
    })
}
