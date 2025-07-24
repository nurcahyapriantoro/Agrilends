README: Fitur Manajemen Kas & Biaya Protokol (Treasury)
Modul: treasury_management
Canister Terkait: Canister_Kas_Protokol
1. Tujuan Fitur
Fitur ini berfungsi sebagai bendahara (treasury) untuk protokol Agrilends. Tujuannya adalah untuk mengumpulkan pendapatan yang dihasilkan dari operasi platform dan menggunakannya untuk memastikan keberlanjutan jangka panjang, terutama untuk mendanai biaya operasional (cycles) dari semua canister lain.
2. Canister Terkait
Logika ini akan dienkapsulasi di dalam Canister_Kas_Protokol. Canister ini akan menerima dana dari Canister_Manajemen_Pinjaman dan memiliki wewenang untuk mengisi ulang cycles canister lain.
3. Struktur Data (Types)
// src/treasury/types.mo

// Menyimpan saldo total kas dalam ckBTC
var treasury_balance_ckbtc: Nat = 0;

// Daftar canister operasional yang perlu didanai
var operational_canisters: StableBTreeMap<Text, Principal> = StableBTreeMap.init(Text.compare);

// Log transaksi pendapatan
public type RevenueEntry = {
    source_loan_id: Nat;
    amount: Nat;
    revenue_type: {#AdminFee; #InterestShare};
    timestamp: Time.Time;
};
var revenue_log: Vec<RevenueEntry> = [];


4. Fungsi Publik (Public Functions)
a. collect_fees(source_loan_id: Nat, amount: Nat, revenue_type: RevenueType)
Tipe: update
Deskripsi: Menerima dana biaya dari canister lain.
Keamanan: Fungsi ini harus dilindungi agar hanya bisa dipanggil oleh Canister_Manajemen_Pinjaman.
Input:
source_loan_id: Nat: ID pinjaman asal pendapatan.
amount: Nat: Jumlah ckBTC yang ditransfer.
revenue_type: RevenueType: Jenis pendapatan (#AdminFee atau #InterestShare).
Output: async Result<Text, Text>
Logika:
Kontrol Akses: Verifikasi bahwa caller adalah Principal dari Canister_Manajemen_Pinjaman.
Panggilan Antar-Canister: Fungsi ini dipicu setelah Canister_Manajemen_Pinjaman menerima pembayaran bunga dan mentransfer sebagian ke sini menggunakan icrc1_transfer.
Perbarui treasury_balance_ckbtc.
Catat transaksi ke dalam revenue_log.
Kembalikan Result.ok("Fees collected successfully.").
b. top_up_canister_cycles(canister_name: Text)
Tipe: update
Deskripsi: Mengisi ulang cycles untuk canister operasional.
Keamanan: Hanya dapat dipanggil oleh admin atau melalui mekanisme tata kelola (DAO) di masa depan.
Logika:
Konversi Aset: Konversi sebagian kecil dari treasury_balance_ckbtc menjadi cycles melalui Cycles Minting Canister.
Panggilan Manajemen Canister: Gunakan fungsi deposit_cycles dari IC Management Canister untuk mengirim cycles ke Principal canister yang dituju.
Kembalikan status sukses.
c. get_treasury_stats()
Tipe: query
Deskripsi: Mengambil statistik kas protokol.
Output: async record { balance_ckbtc: Nat; total_canisters_managed: Nat }
Logika: Kembalikan data dari stable variables.
5. Keterkaitan dengan Modul Lain
Siklus Hidup Pinjaman: Saat fungsi repay_loan dieksekusi di Canister_Manajemen_Pinjaman, canister tersebut akan menghitung bagian bunga untuk protokol dan memanggil collect_fees di canister ini.
Semua Canister: Canister ini secara proaktif memonitor dan mendanai cycles untuk Canister_Manajemen_Pinjaman, Canister_RWA_NFT, dan Canister_Liquidity_Pool.
