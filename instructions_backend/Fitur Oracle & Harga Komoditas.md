README: Fitur Oracle & Harga Komoditas
Modul: oracle_management
Canister Terkait: Canister_Manajemen_Pinjaman (atau Canister Oracle terpisah)
1. Tujuan Fitur
Fitur ini bertujuan untuk secara aman dan andal mengambil data harga komoditas dari dunia luar (Web2 API) dan menyediakannya untuk digunakan di dalam dApp, terutama untuk menilai agunan. Fitur ini memanfaatkan kemampuan HTTPS Outcalls dari Internet Computer untuk berfungsi sebagai oracle yang trustless.
2. Canister Terkait
Untuk MVP, logika ini bisa diintegrasikan langsung ke dalam Canister_Manajemen_Pinjaman untuk menyederhanakan arsitektur. Di masa depan, bisa dipisahkan menjadi Canister_Oracle khusus.
3. Struktur Data (Types)
// src/loan_manager/types.mo atau src/oracle/types.mo

// Menyimpan harga terakhir yang diambil untuk setiap komoditas
// Kunci adalah ID komoditas, mis. "RICE_IDR"
var price_feeds: StableBTreeMap<Text, Nat> = StableBTreeMap.init(Text.compare);

// Menyimpan timestamp terakhir kali harga diperbarui
var last_updated: StableBTreeMap<Text, Time.Time> = StableBTreeMap.init(Text.compare);


4. Fungsi Kunci (Key Functions)
a. fetch_commodity_price(commodity_id: Text)
Tipe: update (karena HTTPS Outcalls memerlukan konsensus)
Deskripsi: Membuat panggilan HTTPS GET ke API harga eksternal.
Keamanan: Dapat dijadwalkan untuk berjalan secara periodik menggunakan heartbeat atau dipanggil oleh admin.
Logika:
Bentuk HttpRequest untuk memanggil API eksternal (misalnya, API dari Kementerian Perdagangan atau penyedia data lainnya). Pastikan API mendukung IPv6.
Krusial: Definisikan fungsi transform yang menerima HttpResponse mentah.
Di dalam transform, parse body JSON dari respons.
Ekstrak hanya nilai harga (misalnya, 15000).
Buang semua data lain (header, timestamp dari API, dll.) untuk memastikan semua node mencapai konsensus pada data yang sama persis.
Kembalikan HttpResponse yang sudah dibersihkan.
Panggil IC.http_request dengan HttpRequest yang telah dibuat dan fungsi transform.
Setelah mendapatkan respons yang telah ditransformasi, perbarui price_feeds dan last_updated map.
b. get_commodity_price(commodity_id: Text)
Tipe: query
Deskripsi: Mengambil harga terakhir yang valid dari stable storage.
Output: async Result<Nat, Text>
Logika:
Cari commodity_id di dalam price_feeds.
Jika ditemukan, kembalikan Result.ok(price).
Jika tidak, kembalikan Result.err("Price not available for this commodity.").
5. Rencana Pengujian (Testing Plan)
Pengambilan Harga Berhasil:
Panggil fetch_commodity_price.
Ekspektasi: Panggilan berhasil (memerlukan cycles untuk membayar).
Verifikasi: Panggil get_commodity_price dan pastikan harga yang dikembalikan sesuai dengan yang diharapkan dari API.
Pengujian Fungsi transform:
Secara manual, berikan contoh respons JSON mentah ke fungsi transform.
Ekspektasi: Output dari transform hanya berisi body dengan nilai harga yang sudah diekstrak dan header yang minimal.
Mengambil Harga yang Belum Ada:
Panggil get_commodity_price untuk komoditas yang belum pernah di-fetch.
Ekspektasi: Respon error "Price not available...".
6. Keterkaitan dengan Modul Lain
Siklus Hidup Pinjaman: Fungsi submit_loan_application di Canister_Manajemen_Pinjaman akan memanggil get_commodity_price dari modul ini untuk menghitung nilai agunan (collateral_value_btc) secara dinamis.
Dokumentasi Proyek Utama
Ini adalah README.md tingkat atas yang mengikat semua modul menjadi satu kesatuan.
README: Proyek Backend Agrilends
Selamat datang di repositori backend untuk Agrilends, sebuah platform keuangan agrikultur terdesentralisasi yang dibangun di atas Internet Computer.
1. Gambaran Umum & Arsitektur
Agrilends menggunakan arsitektur multi-canister untuk memastikan keamanan, skalabilitas, dan kemudahan pemeliharaan. Setiap canister bertanggung jawab atas satu domain fungsional yang jelas.
Canister_Manajemen_Pinjaman: Otak dari aplikasi, mengelola pengguna dan siklus hidup pinjaman.
Canister_RWA_NFT: Ledger untuk agunan yang ditokenisasi (standar ICRC-7).
Canister_Liquidity_Pool: Mengelola dana investor (ckBTC) dan pencairan.
Canister_Kas_Protokol: Bendahara protokol untuk mengelola pendapatan dan biaya cycles.
2. Panduan Fitur & Modul
Proyek ini dibagi menjadi beberapa modul fungsional. Untuk detail implementasi dan pengujian setiap modul, silakan merujuk ke README.md masing-masing:
Manajemen Pengguna & Otentikasi
Mengelola pendaftaran dan data pengguna (Petani & Investor) menggunakan Internet Identity.
Lihat README: Fitur Manajemen Pengguna untuk detail.
Manajemen Agunan (RWA-NFT)
Mencetak dan mengelola NFT (ICRC-7) yang merepresentasikan Resi Gudang.
Lihat README: Fitur Manajemen Agunan (RWA-NFT) untuk detail.
Siklus Hidup Pinjaman
Mengelola alur kerja pinjaman dari pengajuan, penilaian, persetujuan, hingga pelunasan.
Lihat README: Fitur Siklus Hidup Pinjaman untuk detail.
Manajemen Likuiditas & Pencairan
Mengelola deposit dari investor dan mencairkan dana pinjaman ke petani.
Lihat README: Fitur Manajemen Likuiditas & Pencairan untuk detail.
Manajemen Kas & Biaya Protokol
Mengumpulkan pendapatan platform dan mendanai biaya operasional cycles.
Lihat README: Fitur Manajemen Kas & Biaya Protokol untuk detail.
Oracle & Harga Komoditas
Mengambil data harga dari API eksternal secara aman menggunakan HTTPS Outcalls.
Lihat README: Fitur Oracle & Harga Komoditas untuk detail.
3. Diagram Interaksi Antar-Canister
Diagram berikut menunjukkan bagaimana canister-canister utama saling berinteraksi.
graph TD
    subgraph Pengguna
        A[Frontend React]
    end

    subgraph "Backend Agrilends (ICP)"
        B(Manajemen Pinjaman)
        C(RWA-NFT ICRC-7)
        D(Liquidity Pool ckBTC)
        E(Kas Protokol)
    end

    subgraph "Layanan Eksternal"
        F[Ledger & Minter ckBTC]
        G[API Harga Komoditas]
    end

    A -- "submit_loan_application()" --> B
    A -- "deposit_liquidity()" --> D

    B -- "Verifikasi & Transfer NFT" --> C
    B -- "Minta Pencairan Dana" --> D
    B -- "Kirim Biaya" --> E
    B -- "Ambil Harga" --> G

    D -- "Interaksi ckBTC" --> F


4. Setup & Deployment Lokal
Instal DFX: Pastikan Anda memiliki versi terbaru dari DFINITY Canister SDK.
sh -ci "$(curl -fsSL https://internetcomputer.org/install.sh)"


Clone Repositori:
git clone <url_repositori_anda>
cd agrilends-backend


Mulai Jaringan Lokal:
dfx start --background --clean


Deploy Semua Canister:
dfx deploy


Jalankan Pengujian: Ikuti rencana pengujian di setiap README.md fitur menggunakan dfx canister call atau Postman.
