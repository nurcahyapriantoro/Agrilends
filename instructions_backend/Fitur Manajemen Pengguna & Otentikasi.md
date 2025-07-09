Modul: user_management
Canister Terkait: Canister_Manajemen_Pinjaman
1. Tujuan Fitur
Fitur ini bertujuan untuk menyediakan fondasi untuk mengelola data pengguna (Petani & Investor) di platform Agrilends. Setiap pengguna akan diidentifikasi secara unik oleh Principal mereka yang berasal dari otentikasi Internet Identity.
2. Canister Terkait
Untuk mempercepat pengembangan MVP, logika dan data untuk fitur ini akan ditempatkan di dalam Canister_Manajemen_Pinjaman.
3. Struktur Data (Types)
Data pengguna akan disimpan dalam StableBTreeMap untuk memastikan persistensi data saat canister di-upgrade.
// src/loan_manager/types.mo

// Mendefinisikan peran pengguna
public type Role = {
    #Farmer;
    #Investor;
};

// Struktur data utama untuk pengguna
public type User = {
    id: Principal;
    role: Role;
    createdAt: Time.Time;
    btcAddress: ?Text; // Alamat BTC untuk pencairan, opsional saat registrasi
};

// Penyimpanan data pengguna
var users: StableBTreeMap<Principal, User> = StableBTreeMap.init(Principal.compare);


4. Fungsi Publik (Public Functions)
a. register_as_farmer()
Tipe: update
Deskripsi: Mendaftarkan caller sebagai pengguna dengan peran "Petani".
Input: -
Output: async Result<User, Text>
Logika:
Dapatkan caller dari pesan.
Periksa apakah caller sudah ada di dalam users map. Jika ya, kembalikan Result.err("User already registered").
Buat instance User baru dengan role = #Farmer.
Masukkan pengguna baru ke dalam users map.
Kembalikan Result.ok(newUser).
b. register_as_investor()
Tipe: update
Deskripsi: Mendaftarkan caller sebagai pengguna dengan peran "Investor".
Input: -
Output: async Result<User, Text>
Logika: Sama seperti register_as_farmer(), tetapi dengan role = #Investor.
c. get_user()
Tipe: query
Deskripsi: Mengambil detail pengguna untuk caller yang sedang terotentikasi.
Input: -
Output: async Result<User, Text>
Logika:
Dapatkan caller dari pesan.
Cari caller di dalam users map.
Jika ditemukan, kembalikan Result.ok(user).
Jika tidak ditemukan, kembalikan Result.err("User not found. Please register first.").
5. Rencana Pengujian (Testing Plan)
Gunakan Postman atau dfx untuk menguji skenario berikut:
Registrasi Petani Baru:
Panggil register_as_farmer().
Ekspektasi: Respon sukses 200 OK dengan data pengguna yang baru dibuat.
Registrasi Investor Baru:
Panggil register_as_investor().
Ekspektasi: Respon sukses 200 OK dengan data pengguna yang baru dibuat.
Mencoba Registrasi Ganda:
Panggil register_as_farmer() dua kali dengan caller yang sama.
Ekspektasi: Panggilan kedua menghasilkan respon error dengan pesan "User already registered".
Mengambil Data Pengguna Terdaftar:
Setelah registrasi, panggil get_user().
Ekspektasi: Respon sukses 200 OK dengan data pengguna yang sesuai.
Mengambil Data Pengguna Tidak Terdaftar:
Gunakan Principal baru yang belum pernah mendaftar, panggil get_user().
Ekspektasi: Respon error dengan pesan "User not found. Please register first.".
