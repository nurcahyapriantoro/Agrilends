# Loan Repayment API Documentation

## Overview
Implementasi lengkap fitur pelunasan pinjaman dan penarikan agunan sesuai dengan spesifikasi README. Fitur ini memungkinkan peminjam melakukan pembayaran kembali pinjaman mereka dalam bentuk ckBTC dan secara otomatis menerima kembali agunan RWA-NFT setelah lunas.

## Core Features

### 1. Main Repayment Function

#### `repay_loan(loan_id: u64, amount: u64) -> Result<RepaymentResponse, String>`
**Tipe**: `update`  
**Deskripsi**: Memproses pembayaran kembali dari peminjam dengan validasi komprehensif.

**Logika & Keamanan**:
1. Validasi input dan verifikasi caller adalah peminjam
2. Cek status pinjaman (harus Active)
3. Panggilan Antar-Canister untuk transfer ckBTC (`icrc1_transfer_from`)
4. Update loan dengan payment information
5. Hitung dan alokasi payment (penalty → interest → principal)
6. Collect protocol fees (10% dari porsi bunga)
7. Jika lunas: ubah status ke Repaid dan release NFT collateral
8. Log audit dan return response

**Input**:
- `loan_id: u64` - ID pinjaman yang akan dibayar
- `amount: u64` - Jumlah ckBTC dalam satoshi

**Output**: `RepaymentResponse` dengan detail pembayaran

### 2. Loan Information Functions

#### `get_loan_repayment_summary(loan_id: u64) -> Result<LoanRepaymentSummary, String>`
**Tipe**: `query`  
**Deskripsi**: Mendapatkan ringkasan lengkap status pembayaran pinjaman.

#### `get_repayment_plan(loan_id: u64) -> Result<RepaymentPlan, String>`
**Tipe**: `query`  
**Deskripsi**: Mendapatkan rencana pembayaran dengan breakdown amount due.

#### `get_loan_payment_history(loan_id: u64) -> Result<Vec<Payment>, String>`
**Tipe**: `query`  
**Deskripsi**: Mendapatkan riwayat pembayaran untuk pinjaman tertentu.

### 3. Administrative Functions

#### `get_comprehensive_repayment_analytics() -> Result<ComprehensiveRepaymentAnalytics, String>`
**Tipe**: `query` (Admin only)  
**Deskripsi**: Mendapatkan analytics komprehensif untuk dashboard admin.

#### `process_batch_repayments(requests: Vec<BatchRepaymentRequest>) -> Result<Vec<BatchRepaymentResult>, String>`
**Tipe**: `update` (Admin only)  
**Deskripsi**: Memproses multiple repayments secara batch untuk efisiensi.

#### `emergency_repayment(loan_id: u64, amount: u64, reason: String) -> Result<String, String>`
**Tipe**: `update` (Admin only)  
**Deskripsi**: Proses pembayaran darurat tanpa transfer ckBTC (untuk pembayaran off-chain manual).

### 4. Advanced Features

#### `calculate_early_repayment_benefits(loan_id: u64) -> Result<u64, String>`
**Tipe**: `query`  
**Deskripsi**: Menghitung diskon early repayment (5% pada bunga jika < 80% loan term).

#### `schedule_automatic_repayment(loan_id: u64, amount: u64, frequency_days: u64) -> Result<String, String>`
**Tipe**: `update`  
**Deskripsi**: Menjadwalkan pembayaran otomatis recurring.

#### `get_repayment_forecast(loan_id: u64, months_ahead: u64) -> Result<Vec<RepaymentForecast>, String>`
**Tipe**: `query`  
**Deskripsi**: Mendapatkan proyeksi pembayaran untuk perencanaan finansial.

## Data Structures

### Payment
```rust
struct Payment {
    amount: u64,
    timestamp: u64,
    payment_type: PaymentType, // Principal, Interest, Mixed
    transaction_id: Option<String>,
}
```

### PaymentBreakdown
```rust
struct PaymentBreakdown {
    principal_amount: u64,
    interest_amount: u64,
    protocol_fee_amount: u64,
    penalty_amount: u64,
    total_amount: u64,
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

## Production Features

### Security & Validation
- **Daily Limits**: Maximum 10 BTC per day per user
- **Minimum Payment**: 1000 satoshi minimum
- **Overpayment Tolerance**: 100 satoshi tolerance
- **Rate Limiting**: Protection against spam transactions

### Interest Calculation
- **Simple Interest**: Principal × Rate × Time
- **Late Penalty**: 2% per month for overdue loans (capped at 10% of principal)
- **Protocol Fee**: 10% of interest portion goes to protocol treasury

### Audit & Compliance
- Comprehensive audit logging untuk semua operasi
- Transaction tracking dengan block index
- Payment history untuk transparency
- Automated compliance checks

### Inter-Canister Integrations
1. **ckBTC Ledger**: Untuk transfer pembayaran
2. **RWA-NFT Canister**: Untuk release collateral
3. **Treasury Canister**: Untuk protocol fee collection
4. **Liquidity Pool**: Untuk update liquidity state

## Error Handling

### Common Errors
- `"Loan not found"` - Invalid loan ID
- `"Unauthorized: Only the borrower can repay the loan"` - Access control
- `"Loan is not active for repayment"` - Invalid loan status
- `"Payment amount must be at least X satoshi"` - Below minimum
- `"Payment amount exceeds daily limit"` - Above daily limit
- `"Loan is already fully repaid"` - Already completed

### Production Error Recovery
- Automatic retry mechanism untuk inter-canister calls
- Rollback mechanism jika collateral release gagal
- Comprehensive error logging untuk troubleshooting

## Usage Examples

### Basic Repayment
```typescript
// Make a loan repayment
const result = await repay_loan(1, 10_000_000); // Loan ID 1, 10M satoshi
if (result.success) {
    console.log(`Payment successful: ${result.message}`);
    if (result.collateral_released) {
        console.log("Collateral NFT has been released!");
    }
}
```

### Check Repayment Plan
```typescript
// Get repayment plan
const plan = await get_repayment_plan(1);
console.log(`Total due: ${plan.total_amount_due} satoshi`);
console.log(`Principal: ${plan.principal_amount}, Interest: ${plan.interest_amount}`);
```

### Admin Analytics
```typescript
// Get comprehensive analytics (admin only)
const analytics = await get_comprehensive_repayment_analytics();
console.log(`Total loans: ${analytics.total_loans_count}`);
console.log(`Repayment rate: ${analytics.repaid_loans_count / analytics.total_loans_count * 100}%`);
```

## Testing

### Unit Tests
- Payment calculation logic
- Interest accrual calculation
- Early repayment discount calculation
- Data structure validation

### Integration Tests
- ckBTC transfer simulation
- NFT release workflow
- Protocol fee collection
- Audit logging verification

### Production Testing
- Load testing dengan multiple concurrent repayments
- Failure recovery testing
- Inter-canister communication testing
- Security penetration testing

## Monitoring & Metrics

### Key Metrics
- Total repayment volume
- Average repayment time
- Early repayment rate
- Default rate
- Protocol fee collection
- System performance metrics

### Alerts
- Overdue loan notifications
- Failed repayment alerts
- System health monitoring
- Unusual transaction pattern detection

## Deployment Notes

### Prerequisites
- ckBTC ledger integration setup
- RWA-NFT canister deployed
- Treasury/liquidity management canister ready
- Admin principals configured

### Configuration
```rust
const PROTOCOL_FEE_PERCENTAGE: u64 = 10; // 10%
const MINIMUM_PAYMENT_AMOUNT: u64 = 1000; // 1000 satoshi
const MAX_DAILY_REPAYMENT_LIMIT: u64 = 1_000_000_000; // 10 BTC
const LATE_PAYMENT_PENALTY_RATE: u64 = 2; // 2% per month
const EARLY_REPAYMENT_DISCOUNT_RATE: u64 = 5; // 5%
```

### Maintenance
- Regular audit log cleanup
- Performance monitoring
- Database optimization
- Security updates

Implementasi ini memberikan fitur pelunasan pinjaman yang lengkap, aman, dan production-ready sesuai dengan spesifikasi dalam README.
