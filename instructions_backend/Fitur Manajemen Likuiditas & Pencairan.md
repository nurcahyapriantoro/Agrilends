README: Fitur Manajemen Likuiditas & Pencairan
Modul: liquidity_management
Canister Terkait: Canister_Liquidity_Pool
1. Tujuan Fitur
Fitur ini bertanggung jawab untuk mengelola dana (ckBTC) yang disediakan oleh investor dan mengeksekusi pencairan dana untuk pinjaman yang telah disetujui. Ini adalah komponen paling kritis dari segi keamanan karena secara langsung mengelola aset pengguna.
Catatan: Canister ini adalah kandidat utama untuk diimplementasikan menggunakan Rust di masa depan untuk jaminan keamanan memori yang lebih tinggi. Untuk MVP, dapat dimulai dengan Motoko.
2. Canister Terkait
Logika utama berada di Canister_Liquidity_Pool. Canister ini akan berinteraksi dengan Ledger ckBTC dan Minter ckBTC.
3. Struktur Data (Types)
// src/liquidity_pool/types.mo

// Melacak total likuiditas yang tersedia di pool
var total_liquidity: Nat = 0;

// Melacak saldo deposit untuk setiap investor
var investor_balances: StableBTreeMap<Principal, Nat> = StableBTreeMap.init(Principal.compare);

// Menyimpan catatan transaksi untuk idempotensi
type TxId = Nat;
var processed_transactions: StableBTreeMap<TxId, Bool> = StableBTreeMap.init(Nat.compare);


4. Fungsi Publik (Public Functions)
a. deposit_liquidity(amount: Nat, tx_id: Nat)
Tipe: update
Deskripsi: Memungkinkan investor untuk mendepositkan ckBTC ke dalam pool.
Input:
amount: Nat: Jumlah ckBTC dalam satoshi.
tx_id: Nat: ID transaksi unik yang dibuat oleh klien untuk idempotensi.
Output: async Result<Text, Text>
Logika & Keamanan:
Idempotensi: Periksa apakah tx_id sudah ada di processed_transactions. Jika ya, kembalikan Result.ok("Transaction already processed.").
Dapatkan caller (Investor).
Panggilan Antar-Canister (Transfer ckBTC): Panggil fungsi icrc1_transfer_from di ledger ckBTC. Ini memerlukan approve dari investor sebelumnya.
Jika transfer berhasil:
a. Tambahkan amount ke total_liquidity.
b. Perbarui saldo di investor_balances untuk caller.
c. Tandai tx_id sebagai telah diproses di processed_transactions.
Kembalikan Result.ok("Deposit successful").
b. disburse_loan(borrower_btc_address: Text, amount: Nat)
Tipe: update
Deskripsi: Mencairkan dana ke alamat Bitcoin petani. Fungsi ini harus sangat dilindungi.
Input:
borrower_btc_address: Text: Alamat Bitcoin tujuan.
amount: Nat: Jumlah ckBTC dalam satoshi.
Output: async Result<Text, Text>
Logika & Keamanan:
KONTROL AKSES KRITIS: Verifikasi bahwa caller adalah Principal dari Canister_Manajemen_Pinjaman. Jika tidak, trap("Unauthorized: Only the loan manager can disburse funds.").
Periksa apakah total_liquidity cukup untuk menutupi amount. Jika tidak, kembalikan Result.err("Insufficient liquidity in the pool.").
Panggilan Antar-Canister (Integrasi ckBTC):
a. Panggil icrc2_approve di ledger ckBTC untuk memberikan izin kepada Minter ckBTC untuk menarik amount dari canister ini.
b. Panggil retrieve_btc_with_approval pada Minter ckBTC dengan amount dan borrower_btc_address.
Jika panggilan berhasil, kurangi amount dari total_liquidity.
Kembalikan Result.ok("Disbursement initiated successfully.").
c. get_pool_stats()
Tipe: query
Deskripsi: Mengambil statistik dasar dari liquidity pool.
Output: async record { total_liquidity: Nat }
Logika: Kembalikan nilai total_liquidity.
5. Rencana Pengujian (Testing Plan)
Deposit Berhasil:
Prasyarat: Investor harus memanggil icrc2_approve di ledger ckBTC untuk canister ini.
Panggil deposit_liquidity dengan jumlah dan tx_id baru.
Ekspektasi: Respon sukses. total_liquidity bertambah.
Mencegah Deposit Ganda (Idempotensi):
Panggil deposit_liquidity dua kali dengan tx_id yang sama.
Ekspektasi: Panggilan pertama berhasil, panggilan kedua juga mengembalikan sukses tetapi tanpa mengubah saldo (karena sudah diproses).
Gagal Pencairan oleh Pengguna Asing:
Panggil disburse_loan menggunakan caller yang bukan Canister_Manajemen_Pinjaman.
Ekspektasi: Panggilan gagal dengan pesan Canister trapped: Unauthorized....
Gagal Pencairan (Likuiditas Kurang):
Coba cairkan jumlah yang lebih besar dari total_liquidity.
Ekspektasi: Respon error "Insufficient liquidity...".
Pencairan Berhasil (Simulasi):
Tambahkan Principal admin/developer sebagai caller yang diizinkan sementara.
Pastikan likuiditas cukup.
Panggil disburse_loan.
Ekspektasi: Respon sukses. total_liquidity berkurang.
