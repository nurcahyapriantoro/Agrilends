README: Fitur Mekanisme Likuidasi (Gagal Bayar)
Modul: liquidation
Canister Terkait: Canister_Manajemen_Pinjaman, Canister_RWA_NFT
1. Tujuan Fitur
Fitur ini menangani skenario gagal bayar, yang merupakan bagian krusial dari protokol pinjaman berbasis agunan. Tujuannya adalah untuk secara terprogram mengelola penyitaan agunan untuk menutupi kerugian pada liquidity pool.
2. Fungsi Kunci (Key Functions)
a. trigger_liquidation(loan_id: Nat)
Tipe: update
Deskripsi: Memulai proses likuidasi untuk pinjaman yang telah melewati batas waktu pembayaran.
Keamanan: Hanya dapat dipanggil oleh Principal admin atau melalui proses otomatis (heartbeat).
Logika:
Ambil data pinjaman. Verifikasi bahwa pinjaman sudah melewati periode gagal bayar (misalnya, 30 hari setelah jatuh tempo) dan statusnya adalah #Active.
Ubah status pinjaman menjadi #Defaulted.
Panggilan Antar-Canister (Transfer Agunan): Panggil icrc7_transfer di Canister_RWA_NFT untuk mentransfer NFT agunan dari escrow ke Liquidation Wallet (sebuah Principal yang ditunjuk untuk mengelola penjualan aset sitaan).
Atestasi On-Chain: Gunakan Threshold ECDSA (sign_with_ecdsa) untuk menandatangani pesan yang menyatakan bahwa loan_id telah dilikuidasi. Tanda tangan ini berfungsi sebagai bukti kriptografis yang dapat diverifikasi oleh pihak hukum atau mitra off-chain.
Penyeimbangan Akuntansi: Catat kerugian pada liquidity pool. Nilai kerugian adalah sisa utang pokok. Ini akan memengaruhi perhitungan APY bagi investor.
Kembalikan Result.ok("Liquidation process initiated.").
3. Keterkaitan dengan Modul Lain
Siklus Hidup Pinjaman: Mengubah status pinjaman menjadi #Defaulted.
Manajemen Agunan (RWA-NFT): Mengambil alih kepemilikan NFT agunan.
Threshold ECDSA: Memanfaatkan fitur canggih ICP untuk jembatan kepercayaan ke dunia off-chain.
4. Rencana Pengujian (Testing Plan)
Likuidasi Berhasil:
Panggil trigger_liquidation untuk pinjaman yang memenuhi kriteria gagal bayar.
Ekspektasi: Respon sukses. Status pinjaman menjadi #Defaulted. Kepemilikan NFT berpindah ke Liquidation Wallet.
Gagal Likuidasi (Pinjaman Belum Gagal Bayar):
Coba likuidasi pinjaman yang masih aktif dan dalam periode pembayaran.
Ekspektasi: Error "Loan is not eligible for liquidation."
