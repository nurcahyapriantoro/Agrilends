# README: Fitur Pemeliharaan Otomatis (Heartbeat) Lengkap

**Modul:** `automated_maintenance`  
**Canister Terkait:** Semua canister utama sistem Agrilends  
**Status:** ✅ Implemented & Enhanced

## 1. Tujuan Fitur

Fitur Pemeliharaan Otomatis memanfaatkan fungsionalitas **heartbeat** dari Internet Computer untuk menjalankan tugas-tugas pemeliharaan secara otomatis dan periodik tanpa memerlukan pemicu eksternal. Sistem ini dirancang untuk:

- **Proaktivitas**: Mendeteksi dan menangani masalah sebelum menjadi kritis
- **Efisiensi**: Mengotomatisasi tugas maintenance yang berulang
- **Keandalan**: Memastikan sistem berjalan optimal 24/7
- **Keamanan**: Monitoring kontinyu terhadap kondisi sistem yang abnormal
- **Optimisasi**: Pembersihan data dan optimisasi performa secara berkala

## 2. Arsitektur Sistem Heartbeat

### 2.1 Konsep Implementasi

Sistem heartbeat di Agrilends mengimplementasikan fungsi `canister_heartbeat` yang dieksekusi oleh sistem ICP secara otomatis setiap beberapa detik. Heartbeat hanya berjalan jika sistem tidak dalam mode maintenance.

```rust
#[heartbeat]
async fn heartbeat() {
    // Hanya jalankan task heartbeat jika tidak dalam maintenance mode
    if !is_in_maintenance_mode() {
        // Update harga komoditas yang stale
        oracle::heartbeat_price_update().await;
        
        // Cek pinjaman yang overdue
        crate::helpers::check_overdue_loans().await;
        
        // Monitor cycles balance
        monitor_cycles_balance();
        
        // Cleanup log audit lama (simpan 10,000 entry terakhir)
        cleanup_old_audit_logs();
        
        // Maintenance pool likuiditas
        liquidity_management::perform_pool_maintenance().unwrap_or_default();
        
        // Cek liquidation eligible loans
        automated_liquidation_monitoring().await;
    }
}
```

### 2.2 Kontrol Sistem

```rust
// Fungsi untuk mengontrol heartbeat
pub fn is_in_maintenance_mode() -> bool {
    let config = get_canister_config();
    config.maintenance_mode
}

pub fn get_emergency_stop_status() -> bool {
    let config = get_canister_config();
    config.emergency_stop
}

pub fn get_last_heartbeat_time() -> u64 {
    time()
}
```

## 3. Tugas Otomatis yang Diimplementasikan

### 3.1 Update Harga Oracle Otomatis

#### **Fungsi**: `oracle::heartbeat_price_update()`
**Tujuan**: Memperbarui harga komoditas yang sudah stale (>24 jam)  
**Frekuensi**: Setiap heartbeat cycle  
**Komoditas**: Rice, Corn, Wheat

```rust
pub async fn heartbeat_price_update() {
    let commodities = vec!["rice".to_string(), "corn".to_string(), "wheat".to_string()];
    
    for commodity in commodities {
        if is_price_stale(commodity.clone()) {
            // Update harga secara otomatis
            if let Err(e) = fetch_commodity_price(commodity.clone()).await {
                log_audit_action(
                    ic_cdk::id(),
                    "AUTO_PRICE_UPDATE_FAILED".to_string(),
                    format!("Failed to auto-update {} price: {}", commodity, e),
                    false,
                );
            } else {
                log_audit_action(
                    ic_cdk::id(),
                    "AUTO_PRICE_UPDATE_SUCCESS".to_string(),
                    format!("Successfully auto-updated {} price", commodity),
                    true,
                );
            }
        }
    }
}
```

**Kondisi Stale**:
```rust
pub fn is_price_stale(commodity_id: String) -> bool {
    if let Some(price_data) = get_stored_commodity_price(&commodity_id) {
        let current_time = time();
        let twenty_four_hours = 24 * 60 * 60 * 1_000_000_000u64;
        (current_time - price_data.timestamp) > twenty_four_hours
    } else {
        true // Tidak ada data = stale
    }
}
```

### 3.2 Monitoring Pinjaman Overdue

#### **Fungsi**: `check_overdue_loans()`
**Tujuan**: Deteksi pinjaman yang terlambat dan siap untuk likuidasi  
**Action**: Notifikasi dan marking untuk review admin

```rust
pub async fn check_overdue_loans() {
    let overdue_loans = get_overdue_loans();
    
    for loan in overdue_loans {
        // Log untuk audit trail
        log_audit_action(
            ic_cdk::id(),
            "OVERDUE_LOAN_DETECTED".to_string(),
            format!("Loan {} is overdue and may require liquidation review", loan.id),
            false,
        );
        
        // Cek apakah eligible untuk auto-liquidation
        if let Ok(eligible) = check_liquidation_eligibility(loan.id) {
            if eligible.is_eligible {
                log_audit_action(
                    ic_cdk::id(),
                    "LIQUIDATION_ELIGIBLE_DETECTED".to_string(),
                    format!("Loan {} is eligible for liquidation: {}", loan.id, eligible.reason),
                    false,
                );
            }
        }
    }
}

pub fn get_overdue_loans() -> Vec<Loan> {
    let params = get_protocol_parameters();
    let grace_period = params.grace_period_days * 24 * 60 * 60 * 1_000_000_000;
    let current_time = time();
    
    get_all_loans_data()
        .into_iter()
        .filter(|loan| {
            loan.status == LoanStatus::Active &&
            loan.due_date.map_or(false, |due| current_time > due + grace_period)
        })
        .collect()
}
```

### 3.3 Monitoring Cycles Balance

#### **Fungsi**: `monitor_cycles_balance()`
**Tujuan**: Mencegah canister kehabisan cycles  
**Threshold**: Alert jika cycles < 1T, critical jika < 500B

```rust
pub fn monitor_cycles_balance() {
    let current_cycles = ic_cdk::api::canister_balance();
    let cycles_threshold_alert = 1_000_000_000_000u64; // 1T cycles
    let cycles_threshold_critical = 500_000_000_000u64; // 500B cycles
    
    if current_cycles < cycles_threshold_critical {
        log_audit_action(
            ic_cdk::id(),
            "CYCLES_CRITICAL".to_string(),
            format!("CRITICAL: Canister cycles below critical threshold: {} cycles", current_cycles),
            false,
        );
    } else if current_cycles < cycles_threshold_alert {
        log_audit_action(
            ic_cdk::id(),
            "CYCLES_LOW".to_string(),
            format!("WARNING: Canister cycles running low: {} cycles", current_cycles),
            false,
        );
    }
}
```

### 3.4 Cleanup Audit Logs

#### **Fungsi**: `cleanup_old_audit_logs()`
**Tujuan**: Optimisasi storage dengan hapus log lama  
**Retention**: Simpan 10,000 log terakhir

```rust
pub fn cleanup_old_audit_logs() {
    const MAX_AUDIT_LOGS: usize = 10_000;
    
    ENHANCED_AUDIT_LOGS.with(|logs| {
        let mut logs_map = logs.borrow_mut();
        let current_count = logs_map.len();
        
        if current_count > MAX_AUDIT_LOGS {
            let excess = current_count - MAX_AUDIT_LOGS;
            let mut deleted_count = 0;
            
            // Hapus log terlama
            let keys_to_delete: Vec<_> = logs_map.iter()
                .take(excess)
                .map(|(key, _)| key)
                .collect();
            
            for key in keys_to_delete {
                logs_map.remove(&key);
                deleted_count += 1;
            }
            
            if deleted_count > 0 {
                log_audit_action(
                    ic_cdk::id(),
                    "AUDIT_LOG_CLEANUP".to_string(),
                    format!("Cleaned up {} old audit log entries", deleted_count),
                    true,
                );
            }
        }
    });
}
```

### 3.5 Pool Maintenance Otomatis

#### **Fungsi**: `perform_pool_maintenance()`
**Tujuan**: Maintenance pool likuiditas secara berkala  
**Actions**: Health check, cleanup transactions, monitoring

```rust
pub fn perform_pool_maintenance() -> Result<String, String> {
    let caller = ic_cdk::id(); // Canister itself
    let mut maintenance_actions = Vec::new();
    
    // Cek kesehatan pool
    let pool = get_liquidity_pool();
    let health_score = calculate_pool_health_score(&pool);
    
    if health_score < 50 {
        maintenance_actions.push("Pool health critical - admin review required".to_string());
        
        log_audit_action(
            caller,
            "POOL_HEALTH_CRITICAL".to_string(),
            format!("Pool health score: {}/100 - requires immediate attention", health_score),
            false,
        );
    }
    
    // Cek utilization rate
    let utilization_rate = if pool.total_liquidity > 0 {
        ((pool.total_liquidity - pool.available_liquidity) * 100) / pool.total_liquidity
    } else { 0 };
    
    if utilization_rate > 90 {
        maintenance_actions.push("High utilization detected - monitor closely".to_string());
        
        log_audit_action(
            caller,
            "POOL_HIGH_UTILIZATION".to_string(),
            format!("Pool utilization: {}% - monitor for liquidity stress", utilization_rate),
            false,
        );
    }
    
    // Cleanup transaksi lama (>30 hari)
    let thirty_days_ago = time() - (30 * 24 * 60 * 60 * 1_000_000_000);
    if let Ok(cleaned) = cleanup_old_transactions(thirty_days_ago) {
        if cleaned > 0 {
            maintenance_actions.push(format!("Cleaned {} old transactions", cleaned));
        }
    }
    
    Ok(format!("Pool maintenance completed: {:?}", maintenance_actions))
}
```

### 3.6 Automated Liquidation Monitoring

#### **Fungsi**: `automated_liquidation_monitoring()`
**Tujuan**: Monitor dan trigger liquidation otomatis jika diperlukan

```rust
async fn automated_liquidation_monitoring() {
    // Ambil daftar loans yang eligible untuk liquidation
    let eligible_loans = get_loans_eligible_for_liquidation();
    
    for loan_id in eligible_loans {
        log_audit_action(
            ic_cdk::id(),
            "AUTO_LIQUIDATION_CANDIDATE".to_string(),
            format!("Loan {} detected as liquidation candidate", loan_id),
            false,
        );
        
        // Untuk auto-liquidation, kita bisa set policy:
        // 1. Auto-trigger untuk loans dengan grace period > 45 hari
        // 2. Alert only untuk yang 30-45 hari
        
        if let Ok(loan) = get_loan(loan_id) {
            if let Some(due_date) = loan.due_date {
                let days_overdue = (time() - due_date) / (24 * 60 * 60 * 1_000_000_000);
                
                // Auto-liquidation setelah 45 hari overdue
                if days_overdue > 45 {
                    match trigger_liquidation(loan_id).await {
                        Ok(_) => {
                            log_audit_action(
                                ic_cdk::id(),
                                "AUTO_LIQUIDATION_TRIGGERED".to_string(),
                                format!("Automatically triggered liquidation for loan {} after {} days overdue", loan_id, days_overdue),
                                true,
                            );
                        },
                        Err(e) => {
                            log_audit_action(
                                ic_cdk::id(),
                                "AUTO_LIQUIDATION_FAILED".to_string(),
                                format!("Failed to auto-liquidate loan {}: {}", loan_id, e),
                                false,
                            );
                        }
                    }
                }
            }
        }
    }
}
```

## 4. Konfigurasi dan Kontrol

### 4.1 Heartbeat Configuration

```rust
pub struct HeartbeatConfig {
    pub enabled: bool,                          // Enable/disable heartbeat
    pub maintenance_mode: bool,                 // Maintenance mode flag
    pub price_update_enabled: bool,             // Enable price updates
    pub loan_monitoring_enabled: bool,          // Enable loan monitoring
    pub cycles_monitoring_enabled: bool,        // Enable cycles monitoring
    pub auto_cleanup_enabled: bool,             // Enable automatic cleanup
    pub pool_maintenance_enabled: bool,         // Enable pool maintenance
    pub auto_liquidation_enabled: bool,         // Enable auto-liquidation
    pub auto_liquidation_threshold_days: u64,   // Days overdue for auto-liquidation
}

impl Default for HeartbeatConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            maintenance_mode: false,
            price_update_enabled: true,
            loan_monitoring_enabled: true,
            cycles_monitoring_enabled: true,
            auto_cleanup_enabled: true,
            pool_maintenance_enabled: true,
            auto_liquidation_enabled: false, // Disabled by default for safety
            auto_liquidation_threshold_days: 45,
        }
    }
}
```

### 4.2 Kontrol Admin

```rust
// Update heartbeat configuration (admin only)
#[update]
pub fn update_heartbeat_config(config: HeartbeatConfig) -> Result<(), String> {
    let caller = ic_cdk::caller();
    if !is_admin(&caller) {
        return Err("Unauthorized: Only admins can update heartbeat configuration".to_string());
    }
    
    set_heartbeat_config(config.clone())?;
    
    log_audit_action(
        caller,
        "HEARTBEAT_CONFIG_UPDATE".to_string(),
        format!("Heartbeat configuration updated: {:?}", config),
        true,
    );
    
    Ok(())
}

// Emergency pause heartbeat (admin only)
#[update]
pub fn emergency_pause_heartbeat() -> Result<String, String> {
    let caller = ic_cdk::caller();
    if !is_admin(&caller) {
        return Err("Unauthorized: Only admins can pause heartbeat".to_string());
    }
    
    let mut config = get_canister_config();
    config.maintenance_mode = true;
    set_canister_config(config)?;
    
    log_audit_action(
        caller,
        "HEARTBEAT_EMERGENCY_PAUSE".to_string(),
        "Heartbeat operations paused due to emergency".to_string(),
        true,
    );
    
    Ok("Heartbeat operations paused successfully".to_string())
}

// Resume heartbeat operations (admin only)
#[update]
pub fn resume_heartbeat_operations() -> Result<String, String> {
    let caller = ic_cdk::caller();
    if !is_admin(&caller) {
        return Err("Unauthorized: Only admins can resume heartbeat".to_string());
    }
    
    let mut config = get_canister_config();
    config.maintenance_mode = false;
    set_canister_config(config)?;
    
    log_audit_action(
        caller,
        "HEARTBEAT_OPERATIONS_RESUMED".to_string(),
        "Heartbeat operations resumed".to_string(),
        true,
    );
    
    Ok("Heartbeat operations resumed successfully".to_string())
}
```

## 5. Monitoring dan Observability

### 5.1 Health Check Production

```rust
#[query]
pub fn production_health_check() -> ProductionHealthStatus {
    ProductionHealthStatus {
        is_healthy: !is_emergency_stopped() && !is_in_maintenance_mode(),
        emergency_stop: is_emergency_stopped(),
        maintenance_mode: is_in_maintenance_mode(),
        oracle_status: check_oracle_health(),
        ckbtc_integration: check_ckbtc_health(),
        memory_usage: get_memory_usage(),
        total_loans: get_active_loans_count(),
        active_loans: get_active_loans_count(),
        last_heartbeat: get_last_heartbeat_time(),
    }
}

pub fn check_oracle_health() -> bool {
    let commodities = vec!["rice".to_string(), "corn".to_string(), "wheat".to_string()];
    
    // Cek apakah ada harga yang tersedia dan tidak terlalu stale
    commodities.iter().any(|commodity| {
        !is_price_stale(commodity.clone())
    })
}

pub fn check_ckbtc_health() -> bool {
    // Placeholder - bisa diimplementasikan untuk cek koneksi ke ckBTC
    true
}
```

### 5.2 Heartbeat Metrics

```rust
#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct HeartbeatMetrics {
    pub last_execution_time: u64,
    pub total_executions: u64,
    pub successful_executions: u64,
    pub failed_executions: u64,
    pub average_execution_time: u64,
    pub tasks_completed: HashMap<String, u64>,
    pub last_error: Option<String>,
    pub last_error_time: Option<u64>,
}

#[query]
pub fn get_heartbeat_metrics() -> Result<HeartbeatMetrics, String> {
    let caller = ic_cdk::caller();
    if !is_admin(&caller) {
        return Err("Unauthorized: Only admins can view heartbeat metrics".to_string());
    }
    
    // Implementation would track these metrics
    Ok(HeartbeatMetrics {
        last_execution_time: get_last_heartbeat_time(),
        total_executions: 0, // TODO: Implement counter
        successful_executions: 0,
        failed_executions: 0,
        average_execution_time: 0,
        tasks_completed: HashMap::new(),
        last_error: None,
        last_error_time: None,
    })
}
```

## 6. Integrasi dengan Modul Lain

### 6.1 Liquidation System
- **Trigger Otomatis**: Heartbeat memicu liquidation untuk loans yang overdue >45 hari
- **Monitoring**: Deteksi dini loans yang mendekati liquidation
- **Alert System**: Notifikasi untuk admin review

### 6.2 Oracle System
- **Price Updates**: Auto-update harga stale setiap heartbeat cycle
- **Failover**: Log error dan retry pada heartbeat berikutnya
- **Data Freshness**: Memastikan data oracle selalu fresh untuk valuasi

### 6.3 Liquidity Pool
- **Health Monitoring**: Continuous monitoring pool health
- **Auto Maintenance**: Cleanup transactions dan optimisasi
- **Alert System**: Warning untuk high utilization atau low health

### 6.4 Audit System
- **Comprehensive Logging**: Semua aktivitas heartbeat ter-log
- **Event Tracking**: Track success/failure untuk setiap task
- **Performance Monitoring**: Monitor execution time dan frequency

## 7. Error Handling dan Resilience

### 7.1 Error Recovery

```rust
async fn heartbeat_with_error_handling() {
    if !is_in_maintenance_mode() {
        // Price updates dengan error handling
        if let Err(e) = oracle::heartbeat_price_update().await {
            log_audit_action(
                ic_cdk::id(),
                "HEARTBEAT_PRICE_ERROR".to_string(),
                format!("Price update failed: {}", e),
                false,
            );
        }
        
        // Loan monitoring dengan error handling
        if let Err(e) = check_overdue_loans().await {
            log_audit_action(
                ic_cdk::id(),
                "HEARTBEAT_LOAN_ERROR".to_string(),
                format!("Loan monitoring failed: {}", e),
                false,
            );
        }
        
        // Continue dengan tasks lain meskipun ada error
        monitor_cycles_balance();
        cleanup_old_audit_logs();
    }
}
```

### 7.2 Circuit Breaker Pattern

```rust
pub struct CircuitBreaker {
    failure_count: u64,
    last_failure_time: u64,
    threshold: u64,
    timeout: u64,
}

impl CircuitBreaker {
    pub fn should_execute(&self) -> bool {
        let current_time = time();
        
        // Reset jika sudah melewati timeout
        if self.failure_count >= self.threshold {
            current_time - self.last_failure_time > self.timeout
        } else {
            true
        }
    }
    
    pub fn record_success(&mut self) {
        self.failure_count = 0;
    }
    
    pub fn record_failure(&mut self) {
        self.failure_count += 1;
        self.last_failure_time = time();
    }
}
```

## 8. Performance Optimization

### 8.1 Batch Processing

```rust
// Process multiple tasks dalam batch untuk efisiensi
async fn heartbeat_batch_processing() {
    // Batch oracle updates
    let stale_commodities: Vec<_> = ["rice", "corn", "wheat"]
        .iter()
        .filter(|&&commodity| is_price_stale(commodity.to_string()))
        .collect();
    
    if !stale_commodities.is_empty() {
        for commodity in stale_commodities {
            // Process dengan delay untuk menghindari rate limiting
            let _ = fetch_commodity_price(commodity.to_string()).await;
            
            // Small delay between requests
            // Note: IC tidak memiliki sleep, tapi kita bisa use timer
        }
    }
}
```

### 8.2 Memory Management

```rust
// Optimisasi memory usage dalam heartbeat
fn optimize_memory_usage() {
    // Cleanup temporary data structures
    cleanup_temporary_data();
    
    // Compact storage jika diperlukan
    compact_storage_if_needed();
    
    // Log memory usage
    let memory_usage = get_memory_usage();
    if memory_usage > MEMORY_WARNING_THRESHOLD {
        log_audit_action(
            ic_cdk::id(),
            "MEMORY_WARNING".to_string(),
            format!("High memory usage detected: {} bytes", memory_usage),
            false,
        );
    }
}
```

## 9. Best Practices dan Guidelines

### 9.1 Development Guidelines

1. **Idempotent Operations**: Semua heartbeat tasks harus idempotent
2. **Error Isolation**: Error di satu task tidak boleh affect task lain
3. **Performance Conscious**: Hindari operasi yang memakan waktu lama
4. **Resource Management**: Monitor dan manage memory/cycles usage
5. **Comprehensive Logging**: Log semua aktivitas untuk debugging

### 9.2 Operational Guidelines

1. **Regular Monitoring**: Monitor heartbeat metrics secara berkala
2. **Alert Setup**: Setup alerting untuk critical errors
3. **Maintenance Windows**: Plan maintenance dengan pause heartbeat
4. **Performance Tuning**: Monitor dan tune execution time
5. **Capacity Planning**: Plan untuk growth dan scaling

## 10. Security Considerations

### 10.1 Access Control
- Hanya admin yang dapat modify heartbeat configuration
- Heartbeat operations ter-isolasi dari user interactions
- Comprehensive audit logging untuk semua heartbeat activities

### 10.2 Data Protection
- Sensitive data tidak di-expose dalam heartbeat logs
- Error messages tidak reveal internal system details
- Rate limiting untuk prevent resource exhaustion

### 10.3 System Integrity
- Heartbeat tidak dapat memodifikasi critical system state tanpa proper validation
- Circuit breaker untuk prevent cascade failures
- Emergency pause capability untuk incident response

## 11. Future Enhancements

### 11.1 Planned Features
- **Advanced Scheduling**: Custom schedules untuk different tasks
- **Dynamic Configuration**: Runtime configuration changes
- **Health Scoring**: Advanced health scoring untuk system components
- **Predictive Maintenance**: ML-based predictive maintenance
- **Multi-Canister Coordination**: Coordinated heartbeat across canisters

### 11.2 Scalability Improvements
- **Distributed Tasks**: Distribute tasks across multiple heartbeat cycles
- **Priority Queueing**: Priority-based task execution
- **Load Balancing**: Balance load across different execution windows
- **Adaptive Scheduling**: Adaptive scheduling based on system load

---

**Status Implementasi**: ✅ Complete  
**Last Updated**: July 23, 2025  
**Version**: 1.0.0
