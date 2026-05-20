# Frontend Sidebar Menu Structure

Dokumen ini menjadi referensi struktur menu dan submenu sidebar frontend berdasarkan fitur API di `api-flow.md`.

Route frontend di bawah ini adalah route yang disarankan untuk implementasi UI. Endpoint API ditulis tanpa prefix `{{base_url}}`, sehingga semuanya berada di bawah `/api/v1`.

## Menu Tree

- Dashboard
  - Ringkasan
- Point of Sale
  - Transaksi Baru
  - Riwayat Penjualan
- Master Data
  - Users
  - Product Categories
  - Products
  - Units
  - Product SKUs
  - Suppliers
  - Employees
- Procurement
  - Purchase Orders
  - Goods Receipts
- Inventory
  - Stock Adjustments
  - Stock Conversions
  - Stock Report
- Payroll
  - Payroll Slips
  - Payroll Report
- Reports
  - Daily Sales
  - Monthly Sales
  - Purchase Orders
  - Goods Receipts
  - Stock Adjustments
- Account
  - Current User
  - Logout

## Route and API Reference

### Dashboard

- Ringkasan
  - route: `/dashboard`
  - api: `GET /dashboard`
  - role: `admin`, `cashier`

### Point of Sale

- Transaksi Baru
  - route: `/pos/sales/new`
  - api: `GET /product-skus`, `POST /sales`
  - role: `admin`, `cashier`
- Riwayat Penjualan
  - route: `/pos/sales`
  - api: `GET /sales`, `GET /sales/{id}`, `POST /sales/{id}/cancel`
  - role: `admin`, `cashier`

### Master Data

- Users
  - route: `/master/users`
  - api: `GET /users`, `POST /users`, `GET /users/{id}`, `PUT /users/{id}`, `DELETE /users/{id}`
  - role: `admin`
- Product Categories
  - route: `/master/product-categories`
  - api: `GET /product-categories`, `POST /product-categories`, `GET /product-categories/{id}`, `PUT /product-categories/{id}`, `DELETE /product-categories/{id}`
  - role: `admin`
- Products
  - route: `/master/products`
  - api: `GET /products`, `POST /products`, `GET /products/{id}`, `PUT /products/{id}`, `DELETE /products/{id}`
  - role: `admin`
- Units
  - route: `/master/units`
  - api: `GET /units`, `POST /units`, `GET /units/{id}`, `PUT /units/{id}`, `DELETE /units/{id}`
  - role: `admin`
- Product SKUs
  - route: `/master/product-skus`
  - api: `GET /product-skus`, `POST /product-skus`, `GET /product-skus/{id}`, `PUT /product-skus/{id}`, `DELETE /product-skus/{id}`
  - role: `admin`
- Suppliers
  - route: `/master/suppliers`
  - api: `GET /suppliers`, `POST /suppliers`, `GET /suppliers/{id}`, `PUT /suppliers/{id}`, `DELETE /suppliers/{id}`
  - role: `admin`
- Employees
  - route: `/master/employees`
  - api: `GET /employees`, `POST /employees`, `GET /employees/{id}`, `PUT /employees/{id}`, `DELETE /employees/{id}`
  - role: `admin`

### Procurement

- Purchase Orders
  - route: `/procurement/purchase-orders`
  - api: `GET /purchase-orders`, `POST /purchase-orders`, `GET /purchase-orders/{id}`, `POST /purchase-orders/{id}/order`, `POST /purchase-orders/{id}/cancel`
  - role: `admin`
- Goods Receipts
  - route: `/procurement/goods-receipts`
  - api: `GET /goods-receipts`, `POST /goods-receipts`, `GET /goods-receipts/{id}`, `POST /goods-receipts/{id}/post`, `POST /goods-receipts/{id}/cancel`
  - role: `admin`

### Inventory

- Stock Adjustments
  - route: `/inventory/stock-adjustments`
  - api: `GET /stock-adjustments`, `POST /stock-adjustments`, `GET /stock-adjustments/{id}`, `POST /stock-adjustments/{id}/post`, `POST /stock-adjustments/{id}/cancel`
  - role: `admin`
- Stock Conversions
  - route: `/inventory/stock-conversions`
  - api: `GET /stock-conversions`, `POST /stock-conversions`, `GET /stock-conversions/{id}`, `POST /stock-conversions/{id}/post`, `POST /stock-conversions/{id}/cancel`
  - role: `admin`
- Stock Report
  - route: `/inventory/stock-report`
  - api: `GET /reports/stock`
  - role: `admin`

### Payroll

- Payroll Slips
  - route: `/payroll/slips`
  - api: `GET /payroll-slips`, `POST /payroll-slips`, `GET /payroll-slips/{id}`
  - role: `admin`
- Payroll Report
  - route: `/payroll/report`
  - api: `GET /reports/payroll`
  - role: `admin`

### Reports

- Daily Sales
  - route: `/reports/daily-sales`
  - api: `GET /reports/daily-sales`
  - role: `admin`
- Monthly Sales
  - route: `/reports/monthly-sales`
  - api: `GET /reports/monthly-sales`
  - role: `admin`
- Purchase Orders
  - route: `/reports/purchase-orders`
  - api: `GET /reports/purchase-orders`
  - role: `admin`
- Goods Receipts
  - route: `/reports/goods-receipts`
  - api: `GET /reports/goods-receipts`
  - role: `admin`
- Stock Adjustments
  - route: `/reports/stock-adjustments`
  - api: `GET /reports/stock-adjustments`
  - role: `admin`

### Account

- Current User
  - route: `/account/me`
  - api: `GET /auth/me`
  - role: `admin`, `cashier`
- Logout
  - route: `/account/logout`
  - api: `POST /auth/logout`
  - role: `admin`, `cashier`

## Not Sidebar Items

Fitur berikut tetap dipakai aplikasi, tetapi tidak ditampilkan sebagai item sidebar utama:

- Health Check
  - api: `GET /health`
  - usage: service readiness check
- Bootstrap Admin
  - api: `POST /auth/bootstrap-admin`
  - usage: setup admin pertama
- Login
  - api: `POST /auth/login`
  - usage: public auth page
- Refresh Token
  - api: `POST /auth/refresh`
  - usage: token refresh otomatis dari client
