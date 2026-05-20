# Proposal Penawaran Harga Pengembangan Aplikasi Point of Sale (POS)

## 1. Ringkasan

Proposal ini mencakup pengembangan aplikasi POS berbasis web untuk membantu menjalankan operasional harian, mulai dari transaksi kasir/penjualan, dukungan barcode/QR code untuk pencarian item, manajemen produk dan SKU/kemasan, pengelolaan satuan, pengelolaan stok per SKU, pembelian barang, penerimaan barang, koreksi stok, konversi stok antar kemasan, pengelolaan karyawan, slip gaji, dashboard, sampai laporan operasional.

## 2. Daftar Fitur

| No | Modul | Fitur |
| --- | --- | --- |
| 1 | Authentication & User Management | Login, refresh token, logout, current user, manajemen user, role admin dan kasir, status user aktif/nonaktif. |
| 2 | Dashboard | Ringkasan penjualan hari ini, jumlah transaksi hari ini, pendapatan hari ini, produk hampir habis, purchase order pending, stock adjustment draft, dan jumlah karyawan aktif. |
| 3 | Product Category Management | CRUD kategori produk, deskripsi kategori, status aktif/nonaktif, dan soft delete. |
| 4 | Unit & Product SKU Management | CRUD satuan, produk induk, SKU/kemasan produk, SKU unik, barcode/QR code per SKU, harga beli/jual per SKU, stok per SKU, minimum stok per SKU, isi kemasan, status aktif/nonaktif, dan peringatan stok rendah. |
| 5 | Inventory & Stock Movements | Pencatatan setiap perubahan stok final per SKU, stok masuk, stok keluar, penjualan, penerimaan pembelian, koreksi stok, konversi stok masuk/keluar, stok sebelum, dan stok sesudah. |
| 6 | Supplier Management | CRUD supplier, kontak supplier, telepon, email, alamat, status aktif/nonaktif, dan soft delete. |
| 7 | Purchase Orders | Pembuatan purchase order berbasis SKU/kemasan, item pembelian, nomor PO otomatis, status draft/ordered/partially received/received/cancelled, subtotal, dan pembatalan PO. |
| 8 | Goods Receipts | Penerimaan barang dari purchase order, penerimaan parsial, validasi jumlah diterima, posting penerimaan, update stok SKU, update progress PO, dan pencatatan stock movement. |
| 9 | Stock Adjustments | Dokumen koreksi stok per SKU/kemasan, alasan adjustment, item adjustment, stok sebelum, stok sesudah, validasi stok tidak negatif, posting adjustment, dan pembatalan adjustment. |
| 10 | Stock Conversions | Dokumen konversi stok manual antar SKU dalam produk yang sama, misalnya 1 dus menjadi 40 pcs, dengan pencatatan stok masuk/keluar akibat konversi. |
| 11 | Point of Sale | Pencarian produk berdasarkan nama/SKU/barcode/QR code, input scan di layar kasir, keranjang kasir berbasis SKU/kemasan, ubah jumlah item, hapus item, subtotal otomatis, total pembayaran, uang dibayar, kembalian, simpan transaksi, dan struk sederhana. |
| 12 | Transaction History | Daftar transaksi, detail transaksi, filter tanggal, akses semua transaksi untuk admin, dan pembatasan transaksi milik sendiri untuk kasir. |
| 13 | Employee Management | CRUD karyawan, posisi, telepon, alamat, gaji pokok, status aktif/nonaktif, dan soft delete. |
| 14 | Payroll Slips | Pembuatan slip gaji per karyawan, periode gaji, bonus, potongan, perhitungan gaji bersih, detail slip, dan cetak/tampilan slip sederhana. |
| 15 | Reports | Laporan penjualan harian, penjualan bulanan, stok barang per SKU, purchase order, goods receipt, stock adjustment, dan payroll. |

## 3. Scope Teknis

Termasuk dalam penawaran:

- Struktur database POS sesuai kebutuhan.
- Backend API untuk seluruh modul utama.
- Validasi input, pesan sukses, dan pesan error.
- Role-based access untuk admin dan kasir.
- Soft delete untuk master data.
- Master satuan, produk induk, dan SKU/kemasan produk.
- Stok, harga, SKU, barcode, dan QR code dikelola di level SKU/kemasan.
- UI kasir mendukung pencarian item menggunakan nama produk, SKU, barcode, atau QR code.
- Input barcode/QR code dapat berasal dari scanner standar yang terbaca sebagai keyboard input.
- Dokumen transaksi dengan status draft, posted, completed, cancelled, atau status lain sesuai modul.
- Konversi stok manual antar SKU/kemasan dalam produk yang sama.
- Perhitungan subtotal, total, line total, quantity delta, net salary, dan agregasi laporan.
- UI web dasar yang bersih, modern, responsif, dan cukup untuk versi awal.
- Dokumentasi alur penggunaan API dan urutan pengujian.
- Data dummy untuk testing.
- Instruksi menjalankan aplikasi secara lokal.
- Training online

Tidak termasuk dalam versi awal:

- Multi cabang.
- Pengadaan perangkat barcode scanner atau QR scanner.
- Integrasi driver khusus, SDK vendor, troubleshooting perangkat fisik, atau konfigurasi hardware di lokasi klien.
- Scan QR menggunakan kamera webcam/mobile secara native.
- Integrasi payment gateway.
- Akuntansi lengkap.
- Hutang supplier atau pembayaran invoice pembelian.
- Absensi karyawan.
- Pajak otomatis.
- Hosting berbayar, domain, SSL production, dan biaya infrastruktur pihak ketiga.
- Migrasi data real dari sistem lama.
- Training onsite.

## 4. Estimasi Harga Penawaran

Harga yang direkomendasikan untuk proposal ini adalah:

**15.000.000**

Harga Rp15 juta berlaku untuk pengembangan versi awal sesuai scope di atas. Harga belum mencakup deployment produksi, hosting, pengadaan/perawatan hardware, integrasi driver khusus perangkat, dukungan berkala, dan perubahan scope besar.

## 5. Estimasi Waktu Pengerjaan

Estimasi waktu pengerjaan untuk sistem versi awal:

**3 sampai 5 minggu kerja**

Estimasi ini berlaku dengan asumsi kebutuhan fitur mengikuti scope yang sudah ada dan tidak ada perubahan scope besar selama pengerjaan.

## 6. Skema Pembayaran

Rekomendasi termin pembayaran:

| Termin | Persentase | Keterangan |
| --- | ---: | --- |
| DP | 15% | Dibayarkan saat project dimulai. |
| Pelunasan | 85% | Dibayarkan setelah serah terima project. |

## 7. Kriteria Serah Terima

Project dianggap selesai untuk scope di atas apabila:

- Admin dapat login dan mengelola user.
- Admin dapat mengelola kategori produk, satuan, produk induk, SKU/kemasan, supplier, karyawan, dan slip gaji.
- Admin dapat membuat purchase order, goods receipt, stock adjustment, stock conversion, dan melihat laporan.
- Kasir dapat melakukan transaksi penjualan berbasis SKU/kemasan melalui pencarian manual maupun input/scan barcode/QR code, serta melihat riwayat transaksi miliknya.
- Stok SKU/kemasan berubah sesuai flow final: penjualan completed, goods receipt posted, stock adjustment posted, dan stock conversion posted.
- Dashboard dan laporan utama dapat menampilkan data dari transaksi yang sudah dibuat.
- Aplikasi versi awal dapat dijalankan secara lokal untuk proses review dan serah terima.
- Dokumentasi API dan urutan pengujian tersedia.

## 8. Catatan

Perubahan fitur, integrasi eksternal, kebutuhan produksi, integrasi hardware khusus, atau permintaan custom di luar scope dapat dihitung sebagai pekerjaan tambahan.
