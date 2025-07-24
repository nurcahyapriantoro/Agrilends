README: Fitur Pelunasan Pinjaman & Penarikan Agunan
Modul: loan_repayment
Canister Terkait: Canister_Manajemen_Pinjaman
1. Tujuan Fitur
Fitur ini melengkapi siklus hidup pinjaman dari sisi peminjam. Ini memungkinkan peminjam untuk melakukan pembayaran kembali pinjaman mereka (dalam bentuk ckBTC) dan, setelah lunas, secara otomatis menerima kembali agunan RWA-NFT mereka dari escrow.
2. Canister Terkait
Logika ini merupakan bagian inti dari Canister_Manajemen_Pinjaman dan akan berinteraksi erat dengan Canister_RWA_NFT dan Canister_Kas_Protokol.
3. Struktur Data (Types)
Kita akan memperluas type Loan untuk melacak pembayaran.
// src/loan_manager/types.mo

public type Payment = {
    amount: Nat;
    timestamp: Time.Time;
};

// Di dalam type Loan, tambahkan:
public type Loan = {
    // ... field yang sudah ada
    total_repaid: Nat;
    repayment_history: Vec<Payment>;
};


4. Fungsi Publik (Public Functions)
a. repay_loan(loan_id: Nat, amount: Nat)
Tipe: update
Deskripsi: Memproses pembayaran kembali dari peminjam.
Input:
loan_id: Nat: ID pinjaman yang akan dibayar.
amount: Nat: Jumlah ckBTC yang dibayarkan.
Output: async Result<Text, Text>
Logika & Keamanan:
Dapatkan caller. Verifikasi bahwa caller adalah peminjam (borrower) dari loan_id.
Ambil data pinjaman. Pastikan statusnya adalah #Active.
Panggilan Antar-Canister (Terima Pembayaran): Panggil icrc1_transfer_from di ledger ckBTC untuk menarik amount dari caller. Ini memerlukan approve dari peminjam sebelumnya.
Jika transfer berhasil:
a. Hitung total utang yang tersisa (pokok + bunga akumulasi).
b. Perbarui total_repaid pada objek pinjaman.
c. Tambahkan Payment baru ke repayment_history.
d. Panggilan Antar-Canister (Kirim Biaya): Hitung bagian bunga untuk protokol (misalnya, 10% dari porsi bunga pembayaran) dan panggil collect_fees di Canister_Kas_Protokol.
Logika Pelunasan: Jika total_repaid >= total utang:
a. Ubah status pinjaman menjadi #Repaid.
b. Panggilan Antar-Canister (Kembalikan Agunan): Panggil icrc7_transfer di Canister_RWA_NFT untuk mentransfer NFT agunan dari escrow (canister ini) kembali ke peminjam.
Kembalikan Result.ok("Repayment successful. Loan status: [status baru]").
5. Rencana Pengujian (Testing Plan)
Pembayaran Parsial Berhasil:
Panggil repay_loan dengan jumlah lebih kecil dari total utang.
Ekspektasi: Respon sukses. total_repaid diperbarui, status tetap #Active. Saldo di Canister_Kas_Protokol bertambah.
Pembayaran Lunas Berhasil:
Panggil repay_loan dengan jumlah yang cukup untuk melunasi sisa utang.
Ekspektasi: Respon sukses. Status pinjaman menjadi #Repaid. Kepemilikan NFT di Canister_RWA_NFT kembali ke peminjam.
Gagal Bayar oleh Pihak Tidak Berwenang:
Panggil repay_loan dengan caller yang bukan peminjam.
Ekspektasi: Error "Unauthorized caller".
