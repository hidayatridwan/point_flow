-- Reset demo — truncate semua tabel transaksi, pertahankan default data
-- ---------------------------------------------------------------------------
-- Untuk siklus demo: jalankan -> bootstrap admin -> seed -> demo -> reset lagi.
--
-- DIPERTAHANKAN (default data dari migrations, bukan data transaksi):
--   roles, permissions, role_permissions  (migration 004_add_rbac.sql)
--   units                                 (migration 001_initial_point_flow_schema.sql)
--   _point_flow_schema_migrations         (tabel penjejak migrasi workflow)
--
-- DI-TRUNCATE (data transaksi/demo; kemunculan "CASCADE" menangkap tabel baru
-- hasil migration yang mereferensi tabel-tabel ini, khususnya leaf tables):
--   users, refresh_tokens, employees, payroll_slips, suppliers,
--   product_categories, products, product_skus, sales(+items),
--   sale_returns(+items), cash_registers, cash_register_sessions,
--   cash_drawer_movements, purchase_orders(+items), goods_receipts(+items),
--   stock_adjustments(+items), stock_conversions(+items), stock_movements,
--   audit_logs
--
-- RESTART IDENTITY: sequence PK kembali ke 1 (data demo terlihat "baru").
--
-- CARA PAKAI:
--   psql "$SUPABASE_DATABASE_URL" --single-transaction -v ON_ERROR_STOP=1 \
--     -f backend/seeds/reset_demo.sql
--
-- Setelah reset, urutan siklus demo berikutnya:
--   1. POST /api/v1/auth/bootstrap-admin (users kosong lagi, endpoint aktif)
--   2. Seed: workflow Deploy to Render dengan apply_seeds=true, atau manual:
--      psql "$SUPABASE_DATABASE_URL" -v ON_ERROR_STOP=1 \
--        -f backend/seeds/dashboard_sample.sql
--
-- CATATAN: tidak menyentuh schema/struktur tabel — nilainya saja dihapus.
-- ---------------------------------------------------------------------------

BEGIN;

TRUNCATE TABLE
  users,
  refresh_tokens,
  employees,
  payroll_slips,
  suppliers,
  product_categories,
  products,
  product_skus,
  sales,
  sale_items,
  sale_returns,
  sale_return_items,
  cash_registers,
  cash_register_sessions,
  cash_drawer_movements,
  purchase_orders,
  purchase_order_items,
  goods_receipts,
  goods_receipt_items,
  stock_adjustments,
  stock_adjustment_items,
  stock_conversions,
  stock_conversion_items,
  stock_movements,
  audit_logs
RESTART IDENTITY CASCADE;

COMMIT;

-- Verifikasi: developer bisa memastikan sisa row hanya default data.
SELECT 'users'                          AS table_name, count(*) FROM users
UNION ALL SELECT 'roles (default)',            count(*) FROM roles
UNION ALL SELECT 'permissions (default)',      count(*) FROM permissions
UNION ALL SELECT 'role_permissions (default)', count(*) FROM role_permissions
UNION ALL SELECT 'units (default)',            count(*) FROM units
UNION ALL SELECT 'cash_registers',             count(*) FROM cash_registers
UNION ALL SELECT 'products',                   count(*) FROM products
UNION ALL SELECT 'sales',                      count(*) FROM sales
ORDER BY 1;
