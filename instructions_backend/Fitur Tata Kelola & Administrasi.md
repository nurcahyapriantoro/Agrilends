README: Fitur Tata Kelola & Administrasi
Modul: governance
Canister Terkait: Semua canister (melalui kontrol akses)
1. Tujuan Fitur
Fitur ini menyediakan mekanisme bagi admin atau DAO (di masa depan) untuk mengelola dan memperbarui parameter kunci protokol tanpa perlu melakukan upgrade kode canister. Ini penting untuk adaptasi dan manajemen risiko jangka panjang.
2. Struktur Data (Types)
// Di setiap canister yang relevan

// Menyimpan Principal admin yang berwenang
var admin: Principal = Principal.fromText("your_admin_principal");

// Contoh parameter di Canister_Manajemen_Pinjaman
var loan_to_value_ratio: Nat = 60; // LTV 60%
var base_apr: Nat = 10; // APR 10%


3. Fungsi Publik (Public Functions)
a. set_protocol_parameter(key: Text, value: Nat)
Tipe: update
Deskripsi: Mengubah nilai parameter protokol.
Keamanan: Sangat kritis. Harus dilindungi dengan ketat.
Logika:
KONTROL AKSES KRITIS: Verifikasi bahwa caller adalah admin. Jika tidak, trap("Unauthorized").
Gunakan switch pada key untuk menentukan parameter mana yang akan diubah.
case "ltv": loan_to_value_ratio := value;
case "apr": base_apr := value;
Kembalikan Result.ok("Parameter updated.").
b. transfer_admin_role(new_admin: Principal)
Tipe: update
Deskripsi: Mentransfer kepemilikan administrasi ke Principal baru.
Logika:
Verifikasi caller adalah admin saat ini.
admin := new_admin;
Kembalikan Result.ok("Admin role transferred.").
4. Rencana Pengujian (Testing Plan)
Update Parameter Berhasil oleh Admin:
Panggil set_protocol_parameter sebagai admin.
Ekspektasi: Respon sukses.
Verifikasi: Panggil fungsi lain yang menggunakan parameter tersebut (misal, submit_loan_application) untuk memastikan nilai baru digunakan.
Gagal Update oleh Pengguna Biasa:
Panggil set_protocol_parameter sebagai pengguna biasa.
Ekspektasi: Panggilan gagal dengan Canister trapped: Unauthorized.
