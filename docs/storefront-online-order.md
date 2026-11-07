# Rencana Fitur: Storefront Publik & Pesanan Online (MVP)

| Item | Isi |
| --- | --- |
| Status | Rencana — keputusan kunci dikunci, siap implementasi |
| Tanggal | 2026-11-05 (keputusan kunci butir 5–9 dikunci 2026-11-07) |
| Cakupan | Pembelian produk oleh end user lewat browser HP, diproses kasir dari dashboard Point Flow yang sudah ada |
| Prinsip | Sesederhana mungkin. Satu toko. Satu repo. Tanpa akun pembeli. |

## 1. Keputusan yang Sudah Disepakati

1. **Identitas pembeli**: cukup nama + nomor WhatsApp. Tanpa registrasi, tanpa login, tanpa verifikasi OTP pada MVP.
2. **Pembayaran**: bayar di tempat (saat ambil / saat barang diantar) dan transfer manual. Halaman pembayaran menampilkan nomor rekening, nomor e-wallet, dan gambar QRIS statis milik toko. Tanpa payment gateway pada MVP.
3. **Pengantaran**: dua pilihan — ambil sendiri di toko, atau diantar oleh pihak toko. Untuk MVP, area antar mengikuti jangkauan layanan yang ditetapkan toko.
4. **Jumlah toko**: satu toko. Tidak ada konsep cabang, multi-outlet, atau pilihan toko.
5. **Penyimpanan file (foto produk, gambar QRIS)**: object storage S3-compatible. Lokal memakai MinIO lewat docker compose; produksi memakai layanan S3-compatible apa pun, kredensial diisi manual via env (`S3_*`, lihat Bagian 5.4). Database hanya menyimpan object key; URL publik disusun dari `S3_PUBLIC_URL` + key. Browser tidak pernah menerima secret key.
6. **Sesi register saat penyelesaian pesanan (opsi A)**: `complete` (pembuatan penjualan) wajib dilakukan saat ada sesi register terbuka — kasir yang menyelesaikan memakai sesi terbukanya sendiri (`cashier_id` + `cash_register_session_id` miliknya). Packing (potong stok, tanpa pergerakan uang) boleh tanpa sesi terbuka. Pesanan yang masuk di luar jam buka menunggu sebagai `Baru`/`Dipacking` sampai kasir memproses saat toko buka.
7. **Pengaturan toko**: info pembayaran, jam operasional, dan ongkir tetap dikelola di halaman pengaturan toko yang sudah ada (bukan halaman baru). Upload gambar QRIS dan foto produk lewat dashboard, diteruskan backend ke S3 (browser tidak upload langsung ke S3).
8. **Rate limit**: memakai `tower_governor` (`GovernorLayer`) yang dipasang hanya pada router publik, per IP, dengan kuota terpisah untuk katalog (longgar) dan checkout (ketat). Detail di Bagian 10.
9. **Permission RBAC baru**: `online_orders` (bukan menumpang `point_flow`). Endpoint kasir mensyaratkan permission ini; di-seed ke role `admin` dan `cashier` (atau setara) saat migrasi.

Alur utama pada dokumen ini sudah disetujui pemilik produk: katalog publik → checkout → pesanan masuk dashboard kasir → kasir packing → serah terima (diambil / diantar).

## 2. Tujuan dan Batasan MVP

### Tujuan

- Pelanggan bisa melihat produk yang dijual toko dan memesan dari HP tanpa datang dan mengantre di kasir.
- Kasir menerima semua pesanan online di satu tempat (dashboard Point Flow yang sudah dipakai setiap hari), lalu menyiapkan dan menyerahkan barang.
- Stok dan laporan penjualan tetap satu sumber kebenaran: data Point Flow yang sudah ada.

### Bukan tujuan MVP (fase lanjutan, lihat Bagian 13)

- Akun pembeli, riwayat pesanan per akun, dan OTP.
- Pembayaran online otomatis (QRIS dinamis / virtual account via payment gateway).
- Integrasi kurir pihak ketiga dan perhitungan ongkir otomatis per jarak.
- Notifikasi otomatis ke pembeli (WhatsApp gateway, push, email).
- Voucher, promo, poin, dan program loyalitas.

## 3. Rekomendasi Bentuk Sistem: Tetap Satu Repo

**Tidak membuat repo baru.** Storefront ditambahkan ke repo Point Flow yang sekarang sebagai modul dan route baru.

Alasan:

- Backend Rust/Axum saat ini adalah sumber kebenaran untuk produk, harga jual, dan stok (`product_skus`). Repo terpisah dengan backend sendiri akan memaksa sinkronisasi stok dan harga — sumber bug klasik (stok di web beda dengan stok di toko).
- Pola repo sudah modular: setiap domain punya folder sendiri di `backend/src/modules/` dan halaman sendiri di `frontend/src/routes/(protected)/`. Fitur ini tinggal mengikuti pola yang sama.
- Satu proses deploy, satu database, satu backup. Operasional toko kecil tidak perlu infrastruktur tambahan.

Pemisahan yang dilakukan hanya di level akses:

| Area | Sifat | Contoh jalur |
| --- | --- | --- |
| Storefront pembeli | Publik, tanpa login | Halaman web `/shop`, API `GET /api/v1/public/...` |
| Dashboard kasir/admin | Terlindungi JWT + RBAC seperti sekarang | Halaman `(protected)`, API yang sudah ada + endpoint pesanan online khusus kasir |

Nama domain bebas dipilih saat implementasi (mis. `point_flow.<domain>` tetap untuk kasir dan storefront di path `/shop` pada domain yang sama). Keputusan domain tidak mengubah desain di dokumen ini.

## 4. Alur Pengguna

### 4.1 Alur pembeli (semua dari browser HP)

1. Pembeli membuka halaman storefront. Tampil daftar produk yang aktif — termasuk yang stoknya sedang habis — dikelompokkan per kategori, lengkap dengan foto (bila ada), nama, satuan, dan harga jual. Produk yang stoknya habis tetap terlihat dengan penanda “Stok habis” agar pembeli tahu produk itu pernah/selalu dijual dan bisa menanyakan restok.
2. Pembeli mencari produk (kolom pencarian) dan mengetuk tombol tambah untuk memasukkan ke keranjang. Tombol tambah pada produk stok habis dinonaktifkan sehingga tidak bisa dimasukkan ke keranjang. Jumlah bisa diubah selama tidak melebihi stok yang tampil.
3. Pembeli membuka keranjang, memeriksa isi dan total belanja, lalu mengetuk tombol pesan.
4. Pembeli mengisi formulir checkout satu halaman:
   - Nama
   - Nomor WhatsApp (dipakai kasir untuk konfirmasi dan koordinasi antar)
   - Metode serah terima: `Ambil di toko` atau `Diantar`
   - Bila diantar: alamat lengkap pengantaran (jalan/nomor, patokan bila perlu) dan catatan bila perlu
   - Metode pembayaran: `Bayar di tempat` atau `Transfer`
5. Pembeli mengirim pesanan. Sistem membalas dengan **nomor pesanan** (mis. `ONL-20261105-000001`) dan ringkasan pesanan.
   - Bila memilih transfer, halaman sukses menampilkan nomor rekening, nomor e-wallet, dan gambar QRIS beserta total yang harus dibayar.
   - Bila memilih bayar di tempat, halaman sukses menampilkan nomor pesanan dan perkiraan langkah berikutnya (menunggu diproses kasir).
6. Pembeli cukup menyimpan nomor pesanan / halaman sukses. Tidak ada akun dan tidak ada halaman lacak status pada MVP; konfirmasi lanjutan dilakukan kasir lewat WhatsApp secara manual.

### 4.2 Alur kasir (dashboard Point Flow)

1. Pesanan baru muncul di menu baru **Pesanan Online** dengan penanda jumlah pesanan berstatus `Baru`.
2. Kasir membuka detail pesanan: isi keranjang, nama + nomor WA pembeli, metode serah terima, alamat antar (bila ada), metode pembayaran, dan total.
3. Kasir mengetuk **Terima & Packing**. Status menjadi `Dipacking`. Pada titik ini stok resmi terpotong (lihat Bagian 7).
   - Bila stok ternyata tidak cukup, kasir membatalkan item/pesanan dan menghubungi pembeli via WhatsApp. Tombol hubungi pembeli membuka `wa.me/<nomor>` dengan teks awal berisi nomor pesanan (kenyamanan kasir, bukan notifikasi otomatis).
4. Setelah barang siap:
   - Ambil di toko: status menjadi `Siap Diambil`, lalu `Selesai` setelah pembeli mengambil dan membayar (bila bayar di tempat).
   - Diantar: status menjadi `Diantar`, lalu `Selesai` setelah karyawan kembali / pembayaran diterima.
5. Pesanan berstatus `Selesai` otomatis tercatat sebagai penjualan pada laporan, dengan penanda channel online (lihat Bagian 7). Pesanan `Dibatalkan` tidak tercatat sebagai penjualan dan tidak memotong stok (atau stok dikembalikan bila sudah terpotong — lihat Bagian 6).

### 4.3 Diagram status pesanan

```text
                 ┌────────────┐
   Checkout ───► │    Baru    │
                 └─────┬──────┘
              Terima & │ Packing          Batalkan (kasir / pembeli via WA)
                 ┌─────▼──────┐      ┌─────────────┐
                 │  Dipacking ├─────►│  Dibatalkan │
                 └─────┬──────┘      └─────────────┘
            ┌──────────┴──────────┐
     Ambil di toko            Diantar
     ┌──────▼───────┐      ┌─────▼─────┐
     │ Siap Diambil │      │  Diantar  │
     └──────┬───────┘      └─────┬─────┘
            └──────────┬─────────┘
                 ┌─────▼─────┐
                 │  Selesai  │ ──► tercatat sebagai penjualan (channel: online)
                 └───────────┘
```

Pembatalan hanya tersedia pada status `Baru` dan `Dipacking`. Setelah `Diantar` / `Siap Diambil`, pembatalan ditangani manual di luar sistem (retur mengikuti flow retur penjualan yang sudah ada bila barang sudah tercatat sebagai penjualan).

## 5. Rancangan Data

Prinsip: data master (produk, SKU, harga, stok) tidak diduplikasi. Storefront hanya membaca data yang sudah ada dan menulis ke tabel pesanan baru.

### 5.1 Tabel baru `online_orders` (usulan migrasi `009_add_online_orders.sql`)

| Kolom | Tipe | Keterangan |
| --- | --- | --- |
| `id` | `BIGINT IDENTITY PK` | |
| `order_no` | `VARCHAR(40) UNIQUE` | Nomor pesanan untuk pembeli, mis. `ONL-20261105-000001` (sequence harian terpisah dari `sales_transaction_no_seq`) |
| `customer_name` | `VARCHAR(120)` | Nama pembeli dari checkout |
| `customer_phone` | `VARCHAR(30)` | Nomor WhatsApp, disimpan apa adanya (format bebas, validasi ringan) |
| `fulfillment_method` | `pickup \| delivery` | Ambil di toko / diantar karyawan |
| `delivery_address` | `TEXT NULL` | Wajib bila `delivery`, kosong bila `pickup` |
| `payment_method` | `pay_at_store \| transfer` | Bayar di tempat / transfer manual |
| `payment_status` | `unpaid \| paid` | Diubah manual oleh kasir setelah pembayaran diverifikasi/diterima |
| `status` | `new \| packing \| ready \| delivering \| completed \| cancelled` | Siklus pada Bagian 4.3 |
| `subtotal_amount` | `NUMERIC(14,2)` | Dihitung server dari harga database saat checkout |
| `delivery_fee` | `NUMERIC(14,2) DEFAULT 0` | MVP: gratis atau nominal tetap dari pengaturan (Bagian 8), tidak dihitung per jarak |
| `total_amount` | `NUMERIC(14,2)` | `subtotal_amount + delivery_fee` |
| `notes` | `TEXT NULL` | Catatan pembeli |
| `sale_id` | `BIGINT NULL → sales(id)` | Diisi saat pesanan `completed` dan penjualan dibuat (Bagian 7) |
| `processed_by` | `BIGINT NULL → users(id)` | Kasir yang memproses |
| `cancel_reason` | `TEXT NULL` | Alasan bila dibatalkan |
| `created_at / updated_at` | `TIMESTAMPTZ` | Mengikuti pola tabel lain |

### 5.2 Tabel baru `online_order_items`

| Kolom | Tipe | Keterangan |
| --- | --- | --- |
| `id` | `BIGINT IDENTITY PK` | |
| `online_order_id` | `BIGINT → online_orders(id) ON DELETE CASCADE` | |
| `product_id / product_sku_id` | `BIGINT → products / product_skus` | Sama seperti `sale_items` |
| `product_name / product_sku / unit` | `VARCHAR` snapshot | Snapshot saat checkout, sama seperti pola `sale_items` — nama/harga historis tidak berubah bila master diubah |
| `quantity` | `NUMERIC(14,3) CHECK > 0` | |
| `unit_price` | `NUMERIC(14,2)` | Harga jual dari database saat checkout, bukan dari browser |
| `line_total` | generated `quantity * unit_price` | Sama seperti `sale_items` |

### 5.3 Perubahan kecil pada tabel lama

- `sales`: tambah kolom penanda channel agar laporan bisa membedakan penjualan kasir dan online, mis. `channel VARCHAR(20) NOT NULL DEFAULT 'point_flow'` dengan nilai `point_flow | online`, plus `online_order_id BIGINT NULL` unik bila penjualan berasal dari pesanan online. Ini perubahan aditif (kolom baru ber-default), penjualan lama tidak terpengaruh.
- `products`: tambah kolom `image_url VARCHAR(500) NULL` berisi object key S3 (mis. `products/42/photo.jpg`), bukan URL penuh. Backend menyusun URL publik sebagai `S3_PUBLIC_URL + '/' + image_url` saat serving katalog. Satu foto per produk (dipakai bersama semua SKU-nya). Produk tanpa foto tetap tampil memakai placeholder di frontend.
- Tidak ada perubahan pada `product_skus` dan tabel stok lain selain bacaan stok untuk katalog.

### 5.4 Pengaturan toko dan penyimpanan file

Tabel baru `store_settings` kunci-nilai sederhana (usulan bagian dari migrasi `009`):

| Kolom | Tipe | Keterangan |
| --- | --- | --- |
| `key` | `VARCHAR(80) PK` | Mis. `store_name`, `operating_hours`, `bank_accounts`, `ewallet_numbers`, `qris_image_key`, `delivery_fee` |
| `value` | `TEXT NOT NULL` | Nilai teks; untuk gambar berisi object key S3 |
| `updated_at / updated_by` | `TIMESTAMPTZ / BIGINT → users(id)` | Audit perubahan terakhir |

Isi yang dikelola dari halaman pengaturan toko yang sudah ada (`(protected)/config`, diperluas — bukan halaman baru): nama toko, jam operasional, nomor rekening, nomor e-wallet, gambar QRIS, ongkir tetap (`delivery_fee`, default Rp0). Kasir/admin bisa mengubahnya tanpa deploy ulang.

Alur upload (QRIS dan foto produk): dashboard mengirim multipart ke endpoint backend terautentikasi → backend memvalidasi tipe/ukuran file → backend PUT object ke S3 → object key disimpan di `store_settings` / `products.image_url`. Browser tidak pernah upload langsung ke S3 dan tidak pernah menerima secret key.

Kredensial S3 lewat env (nama variabel dikunci, nilai produksi diisi manual):

| Variabel | Lokal (MinIO) | Produksi |
| --- | --- | --- |
| `S3_ENDPOINT_URL` | `http://localhost:9000` (`cargo run` dari host) / `http://minio:9000` (dari container backend) | URL S3-compatible milik sendiri |
| `S3_REGION` | `us-east-1` | Region bucket |
| `S3_BUCKET` | `point-flow-dev` | Nama bucket produksi |
| `S3_ACCESS_KEY_ID` | `minioadmin` | Diisi manual |
| `S3_SECRET_ACCESS_KEY` | `minioadmin123` | Diisi manual |
| `S3_PUBLIC_URL` | `http://localhost:9000/point-flow-dev` | Domain publik/CDN bucket |
| `S3_FORCE_PATH_STYLE` | `true` (wajib untuk MinIO) | `false` (atau sesuai penyedia) |

Bucket bersifat public-read untuk prefix gambar (`products/*`, `qris/*`) sehingga `<img>` storefront bisa memuat langsung tanpa presigned URL. Secret key hanya dipakai server-side.

## 6. Aturan Stok

Mengikuti model persediaan yang sudah berlaku: stok hanya berubah dari dokumen yang diposting dan dari penjualan selesai. Untuk pesanan online ditetapkan satu titik potong yang jelas:

1. **Saat katalog tampil**: semua SKU aktif ditampilkan, termasuk yang `stock = 0`. Stok yang tampil adalah stok saat itu juga (bukan reservasi). Produk dengan stok habis diberi penanda “Stok habis” dan tombol tambahnya dinonaktifkan di frontend — pembeli tetap bisa melihat bahwa produk itu ada/dijual dan menanyakan kemungkinan restok.
2. **Saat checkout**: server memeriksa ulang setiap item — SKU masih aktif, `stock > 0`, dan `quantity <= stock`. Item stok habis atau quantity melebihi stok membuat checkout ditolak dengan pesan item mana yang tidak tersedia, keranjang pembeli tidak hilang. Harga selalu diambil dari `product_skus.selling_price` di database.
3. **Saat kasir menekan Terima & Packing**: stok terpotong dalam satu transaksi database bersamaan dengan perubahan status ke `packing`. Bila stok tidak mencukupi pada titik ini, transisi ditolak dan kasir diminta membatalkan / menghubungi pembeli.
4. **Saat pesanan dibatalkan dari status `packing`**: stok item dikembalikan dalam transaksi yang sama. Dari status `new` belum ada potongan, jadi tidak ada pengembalian.
5. **Saat pesanan `completed`**: tidak ada potongan stok kedua; penjualan dibuat dari item pesanan (Bagian 7).

Konsekuensi yang disadari dan diterima untuk MVP: dua pembeli bisa memesan item terakhir yang sama selama belum ada yang dipacking — yang diproses kasir lebih dulu yang dapat barang. Reservasi stok otomatis (hold dengan kedaluwarsa) adalah peningkatan fase lanjutan, bukan MVP, karena menambah kompleksitas besar untuk tahap awal satu toko ini.

Catatan jam layanan: pesanan boleh masuk kapan saja (sistem tidak mengunci checkout di luar jam buka), tetapi kasir hanya memproses saat toko buka. Halaman storefront menampilkan jam operasional dari pengaturan toko.

## 7. Hubungan dengan Penjualan dan Sesi Register

- Saat status berubah ke `completed`, backend membuat satu baris `sales` (status `completed`, `channel = 'online'`, `online_order_id` terisi, `cashier_id` = kasir yang menyelesaikan, `cash_register_session_id` = sesi terbuka milik kasir tersebut) beserta `sale_items` yang disalin dari `online_order_items`. `online_orders.sale_id` diisi balik. Dengan ini laporan penjualan, laporan stok, dan dashboard yang sudah ada otomatis ikut membaca penjualan online tanpa query khusus baru, selain membedakan channel bila ditampilkan.
- Pembayaran:
  - `transfer`: kasir mengubah `payment_status` menjadi `paid` setelah mutasi/bukti diterima. Aturan dikunci: `delivery + transfer` wajib `paid` sebelum status `delivering`; `pickup` fleksibel (`paid` boleh ditandai saat serah terima).
  - `pay_at_store`: pembayaran terjadi saat serah terima; `paid_amount` pada penjualan diisi sebesar `total_amount` saat `completed`.
- **Sesi register (keputusan A, dikunci)**: checkout dan packing tidak mensyaratkan sesi register terbuka (pesanan boleh masuk malam hari; potong stok tidak menyentuh uang). `complete` mensyaratkan sesi register terbuka milik kasir yang menyelesaikan — tanpa sesi terbuka, tombol Selesai ditolak dengan pesan yang jelas. Karena penjualan online selalu terikat sesi, uangnya otomatis tercatat mengikuti pola pergerakan kas yang sudah ada dan terlihat di laporan sesi yang berjalan. Konsekuensi yang diterima: pesanan dari malam hari baru bisa diselesaikan setelah kasir membuka sesi pagi harinya.

## 8. API yang Diusulkan

Semua respons mengikuti envelope `ApiResponse<T>` dan paginasi `PaginationMeta` yang sudah ada.

### 8.1 Publik (tanpa login, namespace baru terpisah dari endpoint admin)

| Method & path | Fungsi |
| --- | --- |
| `GET /api/v1/public/products?search=&category_id=&page=&limit=` | Katalog: semua SKU aktif termasuk stok habis. Hanya kolom aman: id SKU, nama produk, nama SKU, satuan, harga jual, stok, kategori, URL foto publik (`S3_PUBLIC_URL` + `products.image_url`, kosong bila belum ada foto). Frontend memakai nilai stok untuk menonaktifkan tombol tambah pada item stok habis. **Tidak pernah** mengekspos `purchase_price`, supplier, atau data master internal |
| `GET /api/v1/public/categories` | Daftar kategori untuk filter katalog |
| `GET /api/v1/public/store-info` | Nama toko, jam operasional, info pembayaran (rekening, e-wallet, URL gambar QRIS), ongkir tetap bila ada |
| `POST /api/v1/public/orders` | Checkout. Body: identitas pembeli, metode serah terima, alamat bila antar, metode pembayaran, daftar `{product_sku_id, quantity}`. Harga dan total dihitung server. Balasan: `order_no`, ringkasan, info pembayaran bila transfer |

### 8.2 Kasir/admin (JWT + RBAC permission `online_orders`, mengikuti pola modul yang ada)

| Method & path | Fungsi |
| --- | --- |
| `GET /api/v1/online-orders?status=&search=&page=&limit=` | Daftar pesanan untuk dashboard, filter per status |
| `GET /api/v1/online-orders/{id}` | Detail pesanan + item |
| `POST /api/v1/online-orders/{id}/pack` | `new → packing`, potong stok (transaksi) |
| `POST /api/v1/online-orders/{id}/ready` | `packing → ready` (khusus ambil di toko) |
| `POST /api/v1/online-orders/{id}/deliver` | `packing → delivering` (khusus antar) |
| `POST /api/v1/online-orders/{id}/complete` | `ready/delivering → completed`, buat penjualan, tandai pembayaran sesuai metode |
| `POST /api/v1/online-orders/{id}/cancel` | `new/packing → cancelled`, kembalikan stok bila sudah terpotong |
| `POST /api/v1/online-orders/{id}/mark-paid` | `payment_status → paid` (verifikasi transfer manual oleh kasir) |

Backend mengikuti struktur modul baku repo: modul baru `backend/src/modules/online_orders/` berisi `routes.rs`, `handler.rs`, `service.rs`, `repository.rs`, `model.rs`, `dto.rs`, didaftarkan di `backend/src/http/mod.rs` dan `backend/src/modules/mod.rs`. Endpoint publik ditempatkan pada modul yang sama namun route terpisah tanpa middleware auth, dengan DTO publik khusus (bukan DTO admin yang dipakai ulang).

## 9. Rancangan Frontend dan UI/UX (Prinsip: Paling Simpel)

### Storefront pembeli — mobile-first, tanpa login

Lokasi usulan: route group baru `frontend/src/routes/(public)/` dengan halaman katalog pada path `/shop`, di luar layout `(protected)`. File API baru `frontend/src/lib/api/storefront.ts` terpisah dari API admin. Isi cart disimpan di `localStorage` HP pembeli (tidak perlu tabel cart di server).

| Halaman | Isi (hanya ini, tidak lebih) |
| --- | --- |
| Catalog `/shop` | Header nama toko + jam buka, kolom pencarian, chip kategori, grid produk 2 kolom (foto, nama, satuan, harga, sisa stok sederhana: “Tersedia” / angka bila sedikit / “Stok habis” bila 0), tombol **+** per produk yang dinonaktifkan pada item stok habis, bilah cart lengket di bawah berisi jumlah item, total, dan tombol “Lihat Keranjang”. Produk stok habis tetap tampil agar pembeli tahu produk itu ada dan bisa menanyakan restok |
| Cart `/shop/cart` | Daftar item dengan pengubah jumlah (+/− dan hapus), subtotal, tombol lanjut ke checkout. Tanpa kode promo dan tanpa saran produk pada MVP |
| Checkout `/shop/checkout` | Satu formulir pendek sesuai Bagian 4.1 langkah 4, pilihan metode tampil sebagai dua kartu radio besar yang mudah diketuk, ringkasan total selalu terlihat, satu tombol “Buat Pesanan”. Alamat antar hanya muncul bila metode antar dipilih |
| Success `/shop/success/{order_no}` | Nomor pesanan besar dan mudah disalin, ringkasan item + total, instruksi pembayaran bila transfer (rekening, e-wallet, gambar QRIS yang bisa diperbesar), instruksi pengambilan/antaran, tombol hubungi toko via WhatsApp. Tanpa halaman lacak status |

Aturan UI/UX MVP:

- Satu kolom, target ketuk besar, teks harga menonjol, tanpa carousel dan tanpa animasi berat. Pembeli memakai HP dengan keterbatasan layar dan koneksi, jadi tampilan dibuat ringan dan mudah dipakai oleh publik umum.
- Semua teks antarmuka lewat `t()` dari sistem i18n yang sudah ada (`id` default). Isi cart tidak hilang saat halaman dimuat ulang.
- Tidak ada halaman detail produk terpisah pada MVP; info di kartu katalog sudah cukup. Detail produk ditambah hanya bila terbukti dibutuhkan.

Batasan library khusus storefront (keputusan pemilik produk):

- Halaman storefront (`(public)/shop` dan turunannya) **dilarang memakai library UI pihak ketiga** seperti shadcn-svelte (`lib/components/ui`), TanStack Table, `sveltekit-superforms`, maupun `svelte-sonner`. Semua komponen dibangun dari bawaan Svelte/SvelteKit saja: elemen HTML semantik, runes (`$state`, `$derived`, `$props`, `$effect`), `{#if}` / `{#each}` / `{#await}`, form dan `fetch` standar.
- Pengecualian satu-satunya adalah **Tailwind CSS** untuk styling, mengikuti setup Tailwind yang sudah ada di repo.
- Alasan: halaman publik harus ringan dan cepat dimuat di HP, permukaannya kecil (katalog, cart, checkout, success) sehingga tidak butuh komponen berat, dan ketergantungan yang sedikit berarti risiko keamanan serta biaya perawatan yang kecil.
- Batasan ini hanya berlaku untuk storefront publik. Dashboard kasir/admin tetap mengikuti pola repo yang sudah ada (shadcn-svelte, TanStack Table, superforms, sonner) sesuai Bagian dashboard kasir di bawah.
- Validasi formulir checkout memakai validasi manual sederhana di sisi klien (atribut HTML + pemeriksaan di handler submit) ditambah validasi penuh di server saat `POST /api/v1/public/orders` — tidak memakai zod/superforms di storefront.

### Dashboard kasir

- Menu baru **Pesanan Online** pada sidebar area terproteksi, mis. `frontend/src/routes/(protected)/sales/online-orders/` (disesuaikan dengan pengelompokan menu sales yang ada), dengan penanda angka jumlah pesanan `Baru`.
- Halaman daftar mengikuti pola CRUD repo: pencarian, filter status berbentuk tab/chip (`Baru`, `Dipacking`, `Siap Diambil`, `Diantar`, `Selesai`, `Dibatalkan`), paginasi, dan penyegaran berkala sederhana (tombol muat ulang; auto-refresh adalah peningkatan).
- Halaman/panel detail menampilkan item, data pembeli, tombol “Chat WA Pembeli”, dan tombol aksi status sesuai Bagian 4.2 — satu tombol utama per status agar kasir tidak perlu berpikir. Perubahan status memakai dialog konfirmasi pola yang sudah ada, dengan `onSaved` memuat ulang daftar.
- Pengaturan info pembayaran dan jam operasional ditambahkan pada halaman pengaturan toko yang sudah ada, bukan halaman baru.

## 10. Keamanan dan Perlindungan Endpoint Publik

Endpoint publik adalah permukaan serangan baru; MVP tetap menerapkan dasar berikut:

- DTO publik khusus: hanya kolom pada Bagian 8.1. Endpoint katalog tidak memakai ulang DTO/handler produk admin yang mengekspos `purchase_price` dan data internal.
- Harga, total, dan ketersediaan stok selalu dihitung dan divalidasi di server saat checkout. Nilai harga dari browser diabaikan.
- Pembatasan laju memakai `tower_governor` (`GovernorLayer`, algoritma token bucket) yang dipasang **hanya pada router publik** — endpoint kasir/admin tidak di-rate-limit agar operasional tidak terganggu. Kunci pembatas per IP (`PeerIpKeyExtractor`; bila di balik proxy, gunakan header `X-Forwarded-For` yang tepercaya). Dua kuota terpisah: katalog/store-info longgar (baca, mis. puluhan request per menit per IP) dan checkout ketat (tulis, mis. beberapa request per menit per IP; nilai pasti dituning saat implementasi). Respons 429 mengikuti envelope error yang sudah ada. Validasi panjang input (nama, alamat, catatan) tetap berlaku agar endpoint tidak disalahgunakan sebagai spam order. Honeypot/captcha hanya ditambah bila spam benar-benar terjadi.
- Tetap di balik HTTPS saat produksi. Tidak ada data pembeli selain nama, nomor WA, dan alamat antar yang dikumpulkan; data ini hanya tampil untuk user dashboard dengan permission `online_orders`.
- Transisi status hanya boleh berurutan sesuai Bagian 4.3 dan hanya oleh kasir/admin terautentikasi; endpoint publik tidak bisa mengubah status pesanan.

## 11. Langkah Implementasi (Urutan Usulan, Belum Dikerjakan)

0. **Infrastruktur lokal**: service MinIO di `docker-compose.yml` (+ init bucket `point-flow-dev` public-read prefix `products/*`, `qris/*`), variabel `S3_*` di `.env.example` / `.env.prod.example`, dan teruskan ke service backend. Kredensial produksi diisi manual (lihat Bagian 5.4).
1. **Migrasi database** `009_add_online_orders.sql`: tabel `online_orders`, `online_order_items`, tabel `store_settings` (+ seed awal: nama toko, jam operasional, `delivery_fee = 0`), sequence nomor pesanan, kolom channel + `online_order_id` pada `sales`, kolom `image_url` pada `products`, seed permission `online_orders` ke role yang sesuai. Uji migrasi pada database kosong dan pada salinan data yang ada.
2. **Backend modul `online_orders`**: katalog publik, checkout publik (validasi stok + hitung harga server-side), dan endpoint kasir termasuk transisi status transaksional (potong/kembalikan stok, pembuatan penjualan saat selesai dengan sesi register kasir yang menyelesaikan). Tambah klien S3 (`aws-sdk-s3` + env `S3_*` dari Bagian 5.4), endpoint upload terautentikasi (QRIS, foto produk), dan `tower_governor` pada router publik. Daftarkan route di router utama.
3. **API client frontend**: `storefront.ts` untuk pembeli dan `online-orders.ts` untuk kasir, mengikuti pola `lib/api/` yang ada.
4. **Storefront publik** `(public)/shop`: catalog, cart (`localStorage`), checkout, halaman success. Mobile-first, semua teks via `t()`, dan tanpa library UI pihak ketiga kecuali Tailwind (lihat batasan library di Bagian 9).
5. **Dashboard kasir**: menu Pesanan Online, daftar + detail + aksi status, penanda pesanan baru pada sidebar, perluasan pengaturan info pembayaran.
6. **Uji end-to-end manual** sesuai Bagian 12, lalu rilis ke satu toko dan pantau pesanan masuk selama masa awal.

Setiap langkah di atas adalah unit kerja terpisah yang bisa ditinjau sendiri; tidak ada perubahan perilaku pada flow Point Flow kasir yang sudah berjalan selain kolom channel ber-default `point_flow`.

## 12. Rencana Validasi

Backend:

```bash
cargo test --manifest-path backend/Cargo.toml
cargo clippy --manifest-path backend/Cargo.toml --all-targets
```

Frontend:

```bash
bun run check   # dari folder frontend/
bun run lint
```

Skenario uji manual wajib (browser HP atau mode perangkat pada browser):

1. Sebagai pembeli tanpa login: produk stok habis tetap tampil dengan penanda “Stok habis” dan tombol tambah yang tidak bisa diklik; tambahkan ke keranjang melebihi stok → ditolak/dibatasi; kirim checkout berisi item stok habis secara paksa (mis. via request manual) → ditolak server; checkout ambil di toko + bayar di tempat → nomor pesanan tampil.
2. Checkout antar + transfer → info rekening/e-wallet/QRIS dan total tampil di halaman sukses; pesanan tampil di dashboard kasir sebagai `Baru`.
3. Sebagai kasir: terima & packing → stok SKU berkurang tepat sejumlah item. Batalkan pesanan lain yang sudah dipacking → stok kembali.
4. Selesaikan pesanan → penjualan muncul di daftar penjualan dan laporan dengan channel online; selesaikan pesanan antar-transfer sebelum `mark-paid` → tertahan sesuai aturan Bagian 7.
5. Checkout dengan stok diubah kasir di tengah jalan (stok dibuat tidak cukup sebelum packing) → transisi packing ditolak dengan pesan jelas.
6. Endpoint katalog publik diperiksa tidak memuat harga beli dan data internal lain.

## 13. Di Luar MVP (Fase Lanjutan, Sesuai Kebutuhan Nyata)

- Akun pembeli + OTP WhatsApp dan halaman riwayat/lacak status untuk pembeli.
- Payment gateway (QRIS dinamis per transaksi, VA) dengan status pembayaran otomatis.
- Reservasi stok saat checkout dengan kedaluwarsa otomatis.
- Notifikasi WhatsApp otomatis ke pembeli pada setiap perubahan status.
- Ongkir bertingkat per zona/jarak, jadwal pengantaran, dan minimal belanja antar.
- PWA (ikon “tambah ke layar utama”) dan mode hemat data untuk katalog.

## 14. Risiko yang Diterima dan Mitigasinya

| Risiko | Dampak | Mitigasi MVP |
| --- | --- | --- |
| Dua pembeli memesan stok terakhir yang sama | Satu pesanan harus dibatalkan kasir | Terima di MVP (Bagian 6); kasir menghubungi pembeli via WA. Reservasi stok = fase lanjutan |
| Pesanan iseng / spam karena tanpa verifikasi | Kasir terganggu pesanan palsu | Rate limit checkout, pesanan tidak memotong stok sebelum kasir menerima, pemblokiran nomor manual dicatat di luar sistem |
| Pembeli transfer tapi kasir belum tahu | Pesanan tertahan | `payment_status` manual + tombol Chat WA; pembeli diinstruksikan konfirmasi via WA pada halaman sukses |
| Kebingungan uang online dengan laci register | Rekonsiliasi sesi meleset | Selesai by design (keputusan A): `complete` wajib sesi terbuka sehingga penjualan online selalu tercatat di laporan sesi yang berjalan |
| Alamat antar tidak jelas | Pengantaran lambat | Alamat bebas terstruktur longgar (jalan/nomor, patokan) + catatan + nomor WA aktif yang bisa dihubungi pengantar |

## 15. Hal yang Masih Terbuka (Tidak Menghalangi Mulai, Diputuskan Saat Implementasi)

- Besaran ongkir antar: gratis, satu nominal tetap, atau gratis di atas minimal belanja. Dokumen ini mengasumsikan satu kolom `delivery_fee` dari pengaturan (default Rp0).
- Batas jam penerimaan pesanan antar di halaman storefront (mis. pesanan setelah jam tertentu diproses besok) — cukup ditampilkan sebagai teks info pada MVP.
- Apakah halaman sukses perlu tautan “pesan lagi” yang mengosongkan keranjang — diasumsikan ya, satu tombol sederhana.

---

Dokumen ini adalah rencana. Implementasi dimulai hanya setelah pemilik produk menyatakan lanjut (`go`), dan dikerjakan mengikuti urutan Bagian 11.
