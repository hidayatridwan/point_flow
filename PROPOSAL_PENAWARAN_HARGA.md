# Proposal Penawaran Harga Pengembangan Aplikasi POS MVP

## 1. Ringkasan

Proposal ini mencakup pengembangan aplikasi POS berbasis web untuk membantu toko kecil menjalankan operasional harian, mulai dari transaksi kasir, manajemen produk, pengelolaan stok, pembelian barang, penerimaan barang, koreksi stok, pengelolaan karyawan, slip gaji, dashboard, sampai laporan operasional.

Scope proposal mengikuti dokumentasi project pada `spec.md` dan `api-flow.md`, dengan target utama sebagai aplikasi MVP yang sederhana, rapi, mudah digunakan untuk demo, dan mudah dikembangkan ke tahap berikutnya.

## 2. Daftar Fitur

| No | Modul | Fitur |
| --- | --- | --- |
| 1 | Authentication & User Management | Login, refresh token, logout, current user, manajemen user, role admin dan cashier, status user aktif/nonaktif. |
| 2 | Dashboard | Ringkasan penjualan hari ini, jumlah transaksi hari ini, pendapatan hari ini, produk hampir habis, purchase order pending, stock adjustment draft, dan jumlah karyawan aktif. |
| 3 | Product Category Management | CRUD kategori produk, deskripsi kategori, status aktif/nonaktif, dan soft delete. |
| 4 | Product Management | CRUD produk, SKU unik, harga beli, harga jual, stok, minimum stok, satuan produk, kategori produk, status aktif/nonaktif, dan peringatan stok rendah. |
| 5 | Inventory & Stock Movements | Pencatatan setiap perubahan stok final, stock in, stock out, sale, purchase receipt, adjustment, correction, stok sebelum dan stok sesudah. |
| 6 | Supplier Management | CRUD supplier, kontak supplier, telepon, email, alamat, status aktif/nonaktif, dan soft delete. |
| 7 | Purchase Orders | Pembuatan purchase order, item pembelian, nomor PO otomatis, status draft/ordered/partially received/received/cancelled, subtotal, dan pembatalan PO. |
| 8 | Goods Receipts | Penerimaan barang dari purchase order, penerimaan parsial, validasi jumlah diterima, posting penerimaan, update stok, update progress PO, dan pencatatan stock movement. |
| 9 | Stock Adjustments | Dokumen koreksi stok, alasan adjustment, item adjustment, stok sebelum, stok sesudah, validasi stok tidak negatif, posting adjustment, dan pembatalan adjustment. |
| 10 | Point of Sale | Pencarian produk berdasarkan nama/SKU, cart kasir, ubah jumlah item, hapus item, subtotal otomatis, total pembayaran, uang dibayar, kembalian, simpan transaksi, dan struk sederhana. |
| 11 | Transaction History | Daftar transaksi, detail transaksi, filter tanggal, akses semua transaksi untuk admin, dan pembatasan transaksi milik sendiri untuk cashier. |
| 12 | Employee Management | CRUD karyawan, posisi, telepon, alamat, gaji pokok, status aktif/nonaktif, dan soft delete. |
| 13 | Payroll Slips | Pembuatan slip gaji per karyawan, periode gaji, bonus, potongan, perhitungan gaji bersih, detail slip, dan cetak/tampilan slip sederhana. |
| 14 | Reports | Laporan penjualan harian, penjualan bulanan, stok barang, purchase order, goods receipt, stock adjustment, dan payroll. |
| 15 | API Flow & Testing Data | Endpoint API terstruktur, Postman collection, flow pengujian end-to-end, dan data dummy untuk kebutuhan demo/testing. |

## 3. Scope Teknis MVP

Termasuk dalam penawaran:

- Struktur database POS sesuai kebutuhan MVP.
- Backend API untuk seluruh modul utama.
- Validasi input, pesan sukses, dan pesan error.
- Role-based access untuk admin dan cashier.
- Soft delete untuk master data.
- Dokumen transaksi dengan status draft, posted, completed, cancelled, atau status lain sesuai modul.
- Perhitungan subtotal, total, line total, quantity delta, net salary, dan agregasi laporan.
- UI web dasar yang bersih, modern, responsif, dan cukup untuk demo MVP.
- Dokumentasi alur penggunaan API dan urutan pengujian.
- Data dummy untuk testing.
- Instruksi menjalankan aplikasi secara lokal.

Tidak termasuk dalam MVP:

- Multi cabang.
- Barcode scanner fisik.
- Integrasi payment gateway.
- Akuntansi lengkap.
- Hutang supplier atau pembayaran invoice pembelian.
- Absensi karyawan.
- Pajak otomatis.
- Export Excel/PDF kompleks.
- Integrasi printer thermal secara native.
- Hosting berbayar, domain, SSL production, dan biaya infrastruktur pihak ketiga.
- Migrasi data real dari sistem lama.
- Maintenance bulanan, SLA, atau support jangka panjang.
- Training onsite.

## 4. Estimasi Harga

Estimasi harga wajar untuk scope MVP demo seperti dokumentasi:

| Paket | Estimasi Harga | Keterangan |
| --- | ---: | --- |
| MVP Demo Minimum | Rp15.000.000 | Cocok untuk demo aplikasi, flow utama berjalan, UI dasar, dokumentasi, dan pengujian lokal/staging. |
| MVP Lebih Lengkap | Rp20.000.000 - Rp25.000.000 | Cocok jika UI dibuat lebih matang, validasi diperluas, polishing lebih banyak, dan ada beberapa siklus revisi. |
| Siap Produksi | Mulai Rp30.000.000+ | Perlu tambahan deployment production, backup, monitoring, security hardening, support, dan proses UAT lebih formal. |

## 5. Rekomendasi Harga Penawaran

Harga yang direkomendasikan untuk proposal ini adalah:

**Rp15.000.000**

Harga Rp15 juta cocok apabila proyek diposisikan sebagai **MVP demo** dengan batasan scope yang jelas. Dengan cakupan fitur yang cukup banyak, harga ini sebaiknya tidak mencakup deployment production penuh, maintenance bulanan, support panjang, integrasi hardware, migrasi data real, atau revisi besar di luar dokumentasi.

Jika klien meminta aplikasi benar-benar siap operasional produksi dengan support dan hardening tambahan, harga Rp15 juta terlalu rendah. Untuk kondisi tersebut, penawaran sebaiknya dinaikkan minimal ke range Rp30 juta ke atas, tergantung ekspektasi deployment, support, keamanan, dan kualitas UI.

## 6. Estimasi Waktu Pengerjaan

Estimasi waktu pengerjaan untuk MVP demo:

**3 sampai 5 minggu kerja**

Estimasi ini berlaku dengan asumsi kebutuhan fitur mengikuti dokumentasi yang sudah ada dan tidak ada perubahan scope besar selama pengerjaan.

## 7. Skema Pembayaran

Rekomendasi termin pembayaran:

| Termin | Persentase | Keterangan |
| --- | ---: | --- |
| DP | 50% | Dibayarkan saat project dimulai. |
| Progress | 30% | Dibayarkan setelah fitur utama dapat didemokan. |
| Pelunasan | 20% | Dibayarkan setelah serah terima MVP. |

## 8. Kriteria Serah Terima

Project dianggap selesai untuk scope MVP apabila:

- Admin dapat login dan mengelola user.
- Admin dapat mengelola kategori produk, produk, supplier, karyawan, dan slip gaji.
- Admin dapat membuat purchase order, goods receipt, stock adjustment, dan melihat laporan.
- Cashier dapat melakukan transaksi penjualan dan melihat riwayat transaksi miliknya.
- Stok produk berubah sesuai flow final: sale completed, goods receipt posted, dan stock adjustment posted.
- Dashboard dan laporan utama dapat menampilkan data dari transaksi yang sudah dibuat.
- Aplikasi dapat dijalankan secara lokal untuk demo.
- Dokumentasi API dan urutan pengujian tersedia.

## 9. Catatan

Proposal ini menggunakan asumsi bahwa target project adalah MVP demo sesuai dokumentasi internal. Perubahan fitur, integrasi eksternal, kebutuhan produksi, atau permintaan custom di luar scope dapat dihitung sebagai pekerjaan tambahan.
