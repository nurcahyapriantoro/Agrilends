README: Fitur Manajemen Agunan RWANFT
Modul: rwa_nft_management
Canister Terkait: Canister_RWA_NFT
1. Tujuan Fitur
Fitur ini bertanggung jawab untuk menokenisasi aset dunia nyata (Resi Gudang) menjadi Non-Fungible Token (NFT) menggunakan standar ICRC-7. NFT ini akan berfungsi sebagai agunan digital yang tidak dapat diubah dan dapat diverifikasi di dalam platform Agrilends.
2. Canister Terkait
Seluruh logika NFT akan dienkapsulasi di dalam Canister_RWA_NFT. Canister ini harus mengimplementasikan antarmuka standar ICRC-7.
3. Struktur Data (Types)
Struktur data akan mengikuti standar ICRC-7, dengan penekanan khusus pada metadata.
// src/rwa_nft/types.mo

// Metadata akan menjadi sebuah Vec dari pasangan kunci-nilai
// Contoh metadata untuk satu NFT:
// [
//   ("rwa:legal_doc_hash", {Text = "sha256_hash_of_pdf"}),
//   ("rwa:valuation_idr", {Nat = 300_000_000}),
//   ("rwa:asset_description", {Text = "Gabah, 20 Ton, Kualitas A"}),
//   ("immutable", {Bool = true}) // Properti kustom untuk menandai metadata inti
// ]


4. Fungsi Publik (Public Functions)
a. mint_nft(owner: Principal, metadata: Vec<(Text, Value)>)
Tipe: update
Deskripsi: Mencetak NFT baru. Fungsi ini harus sangat dilindungi.
Input:
owner: Principal: Principal dari petani yang memiliki aset.
metadata: Vec<(Text, Value)>: Metadata yang terkait dengan aset.
Output: async Result<Nat, Text> (mengembalikan token_id)
Logika & Keamanan:
KONTROL AKSES KRITIS: Dapatkan caller. Verifikasi bahwa caller adalah Principal dari Canister_Manajemen_Pinjaman. Jika tidak, hentikan eksekusi dengan trap("Unauthorized: Only the loan manager can mint NFTs.").
Validasi metadata: Pastikan semua field yang diperlukan (misalnya, rwa:legal_doc_hash) ada di dalam input.
Gunakan logika internal standar ICRC-7 untuk membuat token baru dan men-assign owner.
Simpan metadata ke dalam stable storage yang terasosiasi dengan token_id yang baru.
Kembalikan Result.ok(new_token_id).
b. icrc7_transfer(from: Account, to: Account, token_id: Nat)
Tipe: update
Deskripsi: Fungsi standar ICRC-7 untuk mentransfer kepemilikan NFT. Akan digunakan untuk memindahkan NFT ke escrow dan mengembalikannya.
Logika & Keamanan:
Implementasi standar ICRC-7.
Pastikan caller memiliki izin untuk melakukan transfer (biasanya caller harus sama dengan from).
c. get_nft_details(token_id: Nat)
Tipe: query
Deskripsi: Mengambil semua detail dari NFT tertentu.
Input: token_id: Nat
Output: async ?(Principal, Vec<(Text, Value)>) (pemilik dan metadata)
Logika: Ambil data pemilik dan metadata dari stable storage berdasarkan token_id dan kembalikan.
5. Rencana Pengujian (Testing Plan)
Gagal Minting oleh Pengguna Asing:
Gunakan Principal pengguna biasa (bukan canister pinjaman) untuk memanggil mint_nft.
Ekspektasi: Panggilan gagal dengan pesan Canister trapped: Unauthorized....
Berhasil Minting (Simulasi):
Untuk tujuan pengujian, tambahkan Principal admin/developer ke daftar yang diizinkan untuk memanggil mint_nft.
Panggil mint_nft dengan Principal yang diizinkan.
Ekspektasi: Respon sukses 200 OK dengan token_id baru.
Verifikasi Detail NFT:
Setelah minting berhasil, panggil get_nft_details dengan token_id yang baru.
Ekspektasi: Respon sukses dengan data pemilik dan metadata yang benar.
Transfer NFT Berhasil:
Panggil icrc7_transfer sebagai pemilik NFT.
Ekspektasi: Respon sukses.
Verifikasi: Panggil get_nft_details lagi untuk memastikan pemilik telah berubah.
Gagal Transfer NFT oleh Pihak Tidak Berwenang:
Panggil icrc7_transfer menggunakan caller yang bukan pemilik NFT.
Ekspektasi: Panggilan gagal dengan error otorisasi dari standar ICRC-7.
