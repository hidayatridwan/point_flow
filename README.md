# POS Web System

POS Web System adalah aplikasi kasir berbasis web untuk toko kecil. Sistem ini mencakup autentikasi, master data produk, manajemen SKU/kemasan, purchase order, penerimaan barang, stock adjustment, stock conversion, transaksi penjualan, payroll slip, dashboard, dan report operasional.

Repository ini berisi backend API berbasis Rust dan frontend berbasis SvelteKit. Backend sudah menjadi sumber utama business logic dan database schema, sedangkan frontend berisi scaffold SvelteKit beserta komponen UI awal untuk pengembangan aplikasi kasir.

## Tech Stack

Backend:

- Rust 2024
- Axum 0.8 untuk HTTP routing
- Tokio async runtime
- SQLx dengan PostgreSQL
- Argon2 untuk password hashing
- JSON Web Token untuk access token
- HttpOnly cookie untuk refresh token
- Tower HTTP untuk CORS, tracing, dan compression
- Serde, Chrono, UUID, Validator, Tracing

Frontend:

- SvelteKit 2
- Svelte 5
- TypeScript
- Vite
- Tailwind CSS 4
- shadcn-svelte, Bits UI, Formsnap
- TanStack Table Core
- Lucide Svelte icons
- Bun sebagai package manager

Infrastructure:

- PostgreSQL 16
- Docker Compose untuk development services
- Postman collection untuk eksplorasi API

## System Overview

Backend menyediakan REST API di bawah prefix:

```text
/api/v1
```

Area utama sistem:

- Auth: bootstrap admin, login, refresh token, logout, current user.
- Users: akun admin dan cashier.
- Products: category, product induk, unit, dan product SKU.
- Suppliers: data supplier.
- Procurement: purchase order dan goods receipt.
- Inventory: stock adjustment dan stock conversion.
- Sales: transaksi kasir dan pembatalan transaksi.
- Employees dan Payroll: data karyawan dan slip gaji.
- Reports: dashboard, sales report, stock report, purchase order report, goods receipt report, stock adjustment report, payroll report.

Model inventory memakai `product_skus` sebagai titik utama stok. Perubahan stok final dicatat lewat dokumen operasional:

- Goods receipt posted menambah stock.
- Stock adjustment posted mengubah stock menjadi nilai hasil koreksi.
- Stock conversion posted mengurangi stock SKU sumber dan menambah stock SKU tujuan.
- Sale completed mengurangi stock.

## Project Structure

```text
.
├── backend/
│   ├── migrations/                 # SQL schema dan migration database
│   └── src/
│       ├── bin/server.rs           # entrypoint backend
│       ├── bootstrap/              # app bootstrap, CORS, tracing, server bind
│       ├── common/                 # auth extractor, models, services shared
│       ├── config/                 # environment config parser
│       ├── http/                   # root API router
│       ├── infra/                  # database connection
│       └── modules/                # auth, users, products, inventory, sales, dll
├── frontend/
│   ├── src/lib/components/         # komponen UI awal
│   └── src/routes/                 # routes SvelteKit
├── docker-compose.yml
├── .env.example
├── api-flow.md
└── pos-backend.postman_collection.json
```

## Environment

Salin template env untuk development lokal:

```bash
cp .env.example .env
```

Variable utama:

```text
APP_HOST=0.0.0.0
APP_PORT=8080
DATABASE_URL=postgres://pos:pos@localhost:5432/pos
JWT_SECRET=dev-only-change-this-secret-before-running-in-production
JWT_TTL_SECONDS=900
REFRESH_TOKEN_TTL_SECONDS=2592000
REFRESH_TOKEN_COOKIE_NAME=refresh_token
REFRESH_TOKEN_COOKIE_SECURE=false
REFRESH_TOKEN_COOKIE_SAME_SITE=Lax
```

Untuk local HTTP tanpa HTTPS, browser bisa menolak cookie `Secure`. Saat testing lokal dengan Postman biasanya aman, tetapi untuk browser development gunakan:

```text
REFRESH_TOKEN_COOKIE_SECURE=false
```

Untuk environment production HTTPS, set `REFRESH_TOKEN_COOKIE_SECURE=true`. Jangan gunakan `JWT_SECRET` default untuk environment production.

## Run With Docker Compose

Jalankan backend dan PostgreSQL:

```bash
docker compose up --build
```

Service yang terbuka:

```text
Backend API: http://localhost:8080/api/v1
PostgreSQL:  localhost:5432
```

Cek health endpoint:

```bash
curl http://localhost:8080/api/v1/health
```

Response:

```json
{
  "message": "ok"
}
```

Migration SQL di `backend/migrations` dipasang ke container PostgreSQL melalui `/docker-entrypoint-initdb.d`. File migration akan otomatis dijalankan saat database volume pertama kali dibuat.

## Run Backend Locally

Jalankan PostgreSQL lewat Docker:

```bash
docker compose up -d postgres
```

Jalankan backend dari root repository:

```bash
cargo run --manifest-path backend/Cargo.toml --bin server
```

Backend akan membaca `.env` dari root repository dan listen di `APP_HOST:APP_PORT`.

Command development backend:

```bash
cargo fmt --manifest-path backend/Cargo.toml
cargo test --manifest-path backend/Cargo.toml
cargo clippy --manifest-path backend/Cargo.toml --all-targets
```

## Run Frontend

Masuk ke folder frontend:

```bash
cd frontend
bun install
bun run dev
```

Default Vite development server:

```text
http://localhost:5173
```

Command frontend:

```bash
bun run check
bun run lint
bun run build
```

## API Documentation

Dokumen dan collection API:

- `pos-backend.postman_collection.json`: Postman collection untuk semua endpoint backend.
- `api-flow.md`: urutan pemakaian API dari bootstrap admin sampai report.

Flow awal yang disarankan:

1. `Health > Health Check`
2. `Auth > Bootstrap Admin`
3. `Product Categories > Create Product Category`
4. `Units > Create Base Unit (PCS)`
5. `Units > Create Package Unit (DUS)`
6. `Products > Create Product`
7. `Product SKUs > Create Retail Product SKU`
8. `Product SKUs > Create Package Product SKU`
9. `Suppliers > Create Supplier`
10. `Purchase Orders > Create Purchase Order`
11. `Purchase Orders > Order Purchase Order`
12. `Goods Receipts > Create Goods Receipt`
13. `Goods Receipts > Post Goods Receipt`
14. `Sales > Create Sale`
15. `Reports > Dashboard`

Protected endpoint memakai header:

```text
Authorization: Bearer {{access_token}}
```

Refresh token dikirim sebagai cookie `HttpOnly`, bukan sebagai JSON response.

## Endpoint Groups

```text
GET    /health

POST   /auth/bootstrap-admin
POST   /auth/login
POST   /auth/refresh
POST   /auth/logout
GET    /auth/me

GET    /users
POST   /users
GET    /users/{id}
PUT    /users/{id}
DELETE /users/{id}

GET    /product-categories
POST   /product-categories
GET    /product-categories/{id}
PUT    /product-categories/{id}
DELETE /product-categories/{id}

GET    /products
POST   /products
GET    /products/{id}
PUT    /products/{id}
DELETE /products/{id}

GET    /units
POST   /units
GET    /units/{id}
PUT    /units/{id}
DELETE /units/{id}

GET    /product-skus
POST   /product-skus
GET    /product-skus/{id}
PUT    /product-skus/{id}
DELETE /product-skus/{id}

GET    /suppliers
POST   /suppliers
GET    /suppliers/{id}
PUT    /suppliers/{id}
DELETE /suppliers/{id}

GET    /purchase-orders
POST   /purchase-orders
GET    /purchase-orders/{id}
POST   /purchase-orders/{id}/order
POST   /purchase-orders/{id}/cancel

GET    /goods-receipts
POST   /goods-receipts
GET    /goods-receipts/{id}
POST   /goods-receipts/{id}/post
POST   /goods-receipts/{id}/cancel

GET    /stock-adjustments
POST   /stock-adjustments
GET    /stock-adjustments/{id}
POST   /stock-adjustments/{id}/post
POST   /stock-adjustments/{id}/cancel

GET    /stock-conversions
POST   /stock-conversions
GET    /stock-conversions/{id}
POST   /stock-conversions/{id}/post
POST   /stock-conversions/{id}/cancel

GET    /sales
POST   /sales
GET    /sales/{id}
POST   /sales/{id}/cancel

GET    /employees
POST   /employees
GET    /employees/{id}
PUT    /employees/{id}
DELETE /employees/{id}

GET    /payroll-slips
POST   /payroll-slips
GET    /payroll-slips/{id}

GET    /dashboard
GET    /reports/daily-sales
GET    /reports/monthly-sales
GET    /reports/stock
GET    /reports/purchase-orders
GET    /reports/goods-receipts
GET    /reports/stock-adjustments
GET    /reports/payroll
```

## Development Notes

- Backend route definition ada di `backend/src/modules/*/routes.rs`.
- Request/response model utama ada di `backend/src/common/models.rs`.
- Business logic utama untuk master data, dokumen, dan report ada di `backend/src/common/services/`.
- Database schema ada di `backend/migrations/`.
- Frontend masih bisa dikembangkan bertahap di atas scaffold SvelteKit dan komponen UI yang sudah ada.
