README: Fitur Log Audit & Aktivitas
Modul: logging
Canister Terkait: Semua canister
1. Tujuan Fitur
Fitur ini menyediakan jejak audit yang tidak dapat diubah (immutable) dari semua transaksi dan peristiwa penting yang terjadi di dalam protokol. Ini krusial untuk transparansi, debugging, dan membangun kepercayaan pengguna.
2. Implementasi
Daripada membuat canister terpisah, setiap canister akan memiliki fungsi logging internalnya sendiri yang menulis ke Vec atau StableBTreeMap yang terbatas ukurannya (untuk mengelola penyimpanan).
// Di setiap canister, mis. Canister_Manajemen_Pinjaman

public type LogEntry = {
    timestamp: Time.Time;
    event: Text; // mis. "LOAN_CREATED", "REPAYMENT_PROCESSED"
    details: Vec<(Text, Text)>;
};

// Gunakan buffer melingkar untuk menyimpan N log terakhir
var activity_log: [var LogEntry] = Array.init(1000, null);
var log_cursor: Nat = 0;

func add_log(event: Text, details: Vec<(Text, Text)>) {
    let entry = { timestamp = Time.now(); event; details };
    activity_log[log_cursor] := ?entry;
    log_cursor := (log_cursor + 1) % activity_log.size();
};


3. Fungsi Publik
a. get_activity_log()
Tipe: query
Deskripsi: Mengambil N log aktivitas terakhir dari canister.
Output: async [?LogEntry]
Logika: Kembalikan seluruh array activity_log.
4. Contoh Penggunaan
Setelah submit_loan_application berhasil, panggil:
add_log("LOAN_CREATED", [("loan_id", Nat.toText(new_loan.id))])
Setelah repay_loan berhasil, panggil:
add_log("REPAYMENT_PROCESSED", [("loan_id", Nat.toText(loan_id)), ("amount", Nat.toText(amount))])
5. Rencana Pengujian
Lakukan beberapa aksi (buat pinjaman, bayar, dll.).
Panggil get_activity_log.
Ekspektasi: Respon berisi daftar log yang akurat dan sesuai dengan urutan aksi yang dilakukan.
