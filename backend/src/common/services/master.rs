use sqlx::PgPool;

use crate::{
    error::{AppError, AppResult},
    models::{
        CreateEmployeeRequest, CreateProductCategoryRequest, CreateProductRequest,
        CreateProductSkuRequest, CreateSupplierRequest, CreateUnitRequest, CreateUserRequest,
        EmployeeResponse, ListQuery, ProductCategoryResponse, ProductResponse, ProductSkuResponse,
        SupplierResponse, UnitResponse, UpdateEmployeeRequest, UpdateProductCategoryRequest,
        UpdateProductRequest, UpdateProductSkuRequest, UpdateSupplierRequest, UpdateUnitRequest,
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
        p.is_active,
        p.created_at,
        p.updated_at,
        p.deleted_at
    FROM products p
    LEFT JOIN product_categories c ON c.id = p.category_id
"#;

const UNIT_SELECT: &str = r#"
    SELECT id, code, name, allow_decimal, is_active, created_at, updated_at, deleted_at
    FROM units
"#;

const PRODUCT_SKU_SELECT: &str = r#"
    SELECT
        ps.id,
        ps.product_id,
        p.name AS product_name,
        p.category_id,
        c.name AS category_name,
        ps.sku,
        ps.barcode,
        ps.unit_id,
        u.code AS unit_code,
        u.name AS unit_name,
        ps.contents_qty::float8 AS contents_qty,
        ps.purchase_price::float8 AS purchase_price,
        ps.selling_price::float8 AS selling_price,
        ps.stock::float8 AS stock,
        ps.minimum_stock::float8 AS minimum_stock,
        ps.is_active,
        ps.created_at,
        ps.updated_at,
        ps.deleted_at
    FROM product_skus ps
    JOIN products p ON p.id = ps.product_id
    LEFT JOIN product_categories c ON c.id = p.category_id
    JOIN units u ON u.id = ps.unit_id
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
            AND ($1 = '%%' OR lower(p.name) LIKE $1)
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

    let id: i64 = sqlx::query_scalar(
        r#"
        INSERT INTO products (category_id, name, is_active)
        VALUES ($1, $2, $3)
        RETURNING id
        "#,
    )
    .bind(request.category_id)
    .bind(request.name.trim())
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

    let updated_id: Option<i64> = sqlx::query_scalar(
        r#"
        UPDATE products
        SET
            category_id = COALESCE($2, category_id),
            name = COALESCE($3, name),
            is_active = COALESCE($4, is_active)
        WHERE id = $1 AND deleted_at IS NULL
        RETURNING id
        "#,
    )
    .bind(id)
    .bind(request.category_id)
    .bind(request.name)
    .bind(request.is_active)
    .fetch_optional(pool)
    .await?;

    match updated_id {
        Some(id) => find_product(pool, id).await,
        None => Err(AppError::NotFound),
    }
}

pub async fn delete_product(pool: &PgPool, id: i64) -> AppResult<()> {
    let mut tx = pool.begin().await?;
    let result =
        sqlx::query("UPDATE products SET deleted_at = now() WHERE id = $1 AND deleted_at IS NULL")
            .bind(id)
            .execute(&mut *tx)
            .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }
    sqlx::query(
        "UPDATE product_skus SET deleted_at = now() WHERE product_id = $1 AND deleted_at IS NULL",
    )
    .bind(id)
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(())
}

pub async fn list_units(pool: &PgPool, query: ListQuery) -> AppResult<Vec<UnitResponse>> {
    let search = query.search_pattern();
    let sql = format!(
        "{UNIT_SELECT}
        WHERE deleted_at IS NULL
            AND ($1 = '%%' OR lower(code) LIKE $1 OR lower(name) LIKE $1)
        ORDER BY code
        LIMIT $2 OFFSET $3"
    );

    sqlx::query_as::<_, UnitResponse>(&sql)
        .bind(search)
        .bind(query.limit())
        .bind(query.offset())
        .fetch_all(pool)
        .await
        .map_err(Into::into)
}

pub async fn find_unit(pool: &PgPool, id: i64) -> AppResult<UnitResponse> {
    let sql = format!("{UNIT_SELECT} WHERE id = $1 AND deleted_at IS NULL");

    sqlx::query_as::<_, UnitResponse>(&sql)
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or(AppError::NotFound)
}

pub async fn create_unit(pool: &PgPool, request: CreateUnitRequest) -> AppResult<UnitResponse> {
    validate_required(&request.code, "code")?;
    validate_required(&request.name, "name")?;
    let id: i64 = sqlx::query_scalar(
        r#"
        INSERT INTO units (code, name, allow_decimal, is_active)
        VALUES ($1, $2, $3, $4)
        RETURNING id
        "#,
    )
    .bind(request.code.trim())
    .bind(request.name.trim())
    .bind(request.allow_decimal.unwrap_or(false))
    .bind(request.is_active.unwrap_or(true))
    .fetch_one(pool)
    .await?;

    find_unit(pool, id).await
}

pub async fn update_unit(
    pool: &PgPool,
    id: i64,
    request: UpdateUnitRequest,
) -> AppResult<UnitResponse> {
    if let Some(code) = &request.code {
        validate_required(code, "code")?;
    }
    if let Some(name) = &request.name {
        validate_required(name, "name")?;
    }

    let updated_id: Option<i64> = sqlx::query_scalar(
        r#"
        UPDATE units
        SET
            code = COALESCE($2, code),
            name = COALESCE($3, name),
            allow_decimal = COALESCE($4, allow_decimal),
            is_active = COALESCE($5, is_active)
        WHERE id = $1 AND deleted_at IS NULL
        RETURNING id
        "#,
    )
    .bind(id)
    .bind(request.code)
    .bind(request.name)
    .bind(request.allow_decimal)
    .bind(request.is_active)
    .fetch_optional(pool)
    .await?;

    match updated_id {
        Some(id) => find_unit(pool, id).await,
        None => Err(AppError::NotFound),
    }
}

pub async fn delete_unit(pool: &PgPool, id: i64) -> AppResult<()> {
    soft_delete(pool, "units", id).await
}

pub async fn list_product_skus(
    pool: &PgPool,
    query: ListQuery,
) -> AppResult<Vec<ProductSkuResponse>> {
    let search = query.search_pattern();
    let sql = format!(
        "{PRODUCT_SKU_SELECT}
        WHERE ps.deleted_at IS NULL
            AND p.deleted_at IS NULL
            AND (
                $1 = '%%'
                OR lower(p.name) LIKE $1
                OR lower(ps.sku) LIKE $1
                OR lower(coalesce(ps.barcode, '')) LIKE $1
                OR lower(u.code) LIKE $1
            )
        ORDER BY p.name, ps.sku
        LIMIT $2 OFFSET $3"
    );

    sqlx::query_as::<_, ProductSkuResponse>(&sql)
        .bind(search)
        .bind(query.limit())
        .bind(query.offset())
        .fetch_all(pool)
        .await
        .map_err(Into::into)
}

pub async fn find_product_sku(pool: &PgPool, id: i64) -> AppResult<ProductSkuResponse> {
    let sql = format!(
        "{PRODUCT_SKU_SELECT}
        WHERE ps.id = $1 AND ps.deleted_at IS NULL AND p.deleted_at IS NULL"
    );

    sqlx::query_as::<_, ProductSkuResponse>(&sql)
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or(AppError::NotFound)
}

pub async fn create_product_sku(
    pool: &PgPool,
    request: CreateProductSkuRequest,
) -> AppResult<ProductSkuResponse> {
    validate_required(&request.sku, "sku")?;
    validate_non_negative(request.purchase_price, "purchase_price")?;
    validate_non_negative(request.selling_price, "selling_price")?;
    validate_non_negative(request.stock.unwrap_or(0.0), "stock")?;
    validate_non_negative(request.minimum_stock.unwrap_or(0.0), "minimum_stock")?;
    let contents_qty = request.contents_qty.unwrap_or(1.0);
    if contents_qty <= 0.0 {
        return Err(AppError::Validation(
            "contents_qty must be greater than 0".to_string(),
        ));
    }

    let id: i64 = sqlx::query_scalar(
        r#"
        INSERT INTO product_skus (
            product_id, sku, barcode, unit_id, contents_qty, purchase_price,
            selling_price, stock, minimum_stock, is_active
        )
        SELECT $1, $2, $3, $4, $5::numeric, $6::numeric, $7::numeric, $8::numeric, $9::numeric, $10
        WHERE EXISTS (SELECT 1 FROM products WHERE id = $1 AND deleted_at IS NULL)
            AND EXISTS (SELECT 1 FROM units WHERE id = $4 AND deleted_at IS NULL AND is_active = TRUE)
        RETURNING id
        "#,
    )
    .bind(request.product_id)
    .bind(request.sku.trim())
    .bind(normalize_optional_string(request.barcode))
    .bind(request.unit_id)
    .bind(contents_qty)
    .bind(request.purchase_price)
    .bind(request.selling_price)
    .bind(request.stock.unwrap_or(0.0))
    .bind(request.minimum_stock.unwrap_or(0.0))
    .bind(request.is_active.unwrap_or(true))
    .fetch_optional(pool)
    .await?
    .ok_or(AppError::NotFound)?;

    find_product_sku(pool, id).await
}

pub async fn update_product_sku(
    pool: &PgPool,
    id: i64,
    request: UpdateProductSkuRequest,
) -> AppResult<ProductSkuResponse> {
    if let Some(sku) = &request.sku {
        validate_required(sku, "sku")?;
    }
    if let Some(value) = request.contents_qty {
        if value <= 0.0 {
            return Err(AppError::Validation(
                "contents_qty must be greater than 0".to_string(),
            ));
        }
    }
    if let Some(value) = request.purchase_price {
        validate_non_negative(value, "purchase_price")?;
    }
    if let Some(value) = request.selling_price {
        validate_non_negative(value, "selling_price")?;
    }
    if let Some(value) = request.stock {
        validate_non_negative(value, "stock")?;
    }
    if let Some(value) = request.minimum_stock {
        validate_non_negative(value, "minimum_stock")?;
    }

    let updated_id: Option<i64> = sqlx::query_scalar(
        r#"
        UPDATE product_skus
        SET
            sku = COALESCE($2, sku),
            barcode = COALESCE($3, barcode),
            unit_id = COALESCE($4, unit_id),
            contents_qty = COALESCE($5::numeric, contents_qty),
            purchase_price = COALESCE($6::numeric, purchase_price),
            selling_price = COALESCE($7::numeric, selling_price),
            stock = COALESCE($8::numeric, stock),
            minimum_stock = COALESCE($9::numeric, minimum_stock),
            is_active = COALESCE($10, is_active)
        WHERE id = $1
            AND deleted_at IS NULL
            AND ($4::bigint IS NULL OR EXISTS (
                SELECT 1 FROM units WHERE id = $4 AND deleted_at IS NULL AND is_active = TRUE
            ))
        RETURNING id
        "#,
    )
    .bind(id)
    .bind(request.sku)
    .bind(normalize_optional_string(request.barcode))
    .bind(request.unit_id)
    .bind(request.contents_qty)
    .bind(request.purchase_price)
    .bind(request.selling_price)
    .bind(request.stock)
    .bind(request.minimum_stock)
    .bind(request.is_active)
    .fetch_optional(pool)
    .await?;

    match updated_id {
        Some(id) => find_product_sku(pool, id).await,
        None => Err(AppError::NotFound),
    }
}

pub async fn delete_product_sku(pool: &PgPool, id: i64) -> AppResult<()> {
    soft_delete(pool, "product_skus", id).await
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

fn normalize_optional_string(value: Option<String>) -> Option<String> {
    value.and_then(|value| {
        let value = value.trim().to_string();
        if value.is_empty() { None } else { Some(value) }
    })
}
