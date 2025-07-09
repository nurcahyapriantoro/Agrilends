README: Fitur Kueri Lanjutan & Dukungan Dasbor
Modul: dashboard_support
Canister Terkait: Canister_Manajemen_Pinjaman, Canister_Liquidity_Pool
1. Tujuan Fitur
Fitur ini menyediakan endpoint kueri yang dioptimalkan untuk frontend, memungkinkan pembuatan dasbor pengguna yang kaya fitur. Daripada frontend harus memanggil banyak fungsi dan menggabungkan data, canister menyediakan data yang sudah terstruktur.
2. Konsep Implementasi
Membuat fungsi query gabungan yang mengumpulkan data dari berbagai struktur data dan menyajikannya dalam format yang mudah digunakan oleh UI.
3. Fungsi Publik (Public Functions)
a. get_farmer_dashboard()
Tipe: query
Canister: Canister_Manajemen_Pinjaman
Deskripsi: Mengambil semua data relevan untuk dasbor petani.
Output: async record { user_details: User; active_loans: Vec<Loan>; historical_loans: Vec<Loan>; owned_nfts: Vec<NFT_Summary> }
Logika:
Dapatkan caller.
Ambil User detail.
Iterasi melalui loans map dan filter pinjaman yang dimiliki oleh caller, pisahkan antara yang aktif dan yang sudah selesai.
Panggilan Antar-Canister: Panggil Canister_RWA_NFT untuk mendapatkan daftar NFT yang dimiliki oleh caller.
Gabungkan semua data ini ke dalam satu objek dan kembalikan.
b. get_investor_dashboard()
Tipe: query
Canister: Canister_Liquidity_Pool
Deskripsi: Mengambil semua data relevan untuk dasbor investor.
Output: async record { user_details: User; current_balance: Nat; total_earnings: Nat; pool_stats: PoolStats }
Logika:
Dapatkan caller.
Panggilan Antar-Canister: Panggil get_user di Canister_Manajemen_Pinjaman.
Ambil saldo investor dari investor_balances.
Ambil statistik umum dari pool.
Hitung total pendapatan (memerlukan penyimpanan data historis).
Gabungkan dan kembalikan.
4. Keterkaitan dengan Modul Lain
Fitur ini tidak mengubah logika bisnis, tetapi secara signifikan meningkatkan kemampuan observasi dan pengalaman pengguna dengan menyediakan data siap pakai untuk antarmuka React.
