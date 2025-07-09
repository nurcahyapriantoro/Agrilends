README: Fitur Sistem Notifikasi On-Chain
Modul: notification_system
Canister Terkait: Canister_Manajemen_Pinjaman (atau Canister Notifikasi terpisah)
1. Tujuan Fitur
Fitur ini bertujuan untuk memberikan notifikasi on-chain kepada pengguna (Petani dan Investor) tentang peristiwa penting yang terjadi pada akun atau transaksi mereka. Ini secara drastis meningkatkan pengalaman pengguna, karena mereka tidak perlu terus-menerus memeriksa status secara manual.
2. Konsep Implementasi
Setiap pengguna akan memiliki "kotak masuk" notifikasi di dalam stable storage. Canister lain akan mengirim pesan ke sistem ini ketika peristiwa penting terjadi.
// src/loan_manager/types.mo

public type Notification = {
    id: Nat;
    recipient: Principal;
    event_type: Text; // mis. "LOAN_OFFER_READY", "PAYMENT_RECEIVED", "LIQUIDITY_WITHDRAWN"
    message: Text;
    is_read: Bool;
    timestamp: Time.Time;
};

// Kunci: Principal pengguna, Nilai: Vektor notifikasi untuk pengguna tersebut
var user_notifications: StableBTreeMap<Principal, Vec<Notification>> = StableBTreeMap.init(Principal.compare);


3. Fungsi Kunci
a. create_notification(recipient: Principal, event_type: Text, message: Text)
Tipe: update (private atau hanya bisa dipanggil oleh canister lain dalam protokol)
Deskripsi: Membuat entri notifikasi baru untuk pengguna.
Keamanan: Fungsi ini tidak boleh diekspos ke publik. Ini adalah fungsi internal yang dipanggil oleh fungsi lain (misalnya, submit_loan_application).
Logika:
Buat objek Notification baru.
Ambil daftar notifikasi untuk recipient.
Tambahkan notifikasi baru ke daftar tersebut.
Simpan kembali daftar yang telah diperbarui ke user_notifications.
b. get_my_notifications()
Tipe: query
Deskripsi: Mengambil semua notifikasi untuk pengguna yang sedang login.
Output: async Vec<Notification>
Logika: Ambil dan kembalikan daftar notifikasi untuk caller.
c. mark_notification_as_read(notification_id: Nat)
Tipe: update
Deskripsi: Menandai notifikasi sebagai telah dibaca.
Logika:
Cari notifikasi dengan notification_id di dalam daftar notifikasi caller.
Ubah is_read menjadi true.
4. Keterkaitan dengan Modul Lain
Siklus Hidup Pinjaman: Setelah submit_loan_application berhasil menghitung penawaran, ia akan memanggil create_notification untuk memberitahu petani, "Penawaran pinjaman Anda siap untuk ditinjau."
Pelunasan Pinjaman: Setelah repay_loan berhasil, ia akan memanggil create_notification untuk konfirmasi, "Pembayaran Anda telah kami terima."
Penarikan Likuiditas: Setelah withdraw_liquidity berhasil, ia akan memanggil create_notification untuk investor.
