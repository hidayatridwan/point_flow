# Harga Beli Terakhir + Margin Laba di Penerimaan Barang

Dokumen desain + rencana implementasi untuk fitur: **update otomatis harga beli
(`product_skus.purchase_price`) dari penerimaan barang, dan penentuan harga jual
(`selling_price`) lewat persentase laba (margin) per baris di form penerimaan.**

Status: siap diimplementasikan — checklist di Bagian 5 belum dikerjakan.

---

## 1. Latar belakang & masalah

Per saat ini (per 2026-11-10):

- `post_goods_receipt` (`backend/src/common/services/documents.rs`) hanya
  menambah stok (`product_skus.stock`), memperbarui
  `purchase_order_items.quantity_received`, dan mencatat mutasi stok
  `purchase_receipt`. Kolom `purchase_price` **tidak disentuh**.
- Satu-satunya tempat `purchase_price` berubah adalah edit manual di
  Master Data → Product SKUs (`backend/src/common/services/master.rs`).
- Akibatnya saat harga supplier naik:
  - HPP snapshot penjualan baru (`sale_items.unit_cost`, diambil dari
    `product_skus.purchase_price` saat sale dibuat) understated →
    **laba rugi inflated**.
  - Nilai Persediaan di Posisi Keuangan
    (`SUM(stock × purchase_price)` dari `v_stock_report`,
    `backend/src/common/services/reports.rs`) understated.

Metode saat ini adalah *last cost manual*; tidak ada weighted average/FIFO.
Fitur ini mengotomatiskan *last cost* dan menambahkan cara menetapkan harga jual
saat barang diterima.

## 2. Keputusan desain

| # | Keputusan | Pilihan | Alasan |
|---|---|---|---|
| D1 | Cakupan margin | **Per baris barang** (kolom di tiap item form) | Tiap barang boleh punya laba berbeda; sesuai permintaan "tambahin kolom persentase" |
| D2 | Penerimaan yang sudah diposting lalu dibatalkan | **Harga dibiarkan** (pembatalan tetap membalikkan stok sesuai perilaku existing) | Sederhana; harga master dipisahkan dari dokumen; konsisten prinsip "last cost terakhir yang valid" |
| D3 | Nilai margin | **Minimum 0** (tidak boleh negatif), maksimal 2 desimal | Konsisten dengan konvensi CHECK non-negative repo; jual di bawah modal tetap bisa lewat edit manual harga jual di master data |
| D4 | Waktu penerapan harga | **Saat posting** (bukan saat draft dibuat) | Konsisten dengan prinsip sistem: draft tidak berpengaruh apa-apa |
| D5 | Margin kosong (NULL) | **Harga jual tidak diubah** | Admin tetap bisa mengedit `selling_price` manual di master data (fitur lama tetap ada) |
| D6 | Tidak ada backfill data lama | Harga lama tetap sampai ada penerimaan baru | Menghindari perubahan historis di luar kendali pengguna |

## 3. Spesifikasi perilaku

1. **Posting penerimaan barang** (`POST /goods-receipts/{id}/post`), per item:
   - `product_skus.purchase_price` = `goods_receipt_items.unit_cost`
     (harga beli terakhir dari supplier — termasuk bila kasir meng-overwrite
     unit cost di form, dipakai nilai akhir yang diinput).
   - Bila `margin_percent` diisi: `product_skus.selling_price` =
     `ROUND(unit_cost × (1 + margin_percent/100), 2)`.
   - Bila `margin_percent` NULL: `selling_price` tetap (tidak berubah).
2. **Draft** tidak mengubah stok maupun harga (perilaku existing, tidak berubah).
3. **Pembatalan setelah posting**: stok dikembalikan (perilaku existing),
   `purchase_price` dan `selling_price` **dibiarkan** (D2).
4. Penerimaan parsial: tiap posting memakai `unit_cost` penerimaan itu; SKU
   selalu mengikuti **penerimaan terakhir yang diposting**.
5. Kolom `margin_percent` tersimpan di `goods_receipt_items` (audit trail) dan
   tampil di detail penerimaan.
6. `v_stock_report`, dashboard, dan semua laporan **tidak perlu diubah** —
   semuanya membaca `product_skus`, jadi otomatis mengikuti harga baru.
7. HPP penjualan: snapshot `sale_items.unit_cost` (existing) otomatis menjadi
   lebih akurat untuk penjualan setelah posting karena `purchase_price` kini
   selalu ter-update.

## 4. Detail teknis

### 4.1 Migrasi `backend/migrations/014_add_goods_receipt_margin.sql` (baru)

```sql
ALTER TABLE goods_receipt_items
    ADD COLUMN IF NOT EXISTS margin_percent NUMERIC(5, 2);

ALTER TABLE goods_receipt_items
    ADD CONSTRAINT goods_receipt_items_margin_percent_non_negative
    CHECK (margin_percent IS NULL OR margin_percent >= 0);
```

- Nullable: NULL = jangan sentuh `selling_price` (D5).
- `NUMERIC(5,2)` → margin 0–999.99%.
- Idempotent-style seperti migrasi lain di repo (`ADD COLUMN IF NOT EXISTS`).

### 4.2 Backend — `backend/src/common/models.rs`

- `GoodsReceiptItemInput`: tambah `pub margin_percent: Option<f64>`.
- `GoodsReceiptItemResponse`: tambah `pub margin_percent: Option<f64>`.
- Validasi di service: `validate_non_negative(margin, "margin_percent")`
  bila Some.

### 4.3 Backend — `backend/src/common/services/documents.rs`

`create_goods_receipt`:

- INSERT `goods_receipt_items` menambah kolom `margin_percent`
  (bind `item.margin_percent`).

`post_goods_receipt` — gabungkan update stok + harga dalam satu statement
per item (menggantikan `UPDATE product_skus SET stock = $2` yang ada):

```sql
UPDATE product_skus
SET stock = $2::numeric,
    purchase_price = $3::numeric,
    selling_price = COALESCE(
        ROUND($3::numeric * (1 + $4::numeric / 100), 2),
        selling_price
    )
WHERE id = $1
```

- `$4` NULL → `COALESCE(..., selling_price)` menjaga harga jual lama (D5).
- Pembulatan 2 desimal di SQL sesuai tipe `NUMERIC(14,2)`.

SELECT item penerimaan (`GOODS_RECEIPT_ITEM_SELECT` dan pemakaian serupa):
tambah `margin_percent` agar ikut ke response detail.

### 4.4 Frontend — `frontend/src/lib/api/goods-receipts.ts`

- `CreateGoodsReceiptItemPayload`: tambah `margin_percent?: number | null`.
- `GoodsReceiptItem`: tambah `margin_percent: number | null`.

### 4.5 Frontend — `goods-receipt-create-dialog.svelte`

- Tipe form item + field `margin_percent: string` (default `''`,
  opsional).
- Validasi zod per item: opsional; bila diisi angka ≥ 0 (maks 2 desimal).
- Grid item: `lg:grid-cols-4` → `lg:grid-cols-5`.
- Kolom baru per baris: input **"Margin Laba (%)"** + preview
  **"Harga jual baru: Rp X"** = `unit_cost × (1 + margin/100)`,
  terhitung live (`$derived`), dibulatkan 2 desimal, format `formatIdr`.
- Submit: kirim `margin_percent: item.margin_percent.trim() === '' ? null : Number(...)`.
- Preview hanya tampil bila margin terisi; unit cost diubah → preview ikut.

### 4.6 Frontend — `goods-receipt-detail-dialog.svelte`

- Tampilkan margin per item (mis. kolom/label "Margin" → `12%`) dan
  harga jual baru bila ada — sebagai jejak audit harga.

### 4.7 i18n — `frontend/src/lib/i18n/messages.ts`

Key baru di bawah `procurement.goodsReceipts.*` (ID + EN):

- `marginLabel` — "Margin Laba (%)" / "Profit Margin (%)"
- `marginPlaceholder` — "mis. 15" / "e.g. 15"
- `newSellingPrice` — "Harga jual baru" / "New selling price"

## 5. Checklist implementasi

Kerjakan berurutan; satu fase selesai → build/test → lanjut.

### Fase 1 — Migrasi `014_add_goods_receipt_margin.sql`

- [x] Buat migrasi sesuai 4.1 (kolom + CHECK non-negative)
- [x] Uji di scratch Postgres: migrasi naik bersih 001–014; INSERT margin
      negatif ditolak; NULL lolos
- [x] `cargo test` hijau

Validasi: ✅ migrasi naik bersih di scratch Postgres (2026-11-10, container
`postgres:16-alpine`: 001–014 berurutan, `ON_ERROR_STOP` tanpa error) + 3 uji:
kolom `margin_percent NUMERIC(5,2)` nullable terdaftar, constraint
`goods_receipt_items_margin_percent_non_negative` sesuai definisi, semantik
CHECK (bentuk identik): NULL/0/12.5/999.99 diterima, −0.01 ditolak
(`check_violation`). `cargo test` hijau (28 passed, 0 failed).
Catatan deviasi kecil: migrasi dibungkus `BEGIN; ... COMMIT;` — tidak ada di
snippet Bagian 4.1, tapi konsisten dengan gaya seluruh migrasi repo (001–013).
Gate review: diff `backend/migrations/014_add_goods_receipt_margin.sql`.

### Fase 2 — Backend service & model

- [x] `models.rs`: field input + response (4.2)
- [x] `documents.rs`: simpan margin saat create; update stok+harga saat
      post (4.3); SELECT item menyertakan `margin_percent`
- [x] Uji alur: PO → GR (dengan margin & tanpa margin) → post → cek
      `product_skus.purchase_price` = unit_cost dan `selling_price`
      sesuai rumus; GR tanpa margin → `selling_price` tetap
- [x] `cargo build` + `cargo test` hijau

Validasi: ✅ E2E di scratch DB (2026-11-10, server 8090 + Postgres 16,
migrasi 001–014): (1) GR unit_cost 10.000 + margin 15% → post →
`purchase_price` 10.000, `selling_price` 11.500, stok 0→10; (2) GR berikutnya
unit_cost 11.000 TANPA field `margin_percent` (backward compatible, response
`margin_percent: null`) → post → `purchase_price` 11.000, `selling_price` tetap
11.500, stok 15; (3) margin −5 ditolak API: "margin_percent must be greater
than or equal to 0"; (4) `margin_percent` ikut di response detail GR.
`cargo test` 28 passed, 0 failed. Catatan: `cancel_goods_receipt` hanya
menerima draft (`WHERE status = 'draft'`) sehingga D2 otomatis berlaku.
Gate review: diff `backend/src/common/services/documents.rs`.

### Fase 3 — Frontend form, detail, i18n

- [x] `goods-receipts.ts`: tipe payload + response (4.4)
- [x] Form penerimaan: kolom margin + preview harga jual (4.5)
- [x] Detail dialog: tampil margin (4.6)
- [x] `messages.ts`: key ID + EN (4.7)
- [x] Build frontend hijau (`bun run build` / `npm run build`)

Validasi: ✅ `svelte-check` 0 error/0 warning, `prettier --check` bersih,
`npm run build` sukses (adapter-static, site ter-tulis). Kolom margin hanya
tampil di form penerimaan & detail GR; preview "Harga jual baru" muncul live
saat margin terisi dan mengikuti perubahan unit cost.
Gate review: diff `goods-receipt-create-dialog.svelte`.

## 6. Rencana pengujian manual (setelah semua fase)

1. Buat PO harga beli Rp 10.000 → terima semua dengan margin 15% → posting.
   Hasil: `purchase_price` = 10.000, `selling_price` = 11.500.
2. Ulangi dengan margin **kosong** → `purchase_price` ter-update,
   `selling_price` tidak berubah.
3. Buat penjualan baru di kasir → `sale_items.unit_cost` = 10.000
   (HPP snapshot harga terakhir).
4. Cek Laporan Stok (nilai persediaan) dan Laba Rugi → memakai harga baru.
5. Buat penerimaan kedua (parsial) dengan harga beda (mis. 11.000, margin 10%)
   → posting → harga SKU ikut penerimaan terakhir.
6. Batalkan penerimaan yang sudah diposting → stok kembali, harga SKU
   **tidak** berubah (D2).
7. Detail penerimaan menampilkan margin yang diisi.
