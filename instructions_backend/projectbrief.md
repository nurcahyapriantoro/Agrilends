Laporan Brief Proyek Agrilends untuk Tim Pengembangan
Dokumen ini berfungsi sebagai panduan terpusat untuk seluruh tim pengembangan Agrilends. Tujuannya adalah untuk menyelaraskan pemahaman kita tentang visi proyek, arsitektur teknis, dan tanggung jawab spesifik untuk setiap peran.
1. Visi & Misi Proyek
Visi: Menjadi platform keuangan terdesentralisasi (DeFi) terdepan di Indonesia yang menjembatani kesenjangan pembiayaan di sektor agrikultur.
Misi: Memberdayakan petani dan koperasi dengan menyediakan akses ke likuiditas global secara adil dan transparan, menggunakan aset agrikultur nyata (Resi Gudang) sebagai agunan yang ditokenisasi di Internet Computer (ICP).
Masalah yang Dipecahkan: Mengatasi masalah sulitnya akses petani ke pembiayaan, sekaligus memotong peran perantara yang tidak efisien dan seringkali eksploitatif.
2. Tujuan Utama (Fokus MVP untuk WCHL25)
Tujuan utama kita dalam waktu dekat adalah membangun Minimum Viable Product (MVP) yang fungsional dan dapat didemonstrasikan untuk kompetisi World Championship Hackathon League (WCHL). MVP ini harus mencakup satu alur pengguna ujung-ke-ujung yang lengkap:
Investor menyediakan likuiditas ckBTC ke dalam Liquidity Pool.
Petani mendaftar, mengunggah Resi Gudang, dan mengajukan pinjaman.
Sistem menokenisasi Resi Gudang menjadi RWA-NFT.
Pinjaman disetujui dan dana ckBTC dicairkan ke petani.
Petani melakukan satu kali pembayaran kembali.
3. Arsitektur & Tumpukan Teknologi Inti
Kita akan mengadopsi arsitektur multi-canister yang modular dan aman di Internet Computer.
Canister Frontend: Menyajikan antarmuka React kepada pengguna.
Canister Manajemen Pinjaman (Motoko): Otak dari platform, mengelola seluruh alur logika pinjaman.
Canister RWA-NFT (Motoko/ICRC-7): Ledger khusus untuk agunan NFT yang merepresentasikan Resi Gudang.
Canister Liquidity Pool (Motoko, migrasi ke Rust): Mengelola dana ckBTC dari investor. Ini adalah komponen paling kritis dari segi keamanan.
Canister Kas Protokol (Motoko): Mengelola pendapatan protokol dan biaya operasional (cycles).
Bahasa & Interoperabilitas: Kita akan menggunakan pendekatan hibrida. Motoko untuk pengembangan cepat logika bisnis, dan merencanakan migrasi komponen kritis (Liquidity Pool) ke Rust untuk keamanan maksimal. Interaksi antar canister dijamin oleh Candid.
4. Rincian Tugas & Tanggung Jawab Berdasarkan Peran
Berikut adalah fokus utama untuk setiap peran dalam tim:
A. Untuk Pengembang Backend (Motoko/Rust)
Anda bertanggung jawab untuk membangun tulang punggung dari platform Agrilends.
Tugas Utama: Mengimplementasikan logika untuk semua canister backend.
Desain Canister:
Manajemen Pinjaman: Implementasikan fungsi untuk submit_loan_application, get_loan_status, request_disbursement, dan notify_repayment.
RWA-NFT (ICRC-7): Implementasikan standar ICRC-7, dengan fokus pada fungsi icrc7_mint yang aman (hanya bisa dipanggil oleh canister pinjaman) dan icrc7_transfer untuk proses escrow. Pastikan metadata NFT (hash dokumen, valuasi) bersifat immutable.
Liquidity Pool: Implementasikan fungsi untuk deposit_liquidity dan withdraw_liquidity. Ini adalah kandidat utama untuk implementasi Rust.
Implementasi Fitur Kunci ICP (Sangat Penting):
Integrasi ckBTC: Gunakan API dari ckBTC Minter untuk retrieve_btc_with_approval (pencairan pinjaman) dan update_balance (pembayaran kembali).
HTTPS Outcalls: Buat fungsi untuk memanggil API harga komoditas eksternal secara periodik. Wajib mengimplementasikan fungsi transform untuk membersihkan respons JSON dan memastikan konsensus.
Threshold ECDSA: Implementasikan fungsi sign_with_ecdsa untuk membuat atestasi digital saat terjadi gagal bayar. Ini akan digunakan untuk verifikasi oleh pihak hukum off-chain.
Keamanan & Data:
Pencegahan Re-entrancy: Terapkan mekanisme locking (misalnya, dengan Bool atau HashMap) sebelum melakukan panggilan await eksternal.
Idempotensi: Pastikan fungsi kritis seperti deposit dan pembayaran kembali menggunakan ID unik dari klien untuk mencegah eksekusi ganda.
Kontrol Akses: Gunakan pemeriksaan caller di setiap fungsi yang memiliki hak istimewa.
Penyimpanan: Semua data state (catatan pinjaman, saldo, dll.) wajib disimpan dalam Stable Memory untuk bertahan saat proses upgrade.
B. Untuk Pengembang Frontend (React)
Anda bertanggung jawab untuk menciptakan pengalaman pengguna yang mulus dan menyembunyikan semua kompleksitas Web3 dari pengguna.
Tugas Utama: Membangun antarmuka pengguna (UI) yang responsif dan intuitif menggunakan React.
Tumpukan Teknologi: React, agent-js untuk komunikasi dengan backend, dan UI akan di-host di assets canister.
Integrasi Kunci:
Internet Identity (II): Ini adalah prioritas utama. Implementasikan alur onboarding yang menggunakan II untuk otentikasi biometrik. Hindari istilah "Connect Wallet"; gunakan "Masuk/Daftar". Pengguna tidak boleh tahu tentang seed phrase.
Alur Pengguna untuk Diimplementasikan:
Halaman Peminjam (Petani):
Formulir login/registrasi via II.
Dasbor untuk melihat status pinjaman.
Formulir pengajuan pinjaman dengan fitur unggah dokumen (Resi Gudang).
Halaman untuk melihat penawaran pinjaman dan menyetujuinya.
Halaman Pemodal (Investor):
Formulir login/registrasi via II.
Dasbor untuk melihat statistik Liquidity Pool (Total Dana, APY).
Formulir untuk deposit dan penarikan ckBTC.
Komunikasi Backend: Gunakan agent-js untuk memanggil fungsi-fungsi publik yang diekspos oleh canister backend (misalnya, get_loan_status(loan_id)).
C. Untuk Pimpinan Proyek / Analis Bisnis
Anda bertanggung jawab untuk menjaga proyek tetap pada jalurnya, mengelola sumber daya, dan memastikan kita membangun produk yang tepat untuk pasar yang tepat.
Tugas Utama: Mengelola siklus hidup proyek, menyelaraskan tim, dan menjembatani antara kebutuhan teknis dan bisnis.
Fokus Strategis:
MVP Scope: Pastikan tim tetap fokus pada alur kerja MVP yang telah ditentukan untuk WCHL25.
Naratif & Dampak: Kembangkan narasi yang kuat yang berpusat pada dampak dunia nyata dari Agrilends untuk presentasi dan materi pemasaran.
Manajemen Risiko & Kemitraan:
Kepatuhan Regulasi: Bekerja sama dengan konsultan hukum untuk memahami dan merencanakan kepatuhan terhadap regulasi OJK.
Kemitraan Off-Chain: Mulai jalin komunikasi dengan pemangku kepentingan di ekosistem Sistem Resi Gudang (SRG) Indonesia (Pengelola Gudang, LPK). Ini krusial untuk validasi aset.
Manajemen Sumber Daya:
Pantau biaya operasional protokol (cycles) dan pastikan model pendapatan (biaya administrasi, bagi hasil bunga) cukup untuk menopang platform.
D. Untuk Konsultan Hukum / Kepatuhan
Peran Anda sangat penting untuk memastikan keberlanjutan jangka panjang dan legitimasi Agrilends di Indonesia.
Tugas Utama: Menganalisis dan menavigasi lanskap hukum Indonesia untuk aset digital dan layanan keuangan.
Fokus Regulasi Utama:
Transisi OJK: Fokus utama adalah pada implikasi peralihan pengawasan aset kripto dari BAPPEBTI ke Otoritas Jasa Keuangan (OJK), sesuai UU P2SK dan POJK No. 27 Tahun 2024.
Area Analisis Kritis:
Status RWA-NFT: Bagaimana OJK akan mengklasifikasikan NFT yang didukung oleh Resi Gudang?
Persyaratan Perizinan: Apakah Agrilends perlu mendaftar sebagai Penyelenggara Perdagangan Aset Keuangan Digital atau platform layanan keuangan lainnya?
KYC/AML: Apa saja persyaratan Kenali Pelanggan Anda (KYC) dan Anti Pencucian Uang (AML) yang perlu kita implementasikan?
Penggunaan ckBTC: Analisis legalitas penggunaan ckBTC sebagai instrumen pinjaman dan pembayaran bunga di Indonesia.
Kerangka Hukum SRG: Pastikan model operasional kita selaras dengan UU No. 9 Tahun 2011 tentang Sistem Resi Gudang.
5. Langkah Selanjutnya & Faktor Keberhasilan
Penyelarasan Tim: Semua anggota tim harus membaca dan memahami dokumen ini.
Sprint Planning: Segera lakukan perencanaan sprint untuk mengalokasikan tugas-tugas di atas.
Fokus: Tetap fokus pada MVP. Fitur-fitur canggih lainnya dapat ditambahkan pasca-hackathon.
Kolaborasi: Komunikasi yang terbuka dan sering antar peran adalah kunci keberhasilan.
Mari kita bangun solusi yang tidak hanya canggih secara teknis, tetapi juga memberikan dampak nyata dan positif bagi sektor agrikultur Indonesia.
