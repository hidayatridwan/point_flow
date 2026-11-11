## Goal

Menambah pembayaran non-tunai via Midtrans Snap (QRIS + Virtual Account bank + e-wallet) untuk dua alur: kasir POS dan online order storefront, dengan prinsip yang sudah disepakati: transaksi dibuat sebagai pending, stok baru berkurang saat webhook menyatakan lunas.

## Success Criteria

- Kasir POS dengan sesi register terbuka bisa membuat transaksi Midtrans, menampilkan QRIS/VA/e-wallet via Snap, dan stok (`product_skus`) hanya berkurang setelah status settlement/capture terverifikasi.
- Online order bisa checkout dengan `payment_method = midtrans`, menerima `snap_token`/`redirect_url`, dan status order berubah lunas hanya via webhook terverifikasi atau rekonsiliasi GET Status.
- Notifikasi Midtrans yang duplikat, out-of-order, atau bertanda tangan invalid tidak mengubah stok/saldo dua kali.
- `order_id` Midtrans unik dan tidak pernah dipakai ulang; nominal `gross_amount` integer sesuai subtotal item.
- Semua string UI baru memakai `t()` (id + en), tidak ada hardcode.
- Jalur non-tunai bersifat eksklusif mengikuti mode toko: `manual` → non-tunai manual, `midtrans` → non-tunai Midtrans. Tunai selalu tersedia (kasir tunai; `pay_at_store` di `/shop`).

## Approach

Buat modul backend baru `payments` (mengikuti struktur `routes/handler/service/repository/model/dto`) sebagai satu-satunya pemilik komunikasi Midtrans, lalu integrasikan tipis ke `sales` dan `online_orders`. Jangan menaruh kode Snap di dalam `documents.rs` atau `sales/service.rs` selain pemanggilan yang sudah ada.

Alur yang direkomendasikan:

1. Frontend minta pembuatan pembayaran → backend validasi stok/ketersediaan + hitung subtotal (tanpa mengurangi stok), generate `order_id` unik, panggil Midtrans Snap `POST /transactions`, simpan baris pending di tabel baru, kembalikan `snap_token` + `redirect_url` + `expires_at`.
2. Pelanggan bayar di Snap → Midtrans POST ke webhook publik kita → verifikasi `signature_key`, update baris pembayaran secara idempoten, dan hanya pada transisi pertama ke `settlement`/`capture (ACCEPT)` jalankan transaksi DB yang (a) untuk POS: membuat row `sales` + mengurangi stok lewat logika `create_sale` yang direfaktor, (b) untuk online: menandai order lunas + mengurangi stok lewat jalur yang sama dengan fulfilment online.
3. Kedaluwarsa (`expire`) dan kegagalan (`cancel/deny/failure`) hanya menandai pembayaran terminal tanpa menyentuh stok; polling status manual tersedia untuk kasir saat webhook telat.

Alternatif yang ditolak: mengurangi stok saat transaksi pending dibuat (melanggar keputusan diskusi dan mengotori `stock_movements`), serta menaruh webhook di belakang JWT (Midtrans tidak bisa mengirim `Authorization` Bearer kita).

### Flag mode pembayaran toko (manual vs Midtrans)

Konteks saat ini: checkout publik `/shop` mengambil info pembayaran manual (`bank_accounts`, `ewallet_numbers`, `qris_image_url`) dari pengaturan toko (`store_settings` key-value). Desain teknisnya:

- Satu key baru di `store_settings`: `payment_mode` dengan nilai `manual` | `midtrans`, default `manual` (backward-compatible: toko yang sudah jalan tidak berubah perilaku).
- Eksklusivitas dijamin oleh bentuk datanya (satu field bernilai tunggal), bukan dua boolean — jadi tidak ada state invalid "keduanya aktif".
- Backend interface:
  - `UpdateStoreSettingsRequest.payment_mode: Option<String>` + `ValidatedStoreSettings.payment_mode: Option<String>`; `validate_store_settings` menolak nilai selain `manual`/`midtrans`.
  - `StoreInfoResponse.payment_mode: String` — dibaca publik via `get_store_info`/`getPublicStoreInfo`, sehingga halaman checkout `/shop` tahu mode aktif tanpa endpoint baru.
  - `validate_checkout`: bila mode `manual`, terima `pay_at_store`/`transfer` seperti sekarang; bila mode `midtrans`, terima hanya `midtrans` dan tolak dua nilai lama dengan error validasi yang jelas. Request lintas-mode (mis. kirim `transfer` saat mode `midtrans`) ditolak server, tidak hanya disembunyikan di UI.
  - Data manual lama (`bank_accounts`, dst.) tetap tersimpan saat mode `midtrans` (tidak dihapus) agar bisa kembali ke `manual` tanpa isi ulang.
- Frontend interface:
  - Halaman pengaturan toko (`frontend/src/routes/(protected)/config/store-settings/+page.svelte`): kontrol tunggal SegmentedControl/RadioGroup "Mode pembayaran" berisi dua opsi (Manual / Midtrans Snap). Pilih satu otomatis menonaktifkan yang lain. Saat `midtrans` aktif, field manual (`bank_accounts`, `ewallet_numbers`, upload QRIS) ditampilkan disabled dengan pesan "tidak dipakai pada mode Midtrans".
  - Halaman checkout `/shop/checkout`: bila `payment_mode == manual`, tampilkan dua kartu seperti sekarang (`pay_at_store`/`transfer` + info rekening bila transfer). Bila `midtrans`, tampilkan satu opsi "Bayar online (QRIS/VA/e-wallet)" yang memanggil Snap + tombol "Cek status". Tipe `Payment` diperluas menjadi `'pay_at_store' | 'transfer' | 'midtrans'` dengan penjagaan: nilai awal dan opsi yang dirender mengikuti `storeInfo.payment_mode`.
  - i18n: key baru di `config.storeSettings.*` (label mode + opsi + hint) dan `shop.*` (label bayar online), id + en.

### Matriks perilaku per mode

| Jalur | Mode `manual` | Mode `midtrans` |
|---|---|---|
| `/shop` ambil di toko (`pay_at_store`) | Ada (seperti sekarang) | Ada (tetap — bayar fisik di toko) |
| `/shop` transfer | `transfer` manual + `payment_info` (rekening/e-wallet/QRIS) | `midtrans` via Snap (`snap_token`/`redirect_url`); `transfer` manual disembunyikan dan ditolak server |
| Kasir tunai | Ada (alur `CreateSaleRequest` sekarang) | Ada (tidak berubah) |
| Kasir non-tunai | Pencatatan manual (kasir verifikasi bukti transfer, sale langsung lunas) | Snap pending → sale + kurang stok saat settlement via webhook |

Aturan server-side: `validate_checkout` (online) dan validasi kasir membaca `payment_mode` dari `store_settings`; request non-tunai yang tidak sesuai mode aktif ditolak dengan error validasi, tidak hanya disembunyikan di UI.

## Steps

1. Konfigurasi + migrasi DB.
   - Tambah env di repo root (`.env.example`): `MIDTRANS_SERVER_KEY`, `MIDTRANS_CLIENT_KEY`, `MIDTRANS_SIGNATURE_KEY`, `MIDTRANS_IS_PRODUCTION=false`, `MIDTRANS_SNAP_BASE_URL`, `MIDTRANS_NOTIFICATION_URL`, `MIDTRANS_PAYMENT_EXPIRY_MINUTES`. Backend baca via `backend/src/config/mod.rs` seperti env lain. `MIDTRANS_SIGNATURE_KEY` = Signature Secret Key dashboard — wajib untuk verifikasi webhook karena sejak 28 Okt 2026 Midtrans menandatangani notifikasi dengan kunci ini, bukan Server Key (kosong = fallback ke Server Key/skema lama).
   - Migrasi `015_add_midtrans_payments.sql` (014 sudah dipakai `014_add_goods_receipt_margin.sql`): (a) `INSERT INTO store_settings (key, value) VALUES ('payment_mode', 'manual') ON CONFLICT DO NOTHING`; (b) tabel `midtrans_payments` (`order_id UNIQUE`, `source_type pos|online`, `gross_amount INTEGER`, `snap_token`, `redirect_url`, `transaction_status`, `fraud_status`, `payment_type`, `va_info JSONB`, `raw_notification JSONB`, `sale_id NULL FK`, `online_order_id NULL FK`, `expires_at`, `settled_at`, `created_by`), index di `order_id` + `transaction_status`. Tanpa mengubah skema `sales`; relasi dari sisi payments saja.
   - `order_id` format `PF-{POS|ONL}-{uuid8}` hanya berisi `[A-Za-z0-9-_.~]`, maks 50 char.
2. Modul backend `payments` (baru).
   - Surface: `backend/src/modules/payments/{routes,handler,service,repository,model,dto}.rs`, daftarkan di `backend/src/http/mod.rs` + `backend/src/modules/mod.rs`.
   - `service`: klien Snap minimal dengan `reqwest` (POST Snap transactions dengan Basic Auth server key, `transaction_details`, `item_details`, `customer_details`, `expiry`, `enabled_payments: [qris, bank_transfer, e-wallet]`), verifikasi `SHA512(order_id + status_code + gross_amount + server_key)`, fungsi idempoten `apply_notification`, dan `get_status` (challenge ke Midtrans) untuk rekonsiliasi.
   - Reuse: `cash_registers::ensure_open_cash_register_session`, validasi member aktif, `load_product_sku_snapshot`, penulis audit, envelope `ApiResponse`.
   - Endpoint: `POST /payments/midtrans/charges` (auth, buat pending), `GET /payments/midtrans/{order_id}` (auth, polling kasir), `POST /payments/midtrans/notification` (publik, tanpa JWT, verifikasi signature dulu).
3. Refaktor kecil `documents.rs` agar bisa dipakai dua kali.
   - Pisahkan fungsi internal `create_sale_tx(tx, ...)` dari `create_sale` agar webhook settlement bisa membuat sale + kurangi stok dalam transaksi yang sama dengan update status pembayaran. Tidak ada perubahan perilaku tunai.
4. Alur POS (tunai selalu ada, non-tunai ikut mode).
   - Tunai: `CreateSaleRequest` tidak berubah.
   - Non-tunai mode `manual`: tambah `payment_method: transfer` + `payment_reference` (tujuan transfer + no. bukti) pada pembuatan sale; kasir yang verifikasi manual, sale langsung lunas dan stok langsung kurang seperti tunai.
   - Non-tunai mode `midtrans`: request terpisah `CreateMidtransChargeRequest { cash_register_session_id, member_id, notes, items }` (tanpa `paid_amount`); pending dulu, sale + kurang stok saat settlement via webhook.
   - Halaman kasir membaca `payment_mode` dan menampilkan tombol non-tunai sesuai mode (Manual transfer / Midtrans Snap); request non-tunai lintas-mode ditolak server.
   - Tetap wajib sesi register terbuka (prasyarat kasir tidak berubah).
   - Fraksi kuantitas (`quantity f64`) vs `item_details` Midtrans yang integer: bulatkan ke satuan terkecil per-SKU saat kirim ke Midtrans, tapi subtotal internal tetap `quantity * selling_price`; selisih pembulatan didokumentasikan sebagai diskon/baris penyesuai bila perlu.
5. Alur online order (tunduk pada flag `payment_mode`).
   - Tambah `payment_method = midtrans` di `validate_checkout` (`backend/src/modules/online_orders/service.rs`), simpan `midtrans_order_id` di sisi payments (tanpa migrasi besar ke `online_orders` bila bisa dihindari). Aturan per mode: mode `manual` → terima `pay_at_store`/`transfer` (tolak `midtrans`); mode `midtrans` → terima `pay_at_store`/`midtrans` (tolak `transfer`). `payment_info` manual hanya dikembalikan saat mode `manual` + metode `transfer`; saat metode `midtrans`, respons checkout mengembalikan `snap_token`/`redirect_url`, bukan `payment_info`.
   - `checkout` membaca `payment_mode` dari `store_settings` di awal dan meneruskannya ke validasi agar penolakan lintas-mode terjadi server-side.
6. Webhook hardening.
   - Verifikasi signature → cari baris by `order_id` → abaikan jika status sudah terminal atau notifikasi lebih lama dari status tersimpan (out-of-order) → update + Settlement hanya sekali (guard `settled_at IS NULL`) → balas 200 cepat (<5s), tanpa kerja berat di luar DB tx.
   - Terima `settlement|capture+ACCEPT` sebagai lunas; `pending` hanya update tampilan; `expire|cancel|deny|failure` terminal tanpa stok.
7. Frontend.
   - `frontend/src/lib/api/payments.ts`: `createMidtransCharge`, `getMidtransPayment`, `createMidtransOnlineCheckout`.
   - POS: tombol non-tunai mengikuti mode — mode `manual`: form transfer manual (tujuan + no. bukti); mode `midtrans`: tombol QRIS/VA/e-wallet → muat `snap.js` (sandbox vs prod via env) → `snap.pay(snap_token)` → polling status tiap ~5 detik sampai `settlement`/`expire` → tampilkan sukses/gagal via `svelte-sonner`. Tunai tidak berubah.
   - Online checkout + halaman status order: opsi yang dirender mengikuti `payment_mode` (manual: dua kartu seperti sekarang; midtrans: `pay_at_store` + tombol "Bayar online"), plus tombol "Cek status" (rekonsiliasi manual) saat metode `midtrans`.
   - Semua label via `t()` di `frontend/src/lib/i18n/messages.ts` (id + en); state dengan runes (`$state`/`$derived`), error via `ErrorAlert`, loading via pola `onMount` yang sudah ada.
8. Laporan, audit, RBAC.
   - Penjualan non-tunai yang sudah lunas (settlement Midtrans maupun transfer manual) masuk ke dashboard/laporan penjualan seperti tunai (karena row `sales` baru dibuat saat lunas); laporan memisahkan omzet tunai vs non-tunai; pembayaran pending tidak masuk omzet.
   - Tulis audit log untuk charge, settlement, expire; permission mengikuti peran kasir yang sudah ada.

## Validation Plan

- Backend: `cargo fmt --manifest-path backend/Cargo.toml`, `cargo clippy --manifest-path backend/Cargo.toml --all-targets`, `cargo test --manifest-path backend/Cargo.toml` (tambah unit test: validasi signature, idempotensi `apply_notification`, penolakan notifikasi out-of-order, `validate_checkout` menerima `midtrans`).
- Frontend: `bun run check`, `bun run lint`, `bun run build` dari `frontend/`.
- E2E sandbox (langkah risiko tertinggi): jalankan `docker compose up -d postgres`, `cargo run --manifest-path backend/Cargo.toml --bin server`, `bun run dev`; buat charge QRIS sandbox → bayar via simulator Midtrans → pastikan webhook (via HTTPS publik sementara, mis. tunnel) mengubah status ke settlement, row `sales` muncul, dan stok berkurang tepat sekali; ulangi notifikasi yang sama untuk membuktikan idempoten; uji expire dengan expiry 1 menit.

## Risks / Open Questions (keputusan sudah dikunci)

- Webhook wajib HTTPS publik — **diputuskan**: E2E lokal memakai Cloudflare tunnel (`cloudflared` CLI sudah terinstall); `MIDTRANS_NOTIFICATION_URL` menunjuk ke URL tunnel saat pengujian.
- `gross_amount` Midtrans integer sedangkan harga internal `f64` dan kuantitas bisa fraksional — **diputuskan**: bulatkan per-SKU ke rupiah terdekat dengan aturan fraksi desimal lebih dari 5 ke atas, 5 ke bawah ke bawah; selisih pembulatan didokumentasikan sebagai diskon/baris penyesuai bila perlu agar nominal Snap == subtotal DB.
- Sesi register bisa ditutup saat pembayaran pending masih hidup — **diputuskan**: settlement membuat sale pada sesi register yang sedang terbuka saat settlement terjadi (sesi baru), bukan sesi asal yang sudah tutup.
- **Disetujui default**: expiry 60 menit; `enabled_payments` awal = `qris, bank_transfer, gopay, shopeepay, dana, ovo`. Akun/keys sandbox disiapkan sebelum implementasi fase 2; jadwal beralih ke production ditentukan kemudian.
- **Diputuskan**: pencatatan non-tunai manual di kasir memakai `payment_reference` berisi tujuan transfer + no. bukti; laporan memisahkan omzet tunai vs non-tunai.

## Sources

- Signature + praktik notifikasi: [Handle Notifications](https://docs.midtrans.com/reference/handle-notifications)
- Objek Snap (`transaction_details`, `item_details`, expiry): [JSON Objects](https://docs.midtrans.com/reference/json-objects)
