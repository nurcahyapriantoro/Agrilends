README: Proyek Backend Agrilends (Final)Selamat datang di repositori backend untuk Agrilends, sebuah platform keuangan agrikultur terdesentralisasi yang dibangun di atas Internet Computer. Dokumen ini adalah panduan utama untuk arsitektur, fungsionalitas, dan modul dari keseluruhan sistem.1. Gambaran Umum & ArsitekturAgrilends menggunakan arsitektur multi-canister untuk memastikan keamanan, skalabilitas, dan kemudahan pemeliharaan. Setiap canister bertanggung jawab atas satu domain fungsional yang jelas, menciptakan sistem microservices on-chain.Canister_Manajemen_Pinjaman: Otak dari aplikasi, mengelola pengguna dan siklus hidup pinjaman.Canister_RWA_NFT: Ledger untuk agunan yang ditokenisasi (standar ICRC-7).Canister_Liquidity_Pool: Mengelola dana investor (ckBTC) dan pencairan.Canister_Kas_Protokol: Bendahara protokol untuk mengelola pendapatan dan biaya cycles.2. Panduan Fitur & Modul (Fungsionalitas Lengkap)Proyek ini dibagi menjadi beberapa modul fungsional. Untuk detail implementasi dan pengujian setiap modul, silakan merujuk ke README.md masing-masing:Fondasi & Alur Utama:Manajemen Pengguna & Otentikasi: Mengelola pendaftaran dan data pengguna (Petani & Investor) menggunakan Internet Identity.Manajemen Agunan (RWA-NFT): Mencetak dan mengelola NFT (ICRC-7) yang merepresentasikan Resi Gudang.Siklus Hidup Pinjaman: Mengelola alur kerja pinjaman dari pengajuan, penilaian, hingga persetujuan.Manajemen Likuiditas & Pencairan: Mengelola deposit dari investor dan mencairkan dana pinjaman ke petani.Pelengkap Siklus & Skenario Dunia Nyata:Pelunasan Pinjaman & Penarikan Agunan: Memproses pembayaran kembali dari peminjam dan mengembalikan agunan setelah lunas.Penarikan Likuiditas oleh Investor: Memungkinkan investor untuk menarik dana mereka dari pool.Mekanisme Likuidasi (Gagal Bayar): Menangani penyitaan agunan secara terprogram jika terjadi gagal bayar.Manajemen & Kepercayaan Protokol:Oracle & Harga Komoditas: Mengambil data harga dari API eksternal secara aman menggunakan HTTPS Outcalls.Manajemen Kas & Biaya Protokol: Mengumpulkan pendapatan platform dan mendanai biaya operasional cycles.Tata Kelola & Administrasi: Menyediakan kontrol admin untuk memperbarui parameter protokol.Log Audit & Aktivitas: Mencatat semua peristiwa penting untuk transparansi dan pelacakan.3. Diagram Interaksi Antar-Canistergraph TD
    subgraph Pengguna
        A[Frontend React]
    end

    subgraph "Backend Agrilends (ICP)"
        B(Manajemen Pinjaman)
        C(RWA-NFT ICRC-7)
        D(Liquidity Pool ckBTC)
        E(Kas Protokol)
        H[Admin/DAO]
    end

    subgraph "Layanan Eksternal"
        F[Ledger & Minter ckBTC]
        G[API Harga Komoditas]
    end

    A -- "submit_loan, repay_loan" --> B
    A -- "deposit, withdraw" --> D

    B -- "Verifikasi, Transfer, Likuidasi NFT" --> C
    B -- "Minta Pencairan" --> D
    B -- "Kirim Biaya" --> E
    B -- "Ambil Harga" --> G

    D -- "Interaksi ckBTC" --> F

    H -- "Set Parameter" --> B
    H -- "Trigger Liquidation" --> B
    H -- "Top-up Cycles" --> E
4. Setup & Deployment LokalInstal DFX: Pastikan Anda memiliki versi terbaru dari DFINITY Canister SDK.sh -ci "$(curl -fsSL https://internetcomputer.org/install.sh)"
Clone Repositori:git clone <url_repositori_anda>
cd agrilends-backend
Mulai Jaringan Lokal:dfx start --background --clean
Deploy Semua Canister:dfx deploy
Jalankan Pengujian: Ikuti rencana pengujian di setiap README.md fitur menggunakan dfx canister call atau Postman.