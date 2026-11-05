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
CORS_ALLOWED_ORIGINS=
JWT_SECRET=dev-only-change-this-secret-before-running-in-production
JWT_TTL_SECONDS=900
REFRESH_TOKEN_TTL_SECONDS=2592000
REFRESH_TOKEN_COOKIE_NAME=refresh_token
REFRESH_TOKEN_COOKIE_SECURE=false
REFRESH_TOKEN_COOKIE_SAME_SITE=Lax
REFRESH_TOKEN_COOKIE_DOMAIN=
```

Untuk local HTTP tanpa HTTPS, browser bisa menolak cookie `Secure`. Saat testing lokal dengan Postman biasanya aman, tetapi untuk browser development gunakan:

```text
REFRESH_TOKEN_COOKIE_SECURE=false
```

Untuk environment production HTTPS dengan frontend di Netlify dan backend di domain sendiri, gunakan `CORS_ALLOWED_ORIGINS=https://nama-site.netlify.app`, `REFRESH_TOKEN_COOKIE_SECURE=true`, dan `REFRESH_TOKEN_COOKIE_SAME_SITE=None`. Biarkan `REFRESH_TOKEN_COOKIE_DOMAIN` kosong jika cookie cukup host-only di domain backend. Jangan gunakan `JWT_SECRET` default untuk environment production.

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

## Production Deploy

Backend production **tidak dibuild di VPS**. Image dikompilasi di GitHub Actions
(`backend/Dockerfile.prod`, multi-stage) lalu dipush ke GHCR
(`ghcr.io/<owner>/pos-backend:latest`). VPS hanya menerima artefak — tidak ada
source code maupun `.git` yang dikirim ke server.

Alur otomatis (`.github/workflows/deploy-backend.yml`, trigger push ke `main`
pada `backend/**`, atau `workflow_dispatch`):

1. Job `build` — build image dan push ke GHCR.
2. Job `deploy` — `scp` hanya 4 artefak ke `~/app` di VPS
   (`docker-compose.prod.yml`, `deploy.sh`, `backend/migrations/`, `backend/seeds/`),
   lalu jalankan `./deploy.sh`. `deploy.sh` mengambil secret dari Vault
   (`kv/pos/config`), login GHCR, `docker compose pull`, lalu `up -d`.

Compose production hanya menjalankan backend dan PostgreSQL. Data PostgreSQL
disimpan sebagai **bind mount ke direktori host** (`POSTGRES_DATA_DIR`, default
`/srv/pos/postgres`) — **bukan** Docker named volume. Konsekuensinya: data tetap
aman walau Docker volume dihapus atau `docker compose down -v` dijalankan; satu-
satunya cara kehilangan data adalah menghapus direktori itu sendiri. Direktori
sengaja diletakkan **di luar `~/app`** agar tidak ikut terhapus saat cleanup atau
tertimpa saat sync artefak deploy. PostgreSQL tidak publish port ke host, sedangkan
backend bind ke `127.0.0.1:8081` untuk dipakai reverse proxy HTTPS.

### Setup VPS sekali di awal

Siapkan direktori data DB (di luar `~/app`), lalu bersihkan `~/app` dari git clone
lama agar tidak ada `.git`/source yang tertinggal:

```bash
sudo mkdir -p /srv/pos/postgres          # direktori data DB (bind mount)
cd ~ && rm -rf ~/app && mkdir -p ~/app   # aman: data DB ada di /srv/pos, bukan ~/app
# verifikasi setelah deploy berjalan: tidak ada .git, src, atau Cargo.*
ls -la ~/app
```

### Migrasi & seed (manual)

Migrasi tidak pernah dijalankan otomatis saat deploy. Jalankan satu file per
perintah, berurutan, dari `~/app` di VPS (butuh `VAULT_TOKEN` di environment):

```bash
./deploy.sh migrate 001_initial_pos_schema.sql
# ... lanjutkan sampai migrasi terakhir secara berurutan
./deploy.sh seed dashboard_sample.sql
```

> Catatan: `docker-compose.prod.yml` me-mount `backend/migrations` dan
> `backend/seeds` read-only ke `/sql/migrations` dan `/sql/seeds` di container
> PostgreSQL — itulah yang dibaca oleh helper di atas.

Frontend production dideploy oleh Netlify dari folder `frontend`. `netlify.toml` sudah mengatur:

```text
base = "frontend"
command = "bun run build"
publish = "build"
```

Set environment variable berikut di Netlify:

```text
PUBLIC_API_BASE_URL=https://domain-backend-anda.com/api/v1
```

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
