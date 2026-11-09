# Roadmap: Laporan Keuangan & Profit Sharing (SHU)

Desain acuan: [financial-reports-shu.md](./financial-reports-shu.md).
Cara pakai: kerjakan satu fase sampai selesai, lalu minta review sebelum lanjut.
Balas `go fase N` untuk mengeksekusi satu fase, atau sebut yang mau diubah.

Urutan mengikuti section 5 desain: Keuangan → Anggota + member auth → Profit Sharing.

## Fase 1 — Migrasi `011_add_finances.sql`

Tujuan: skema keuangan manual + fondasi HPP snapshot dan permission.

- [x] Tabel `financial_categories` + `financial_transactions` (desain Bagian 2.1)
- [x] Sequence harian nomor transaksi (`FIN-YYYYMMDD-NNNNNN`, 6 digit ikut konvensi repo)
- [x] Kolom `unit_cost NUMERIC(14,2) NOT NULL DEFAULT 0` di `sale_items` (desain Bagian 2.4)
- [x] Field `is_cash BOOLEAN NOT NULL DEFAULT true` di `financial_transactions` (keputusan tercatat)
- [x] Seed permission `finances` + laporan keuangan ke role yang sesuai (pola `004_add_rbac.sql`) —
      permission baru `finances`, di-grant ke role `admin` (pola CROSS JOIN dari 009)
- [x] Uji di database kosong dan salinan data (2026-11-09, scratch Postgres: 001–011 berurutan
      + 6 uji: default `unit_cost = 0`, CHECK unit_cost negatif ditolak, UNIQUE kategori
      (name, type) ditolak, nama sama type beda lolos, CHECK amount > 0 ditolak, nomor
      `FIN-20261108-000002` terformat + default `is_cash`/`status` + trigger `set_updated_at`)
- [x] `cargo test` hijau (16 passed, 0 failed)

Validasi: ✅ migrasi naik bersih di scratch Postgres (2026-11-09), `cargo test` tetap hijau.
Catatan deviasi kecil: ditambah `CONSTRAINT sale_items_unit_cost_non_negative CHECK
(unit_cost >= 0)` — tidak ada di desain Bagian 2.4, tapi konsisten dengan konvensi CHECK
non-negative repo (`unit_price_non_negative`, `purchase_price_non_negative`).
Gate review: diff `backend/migrations/011_add_finances.sql`.

## Fase 2 — Backend modul finances & snapshot HPP

Tujuan: pengeluaran/pemasukan manual tercatat, dan HPP tiap penjualan terkunci saat transaksi.

- [x] Modul `backend/src/modules/finances/` (routes/handler/service/repository/model/dto)
- [x] CRUD `/financial-categories` + `/financial-transactions` + cancel (desain Bagian 2.2) —
      permission `finances`; kategori: duplicate (nama,type) → 409, type immutable, delete
      dipakai → 409; transaksi: validasi type cocok kategori, amount > 0, update/cancel
      hanya saat `completed` (cancel ganda → 409); filter list: date range, `type`,
      `financial_category_id`, `status`, search
- [x] Snapshot `unit_cost` dari `product_skus.purchase_price` saat sale dibuat (service sale
      POS + konversi order online via JOIN `product_skus`)
- [x] Sale return membalikkan COGS sebesar `unit_cost × qty` yang diretur (desain Bagian 2.4) —
      tanpa kolom baru: retur menunjuk `sale_item_id`, net COGS = Σ qty×unit_cost −
      qty-diretur×unit_cost (dipakai formula Fase 3)
- [x] `is_cash` memengaruhi apakah transaksi masuk arus kas (non-tunai tidak) —
      disimpan di create/update; efek ke laporan dihitung di Fase 3

Validasi (2026-11-09, scratch Postgres `point_flow_fase2`, 001–011): `cargo test`
16 passed, 0 failed; `cargo clippy --all-targets` bersih kecuali 2 warning
pre-existing di `master.rs`; `cargo fmt --check` bersih. `curl` e2e: kategori
duplicate → 409, type bogus → 422 (body) / 400 (query); transaksi type-mismatch →
422, amount negatif → 422, kategori tak ada → 404; cancel ganda → 409, update
cancelled → 409, delete kategori terpakai → 409, tanpa token → 401; penomoran
`FIN-20261109-000001` berurutan. Sale POS 4× @beli 45000 → `unit_cost: 45000`;
harga beli diubah 50000 → sale baru snapshot 50000, sale lama tetap 45000;
order online → complete → sale `channel=online` ikut snapshot. Retur parsial
1 unit → gross 180000, net 135000 (proporsional). Expense `is_cash=false`
25000 tidak masuk hitungan kas-keluar-manual (175000 vs 200000).
Catatan deviasi kecil: filter enum dibandingkan sebagai text
(`ft.type::text = $4`, pola `status::text` repo) karena Postgres menolak
operator enum = text untuk bind parameter; filter `type` invalid → 400
(konsisten `parse_status_filter`), body invalid tetap 422.
Gate review: daftar endpoint + contoh respons create/cancel transaksi.

## Fase 3 — Backend laporan keuangan

Tujuan: laba rugi, arus kas, dan posisi keuangan tersaji dari data operasional + transaksi manual.

- [x] `GET /reports/income-statement?date_from&date_to` — pendapatan net retur, HPP dari
      `unit_cost`, beban payroll + manual, laba kotor & bersih (formula desain Bagian 2.2)
- [x] `GET /reports/cash-flow?date_from&date_to` — masuk: sales + income `is_cash`;
      keluar: drawer movements + expense `is_cash`
- [x] `GET /reports/financial-position?as_of` — kas kumulatif, persediaan (stok × harga
      beli), laba kumulatif (desain Bagian 2.2)
- [x] RBAC: laporan keuangan hanya role berpermission (default admin) — dipakai permission
      `finances` (seed 011); kasir → 403, tanpa token → 401
- [x] Registrasi route di modul reports tanpa mengubah struktur yang ada (tambah handler +
      repository wrapper + query struct `FinancialPeriodQuery`/`FinancialPositionQuery`)

Validasi (2026-11-09, scratch Postgres `point_flow_fase3`, 001–011 + fixture: 3 sales —
satu `sold_at` Okt, 1 retur parsial restock, 1 slip gaji Nov 3300000, expense cash
150000 + non-cash 25000 + 1 cancelled, income cash 100000, drawer `petty_cash`
50000): `cargo test` 16 passed, clippy bersih kecuali 2 warning pre-existing
`master.rs`, `fmt --check` bersih. Rekonsiliasi cocok semua — laba rugi full:
440000/75000/365000/215000/150000/3300000/175000/−3325000; filter Nov:
370000/…/−3355000; arus kas: masuk 540000, keluar 200000, net 340000 (non-cash
25000 terdepak dari kas tapi masuk beban); posisi default: kas 340000, persediaan
5285000, aset 5625000; `as_of=2026-10-15`: kas 70000, laba 30000; `as_of=2026-12-01`:
laba −3325000 (slip Nov masuk setelah `period_end`, konsisten predikat laporan
payroll repo `period_end <= to`).
Catatan deviasi kecil: HPP memakai `COALESCE(NULLIF(unit_cost,0), purchase_price)`
karena kolom `unit_cost NOT NULL DEFAULT 0` membuat `COALESCE(unit_cost, …)` desain
tidak pernah fallback untuk penjualan pra-migrasi — intent "harga berjalan" tetap
terpenuhi. Observasi (sesuai desain, tidak diubah): pendapatan manual (`income`)
tidak masuk laba rugi; retur tidak mengurangi arus kas; `cash_in` drawer tidak
dihitung; persediaan posisi selalu snapshot saat ini.
Gate review: JSON contoh ketiga laporan dari fixture yang sama.

## Fase 4 — Frontend keuangan

Tujuan: admin mencatat keuangan manual dan membaca tiga laporan dari dashboard.

- [x] Halaman `finances/transactions`: list (filter tanggal/type/kategori, paginasi) +
      dialog create/edit + cancel dengan alasan + dialog kategori (Bagian 2.3)
- [x] Laporan `reports/income-statement`, `reports/cash-flow`, `reports/financial-position`
      (kartu ringkasan + tabel, filter periode)
- [x] `lib/api/finances.ts` + extend `lib/api/reports.ts`
- [x] i18n `finances.*`, `reports.incomeStatement.*`, `reports.cashFlow.*`,
      `reports.financialPosition.*` (id/en) + registrasi nav sidebar (grup Keuangan,
      ikon wallet; `common.all` baru untuk opsi "Semua")

Validasi (2026-11-09): `bun run check` 0 error, `bun run lint` bersih di file
tersentuh (file pre-existing yang belum prettier-clean tidak disentuh),
`bun run build` sukses. Smoke API dengan bentuk request persis seperti frontend
kirim (scratch `point_flow_fase4`): filter `type`+`status`, `financial_category_id`+
`search`, `date_from` saja, `as_of` — semua cocok. Tanpa smoke browser (tidak ada
Chromium di mesin ini; tidak install apa pun).
Gate review: buka halaman di browser, satu alur penuh: catat expense → lihat laporan berubah.

## Fase 5 — Migrasi `012_add_members.sql`

Tujuan: skema member + kredensial login dan kolom atribusi.

- [x] Tabel `members` — `phone` unique NOT NULL (identitas login), `email` opsional,
      `password_hash` NOT NULL, `is_active` (desain Bagian 3.1)
- [x] Tabel `member_refresh_tokens` terpisah dari `refresh_tokens` staff (Bagian 3.1)
- [x] Kolom `member_id` di `sales` dan `online_orders` + index
- [x] Seed permission `members` ke role yang sesuai — `admin` + `cashier` (kasir butuh
      baca member untuk picker POS Fase 8; tulis tetap lewat endpoint berpermission sama)
- [x] Uji di database kosong dan salinan data (001–012 berurutan; unik telepon enforced)

Validasi (2026-11-09, scratch Postgres `point_flow_fase5`, 001–012): migrasi naik
bersih; `MBR-20261109-000001` terformat + default `is_active`; duplikat telepon,
nama kosong, telepon kosong ditolak (UNIQUE + CHECK); duplikat `token_hash`
ditolak; trigger `set_updated_at` jalan; hapus member → `sales.member_id` jadi
NULL (SET NULL), token ikut terhapus (CASCADE); `cargo test` 16 passed.
Catatan deviasi kecil: tambah CHECK nama/telepon non-kosong (pola
`customer_name/phone_required` order), UNIQUE `member_no` (pola nomor dokumen),
trigger `set_updated_at` (konvensi 011), `token_hash VARCHAR(64)` menyerupai
`refresh_tokens` staff (bukan TEXT desain).
Gate review: diff `backend/migrations/012_add_members.sql`.

## Fase 6 — Backend modul members (CRUD dashboard)

Tujuan: admin mendaftarkan dan mengelola member beserta kredensialnya.

- [x] Modul `backend/src/modules/members/` — CRUD `/members` (search name/phone/member_no,
      filter `is_active`, soft-delete/hard-delete sesuai pemakaian) (desain Bagian 3.2)
- [x] `POST /members/{id}/reset-password` (staff, admin) — set password baru
- [x] Password di-hash Argon2 (pola auth staff yang ada)
- [x] DTO sale: field `member_id` opsional — validasi member ada & `is_active`
- [x] RBAC permission `members`

Validasi (2026-11-09, scratch Postgres `point_flow_fase6`, 001–012): `cargo test`
16 passed; clippy bersih di file tersentuh; `fmt --check` bersih. `curl` e2e:
create → `MBR-20261109-000001` tanpa `password_hash` di respons; duplikat telepon
→ 409 (create & update), email jelek → 422, password < 8 → 422, nama kosong → 422;
search nama/telepon/nomor + filter `is_active` cocok; reset → hash berputar
(verifikasi SQL), reset pendek → 422, reset id tak ada → 404; sale dengan
`member_id` aktif → teratribusi, member nonaktif → 422, id tak ada → 404, tanpa
`member_id` → guest NULL; hapus member terpakai → soft (`is_active=false`, tetap
ada), hapus yang tak terpakai → hard (lalu 404); tanpa token → 401. Login member
nonaktif diuji di Fase 7 (endpoint auth belum ada).
Gate review: daftar endpoint + contoh respons create/reset.

## Fase 7 — Backend member auth & atribusi checkout

Tujuan: member bisa login di storefront dan belanjanya teratribusi otomatis untuk SHU.

- [x] `/members/auth/login|refresh|logout|me|change-password` — public, digabung ke
      `public_router` (pola sub-router `/public/*` modul online_orders, desain Bagian 3.2)
- [x] JWT member-scope terpisah + refresh cookie HttpOnly; `member_refresh_tokens`
      digunakan untuk refresh/revocation
- [x] Isolasi token: extractor staff menolak token member, extractor member menolak
      token staff (dua dunia tidak saling menembus)
- [x] Checkout: `member_id` diambil **dari token**, bukan request body (anti-spoof);
      identitas pembeli dari profil member; tanpa token → guest, `member_id` NULL
- [x] Konversi order → sale (pembayaran online dikonfirmasi) menyalin `member_id` ke sales
- [x] Member `is_active=false` → login ditolak; rate limit ketat di endpoint auth
      (pola `tower_governor` checkout)

Validasi (2026-11-09, scratch Postgres `point_flow_fase7`, 001–012): `cargo test`
22 passed, 0 failed (16 lama + 6 baru: 2 round-trip scope token, 4 resolusi
identitas checkout); `cargo clippy --all-targets` bersih di file tersentuh
(4 warning sisa pre-existing di `master.rs`); `cargo fmt --check` bersih.
`curl` e2e: login member → `MBR-20261109-000001` + cookie `member_refresh_token`
(HttpOnly, `Path=/api/v1/members/auth`) tanpa `password_hash` di body; checkout
member (body nama "Guest Bohongan") → order `member_id=1`, nama/telepon dari
profil, alamat default profil; pack → ready → complete → sale `member_id=1`;
checkout guest → order + sale `member_id` NULL; refresh rotasi (token lama 401),
change-password mencabut sesi lama (login lama 401, baru 200), logout mencabut
refresh (401); deaktif member → login 401 dan `/me` 401; burst 14 login salah →
13× 429.
Matriks scope (diamati): token member → `GET /members` 401, token staff →
`GET /members/auth/me` 401, token staff → `POST /public/orders` 401 (tidak turun
diam-diam jadi guest).
Catatan deviasi kecil: scope member diwujudkan sebagai JWT `role="member"`
(tanpa kolom klaim baru; kompatibel dengan token staff lama); extractor staff
menolak `role="member"` sebelum lookup DB (menutup lubang tabrakan id
member↔user); cookie refresh member `"member_refresh_token"` terpisah dari
cookie staff agar dua sesi tidak saling menimpa; rate limit = burst 10 +
1/menit sustained, persis parameter checkout (bukan bucket terpisah per endpoint).
Gate review: matriks token scope di atas + hasil e2e.

## Fase 8 — Frontend anggota & storefront auth

Tujuan: admin mengelola member, anggota login & belanja di storefront, kasir bisa
menandai pembeli member di POS.

- [x] Dashboard `master/members` (CRUD) — form: telepon wajib & unique, password awal
      di-set admin, aksi reset password per member (desain Bagian 3.3)
- [x] Picker anggota opsional di alur kasir POS: cari via member_no/telepon, member
      tampil di detail transaksi
- [x] Storefront: login/logout, halaman profil + ganti password; checkout terdeteksi
      login → identitas prefill dari profil (alamat bisa diedit); alur guest tetap
- [x] `lib/api/members.ts` (dashboard); auth & profil member di `lib/api/storefront.ts`
- [x] i18n id/en untuk semua string baru

Validasi (2026-11-09): `bun run check` 0 error 0 warning, eslint + prettier bersih
di file tersentuh (file pre-existing yang belum prettier-clean tidak disentuh —
`bun run lint` repo masih merah di file lain yang tidak terkait Fase 8),
`bun run build` sukses. Tanpa smoke browser (tidak ada Chromium di mesin ini;
tidak install apa pun).
Alur manual di browser (gate review, belum dijalankan): daftar member (admin) →
login storefront → checkout member → order muncul teratribusi di dashboard;
alur guest tak berubah.
Gate review: coba dua checkout dari HP (member & guest), bandingkan hasilnya.

## Fase 9 — Migrasi `013_add_profit_sharing.sql`

Tujuan: skema periode profit sharing (SHU) dan snapshot distribusi.

- [ ] Tabel `profit_sharing_periods` — 6 bucket kolom fixed + CHECK total 100%
      (desain Bagian 4.1)
- [ ] Tabel `profit_sharing_distributions` + UNIQUE (period, member) (desain Bagian 4.1)
- [ ] Enum status `draft/calculated/finalized/cancelled` + kolom jejak
      (`auto_net_profit`, `finalized_by`, `cancel_reason`, dst.)
- [ ] Seed permission `profit_sharing` ke role yang sesuai
- [ ] Uji di database kosong dan salinan data (001–013 berurutan; CHECK 100% enforced)

Validasi: migrasi naik bersih; insert persentase ≠ 100 ditolak.
Gate review: diff `backend/migrations/013_add_profit_sharing.sql`.

## Fase 10 — Backend modul profit sharing

Tujuan: kalkulasi SHU transparan dengan snapshot yang terkunci saat final.

- [ ] CRUD `/profit-sharing/periods` — PUT hanya draft; calculate/finalize/cancel
      (desain Bagian 4.2–4.3)
- [ ] Kalkulasi: pool = `total_pool` ?? `auto_net_profit` (dari income-statement);
      `member_pool` = pool × `member_pct`/100; per anggota net transaksi (penjualan −
      retur), hanya member `is_active`; `share_amount` = member_pool × share
- [ ] Snapshot `profit_sharing_distributions`; recalc hanya sebelum finalize; setelah
      `finalized` immutable (recalc ditolak); cancel wajib `cancel_reason`
- [ ] `GET /profit-sharing/periods/{id}/distributions` (paginated + search) + export CSV
- [ ] i18n keys: locale `id` "SHU"/"Periode SHU"/"Tahun Buku", locale `en`
      "Profit Sharing"/"Fiscal Year" (pemisahan layer — desain Bagian 4.4)

Validasi: `cargo test` unit test kalkulasi (pool manual vs auto, tanpa anggota bertransaksi,
member inactive dikecualikan, retur mengurangi kontribusi) + `curl` e2e dengan fixture:
period → calculate → angka distribusi direkonsiliasi manual → finalize → recalc ditolak.
Gate review: hasil rekonsiliasi fixture + screenshot CSV.

## Fase 11 — Frontend profit sharing

Tujuan: admin menjalankan siklus SHU dari dashboard: konfigurasi → hitung → final → distribusi.

- [ ] Route `(protected)/profit-sharing/`: list periode + dialog konfigurasi (nama,
      rentang tanggal, pool opsional, 6 persentase dengan validasi total 100%)
- [ ] Detail periode: ringkasan (pool, auto net profit, alokasi per bucket, total
      transaksi anggota) + tabel distribusi per anggota + aksi calculate/finalize/cancel
      + export CSV (desain Bagian 4.4)
- [ ] Label UI: locale `id` istilah "SHU", locale `en` "Profit Sharing"
- [ ] `lib/api/profit-sharing.ts` + nav sidebar + i18n id/en

Validasi: `bun run check`, lint file tersentuh, `bun run build`; alur manual: buat
periode → calculate → cek angka vs backend → finalize → export CSV.
Gate review: siklus penuh SHU di browser dengan data nyata dev.

## Fase 12 — Validasi akhir & integrasi

Tujuan: satu skenario menyeluruh membuktikan rantai transaksi → laporan → SHU, lalu rilis.

- [ ] Skenario e2e menyeluruh (scratch DB): POS sale ber-member + online member + guest +
      retur + expense manual → income-statement → cash-flow → financial-position →
      periode SHU → calculate → finalize → CSV distribusi
- [ ] `cargo test`, `cargo clippy --all-targets`, `bun run check`, `bun run lint`,
      `bun run build` semua hijau
- [ ] Review keamanan auth member: scope token, rate limit, password hashing,
      permukaan endpoint publik (tidak ada data internal bocor)
- [ ] Deploy dev + smoke test (Fly/Supabase + Netlify sesuai README)

Gate review: hasil skenario e2e + keputusan rilis.
