README: Fitur Penarikan Likuiditas oleh Investor
Modul: investor_actions
Canister Terkait: Canister_Liquidity_Pool
1. Tujuan Fitur
Fitur ini melengkapi siklus hidup investasi bagi para pemodal. Ini memungkinkan mereka untuk menarik dana (pokok + imbal hasil yang terakumulasi) dari liquidity pool, memberikan fleksibilitas dan melengkapi alur investasi mereka.
2. Canister Terkait
Logika ini berada di dalam Canister_Liquidity_Pool dan berinteraksi dengan Ledger ckBTC.
3. Fungsi Publik (Public Functions)
a. withdraw_liquidity(amount: Nat)
Tipe: update
Deskripsi: Memproses penarikan dana oleh investor.
Input: amount: Nat: Jumlah ckBTC yang ingin ditarik.
Output: async Result<Text, Text>
Logika & Keamanan:
Dapatkan caller (Investor).
Ambil saldo investor dari investor_balances. Pastikan amount yang diminta tidak melebihi saldo yang dimiliki.
Periksa apakah total_liquidity di pool cukup untuk menutupi amount. Jika tidak, kembalikan Result.err("Withdrawal failed due to insufficient available liquidity.").
Panggilan Antar-Canister (Kirim ckBTC): Panggil icrc1_transfer di ledger ckBTC untuk mengirim amount dari canister ini ke caller.
Jika transfer berhasil:
a. Kurangi amount dari total_liquidity.
b. Kurangi amount dari saldo caller di investor_balances.
Kembalikan Result.ok("Withdrawal successful.").
b. get_investor_balance()
Tipe: query
Deskripsi: Mengambil saldo terkini dari investor yang sedang login.
Input: -
Output: async Nat
Logika: Ambil dan kembalikan saldo caller dari investor_balances.
4. Rencana Pengujian (Testing Plan)
Penarikan Berhasil:
Panggil withdraw_liquidity dengan jumlah lebih kecil dari saldo investor dan total likuiditas.
Ekspektasi: Respon sukses. Saldo investor dan total_liquidity berkurang.
Gagal Tarik (Jumlah Melebihi Saldo):
Coba tarik jumlah yang lebih besar dari yang dimiliki investor.
Ekspektasi: Error "Withdrawal amount exceeds your balance."
Gagal Tarik (Likuiditas Pool Tidak Cukup):
Coba tarik dana saat sebagian besar likuiditas sedang digunakan untuk pinjaman aktif.
Ekspektasi: Error "Insufficient available liquidity."
