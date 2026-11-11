# Panduan Pengguna Point Flow

Panduan ini menjelaskan cara memakai aplikasi Point Flow dari awal sampai akhir mengikuti menu yang ada di website. Bahasanya dibuat semanusiawi mungkin: tidak ada istilah API, JSON, atau endpoint. Untuk dokumentasi teknis, lihat [api-flow.md](api-flow.md).

Point Flow punya dua wajah:

- **Aplikasi kasir** (dipakai karyawan di toko): login dengan email, ada menu di sisi kiri — Dashboard, Kasir, Master Data, Pembelian, Inventori, Payroll, Keuangan, Laporan, dan Pengaturan.
- **Toko online / storefront** (dibuka pelanggan lewat browser): katalog produk, keranjang, checkout, dan akun anggota — tanpa perlu login karyawan.

Alur besarnya: siapkan data toko dulu (barang, supplier, karyawan), beli barang dari supplier sampai stok masuk, lalu jualan — di kasir maupun online. Uang kas dicatat lewat sesi laci, gaji lewat payroll, biaya lain lewat keuangan, dan sisa hasilnya bisa dibagi lewat SHU di akhir periode.

## Cara Membaca Panduan Ini

Mulai versi ini, setiap form dijelaskan dengan tabel **"Isi dari mana → ke mana"** supaya jelas perjalanan datanya:

- **Diisi manual** — kamu yang mengetik/memilih sendiri.
- **Otomatis dari master data** — kolom terisi sendiri begitu kamu memilih data lain (contoh: harga jual muncul otomatis saat memilih barang).
- **Dibuat otomatis sistem** — nomor dokumen, tanggal, snapshot nama/harga, dan angka hasil perhitungan; tidak perlu (dan tidak bisa) diisi.
- **Ke mana setelah simpan** — dokumen apa yang lahir, angka apa yang berubah (stok, kas), dan laporan apa yang ikut terupdate.

Kadang ada **diagram status** seperti ini, yang menunjukkan perjalanan dokumen:

```
draft ──(tekan Order)──> ordered ──> barang datang
  │
  └──(batal)──> cancelled
```

Prinsip yang berlaku di seluruh aplikasi: selama dokumen masih **draft**, ia belum berpengaruh apa-apa (stok dan uang belum berubah) dan masih bisa diperbaiki. Efek baru terjadi saat tombol **posting/aksi** ditekan. Setelah berpengaruh, yang bisa dilakukan biasanya hanya **membatalkan dengan alasan** — dan semua kejadian tercatat di Audit Log.

## 1. Masuk dan Dashboard

### Login karyawan

Buka aplikasi kasir dan masuk dengan email dan password akun karyawan. Setelah masuk, nama dan email tampil di pojok kiri bawah. Kalau sesi habis, cukup login ulang — data tidak hilang.

Ada dua peran bawaan:

- **Admin** — bisa membuka semua menu.
- **Kasir** — aksesnya terbatas (biasanya hanya kasir dan dashboard). Halaman yang dibuka tanpa izin akan menampilkan pesan penolakan, dan datanya pun dibatasi (kasir hanya melihat transaksinya sendiri).

Kalau baru pertama kali memakai sistem dan belum ada akun sama sekali, akun admin pertama dibuat lewat proses bootstrap (dibantu tim teknis).

### Dashboard

Halaman pertama setelah login. Semua angka di sini **dihitung otomatis** dari dokumen-dokumen lain — tidak ada yang perlu diisi manual. Sumbernya:

| Kartu/grafik | Sumber data | Contoh |
|---|---|---|
| Omzet hari ini | Semua transaksi kasir & pesanan online yang selesai hari ini | Jualan 08.00 pagi langsung naik |
| Jumlah transaksi hari ini | Hitung baris transaksi selesai hari ini | — |
| Produk stok menipis | SKU yang stoknya di bawah/di angka stok minimum (Master Data → SKU) | Air mineral sisa 5, minimum 10 → muncul |
| PO menunggu barang | Purchase order berstatus *ordered* atau *sebagian diterima* | PO kemarin belum diterima → muncul |
| Grafik tren penjualan | Transaksi selesai, dikelompokkan per hari | — |
| Produk terlaris | Item transaksi selesai, dijumlah per barang | — |
| Jam ramai | Jam pada setiap transaksi selesai | — |
| Penjualan per kategori | Item transaksi × kategori produknya | — |
| Kinerja kasir | Transaksi per kasir (admin melihat semua; kasir melihat miliknya) | — |
| Alur pembelian | Progress PO: dipesan vs sudah diterima | — |

Setiap penjualan, retur, atau penerimaan barang yang diposting langsung mengubah angka-angka ini.

## 2. Menyiapkan Data Toko (Master Data)

Menu **Master Data** adalah fondasi. Sekali diisi dengan benar, semua transaksi berikutnya jadi mudah — karena form-form lain mengambil isian otomatis dari sini. Urutan yang disarankan: Kategori Produk → Satuan → Produk → SKU Produk → Supplier → Register Kas → Karyawan → Anggota.

### Kategori Produk

Kelompok barang, misalnya Minuman, Makanan Ringan, atau Kebutuhan Dapur.

| Field | Isi dari mana | Ke mana setelah simpan |
|---|---|---|
| Nama | Diisi manual | Pilihan kategori saat membuat produk & filter toko online |
| Deskripsi | Diisi manual (opsional) | Tampil di daftar kategori |
| Aktif | Pilihan aktif/nonaktif | Kategori nonaktif tidak lagi bisa dipilih saat buat produk |

Kategori yang sudah dipakai produk sebaiknya tidak dihapus — nonaktifkan saja. Laporan penjualan per kategori mengelompokkan transaksi lewat kategori ini.

### Satuan

Satuan barang, misalnya pcs, dus, atau kg.

| Field | Isi dari mana | Ke mana setelah simpan |
|---|---|---|
| Kode | Diisi manual (singkat: pcs, dus) | Label satuan di semua dokumen & laporan |
| Nama | Diisi manual | — |
| Boleh desimal | Pilihan (aktif kalau satuan bisa pecahan, misal kg) | Saat input jumlah di transaksi, nilai desimal diterima hanya untuk satuan yang mengizinkan |
| Aktif | Pilihan aktif/nonaktif | Satuan nonaktif tidak bisa dipilih SKU baru |

Satu trik penting: buat satuan eceran (pcs) dan satuan kemasan (dus) sekaligus, karena nanti berguna saat memecah kemasan besar menjadi eceran (Inventori → Konversi Stok).

### Produk dan SKU Produk

Ini dua hal berbeda yang sering membingungkan:

- **Produk** adalah barangnya — misalnya "Air Mineral". Di sini cukup diisi nama dan kategorinya.
- **SKU Produk** adalah kemasannya — misalnya "Air Mineral botol (pcs)" dan "Air Mineral dus isi 40". Setiap SKU punya kode unik, barcode, harga beli, harga jual, stok, dan stok minimum sendiri.

Harga dan stok selalu menempel di SKU, bukan di produk. Jadi kalau harga botol naik tapi harga dus tetap, yang diubah hanya SKU botol.

**Form Produk:**

| Field | Isi dari mana | Ke mana setelah simpan |
|---|---|---|
| Nama | Diisi manual | Nama tampil di kasir, pembelian, dan toko online |
| Kategori | Pilih dari Master Data → Kategori | Dipakai laporan penjualan per kategori & filter toko online |
| Foto | Upload file gambar (maks 2 MB) | Tampil di katalog toko online |
| Aktif | Pilihan aktif/nonaktif | Produk nonaktif tidak tampil di toko online |

**Form SKU:**

| Field | Isi dari mana | Ke mana setelah simpan |
|---|---|---|
| Produk induk | Pilih dari daftar produk | SKU terikat ke produk itu |
| Kode SKU | Diisi manual, **harus unik** | Identitas kemasan di PO, penerimaan, kasir, laporan |
| Barcode | Diisi manual, **harus unik** (opsional) | Berguna untuk pemindai barcode di kasir |
| Satuan | Pilih dari Master Data → Satuan | Label satuan di semua dokumen |
| Isi kemasan | Diisi manual — berapa unit dasar dalam 1 kemasan ini (botol = 1, dus = 40) | Dipakai saat konversi stok antar kemasan |
| Harga beli | Diisi manual (harga awal); **otomatis terupdate** tiap posting penerimaan barang | Jadi dasar harga pokok (HPP) di Laba Rugi & nilai persediaan di Posisi Keuangan |
| Harga jual | Diisi manual; **otomatis terhitung ulang** saat penerimaan diisi margin (%) | Harga yang terisi otomatis di kasir & toko online |
| Stok awal | Diisi manual sekali di awal | Titik awal stok; berubahnya lewat dokumen (bukan diedit langsung) |
| Stok minimum | Diisi manual | Dasar peringatan "stok menipis" di Dashboard & Laporan Stok |
| Aktif | Pilihan aktif/nonaktif | SKU nonaktif tidak bisa ditransaksikan & tidak tampil di toko online |

Perjalanan data stok SKU: naik saat **posting penerimaan barang**, turun saat **transaksi kasir** dan saat **pesanan online dipack**, berubah sesuai angka saat **penyesuaian**, pindah antar kemasan saat **konversi**, naik lagi saat **retur yang direstock**. Semua perubahan tercatat di Mutasi Stok.

### Supplier

Data pemasok barang.

| Field | Isi dari mana | Ke mana setelah simpan |
|---|---|---|
| Nama | Diisi manual | Pilihan supplier saat membuat PO; tampil di laporan pembelian |
| Nama kontak / telepon / email / alamat | Diisi manual (opsional) | Referensi saat follow-up pesanan |

Supplier wajib ada sebelum membuat purchase order. Supplier nonaktif tidak bisa dipilih PO baru.

### Register Kas

Daftar laci/meja kasir fisik, misalnya "Kasir Depan".

| Field | Isi dari mana | Ke mana setelah simpan |
|---|---|---|
| Kode | Diisi manual, unik | Label di sesi & laporan |
| Nama | Diisi manual | Pilihan saat buka sesi |
| Deskripsi | Diisi manual (opsional) | — |
| Status | Pilihan aktif/nonaktif | Register nonaktif tidak bisa buka sesi baru |

Setiap transaksi penjualan harus terhubung ke salah satu register lewat sesinya (lihat bagian 5). Satu register hanya boleh punya satu sesi terbuka dalam satu waktu.

### Karyawan

Data karyawan untuk keperluan gaji — terpisah dari akun login.

| Field | Isi dari mana | Ke mana setelah simpan |
|---|---|---|
| Nama | Diisi manual | Terisi otomatis (snapshot) saat membuat slip payroll |
| Posisi | Diisi manual | Ikut tersnapshot ke slip |
| Telepon / alamat | Diisi manual (opsional) | — |
| Gaji pokok | Diisi manual | Otomatis terisi di slip payroll periode berikutnya (sesuai snapshot saat slip dibuat) |
| Aktif | Pilihan aktif/nonaktif | Karyawan nonaktif tidak bisa dipilih slip baru |

Karyawan di sini belum tentu punya akun aplikasi; akun login diatur di menu User (bagian 9).

### Anggota (Member)

Daftar pelanggan terdaftar untuk toko online.

| Field | Isi dari mana | Ke mana setelah simpan |
|---|---|---|
| Nomor anggota | **Dibuat otomatis sistem** (MBR-tanggal-nomor) | Identitas anggota di daftar & pesanan |
| Nama lengkap | Diisi manual | Otomatis terisi di checkout anggota |
| Nomor HP | Diisi manual, **harus unik** — dipakai sebagai **username login** toko online | Login anggota di storefront |
| Email / alamat | Diisi manual (opsional); alamat terisi otomatis saat checkout diantar | — |
| Password | Diisi manual (min. 8 karakter) | Login anggota; bisa direset admin atau diganti sendiri oleh anggota |
| Catatan | Diisi manual (opsional) | — |
| Aktif | Pilihan aktif/nonaktif | Anggota nonaktif tidak bisa login & tidak menerima pembagian SHU |

Anggota dibuat oleh admin/kasir, lalu anggota bisa login sendiri di toko online dengan nomor HP-nya. Keuntungan jadi anggota: data nama dan alamat tersimpan, dan transaksinya tercatat atas nama mereka — berguna untuk pembagian SHU.

Menghapus anggota yang sudah pernah belanja tidak menghilangkan datanya — statusnya hanya dinonaktifkan supaya riwayat transaksi tetap utuh.

## 3. Membeli Barang dari Supplier (Pembelian)

Menu **Pembelian** punya dua langkah: pesan dulu, terima kemudian.

```
Purchase Order:  draft ──(Order)──> ordered ──> partially_received ──> received
                    │                    │                │
                    └──(batal)───────────┴──(PO batal)───┘   (hanya sebelum semua diterima)

Penerimaan:      draft ──(Posting: stok naik!)──> posted
                    └──(batal)──> cancelled
```

### Purchase Order (pesan barang)

Purchase order (PO) adalah surat pesanan ke supplier. Nomor PO dibuat otomatis (PO-tanggal-nomor).

| Field | Isi dari mana | Ke mana setelah simpan |
|---|---|---|
| Nomor PO | **Dibuat otomatis sistem** | Identitas di daftar & laporan pembelian |
| Supplier | Pilih dari Master Data → Supplier | Nama supplier ikut ke laporan pembelian |
| Tanggal pesan / tanggal harapan datang | Diisi manual | Dipakai memantau keterlambatan barang |
| Barang | Pilih SKU dari Master Data → SKU; **nama/barang di-snapshot** saat itu | Item PO |
| Jumlah dipesan | Diisi manual | Menjadi sasaran penerimaan; sisa dipesan terhitung otomatis |
| Harga beli per unit | Diisi manual (nilai awal) | Subtotal PO dihitung otomatis; jadi **nilai default** harga beli di penerimaan |
| Catatan | Diisi manual (opsional) | — |

- Saat dibuat, statusnya **draft** — masih bisa diubah, **stok belum berubah**.
- Tekan **Order** untuk mengirim pesanan. Status menjadi **ordered**, dan sistem menyimpan siapa serta kapan yang mengorder.
- Kalau salah input dan barang belum diterima, PO bisa **dibatalkan** dengan alasan.

### Penerimaan Barang (terima barang)

Saat barang datang, catat di **Penerimaan Barang** dengan memilih PO-nya (hanya PO berstatus ordered/sebagian yang bisa dipilih). Nomor penerimaan dibuat otomatis (GR-tanggal-nomor).

| Field | Isi dari mana | Ke mana setelah simpan |
|---|---|---|
| Purchase order | Pilih dari daftar PO | Item PO muncul sebagai baris yang harus diisi |
| Waktu terima | Diisi manual (default sekarang) | Tanggal di laporan penerimaan |
| Jumlah diterima per barang | Diisi manual — boleh **sebagian** dulu (pesan 20, datang 10) | Saat posting: stok SKU naik sebanyak ini |
| Harga beli per unit | **Otomatis dari PO**, boleh dikoreksi jika harga real berbeda | Saat posting: **harga beli SKU terupdate** ke angka ini |
| Margin (%) | Diisi manual (opsional, dilihat dari keuntungan yang diinginkan) | **Preview harga jual** dihitung di form; saat posting: **harga jual SKU dihitung ulang** = harga beli baru × (1 + margin) |
| Catatan | Diisi manual (opsional) | — |

- Saat dibuat, statusnya **draft** — **stok belum berubah**.
- Tekan **Posting** setelah barang dihitung. Saat itulah, sekaligus dalam satu langkah:
  1. **Stok SKU bertambah**,
  2. **Harga beli SKU** terupdate ke harga yang diisi (harga baru jadi HPP untuk transaksi berikutnya),
  3. **Harga jual SKU** dihitung ulang kalau margin diisi (kosong = harga jual tidak berubah),
  4. **PO terupdate**: item tercatat sebagian diterima atau lunas,
  5. **Pergerakan stok tercatat** dengan jenis *penerimaan pembelian*,
  6. Angka di **Laporan Stok, Pembelian, dan Laba Rugi** (lewat nilai persediaan) ikut berubah.
- Penerimaan yang salah bisa **dibatalkan** dengan alasan sebelum dipakai transaksi lain.

## 4. Mengoreksi Stok (Inventori)

Menu **Inventori** untuk merapikan stok di luar jual-beli: hasil opname, barang rusak/hilang, atau pecah kemasan.

### Penyesuaian Stok

Dipakai kalau stok di sistem beda dengan fisik.

```
draft ──(Posting: stok berubah sesuai angka)──> posted
   └──(batal)──> cancelled
```

| Field | Isi dari mana | Ke mana setelah simpan |
|---|---|---|
| Nomor dokumen | **Dibuat otomatis sistem** (SA-tanggal-nomor) | Identitas di daftar & laporan |
| Barang | Pilih SKU | — |
| Stok sebelum | **Dibuat otomatis sistem** — angka stok saat form dibuka | Dipakai menghitung selisih (delta) |
| Stok setelah | Diisi manual — hasil hitung fisik | Saat posting: stok diubah persis ke angka ini |
| Selisih | **Dihitung otomatis** (setelah − sebelum; bisa minus/plus) | Tercatat di laporan penyesuaian & mutasi stok |
| Alasan / catatan | Diisi manual (mis. "stock opname", "rusak") | Dokumen |

Detail penting di saat **Posting**: sistem mengecek stok saat itu masih sama dengan "stok sebelum" yang tersimpan. Kalau di antara buka form dan posting ada transaksi lain yang mengubah stok barang itu, posting ditolak — buat ulang dokumen supaya angkanya akurat. Ini mencegah koreksi menimpa perubahan yang valid.

Cocok untuk hasil stock opname atau mencatat barang rusak/hilang.

### Konversi Stok

Dipakai untuk memecah/menggabung kemasan. Contoh: 1 dus air mineral (isi 40) dipecah menjadi 40 botol eceran.

| Field | Isi dari mana | Ke mana setelah simpan |
|---|---|---|
| Kemasan asal + jumlah | Pilih SKU (mis. dus), isi jumlah yang dipecah (1) | Saat posting: **stok dus −1** |
| Kemasan tujuan + jumlah | Pilih SKU (mis. botol), isi hasilnya (40) | Saat posting: **stok botol +40** |
| Catatan per baris | Diisi manual (opsional) | — |

Seperti biasa: buat dokumen (draft), lalu **Posting** baru stok berpindah. Pergerakan tercatat dua arah: *keluar konversi* dan *masuk konversi*.

### Laporan Stok dan Mutasi Stok

- **Laporan Stok** menampilkan posisi terkini semua SKU. Semuanya dihitung otomatis: stok sekarang (hasil semua dokumen), harga beli/jual terakhir, nilai persediaan (stok × harga beli), status menipis (dibanding stok minimum), dan **jumlah yang masih dipesan ke supplier** (dipesan − diterima dari PO aktif).
- **Mutasi Stok** adalah buku harian stok. Setiap baris terisi otomatis dari dokumen: *penerimaan pembelian* (barang masuk), *penjualan* (barang keluar — kasir & pesanan online yang selesai), *penyesuaian*, *konversi masuk/keluar*, *retur masuk*, dan *koreksi* (hasil pembatalan retur). Lengkap dengan stok sebelum–sesudah, tanggal, dan pelakunya. Kalau ada selisih stok yang misterius, mulailah mencari dari sini.

## 5. Jualan di Kasir

Menu **Kasir** adalah tempat kerja utama setiap hari. Urutannya: buka sesi → jualan → (retur kalau ada) → catat uang kas manual → tutup sesi.

### Sesi Register (buka dan tutup laci)

Sebelum jualan, **buka sesi** di register yang dipakai.

```
open ──(Tutup Sesi)──> closed   (satu register hanya satu sesi open)
```

**Form buka sesi:**

| Field | Isi dari mana | Ke mana setelah simpan |
|---|---|---|
| Nomor sesi | **Dibuat otomatis sistem** (CRS-tanggal-nomor) | Ditautkan ke semua transaksi & pergerakan kas sesi ini |
| Register | Pilih dari Master Data → Register Kas | Transaksi terikat ke laci fisik itu |
| Uang modal awal | Diisi manual — hitung uang yang ditaruh di laci | Menjadi angka awal perhitungan tutup sesi |
| Catatan awal | Diisi manual (opsional) | — |

**Form tutup sesi:**

| Field | Isi dari mana | Ke mana setelah simpan |
|---|---|---|
| Uang fisik dihitung | Diisi manual — hitung uang di laci dan ketik angkanya | Dibandingkan dengan angka sistem |
| Uang seharusnya | **Dihitung otomatis sistem**, rumusnya di bawah | Ditampilkan saat tutup |
| Selisih | **Dihitung otomatis** (fisik − seharusnya) | Tersimpan di sesi; selisih nol berarti beres |
| Catatan penutupan | Diisi manual (opsional) | — |

Rumus uang seharusnya:

```
modal awal
+ penjualan tunai selesai di sesi ini
− semua retur selesai di sesi ini
+ pergerakan kas masuk
− pergerakan kas keluar/drop/petty
= uang seharusnya di laci
```

> Catatan: hanya penjualan **tunai** yang dihitung — uang transfer/Midtrans tidak pernah lewat laci fisik, jadi tidak masuk rumus. Penjualan online "bayar di toko" dihitung tunai karena uangnya benar-benar masuk laci. Kalau dana non-tunai kebetulan ikut disetor ke laci, catat lewat **Kas Masuk** agar selisih tetap nol.

### Transaksi Baru

Halaman kasir untuk melayani pembeli:

| Field | Isi dari mana | Ke mana setelah simpan |
|---|---|---|
| Barang | Pilih SKU aktif (cari nama/kode/barcode) | Nama & kemasan di-snapshot ke transaksi |
| Jumlah | Diisi manual; dicek langsung vs stok tersedia | **Stok langsung berkurang saat simpan** + mutasi stok tercatat |
| Harga jual per item | **Otomatis** dari harga jual SKU saat itu | Ikut tersnapshot ke transaksi — perubahan harga SKU nanti tidak mengubah transaksi lama |
| Harga pokok (HPP) | **Otomatis** dari harga beli SKU saat itu (tidak terlihat kasir) | Tersimpan di item transaksi — inilah sumber **harga pokok di Laba Rugi** |
| Anggota | Pilih anggota (opsional) — untuk pembeli terdaftar | Transaksi tercatat atas nama anggota → **dihitung di pembagian SHU** |
| Uang dibayar | Diisi manual; wajib ≥ total | Kembalian dihitung otomatis (dibayar − total) |
| Metode bayar | Pilih **tunai** atau **transfer** (mengikuti mode toko; Midtrans ada di mode *midtrans*) | Tunai: ikut kas laci. Transfer: isi tujuan transfer + nomor bukti (wajib), tidak masuk kas tunai |
| Catatan | Diisi manual (opsional) | — |
| Nomor struk | **Dibuat otomatis sistem** (TRX-tanggal-nomor) | Struk tercetak/tercatat, tampil di Riwayat Penjualan |

Saat **Simpan** ditekan: stok turun, mutasi stok tercatat, angka dashboard & laporan harian langsung naik.

**Pembayaran Midtrans (mode *midtrans*)**: alurnya sedikit berbeda — kasir menyiapkan keranjang dan membuat tagihan QRIS/Snap (validasi stok & sesi tetap dilakukan, tapi stok **belum berubah**). Pelanggan bayar lewat aplikasi pembayaran. Saat pembayaran dinyatakan **lunas** (notifikasi otomatis dari Midtrans, atau kasir menekan refresh di halaman tagihan), sistem mencatat transaksinya: **stok dipotong saat lunas**, HPP di-snapshot saat itu, dan transaksi lahir di **sesi yang sedang terbuka** (bukan sesi saat tagihan dibuat) dengan metode *midtrans* — lunas pas, tanpa kembalian. Kalau tagihan hangus (lewat waktu/dibatalkan/ditolak), tidak ada transaksi dan stok tidak berubah. Pembayaran ganda/out-of-order ditolak otomatis, jadi tidak double.

### Riwayat Penjualan

Daftar semua transaksi: yang selesai (**completed**) maupun yang dibatalkan (**cancelled**). Kasir hanya melihat transaksinya sendiri; admin melihat semua.

```
completed ──(batal dengan alasan)──> cancelled
```

Transaksi yang salah input bisa dibatalkan dengan alasan. Catatan penting: pembatalan **mengubah status saja** — stok tidak otomatis kembali dan uang tidak otomatis tercatat. Kalau stok perlu dikoreksi, pakai Penyesuaian Stok. Transaksi batal tidak ikut laporan apa pun.

### Retur Penjualan

Kalau pembeli mengembalikan barang dari transaksi yang sudah selesai:

| Field | Isi dari mana | Ke mana setelah simpan |
|---|---|---|
| Transaksi asal | Pilih dari Riwayat Penjualan (harus berstatus selesai) | Sistem menampilkan daftar barang yang **masih bisa diretur** — dihitung otomatis dari sisa yang belum pernah diretur, jadi barang yang sama tidak bisa diretur dua kali |
| Barang & jumlah retur | Pilih baris + isi jumlah (≤ sisa) | — |
| Barang masuk stok lagi? | Pilihan per item: **restock** (masih bagus) atau tidak (rusak/cacat) | Restock: stok naik otomatis saat retur selesai + mutasi stok *retur*. Tidak restock: uang balik tercatat, stok tidak berubah |
| Alasan (dokumen & per item) | Diisi manual | Dokumen |
| Sesi laci | **Otomatis** — sesi yang sedang terbuka | Pengembalian uang tercatat di sesi ini → ikut rumus tutup sesi |
| Nomor retur | **Dibuat otomatis sistem** (RET-tanggal-nomor) | Daftar retur & laporan |

```
completed ──(batal, hanya selagi sesi laci masih terbuka)──> cancelled
```

Retur yang salah bisa dibatalkan selama sesi laci yang dipakai masih terbuka; kalau barangnya sempat masuk stok, pembatalan akan **mengurangi stok kembali** dan mencatat mutasi *koreksi*.

### Pergerakan Kas

Mencatat uang laci di luar jualan.

| Jenis | Arah di sesi | Contoh |
|---|---|---|
| Kas masuk | masuk (+) | Tarik uang tambahan dari brankas ke laci |
| Cash drop | keluar (−) | Setoran uang tunai ke brankas di siang hari |
| Kas keluar | keluar (−) | Ambil uang dari laci untuk keperluan lain |
| Petty cash | keluar (−) | Belanja kecil operasional (parkir, plastik) |

| Field | Isi dari mana | Ke mana setelah simpan |
|---|---|---|
| Nomor | **Dibuat otomatis sistem** (CDM-tanggal-nomor) | Daftar pergerakan |
| Sesi | **Otomatis** — wajib sesi yang sedang terbuka | Ikut perhitungan tutup sesi sesuai arahnya |
| Waktu | Diisi manual (default sekarang) | — |
| Jumlah & alasan | Diisi manual | Laporan arus kas (kas keluar) & tutup sesi |
| Status | Langsung *completed* | Pergerakan batal tidak dihitung di tutup sesi |

Pergerakan yang salah bisa dibatalkan **selama sesinya masih terbuka**.

## 6. Jualan Online (Storefront dan Pesanan Online)

Toko online punya dua sisi: halaman yang dilihat pelanggan, dan halaman **Pesanan Online** di aplikasi kasir untuk memprosesnya.

Perjalanan stok pesanan online berbeda dari kasir — stok **belum dipotong saat checkout**, hanya dicek ketersediaannya. Stok baru benar-benar berkurang saat pesanan dipack, dan transaksi penjualannya baru lahir saat pesanan selesai.

### Sisi pelanggan: katalog, keranjang, checkout

Pelanggan membuka halaman toko (`/shop`) tanpa perlu login untuk melihat-lihat:

- **Katalog** menampilkan produk yang aktif dan stoknya tersedia, lengkap dengan foto, harga, dan kategori. Ada juga info toko (jam buka, rekening, e-wallet, ongkir, WhatsApp) — semuanya dari menu Pengaturan Toko.
- Pelanggan memasukkan barang ke **keranjang**, lalu **checkout**. Isian formnya:

| Field | Isi dari mana | Ke mana setelah checkout |
|---|---|---|
| Nama & nomor HP | **Tamu**: diisi manual di form. **Anggota**: otomatis dari akunnya (tidak bisa — dan tidak perlu — mengaku-aku jadi anggota lain; identitas diambil dari login) | Menempel di pesanan; anggota dihubungkan ke datanya |
| Alamat | Diisi manual (wajib untuk diantar); anggota terisi otomatis | Catatan kurir |
| Ambil di tempat / diantar | Pilihan | Menentukan jalur status (Ready vs Deliver) & ongkir |
| Bayar di toko / transfer | Pilihan | Transfer: halaman sukses menampilkan rekening/e-wallet toko + total tagihan |
| Barang | Dari keranjang (SKU + jumlah) | Stok **dicek** ketersediaannya, belum dipotong |
| Nomor pesanan | **Dibuat otomatis sistem** (OO-tanggal-nomor) | Halaman sukses + badge di menu Pesanan Online |

- Anggota bisa membuka halaman **profil** untuk melihat datanya dan **ganti password** sendiri. Pesanan transfer juga bisa dibayar via **QRIS/Midtrans** — lunasnya otomatis terdeteksi.

### Sisi kasir: memproses pesanan

Setiap ada pesanan baru, angka badge muncul di menu **Pesanan Online** — jadi kasir langsung tahu tanpa harus mengecek manual.

```
              ┌──(Pack: stok DIPOTONG di sini)──> packing
new ──────────┤                                      ├──(Ready: ambil di tempat)──> ready ──(Complete)──> completed
              │                                      └──(Deliver: diantar*)──> delivering ──(Complete)──> completed
              └──(batal dengan alasan, hanya dari new/packing)──> cancelled
                                    (*hanya jika pesanan sudah lunas, atau metodenya bayar di toko)
```

| Aksi | Kapan ditekan | Efek datanya |
|---|---|---|
| **Pack** | Barang mulai disiapkan | **Stok dipotong + mutasi stok tercatat** di langkah ini |
| **Ready** | Pesanan ambil-di-tempat siap diambil | — |
| **Deliver** | Kurir berangkat | — |
| **Complete** | Pesanan diserahkan/diterima | **Transaksi penjualan lahir otomatis** di sesi laci kasir yang sedang terbuka — tercatat sebagai penjualan channel *online*, lunas pas, di atas nama anggota (kalau ada). Metode bayarnya mengikuti pesanan: **bayar di toko** dicatat tunai (uangnya masuk laci), **transfer/QRIS** dicatat non-tunai. Karena itu: sebelum menekan Complete, kasir harus punya **sesi laci terbuka** |
| **Mark Paid** | Pesanan transfer yang pembayarannya dikonfirmasi manual (cek mutasi bank) | Status pesanan jadi lunas |
| **Batal** | Pesanan tak bisa dipenuhi (hanya selagi baru/disiapkan) | Kalau sudah dipack, **stok dikembalikan** otomatis |

Karena transaksi baru lahir saat Complete, omzet online muncul di Dashboard/laporan pada hari pesanan selesai — bukan hari checkout. Pesanan yang dibayar transfer/QRIS juga tetap terhitung sebagai penjualan — uangnya tidak masuk kas fisik laci, tapi tetap dihitung di laporan.

### Pengaturan Toko

Di menu **Pengaturan Toko** (bagian 9), atur nama toko, jam operasional, rekening, e-wallet, ongkir, nomor WhatsApp, gambar QRIS, dan **mode pembayaran kasir** (manual/midtrans) — semuanya langsung tampil atau berlaku di halaman toko online & kasir.

## 7. Gaji (Payroll)

### Slip Payroll

Buat slip gaji per karyawan per periode. Nomor slip dibuat otomatis (PAY-tanggal-nomor).

| Field | Isi dari mana | Ke mana setelah simpan |
|---|---|---|
| Karyawan | Pilih dari Master Data → Karyawan | Nama, posisi, dan **gaji pokok terisi otomatis** (di-snapshot dari data karyawan saat slip dibuat; mengubah gaji pokok karyawan belakangan tidak mengubah slip lama) |
| Periode mulai–selesai | Diisi manual | Kelompok slip di Laporan Payroll & dipakai Laba Rugi (beban gaji) |
| Bonus | Diisi manual (default 0) | — |
| Potongan | Diisi manual (default 0) | — |
| Gaji bersih | **Dihitung otomatis** (pokok + bonus − potongan) | Tampil di slip, Laporan Payroll, dan Laba Rugi |
| Status | **Langsung diatur sistem** — slip lahir dengan status *Selesai* | Slip yang **dibatalkan** keluar dari Laporan Payroll dan beban gaji di Laba Rugi |

> Catatan: slip payroll **tidak bisa diedit** setelah dibuat. Kalau salah buat, **batalkan** slipnya dengan alasan lewat menu aksi (titik tiga) di daftar slip, lalu buat ulang slip yang benar — slip batal tidak ikut laporan apa pun. Pembatalan tercatat di Audit Log.

### Laporan Payroll

Rekap gaji per periode — semua dihitung otomatis dari slip berstatus *Selesai* (slip yang dibatalkan tidak dihitung): berapa slip, total gaji pokok, bonus, potongan, dan gaji bersih. Berguna untuk melihat total beban gaji, dan angka inilah yang dipakai baris **beban gaji** di Laba Rugi (slip yang periode akhirnya jatuh dalam rentang laporan).

## 8. Keuangan dan SHU

Menu **Keuangan** mencatat uang di luar kasir: biaya operasional, pemasukan lain, dan sejenisnya.

### Kategori Keuangan

| Field | Isi dari mana | Ke mana setelah simpan |
|---|---|---|
| Nama | Diisi manual | Pilihan saat membuat transaksi |
| Jenis kategori | Pilihan **pemasukan/pengeluaran** | Hanya kategori dengan jenis cocok yang muncul saat transaksi dibuat |
| Aktif | Pilihan aktif/nonaktif | Kategori nonaktif tidak bisa dipilih transaksi baru |

Kategori yang sudah dipakai transaksi tidak bisa dihapus.

### Transaksi Keuangan

| Field | Isi dari mana | Ke mana setelah simpan |
|---|---|---|
| Jenis | Pilihan **pemasukan/pengeluaran** (tidak bisa diubah setelah dibuat) | Menentukan arah di Laba Rugi & Arus Kas |
| Kategori | Pilih dari kategori dengan jenis yang cocok | — |
| Jumlah | Diisi manual | Laporan |
| Tunai / non-tunai | Pilihan — transfer bank = non-tunai | **Tunai** masuk Arus Kas & Posisi Keuangan (kas). **Non-tunai** tetap masuk Laba Rugi tapi tidak masuk arus kas tunai |
| Deskripsi & waktu | Diisi manual | Laporan |
| Nomor | **Dibuat otomatis sistem** | Daftar transaksi |

Transaksi bisa diedit (kecuali jenisnya) dan bisa **dibatalkan dengan alasan** — transaksi yang dibatalkan tidak ikut laporan apa pun.

### Laba Rugi, Arus Kas, Posisi Keuangan

Tiga laporan ini terisi **sepenuhnya otomatis** dari data yang sudah ada — tidak perlu input manual. Ini peta sumbernya (dokumen mana menghasilkan baris mana):

**Laba Rugi** (per rentang tanggal):

| Baris | Sumber datanya |
|---|---|
| Pendapatan kotor | Semua transaksi kasir & online yang selesai dalam rentang |
| Retur | Semua retur selesai dalam rentang (dikurangkan) |
| Pendapatan bersih | Otomatis: kotor − retur |
| Harga pokok barang (HPP) | **HPP yang tersimpan di setiap item transaksi** (harga beli SKU saat transaksi itu terjadi) — termasuk penyesuaian untuk retur yang direstock |
| Laba kotor | Otomatis: bersih − HPP |
| Beban gaji | Gaji bersih dari slip payroll berstatus *Selesai* yang periodenya jatuh dalam rentang (slip batal tidak dihitung) |
| Beban lain-lain | Semua transaksi Keuangan jenis *pengeluaran* yang selesai (tunai maupun non-tunai) |
| Laba/rugi bersih | Otomatis: laba kotor − beban gaji − beban lain |

**Arus Kas** (per rentang tanggal) — laporan uang tunai:

| Baris | Sumber datanya |
|---|---|
| Kas masuk dari penjualan | Penjualan selesai yang dibayar **tunai** dalam rentang (kasir & pesanan online bayar di toko). Penjualan transfer/Midtrans tidak masuk — uangnya tidak lewat laci |
| Kas masuk manual | Transaksi Keuangan *pemasukan* yang ditandai **tunai** |
| Kas keluar laci | Pergerakan kas keluar/drop/petty cash dalam rentang |
| Kas keluar manual | Transaksi Keuangan *pengeluaran* yang ditandai **tunai** |
| Arus kas bersih | Otomatis: masuk − keluar |

> Catatan: penjualan non-tunai tetap dihitung sebagai **omzet** (Dashboard, Laba Rugi, laporan harian/bulanan), hanya saja tidak masuk laporan arus kas tunai ini.

**Posisi Keuangan** (per satu tanggal, angka kumulatif sejak awal):

| Baris | Sumber datanya |
|---|---|
| Kas | Semua **penjualan tunai** selesai + pemasukan manual tunai − pergerakan laci keluar − pengeluaran manual tunai, diakumulasi sampai tanggal itu |
| Nilai persediaan | Stok terkini setiap SKU × harga beli SKU saat ini |
| Total aset | Otomatis: kas + persediaan |
| Laba kumulatif | Rumus Laba Rugi, diakumulasi sampai tanggal itu |

Ketiganya bisa difilter rentang tanggal, cocok dibuka tiap akhir bulan sebelum mengambil keputusan.

### SHU (Sisa Hasil Usaha)

Di akhir periode (misalnya tutup tahun buku), laba bisa dibagi ke anggota lewat menu **SHU**:

```
draft ──(Hitung/Calculate)──> calculated ──(Finalize)──> finalized (terkunci)
  │                               │
  └──(batal dengan alasan)────────┘
```

1. **Buat periode** — isi nama periode, tanggal mulai-selesai, dan persentase pembagian (anggota, cadangan, pengurus, pengawas, sosial, pendidikan; totalnya bebas tapi harus menggambarkan kebijakan koperasi).
2. **Hitung (Calculate)** — sistem mengambil angka-angka ini otomatis:
   - **Laba bersih periode** dari Laba Rugi (pendapatan − retur − HPP − gaji − beban lain) sebagai dasar;
   - **Pool pembagian** = isi pool manual kalau kamu mengisinya, kalau tidak kosong = laba bersih otomatis;
   - **Bagian anggota** = pool × persentase anggota;
   - **Belanja tiap anggota** = penjualan yang menempel atas namanya dalam rentang periode (transaksi kasir yang memilih anggota itu + pesanan online anggota) dikurangi retur yang menempel padanya. Anggota yang nonaktif tidak ikut; anggota tanpa belanja tidak menerima bagian.
   - **Porsi tiap anggota** = proporsional: belanja bersih anggota ÷ total belanja semua anggota × bagian anggota. Hasilnya tersimpan sebagai rincian distribusi per anggota.
3. Periksa hasilnya di **detail periode** dan **daftar distribusi**. Kalau rentangnya salah, **batalkan** dan buat ulang.
4. Kalau sudah benar, **Finalize** untuk mengunci — periode yang dikunci tidak bisa diubah lagi.
5. Hasilnya bisa diunduh sebagai **file CSV** untuk dibagikan atau dicetak.

## 9. Pengaturan

### Pengaturan Toko

Identitas toko yang tampil di toko online (lihat bagian 6):

| Field | Isi dari mana | Ke mana setelah simpan |
|---|---|---|
| Nama toko, jam operasional | Diisi manual | Tampil di halaman toko |
| Rekening & e-wallet | Diisi manual (satu kolom teks bebas, boleh beberapa baris) | Tampil di petunjuk pembayaran transfer |
| Ongkir | Diisi manual (angka) | Otomatis masuk total tagihan pesanan diantar |
| WhatsApp | Diisi manual | Tombol chat di toko online |
| Gambar QRIS | Upload gambar (maks 2 MB) | Tampil di petunjuk pembayaran |
| Mode pembayaran kasir | Pilihan **manual** (pencatatan transfer manual) atau **midtrans** (QRIS/Snap aktif di kasir & toko online) | Kasir Transaksi Baru menyesuaikan pilihan metode bayar yang tersedia |

### User dan Role

- **User** — akun login karyawan (nama, email, password, peran). Akun yang sudah tidak dipakai sebaiknya dinonaktifkan, bukan dihapus — karena riwayat transaksi tetap menunjukkan nama pelakunya.
- **Role** — peran dan hak aksesnya. Hak akses per luasan menu: dashboard, master data, kasir (point flow), pembelian (procurement), inventori, payroll, keuangan, laporan, toko online (pesanan, anggota), user, dan SHU. Role bawaan sistem (admin = semua akses, kasir = terbatas) tidak bisa dihapus. Mengganti hak akses sebuah role langsung berlaku untuk semua user dengan role tersebut.
- Ganti password sendiri ada di menu akun di pojok kiri bawah.

### Audit Log

Catatan otomatis atas kejadian penting: siapa login, siapa membuat/membatalkan transaksi, siapa memposting penerimaan/penyesuaian, siapa membuka/menutup sesi, siapa menekan aksi pesanan online, hingga pembayaran Midtrans yang lunas. Isinya hanya bisa dibaca — tidak bisa diubah atau dihapus. Kalau ada kejadian janggal (misalnya transaksi batal misterius), cari tahu siapa pelakunya di sini.

### Akun Saya dan Help Desk

- Lewat menu user di pojok kiri bawah, setiap karyawan bisa melihat profilnya, **ganti password**, dan **keluar**.
- **Help Desk** berisi kumpulan pertanyaan yang sering ditanyakan (bisa dicari dan difilter per topik: kasir, pembelian, inventori, payroll, laporan, akses) plus kontak bantuan.

## 10. Alur Sehari-hari yang Disarankan

Untuk operasional normal satu hari kerja:

**Pagi (persiapan):**

1. Login dan lihat Dashboard.
2. Cek menu Pesanan Online — proses pesanan yang masuk semalam (badge angkanya).
3. Buka Sesi Register di kasir yang dipakai, isi modal awal.
4. Kalau ada barang datang, catat Penerimaan Barangnya dan posting.

**Siang (jualan):**

5. Layani pembeli lewat Transaksi Baru.
6. Pesanan online yang siap: update statusnya (Pack → Ready/Deliver → Complete).
7. Retur atau pergerakan kas dicatat saat itu juga, jangan ditunda.

**Malam (tutup):**

8. Tutup Sesi Register — hitung uang fisik, cocokkan dengan angka sistem.
9. Lihat Dashboard: omzet hari ini sudah sesuai harapan?

**Berkala (mingguan/bulanan):**

10. Buat slip gaji tiap akhir periode gaji, cek Laporan Payroll.
11. Catat biaya operasional di Transaksi Keuangan.
12. Buka Laba Rugi, Arus Kas, dan Posisi Keuangan untuk evaluasi.
13. Tutup tahun buku lewat SHU: buat periode, hitung, periksa, kunci.

## 11. Istilah Status yang Sering Ditemui

Dokumen dan pesanan punya status yang menunjukkan posisinya. Yang paling sering muncul:

- **Draft** — baru dicatat, belum berpengaruh apa-apa (stok dan uang belum berubah). Masih bisa diubah atau dibatalkan.
- **Ordered** — pesanan ke supplier sudah dikirim, tinggal menunggu barang.
- **Partially received / Received** — PO diterima sebagian / lunas barangnya.
- **Posted** — sudah resmi dicatat; stok sudah berubah. Dokumen yang sudah diposting tidak bisa diubah lagi.
- **Completed / Posted / Received** — transaksi selesai dan berpengaruh penuh.
- **Cancelled** — dibatalkan dengan alasan. Data tetap tersimpan sebagai riwayat, tapi tidak ikut perhitungan (stok, kas, laporan).
- **New → Packing → Ready/Delivering → Completed** — perjalanan pesanan online dari masuk sampai selesai (stok dipotong saat *packing*, transaksi penjualan lahir saat *completed*).
- **Open / Closed** — sesi laci sedang berjalan atau sudah ditutup.
- **Paid / Unpaid** — pesanan online sudah dibayar atau belum.
- **Pending → Settlement / Expired** — perjalanan tagihan Midtrans: menunggu dibayar → lunas (transaksi lahir) atau hangus (tidak terjadi apa-apa).

Prinsip umumnya: selama masih draft/baru, hampir semua bisa diperbaiki. Setelah diposting atau dikunci (finalize), yang bisa dilakukan hanya membatalkan dengan alasan — dan pembatalan selalu meninggalkan jejak di audit log. Slip payroll tidak bisa diedit — kalau salah, batalkan dengan alasan lalu buat ulang slip yang benar.
