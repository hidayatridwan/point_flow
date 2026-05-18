Buatkan aplikasi MVP sistem kasir berbasis web yang sederhana, rapi, dan mudah dikembangkan.

Tujuan aplikasi:
Membantu toko kecil melakukan transaksi penjualan, mencatat pembelian barang, menerima barang masuk, melakukan stock adjustment, mengelola stok barang, dan membuat slip gaji karyawan.

Catatan naming:
- Label UI boleh memakai Bahasa Indonesia.
- Nama teknis database, enum, field, API payload, dan kode aplikasi memakai English identifiers.
- Istilah teknis penerimaan barang memakai `goods_receipts`.

Fitur utama MVP:

1. Authentication and User Management
- Login, refresh access token, dan logout.
- `access_token` berlaku 15 menit dan dikembalikan di response JSON untuk disimpan di memory client.
- `refresh_token` berlaku 30 hari, dibuat oleh backend, disimpan sebagai `HttpOnly`, `Secure`, `SameSite=Lax` cookie, dan tidak dikembalikan di response JSON.
- Nilai asli `refresh_token` tidak disimpan di database; database hanya menyimpan `token_hash`.
- Endpoint refresh melakukan token rotation: refresh token lama diberi `revoked_at`, lalu diganti dengan refresh token baru.
- Logout mencabut refresh token aktif dan menghapus cookie dari client melalui `Set-Cookie`.
- Role pengguna menggunakan enum `user_role`:
  - `admin`
  - `cashier`
- Admin bisa mengakses semua fitur.
- Cashier hanya bisa melakukan transaksi penjualan dan melihat riwayat transaksi miliknya.
- Field `users`:
  - `name`
  - `email`
  - `password_hash`
  - `role`
  - `is_active`
  - `last_login_at`
  - `created_at`, `updated_at`, `deleted_at`
- `email` harus unik untuk user yang belum soft deleted.
- Field `refresh_tokens`:
  - `user_id`
  - `token_hash`
  - `expires_at`
  - `revoked_at`
  - `replaced_by_token_id`
  - `created_at`
- `refresh_tokens.user_id` berelasi ke `users.id`.
- `refresh_tokens.token_hash` harus unik.

2. Dashboard
- Ringkasan penjualan hari ini.
- Total transaksi hari ini.
- Total pendapatan hari ini.
- Jumlah produk hampir habis.
- Jumlah `purchase_orders` dengan status `ordered` atau `partially_received`.
- Jumlah draft `stock_adjustments` yang belum diposting.
- Jumlah karyawan aktif.

3. Product Category and Product Management
- CRUD kategori produk menggunakan tabel `product_categories`.
- Field `product_categories`:
  - `name`
  - `description`
  - `is_active`
  - `created_at`, `updated_at`, `deleted_at`
- CRUD produk menggunakan tabel `products`.
- Field `products`:
  - `category_id`
  - `name`
  - `sku`
  - `purchase_price`
  - `selling_price`
  - `stock`
  - `minimum_stock`
  - `unit`, contoh: pcs, kg, liter
  - `is_active`
  - `created_at`, `updated_at`, `deleted_at`
- `sku` harus unik untuk produk yang belum soft deleted.
- Tampilkan peringatan jika `stock <= minimum_stock`.

4. Inventory Flow and Stock Movements
- `products.stock` adalah stok tersedia saat ini.
- Setiap perubahan stok final wajib dicatat di `stock_movements`.
- Field `stock_movements`:
  - `product_id`
  - `sale_id`, `sale_item_id` untuk stok keluar dari penjualan
  - `goods_receipt_id`, `goods_receipt_item_id` untuk stok masuk dari penerimaan barang
  - `stock_adjustment_id`, `stock_adjustment_item_id` untuk adjustment stok
  - `movement_type`: `stock_in`, `stock_out`, `sale`, `purchase_receipt`, `adjustment`, `correction`
  - `quantity`
  - `stock_before`
  - `stock_after`
  - `notes`
  - `created_by`, `created_at`
- `stock_movements.quantity` selalu bernilai positif; arah perubahan stok dibaca dari `stock_before` dan `stock_after`.
- Stok berkurang saat `sales.status = completed`.
- Stok bertambah saat `goods_receipts.status = posted`.
- Stok berubah sesuai `stock_adjustment_items.stock_after` saat `stock_adjustments.status = posted`.
- Dokumen dengan status `draft` atau `cancelled` tidak boleh mengubah stok.
- Dokumen yang sudah `posted` tidak boleh diedit; koreksi dilakukan dengan dokumen baru.

5. Supplier Management
- CRUD supplier menggunakan tabel `suppliers`.
- Field `suppliers`:
  - `name`
  - `contact_name`
  - `phone`
  - `email`
  - `address`
  - `is_active`
  - `created_at`, `updated_at`, `deleted_at`

6. Purchase Orders and Goods Receipts
- `purchase_orders` dibuat oleh Admin untuk memesan barang ke supplier.
- `goods_receipts` dibuat saat barang dari supplier diterima.
- Mendukung penerimaan parsial dari satu `purchase_order`.
- Field `purchase_orders`:
  - `po_no`
  - `supplier_id`
  - `order_date`
  - `expected_date`
  - `status`: `draft`, `ordered`, `partially_received`, `received`, `cancelled`
  - `subtotal_amount`
  - `notes`
  - `ordered_by`, `ordered_at`
  - `cancelled_at`, `cancelled_by`, `cancel_reason`
  - `created_by`, `created_at`, `updated_at`
- Field `purchase_order_items`:
  - `purchase_order_id`
  - `product_id`
  - `product_name`, `product_sku`, `unit` sebagai snapshot data produk
  - `quantity_ordered`
  - `quantity_received`
  - `unit_cost`
  - `line_total`
  - `created_at`, `updated_at`
- Field `goods_receipts`:
  - `gr_no`
  - `purchase_order_id`
  - `received_at`
  - `status`: `draft`, `posted`, `cancelled`
  - `received_by`
  - `posted_at`, `posted_by`
  - `cancelled_at`, `cancelled_by`, `cancel_reason`
  - `notes`
  - `created_at`, `updated_at`
- Field `goods_receipt_items`:
  - `goods_receipt_id`
  - `purchase_order_item_id`
  - `product_id`
  - `product_name`, `product_sku`, `unit` sebagai snapshot data produk
  - `quantity_received`
  - `unit_cost`
  - `line_total`
  - `created_at`, `updated_at`
- Validasi:
  - Produk yang diterima harus berasal dari item `purchase_orders`.
  - `goods_receipt_items.quantity_received` tidak boleh melebihi sisa `purchase_order_items.quantity_ordered - purchase_order_items.quantity_received`.
  - Saat `goods_receipts` diposting, aplikasi menambah `products.stock`, memperbarui `purchase_order_items.quantity_received`, memperbarui status `purchase_orders`, dan menulis `stock_movements`.

7. Stock Adjustments
- Admin bisa membuat adjustment stok untuk koreksi stok fisik, barang rusak, barang hilang, atau koreksi input.
- Adjustment dibuat sebagai dokumen `stock_adjustments` dengan item detail `stock_adjustment_items`.
- Field `stock_adjustments`:
  - `adjustment_no`
  - `adjusted_at`
  - `status`: `draft`, `posted`, `cancelled`
  - `reason`
  - `notes`
  - `created_by`
  - `posted_at`, `posted_by`
  - `cancelled_at`, `cancelled_by`, `cancel_reason`
  - `created_at`, `updated_at`
- Field `stock_adjustment_items`:
  - `stock_adjustment_id`
  - `product_id`
  - `product_name`, `product_sku`, `unit` sebagai snapshot data produk
  - `stock_before`
  - `stock_after`
  - `quantity_delta`
  - `notes`
  - `created_at`, `updated_at`
- Validasi:
  - `stock_after` tidak boleh negatif.
  - `stock_after` harus berbeda dari `stock_before`.
  - Satu produk hanya boleh muncul satu kali dalam satu dokumen adjustment.
  - Saat adjustment diposting, aplikasi mengubah `products.stock` menjadi `stock_after` dan menulis `stock_movements` dengan `movement_type = adjustment`.

8. Point of Sale
- Cari produk berdasarkan `name` atau `sku`.
- Tambahkan produk ke cart.
- Ubah jumlah item.
- Hapus item dari cart.
- Hitung subtotal otomatis.
- Hitung total pembayaran.
- Input jumlah uang dibayar.
- Hitung kembalian.
- Simpan transaksi.
- Cetak atau tampilkan struk sederhana.
- Field `sales`:
  - `transaction_no`
  - `sold_at`
  - `cashier_id`
  - `subtotal_amount`
  - `total_amount`
  - `paid_amount`
  - `change_amount`
  - `status`: `completed`, `cancelled`
  - `notes`
  - `cancelled_at`, `cancelled_by`, `cancel_reason`
  - `created_at`, `updated_at`
- Field `sale_items`:
  - `sale_id`
  - `product_id`
  - `product_name`, `product_sku`, `unit` sebagai snapshot data produk
  - `quantity`
  - `unit_price`
  - `line_total`
  - `created_at`
- Saat transaksi berhasil, aplikasi mengurangi `products.stock` dan menulis `stock_movements` dengan `movement_type = sale`.

9. Transaction History
- Lihat daftar transaksi.
- Detail transaksi.
- Filter berdasarkan tanggal.
- Admin bisa melihat semua transaksi.
- Cashier hanya bisa melihat transaksi miliknya.

10. Employee Management
- CRUD karyawan menggunakan tabel `employees`.
- Field `employees`:
  - `name`
  - `position`
  - `phone`
  - `address`
  - `base_salary`
  - `is_active`
  - `created_at`, `updated_at`, `deleted_at`

11. Payroll Slips
- Buat slip gaji per karyawan menggunakan tabel `payroll_slips`.
- Field `payroll_slips`:
  - `slip_no`
  - `employee_id`
  - `employee_name`
  - `employee_position`
  - `period_start`
  - `period_end`
  - `base_salary`
  - `bonus`
  - `deduction`
  - `net_salary`
  - `created_by`
  - `created_at`, `updated_at`
- Rumus:
  `net_salary = base_salary + bonus - deduction`
- Slip gaji bisa dilihat dan dicetak sederhana.

12. Reports
- Laporan penjualan harian dari `v_daily_sales_report`.
- Laporan penjualan bulanan dari `v_monthly_sales_report`.
- Laporan stok barang dari `v_stock_report`.
- Laporan purchase order dari `v_purchase_order_report`.
- Laporan goods receipt dari `v_goods_receipt_report`.
- Laporan stock adjustment dari `v_stock_adjustment_report`.
- Laporan slip gaji dari `v_payroll_report`.

Kebutuhan teknis:
- Gunakan desain UI yang bersih, modern, dan responsif.
- Gunakan struktur kode yang rapi dan mudah dikembangkan.
- Buat database schema yang jelas.
- Pastikan setiap fitur CRUD berjalan.
- Validasi input wajib ada.
- Tampilkan pesan sukses dan error.
- Fokus pada MVP dulu, jangan membuat fitur yang terlalu kompleks.
- `subtotal_amount`, `total_amount`, dan status dokumen dihitung dan dijaga oleh aplikasi.
- Generated columns seperti `line_total`, `quantity_delta`, dan `net_salary` dihitung oleh database.
- Soft delete menggunakan `deleted_at` untuk master data.
- Immutable posted document dijaga oleh aplikasi untuk MVP; trigger database bisa ditambahkan nanti jika diperlukan.

Hal yang tidak perlu dulu untuk MVP:
- Multi cabang.
- Barcode scanner fisik.
- Integrasi payment gateway.
- Akuntansi lengkap.
- Hutang supplier / pembayaran invoice pembelian.
- Absensi karyawan.
- Pajak otomatis.
- Export Excel/PDF kompleks.
- Integrasi printer thermal secara native.

Buatkan:
1. Struktur database.
2. Struktur halaman aplikasi.
3. Alur fitur utama.
4. Implementasi kode aplikasi.
5. Data dummy untuk testing.
6. Instruksi menjalankan aplikasi secara lokal.

Prioritaskan aplikasi yang bisa langsung digunakan untuk demo MVP.
