README: Fitur Siklus Hidup Pinjaman (Loan Lifecycle)
Modul: loan_lifecycle
Canister Terkait: Canister_Manajemen_Pinjaman
1. Tujuan Fitur
Fitur ini mengelola seluruh alur kerja pinjaman, mulai dari pengajuan oleh petani, validasi agunan, persetujuan, hingga pelunasan atau gagal bayar. Ini adalah otak dari logika bisnis Agrilends.
2. Canister Terkait
Semua logika utama berada di Canister_Manajemen_Pinjaman. Canister ini akan berinteraksi secara intensif dengan Canister_RWA_NFT dan Canister_Liquidity_Pool.
3. Struktur Data (Types)
// src/loan_manager/types.mo

public type LoanStatus = {
    #PendingApplication; // Menunggu data agunan dan valuasi
    #PendingApproval;   // Menunggu persetujuan dari peminjam
    #Approved;          // Disetujui, menunggu pencairan dana
    #Active;            // Dana sudah cair, pinjaman aktif
    #Repaid;            // Lunas
    #Defaulted;         // Gagal bayar
};

public type Loan = {
    id: Nat;
    borrower: Principal;
    nft_id: Nat;
    collateral_value_btc: Nat; // Nilai agunan dalam satoshi ckBTC
    amount_requested: Nat;     // Jumlah yang diminta dalam satoshi
    amount_approved: Nat;      // Jumlah yang disetujui (mis. 60% dari nilai agunan)
    apr: Nat;                  // Suku bunga per tahun, mis. 10
    status: LoanStatus;
    createdAt: Time.Time;
};

var loans: StableBTreeMap<Nat, Loan> = StableBTreeMap.init(Nat.compare);
var next_loan_id: Nat = 0;


4. Fungsi Publik (Public Functions)
a. submit_loan_application(nft_id: Nat, amount_requested: Nat)
Tipe: update
Deskripsi: Memulai proses pengajuan pinjaman oleh petani.
Input:
nft_id: Nat: ID dari RWA-NFT yang akan dijadikan agunan.
amount_requested: Nat: Jumlah pinjaman yang diinginkan dalam satoshi ckBTC.
Output: async Result<Loan, Text>
Logika:
Dapatkan caller (Petani). Pastikan pengguna terdaftar.
Panggilan Antar-Canister (Verifikasi): Panggil Canister_RWA_NFT untuk memverifikasi bahwa caller adalah pemilik sah dari nft_id.
Panggilan Antar-Canister (HTTPS Outcall): Panggil API harga komoditas untuk mendapatkan harga pasar terkini. Wajib menggunakan fungsi transform.
Ambil nilai valuasi awal dari metadata NFT (misalnya, rwa:valuation_idr).
Kombinasikan data harga pasar dan valuasi untuk menentukan collateral_value_btc.
Hitung amount_approved (misalnya, 60% dari collateral_value_btc).
Buat objek Loan baru dengan status #PendingApproval.
Simpan pinjaman ke loans map.
Kembalikan Result.ok(newLoan) agar frontend bisa menampilkan penawaran.
b. accept_loan_offer(loan_id: Nat)
Tipe: update
Deskripsi: Fungsi bagi petani untuk menyetujui penawaran pinjaman.
Input: loan_id: Nat
Output: async Result<Text, Text>
Logika:
Dapatkan caller. Verifikasi bahwa caller adalah peminjam (borrower) dari loan_id tersebut.
Pastikan status pinjaman adalah #PendingApproval.
Panggilan Antar-Canister (Escrow): Panggil icrc7_transfer di Canister_RWA_NFT untuk mentransfer NFT agunan dari caller ke Principal canister ini (sebagai escrow).
Jika transfer berhasil, ubah status pinjaman menjadi #Approved.
Panggilan Antar-Canister (Pencairan): Panggil disburse_loan di Canister_Liquidity_Pool.
Jika panggilan pencairan berhasil, ubah status pinjaman menjadi #Active.
Kembalikan Result.ok("Loan approved, collateral secured, and disbursement initiated.").
c. get_loan_status(loan_id: Nat)
Tipe: query
Deskripsi: Mengambil detail dan status pinjaman.
Input: loan_id: Nat
Output: async ?Loan
Logika: Ambil data pinjaman dari loans map dan kembalikan.
5. Rencana Pengujian (Testing Plan)
Pengajuan Pinjaman Gagal (NFT tidak valid):
Panggil submit_loan_application dengan nft_id yang tidak dimiliki oleh caller.
Ekspektasi: Error dari verifikasi kepemilikan NFT.
Pengajuan Pinjaman Berhasil:
Gunakan caller yang memiliki NFT yang valid.
Panggil submit_loan_application.
Ekspektasi: Respon sukses 200 OK dengan detail pinjaman baru berstatus #PendingApproval.
Persetujuan Pinjaman oleh Pihak Tidak Berwenang:
Panggil accept_loan_offer dengan caller yang bukan peminjam.
Ekspektasi: Error "Unauthorized caller".
Persetujuan Pinjaman Berhasil:
Panggil accept_loan_offer sebagai peminjam yang sah.
Ekspektasi: Respon sukses. Status pinjaman berubah menjadi #Active. Kepemilikan NFT di Canister_RWA_NFT berpindah ke canister pinjaman.
Cek Status Pinjaman:
Panggil get_loan_status di setiap tahap untuk memverifikasi perubahan status.
Ekspektasi: Status yang dikembalikan sesuai dengan tahap terakhir yang berhasil dieksekusi.
