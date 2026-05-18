use sqlx::PgPool;

use crate::{
    error::{AppError, AppResult},
    models::{
        CreateEmployeeRequest, CreateProductCategoryRequest, CreateProductRequest,
        CreateSupplierRequest, CreateUserRequest, EmployeeResponse, ListQuery,
        ProductCategoryResponse, ProductResponse, SupplierResponse, UpdateEmployeeRequest,
        UpdateProductCategoryRequest, UpdateProductRequest, UpdateSupplierRequest,
        UpdateUserRequest, UserResponse,
    },
    security,
};

use super::{validate_non_negative, validate_required, validate_status};

const USER_SELECT: &str = r#"
    SELECT
        id,
        name,
        email,
        role::text AS role,
        is_active,
        last_login_at,
        created_at,
        updated_at,
        deleted_at
    FROM users
"#;

const CATEGORY_SELECT: &str = r#"
    SELECT id, name, description, is_active, created_at, updated_at, deleted_at
    FROM product_categories
"#;

const PRODUCT_SELECT: &str = r#"
    SELECT
        p.id,
        p.category_id,
        c.name AS category_name,
        p.name,
        p.sku,
        p.purchase_price::float8 AS purchase_price,
        p.selling_price::float8 AS selling_price,
        p.stock::float8 AS stock,
        p.minimum_stock::float8 AS minimum_stock,
        p.unit,
        p.is_active,
        p.created_at,
        p.updated_at,
        p.deleted_at
    FROM products p
    LEFT JOIN product_categories c ON c.id = p.category_id
"#;

const SUPPLIER_SELECT: &str = r#"
    SELECT id, name, contact_name, phone, email, address, is_active, created_at, updated_at, deleted_at
    FROM suppliers
"#;

const EMPLOYEE_SELECT: &str = r#"
    SELECT
        id,
        name,
        position,
        phone,
        address,
        base_salary::float8 AS base_salary,
        is_active,
        created_at,
        updated_at,
        deleted_at
    FROM employees
"#;

pub async fn list_users(pool: &PgPool, query: ListQuery) -> AppResult<Vec<UserResponse>> {
    let search = query.search_pattern();
    let sql = format!(
        "{USER_SELECT}
        WHERE deleted_at IS NULL
            AND ($1 = '%%' OR lower(name) LIKE $1 OR lower(email) LIKE $1)
        ORDER BY id DESC
        LIMIT $2 OFFSET $3"
    );

    sqlx::query_as::<_, UserResponse>(&sql)
        .bind(search)
        .bind(query.limit())
        .bind(query.offset())
        .fetch_all(pool)
        .await
        .map_err(Into::into)
}

pub async fn find_user(pool: &PgPool, id: i64) -> AppResult<UserResponse> {
    let sql = format!("{USER_SELECT} WHERE id = $1 AND deleted_at IS NULL");

    sqlx::query_as::<_, UserResponse>(&sql)
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or(AppError::NotFound)
}

pub async fn create_user(pool: &PgPool, request: CreateUserRequest) -> AppResult<UserResponse> {
    validate_required(&request.name, "name")?;
    validate_required(&request.email, "email")?;
    let role = request.role.unwrap_or_else(|| "cashier".to_string());
    validate_status(&role, &["admin", "cashier"], "role")?;
    let password_hash = security::hash_password(&request.password)?;
    let is_active = request.is_active.unwrap_or(true);

    let id: i64 = sqlx::query_scalar(
        r#"
        INSERT INTO users (name, email, password_hash, role, is_active)
        VALUES ($1, $2, $3, $4::user_role, $5)
        RETURNING id
        "#,
    )
    .bind(request.name.trim())
    .bind(request.email.trim())
    .bind(password_hash)
    .bind(role)
    .bind(is_active)
    .fetch_one(pool)
    .await?;

    find_user(pool, id).await
}

pub async fn update_user(
    pool: &PgPool,
    id: i64,
    request: UpdateUserRequest,
) -> AppResult<UserResponse> {
    if let Some(role) = &request.role {
        validate_status(role, &["admin", "cashier"], "role")?;
    }
    if let Some(name) = &request.name {
        validate_required(name, "name")?;
    }
    if let Some(email) = &request.email {
        validate_required(email, "email")?;
    }

    let password_hash = match request.password {
        Some(password) => Some(security::hash_password(&password)?),
        None => None,
    };

    let updated_id: Option<i64> = sqlx::query_scalar(
        r#"
        UPDATE users
        SET
            name = COALESCE($2, name),
            email = COALESCE($3, email),
            password_hash = COALESCE($4, password_hash),
            role = COALESCE($5::user_role, role),
            is_active = COALESCE($6, is_active)
        WHERE id = $1 AND deleted_at IS NULL
        RETURNING id
        "#,
    )
    .bind(id)
    .bind(request.name)
    .bind(request.email)
    .bind(password_hash)
    .bind(request.role)
    .bind(request.is_active)
    .fetch_optional(pool)
    .await?;

    match updated_id {
        Some(id) => find_user(pool, id).await,
        None => Err(AppError::NotFound),
    }
}

pub async fn delete_user(pool: &PgPool, id: i64) -> AppResult<()> {
    soft_delete(pool, "users", id).await
}

pub async fn list_categories(
    pool: &PgPool,
    query: ListQuery,
) -> AppResult<Vec<ProductCategoryResponse>> {
    let search = query.search_pattern();
    let sql = format!(
        "{CATEGORY_SELECT}
        WHERE deleted_at IS NULL
            AND ($1 = '%%' OR lower(name) LIKE $1)
        ORDER BY name
        LIMIT $2 OFFSET $3"
    );

    sqlx::query_as::<_, ProductCategoryResponse>(&sql)
        .bind(search)
        .bind(query.limit())
        .bind(query.offset())
        .fetch_all(pool)
        .await
        .map_err(Into::into)
}

pub async fn find_category(pool: &PgPool, id: i64) -> AppResult<ProductCategoryResponse> {
    let sql = format!("{CATEGORY_SELECT} WHERE id = $1 AND deleted_at IS NULL");

    sqlx::query_as::<_, ProductCategoryResponse>(&sql)
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or(AppError::NotFound)
}

pub async fn create_category(
    pool: &PgPool,
    request: CreateProductCategoryRequest,
) -> AppResult<ProductCategoryResponse> {
    validate_required(&request.name, "name")?;
    let id: i64 = sqlx::query_scalar(
        r#"
        INSERT INTO product_categories (name, description, is_active)
        VALUES ($1, $2, $3)
        RETURNING id
        "#,
    )
    .bind(request.name.trim())
    .bind(request.description)
    .bind(request.is_active.unwrap_or(true))
    .fetch_one(pool)
    .await?;

    find_category(pool, id).await
}

pub async fn update_category(
    pool: &PgPool,
    id: i64,
    request: UpdateProductCategoryRequest,
) -> AppResult<ProductCategoryResponse> {
    if let Some(name) = &request.name {
        validate_required(name, "name")?;
    }

    let updated_id: Option<i64> = sqlx::query_scalar(
        r#"
        UPDATE product_categories
        SET
            name = COALESCE($2, name),
            description = COALESCE($3, description),
            is_active = COALESCE($4, is_active)
        WHERE id = $1 AND deleted_at IS NULL
        RETURNING id
        "#,
    )
    .bind(id)
    .bind(request.name)
    .bind(request.description)
    .bind(request.is_active)
    .fetch_optional(pool)
    .await?;

    match updated_id {
        Some(id) => find_category(pool, id).await,
        None => Err(AppError::NotFound),
    }
}

pub async fn delete_category(pool: &PgPool, id: i64) -> AppResult<()> {
    soft_delete(pool, "product_categories", id).await
}

pub async fn list_products(pool: &PgPool, query: ListQuery) -> AppResult<Vec<ProductResponse>> {
    let search = query.search_pattern();
    let sql = format!(
        "{PRODUCT_SELECT}
        WHERE p.deleted_at IS NULL
            AND ($1 = '%%' OR lower(p.name) LIKE $1 OR lower(p.sku) LIKE $1)
        ORDER BY p.name
        LIMIT $2 OFFSET $3"
    );

    sqlx::query_as::<_, ProductResponse>(&sql)
        .bind(search)
        .bind(query.limit())
        .bind(query.offset())
        .fetch_all(pool)
        .await
        .map_err(Into::into)
}

pub async fn find_product(pool: &PgPool, id: i64) -> AppResult<ProductResponse> {
    let sql = format!("{PRODUCT_SELECT} WHERE p.id = $1 AND p.deleted_at IS NULL");

    sqlx::query_as::<_, ProductResponse>(&sql)
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or(AppError::NotFound)
}

pub async fn create_product(
    pool: &PgPool,
    request: CreateProductRequest,
) -> AppResult<ProductResponse> {
    validate_required(&request.name, "name")?;
    validate_required(&request.sku, "sku")?;
    validate_non_negative(request.purchase_price, "purchase_price")?;
    validate_non_negative(request.selling_price, "selling_price")?;
    validate_non_negative(request.stock.unwrap_or(0.0), "stock")?;
    validate_non_negative(request.minimum_stock.unwrap_or(0.0), "minimum_stock")?;

    let id: i64 = sqlx::query_scalar(
        r#"
        INSERT INTO products (
            category_id, name, sku, purchase_price, selling_price, stock,
            minimum_stock, unit, is_active
        )
        VALUES ($1, $2, $3, $4::numeric, $5::numeric, $6::numeric, $7::numeric, $8, $9)
        RETURNING id
        "#,
    )
    .bind(request.category_id)
    .bind(request.name.trim())
    .bind(request.sku.trim())
    .bind(request.purchase_price)
    .bind(request.selling_price)
    .bind(request.stock.unwrap_or(0.0))
    .bind(request.minimum_stock.unwrap_or(0.0))
    .bind(request.unit.unwrap_or_else(|| "pcs".to_string()))
    .bind(request.is_active.unwrap_or(true))
    .fetch_one(pool)
    .await?;

    find_product(pool, id).await
}

pub async fn update_product(
    pool: &PgPool,
    id: i64,
    request: UpdateProductRequest,
) -> AppResult<ProductResponse> {
    if let Some(name) = &request.name {
        validate_required(name, "name")?;
    }
    if let Some(sku) = &request.sku {
        validate_required(sku, "sku")?;
    }
    if let Some(value) = request.purchase_price {
        validate_non_negative(value, "purchase_price")?;
    }
    if let Some(value) = request.selling_price {
        validate_non_negative(value, "selling_price")?;
    }
    if let Some(value) = request.minimum_stock {
        validate_non_negative(value, "minimum_stock")?;
    }

    let updated_id: Option<i64> = sqlx::query_scalar(
        r#"
        UPDATE products
        SET
            category_id = COALESCE($2, category_id),
            name = COALESCE($3, name),
            sku = COALESCE($4, sku),
            purchase_price = COALESCE($5::numeric, purchase_price),
            selling_price = COALESCE($6::numeric, selling_price),
            minimum_stock = COALESCE($7::numeric, minimum_stock),
            unit = COALESCE($8, unit),
            is_active = COALESCE($9, is_active)
        WHERE id = $1 AND deleted_at IS NULL
        RETURNING id
        "#,
    )
    .bind(id)
    .bind(request.category_id)
    .bind(request.name)
    .bind(request.sku)
    .bind(request.purchase_price)
    .bind(request.selling_price)
    .bind(request.minimum_stock)
    .bind(request.unit)
    .bind(request.is_active)
    .fetch_optional(pool)
    .await?;

    match updated_id {
        Some(id) => find_product(pool, id).await,
        None => Err(AppError::NotFound),
    }
}

pub async fn delete_product(pool: &PgPool, id: i64) -> AppResult<()> {
    soft_delete(pool, "products", id).await
}

pub async fn list_suppliers(pool: &PgPool, query: ListQuery) -> AppResult<Vec<SupplierResponse>> {
    let search = query.search_pattern();
    let sql = format!(
        "{SUPPLIER_SELECT}
        WHERE deleted_at IS NULL
            AND ($1 = '%%' OR lower(name) LIKE $1 OR lower(coalesce(contact_name, '')) LIKE $1)
        ORDER BY name
        LIMIT $2 OFFSET $3"
    );

    sqlx::query_as::<_, SupplierResponse>(&sql)
        .bind(search)
        .bind(query.limit())
        .bind(query.offset())
        .fetch_all(pool)
        .await
        .map_err(Into::into)
}

pub async fn find_supplier(pool: &PgPool, id: i64) -> AppResult<SupplierResponse> {
    let sql = format!("{SUPPLIER_SELECT} WHERE id = $1 AND deleted_at IS NULL");

    sqlx::query_as::<_, SupplierResponse>(&sql)
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or(AppError::NotFound)
}

pub async fn create_supplier(
    pool: &PgPool,
    request: CreateSupplierRequest,
) -> AppResult<SupplierResponse> {
    validate_required(&request.name, "name")?;
    let id: i64 = sqlx::query_scalar(
        r#"
        INSERT INTO suppliers (name, contact_name, phone, email, address, is_active)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING id
        "#,
    )
    .bind(request.name.trim())
    .bind(request.contact_name)
    .bind(request.phone)
    .bind(request.email)
    .bind(request.address)
    .bind(request.is_active.unwrap_or(true))
    .fetch_one(pool)
    .await?;

    find_supplier(pool, id).await
}

pub async fn update_supplier(
    pool: &PgPool,
    id: i64,
    request: UpdateSupplierRequest,
) -> AppResult<SupplierResponse> {
    if let Some(name) = &request.name {
        validate_required(name, "name")?;
    }

    let updated_id: Option<i64> = sqlx::query_scalar(
        r#"
        UPDATE suppliers
        SET
            name = COALESCE($2, name),
            contact_name = COALESCE($3, contact_name),
            phone = COALESCE($4, phone),
            email = COALESCE($5, email),
            address = COALESCE($6, address),
            is_active = COALESCE($7, is_active)
        WHERE id = $1 AND deleted_at IS NULL
        RETURNING id
        "#,
    )
    .bind(id)
    .bind(request.name)
    .bind(request.contact_name)
    .bind(request.phone)
    .bind(request.email)
    .bind(request.address)
    .bind(request.is_active)
    .fetch_optional(pool)
    .await?;

    match updated_id {
        Some(id) => find_supplier(pool, id).await,
        None => Err(AppError::NotFound),
    }
}

pub async fn delete_supplier(pool: &PgPool, id: i64) -> AppResult<()> {
    soft_delete(pool, "suppliers", id).await
}

pub async fn list_employees(pool: &PgPool, query: ListQuery) -> AppResult<Vec<EmployeeResponse>> {
    let search = query.search_pattern();
    let sql = format!(
        "{EMPLOYEE_SELECT}
        WHERE deleted_at IS NULL
            AND ($1 = '%%' OR lower(name) LIKE $1 OR lower(position) LIKE $1)
        ORDER BY name
        LIMIT $2 OFFSET $3"
    );

    sqlx::query_as::<_, EmployeeResponse>(&sql)
        .bind(search)
        .bind(query.limit())
        .bind(query.offset())
        .fetch_all(pool)
        .await
        .map_err(Into::into)
}

pub async fn find_employee(pool: &PgPool, id: i64) -> AppResult<EmployeeResponse> {
    let sql = format!("{EMPLOYEE_SELECT} WHERE id = $1 AND deleted_at IS NULL");

    sqlx::query_as::<_, EmployeeResponse>(&sql)
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or(AppError::NotFound)
}

pub async fn create_employee(
    pool: &PgPool,
    request: CreateEmployeeRequest,
) -> AppResult<EmployeeResponse> {
    validate_required(&request.name, "name")?;
    validate_required(&request.position, "position")?;
    validate_non_negative(request.base_salary, "base_salary")?;

    let id: i64 = sqlx::query_scalar(
        r#"
        INSERT INTO employees (name, position, phone, address, base_salary, is_active)
        VALUES ($1, $2, $3, $4, $5::numeric, $6)
        RETURNING id
        "#,
    )
    .bind(request.name.trim())
    .bind(request.position.trim())
    .bind(request.phone)
    .bind(request.address)
    .bind(request.base_salary)
    .bind(request.is_active.unwrap_or(true))
    .fetch_one(pool)
    .await?;

    find_employee(pool, id).await
}

pub async fn update_employee(
    pool: &PgPool,
    id: i64,
    request: UpdateEmployeeRequest,
) -> AppResult<EmployeeResponse> {
    if let Some(name) = &request.name {
        validate_required(name, "name")?;
    }
    if let Some(position) = &request.position {
        validate_required(position, "position")?;
    }
    if let Some(base_salary) = request.base_salary {
        validate_non_negative(base_salary, "base_salary")?;
    }

    let updated_id: Option<i64> = sqlx::query_scalar(
        r#"
        UPDATE employees
        SET
            name = COALESCE($2, name),
            position = COALESCE($3, position),
            phone = COALESCE($4, phone),
            address = COALESCE($5, address),
            base_salary = COALESCE($6::numeric, base_salary),
            is_active = COALESCE($7, is_active)
        WHERE id = $1 AND deleted_at IS NULL
        RETURNING id
        "#,
    )
    .bind(id)
    .bind(request.name)
    .bind(request.position)
    .bind(request.phone)
    .bind(request.address)
    .bind(request.base_salary)
    .bind(request.is_active)
    .fetch_optional(pool)
    .await?;

    match updated_id {
        Some(id) => find_employee(pool, id).await,
        None => Err(AppError::NotFound),
    }
}

pub async fn delete_employee(pool: &PgPool, id: i64) -> AppResult<()> {
    soft_delete(pool, "employees", id).await
}

async fn soft_delete(pool: &PgPool, table: &str, id: i64) -> AppResult<()> {
    let sql = format!("UPDATE {table} SET deleted_at = now() WHERE id = $1 AND deleted_at IS NULL");
    let result = sqlx::query(&sql).bind(id).execute(pool).await?;

    if result.rows_affected() == 0 {
        Err(AppError::NotFound)
    } else {
        Ok(())
    }
}
