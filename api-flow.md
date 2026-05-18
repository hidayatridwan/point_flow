# POS Backend API Flow

Dokumen ini menjelaskan urutan penggunaan API POS dari awal sampai akhir berdasarkan file Postman collection `pos-backend.postman_collection.json`.

Base URL yang dipakai collection:

```text
{{base_url}} = http://localhost:8080/api/v1
```

Setelah bootstrap admin, login, atau refresh, endpoint yang protected memakai header:

```text
Authorization: Bearer {{access_token}}
```

`access_token` berlaku 15 menit dan disimpan client di memory. `refresh_token` berlaku 30 hari, dibuat oleh backend, dan dikirim sebagai `HttpOnly`, `Secure`, `SameSite=Lax` cookie lewat header `Set-Cookie`.

Catatan:

- Kolom Postman request memakai format `Folder > Request` di dalam file `pos-backend.postman_collection.json`.
- Placeholder seperti `{{product_id}}` adalah variable Postman collection.
- Response contoh dibuat ringkas agar mudah dibaca, tetapi tetap mengikuti field response backend.
- Response error umum berbentuk:

```json
{
  "error": "validation error: items must not be empty"
}
```

## 1. Health Check

Gunakan ini untuk memastikan backend aktif sebelum flow lain dijalankan.

| Postman request | Endpoint |
| --- | --- |
| `Health > Health Check` | `GET {{base_url}}/health` |

Relasi: tidak bergantung ke data lain. Ini hanya mengecek service backend.

Contoh response:

```json
{
  "message": "ok"
}
```

## 2. Bootstrap Admin, Login, Refresh, dan Logout

Flow awal dimulai dari membuat admin pertama. Setelah user pertama ada, endpoint bootstrap tidak bisa dipakai lagi dan login normal dipakai untuk mengambil `access_token`. Backend juga membuat `refresh_token`, menyimpan hash-nya di `refresh_tokens`, dan mengirim cookie ke client.

### 2.1 Bootstrap Admin

| Postman request | Endpoint |
| --- | --- |
| `Auth > Bootstrap Admin` | `POST {{base_url}}/auth/bootstrap-admin` |

Relasi: membuat row pertama di `users` dengan role `admin`, mengembalikan `access_token`, membuat row di `refresh_tokens`, lalu mengirim refresh cookie lewat `Set-Cookie`.

Contoh request:

```json
{
  "name": "Admin",
  "email": "admin@example.com",
  "password": "password123",
  "role": "admin"
}
```

Contoh response:

```json
{
  "access_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "token_type": "Bearer",
  "expires_in": 900,
  "user": {
    "id": 1,
    "name": "Admin",
    "email": "admin@example.com",
    "role": "admin",
    "is_active": true,
    "last_login_at": null,
    "created_at": "2026-05-18T10:00:00Z",
    "updated_at": "2026-05-18T10:00:00Z",
    "deleted_at": null
  }
}
```

Header response juga berisi:

```text
Set-Cookie: refresh_token=...; Max-Age=2592000; Path=/api/v1/auth; HttpOnly; SameSite=Lax; Secure
```

### 2.2 Login

| Postman request | Endpoint |
| --- | --- |
| `Auth > Login` | `POST {{base_url}}/auth/login` |

Relasi: validasi credential dari `users`, update `last_login_at`, membuat row di `refresh_tokens`, lalu menghasilkan `access_token` dan refresh cookie.

Contoh request:

```json
{
  "email": "admin@example.com",
  "password": "password123"
}
```

Contoh response:

```json
{
  "access_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "token_type": "Bearer",
  "expires_in": 900,
  "user": {
    "id": 1,
    "name": "Admin",
    "email": "admin@example.com",
    "role": "admin",
    "is_active": true,
    "last_login_at": "2026-05-18T10:05:00Z",
    "created_at": "2026-05-18T10:00:00Z",
    "updated_at": "2026-05-18T10:00:00Z",
    "deleted_at": null
  }
}
```

Header response juga berisi:

```text
Set-Cookie: refresh_token=...; Max-Age=2592000; Path=/api/v1/auth; HttpOnly; SameSite=Lax; Secure
```

### 2.3 Refresh Token

| Postman request | Endpoint |
| --- | --- |
| `Auth > Refresh Token` | `POST {{base_url}}/auth/refresh` |

Relasi: membaca refresh cookie, mencari `refresh_tokens.token_hash` yang masih aktif, melakukan token rotation, lalu mengembalikan `access_token` baru dan refresh cookie baru.

Contoh request:

```text
Tidak ada JSON body. Cookie `refresh_token` dikirim otomatis oleh client.
```

Contoh response:

```json
{
  "access_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "token_type": "Bearer",
  "expires_in": 900,
  "user": {
    "id": 1,
    "name": "Admin",
    "email": "admin@example.com",
    "role": "admin",
    "is_active": true,
    "last_login_at": "2026-05-18T10:05:00Z",
    "created_at": "2026-05-18T10:00:00Z",
    "updated_at": "2026-05-18T10:00:00Z",
    "deleted_at": null
  }
}
```

Header response juga berisi refresh cookie baru:

```text
Set-Cookie: refresh_token=...; Max-Age=2592000; Path=/api/v1/auth; HttpOnly; SameSite=Lax; Secure
```

### 2.4 Current User

| Postman request | Endpoint |
| --- | --- |
| `Auth > Me` | `GET {{base_url}}/auth/me` |

Relasi: membaca user dari token aktif.

Contoh response:

```json
{
  "id": 1,
  "name": "Admin",
  "email": "admin@example.com",
  "role": "admin",
  "is_active": true,
  "last_login_at": "2026-05-18T10:05:00Z",
  "created_at": "2026-05-18T10:00:00Z",
  "updated_at": "2026-05-18T10:00:00Z",
  "deleted_at": null
}
```

### 2.5 Logout

| Postman request | Endpoint |
| --- | --- |
| `Auth > Logout` | `POST {{base_url}}/auth/logout` |

Relasi: membaca refresh cookie, mengisi `refresh_tokens.revoked_at` untuk token aktif, lalu menghapus cookie lewat `Set-Cookie`.

Contoh request:

```text
Tidak ada JSON body. Cookie `refresh_token` dikirim otomatis oleh client.
```

Contoh response:

```json
{
  "message": "logged out"
}
```

Header response juga berisi cookie clearing:

```text
Set-Cookie: refresh_token=; Max-Age=0; Path=/api/v1/auth; HttpOnly; SameSite=Lax; Expires=Thu, 01 Jan 1970 00:00:00 GMT; Secure
```

## 3. User Management

Admin bisa membuat user cashier. Cashier dipakai di flow POS untuk transaksi penjualan.

| Postman request | Endpoint |
| --- | --- |
| `Users > List Users` | `GET {{base_url}}/users?search=&limit=50&offset=0` |
| `Users > Create User` | `POST {{base_url}}/users` |
| `Users > Get User` | `GET {{base_url}}/users/{{user_id}}` |
| `Users > Update User` | `PUT {{base_url}}/users/{{user_id}}` |
| `Users > Delete User` | `DELETE {{base_url}}/users/{{user_id}}` |

Relasi: `users.role = admin` bisa mengelola master data dan dokumen. `users.role = cashier` dipakai sebagai `cashier_id` saat membuat `sales`.

Contoh create request:

```json
{
  "name": "Cashier",
  "email": "cashier@example.com",
  "password": "password123",
  "role": "cashier",
  "is_active": true
}
```

Contoh create response:

```json
{
  "id": 2,
  "name": "Cashier",
  "email": "cashier@example.com",
  "role": "cashier",
  "is_active": true,
  "last_login_at": null,
  "created_at": "2026-05-18T10:10:00Z",
  "updated_at": "2026-05-18T10:10:00Z",
  "deleted_at": null
}
```

Contoh list response:

```json
[
  {
    "id": 1,
    "name": "Admin",
    "email": "admin@example.com",
    "role": "admin",
    "is_active": true,
    "last_login_at": "2026-05-18T10:05:00Z",
    "created_at": "2026-05-18T10:00:00Z",
    "updated_at": "2026-05-18T10:00:00Z",
    "deleted_at": null
  }
]
```

Contoh update request:

```json
{
  "name": "Cashier Updated",
  "email": "cashier@example.com",
  "role": "cashier",
  "is_active": true
}
```

Contoh delete response:

```json
{
  "message": "deleted"
}
```

## 4. Product Category dan Product Setup

Sebelum pembelian dan penjualan, siapkan kategori dan produk. Produk menjadi pusat relasi untuk purchase order, goods receipt, stock adjustment, sales, dan stock report.

### 4.1 Product Categories

| Postman request | Endpoint |
| --- | --- |
| `Product Categories > List Product Categories` | `GET {{base_url}}/product-categories?search=&limit=50&offset=0` |
| `Product Categories > Create Product Category` | `POST {{base_url}}/product-categories` |
| `Product Categories > Get Product Category` | `GET {{base_url}}/product-categories/{{category_id}}` |
| `Product Categories > Update Product Category` | `PUT {{base_url}}/product-categories/{{category_id}}` |
| `Product Categories > Delete Product Category` | `DELETE {{base_url}}/product-categories/{{category_id}}` |

Relasi: `product_categories.id` disimpan sebagai `products.category_id`.

Contoh create request:

```json
{
  "name": "Beverages",
  "description": "Drink products",
  "is_active": true
}
```

Contoh response:

```json
{
  "id": 1,
  "name": "Beverages",
  "description": "Drink products",
  "is_active": true,
  "created_at": "2026-05-18T10:15:00Z",
  "updated_at": "2026-05-18T10:15:00Z",
  "deleted_at": null
}
```

### 4.2 Products

| Postman request | Endpoint |
| --- | --- |
| `Products > List Products` | `GET {{base_url}}/products?search=&limit=50&offset=0` |
| `Products > Create Product` | `POST {{base_url}}/products` |
| `Products > Get Product` | `GET {{base_url}}/products/{{product_id}}` |
| `Products > Update Product` | `PUT {{base_url}}/products/{{product_id}}` |
| `Products > Delete Product` | `DELETE {{base_url}}/products/{{product_id}}` |

Relasi: `products.stock` berubah saat `goods_receipts` diposting, `stock_adjustments` diposting, atau `sales` dibuat.

Contoh create request:

```json
{
  "category_id": {{category_id}},
  "name": "Mineral Water",
  "sku": "WTR-001",
  "purchase_price": 2500,
  "selling_price": 4000,
  "stock": 100,
  "minimum_stock": 10,
  "unit": "pcs",
  "is_active": true
}
```

Contoh response:

```json
{
  "id": 1,
  "category_id": 1,
  "category_name": "Beverages",
  "name": "Mineral Water",
  "sku": "WTR-001",
  "purchase_price": 2500,
  "selling_price": 4000,
  "stock": 100,
  "minimum_stock": 10,
  "unit": "pcs",
  "is_active": true,
  "created_at": "2026-05-18T10:20:00Z",
  "updated_at": "2026-05-18T10:20:00Z",
  "deleted_at": null
}
```

Contoh update request:

```json
{
  "category_id": {{category_id}},
  "name": "Mineral Water Updated",
  "sku": "WTR-001",
  "purchase_price": 2600,
  "selling_price": 4500,
  "minimum_stock": 15,
  "unit": "pcs",
  "is_active": true
}
```

## 5. Supplier Setup

Supplier harus dibuat sebelum membuat purchase order.

| Postman request | Endpoint |
| --- | --- |
| `Suppliers > List Suppliers` | `GET {{base_url}}/suppliers?search=&limit=50&offset=0` |
| `Suppliers > Create Supplier` | `POST {{base_url}}/suppliers` |
| `Suppliers > Get Supplier` | `GET {{base_url}}/suppliers/{{supplier_id}}` |
| `Suppliers > Update Supplier` | `PUT {{base_url}}/suppliers/{{supplier_id}}` |
| `Suppliers > Delete Supplier` | `DELETE {{base_url}}/suppliers/{{supplier_id}}` |

Relasi: `suppliers.id` dipakai oleh `purchase_orders.supplier_id`.

Contoh create request:

```json
{
  "name": "PT Sumber Barang",
  "contact_name": "Budi",
  "phone": "08123456789",
  "email": "supplier@example.com",
  "address": "Jakarta",
  "is_active": true
}
```

Contoh response:

```json
{
  "id": 1,
  "name": "PT Sumber Barang",
  "contact_name": "Budi",
  "phone": "08123456789",
  "email": "supplier@example.com",
  "address": "Jakarta",
  "is_active": true,
  "created_at": "2026-05-18T10:25:00Z",
  "updated_at": "2026-05-18T10:25:00Z",
  "deleted_at": null
}
```

## 6. Purchase Order

Purchase order mencatat rencana pembelian barang dari supplier. Dokumen dibuat sebagai `draft`, lalu dikirim dengan action `order`.

| Postman request | Endpoint |
| --- | --- |
| `Purchase Orders > List Purchase Orders` | `GET {{base_url}}/purchase-orders?search=&from=&to=&limit=50&offset=0` |
| `Purchase Orders > Create Purchase Order` | `POST {{base_url}}/purchase-orders` |
| `Purchase Orders > Get Purchase Order` | `GET {{base_url}}/purchase-orders/{{purchase_order_id}}` |
| `Purchase Orders > Order Purchase Order` | `POST {{base_url}}/purchase-orders/{{purchase_order_id}}/order` |
| `Purchase Orders > Cancel Purchase Order` | `POST {{base_url}}/purchase-orders/{{purchase_order_id}}/cancel` |

Relasi: `purchase_orders` memakai `supplier_id`. Setiap item memakai `product_id` dan menyimpan snapshot `product_name`, `product_sku`, dan `unit`. Stock belum berubah saat purchase order dibuat atau di-order.

Contoh create request:

```json
{
  "supplier_id": {{supplier_id}},
  "order_date": "2026-05-18",
  "expected_date": "2026-05-20",
  "notes": "Initial stock order",
  "items": [
    {
      "product_id": {{product_id}},
      "quantity_ordered": 20,
      "unit_cost": 2500
    }
  ]
}
```

Contoh create response:

```json
{
  "id": 1,
  "po_no": "PO-20260518-0001",
  "supplier_id": 1,
  "supplier_name": "PT Sumber Barang",
  "order_date": "2026-05-18",
  "expected_date": "2026-05-20",
  "status": "draft",
  "subtotal_amount": 50000,
  "notes": "Initial stock order",
  "ordered_by": null,
  "ordered_at": null,
  "cancelled_at": null,
  "cancelled_by": null,
  "cancel_reason": null,
  "created_by": 1,
  "created_at": "2026-05-18T10:30:00Z",
  "updated_at": "2026-05-18T10:30:00Z",
  "items": [
    {
      "id": 1,
      "purchase_order_id": 1,
      "product_id": 1,
      "product_name": "Mineral Water",
      "product_sku": "WTR-001",
      "unit": "pcs",
      "quantity_ordered": 20,
      "quantity_received": 0,
      "unit_cost": 2500,
      "line_total": 50000,
      "created_at": "2026-05-18T10:30:00Z",
      "updated_at": "2026-05-18T10:30:00Z"
    }
  ]
}
```

Contoh order request:

```text
POST {{base_url}}/purchase-orders/{{purchase_order_id}}/order
Body: none
```

Contoh order response:

```json
{
  "id": 1,
  "po_no": "PO-20260518-0001",
  "supplier_id": 1,
  "supplier_name": "PT Sumber Barang",
  "order_date": "2026-05-18",
  "expected_date": "2026-05-20",
  "status": "ordered",
  "subtotal_amount": 50000,
  "notes": "Initial stock order",
  "ordered_by": 1,
  "ordered_at": "2026-05-18T10:35:00Z",
  "cancelled_at": null,
  "cancelled_by": null,
  "cancel_reason": null,
  "created_by": 1,
  "created_at": "2026-05-18T10:30:00Z",
  "updated_at": "2026-05-18T10:30:00Z",
  "items": [
    {
      "id": 1,
      "purchase_order_id": 1,
      "product_id": 1,
      "product_name": "Mineral Water",
      "product_sku": "WTR-001",
      "unit": "pcs",
      "quantity_ordered": 20,
      "quantity_received": 0,
      "unit_cost": 2500,
      "line_total": 50000,
      "created_at": "2026-05-18T10:30:00Z",
      "updated_at": "2026-05-18T10:30:00Z"
    }
  ]
}
```

Contoh cancel request:

```json
{
  "reason": "Wrong input"
}
```

## 7. Goods Receipt

Goods receipt dipakai untuk menerima barang dari purchase order. Stock baru bertambah ketika goods receipt diposting.

| Postman request | Endpoint |
| --- | --- |
| `Goods Receipts > List Goods Receipts` | `GET {{base_url}}/goods-receipts?search=&from=&to=&limit=50&offset=0` |
| `Goods Receipts > Create Goods Receipt` | `POST {{base_url}}/goods-receipts` |
| `Goods Receipts > Get Goods Receipt` | `GET {{base_url}}/goods-receipts/{{goods_receipt_id}}` |
| `Goods Receipts > Post Goods Receipt` | `POST {{base_url}}/goods-receipts/{{goods_receipt_id}}/post` |
| `Goods Receipts > Cancel Goods Receipt` | `POST {{base_url}}/goods-receipts/{{goods_receipt_id}}/cancel` |

Relasi: `goods_receipts.purchase_order_id` mengarah ke purchase order yang statusnya `ordered` atau `partially_received`. `goods_receipt_items.purchase_order_item_id` harus berasal dari purchase order tersebut. Saat diposting, sistem menaikkan `products.stock`, mengubah `purchase_order_items.quantity_received`, mengubah status purchase order, dan menulis `stock_movements`.

Contoh create request:

```json
{
  "purchase_order_id": {{purchase_order_id}},
  "received_at": "2026-05-18T10:40:00Z",
  "notes": "Goods received",
  "items": [
    {
      "purchase_order_item_id": {{purchase_order_item_id}},
      "quantity_received": 10,
      "unit_cost": 2500
    }
  ]
}
```

Contoh create response:

```json
{
  "id": 1,
  "gr_no": "GR-20260518-0001",
  "purchase_order_id": 1,
  "po_no": "PO-20260518-0001",
  "received_at": "2026-05-18T10:40:00Z",
  "status": "draft",
  "received_by": 1,
  "posted_at": null,
  "posted_by": null,
  "cancelled_at": null,
  "cancelled_by": null,
  "cancel_reason": null,
  "notes": "Goods received",
  "created_at": "2026-05-18T10:40:00Z",
  "updated_at": "2026-05-18T10:40:00Z",
  "items": [
    {
      "id": 1,
      "goods_receipt_id": 1,
      "purchase_order_item_id": 1,
      "product_id": 1,
      "product_name": "Mineral Water",
      "product_sku": "WTR-001",
      "unit": "pcs",
      "quantity_received": 10,
      "unit_cost": 2500,
      "line_total": 25000,
      "created_at": "2026-05-18T10:40:00Z",
      "updated_at": "2026-05-18T10:40:00Z"
    }
  ]
}
```

Contoh post request:

```text
POST {{base_url}}/goods-receipts/{{goods_receipt_id}}/post
Body: none
```

Contoh post response:

```json
{
  "id": 1,
  "gr_no": "GR-20260518-0001",
  "purchase_order_id": 1,
  "po_no": "PO-20260518-0001",
  "received_at": "2026-05-18T10:40:00Z",
  "status": "posted",
  "received_by": 1,
  "posted_at": "2026-05-18T10:45:00Z",
  "posted_by": 1,
  "cancelled_at": null,
  "cancelled_by": null,
  "cancel_reason": null,
  "notes": "Goods received",
  "created_at": "2026-05-18T10:40:00Z",
  "updated_at": "2026-05-18T10:40:00Z",
  "items": [
    {
      "id": 1,
      "goods_receipt_id": 1,
      "purchase_order_item_id": 1,
      "product_id": 1,
      "product_name": "Mineral Water",
      "product_sku": "WTR-001",
      "unit": "pcs",
      "quantity_received": 10,
      "unit_cost": 2500,
      "line_total": 25000,
      "created_at": "2026-05-18T10:40:00Z",
      "updated_at": "2026-05-18T10:40:00Z"
    }
  ]
}
```

Jika purchase order hanya diterima sebagian, status purchase order menjadi `partially_received`. Jika semua item sudah diterima, status menjadi `received`.

Contoh cancel request:

```json
{
  "reason": "Wrong input"
}
```

## 8. Stock Adjustment

Stock adjustment dipakai untuk koreksi stok fisik, barang rusak, barang hilang, atau koreksi input. Dokumen dibuat sebagai `draft`, lalu mengubah stok saat diposting.

| Postman request | Endpoint |
| --- | --- |
| `Stock Adjustments > List Stock Adjustments` | `GET {{base_url}}/stock-adjustments?search=&from=&to=&limit=50&offset=0` |
| `Stock Adjustments > Create Stock Adjustment` | `POST {{base_url}}/stock-adjustments` |
| `Stock Adjustments > Get Stock Adjustment` | `GET {{base_url}}/stock-adjustments/{{stock_adjustment_id}}` |
| `Stock Adjustments > Post Stock Adjustment` | `POST {{base_url}}/stock-adjustments/{{stock_adjustment_id}}/post` |
| `Stock Adjustments > Cancel Stock Adjustment` | `POST {{base_url}}/stock-adjustments/{{stock_adjustment_id}}/cancel` |

Relasi: item adjustment memakai `product_id`. Saat adjustment dibuat, backend menyimpan `stock_before` dari produk saat itu. Saat diposting, stock produk harus masih sama dengan `stock_before`, lalu sistem mengubah `products.stock` menjadi `stock_after` dan menulis `stock_movements`.

Contoh create request:

```json
{
  "adjusted_at": "2026-05-18T10:50:00Z",
  "reason": "Stock opname",
  "notes": "Physical stock correction",
  "items": [
    {
      "product_id": {{product_id}},
      "stock_after": 95,
      "notes": "Counted manually"
    }
  ]
}
```

Contoh create response:

```json
{
  "id": 1,
  "adjustment_no": "SA-20260518-0001",
  "adjusted_at": "2026-05-18T10:50:00Z",
  "status": "draft",
  "reason": "Stock opname",
  "notes": "Physical stock correction",
  "created_by": 1,
  "posted_at": null,
  "posted_by": null,
  "cancelled_at": null,
  "cancelled_by": null,
  "cancel_reason": null,
  "created_at": "2026-05-18T10:50:00Z",
  "updated_at": "2026-05-18T10:50:00Z",
  "items": [
    {
      "id": 1,
      "stock_adjustment_id": 1,
      "product_id": 1,
      "product_name": "Mineral Water",
      "product_sku": "WTR-001",
      "unit": "pcs",
      "stock_before": 110,
      "stock_after": 95,
      "quantity_delta": -15,
      "notes": "Counted manually",
      "created_at": "2026-05-18T10:50:00Z",
      "updated_at": "2026-05-18T10:50:00Z"
    }
  ]
}
```

Contoh post request:

```text
POST {{base_url}}/stock-adjustments/{{stock_adjustment_id}}/post
Body: none
```

Contoh post response:

```json
{
  "id": 1,
  "adjustment_no": "SA-20260518-0001",
  "adjusted_at": "2026-05-18T10:50:00Z",
  "status": "posted",
  "reason": "Stock opname",
  "notes": "Physical stock correction",
  "created_by": 1,
  "posted_at": "2026-05-18T10:55:00Z",
  "posted_by": 1,
  "cancelled_at": null,
  "cancelled_by": null,
  "cancel_reason": null,
  "created_at": "2026-05-18T10:50:00Z",
  "updated_at": "2026-05-18T10:50:00Z",
  "items": [
    {
      "id": 1,
      "stock_adjustment_id": 1,
      "product_id": 1,
      "product_name": "Mineral Water",
      "product_sku": "WTR-001",
      "unit": "pcs",
      "stock_before": 110,
      "stock_after": 95,
      "quantity_delta": -15,
      "notes": "Counted manually",
      "created_at": "2026-05-18T10:50:00Z",
      "updated_at": "2026-05-18T10:50:00Z"
    }
  ]
}
```

Contoh cancel request:

```json
{
  "reason": "Wrong input"
}
```

## 9. Sales / Point of Sale

Sales adalah transaksi kasir. Saat sale berhasil dibuat, backend langsung mengurangi stok produk dan menulis `stock_movements` dengan `movement_type = sale`.

| Postman request | Endpoint |
| --- | --- |
| `Sales > List Sales` | `GET {{base_url}}/sales?search=&from=&to=&limit=50&offset=0` |
| `Sales > Create Sale` | `POST {{base_url}}/sales` |
| `Sales > Get Sale` | `GET {{base_url}}/sales/{{sale_id}}` |
| `Sales > Cancel Sale` | `POST {{base_url}}/sales/{{sale_id}}/cancel` |

Relasi: item sale memakai `product_id`. Backend mengambil snapshot `product_name`, `product_sku`, `unit`, dan `selling_price`. `cashier_id` diambil dari token user yang login. Cashier hanya bisa melihat sales miliknya, admin bisa melihat semua sales.

Contoh create request:

```json
{
  "sold_at": "2026-05-18T11:00:00Z",
  "paid_amount": 10000,
  "notes": "Cash sale",
  "items": [
    {
      "product_id": {{product_id}},
      "quantity": 2
    }
  ]
}
```

Contoh create response:

```json
{
  "id": 1,
  "transaction_no": "TRX-20260518-0001",
  "sold_at": "2026-05-18T11:00:00Z",
  "cashier_id": 1,
  "cashier_name": "Admin",
  "subtotal_amount": 8000,
  "total_amount": 8000,
  "paid_amount": 10000,
  "change_amount": 2000,
  "status": "completed",
  "notes": "Cash sale",
  "cancelled_at": null,
  "cancelled_by": null,
  "cancel_reason": null,
  "created_at": "2026-05-18T11:00:00Z",
  "updated_at": "2026-05-18T11:00:00Z",
  "items": [
    {
      "id": 1,
      "sale_id": 1,
      "product_id": 1,
      "product_name": "Mineral Water",
      "product_sku": "WTR-001",
      "unit": "pcs",
      "quantity": 2,
      "unit_price": 4000,
      "line_total": 8000,
      "created_at": "2026-05-18T11:00:00Z"
    }
  ]
}
```

Contoh cancel request:

```json
{
  "reason": "Wrong input"
}
```

Contoh cancel response:

```json
{
  "id": 1,
  "transaction_no": "TRX-20260518-0001",
  "sold_at": "2026-05-18T11:00:00Z",
  "cashier_id": 1,
  "cashier_name": "Admin",
  "subtotal_amount": 8000,
  "total_amount": 8000,
  "paid_amount": 10000,
  "change_amount": 2000,
  "status": "cancelled",
  "notes": "Cash sale",
  "cancelled_at": "2026-05-18T11:05:00Z",
  "cancelled_by": 1,
  "cancel_reason": "Wrong input",
  "created_at": "2026-05-18T11:00:00Z",
  "updated_at": "2026-05-18T11:00:00Z",
  "items": [
    {
      "id": 1,
      "sale_id": 1,
      "product_id": 1,
      "product_name": "Mineral Water",
      "product_sku": "WTR-001",
      "unit": "pcs",
      "quantity": 2,
      "unit_price": 4000,
      "line_total": 8000,
      "created_at": "2026-05-18T11:00:00Z"
    }
  ]
}
```

Catatan penting: cancel sale mengubah status sale menjadi `cancelled`. Dari implementasi saat ini, cancel sale tidak membuat stock movement balik dan tidak otomatis mengembalikan stock.

## 10. Employee Management

Employee dipakai untuk payroll slip. Data employee terpisah dari `users`, karena employee adalah data karyawan operasional, sedangkan user adalah akun login sistem.

| Postman request | Endpoint |
| --- | --- |
| `Employees > List Employees` | `GET {{base_url}}/employees?search=&limit=50&offset=0` |
| `Employees > Create Employee` | `POST {{base_url}}/employees` |
| `Employees > Get Employee` | `GET {{base_url}}/employees/{{employee_id}}` |
| `Employees > Update Employee` | `PUT {{base_url}}/employees/{{employee_id}}` |
| `Employees > Delete Employee` | `DELETE {{base_url}}/employees/{{employee_id}}` |

Relasi: `employees.id` dipakai oleh `payroll_slips.employee_id`.

Contoh create request:

```json
{
  "name": "Siti Aminah",
  "position": "Cashier",
  "phone": "08123456780",
  "address": "Jakarta",
  "base_salary": 3000000,
  "is_active": true
}
```

Contoh response:

```json
{
  "id": 1,
  "name": "Siti Aminah",
  "position": "Cashier",
  "phone": "08123456780",
  "address": "Jakarta",
  "base_salary": 3000000,
  "is_active": true,
  "created_at": "2026-05-18T11:10:00Z",
  "updated_at": "2026-05-18T11:10:00Z",
  "deleted_at": null
}
```

## 11. Payroll Slips

Payroll slip dibuat berdasarkan employee. Backend mengambil snapshot nama, posisi, dan base salary employee saat slip dibuat.

| Postman request | Endpoint |
| --- | --- |
| `Payroll Slips > List Payroll Slips` | `GET {{base_url}}/payroll-slips?search=&from=&to=&limit=50&offset=0` |
| `Payroll Slips > Create Payroll Slip` | `POST {{base_url}}/payroll-slips` |
| `Payroll Slips > Get Payroll Slip` | `GET {{base_url}}/payroll-slips/{{payroll_slip_id}}` |

Relasi: `payroll_slips.employee_id` mengarah ke `employees.id`. `net_salary` dihitung dari `base_salary + bonus - deduction`.

Contoh create request:

```json
{
  "employee_id": {{employee_id}},
  "period_start": "2026-05-01",
  "period_end": "2026-05-31",
  "bonus": 250000,
  "deduction": 50000
}
```

Contoh response:

```json
{
  "id": 1,
  "slip_no": "PAY-20260518-0001",
  "employee_id": 1,
  "employee_name": "Siti Aminah",
  "employee_position": "Cashier",
  "period_start": "2026-05-01",
  "period_end": "2026-05-31",
  "base_salary": 3000000,
  "bonus": 250000,
  "deduction": 50000,
  "net_salary": 3200000,
  "created_by": 1,
  "created_at": "2026-05-18T11:15:00Z",
  "updated_at": "2026-05-18T11:15:00Z"
}
```

## 12. Dashboard dan Reports

Dashboard dan reports membaca agregasi data dari flow sebelumnya. Endpoint ini tidak mengubah data.

| Postman request | Endpoint |
| --- | --- |
| `Reports > Dashboard` | `GET {{base_url}}/dashboard` |
| `Reports > Daily Sales Report` | `GET {{base_url}}/reports/daily-sales?from=&to=&limit=50&offset=0` |
| `Reports > Monthly Sales Report` | `GET {{base_url}}/reports/monthly-sales?from=&to=&limit=50&offset=0` |
| `Reports > Stock Report` | `GET {{base_url}}/reports/stock?search=&limit=50&offset=0` |
| `Reports > Purchase Order Report` | `GET {{base_url}}/reports/purchase-orders?search=&from=&to=&limit=50&offset=0` |
| `Reports > Goods Receipt Report` | `GET {{base_url}}/reports/goods-receipts?search=&from=&to=&limit=50&offset=0` |
| `Reports > Stock Adjustment Report` | `GET {{base_url}}/reports/stock-adjustments?search=&from=&to=&limit=50&offset=0` |
| `Reports > Payroll Report` | `GET {{base_url}}/reports/payroll?from=&to=&limit=50&offset=0` |

Relasi:

- Dashboard membaca sales hari ini, low stock product, pending purchase order, draft stock adjustment, dan active employee.
- Sales reports membaca transaksi `sales` yang statusnya `completed`.
- Stock report membaca kondisi `products.stock` dan quantity yang masih on order.
- Purchase order report membaca progress ordered vs received.
- Goods receipt report membaca penerimaan barang.
- Stock adjustment report membaca koreksi stok.
- Payroll report membaca agregasi slip gaji.

Contoh dashboard response:

```json
{
  "transaction_count_today": 1,
  "revenue_today": 8000,
  "low_stock_product_count": 0,
  "pending_purchase_order_count": 1,
  "draft_stock_adjustment_count": 0,
  "active_employee_count": 1
}
```

Contoh daily sales report response:

```json
[
  {
    "sales_date": "2026-05-18",
    "transaction_count": 1,
    "subtotal_amount": 8000,
    "total_amount": 8000
  }
]
```

Contoh monthly sales report response:

```json
[
  {
    "sales_month": "2026-05-01",
    "transaction_count": 1,
    "subtotal_amount": 8000,
    "total_amount": 8000
  }
]
```

Contoh stock report response:

```json
[
  {
    "id": 1,
    "name": "Mineral Water",
    "sku": "WTR-001",
    "category_name": "Beverages",
    "purchase_price": 2500,
    "selling_price": 4000,
    "stock": 93,
    "minimum_stock": 10,
    "unit": "pcs",
    "quantity_on_order": 10,
    "is_low_stock": false,
    "is_active": true
  }
]
```

Contoh purchase order report response:

```json
[
  {
    "id": 1,
    "po_no": "PO-20260518-0001",
    "order_date": "2026-05-18",
    "expected_date": "2026-05-20",
    "supplier_name": "PT Sumber Barang",
    "status": "partially_received",
    "item_count": 1,
    "total_quantity_ordered": 20,
    "total_quantity_received": 10,
    "total_quantity_remaining": 10,
    "subtotal_amount": 50000,
    "created_at": "2026-05-18T10:30:00Z",
    "updated_at": "2026-05-18T10:45:00Z"
  }
]
```

Contoh goods receipt report response:

```json
[
  {
    "id": 1,
    "gr_no": "GR-20260518-0001",
    "received_at": "2026-05-18T10:40:00Z",
    "status": "posted",
    "po_no": "PO-20260518-0001",
    "supplier_name": "PT Sumber Barang",
    "item_count": 1,
    "total_quantity_received": 10,
    "total_amount": 25000,
    "created_at": "2026-05-18T10:40:00Z",
    "updated_at": "2026-05-18T10:45:00Z"
  }
]
```

Contoh stock adjustment report response:

```json
[
  {
    "id": 1,
    "adjustment_no": "SA-20260518-0001",
    "adjusted_at": "2026-05-18T10:50:00Z",
    "status": "posted",
    "reason": "Stock opname",
    "item_count": 1,
    "total_quantity_increased": 0,
    "total_quantity_decreased": 15,
    "created_by": 1,
    "posted_at": "2026-05-18T10:55:00Z",
    "posted_by": 1,
    "created_at": "2026-05-18T10:50:00Z",
    "updated_at": "2026-05-18T10:55:00Z"
  }
]
```

Contoh payroll report response:

```json
[
  {
    "period_start": "2026-05-01",
    "period_end": "2026-05-31",
    "slip_count": 1,
    "total_base_salary": 3000000,
    "total_bonus": 250000,
    "total_deduction": 50000,
    "total_net_salary": 3200000
  }
]
```

## 13. Urutan Uji End-to-End di Postman

Urutan yang disarankan saat menjalankan collection:

1. `Health > Health Check`
2. `Auth > Bootstrap Admin` atau `Auth > Login`
3. `Auth > Refresh Token` saat `access_token` kedaluwarsa
4. `Auth > Me`
5. `Users > Create User` jika perlu akun cashier
6. `Product Categories > Create Product Category`
7. `Products > Create Product`
8. `Suppliers > Create Supplier`
9. `Purchase Orders > Create Purchase Order`
10. `Purchase Orders > Order Purchase Order`
11. `Goods Receipts > Create Goods Receipt`
12. `Goods Receipts > Post Goods Receipt`
13. `Stock Adjustments > Create Stock Adjustment` jika butuh koreksi stok
14. `Stock Adjustments > Post Stock Adjustment`
15. `Sales > Create Sale`
16. `Employees > Create Employee`
17. `Payroll Slips > Create Payroll Slip`
18. Jalankan `Reports > Dashboard` dan report lain untuk memeriksa hasil agregasi.
19. `Auth > Logout` untuk mencabut refresh token aktif.

Ringkasan relasi utama:

```text
users
  -> refresh_tokens.user_id
  -> sales.cashier_id

refresh_tokens
  -> token_hash for refresh cookie validation
  -> revoked_at when rotated or logged out

product_categories
  -> products.category_id

products
  -> purchase_order_items.product_id
  -> goods_receipt_items.product_id
  -> stock_adjustment_items.product_id
  -> sale_items.product_id

suppliers
  -> purchase_orders.supplier_id

purchase_orders
  -> goods_receipts.purchase_order_id
  -> purchase_order_items.quantity_received

goods_receipts posted
  -> products.stock increases
  -> stock_movements movement_type = purchase_receipt

stock_adjustments posted
  -> products.stock becomes stock_after
  -> stock_movements movement_type = adjustment

sales completed
  -> products.stock decreases
  -> stock_movements movement_type = sale

employees
  -> payroll_slips.employee_id
```
