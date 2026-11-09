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

- [ ] `GET /reports/income-statement?date_from&date_to` — pendapatan net retur, HPP dari
      `unit_cost`, beban payroll + manual, laba kotor & bersih (formula desain Bagian 2.2)
- [ ] `GET /reports/cash-flow?date_from&date_to` — masuk: sales + income `is_cash`;
      keluar: drawer movements + expense `is_cash`
- [ ] `GET /reports/financial-position?as_of` — kas kumulatif, persediaan (stok × harga
      beli), laba kumulatif (desain Bagian 2.2)
- [ ] RBAC: laporan keuangan hanya role berpermission (default admin)
- [ ] Registrasi route di modul reports tanpa mengubah struktur yang ada

Validasi: `curl` dengan fixture (sales + retur + payroll + expense manual) — rekonsiliasi
angka laporan vs hitungan manual, termasuk kasus `is_cash=false` dan retur.
Gate review: JSON contoh ketiga laporan dari fixture yang sama.

## Fase 4 — Frontend keuangan

Tujuan: admin mencatat keuangan manual dan membaca tiga laporan dari dashboard.

- [ ] Halaman `finances/transactions`: list (filter tanggal/type/kategori, paginasi) +
      dialog create/edit + cancel dengan alasan + dialog kategori (Bagian 2.3)
- [ ] Laporan `reports/income-statement`, `reports/cash-flow`, `reports/financial-position`
      (kartu ringkasan + tabel, filter periode)
- [ ] `lib/api/finances.ts` + extend `lib/api/reports.ts`
- [ ] i18n `finances.*`, `reports.incomeStatement.*`, `reports.cashFlow.*`,
      `reports.financialPosition.*` (id/en) + registrasi nav sidebar

Validasi: `bun run check` 0 error, `bun run lint` bersih di file tersentuh,
`bun run build` sukses; angka di halaman cocok dengan JSON backend (fixture Fase 3).
Gate review: buka halaman di browser, satu alur penuh: catat expense → lihat laporan berubah.

## Fase 5 — Migrasi `012_add_members.sql`

Tujuan: skema member + kredensial login dan kolom atribusi.

- [ ] Tabel `members` — `phone` unique NOT NULL (identitas login), `email` opsional,
      `password_hash` NOT NULL, `is_active` (desain Bagian 3.1)
- [ ] Tabel `member_refresh_tokens` terpisah dari `refresh_tokens` staff (Bagian 3.1)
- [ ] Kolom `member_id` di `sales` dan `online_orders` + index
- [ ] Seed permission `members` ke role yang sesuai
- [ ] Uji di database kosong dan salinan data (001–012 berurutan; unik telepon enforced)

Validasi: migrasi naik bersih di scratch Postgres, insert telepon ganda ditolak.
Gate review: diff `backend/migrations/012_add_members.sql`.

## Fase 6 — Backend modul members (CRUD dashboard)

Tujuan: admin mendaftarkan dan mengelola member beserta kredensialnya.

- [ ] Modul `backend/src/modules/members/` — CRUD `/members` (search name/phone/member_no,
      filter `is_active`, soft-delete/hard-delete sesuai pemakaian) (desain Bagian 3.2)
- [ ] `POST /members/{id}/reset-password` (staff, admin) — set password baru
- [ ] Password di-hash Argon2 (pola auth staff yang ada)
- [ ] DTO sale: field `member_id` opsional — validasi member ada & `is_active`
- [ ] RBAC permission `members`

Validasi: `cargo test` + clippy; `curl` e2e: create member (telepon unik), search,
update, deactivate → tidak bisa login (uji menyusul di Fase 7), reset password.
Gate review: daftar endpoint + contoh respons create/reset.

## Fase 7 — Backend member auth & atribusi checkout

Tujuan: member bisa login di storefront dan belanjanya teratribusi otomatis untuk SHU.

- [ ] `/members/auth/login|refresh|logout|me|change-password` — public, masuk sub-router
      `/public/*` terenkripsi modul online_orders (desain Bagian 3.2)
- [ ] JWT member-scope terpisah + refresh cookie HttpOnly; `member_refresh_tokens`
      digunakan untuk refresh/revocation
- [ ] Isolasi token: extractor staff menolak token member, extractor member menolak
      token staff (dua dunia tidak saling menembus)
- [ ] Checkout: `member_id` diambil **dari token**, bukan request body (anti-spoof);
      identitas pembeli dari profil member; tanpa token → guest, `member_id` NULL
- [ ] Konversi order → sale (pembayaran online dikonfirmasi) menyalin `member_id` ke sales
- [ ] Member `is_active=false` → login ditolak; rate limit ketat di endpoint auth
      (pola `tower_governor` checkout)

Validasi: `cargo test` (termasuk unit test scope token) + `curl` e2e di scratch DB:
login member → checkout teratribusi (`member_id` terisi) → complete → sale punya
`member_id`; checkout guest tetap NULL; token member dipakai ke endpoint dashboard →
ditolak; token staff dipakai ke `/members/auth/me` → ditolak; deaktif member → login ditolak.
Gate review: matriks token scope (endpoint × jenis token) + hasil e2e.

## Fase 8 — Frontend anggota & storefront auth

Tujuan: admin mengelola member, anggota login & belanja di storefront, kasir bisa
menandai pembeli member di POS.

- [ ] Dashboard `master/members` (CRUD) — form: telepon wajib & unique, password awal
      di-set admin, aksi reset password per member (desain Bagian 3.3)
- [ ] Picker anggota opsional di alur kasir POS: cari via member_no/telepon, member
      tampil di detail transaksi
- [ ] Storefront: login/logout, halaman profil + ganti password; checkout terdeteksi
      login → identitas prefill dari profil (alamat bisa diedit); alur guest tetap
- [ ] `lib/api/members.ts` (dashboard); auth & profil member di `lib/api/storefront.ts`
- [ ] i18n id/en untuk semua string baru

Validasi: `bun run check` 0 error, lint bersih di file tersentuh, `bun run build`
sukses; alur manual di browser: daftar member (admin) → login storefront → checkout
member → order muncul teratribusi di dashboard; alur guest tak berubah.
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
