README: Fitur Pemeliharaan Otomatis (Heartbeat)
Modul: automated_maintenance
Canister Terkait: Semua canister utama
1. Tujuan Fitur
Fitur ini memanfaatkan fungsionalitas heartbeat dari Internet Computer untuk menjalankan tugas-tugas pemeliharaan secara otomatis dan periodik tanpa memerlukan pemicu eksternal. Ini membuat protokol lebih tangguh dan proaktif.
2. Konsep Implementasi
Mengimplementasikan fungsi canister_heartbeat di dalam canister yang relevan. Heartbeat akan dieksekusi oleh sistem ICP di setiap putaran (beberapa detik sekali).
// Di Canister_Manajemen_Pinjaman

// Fungsi ini akan dipanggil secara otomatis oleh sistem
system func heartbeat() : async () {
    // Cek pinjaman yang jatuh tempo
    for (loan in loans.values()) {
        if (is_loan_overdue(loan)) {
            // Kirim notifikasi atau tandai untuk likuidasi
            // ...
        };
    };
};


3. Tugas Otomatis yang Diimplementasikan
a. Pemeriksaan Pinjaman Gagal Bayar
Canister: Canister_Manajemen_Pinjaman
Logika heartbeat: Secara periodik, iterasi melalui semua pinjaman #Active. Jika ada pinjaman yang telah melewati tanggal jatuh tempo plus periode tenggang (misalnya, 30 hari), secara otomatis panggil fungsi internal untuk mengubah statusnya menjadi #Defaulted atau memicu notifikasi untuk admin agar meninjaunya.
b. Monitoring Cycles Otomatis
Canister: Canister_Kas_Protokol
Logika heartbeat: Secara periodik, panggil canister_status untuk setiap canister operasional. Jika cycles dari salah satu canister berada di bawah ambang batas aman, secara otomatis panggil fungsi top_up_canister_cycles untuk mengisi ulang.
4. Keterkaitan dengan Modul Lain
Mekanisme Likuidasi: Heartbeat adalah pemicu otomatis yang ideal untuk memulai proses likuidasi, mengurangi kebutuhan intervensi manual.
Manajemen Kas Protokol: Heartbeat mengubah fungsi pengisian cycles dari reaktif menjadi proaktif, mencegah canister berhenti beroperasi secara tak terduga.
