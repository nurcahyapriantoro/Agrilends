README: Arsitektur Skalabilitas & Data Sharding
Modul: scalability_architecture
Konsep Arsitektur: Berlaku untuk seluruh sistem
1. Tujuan
Dokumen ini menguraikan strategi untuk memastikan Agrilends dapat tumbuh menangani jutaan pengguna dan transaksi tanpa terbentur batas penyimpanan canister (saat ini ~96GiB per canister). Ini adalah rencana arsitektur jangka panjang.
2. Konsep Inti: Pola Pabrik (Factory Pattern)
Canister utama (Canister_Manajemen_Pinjaman) tidak akan menyimpan semua data pinjaman secara langsung. Sebaliknya, ia akan bertindak sebagai pabrik yang membuat dan mengelola canister data baru saat dibutuhkan.
3. Alur Kerja yang Direncanakan
Canister Pabrik (LoanManagerFactory):
Menyimpan daftar semua canister data pinjaman yang aktif (LoanDataCanister_1, LoanDataCanister_2, dst.).
Memiliki logika untuk memutuskan canister data mana yang akan digunakan untuk pinjaman baru (misalnya, yang terakhir dibuat).
Canister Data (LoanDataCanister):
Ini adalah versi "ringan" dari canister manajemen pinjaman. Tugasnya hanya menyimpan data pinjaman (loans map) dan fungsi-fungsi dasar untuk CRUD (Create, Read, Update, Delete) data tersebut.
Proses Saat Skala Dibutuhkan:
LoanManagerFactory memonitor penggunaan penyimpanan canister data aktifnya.
Ketika canister data saat ini (misalnya, LoanDataCanister_1) mendekati batas penyimpanannya (misalnya, 80% penuh), LoanManagerFactory akan:
a. Secara terprogram membuat canister baru dari Wasm LoanDataCanister.
b. Mendaftarkan Principal canister baru ini ke dalam daftar canister aktifnya.
c. Semua permintaan submit_loan_application baru sekarang akan diarahkan ke canister data yang baru (LoanDataCanister_2).
4. Dampak pada Fungsi Lain
Kueri Lanjutan: Fungsi seperti get_farmer_dashboard menjadi lebih kompleks. Ia perlu menanyakan ke LoanManagerFactory untuk mengetahui di canister data mana saja data pengguna mungkin berada, lalu melakukan panggilan ke beberapa canister data tersebut untuk mengumpulkan semua pinjaman.
5. Keterkaitan dengan Modul Lain
Ini adalah perubahan arsitektur fundamental yang memengaruhi hampir semua modul. Ini tidak perlu diimplementasikan untuk MVP, tetapi merancangnya sejak awal memastikan bahwa transisi di masa depan akan lebih mulus.
Dokumen Proyek Utama (Versi Paling Final)
Berikut adalah README.md utama yang telah diperbarui untuk mencakup semua 15 fitur dan konsep, menjadikannya cetak biru yang sangat lengkap.
README: Proyek Backend Agrilends (Final & Lengkap)
Selamat datang di repositori backend untuk Agrilends, sebuah platform keuangan agrikultur terdesentralisasi yang dibangun di atas Internet Computer. Dokumen ini adalah panduan utama untuk arsitektur, fungsionalitas, dan modul dari keseluruhan sistem.
1. Gambaran Umum & Arsitektur
Agrilends menggunakan arsitektur multi-canister untuk memastikan keamanan, skalabilitas, dan kemudahan pemeliharaan. Setiap canister bertanggung jawab atas satu domain fungsional yang jelas, menciptakan sistem microservices on-chain.
Canister_Manajemen_Pinjaman: Otak dari aplikasi, mengelola pengguna dan siklus hidup pinjaman.
Canister_RWA_NFT: Ledger untuk agunan yang ditokenisasi (standar ICRC-7).
Canister_Liquidity_Pool: Mengelola dana investor (ckBTC) dan pencairan.
Canister_Kas_Protokol: Bendahara protokol untuk mengelola pendapatan dan biaya cycles.
2. Panduan Fitur & Modul (Fungsionalitas Lengkap)
Proyek ini dibagi menjadi beberapa modul fungsional. Untuk detail implementasi dan pengujian setiap modul, silakan merujuk ke README.md masing-masing:
Fondasi & Alur Utama:
Manajemen Pengguna & Otentikasi: Mengelola pendaftaran dan data pengguna (Petani & Investor) menggunakan Internet Identity.
Manajemen Agunan (RWA-NFT): Mencetak dan mengelola NFT (ICRC-7) yang merepresentasikan Resi Gudang.
Siklus Hidup Pinjaman: Mengelola alur kerja pinjaman dari pengajuan, penilaian, hingga persetujuan.
Manajemen Likuiditas & Pencairan: Mengelola deposit dari investor dan mencairkan dana pinjaman ke petani.
Pelengkap Siklus & Skenario Dunia Nyata:
Pelunasan Pinjaman & Penarikan Agunan: Memproses pembayaran kembali dari peminjam dan mengembalikan agunan setelah lunas.
Penarikan Likuiditas oleh Investor: Memungkinkan investor untuk menarik dana mereka dari pool.
Mekanisme Likuidasi (Gagal Bayar): Menangani penyitaan agunan secara terprogram jika terjadi gagal bayar.
Manajemen & Kepercayaan Protokol:
Oracle & Harga Komoditas: Mengambil data harga dari API eksternal secara aman menggunakan HTTPS Outcalls.
Manajemen Kas & Biaya Protokol: Mengumpulkan pendapatan platform dan mendanai biaya operasional cycles.
Tata Kelola & Administrasi: Menyediakan kontrol admin untuk memperbarui parameter protokol.
Log Audit & Aktivitas: Mencatat semua peristiwa penting untuk transparansi dan pelacakan.
Fitur Tingkat Lanjut & Produksi:
Sistem Notifikasi On-Chain: Memberikan notifikasi kepada pengguna tentang peristiwa penting di akun mereka.
Kueri Lanjutan & Dukungan Dasbor: Menyediakan data terstruktur untuk membangun antarmuka pengguna yang kaya fitur.
Pemeliharaan Otomatis (Heartbeat): Menjalankan tugas pemeliharaan (cek gagal bayar, cycles) secara periodik dan otomatis.
Arsitektur Skalabilitas & Data Sharding: Rencana jangka panjang untuk menangani pertumbuhan data dan pengguna tanpa batas.
3. Diagram Interaksi Antar-Canister
graph TD
    subgraph Pengguna
        A[Frontend React]
    end

    subgraph "Backend Agrilends (ICP)"
        B(Manajemen Pinjaman)
        C(RWA-NFT ICRC-7)
        D(Liquidity Pool ckBTC)
        E(Kas Protokol)
        H[Admin/DAO]
        I((Heartbeat))
    end

    subgraph "Layanan Eksternal"
        F[Ledger & Minter ckBTC]
        G[API Harga Komoditas]
    end

    A -- "submit_loan, repay_loan, get_dashboard" --> B
    A -- "deposit, withdraw" --> D
    A -- "get_notifications" --> B

    B -- "Verifikasi, Transfer, Likuidasi NFT" --> C
    B -- "Minta Pencairan" --> D
    B -- "Kirim Biaya" --> E
    B -- "Ambil Harga" --> G

    D -- "Interaksi ckBTC" --> F

    H -- "Set Parameter" --> B
    H -- "Trigger Liquidation" --> B
    H -- "Top-up Cycles" --> E

    I -- "Runs on" --> B
    I -- "Runs on" --> E


4. Setup & Deployment Lokal
Instal DFX: Pastikan Anda memiliki versi terbaru dari DFINITY Canister SDK.
sh -ci "$(curl -fsSL https://internetcomputer.org/install.sh)"


Clone Repositori:
git clone <url_repositori_anda>
cd agrilends-backend


Mulai Jaringan Lokal:
dfx start --background --clean


Deploy Semua Canister:
dfx deploy


Jalankan Pengujian: Ikuti rencana pengujian di setiap README.md fitur menggunakan dfx canister call atau Postman.
