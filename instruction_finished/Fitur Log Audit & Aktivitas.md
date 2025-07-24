# README: Fitur Log Audit & Aktivitas Lengkap

**Modul:** `audit_logging`  
**Canister Terkait:** Semua canister dalam sistem Agrilends  
**Status:** ✅ Implemented & Enhanced

## 1. Tujuan Fitur

Fitur ini menyediakan sistem audit trail yang komprehensif dan tidak dapat diubah (immutable) untuk semua transaksi dan peristiwa penting yang terjadi di dalam protokol Agrilends. Sistem ini dirancang untuk:

- **Transparansi**: Memberikan visibilitas penuh terhadap semua operasi sistem
- **Keamanan**: Mendeteksi dan mencatat aktivitas mencurigakan atau tidak sah
- **Compliance**: Memenuhi persyaratan regulasi dan audit eksternal
- **Debugging**: Membantu dalam troubleshooting dan analisis performa sistem
- **Akuntabilitas**: Melacak tindakan semua pengguna dan administrator

## 2. Arsitektur Sistem

### 2.1 Struktur Data Enhanced

```rust
// Enhanced audit log dengan metadata lengkap
pub struct EnhancedAuditLog {
    pub id: u64,                          // ID unik log
    pub timestamp: u64,                   // Timestamp nanosecond
    pub block_height: Option<u64>,        // Block height IC (untuk verifikasi)
    pub caller: Principal,                // Principal yang melakukan aksi
    pub category: AuditCategory,          // Kategori operasi
    pub action: String,                   // Nama aksi spesifik
    pub level: AuditEventLevel,           // Tingkat kepentingan
    pub details: AuditDetails,            // Detail lengkap operasi
    pub result: AuditResult,              // Hasil operasi
    pub correlation_id: Option<String>,   // ID untuk tracking operasi terkait
    pub session_id: Option<String>,       // ID sesi pengguna
    pub ip_hash: Option<String>,          // Hash IP (privacy-compliant)
}

// Kategori audit berdasarkan modul sistem
pub enum AuditCategory {
    UserManagement,     // Manajemen pengguna
    NFTOperations,      // Operasi RWA-NFT
    LoanLifecycle,      // Siklus hidup pinjaman
    Liquidation,        // Proses likuidasi
    Governance,         // Tata kelola dan voting
    Treasury,           // Manajemen treasury
    Oracle,             // Oracle dan pricing
    Security,           // Keamanan sistem
    Configuration,      // Perubahan konfigurasi
    Maintenance,        // Pemeliharaan sistem
    Integration,        // Integrasi eksternal
}

// Tingkat kepentingan event
pub enum AuditEventLevel {
    Info,      // Informasi umum
    Warning,   // Peringatan
    Error,     // Error
    Critical,  // Kritis (keamanan)
    Success,   // Operasi berhasil
}
```

### 2.2 Storage Strategy

```rust
// Memory allocation untuk audit logs
thread_local! {
    static ENHANCED_AUDIT_LOGS: RefCell<StableBTreeMap<u64, EnhancedAuditLog, Memory>> = 
        RefCell::new(StableBTreeMap::init(get_memory_by_id(MemoryId::new(100))));
    
    static AUDIT_CONFIG: RefCell<StableBTreeMap<u8, AuditConfiguration, Memory>> = 
        RefCell::new(StableBTreeMap::init(get_memory_by_id(MemoryId::new(101))));
    
    static SESSION_TRACKER: RefCell<HashMap<Principal, String>> = 
        RefCell::new(HashMap::new());
    
    static CORRELATION_TRACKER: RefCell<HashMap<String, Vec<u64>>> = 
        RefCell::new(HashMap::new());
}
```

## 3. Implementasi Fungsi Utama

### 3.1 Core Logging Functions

#### a. log_audit_enhanced()
```rust
pub fn log_audit_enhanced(
    category: AuditCategory,
    action: String,
    level: AuditEventLevel,
    details: AuditDetails,
    result: AuditResult,
    correlation_id: Option<String>,
)
```
**Deskripsi**: Fungsi utama untuk logging dengan metadata lengkap  
**Penggunaan**: Semua operasi penting sistem

#### b. log_audit_action() - Backward Compatible
```rust
pub fn log_audit_action(caller: Principal, action: String, details: String, success: bool)
```
**Deskripsi**: Fungsi simplified untuk kompatibilitas dengan kode existing  
**Penggunaan**: Migrasi bertahap dari sistem lama

### 3.2 Specialized Logging Functions

#### a. NFT Operations
```rust
pub fn log_nft_operation(
    action: &str,
    token_id: u64,
    owner: Principal,
    success: bool,
    error_msg: Option<String>,
)
```
**Contoh Penggunaan**:
```rust
// Di rwa_nft.rs - saat minting NFT
log_nft_operation("MINT_NFT", token_id, owner, true, None);

// Di rwa_nft.rs - saat transfer gagal
log_nft_operation("TRANSFER_NFT", token_id, owner, false, Some("Insufficient authorization".to_string()));
```

#### b. Loan Operations
```rust
pub fn log_loan_operation(
    action: &str,
    loan_id: u64,
    borrower: Principal,
    amount: Option<u64>,
    success: bool,
    error_msg: Option<String>,
)
```
**Contoh Penggunaan**:
```rust
// Di loan_lifecycle.rs - saat approval pinjaman
log_loan_operation("APPROVE_LOAN", loan.id, loan.borrower, Some(loan.amount_approved), true, None);

// Di loan_repayment.rs - saat pembayaran
log_loan_operation("REPAY_LOAN", loan_id, borrower, Some(payment_amount), true, None);
```

#### c. Security Events
```rust
pub fn log_security_event(
    event_type: &str,
    severity: AuditEventLevel,
    description: String,
    affected_principal: Option<Principal>,
)
```
**Contoh Penggunaan**:
```rust
// Di production_security.rs - saat blacklist
log_security_event(
    "PRINCIPAL_BLACKLISTED",
    AuditEventLevel::Critical,
    format!("Principal {} blacklisted after {} failed attempts", principal.to_text(), attempts),
    Some(principal)
);
```

### 3.3 Query Functions

#### a. Advanced Filtering
```rust
#[query]
pub fn get_audit_logs_filtered(filter: AuditLogFilter) -> Result<Vec<EnhancedAuditLog>, String>
```

**Filter Parameters**:
```rust
pub struct AuditLogFilter {
    pub start_time: Option<u64>,           // Filter berdasarkan waktu mulai
    pub end_time: Option<u64>,             // Filter berdasarkan waktu akhir
    pub caller: Option<Principal>,         // Filter berdasarkan principal
    pub category: Option<AuditCategory>,   // Filter berdasarkan kategori
    pub level: Option<AuditEventLevel>,    // Filter berdasarkan level
    pub action_pattern: Option<String>,    // Filter berdasarkan pattern action
    pub success_only: Option<bool>,        // Hanya tampilkan yang berhasil
    pub entity_type: Option<String>,       // Filter berdasarkan tipe entitas
    pub entity_id: Option<String>,         // Filter berdasarkan ID entitas
    pub limit: Option<u64>,                // Batasi jumlah hasil
    pub offset: Option<u64>,               // Offset untuk pagination
}
```

#### b. Statistics & Analytics
```rust
#[query]
pub fn get_audit_statistics() -> Result<AuditStatistics, String>
```

**Output Statistics**:
```rust
pub struct AuditStatistics {
    pub total_logs: u64,                              // Total log entries
    pub logs_by_category: HashMap<String, u64>,       // Distribusi per kategori
    pub logs_by_level: HashMap<String, u64>,          // Distribusi per level
    pub success_rate: f64,                            // Persentase keberhasilan
    pub most_active_callers: Vec<(Principal, u64)>,   // Pengguna paling aktif
    pub recent_critical_events: u64,                  // Event kritis 24 jam terakhir
    pub storage_usage_bytes: u64,                     // Penggunaan storage
    pub oldest_log_timestamp: Option<u64>,            // Log tertua
    pub newest_log_timestamp: Option<u64>,            // Log terbaru
}
```

#### c. Correlation Tracking
```rust
#[query]
pub fn get_logs_by_correlation(correlation_id: String) -> Result<Vec<EnhancedAuditLog>, String>
```
**Penggunaan**: Melacak semua operasi yang terkait dalam satu transaksi kompleks

## 4. Integrasi dengan Modul Sistem

### 4.1 User Management
```rust
// Di user_management.rs
use crate::helpers::log_audit_action;

pub fn register_user(user_data: User) -> UserResult {
    // ... logic registrasi ...
    
    if success {
        log_audit_action(
            caller(), 
            "REGISTER_USER".to_string(), 
            format!("Role: {:?}, Principal: {}", user_data.role, caller().to_text()),
            true
        );
    }
}
```

### 4.2 Loan Lifecycle
```rust
// Di loan_lifecycle.rs
use crate::helpers::{log_loan_audit, log_security_audit};

pub fn submit_loan_application(nft_id: u64, amount_requested: u64) -> Result<Loan, String> {
    let start_time = time();
    
    // ... logic aplikasi pinjaman ...
    
    match result {
        Ok(loan) => {
            log_loan_audit(
                "SUBMIT_APPLICATION",
                loan.id,
                loan.borrower,
                Some(loan.amount_requested),
                true,
                None
            );
        },
        Err(error) => {
            log_loan_audit(
                "SUBMIT_APPLICATION",
                0,
                caller(),
                Some(amount_requested),
                false,
                Some(error.clone())
            );
        }
    }
}
```

### 4.3 Liquidation Process
```rust
// Di liquidation.rs
use crate::helpers::log_liquidation_audit;

pub fn trigger_liquidation(loan_id: u64) -> Result<String, String> {
    // ... logic likuidasi ...
    
    log_liquidation_audit(
        "TRIGGER_LIQUIDATION",
        loan_id,
        loan.borrower,
        collateral_value,
        outstanding_debt,
        success
    );
}
```

### 4.4 Governance Operations
```rust
// Di governance.rs
use crate::helpers::log_governance_audit;

pub fn create_proposal(proposal: Proposal) -> Result<u64, String> {
    // ... logic pembuatan proposal ...
    
    log_governance_audit(
        "CREATE_PROPOSAL",
        Some(proposal.id),
        true,
        format!("Type: {:?}, Title: {}", proposal.proposal_type, proposal.title)
    );
}
```

## 5. Konfigurasi Sistem

### 5.1 Audit Configuration
```rust
pub struct AuditConfiguration {
    pub enabled: bool,                      // Enable/disable audit logging
    pub max_logs_per_category: u64,         // Max logs per kategori (10,000)
    pub auto_cleanup_enabled: bool,         // Auto cleanup logs lama
    pub cleanup_threshold_days: u64,        // Threshold cleanup (365 hari)
    pub critical_event_notification: bool,  // Notifikasi untuk event kritis
    pub detailed_logging: bool,             // Logging detail lengkap
    pub performance_tracking: bool,         // Track performa operasi
    pub anonymization_enabled: bool,        // Anonimisasi data sensitif
}
```

### 5.2 Default Settings
```rust
impl Default for AuditConfiguration {
    fn default() -> Self {
        Self {
            enabled: true,
            max_logs_per_category: 10000,
            auto_cleanup_enabled: true,
            cleanup_threshold_days: 365,
            critical_event_notification: true,
            detailed_logging: true,
            performance_tracking: true,
            anonymization_enabled: false,
        }
    }
}
```

## 6. Administrasi dan Maintenance

### 6.1 Update Configuration
```rust
#[update]
pub fn update_audit_config(config: AuditConfiguration) -> Result<(), String>
```
**Akses**: Admin only  
**Fungsi**: Update konfigurasi audit logging

### 6.2 Manual Cleanup
```rust
#[update]
pub fn cleanup_old_audit_logs(days_to_keep: u64) -> Result<u64, String>
```
**Akses**: Admin only  
**Fungsi**: Cleanup manual logs lama  
**Return**: Jumlah logs yang dihapus

### 6.3 Compliance Export
```rust
#[query]
pub fn export_audit_logs_for_compliance(start_time: u64, end_time: u64) -> Result<Vec<EnhancedAuditLog>, String>
```
**Akses**: Admin only  
**Fungsi**: Export logs untuk keperluan compliance audit

## 7. Use Cases dan Contoh

### 7.1 Monitoring User Activity
```rust
// Query semua aktivitas user dalam 24 jam terakhir
let filter = AuditLogFilter {
    start_time: Some(time() - 24 * 60 * 60 * 1_000_000_000),
    caller: Some(user_principal),
    ..Default::default()
};
let user_activity = get_audit_logs_filtered(filter)?;
```

### 7.2 Security Monitoring
```rust
// Query semua event security kritis
let filter = AuditLogFilter {
    category: Some(AuditCategory::Security),
    level: Some(AuditEventLevel::Critical),
    ..Default::default()
};
let security_events = get_audit_logs_filtered(filter)?;
```

### 7.3 Loan Tracking
```rust
// Track semua operasi untuk loan tertentu
let filter = AuditLogFilter {
    entity_type: Some("loan".to_string()),
    entity_id: Some(loan_id.to_string()),
    ..Default::default()
};
let loan_history = get_audit_logs_filtered(filter)?;
```

### 7.4 Performance Analysis
```rust
// Analisis performa sistem berdasarkan statistik
let stats = get_audit_statistics()?;
println!("Success rate: {}%", stats.success_rate);
println!("Critical events today: {}", stats.recent_critical_events);
```

## 8. Event Categories dan Actions

### 8.1 User Management Events
- `REGISTER_USER` - Registrasi pengguna baru
- `UPDATE_USER_ROLE` - Update role pengguna
- `DEACTIVATE_USER` - Deaktivasi pengguna
- `LOGIN_ATTEMPT` - Percobaan login
- `PERMISSION_DENIED` - Akses ditolak

### 8.2 NFT Operations Events
- `MINT_NFT` - Mint RWA-NFT baru
- `TRANSFER_NFT` - Transfer NFT
- `LOCK_NFT` - Lock NFT untuk agunan
- `UNLOCK_NFT` - Unlock NFT setelah pelunasan
- `UPDATE_NFT_METADATA` - Update metadata NFT

### 8.3 Loan Lifecycle Events
- `SUBMIT_APPLICATION` - Submit aplikasi pinjaman
- `APPROVE_LOAN` - Approval pinjaman
- `REJECT_LOAN` - Reject pinjaman
- `DISBURSE_FUNDS` - Pencairan dana
- `REPAY_LOAN` - Pembayaran pinjaman
- `COMPLETE_LOAN` - Pelunasan pinjaman

### 8.4 Liquidation Events
- `TRIGGER_LIQUIDATION` - Trigger proses likuidasi
- `LIQUIDATION_COMPLETED` - Likuidasi selesai
- `COLLATERAL_SEIZED` - Penyitaan agunan
- `LIQUIDATION_FAILED` - Likuidasi gagal

### 8.5 Security Events
- `UNAUTHORIZED_ACCESS` - Akses tidak sah
- `BLACKLIST_PRINCIPAL` - Blacklist principal
- `RATE_LIMIT_EXCEEDED` - Rate limit terlampaui
- `SUSPICIOUS_ACTIVITY` - Aktivitas mencurigakan
- `ADMIN_ACTION` - Aksi administrator

## 9. Best Practices

### 9.1 Development Guidelines
1. **Consistent Logging**: Selalu gunakan kategori dan action yang konsisten
2. **Meaningful Details**: Berikan context yang cukup dalam audit details
3. **Error Handling**: Log semua error dengan level yang tepat
4. **Performance**: Gunakan async logging untuk operasi yang tidak kritis
5. **Privacy**: Hindari logging data sensitif secara langsung

### 9.2 Operational Guidelines
1. **Regular Monitoring**: Monitor audit logs secara berkala
2. **Alert Setup**: Setup alert untuk critical events
3. **Regular Cleanup**: Jalankan cleanup rutin untuk mengoptimalkan storage
4. **Backup**: Backup audit logs untuk compliance jangka panjang
5. **Access Control**: Batasi akses audit logs hanya untuk admin

## 10. Security Considerations

### 10.1 Data Protection
- Hash IP addresses untuk privacy compliance
- Anonimisasi data sensitif jika diperlukan
- Enkripsi log data saat export untuk compliance

### 10.2 Access Control
- Hanya admin yang dapat mengakses audit logs
- Role-based access untuk berbagai level audit data
- Audit access ke audit logs itu sendiri

### 10.3 Tampering Prevention
- Immutable storage menggunakan IC stable structures
- Correlation tracking untuk verifikasi integritas
- Block height recording untuk verifikasi temporal

## 11. Future Enhancements

### 11.1 Planned Features
- Real-time alerting untuk critical events
- Machine learning untuk anomaly detection
- Integration dengan external SIEM systems
- Advanced analytics dan reporting
- Automated compliance reporting

### 11.2 Scalability Improvements
- Partitioned storage untuk handling volume tinggi
- Compressed storage untuk efficiency
- Distributed logging across multiple canisters
- Stream processing untuk real-time analytics

---

**Status Implementasi**: ✅ Complete  
**Last Updated**: July 23, 2025  
**Version**: 1.0.0
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
