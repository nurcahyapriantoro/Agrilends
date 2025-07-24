# Fitur Pelunasan Pinjaman & Penarikan Agunan - API Documentation

## Overview
Fitur pelunasan pinjaman dan penarikan agunan adalah bagian inti dari sistem Agrilends yang memungkinkan peminjam untuk melakukan pembayaran kembali pinjaman mereka dan secara otomatis menerima kembali agunan RWA-NFT setelah lunas.

## Module: loan_repayment

### Core Functions

#### 1. repay_loan(loan_id: u64, amount: u64) -> Result<RepaymentResponse, String>
**Tipe**: update  
**Deskripsi**: Memproses pembayaran kembali dari peminjam  
**Access**: Borrower only

**Input**:
- `loan_id`: ID pinjaman yang akan dibayar
- `amount`: Jumlah ckBTC yang dibayarkan (dalam satoshi)

**Output**: RepaymentResponse dengan informasi:
- `success`: Status keberhasilan pembayaran
- `message`: Pesan detail hasil pembayaran
- `transaction_id`: ID transaksi ckBTC (block index)
- `new_loan_status`: Status pinjaman setelah pembayaran
- `remaining_balance`: Sisa utang yang harus dibayar
- `collateral_released`: Apakah agunan sudah dikembalikan

**Logika**:
1. Verifikasi caller adalah peminjam
2. Validasi status pinjaman (harus Active)
3. Hitung breakdown pembayaran (pokok, bunga, fee protokol)
4. Proses transfer ckBTC via ICRC-1
5. Update riwayat pembayaran
6. Jika lunas, kembalikan NFT agunan
7. Kirim fee protokol ke treasury
8. Update liquidity pool

#### 2. get_loan_repayment_summary(loan_id: u64) -> Result<LoanRepaymentSummary, String>
**Tipe**: query  
**Deskripsi**: Mendapatkan ringkasan utang dan pembayaran pinjaman  
**Access**: Borrower atau Admin

**Output**: LoanRepaymentSummary dengan:
- `total_debt`: Total utang (pokok + bunga akumulasi)
- `principal_outstanding`: Sisa pokok yang harus dibayar
- `interest_outstanding`: Sisa bunga yang harus dibayar
- `total_repaid`: Total yang sudah dibayar
- `remaining_balance`: Sisa yang harus dibayar
- `is_overdue`: Status keterlambatan
- `days_overdue`: Jumlah hari terlambat

#### 3. get_repayment_plan(loan_id: u64) -> Result<RepaymentPlan, String>
**Tipe**: query  
**Deskripsi**: Mendapatkan rencana pembayaran pinjaman  
**Access**: Borrower only

**Output**: RepaymentPlan dengan:
- `total_amount_due`: Total yang harus dibayar
- `principal_amount`: Jumlah pokok
- `interest_amount`: Jumlah bunga
- `protocol_fee`: Fee protokol
- `due_date`: Tanggal jatuh tempo
- `minimum_payment`: Pembayaran minimum

#### 4. get_loan_payment_history(loan_id: u64) -> Result<Vec<Payment>, String>
**Tipe**: query  
**Deskripsi**: Mendapatkan riwayat pembayaran pinjaman  
**Access**: Borrower atau Admin

**Output**: Vector Payment dengan:
- `amount`: Jumlah pembayaran
- `timestamp`: Waktu pembayaran
- `payment_type`: Jenis pembayaran (Principal/Interest/Mixed)
- `transaction_id`: ID transaksi

### Administrative Functions

#### 5. get_loan_repayment_records(loan_id: u64) -> Result<Vec<RepaymentRecord>, String>
**Tipe**: query  
**Deskripsi**: Mendapatkan record pembayaran lengkap (admin only)  
**Access**: Admin only

#### 6. emergency_repayment(loan_id: u64, amount: u64, reason: String) -> Result<String, String>
**Tipe**: update  
**Deskripsi**: Pembayaran darurat (untuk kasus khusus)  
**Access**: Admin only

#### 7. get_repayment_statistics() -> Result<RepaymentStatistics, String>
**Tipe**: query  
**Deskripsi**: Mendapatkan statistik pembayaran system-wide  
**Access**: Admin only

### Utility Functions

#### 8. check_repayment_eligibility(loan_id: u64) -> Result<bool, String>
**Tipe**: query  
**Deskripsi**: Cek apakah pinjaman bisa dibayar

#### 9. calculate_early_repayment_benefits(loan_id: u64) -> Result<u64, String>
**Tipe**: query  
**Deskripsi**: Hitung benefit pembayaran dini

## Data Structures

### Payment
```rust
struct Payment {
    amount: u64,
    timestamp: u64,
    payment_type: PaymentType,
    transaction_id: Option<String>,
}

enum PaymentType {
    Principal,  // Pembayaran pokok
    Interest,   // Pembayaran bunga  
    Mixed,      // Campuran pokok dan bunga
}
```

### PaymentBreakdown
```rust
struct PaymentBreakdown {
    principal_amount: u64,
    interest_amount: u64,
    protocol_fee_amount: u64,
    total_amount: u64,
}
```

### LoanRepaymentSummary
```rust
struct LoanRepaymentSummary {
    loan_id: u64,
    borrower: Principal,
    total_debt: u64,
    principal_outstanding: u64,
    interest_outstanding: u64,
    total_repaid: u64,
    remaining_balance: u64,
    next_payment_due: Option<u64>,
    is_overdue: bool,
    days_overdue: u64,
}
```

### RepaymentResponse
```rust
struct RepaymentResponse {
    success: bool,
    message: String,
    transaction_id: Option<String>,
    new_loan_status: LoanStatus,
    remaining_balance: u64,
    collateral_released: bool,
}
```

## Security Features

### 1. Access Control
- **Borrower Access**: Hanya peminjam yang bisa membayar pinjamannya sendiri
- **Admin Access**: Admin dapat melihat semua data dan melakukan emergency repayment
- **Authorization**: Setiap function memverifikasi caller identity

### 2. Input Validation
- **Amount Validation**: Minimum payment amount (1000 satoshi)
- **Status Validation**: Pinjaman harus dalam status Active
- **Loan Ownership**: Verifikasi caller adalah borrower yang sah

### 3. Transaction Security
- **ckBTC Integration**: Menggunakan ICRC-1 standard untuk transfer
- **Atomic Operations**: Semua update state dilakukan secara atomic
- **Rollback Protection**: Jika transfer gagal, tidak ada perubahan state

## Interest Calculation

### Simple Interest Formula
```
Interest = Principal × (APR/100) × Time_in_years
Total_Debt = Principal + Interest
```

### Payment Allocation Priority
1. **Interest First**: Pembayaran dialokasikan ke bunga terlebih dahulu
2. **Principal Second**: Sisa pembayaran dialokasikan ke pokok
3. **Protocol Fee**: 10% dari bunga untuk protokol

## Collateral Release Process

### Automatic Release
1. Cek apakah `total_repaid >= total_debt`
2. Update status pinjaman menjadi `Repaid`
3. Panggil `unlock_nft()` untuk release NFT
4. Transfer NFT kembali ke borrower
5. Log audit trail

### Manual Release (Emergency)
- Admin dapat melakukan emergency repayment
- Untuk kasus pembayaran off-chain atau situasi khusus
- Memerlukan reason yang valid

## Error Handling

### Common Errors
- `"Loan not found"`: ID pinjaman tidak valid
- `"Unauthorized: Only the borrower can repay the loan"`: Akses ditolak
- `"Loan is not active for repayment"`: Status pinjaman salah
- `"Payment amount must be greater than zero"`: Input invalid
- `"Payment amount exceeds remaining debt"`: Overpayment
- `"Payment failed: [error]"`: Gagal transfer ckBTC

## Integration Points

### 1. ckBTC Ledger
- `process_ckbtc_repayment()`: Transfer ckBTC dari borrower
- ICRC-1 standard compliance

### 2. Liquidity Management
- `process_loan_repayment()`: Update pool state
- `collect_protocol_fees()`: Kirim fee ke treasury

### 3. RWA NFT Management
- `unlock_nft()`: Release collateral NFT
- ICRC-7 standard compliance

### 4. Audit Logging
- `log_audit_action()`: Catat semua aktivitas
- Compliance dan transparency

## Usage Examples

### Basic Repayment
```javascript
// Get repayment summary first
const summary = await canister.get_loan_repayment_summary(loan_id);
console.log("Remaining balance:", summary.remaining_balance);

// Make payment
const response = await canister.repay_loan(loan_id, payment_amount);
if (response.success) {
    console.log("Payment successful:", response.message);
    if (response.collateral_released) {
        console.log("Collateral NFT has been returned!");
    }
}
```

### Check Payment Plan
```javascript
const plan = await canister.get_repayment_plan(loan_id);
console.log("Total due:", plan.total_amount_due);
console.log("Principal:", plan.principal_amount);
console.log("Interest:", plan.interest_amount);
console.log("Protocol fee:", plan.protocol_fee);
```

### View Payment History
```javascript
const history = await canister.get_loan_payment_history(loan_id);
history.forEach(payment => {
    console.log(`Payment: ${payment.amount} at ${payment.timestamp}`);
});
```

## Testing Strategy

### Unit Tests
- Payment calculation logic
- Interest accumulation
- Payment breakdown allocation
- Data structure validation

### Integration Tests
- Full repayment flow
- Collateral release process
- ckBTC integration
- Protocol fee collection

### Edge Cases
- Overpayment handling
- Early repayment benefits
- Overdue loan scenarios
- Emergency repayment flows

## Performance Considerations

### Gas Optimization
- Efficient storage operations
- Minimal cross-canister calls
- Batch processing where possible

### Scalability
- Pagination for large datasets
- Efficient query patterns
- Memory management

## Compliance & Audit

### Audit Trail
- All repayment activities logged
- Immutable transaction records
- Compliance with regulatory requirements

### Transparency
- Public query functions for borrowers
- Clear fee structures
- Open source implementation

---

## Change Log

### Version 1.0
- Initial implementation
- Basic repayment functionality
- Collateral release automation
- Protocol fee collection

### Future Enhancements
- Partial payment scheduling
- Automated payment reminders
- Advanced interest calculations
- Multi-currency support
