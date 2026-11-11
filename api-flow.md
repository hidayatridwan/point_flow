# Point Flow Backend API Flow

Dokumen ini menjelaskan urutan penggunaan API Point Flow dari awal sampai akhir berdasarkan file Postman collection `point_flow-backend.postman_collection.json`.

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

- Kolom Postman request memakai format `Folder > Request` di dalam file `point_flow-backend.postman_collection.json`.
- Placeholder seperti `{{audit_log_id}}`, `{{cash_register_id}}`, `{{cash_register_session_id}}`, `{{cash_drawer_movement_id}}`, `{{product_id}}`, `{{product_sku_id}}`, `{{sale_id}}`, `{{sale_item_id}}`, `{{sale_return_id}}`, `{{unit_pcs_id}}`, `{{unit_dus_id}}`, `{{product_sku_from_id}}`, `{{product_sku_to_id}}`, `{{member_id}}`, `{{online_order_id}}`, `{{financial_category_id}}`, `{{financial_transaction_id}}`, dan `{{profit_sharing_period_id}}` adalah variable Postman collection. `{{member_access_token}}` adalah variable terpisah untuk token member storefront agar tidak tertukar dengan `{{access_token}}` milik staff.
- Response aktual selalu memakai envelope `data`, `meta`, `error`, dan `request_id`. Beberapa contoh response di dokumen ini menampilkan isi `data` saja agar alurnya lebih ringkas.
- Response error umum berbentuk:

```json
{
  "data": null,
  "meta": null,
  "error": {
    "code": "validation_error",
    "message": "validation error: items must not be empty"
  },
  "request_id": "03ec5711-3f3e-44b5-a7e8-39cab4a6c867"
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

## 2. Bootstrap Admin, Login, Refresh, Change Password, dan Logout

Flow awal dimulai dari membuat admin pertama. Setelah user pertama ada, endpoint bootstrap tidak bisa dipakai lagi dan login normal dipakai untuk mengambil `access_token`. Backend juga membuat `refresh_token`, menyimpan hash-nya di `refresh_tokens`, dan mengirim cookie ke client.

### 2.1 Bootstrap Admin

| Postman request | Endpoint |
| --- | --- |
| `Auth > Bootstrap Admin` | `POST {{base_url}}/auth/bootstrap-admin` |

Relasi: membuat row pertama di `users` dengan `role_id` untuk role system `admin`, mengembalikan `access_token`, membuat row di `refresh_tokens`, lalu mengirim refresh cookie lewat `Set-Cookie`.

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
    "role_id": 1,
    "role_code": "admin",
    "role_name": "Admin",
    "role": "admin",
    "permissions": [
      "dashboard",
      "inventory",
      "master_data",
      "payroll",
      "point_flow",
      "procurement",
      "reports",
      "users"
    ],
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
    "role_id": 1,
    "role_code": "admin",
    "role_name": "Admin",
    "role": "admin",
    "permissions": [
      "dashboard",
      "inventory",
      "master_data",
      "payroll",
      "point_flow",
      "procurement",
      "reports",
      "users"
    ],
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
    "role_id": 1,
    "role_code": "admin",
    "role_name": "Admin",
    "role": "admin",
    "permissions": [
      "dashboard",
      "inventory",
      "master_data",
      "payroll",
      "point_flow",
      "procurement",
      "reports",
      "users"
    ],
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
  "role_id": 1,
  "role_code": "admin",
  "role_name": "Admin",
  "role": "admin",
  "permissions": [
    "dashboard",
    "inventory",
    "master_data",
    "payroll",
    "point_flow",
    "procurement",
    "reports",
    "users"
  ],
  "is_active": true,
  "last_login_at": "2026-05-18T10:05:00Z",
  "created_at": "2026-05-18T10:00:00Z",
  "updated_at": "2026-05-18T10:00:00Z",
  "deleted_at": null
}
```

### 2.5 Change Password

| Postman request | Endpoint |
| --- | --- |
| `Auth > Change Password` | `POST {{base_url}}/auth/change-password` |

Relasi: membaca user dari `access_token`, memverifikasi `current_password` terhadap `users.password_hash`, menyimpan hash baru ke `users.password_hash`, mencabut semua refresh token user di `refresh_tokens`, lalu menghapus refresh cookie lewat `Set-Cookie`.

Endpoint ini tidak membutuhkan permission `users`; cukup user sedang login. Setelah berhasil, client harus menghapus `access_token` dan login ulang.

Contoh request:

```json
{
  "current_password": "password123",
  "new_password": "password123"
}
```

Catatan: collection memakai nilai password yang sama agar contoh credential tidak berubah saat endpoint dites. Untuk pemakaian nyata, isi `new_password` dengan password baru minimal 8 karakter.

Contoh response:

```json
{
  "message": "password changed"
}
```

Header response juga berisi cookie clearing:

```text
Set-Cookie: refresh_token=; Max-Age=0; Path=/api/v1/auth; HttpOnly; SameSite=Lax; Expires=Thu, 01 Jan 1970 00:00:00 GMT; Secure
```

### 2.6 Logout

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

### 2.7 Audit Logs

Audit log mencatat event penting seperti login, bootstrap admin, change password, logout, sale create/cancel, sale return create/cancel, cash register session open/close, dan cash drawer movement create/cancel. Endpoint audit log membutuhkan permission `users`.

| Postman request | Endpoint |
| --- | --- |
| `Audit Logs > List Audit Logs` | `GET {{base_url}}/audit-logs?search=&from=&to=&action=&resource_type=&outcome=success,failure&request_id=&limit=50&offset=0` |
| `Audit Logs > Get Audit Log` | `GET {{base_url}}/audit-logs/{{audit_log_id}}` |

Filter tambahan `actor_id={{user_id}}` bisa ditambahkan saat ingin melihat audit log untuk user tertentu. Collection default tidak mengirim `actor_id=` kosong karena field itu bertipe integer.

Contoh list response:

```json
[
  {
    "id": 1,
    "request_id": "03ec5711-3f3e-44b5-a7e8-39cab4a6c867",
    "actor_id": 1,
    "actor_name": "Admin",
    "actor_email": "admin@example.com",
    "actor_role": "admin",
    "action": "sale.create",
    "resource_type": "sale",
    "resource_id": 1,
    "resource_label": null,
    "outcome": "success",
    "before_data": null,
    "after_data": null,
    "metadata": {
      "cash_register_session_id": 1,
      "total_amount": 8000
    },
    "ip_address": null,
    "user_agent": null,
    "occurred_at": "2026-05-18T11:00:00Z"
  }
]
```

Catatan: audit log bersifat append-only. Backend tidak menyimpan password, password hash, refresh token, access token, atau secret di audit payload.

## 3. Cash Register dan Session Laci

Cash register adalah master data device/laci fisik. Cash register session adalah sesi buka/tutup laci pada register tersebut. Transaksi sale baru wajib memakai `cash_register_session_id` dari session yang masih `open`.

### 3.1 Master Cash Register

| Postman request | Endpoint |
| --- | --- |
| `Cash Registers > List Cash Registers` | `GET {{base_url}}/cash-registers?search=&status=active,inactive&limit=50&offset=0` |
| `Cash Registers > Create Cash Register` | `POST {{base_url}}/cash-registers` |
| `Cash Registers > Update Cash Register` | `PUT {{base_url}}/cash-registers/{{cash_register_id}}` |
| `Cash Registers > Delete Cash Register` | `DELETE {{base_url}}/cash-registers/{{cash_register_id}}` |

Relasi: `cash_registers.id` dipakai oleh `cash_register_sessions.cash_register_id`. CRUD register membutuhkan permission `master_data`.

Contoh create request:

```json
{
  "code": "REG-01",
  "name": "Front Register",
  "description": "Main cashier drawer",
  "status": "active"
}
```

Contoh response:

```json
{
  "id": 1,
  "code": "REG-01",
  "name": "Front Register",
  "description": "Main cashier drawer",
  "status": "active",
  "created_at": "2026-05-18T07:50:00Z",
  "updated_at": "2026-05-18T07:50:00Z",
  "deleted_at": null
}
```

### 3.2 Cash Register Session

| Postman request | Endpoint |
| --- | --- |
| `Cash Registers > List Cash Register Sessions` | `GET {{base_url}}/cash-register-sessions?search=&from=&to=&status=open,closed&limit=50&offset=0` |
| `Cash Registers > Open Cash Register Session` | `POST {{base_url}}/cash-register-sessions/open` |
| `Cash Registers > Close Cash Register Session` | `POST {{base_url}}/cash-register-sessions/{{cash_register_session_id}}/close` |

Relasi: session memakai `cash_register_id`, `opened_by`, dan `closed_by`. Satu register hanya boleh punya satu session `open`. Endpoint session membutuhkan permission `point_flow`.

Rumus `expected_cash` saat close (migration 018): `opening_cash + penjualan selesai ber-payment_method 'cash' − refund retur selesai + drawer in − drawer out`. Penjualan transfer/Midtrans tidak dihitung karena uangnya tidak lewat laci fisik; penjualan online *pay_at_store* tercatat `'cash'` sehingga tetap masuk.

Contoh open request:

```json
{
  "cash_register_id": 1,
  "opening_cash": 500000,
  "opening_notes": "Starting drawer cash"
}
```

Contoh open response:

```json
{
  "id": 1,
  "session_no": "CRS-20260518-000001",
  "cash_register_id": 1,
  "cash_register_code": "REG-01",
  "cash_register_name": "Front Register",
  "opened_at": "2026-05-18T08:00:00Z",
  "opened_by": 1,
  "opened_by_name": "Admin",
  "opening_cash": 500000,
  "opening_notes": "Starting drawer cash",
  "closed_at": null,
  "closed_by": null,
  "closed_by_name": null,
  "closing_cash_counted": null,
  "expected_cash": null,
  "cash_difference": null,
  "closing_notes": null,
  "status": "open",
  "created_at": "2026-05-18T08:00:00Z",
  "updated_at": "2026-05-18T08:00:00Z"
}
```

### 3.3 Cash Drawer Movement

Cash drawer movement dipakai untuk mencatat uang kas manual di luar transaksi sale/refund: `cash_in`, `cash_out`, `cash_drop`, dan `petty_cash`. Backend menentukan `direction`: `cash_in` menjadi `in`, sedangkan `cash_out`, `cash_drop`, dan `petty_cash` menjadi `out`.

| Postman request | Endpoint |
| --- | --- |
| `Cash Registers > List Cash Drawer Movements` | `GET {{base_url}}/cash-drawer-movements?search=&from=&to=&status=completed,cancelled&type=cash_in,cash_out,cash_drop,petty_cash&cash_register_session_id={{cash_register_session_id}}&limit=50&offset=0` |
| `Cash Registers > Create Cash Drawer Cash In` | `POST {{base_url}}/cash-drawer-movements` |
| `Cash Registers > Create Cash Drawer Cash Drop` | `POST {{base_url}}/cash-drawer-movements` |
| `Cash Registers > Create Cash Drawer Cash Out` | `POST {{base_url}}/cash-drawer-movements` |
| `Cash Registers > Create Cash Drawer Petty Cash` | `POST {{base_url}}/cash-drawer-movements` |
| `Cash Registers > Get Cash Drawer Movement` | `GET {{base_url}}/cash-drawer-movements/{{cash_drawer_movement_id}}` |
| `Cash Registers > Cancel Cash Drawer Movement` | `POST {{base_url}}/cash-drawer-movements/{{cash_drawer_movement_id}}/cancel` |

Relasi: movement wajib memakai `cash_register_session_id` yang masih `open`. Movement `completed` ikut closing; movement `cancelled` tidak dihitung. Cancel movement hanya boleh selama session terkait masih `open`.

Contoh create cash in request:

```json
{
  "cash_register_session_id": 1,
  "occurred_at": "2026-05-18T08:30:00Z",
  "movement_type": "cash_in",
  "amount": 100000,
  "reason": "Additional drawer cash",
  "notes": "Added from safe"
}
```

Contoh create cash drop request:

```json
{
  "cash_register_session_id": 1,
  "occurred_at": "2026-05-18T12:30:00Z",
  "movement_type": "cash_drop",
  "amount": 50000,
  "reason": "Midday cash drop",
  "notes": "Moved to back office safe"
}
```

Contoh create cash out request:

```json
{
  "cash_register_session_id": 1,
  "occurred_at": "2026-05-18T13:00:00Z",
  "movement_type": "cash_out",
  "amount": 25000,
  "reason": "Cash removed from drawer",
  "notes": "Manual cash out"
}
```

Contoh create petty cash request:

```json
{
  "cash_register_session_id": 1,
  "occurred_at": "2026-05-18T14:00:00Z",
  "movement_type": "petty_cash",
  "amount": 15000,
  "reason": "Small operational expense",
  "notes": "Parking fee"
}
```

Contoh response:

```json
{
  "id": 1,
  "movement_no": "CDM-20260518-000001",
  "cash_register_session_id": 1,
  "session_no": "CRS-20260518-000001",
  "cash_register_id": 1,
  "cash_register_code": "REG-01",
  "cash_register_name": "Front Register",
  "occurred_at": "2026-05-18T08:30:00Z",
  "created_by": 1,
  "created_by_name": "Admin",
  "movement_type": "cash_in",
  "direction": "in",
  "amount": 100000,
  "reason": "Additional drawer cash",
  "notes": "Added from safe",
  "status": "completed",
  "cancelled_at": null,
  "cancelled_by": null,
  "cancelled_by_name": null,
  "cancel_reason": null,
  "created_at": "2026-05-18T08:30:00Z",
  "updated_at": "2026-05-18T08:30:00Z"
}
```

Contoh close request:

```json
{
  "closing_cash_counted": 508000,
  "closing_notes": "Drawer counted at store closing"
}
```

Saat closing, backend menghitung:

```text
expected_cash = opening_cash
  + SUM(sales.total_amount completed dalam session)
  - SUM(sale_returns.refund_amount completed dalam session)
  + SUM(cash_drawer_movements.amount completed direction in dalam session)
  - SUM(cash_drawer_movements.amount completed direction out dalam session)
cash_difference = closing_cash_counted - expected_cash
```

Contoh close response:

```json
{
  "id": 1,
  "session_no": "CRS-20260518-000001",
  "cash_register_id": 1,
  "cash_register_code": "REG-01",
  "cash_register_name": "Front Register",
  "opening_cash": 500000,
  "closing_cash_counted": 508000,
  "expected_cash": 508000,
  "cash_difference": 0,
  "status": "closed"
}
```

## 4. User Management

Admin bisa membuat user cashier. Cashier dipakai di flow Point Flow untuk transaksi penjualan.

| Postman request | Endpoint |
| --- | --- |
| `Users > List Users` | `GET {{base_url}}/users?search=&limit=50&offset=0` |
| `Users > Create User` | `POST {{base_url}}/users` |
| `Users > Get User` | `GET {{base_url}}/users/{{user_id}}` |
| `Users > Update User` | `PUT {{base_url}}/users/{{user_id}}` |
| `Users > Delete User` | `DELETE {{base_url}}/users/{{user_id}}` |

Relasi: `users.role_id` mengarah ke `roles.id`. Role `admin` bisa mengelola semua modul melalui permissions, sedangkan role `cashier` dipakai sebagai `cashier_id` saat membuat `sales`.

Contoh create request:

```json
{
  "name": "Cashier",
  "email": "cashier@example.com",
  "password": "password123",
  "role_id": 2,
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
  "role_id": 2,
  "role_code": "cashier",
  "role_name": "Cashier",
  "role": "cashier",
  "permissions": [
    "dashboard",
    "point_flow"
  ],
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
    "role_id": 1,
    "role_code": "admin",
    "role_name": "Admin",
    "role": "admin",
    "permissions": [
      "dashboard",
      "inventory",
      "master_data",
      "payroll",
      "point_flow",
      "procurement",
      "reports",
      "users"
    ],
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
  "role_id": 2,
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

### 4.1 Roles dan Permissions

Roles dan permissions mengatur akses user ke modul. Endpoint ini membutuhkan permission `users`. Backend menyediakan role system `admin` dan `cashier`; role system tidak bisa dihapus, dan code role system tidak bisa diubah.

| Postman request | Endpoint |
| --- | --- |
| `Roles & Permissions > List Roles` | `GET {{base_url}}/roles?search=&limit=50&offset=0` |
| `Roles & Permissions > Create Role` | `POST {{base_url}}/roles` |
| `Roles & Permissions > Get Role` | `GET {{base_url}}/roles/{{role_id}}` |
| `Roles & Permissions > Update Role` | `PUT {{base_url}}/roles/{{role_id}}` |
| `Roles & Permissions > Replace Role Permissions` | `PUT {{base_url}}/roles/{{role_id}}/permissions` |
| `Roles & Permissions > Delete Role` | `DELETE {{base_url}}/roles/{{role_id}}` |
| `Roles & Permissions > List Permissions` | `GET {{base_url}}/permissions?search=&limit=50&offset=0` |

Relasi: `roles.id` dipakai oleh `users.role_id`, `role_permissions.role_id`, dan `role_permissions.permission_id` menghubungkan role ke `permissions.id`. Permissions milik role dibaca dari `GET /roles/{{role_id}}` lewat field `permissions`.

Contoh create role request:

```json
{
  "code": "cashier_procurement",
  "name": "Cashier Procurement",
  "description": "Point of sale and procurement access.",
  "permission_codes": [
    "dashboard",
    "point_flow",
    "procurement"
  ]
}
```

Contoh role response:

```json
{
  "id": 3,
  "code": "cashier_procurement",
  "name": "Cashier Procurement",
  "description": "Point of sale and procurement access.",
  "is_system": false,
  "permissions": [
    "dashboard",
    "point_flow",
    "procurement"
  ],
  "created_at": "2026-05-18T10:12:00Z",
  "updated_at": "2026-05-18T10:12:00Z",
  "deleted_at": null
}
```

Contoh replace role permissions request:

```json
{
  "permission_codes": [
    "dashboard",
    "point_flow",
    "procurement"
  ]
}
```

Contoh permission response:

```json
[
  {
    "id": 1,
    "code": "dashboard",
    "name": "Dashboard",
    "description": "Access dashboard summary.",
    "created_at": "2026-05-18T10:00:00Z",
    "updated_at": "2026-05-18T10:00:00Z"
  }
]
```

## 5. Product Category dan Product Setup

Sebelum pembelian dan penjualan, siapkan kategori, satuan, produk induk, dan SKU/kemasan produk. Stok, harga, barcode, purchase order, goods receipt, stock adjustment, sales, dan stock report memakai `product_skus`.

### 5.1 Product Categories

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

### 5.2 Units

| Request | Endpoint |
| --- | --- |
| `Units > List Units` | `GET {{base_url}}/units?search=&limit=50&offset=0` |
| `Units > Create Base Unit (PCS)` | `POST {{base_url}}/units` |
| `Units > Create Package Unit (DUS)` | `POST {{base_url}}/units` |
| `Units > Get Unit` | `GET {{base_url}}/units/{{unit_id}}` |
| `Units > Update Unit` | `PUT {{base_url}}/units/{{unit_id}}` |
| `Units > Delete Unit` | `DELETE {{base_url}}/units/{{unit_id}}` |

Relasi: `units.id` dipakai oleh `product_skus.unit_id`. Collection membuat dua contoh unit agar SKU eceran dan SKU paket bisa langsung dipakai untuk stock conversion.

Contoh create base unit request:

```json
{
  "code": "pcs",
  "name": "Pieces",
  "allow_decimal": false,
  "is_active": true
}
```

Contoh create package unit request:

```json
{
  "code": "dus",
  "name": "Dus",
  "allow_decimal": false,
  "is_active": true
}
```

### 5.3 Products

| Postman request | Endpoint |
| --- | --- |
| `Products > List Products` | `GET {{base_url}}/products?search=&limit=50&offset=0` |
| `Products > Create Product` | `POST {{base_url}}/products` |
| `Products > Get Product` | `GET {{base_url}}/products/{{product_id}}` |
| `Products > Update Product` | `PUT {{base_url}}/products/{{product_id}}` |
| `Products > Delete Product` | `DELETE {{base_url}}/products/{{product_id}}` |
| `Products > Upload Product Photo` | `POST {{base_url}}/products/{{product_id}}/photo` |

Upload foto memakai multipart form-data (field pertama berisi file gambar, maks 2 MB dengan content-type image), bukan JSON. Membutuhkan permission `master_data`. Foto tampil di katalog publik. Di Postman, pilih file pada key `file` di tab Body form-data.

Relasi: `products` adalah produk induk. Produk seperti Mineral Water dibuat satu kali di sini, lalu kemasan eceran/dus dibuat di `product_skus`.

Contoh create request:

```json
{
  "category_id": {{category_id}},
  "name": "Mineral Water",
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
  "is_active": true
}
```

### 5.4 Product SKUs

| Request | Endpoint |
| --- | --- |
| `Product SKUs > List Product SKUs` | `GET {{base_url}}/product-skus?search=&limit=50&offset=0` |
| `Product SKUs > Create Retail Product SKU` | `POST {{base_url}}/product-skus` |
| `Product SKUs > Create Package Product SKU` | `POST {{base_url}}/product-skus` |
| `Product SKUs > Get Product SKU` | `GET {{base_url}}/product-skus/{{product_sku_id}}` |
| `Product SKUs > Update Product SKU` | `PUT {{base_url}}/product-skus/{{product_sku_id}}` |
| `Product SKUs > Delete Product SKU` | `DELETE {{base_url}}/product-skus/{{product_sku_id}}` |

Relasi: `product_skus.stock` berubah saat goods receipt diposting, stock adjustment diposting, stock conversion diposting, atau sale dibuat. SKU dan barcode unik di level kemasan. `contents_qty` menyimpan isi unit tersebut terhadap unit dasar, misalnya 1 dus berisi 40 pcs.

Contoh create retail SKU request:

```json
{
  "product_id": {{product_id}},
  "sku": "WTR-001-PCS",
  "barcode": "8990000000011",
  "unit_id": {{unit_pcs_id}},
  "contents_qty": 1,
  "purchase_price": 2500,
  "selling_price": 4000,
  "stock": 100,
  "minimum_stock": 10,
  "is_active": true
}
```

Contoh create package SKU request:

```json
{
  "product_id": {{product_id}},
  "sku": "WTR-001-DUS",
  "barcode": "8990000000012",
  "unit_id": {{unit_dus_id}},
  "contents_qty": 40,
  "purchase_price": 95000,
  "selling_price": 120000,
  "stock": 5,
  "minimum_stock": 1,
  "is_active": true
}
```

Contoh response:

```json
{
  "id": 1,
  "product_id": 1,
  "product_name": "Mineral Water",
  "category_id": 1,
  "category_name": "Beverages",
  "sku": "WTR-001-PCS",
  "barcode": "8990000000011",
  "unit_id": 1,
  "unit_code": "pcs",
  "unit_name": "Pieces",
  "contents_qty": 1,
  "purchase_price": 2500,
  "selling_price": 4000,
  "stock": 100,
  "minimum_stock": 10,
  "is_active": true,
  "created_at": "2026-05-18T10:20:00Z",
  "updated_at": "2026-05-18T10:20:00Z",
  "deleted_at": null
}
```

## 6. Supplier Setup

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

## 7. Purchase Order

Purchase order mencatat rencana pembelian barang dari supplier. Dokumen dibuat sebagai `draft`, lalu dikirim dengan action `order`.

| Postman request | Endpoint |
| --- | --- |
| `Purchase Orders > List Purchase Orders` | `GET {{base_url}}/purchase-orders?search=&from=&to=&limit=50&offset=0` |
| `Purchase Orders > Create Purchase Order` | `POST {{base_url}}/purchase-orders` |
| `Purchase Orders > Get Purchase Order` | `GET {{base_url}}/purchase-orders/{{purchase_order_id}}` |
| `Purchase Orders > Order Purchase Order` | `POST {{base_url}}/purchase-orders/{{purchase_order_id}}/order` |
| `Purchase Orders > Cancel Purchase Order` | `POST {{base_url}}/purchase-orders/{{purchase_order_id}}/cancel` |

Relasi: `purchase_orders` memakai `supplier_id`. Setiap item memakai `product_sku_id` dan menyimpan snapshot `product_name`, `product_sku`, dan `unit`. Stock belum berubah saat purchase order dibuat atau di-order.

Contoh create request:

```json
{
  "supplier_id": {{supplier_id}},
  "order_date": "2026-05-18",
  "expected_date": "2026-05-20",
  "notes": "Initial stock order",
  "items": [
    {
      "product_sku_id": {{product_sku_id}},
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
      "product_sku_id": 1,
      "product_name": "Mineral Water",
      "product_sku": "WTR-001-PCS",
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
      "product_sku_id": 1,
      "product_name": "Mineral Water",
      "product_sku": "WTR-001-PCS",
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

## 8. Goods Receipt

Goods receipt dipakai untuk menerima barang dari purchase order. Stock baru bertambah ketika goods receipt diposting.

| Postman request | Endpoint |
| --- | --- |
| `Goods Receipts > List Goods Receipts` | `GET {{base_url}}/goods-receipts?search=&from=&to=&limit=50&offset=0` |
| `Goods Receipts > Create Goods Receipt` | `POST {{base_url}}/goods-receipts` |
| `Goods Receipts > Get Goods Receipt` | `GET {{base_url}}/goods-receipts/{{goods_receipt_id}}` |
| `Goods Receipts > Post Goods Receipt` | `POST {{base_url}}/goods-receipts/{{goods_receipt_id}}/post` |
| `Goods Receipts > Cancel Goods Receipt` | `POST {{base_url}}/goods-receipts/{{goods_receipt_id}}/cancel` |

Relasi: `goods_receipts.purchase_order_id` mengarah ke purchase order yang statusnya `ordered` atau `partially_received`. `goods_receipt_items.purchase_order_item_id` harus berasal dari purchase order tersebut. Saat diposting, sistem menaikkan `product_skus.stock`, mengubah `purchase_order_items.quantity_received`, mengubah status purchase order, dan menulis `stock_movements`.

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
      "product_sku_id": 1,
      "product_name": "Mineral Water",
      "product_sku": "WTR-001-PCS",
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
      "product_sku_id": 1,
      "product_name": "Mineral Water",
      "product_sku": "WTR-001-PCS",
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

## 9. Stock Adjustment

Stock adjustment dipakai untuk koreksi stok fisik, barang rusak, barang hilang, atau koreksi input. Dokumen dibuat sebagai `draft`, lalu mengubah stok saat diposting.

| Postman request | Endpoint |
| --- | --- |
| `Stock Adjustments > List Stock Adjustments` | `GET {{base_url}}/stock-adjustments?search=&from=&to=&limit=50&offset=0` |
| `Stock Adjustments > Create Stock Adjustment` | `POST {{base_url}}/stock-adjustments` |
| `Stock Adjustments > Get Stock Adjustment` | `GET {{base_url}}/stock-adjustments/{{stock_adjustment_id}}` |
| `Stock Adjustments > Post Stock Adjustment` | `POST {{base_url}}/stock-adjustments/{{stock_adjustment_id}}/post` |
| `Stock Adjustments > Cancel Stock Adjustment` | `POST {{base_url}}/stock-adjustments/{{stock_adjustment_id}}/cancel` |

Relasi: item adjustment memakai `product_sku_id`. Saat adjustment dibuat, backend menyimpan `stock_before` dari SKU/kemasan saat itu. Saat diposting, stock SKU harus masih sama dengan `stock_before`, lalu sistem mengubah `product_skus.stock` menjadi `stock_after` dan menulis `stock_movements`.

Contoh create request:

```json
{
  "adjusted_at": "2026-05-18T10:50:00Z",
  "reason": "Stock opname",
  "notes": "Physical stock correction",
  "items": [
    {
      "product_sku_id": {{product_sku_id}},
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
      "product_sku_id": 1,
      "product_name": "Mineral Water",
      "product_sku": "WTR-001-PCS",
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
      "product_sku_id": 1,
      "product_name": "Mineral Water",
      "product_sku": "WTR-001-PCS",
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

## 10. Stock Conversion

Stock conversion dipakai untuk memecah atau menggabungkan stok antar SKU/kemasan secara manual. Contoh: 1 SKU dus Mineral Water dikonversi menjadi 40 SKU eceran Mineral Water.

| Request | Endpoint |
| --- | --- |
| `Stock Conversions > List Stock Conversions` | `GET {{base_url}}/stock-conversions?search=&from=&to=&limit=50&offset=0` |
| `Stock Conversions > Create Stock Conversion` | `POST {{base_url}}/stock-conversions` |
| `Stock Conversions > Get Stock Conversion` | `GET {{base_url}}/stock-conversions/{{stock_conversion_id}}` |
| `Stock Conversions > Post Stock Conversion` | `POST {{base_url}}/stock-conversions/{{stock_conversion_id}}/post` |
| `Stock Conversions > Cancel Stock Conversion` | `POST {{base_url}}/stock-conversions/{{stock_conversion_id}}/cancel` |

Contoh create request:

```json
{
  "converted_at": "2026-05-18T10:58:00Z",
  "notes": "Pecah dus ke eceran",
  "items": [
    {
      "from_product_sku_id": {{product_sku_from_id}},
      "from_quantity": 1,
      "to_product_sku_id": {{product_sku_to_id}},
      "to_quantity": 40,
      "notes": "1 dus isi 40 pcs"
    }
  ]
}
```

Saat diposting, backend mengurangi stok SKU sumber, menambah stok SKU tujuan, dan menulis `stock_movements` dengan `movement_type = conversion_out` dan `conversion_in`.

## 11. Sales / Point of Sale

Sales adalah transaksi kasir. Saat sale berhasil dibuat, backend langsung mengurangi stok SKU/kemasan dan menulis `stock_movements` dengan `movement_type = sale`. Sale baru wajib memakai `cash_register_session_id` dari session laci yang masih `open`.

| Postman request | Endpoint |
| --- | --- |
| `Sales > List Sales` | `GET {{base_url}}/sales?search=&from=&to=&status=completed,cancelled&limit=50&offset=0` |
| `Sales > Create Sale` | `POST {{base_url}}/sales` |
| `Sales > Get Sale` | `GET {{base_url}}/sales/{{sale_id}}` |
| `Sales > Cancel Sale` | `POST {{base_url}}/sales/{{sale_id}}/cancel` |

Relasi: item sale memakai `product_sku_id`. Backend mengambil snapshot `product_name`, `product_sku`, `unit`, dan `selling_price`. `cashier_id` diambil dari token user yang login. `cash_register_session_id` mengikat transaksi ke laci/register yang sedang open. Cashier hanya bisa melihat sales miliknya, admin bisa melihat semua sales.

Contoh create request:

```json
{
  "sold_at": "2026-05-18T11:00:00Z",
  "cash_register_session_id": {{cash_register_session_id}},
  "paid_amount": 10000,
  "notes": "Cash sale",
  "items": [
    {
      "product_sku_id": {{product_sku_id}},
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
  "cash_register_session_id": 1,
  "cash_register_id": 1,
  "cash_register_code": "REG-01",
  "cash_register_name": "Front Register",
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
      "product_sku_id": 1,
      "product_name": "Mineral Water",
      "product_sku": "WTR-001-PCS",
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
  "cash_register_session_id": 1,
  "cash_register_id": 1,
  "cash_register_code": "REG-01",
  "cash_register_name": "Front Register",
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
      "product_sku_id": 1,
      "product_name": "Mineral Water",
      "product_sku": "WTR-001-PCS",
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

## 12. Sales Return / Refund

Sales return adalah dokumen refund atas sale yang sudah `completed`. Return bisa partial atau full berdasarkan item sale. Jika `restock = true`, backend menambah stok SKU dan menulis `stock_movements` dengan `movement_type = sale_return`. Jika `restock = false`, refund tetap tercatat tetapi stok tidak berubah.

| Postman request | Endpoint |
| --- | --- |
| `Sale Returns > List Sale Returns` | `GET {{base_url}}/sale-returns?search=&from=&to=&status=completed,cancelled&limit=50&offset=0` |
| `Sale Returns > Get Returnable Sale Items` | `GET {{base_url}}/sales/{{sale_id}}/returnable-items` |
| `Sale Returns > Create Sale Return Restock` | `POST {{base_url}}/sale-returns` |
| `Sale Returns > Create Sale Return No Restock` | `POST {{base_url}}/sale-returns` |
| `Sale Returns > Get Sale Return` | `GET {{base_url}}/sale-returns/{{sale_return_id}}` |
| `Sale Returns > Cancel Sale Return` | `POST {{base_url}}/sale-returns/{{sale_return_id}}/cancel` |

Relasi: `sale_returns.sale_id` mengarah ke sale asli, `sale_return_items.sale_item_id` mengarah ke item sale asli, dan `cash_register_session_id` mengikat refund cash ke sesi laci yang masih `open`. Cashier hanya melihat return yang dibuat olehnya, admin bisa melihat semua return.

Contoh returnable items response:

```json
[
  {
    "id": 1,
    "sale_id": 1,
    "product_id": 1,
    "product_sku_id": 1,
    "product_name": "Mineral Water",
    "product_sku": "WTR-001-PCS",
    "unit": "pcs",
    "quantity": 2,
    "returned_quantity": 0,
    "returnable_quantity": 2,
    "unit_price": 4000,
    "line_total": 8000,
    "created_at": "2026-05-18T11:00:00Z"
  }
]
```

Contoh create return request:

```json
{
  "sale_id": 1,
  "returned_at": "2026-05-18T12:00:00Z",
  "cash_register_session_id": 1,
  "reason": "Customer return",
  "notes": "Restocked item",
  "items": [
    {
      "sale_item_id": 1,
      "quantity": 1,
      "restock": true,
      "reason": "Unopened item"
    }
  ]
}
```

Contoh create return response:

```json
{
  "id": 1,
  "return_no": "RET-20260518-000001",
  "sale_id": 1,
  "sale_transaction_no": "TRX-20260518-000001",
  "returned_at": "2026-05-18T12:00:00Z",
  "cashier_id": 1,
  "cashier_name": "Admin",
  "cash_register_session_id": 1,
  "cash_register_id": 1,
  "cash_register_code": "REG-01",
  "cash_register_name": "Front Register",
  "subtotal_amount": 4000,
  "refund_amount": 4000,
  "status": "completed",
  "reason": "Customer return",
  "notes": "Restocked item",
  "cancelled_at": null,
  "cancelled_by": null,
  "cancel_reason": null,
  "created_at": "2026-05-18T12:00:00Z",
  "updated_at": "2026-05-18T12:00:00Z",
  "items": [
    {
      "id": 1,
      "sale_return_id": 1,
      "sale_item_id": 1,
      "product_id": 1,
      "product_sku_id": 1,
      "product_name": "Mineral Water",
      "product_sku": "WTR-001-PCS",
      "unit": "pcs",
      "quantity": 1,
      "unit_price": 4000,
      "line_total": 4000,
      "restock": true,
      "reason": "Unopened item",
      "created_at": "2026-05-18T12:00:00Z"
    }
  ]
}
```

Contoh cancel return request:

```json
{
  "reason": "Refund reversal"
}
```

Catatan penting: quantity return tidak boleh melebihi sisa quantity item yang belum diretur. Cancel return hanya bisa dilakukan saat cash register session return masih `open`; jika item return sebelumnya `restock = true`, cancel akan mengurangi stok kembali dan menulis movement `correction`.

## 13. Employee Management

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

## 14. Payroll Slips

Payroll slip dibuat berdasarkan employee. Backend mengambil snapshot nama, posisi, dan base salary employee saat slip dibuat.

| Postman request | Endpoint |
| --- | --- |
| `Payroll Slips > List Payroll Slips` | `GET {{base_url}}/payroll-slips?search=&from=&to=&limit=50&offset=0` |
| `Payroll Slips > Create Payroll Slip` | `POST {{base_url}}/payroll-slips` |
| `Payroll Slips > Get Payroll Slip` | `GET {{base_url}}/payroll-slips/{{payroll_slip_id}}` |
| `Payroll Slips > Cancel Payroll Slip` | `POST {{base_url}}/payroll-slips/{{payroll_slip_id}}/cancel` |

Relasi: `payroll_slips.employee_id` mengarah ke `employees.id`. `net_salary` dihitung dari `base_salary + bonus - deduction`. Slip baru lahir dengan `status = 'completed'` (migration 018); cancel memakai body `{"reason": "..."}` hanya sah untuk slip `completed` dan menandai `cancelled` + `cancelled_at/by/reason`. Slip batal dikeluarkan dari `v_payroll_report` sehingga Laporan Payroll serta baris beban gaji di income statement / financial position tidak menghitungnya. Slip tidak punya update; koreksi berarti buat ulang slip yang benar.

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
  "status": "completed",
  "cancelled_at": null,
  "cancelled_by": null,
  "cancel_reason": null,
  "created_by": 1,
  "created_at": "2026-05-18T11:15:00Z",
  "updated_at": "2026-05-18T11:15:00Z"
}
```

## 15. Dashboard dan Reports

Dashboard dan reports membaca agregasi data dari flow sebelumnya. Endpoint ini tidak mengubah data.

| Postman request | Endpoint |
| --- | --- |
| `Reports > Dashboard` | `GET {{base_url}}/dashboard` |
| `Reports > Daily Sales Report` | `GET {{base_url}}/reports/daily-sales?from=&to=&limit=50&offset=0` |
| `Reports > Monthly Sales Report` | `GET {{base_url}}/reports/monthly-sales?from=&to=&limit=50&offset=0` |
| `Reports > Stock Report` | `GET {{base_url}}/reports/stock?search=&limit=50&offset=0` |
| `Reports > Stock Movement Report` | `GET {{base_url}}/reports/stock-movements?search=&from=&to=&type=purchase_receipt,adjustment,sale,sale_return,conversion_in,conversion_out,correction&product_sku_id=&limit=50&offset=0` |
| `Reports > Purchase Order Report` | `GET {{base_url}}/reports/purchase-orders?search=&from=&to=&limit=50&offset=0` |
| `Reports > Goods Receipt Report` | `GET {{base_url}}/reports/goods-receipts?search=&from=&to=&limit=50&offset=0` |
| `Reports > Stock Adjustment Report` | `GET {{base_url}}/reports/stock-adjustments?search=&from=&to=&limit=50&offset=0` |
| `Reports > Payroll Report` | `GET {{base_url}}/reports/payroll?from=&to=&limit=50&offset=0` |

Relasi:

- Dashboard membaca sales hari ini, low stock product, pending purchase order, draft stock adjustment, dan active employee.
- Sales reports membaca transaksi `sales` yang statusnya `completed`.
- Stock report membaca kondisi `product_skus.stock` dan quantity yang masih on order.
- Stock movement report membaca semua pergerakan stok, termasuk `sale_return` dan `correction` dari cancel return.
- Purchase order report membaca progress ordered vs received.
- Goods receipt report membaca penerimaan barang.
- Stock adjustment report membaca koreksi stok.
- Payroll report membaca agregasi slip gaji.
- Laporan keuangan derivatif (income statement, cash flow, financial position) dibahas di bagian 19 karena datanya berasal dari transaksi keuangan manual; di Postman ketiganya ada di folder `Finances >`, bukan `Reports >`.

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
    "product_sku_id": 1,
    "product_id": 1,
    "product_name": "Mineral Water",
    "sku": "WTR-001-PCS",
    "barcode": "8990000000011",
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

## 16. Members

Anggota (member) adalah master data pelanggan terdaftar untuk storefront online. CRUD member membutuhkan permission `members`.

| Postman request | Endpoint |
| --- | --- |
| `Members > List Members` | `GET {{base_url}}/members?search=&is_active=&limit=50&offset=0` |
| `Members > Create Member` | `POST {{base_url}}/members` |
| `Members > Get Member` | `GET {{base_url}}/members/{{member_id}}` |
| `Members > Update Member` | `PUT {{base_url}}/members/{{member_id}}` |
| `Members > Delete Member` | `DELETE {{base_url}}/members/{{member_id}}` |
| `Members > Reset Member Password` | `POST {{base_url}}/members/{{member_id}}/reset-password` |

Relasi: `member_no` dibuat otomatis oleh backend. Telepon bersifat unique dan dipakai sebagai identitas login storefront (bagian 17). Delete member yang sudah punya transaksi tidak menghapus row — hanya menonaktifkan (`is_active = false`, soft delete).

Contoh create request:

```json
{
  "full_name": "Siti Aminah",
  "phone": "08123456780",
  "email": "siti@example.com",
  "password": "password123",
  "address": "Jl. Anggrek 5",
  "is_active": true,
  "notes": null
}
```

Semua field update bersifat opsional. Reset password memakai body `{"new_password": "newpassword123"}` (minimal 8 karakter).

Contoh response:

```json
{
  "id": 1,
  "member_no": "MBR-20260518-000001",
  "full_name": "Siti Aminah",
  "phone": "08123456780",
  "email": "siti@example.com",
  "address": "Jl. Anggrek 5",
  "joined_at": "2026-05-18T10:00:00Z",
  "is_active": true,
  "notes": null,
  "created_at": "2026-05-18T10:00:00Z",
  "updated_at": "2026-05-18T10:00:00Z"
}
```

## 17. Member Auth

Sesi login storefront untuk member. Lima endpoint ini bersifat publik (tanpa `AuthUser`) dan dibatasi rate limit 10 request/menit per IP karena rawan brute-force. Refresh memakai cookie HttpOnly `member_refresh_token` (terpisah dari cookie `refresh_token` milik staff).

| Postman request | Endpoint |
| --- | --- |
| `Member Auth > Member Login` | `POST {{base_url}}/members/auth/login` |
| `Member Auth > Member Refresh` | `POST {{base_url}}/members/auth/refresh` |
| `Member Auth > Member Logout` | `POST {{base_url}}/members/auth/logout` |
| `Member Auth > Member Me` | `GET {{base_url}}/members/auth/me` |
| `Member Auth > Member Change Password` | `POST {{base_url}}/members/auth/change-password` |

Relasi: login memakai `phone` + `password`, mengembalikan `access_token` member dan profil member, lalu mengirim cookie `member_refresh_token`. Test script request `Member Login` otomatis menyimpan `member_access_token` dan `member_id` ke collection variable. Endpoint `Member Me` dan `Member Change Password` memakai header `Authorization: Bearer {{member_access_token}}` — token staff (`{{access_token}}`) tidak berlaku di sini dan sebaliknya.

Contoh login request:

```json
{
  "phone": "08123456780",
  "password": "password123"
}
```

Contoh login response:

```json
{
  "access_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "token_type": "Bearer",
  "expires_in": 900,
  "member": {
    "id": 1,
    "member_no": "MBR-20260518-000001",
    "full_name": "Siti Aminah",
    "phone": "08123456780",
    "email": "siti@example.com",
    "address": "Jl. Anggrek 5",
    "joined_at": "2026-05-18T10:00:00Z",
    "is_active": true,
    "notes": null,
    "created_at": "2026-05-18T10:00:00Z",
    "updated_at": "2026-05-18T10:00:00Z"
  }
}
```

Refresh dan logout tidak memakai JSON body — cookie `member_refresh_token` dikirim otomatis oleh client. Change password memakai body yang sama dengan staff (`current_password`, `new_password`) dan mencabut semua refresh token member tersebut.

## 18. Online Orders dan Public Storefront

Pesanan online dibuat publik lewat checkout storefront, lalu diproses kasir/admin lewat pipeline order. Endpoint staff memakai JWT + permission `online_orders`. Endpoint publik tanpa auth dan dibatasi rate limit per IP: katalog/info toko 60 request/menit, checkout 10 request/menit. Kelebihan kuota mengembalikan envelope error `rate_limited` dengan status 429.

### 18.1 Store Settings

Pengaturan toko yang tampil di storefront (nama, jam operasional, rekening, e-wallet, ongkir, WhatsApp).

| Postman request | Endpoint |
| --- | --- |
| `Online Orders > Get Store Settings` | `GET {{base_url}}/store-settings` |
| `Online Orders > Update Store Settings` | `PUT {{base_url}}/store-settings` |
| `Online Orders > Upload QRIS Image` | `POST {{base_url}}/store-settings/qris-image` |

Upload QRIS memakai multipart form-data seperti foto produk (maks 2 MB) dan tampil di `Public Storefront > Get Store Info`. Membutuhkan permission `online_orders`.

Semua field update opsional. Contoh request:

```json
{
  "store_name": "Toko Point Flow",
  "operating_hours": "08:00-21:00",
  "bank_accounts": "BCA 1234567890 a.n. Toko",
  "ewallet_numbers": "08123456780 (DANA)",
  "delivery_fee": 10000,
  "shop_whatsapp": "08123456780"
}
```

### 18.2 Pipeline Order (Staff)

| Postman request | Endpoint |
| --- | --- |
| `Online Orders > List Orders` | `GET {{base_url}}/online-orders?search=&from=&to=&status=new,packing,ready&limit=50&offset=0` |
| `Online Orders > Get Order` | `GET {{base_url}}/online-orders/{{online_order_id}}` |
| `Online Orders > Pack Order` | `POST {{base_url}}/online-orders/{{online_order_id}}/pack` |
| `Online Orders > Ready Order` | `POST {{base_url}}/online-orders/{{online_order_id}}/ready` |
| `Online Orders > Deliver Order` | `POST {{base_url}}/online-orders/{{online_order_id}}/deliver` |
| `Online Orders > Complete Order` | `POST {{base_url}}/online-orders/{{online_order_id}}/complete` |
| `Online Orders > Mark Order Paid` | `POST {{base_url}}/online-orders/{{online_order_id}}/mark-paid` |
| `Online Orders > Cancel Order` | `POST {{base_url}}/online-orders/{{online_order_id}}/cancel` |

Relasi status: `new` → `pack` → `packing`. Order pickup: `packing` → `ready` → `complete`. Order delivery: `packing` → `delivering` → `complete` (hanya jika sudah dibayar, atau metode `pay_at_store`). Cancel hanya dari status `new`/`packing` memakai body `{"reason": "..."}`. `Mark Order Paid` tanpa body, dipakai untuk order transfer yang pelunasannya dikonfirmasi manual. Aksi pipeline tanpa body selain cancel.

### 18.3 Katalog dan Checkout Publik

| Postman request | Endpoint |
| --- | --- |
| `Public Storefront > List Public Products` | `GET {{base_url}}/public/products?search=&category_id=&limit=50&offset=0` |
| `Public Storefront > List Public Categories` | `GET {{base_url}}/public/categories` |
| `Public Storefront > Get Store Info` | `GET {{base_url}}/public/store-info` |
| `Public Storefront > Checkout (Guest)` | `POST {{base_url}}/public/orders` |
| `Public Storefront > Checkout (Member)` | `POST {{base_url}}/public/orders` |

Relasi: katalog hanya menampilkan SKU aktif yang stoknya tersedia. `fulfillment_method` wajib `pickup` atau `delivery`; `payment_method` wajib `pay_at_store` atau `transfer`. Atribusi member selalu diambil dari JWT member (`Authorization: Bearer {{member_access_token}}`), tidak pernah dari body — body checkout tidak punya field `member_id` (anti-spoof). Checkout guest memakai nama/telepon dari body; checkout member memakai nama/telepon dari profil. Checkout guest wajib tanpa header Authorization — token staff yang terbawa akan ditolak 401.

Contoh checkout guest request:

```json
{
  "customer_name": "Guest",
  "customer_phone": "0811111111",
  "fulfillment_method": "pickup",
  "payment_method": "pay_at_store",
  "items": [
    { "product_sku_id": 1, "quantity": 2 }
  ]
}
```

Contoh checkout member request (dengan header `Authorization: Bearer {{member_access_token}}`):

```json
{
  "customer_name": "Siti Aminah",
  "customer_phone": "08123456780",
  "fulfillment_method": "delivery",
  "delivery_address": "Jl. Anggrek 5",
  "payment_method": "transfer",
  "items": [
    { "product_sku_id": 1, "quantity": 1 }
  ]
}
```

Contoh checkout response:

```json
{
  "order_id": 1,
  "order_no": "OO-20260518-000001",
  "fulfillment_method": "pickup",
  "payment_method": "pay_at_store",
  "total_amount": 8000,
  "items": [
    {
      "product_sku": "WTR-001-PCS",
      "product_name": "Mineral Water",
      "quantity": 2,
      "unit_price": 4000,
      "line_total": 8000
    }
  ],
  "payment_info": null
}
```

`payment_info` berisi rekening/e-wallet toko dan total tagihan untuk order `transfer`/`delivery`.

## 19. Finances

Keuangan mencatat kategori dan transaksi manual (`income`/`expense`) di luar Point Flow dan sale. Semua endpoint membutuhkan permission `finances`, termasuk tiga laporan derivatif di bawah. Di Postman ketiganya ada di folder `Finances >`.

| Postman request | Endpoint |
| --- | --- |
| `Finances > List Financial Categories` | `GET {{base_url}}/financial-categories?search=&limit=50&offset=0` |
| `Finances > Create Financial Category` | `POST {{base_url}}/financial-categories` |
| `Finances > Get Financial Category` | `GET {{base_url}}/financial-categories/{{financial_category_id}}` |
| `Finances > Update Financial Category` | `PUT {{base_url}}/financial-categories/{{financial_category_id}}` |
| `Finances > Delete Financial Category` | `DELETE {{base_url}}/financial-categories/{{financial_category_id}}` |
| `Finances > List Financial Transactions` | `GET {{base_url}}/financial-transactions?search=&from=&to=&type=&status=&limit=50&offset=0` |
| `Finances > Create Financial Transaction` | `POST {{base_url}}/financial-transactions` |
| `Finances > Get Financial Transaction` | `GET {{base_url}}/financial-transactions/{{financial_transaction_id}}` |
| `Finances > Update Financial Transaction` | `PUT {{base_url}}/financial-transactions/{{financial_transaction_id}}` |
| `Finances > Cancel Financial Transaction` | `POST {{base_url}}/financial-transactions/{{financial_transaction_id}}/cancel` |
| `Finances > Income Statement` | `GET {{base_url}}/reports/income-statement?from=&to=` |
| `Finances > Cash Flow` | `GET {{base_url}}/reports/cash-flow?from=&to=` |
| `Finances > Financial Position` | `GET {{base_url}}/reports/financial-position?as_of=` |

Relasi: `category_type` dan `transaction_type` hanya menerima `income` atau `expense`. Kategori yang sudah dipakai transaksi tidak bisa dihapus. Transaksi `is_cash = false` bersifat non-tunai dan tidak masuk arus kas. Cancel transaksi memakai body `{"reason": "..."}` dan mengubah status menjadi `cancelled`. Update transaksi tidak bisa mengubah `transaction_type`.

Rumus arus kas (migration 018): `cash_in_sales` hanya menghitung penjualan selesai ber-payment_method `'cash'` (termasuk pesanan online *pay_at_store*); penjualan transfer/Midtrans tidak masuk arus kas tunai. Income statement dan financial position menghitung beban gaji hanya dari slip ber-`status = 'completed'`.

Contoh create category request:

```json
{
  "name": "Operasional Toko",
  "category_type": "expense",
  "description": "Belanja plastik, listrik, air",
  "is_active": true
}
```

Contoh create transaction request:

```json
{
  "transaction_type": "expense",
  "financial_category_id": 1,
  "amount": 50000,
  "is_cash": true,
  "description": "Belanja plastik"
}
```

Contoh income statement response (pendapatan net retur, HPP dari snapshot `unit_cost`, beban gaji + beban manual):

```json
{
  "gross_revenue": 8000,
  "return_amount": 0,
  "net_revenue": 8000,
  "cogs": 5000,
  "gross_profit": 3000,
  "payroll_expense": 0,
  "other_expense": 50000,
  "net_profit": -47000
}
```

Contoh cash flow response:

```json
{
  "cash_in_sales": 8000,
  "cash_in_manual": 0,
  "total_cash_in": 8000,
  "cash_out_drawer": 0,
  "cash_out_manual": 50000,
  "total_cash_out": 50000,
  "net_cash_flow": -42000
}
```

Contoh financial position response (kas kumulatif + nilai persediaan + laba kumulatif):

```json
{
  "as_of": "2026-05-18",
  "cash": 458000,
  "inventory_value": 250000,
  "total_assets": 708000,
  "cumulative_net_profit": -47000
}
```

## 20. Profit Sharing (SHU)

Periode bagi hasil (SHU) dihitung dari laba bersih income statement pada rentang periode. Semua endpoint membutuhkan permission `profit_sharing` (admin).

| Postman request | Endpoint |
| --- | --- |
| `Profit Sharing > List Periods` | `GET {{base_url}}/profit-sharing/periods?search=&status=draft,calculated&limit=50&offset=0` |
| `Profit Sharing > Create Period` | `POST {{base_url}}/profit-sharing/periods` |
| `Profit Sharing > Get Period Detail` | `GET {{base_url}}/profit-sharing/periods/{{profit_sharing_period_id}}` |
| `Profit Sharing > Update Period (Draft Only)` | `PUT {{base_url}}/profit-sharing/periods/{{profit_sharing_period_id}}` |
| `Profit Sharing > Calculate Period` | `POST {{base_url}}/profit-sharing/periods/{{profit_sharing_period_id}}/calculate` |
| `Profit Sharing > Finalize Period` | `POST {{base_url}}/profit-sharing/periods/{{profit_sharing_period_id}}/finalize` |
| `Profit Sharing > Cancel Period` | `POST {{base_url}}/profit-sharing/periods/{{profit_sharing_period_id}}/cancel` |
| `Profit Sharing > List Distributions` | `GET {{base_url}}/profit-sharing/periods/{{profit_sharing_period_id}}/distributions?search=&limit=50&offset=0` |
| `Profit Sharing > Export Distributions CSV` | `GET {{base_url}}/profit-sharing/periods/{{profit_sharing_period_id}}/distributions/export` |

Relasi lifecycle: `draft` → `calculate` → `calculated` → `finalize` → `finalized`, atau `cancel` (dengan reason) dari `draft`/`calculated`. Update hanya boleh saat `draft`. Calculate mengambil laba bersih otomatis dari income statement sebagai `auto_net_profit`, lalu membuat snapshot distribusi per member; pool yang dipakai adalah `total_pool` manual jika diisi, jika tidak maka laba otomatis. Porsi member dibagi antar member sebanding dengan nilai transaksi masing-masing pada rentang periode. Finalize mengunci periode. Export mengembalikan file `text/csv`, bukan envelope JSON.

Contoh create request:

```json
{
  "period_name": "Tahun Buku 2026",
  "start_date": "2026-01-01",
  "end_date": "2026-12-31",
  "member_pct": 50,
  "reserve_pct": 10,
  "management_pct": 15,
  "supervisor_pct": 10,
  "social_pct": 10,
  "education_pct": 5
}
```

Contoh detail response (field periode di-flatten, ditambah hasil kalkulasi):

```json
{
  "id": 1,
  "period_name": "Tahun Buku 2026",
  "start_date": "2026-01-01",
  "end_date": "2026-12-31",
  "status": "calculated",
  "total_pool": null,
  "auto_net_profit": 1200000,
  "member_pct": 50,
  "reserve_pct": 10,
  "management_pct": 15,
  "supervisor_pct": 10,
  "social_pct": 10,
  "education_pct": 5,
  "notes": null,
  "calculated_at": "2026-05-18T12:00:00Z",
  "finalized_at": null,
  "pool_used": 1200000,
  "member_pool": 600000,
  "member_count": 2,
  "total_member_transaction": 500000
}
```

## 21. Urutan Uji End-to-End di Postman

Urutan yang disarankan saat menjalankan collection:

1. `Health > Health Check`
2. `Auth > Bootstrap Admin` atau `Auth > Login`
3. `Auth > Refresh Token` saat `access_token` kedaluwarsa
4. `Auth > Me`
5. `Roles & Permissions > List Permissions` dan `Roles & Permissions > List Roles` jika perlu memeriksa RBAC.
6. `Cash Registers > Create Cash Register`
7. `Users > Create User` jika perlu akun cashier
8. `Members > Create Member` untuk akun storefront
9. `Product Categories > Create Product Category`
10. `Units > Create Base Unit (PCS)`
11. `Units > Create Package Unit (DUS)`
12. `Products > Create Product`
13. `Product SKUs > Create Retail Product SKU`
14. `Product SKUs > Create Package Product SKU`
15. `Suppliers > Create Supplier`
16. `Purchase Orders > Create Purchase Order`
17. `Purchase Orders > Order Purchase Order`
18. `Goods Receipts > Create Goods Receipt`
19. `Goods Receipts > Post Goods Receipt`
20. `Stock Adjustments > Create Stock Adjustment` jika butuh koreksi stok
21. `Stock Adjustments > Post Stock Adjustment`
22. `Stock Conversions > Create Stock Conversion` jika butuh pecah/gabung kemasan
23. `Stock Conversions > Post Stock Conversion`
24. `Online Orders > Update Store Settings` untuk info toko dan ongkir
25. `Public Storefront > List Public Products` untuk memastikan katalog tampil
26. `Member Auth > Member Login` sebagai member yang dibuat di langkah 8
27. `Public Storefront > Checkout (Guest)` tanpa header Authorization untuk order pickup
28. `Public Storefront > Checkout (Member)` dengan `Bearer {{member_access_token}}` untuk order delivery
29. `Online Orders > Pack Order`, lalu `Ready Order` (pickup) atau `Deliver Order` (delivery, setelah `Mark Order Paid` bila transfer)
30. `Online Orders > Complete Order` setelah order diterima
31. `Cash Registers > Open Cash Register Session`
32. `Cash Registers > Create Cash Drawer Cash In` jika perlu tambah modal laci
33. `Sales > Create Sale`
34. `Sale Returns > Get Returnable Sale Items` jika perlu refund
35. `Sale Returns > Create Sale Return Restock` atau `Sale Returns > Create Sale Return No Restock`
36. `Cash Registers > Create Cash Drawer Cash Drop`, `Create Cash Drawer Cash Out`, atau `Create Cash Drawer Petty Cash` jika ada uang keluar dari laci
37. `Cash Registers > Close Cash Register Session` setelah transaksi/refund/movement selesai
38. `Employees > Create Employee`
39. `Payroll Slips > Create Payroll Slip`
40. `Finances > Create Financial Category` lalu `Finances > Create Financial Transaction` untuk kas/beban manual
41. Jalankan `Reports > Dashboard` dan report lain untuk memeriksa hasil agregasi.
42. Jalankan `Finances > Income Statement`, `Finances > Cash Flow`, dan `Finances > Financial Position` untuk memeriksa laporan keuangan.
43. `Profit Sharing > Create Period`, `Profit Sharing > Calculate Period`, lalu periksa `Profit Sharing > Get Period Detail` dan `Profit Sharing > List Distributions`
44. `Profit Sharing > Finalize Period` setelah distribusi benar, atau `Profit Sharing > Cancel Period` jika periode salah
45. `Audit Logs > List Audit Logs` untuk memeriksa event audit yang sudah tercatat.
46. Opsional: `Auth > Change Password` untuk menguji ganti password akun aktif. Setelah berhasil, login ulang jika masih ingin menjalankan endpoint protected lain.
47. `Auth > Logout` untuk mencabut refresh token aktif.

Ringkasan relasi utama:

```text
roles
  -> users.role_id
  -> role_permissions.role_id

permissions
  -> role_permissions.permission_id

users
  -> roles.id through users.role_id
  -> refresh_tokens.user_id
  -> sales.cashier_id
  -> cash_register_sessions.opened_by / closed_by
  -> audit_logs.actor_id

audit_logs
  -> generic resource_type/resource_id for audited actions

refresh_tokens
  -> token_hash for refresh cookie validation
  -> revoked_at when rotated or logged out

cash_registers
  -> cash_register_sessions.cash_register_id

cash_register_sessions
  -> sales.cash_register_session_id
  -> sale_returns.cash_register_session_id
  -> cash_drawer_movements.cash_register_session_id
  -> closing expected_cash from completed sales total_amount minus completed sale_returns refund_amount plus cash drawer movement in minus movement out

product_categories
  -> products.category_id

products
  -> product_skus.product_id

units
  -> product_skus.unit_id

product_skus
  -> purchase_order_items.product_sku_id
  -> goods_receipt_items.product_sku_id
  -> stock_adjustment_items.product_sku_id
  -> stock_conversion_items.from_product_sku_id
  -> stock_conversion_items.to_product_sku_id
  -> sale_items.product_sku_id

suppliers
  -> purchase_orders.supplier_id

purchase_orders
  -> goods_receipts.purchase_order_id
  -> purchase_order_items.quantity_received

goods_receipts posted
  -> product_skus.stock increases
  -> stock_movements movement_type = purchase_receipt

stock_adjustments posted
  -> product_skus.stock becomes stock_after
  -> stock_movements movement_type = adjustment

stock_conversions posted
  -> product_skus.stock decreases for source SKU
  -> product_skus.stock increases for target SKU
  -> stock_movements movement_type = conversion_out and conversion_in

sales completed
  -> product_skus.stock decreases
  -> stock_movements movement_type = sale

sale_returns completed with restock
  -> product_skus.stock increases
  -> stock_movements movement_type = sale_return

sale_returns cancelled after restock
  -> product_skus.stock decreases
  -> stock_movements movement_type = correction

employees
  -> payroll_slips.employee_id

members
  -> member_refresh_tokens.member_id
  -> online_orders.member_id (nullable, dari JWT saat checkout member)
  -> profit_sharing_distributions.member_id (snapshot per periode)

member_refresh_tokens
  -> token_hash for member_refresh cookie validation
  -> revoked_at when rotated or logged out

online_orders
  -> online_order_items.online_order_id
  -> status flow: new -> packing -> ready -> completed (pickup)
  -> status flow: new -> packing -> delivering -> completed (delivery, harus lunas atau pay_at_store)
  -> cancel only from new/packing

financial_categories
  -> financial_transactions.financial_category_id

financial_transactions cancelled
  -> excluded from income statement, cash flow, and financial position

profit_sharing_periods
  -> profit_sharing_distributions.profit_sharing_period_id
  -> auto_net_profit from income statement at calculate time
  -> pool_used = total_pool manual if set, else auto_net_profit
```
