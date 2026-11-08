# Roadmap: Storefront & Pesanan Online (MVP)

Desain acuan: [storefront-online-order.md](./storefront-online-order.md).
Cara pakai: kerjakan satu fase sampai selesai, lalu minta review sebelum lanjut.
Balas `go fase N` untuk mengeksekusi satu fase, atau sebut yang mau diubah.

## Fase 0 — Infrastruktur & fondasi

Tujuan: MinIO lokal jalan, nama variabel S3 dikunci di semua env.

- [x] Service `minio` + `minio-init` di `docker-compose.yml`
- [x] `S3_*` di `.env.example`, `.env.prod.example`, `.env.render.example`, `.env.render`
- [x] Teruskan `S3_*` ke backend di `docker-compose.yml` dan `docker-compose.prod.yml`
- [x] `docker compose up -d minio minio-init`, verifikasi bucket `point-flow-dev` bisa dibaca publik (roundtrip upload + GET anonim + init idempoten, 2026-11-07)
- [ ] Isi `S3_*` produksi (manual, saat deploy — Render dashboard / `.env.prod`)

Validasi: `docker compose config --quiet` (dev), buka `http://localhost:9001` (konsol MinIO).
Gate review: daftar variabel S3 final, kredensial prod sudah terisi atau eksplisit ditunda.

## Fase 1 — Migrasi `009_add_online_orders.sql`

Tujuan: skema pesanan online + fondasi pengaturan toko dan permission.

- [x] Tabel `online_orders` dan `online_order_items` (lihat Bagian 5.1–5.2)
- [x] Sequence harian nomor pesanan (`ONL-YYYYMMDD-NNNNNN`, 6 digit ikut konvensi repo)
- [x] Tabel `store_settings` + seed awal (nama toko, jam operasional, `delivery_fee = 0`)
- [x] Kolom `channel` + `online_order_id` di `sales`
- [x] Kolom `image_url` (object key S3) di `products`
- [x] Seed permission `online_orders` ke role yang sesuai
- [x] Uji di database kosong dan di salinan data produksi (2026-11-07, scratch Postgres: 001–009 berurutan + seed data lama + 4 uji negatif CHECK/UNIQUE)

Validasi: migrasi naik-turun bersih di kedua database, `cargo test` modul terkait tetap hijau.
Gate review: diff file migrasi 009 — [backend/migrations/009_add_online_orders.sql](/Users/ridwannhidayat/Workspace/labs/point_flow/backend/migrations/009_add_online_orders.sql).

## Fase 2 — Backend publik, upload file, rate limit

Tujuan: pembeli bisa melihat katalog dan checkout dari HP.

- [x] Modul `backend/src/modules/online_orders/` (routes, handler, service, repository, model, dto)
- [x] Route publik tanpa `AuthUser`: katalog, kategori, store-info, checkout
- [x] DTO publik khusus — tanpa `purchase_price`, supplier, data internal
- [x] Checkout: validasi stok + `quantity <= stock` ulang di server, harga dari `product_skus.selling_price`
- [x] Klien S3 dari env `S3_*` + endpoint upload terautentikasi (QRIS, foto produk)
- [x] `tower_governor` hanya di router publik (kuota longgar untuk baca, ketat untuk checkout)

Validasi (2026-11-07, semua lolos): `cargo test` (7 unit test), `cargo clippy --all-targets`
(bersih kecuali 2 warning pre-existing di kode lama), `curl` end-to-end di scratch DB+MinIO:
katalog tanpa harga beli + stok habis tampil, checkout pickup/transfer, 3 penolakan validasi,
burst checkout → 429 envelope `rate_limited`, upload foto + QRIS (termasuk 401/422/404 negatif),
`photo_url`/`qris_image_url` muncul di katalog/store-info.
Gate review: daftar endpoint publik + contoh respons katalog (pastikan tanpa harga beli).

## Fase 3 — Backend kasir

Tujuan: kasir bisa memproses pesanan sampai selesai dan tercatat sebagai penjualan.

- [x] Daftar + detail pesanan (filter status, paginasi), permission `online_orders`
- [x] Transisi `pack` (potong stok transaksional + kunci baris), `ready`, `deliver`, `complete`, `cancel` (kembalikan stok), `mark-paid`
- [x] `complete` wajib sesi register terbuka milik kasir yang menyelesaikan (keputusan A)
- [x] `delivery + transfer` wajib `paid` sebelum `delivering`; `pickup` fleksibel
- [x] `complete` membuat `sales` (`channel = 'online'`, `online_order_id`, `paid_amount = total`)

Validasi (2026-11-07, semua lolos): `cargo test` (10 unit test, termasuk 3 test state machine
murni), `cargo clippy` bersih (2 warning pre-existing), `curl` end-to-end (scratch DB):
pack potong stok 100→97 + movement tertaut pesanan, ready→complete buat penjualan channel
online tanpa potongan ganda + movement tertaut sale, deliver transfer-unpaid ditolak →
mark-paid → deliver lolos, complete tanpa sesi → 409 → buka sesi → lolos, cancel completed
ditolak, cancel-dari-packing kembalikan stok + movement koreksi + alasan tersimpan.
Deviasi tercatat: migrasi `010` menambah kolom nullable di `stock_movements` (lihat Bagian 5.3).
Gate review: matriks transisi status vs hasil aktual per peran kasir.

## Fase 4 — Frontend storefront (publik)

Tujuan: katalog → keranjang → checkout → sukses, mobile-first, tanpa login.

- [x] Route `(public)/shop`: catalog, cart (`localStorage`), checkout, success (+ layout terang Poppins, tanpa toggle tema)
- [x] Hanya Tailwind + bawaan Svelte — tanpa shadcn, TanStack, superforms, sonner, lucide
- [x] Validasi checkout manual sisi klien + penuh di server; semua teks via `t()` (id/en)
- [x] Produk stok habis tampil dengan penanda, tombol tambah nonaktif; grid masonry 2 kolom
- [x] Halaman sukses: nomor pesanan, info transfer (rekening/e-wallet/QRIS), tombol WA toko
- [x] `lib/api/storefront.ts` terpisah dari API admin; cart store `lib/cart.svelte.ts`

Validasi (2026-11-07): `bun run check` 0 error, eslint bersih, `bun run build` sukses,
serve dev: `/shop`, `/shop/cart`, `/shop/checkout`, `/shop/success/…` semua 200,
API store-info + checkout e2e lolos (termasuk `shop_whatsapp` ternormalisasi).
Visual mengacu mockup 2026-11-07 (aksen `#FF6534`, Poppins, kartu 2 kolom, stepper + CTA
pindah ke halaman keranjang karena tidak ada halaman detail).
Yang belum terverifikasi: interaksi nyata di browser (tambah cart, submit, salin nomor) —
perlu dibuka dan dicoba manual.
Gate review: buka `/shop` di HP, nilai kecepatan dan kemudahan checkout.

## Fase 5 — Dashboard kasir

Tujuan: pesanan online diproses dari tempat kasir bekerja setiap hari.

- [x] Menu Pesanan Online + badge jumlah `Baru`; daftar (tab status, cari, paginasi) + auto-refresh ringan
- [x] Detail: item, pembeli, tombol Chat WA (`wa.me`), satu tombol aksi utama per status + dialog konfirmasi
- [x] Perluasan halaman pengaturan toko: info pembayaran, upload QRIS, jam operasional, ongkir
- [x] Upload foto produk (ke S3) dari halaman produk
- [x] `lib/api/online-orders.ts` + kunci i18n id/en

Validasi (2026-11-07, semua lolos): `bun run check` 0 error, eslint bersih
pada file yang disentuh (`bun run lint` repo tetap merah di file lama yang
tidak tersentuh — baseline prettier kotor), `bun run build` sukses,
`cargo test` 13 lolos, `cargo clippy` bersih (2 warning pre-existing),
`curl` end-to-end di scratch DB (001–010) + MinIO sebagai kasir non-admin
berpermission `online_orders` — 25/25: bootstrap, daftar filter `new`,
detail, pack (stok 100→97), ready, complete-tanpa-sesi ditolak → buka sesi →
complete buat sale channel online, deliver transfer-unpaid ditolak →
mark-paid → deliver lolos, cancel-dari-packing kembalikan stok 90→95 +
alasan tersimpan, GET/PUT `/store-settings` (termasuk penolakan fee negatif
+ normalisasi WA `62…`), upload QRIS + foto produk (key + URL publik
muncul di settings/produk), halaman `/point_flow/online-orders`,
`/config/store-settings`, `/master/products`, `/shop` semua 200.
Deviasi tercatat: backend baru `GET/PUT /store-settings` (izin
`online_orders`) karena halaman pengaturan toko belum ada di repo;
`ProductResponse` + `image_url`/`photo_url` (key mentah + URL publik
dari S3) agar foto tampil di dashboard.
Gate review: coba proses satu pesanan fiktif dari Baru sampai Selesai
(di browser; interaksi klik belum dicoba manual — perlu dibuka dan diklik).

## Fase 6 — Validasi akhir & rilis

Tujuan: enam skenario Bagian 12 dokumen desain lolos, rilis ke satu toko.

- [ ] Skenario 1–6 Bagian 12 (stok habis, transfer, pack/cancel, complete, race stok, kebocoran harga beli)
- [ ] `cargo test`, `cargo clippy`, `bun run check`, `bun run lint` semua hijau
- [ ] Storefront & kasir di dua situs Netlify: storefront = situs `pointflow` (deploy Actions), kasir = `dash-pointflow`, env `SHOP_DOMAINS` (Functions/Production) di kedua situs, domain custom (primary) + `CORS_ALLOWED_ORIGINS` kedua origin (lihat README "Dua situs Netlify: storefront & kasir")
- [ ] Rilis ke satu toko, pantau pesanan masuk masa awal

Gate review: hasil tiap skenario + keputusan rilis.
