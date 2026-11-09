# Panduan Pengguna Point Flow

Panduan ini menjelaskan cara memakai aplikasi Point Flow dari awal sampai akhir mengikuti menu yang ada di website. Bahasanya dibuat semanusiawi mungkin: tidak ada istilah API, JSON, atau endpoint. Untuk dokumentasi teknis, lihat [api-flow.md](/Users/ridwannhidayat/Workspace/labs/point_flow/api-flow.md).

Point Flow punya dua wajah:

- **Aplikasi kasir** (dipakai karyawan di toko): login dengan email, ada menu di sisi kiri — Dashboard, Kasir, Master Data, Pembelian, Inventori, Payroll, Keuangan, Laporan, dan Pengaturan.
- **Toko online / storefront** (dibuka pelanggan lewat browser): katalog produk, keranjang, checkout, dan akun anggota — tanpa perlu login karyawan.

Alur besarnya: siapkan data toko dulu (barang, supplier, karyawan), beli barang dari supplier sampai stok masuk, lalu jualan — di kasir maupun online. Uang kas dicatat lewat sesi laci, gaji lewat payroll, biaya lain lewat keuangan, dan sisa hasilnya bisa dibagi lewat SHU di akhir periode.

## 1. Masuk dan Dashboard

### Login karyawan

Buka aplikasi kasir dan masuk dengan email dan password akun karyawan. Setelah masuk, nama dan email tampil di pojok kiri bawah. Kalau sesi habis, cukup login ulang — data tidak hilang.

Ada dua peran bawaan:

- **Admin** — bisa membuka semua menu.
- **Kasir** — aksesnya terbatas (biasanya hanya kasir dan dashboard). Halaman yang dibuka tanpa izin akan menampilkan pesan penolakan, dan datanya pun dibatasi (kasir hanya melihat transaksinya sendiri).

Kalau baru pertama kali memakai sistem dan belum ada akun sama sekali, akun admin pertama dibuat lewat proses bootstrap (dibantu tim teknis).

### Dashboard

Halaman pertama setelah login. Isinya ringkasan kondisi toko hari ini:

- Omzet dan jumlah transaksi hari ini.
- Produk yang stoknya menipis.
- Purchase order yang masih menunggu barang.
- Grafik tren penjualan, produk terlaris, jam ramai, penjualan per kategori, kinerja kasir, dan alur pembelian.

Angka di dashboard selalu mengikuti data asli — setiap penjualan, retur, atau penerimaan barang langsung mengubah angkanya.

## 2. Menyiapkan Data Toko (Master Data)

Menu **Master Data** adalah fondasi. Sekali diisi dengan benar, semua transaksi berikutnya jadi mudah. Urutan yang disarankan: Kategori Produk → Satuan → Produk → SKU Produk → Supplier → Register Kas → Karyawan → Anggota.

### Kategori Produk

Kelompok barang, misalnya Minuman, Makanan Ringan, atau Kebutuhan Dapur. Fungsinya supaya katalog rapi dan laporan bisa dilihat per kategori. Kategori yang sudah dipakai produk sebaiknya tidak dihapus — nonaktifkan saja kalau sudah tidak dipakai.

### Satuan

Satuan barang, misalnya pcs, dus, atau kg. Satu trik penting: buat satuan eceran (pcs) dan satuan kemasan (dus) sekaligus, karena nanti berguna saat memecah kemasan besar menjadi eceran.

### Produk dan SKU Produk

Ini dua hal berbeda yang sering membingungkan:

- **Produk** adalah barangnya — misalnya "Air Mineral". Di sini cukup diisi nama dan kategorinya.
- **SKU Produk** adalah kemasannya — misalnya "Air Mineral botol (pcs)" dan "Air Mineral dus isi 40". Setiap SKU punya kode unik, barcode, harga beli, harga jual, stok, dan stok minimum sendiri.

Harga dan stok selalu menempel di SKU, bukan di produk. Jadi kalau harga botol naik tapi harga dus tetap, yang diubah hanya SKU botol. Foto produk juga dipasang di sini dan otomatis tampil di toko online.

### Supplier

Data pemasok barang: nama, kontak, telepon, dan alamat. Supplier wajib ada sebelum membuat purchase order.

### Register Kas

Daftar laci/meja kasir fisik, misalnya "Kasir Depan". Setiap transaksi penjualan harus terhubung ke salah satu register yang sesinya sedang dibuka (lihat bagian 5).

### Karyawan

Data karyawan untuk keperluan gaji — terpisah dari akun login. Karyawan di sini belum tentu punya akun aplikasi; akun login diatur di menu User (bagian 9).

### Anggota (Member)

Daftar pelanggan terdaftar untuk toko online. Anggota dibuat oleh admin/kasir (nama, nomor HP, password), lalu anggota bisa login sendiri di toko online dengan nomor HP-nya. Keuntungan jadi anggota: data nama dan alamat tersimpan, dan transaksinya tercatat atas nama mereka — berguna untuk pembagian SHU.

Menghapus anggota yang sudah pernah belanja tidak menghilangkan datanya — statusnya hanya dinonaktifkan supaya riwayat transaksi tetap utuh.

## 3. Membeli Barang dari Supplier (Pembelian)

Menu **Pembelian** punya dua langkah: pesan dulu, terima kemudian.

### Purchase Order (pesan barang)

Purchase order (PO) adalah surat pesanan ke supplier. Isinya: supplier mana, barang apa saja, jumlah, dan harga per unit. Nomor PO dibuat otomatis.

- Saat dibuat, statusnya **draft** — masih bisa diubah, stok belum berubah.
- Tekan **Order** untuk mengirim pesanan. Status menjadi **ordered**.
- Kalau salah input dan barang belum diterima, PO bisa **dibatalkan** dengan alasan.

### Penerimaan Barang (terima barang)

Saat barang datang, catat di **Penerimaan Barang** dengan memilih PO-nya. Isi jumlah yang benar-benar diterima per barang — boleh sebagian dulu (misalnya pesan 20, yang datang 10).

- Saat dibuat, statusnya **draft** — stok belum berubah.
- Tekan **Posting** setelah barang dihitung. Saat itulah stok bertambah, PO otomatis terupdate (sebagian diterima atau sudah lunas semua), dan pergerakan stok tercatat.
- Penerimaan yang salah bisa **dibatalkan** dengan alasan sebelum dipakai transaksi lain.

## 4. Mengoreksi Stok (Inventori)

Menu **Inventori** untuk merapikan stok di luar jual-beli: hasil opname, barang rusak/hilang, atau pecah kemasan.

### Penyesuaian Stok

Dipakai kalau stok di sistem beda dengan fisik. Caranya: pilih SKU, isi jumlah yang seharusnya (`stok setelah`). Sistem menyimpan angka sebelum dan sesudahnya, lalu tekan **Posting** — stok langsung berubah ke angka tersebut. Cocok untuk hasil stock opname atau mencatat barang rusak.

### Konversi Stok

Dipakai untuk memecah/menggabung kemasan. Contoh: 1 dus air mineral (isi 40) dipecah menjadi 40 botol eceran. Stok dus berkurang 1, stok botol bertambah 40. Seperti biasa: buat dokumen, lalu **Posting**.

### Laporan Stok dan Mutasi Stok

- **Laporan Stok** menampilkan posisi stok semua SKU: jumlah, harga, nilai, status menipis, dan jumlah yang masih dipesan ke supplier.
- **Mutasi Stok** adalah buku harian stok: setiap barang masuk (penerimaan), keluar (penjualan), koreksi, konversi, dan retur tercatat di sini lengkap dengan tanggalnya. Kalau ada selisih stok yang misterius, mulailah mencari dari sini.

## 5. Jualan di Kasir

Menu **Kasir** adalah tempat kerja utama setiap hari. Urutannya: buka sesi → jualan → (retur kalau ada) → catat uang kas manual → tutup sesi.

### Sesi Register (buka dan tutup laci)

Sebelum jualan, **buka sesi** di register yang dipakai: pilih register, isi uang modal awal di laci, dan simpan. Satu register hanya boleh punya satu sesi terbuka dalam satu waktu.

Saat toko tutup, **tutup sesi**: hitung uang fisik di laci dan masukkan angkanya. Sistem otomatis menghitung uang yang seharusnya ada (modal + penjualan − retur + uang masuk − uang keluar) dan menunjukkan selisihnya. Selisih nol berarti beres; selisih plus/minus langsung ketahuan di sini.

### Transaksi Baru

Halaman kasir untuk melayani pembeli:

1. Pilih barang (SKU) dan jumlah — harga otomatis terisi, stok otomatis berkurang saat transaksi disimpan.
2. Masukkan jumlah uang yang dibayar — kembalian dihitung otomatis.
3. Simpan. Struk bernomor otomatis tercetak/tercatat.

### Riwayat Penjualan

Daftar semua transaksi: yang selesai (**completed**) maupun yang dibatalkan (**cancelled**). Transaksi yang salah input bisa dibatalkan dengan alasan. Catatan: pembatalan mengubah status saja dan tidak otomatis mengembalikan stok — kalau stok perlu dikoreksi, pakai Penyesuaian Stok.

### Retur Penjualan

Kalau pembeli mengembalikan barang dari transaksi yang sudah selesai:

1. Cari transaksi aslinya, lihat daftar barang yang masih bisa diretur (sistem menghitung sisa yang belum pernah diretur, jadi tidak bisa retur barang yang sama dua kali).
2. Pilih barang dan jumlahnya, tentukan apakah barangnya **masuk stok lagi** (masih bagus) atau tidak (rusak/cacat).
3. Simpan dengan alasan. Uang kembali tercatat, dan kalau barang masuk stok lagi, stok bertambah otomatis.
4. Retur yang salah bisa dibatalkan selama sesi laci yang dipakai masih terbuka; kalau barangnya sempat masuk stok, pembatalan akan mengurangi stok kembali.

### Pergerakan Kas

Mencatat uang laci di luar jualan: tambahan modal (**kas masuk**), setoran ke brankas (**cash drop**), pengeluaran (**kas keluar**), dan belanja kecil operasional (**petty cash**). Semua tercatat per sesi dan ikut perhitungan saat tutup sesi. Pergerakan yang salah bisa dibatalkan selama sesinya masih terbuka.

## 6. Jualan Online (Storefront dan Pesanan Online)

Toko online punya dua sisi: halaman yang dilihat pelanggan, dan halaman **Pesanan Online** di aplikasi kasir untuk memprosesnya.

### Sisi pelanggan: katalog, keranjang, checkout

Pelanggan membuka halaman toko (`/shop`) tanpa perlu login untuk melihat-lihat:

- **Katalog** menampilkan produk yang aktif dan stoknya tersedia, lengkap dengan foto, harga, dan kategori. Ada juga info toko (jam buka, rekening, e-wallet, ongkir, WhatsApp).
- Pelanggan memasukkan barang ke **keranjang**, lalu **checkout**. Ada dua cara:
  - **Tamu** — isi nama dan nomor HP di form. Cocok untuk ambil di tempat dan bayar di toko.
  - **Anggota** — login dulu dengan nomor HP dan password, data nama/alamat terisi otomatis, dan pesanan tercatat atas nama mereka. Anggota tercatat otomatis dari akunnya — tidak perlu (dan tidak bisa) mengaku-aku jadi anggota lain.
- Pilih **ambil di tempat** atau **diantar**, lalu **bayar di toko** atau **transfer**. Pesanan transfer menampilkan info pembayaran (rekening/e-wallet toko + total tagihan).
- Setelah berhasil, muncul halaman sukses berisi nomor pesanan.

Anggota bisa membuka halaman **profil** untuk melihat datanya dan **ganti password** sendiri.

### Sisi kasir: memproses pesanan

Setiap ada pesanan baru, angka badge muncul di menu **Pesanan Online** — jadi kasir langsung tahu tanpa harus mengecek manual. Alurnya:

1. **Baru** — pesanan masuk. Tekan **Pack** saat barang mulai disiapkan (status menjadi **disiapkan**).
2. Untuk **ambil di tempat**: tekan **Ready** saat barang siap diambil, lalu **Complete** saat sudah diserahkan.
3. Untuk **diantar**: tekan **Deliver** saat kurir berangkat (hanya bisa jika pesanan sudah dibayar, atau metodenya bayar di toko), lalu **Complete** saat diterima.
4. Pesanan **transfer** yang pelunasannya dikonfirmasi manual ditandai lewat **Mark Paid**.
5. Pesanan yang tidak bisa dipenuhi **dibatalkan** dengan alasan (hanya selagi masih baru/disiapkan).

### Pengaturan Toko

Di menu **Pengaturan Toko** (bagian 9), atur nama toko, jam operasional, rekening, e-wallet, ongkir, nomor WhatsApp, dan gambar QRIS — semuanya langsung tampil di halaman toko online.

## 7. Gaji (Payroll)

### Slip Payroll

Buat slip gaji per karyawan per periode: pilih karyawan (nama, posisi, dan gaji pokok terisi otomatis), isi bonus dan potongan bila ada — gaji bersih dihitung otomatis. Nomor slip dibuat otomatis.

### Laporan Payroll

Rekap gaji per periode: berapa slip, total gaji pokok, bonus, potongan, dan gaji bersih. Berguna untuk melihat total beban gaji, dan angka ini juga dipakai di laporan laba rugi.

## 8. Keuangan dan SHU

Menu **Keuangan** mencatat uang di luar kasir: biaya operasional, pemasukan lain, dan sejenisnya.

### Transaksi Keuangan

Setiap transaksi punya jenis (**pemasukan** atau **pengeluaran**), kategori (misalnya "Operasional Toko"), jumlah, dan penanda **tunai atau non-tunai**. Transaksi non-tunai (misalnya transfer bank) tidak ikut arus kas tunai. Kategori yang sudah dipakai transaksi tidak bisa dihapus. Transaksi yang salah bisa dibatalkan dengan alasan — transaksi yang dibatalkan tidak ikut laporan apa pun.

### Laba Rugi, Arus Kas, Posisi Keuangan

Tiga laporan ini terisi otomatis dari data yang sudah ada — tidak perlu input manual:

- **Laba Rugi** — pendapatan bersih (penjualan dikurangi retur), dikurangi harga pokok barang, dikurangi beban gaji dan beban manual. Hasilnya laba (atau rugi) bersih periode tersebut.
- **Arus Kas** — dari mana uang tunai masuk (penjualan tunai, pemasukan manual tunai) dan ke mana keluarnya (laci, pengeluaran manual tunai), plus selisihnya.
- **Posisi Keuangan** — per tanggal tertentu: berapa kas, berapa nilai persediaan, dan totalnya, plus akumulasi laba sampai tanggal itu.

Ketiganya bisa difilter rentang tanggal, cocok dibuka tiap akhir bulan sebelum mengambil keputusan.

### SHU (Sisa Hasil Usaha)

Di akhir periode (misalnya tutup tahun buku), laba bisa dibagi ke anggota lewat menu **SHU**:

1. **Buat periode** — isi nama periode, tanggal mulai-selesai, dan persentase pembagian (anggota, cadangan, pengurus, pengawas, sosial, pendidikan).
2. **Hitung (Calculate)** — sistem mengambil laba bersih periode itu sebagai dasar perhitungan, lalu membuat rincian per anggota. Porsi tiap anggota sebanding dengan total belanja mereka selama periode tersebut, jadi yang sering belanja dapat lebih besar. Kalau ingin memakai angka dasar sendiri, isi pool manual sebelum menghitung.
3. Periksa hasilnya di **detail periode** dan **daftar distribusi**. Kalau rentangnya salah, **batalkan** dan buat ulang.
4. Kalau sudah benar, **Finalize** untuk mengunci — periode yang dikunci tidak bisa diubah lagi.
5. Hasilnya bisa diunduh sebagai **file CSV** untuk dibagikan atau dicetak.

## 9. Pengaturan

### Pengaturan Toko

Identitas toko yang tampil di toko online (lihat bagian 6): nama, jam operasional, rekening, e-wallet, ongkir, WhatsApp, dan gambar QRIS.

### User dan Role

- **User** — akun login karyawan (nama, email, password, peran). Akun yang sudah tidak dipakai sebaiknya dinonaktifkan, bukan dihapus.
- **Role** — peran dan hak aksesnya, misalnya kasir boleh membuka kasir saja sedangkan admin boleh semua. Role bawaan sistem (admin, kasir) tidak bisa dihapus. Mengganti hak akses sebuah role langsung berlaku untuk semua user dengan role tersebut.

### Audit Log

Catatan otomatis atas kejadian penting: siapa login, siapa membuat/membatalkan transaksi, siapa membuka/menutup sesi, dan sebagainya. Isinya hanya bisa dibaca — tidak bisa diubah atau dihapus. Kalau ada kejadian janggal (misalnya transaksi batal misterius), cari tahu siapa pelakunya di sini.

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
- **Posted** — sudah resmi dicatat; stok sudah berubah. Dokumen yang sudah diposting tidak bisa diubah lagi.
- **Completed / Posted / Received** — transaksi selesai dan berpengaruh penuh.
- **Cancelled** — dibatalkan dengan alasan. Data tetap tersimpan sebagai riwayat, tapi tidak ikut perhitungan (stok, kas, laporan).
- **New → Packing → Ready/Delivering → Completed** — perjalanan pesanan online dari masuk sampai selesai.
- **Open / Closed** — sesi laci sedang berjalan atau sudah ditutup.
- **Paid / Unpaid** — pesanan online sudah dibayar atau belum.

Prinsip umumnya: selama masih draft/baru, hampir semua bisa diperbaiki. Setelah diposting atau dikunci (finalize), yang bisa dilakukan hanya membatalkan dengan alasan — dan pembatalan selalu meninggalkan jejak di audit log.
