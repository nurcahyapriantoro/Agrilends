# Treasury Management Usage Examples

## Overview
Dokumen ini berisi contoh penggunaan lengkap dari fitur Treasury Management yang telah diimplementasikan.

## 1. Setup dan Initialization

### Inisialisasi Treasury
```rust
// Treasury akan otomatis diinisialisasi saat canister start
// Atau dapat dipanggil manual oleh admin
treasury_management::init_treasury();
```

### Registrasi Canister
```rust
// Register canister baru untuk cycle management
let result = register_canister(
    "oracle_canister".to_string(),
    Principal::from_text("rdmx6-jaaaa-aaaah-qcaiq-cai").unwrap(),
    CanisterType::Oracle,
    3 // Priority 3
).await;
```

## 2. Fee Collection dari Loan Operations

### Collection dari Loan Repayment
```rust
// Dipanggil dari loan repayment module
use crate::treasury_management::*;

// Saat borrower melakukan repayment
pub async fn process_repayment_with_fees(
    loan_id: u64,
    repayment_amount: u64
) -> Result<String, String> {
    // Calculate fees
    let admin_fee = (repayment_amount * 2) / 100; // 2% admin fee
    let interest_share = (repayment_amount * 3) / 100; // 3% interest share
    let total_fees = admin_fee + interest_share;
    
    // Process loan repayment first
    // ... loan repayment logic ...
    
    // Then collect fees to treasury
    let fee_result = process_loan_fee_collection(
        loan_id,
        total_fees,
        admin_fee,
        interest_share
    ).await;
    
    match fee_result {
        Ok(msg) => {
            log_audit_action(
                ic_cdk::caller(),
                "LOAN_REPAYMENT_WITH_FEES".to_string(),
                format!("Loan #{} repaid with {} fees collected", loan_id, total_fees),
                true,
            );
            Ok(format!("Repayment processed. {}", msg))
        },
        Err(e) => Err(format!("Fee collection failed: {}", e))
    }
}
```

### Collection dari Liquidation
```rust
// Dipanggil dari liquidation module
pub async fn process_liquidation_with_penalty(
    loan_id: u64,
    collateral_value: u64
) -> Result<String, String> {
    // Calculate liquidation penalty (5% of collateral value)
    let penalty_amount = (collateral_value * 5) / 100;
    
    // Process liquidation first
    // ... liquidation logic ...
    
    // Collect penalty to treasury
    let penalty_result = process_liquidation_penalty(
        loan_id,
        penalty_amount,
        "Loan default liquidation".to_string()
    ).await;
    
    match penalty_result {
        Ok(msg) => Ok(format!("Liquidation completed. {}", msg)),
        Err(e) => Err(format!("Penalty collection failed: {}", e))
    }
}
```

## 3. Manual Cycle Management

### Manual Top-up Canister
```rust
// Admin dapat melakukan top-up manual
pub async fn admin_top_up_canister() -> Result<String, String> {
    let result = top_up_canister_cycles("oracle_canister".to_string()).await;
    
    match result {
        Ok(msg) => {
            ic_cdk::println!("Top-up successful: {}", msg);
            Ok(msg)
        },
        Err(e) => {
            ic_cdk::println!("Top-up failed: {}", e);
            Err(e)
        }
    }
}
```

### Trigger Manual Cycle Distribution
```rust
// Memicu distribusi cycles ke semua canister yang membutuhkan
pub async fn admin_trigger_distribution() -> Result<String, String> {
    let result = trigger_cycle_distribution().await;
    
    match result {
        Ok(msg) => {
            ic_cdk::println!("Distribution completed: {}", msg);
            Ok(msg)
        },
        Err(e) => {
            ic_cdk::println!("Distribution failed: {}", e);
            Err(e)
        }
    }
}
```

## 4. Monitoring dan Reporting

### Get Treasury Health Report
```rust
pub fn check_treasury_health() -> TreasuryHealthReport {
    let health_report = get_treasury_health_report();
    
    ic_cdk::println!("Treasury Status: {}", health_report.overall_status);
    ic_cdk::println!("Current Balance: {} satoshi", health_report.current_balance);
    ic_cdk::println!("Runway: {} days", health_report.projected_runway_days);
    
    // Act on recommendations
    for recommendation in &health_report.recommendations {
        ic_cdk::println!("Recommendation: {}", recommendation);
    }
    
    health_report
}
```

### Get Treasury Statistics
```rust
pub fn display_treasury_stats() -> TreasuryStats {
    let stats = get_treasury_stats();
    
    ic_cdk::println!("=== TREASURY STATISTICS ===");
    ic_cdk::println!("Current Balance: {} satoshi", stats.current_balance);
    ic_cdk::println!("Total Revenue: {} satoshi", stats.total_revenue_collected);
    ic_cdk::println!("Total Cycles Distributed: {}", stats.total_cycles_distributed);
    ic_cdk::println!("Emergency Reserve: {} satoshi", stats.emergency_reserve);
    ic_cdk::println!("Active Canisters: {}", stats.active_canisters_count);
    ic_cdk::println!("Average Daily Revenue: {} satoshi", stats.average_daily_revenue);
    ic_cdk::println!("Projected Runway: {} days", stats.projected_runway_days);
    
    stats
}
```

### Monitor Canister Cycles
```rust
pub async fn monitor_canister_cycles() -> Vec<CanisterCycleStatus> {
    let cycle_status = get_canister_cycle_status().await;
    
    for status in &cycle_status {
        ic_cdk::println!("Canister: {}", status.canister_info.name);
        ic_cdk::println!("  Current Cycles: {}", status.current_cycles);
        ic_cdk::println!("  Days Remaining: {}", status.days_remaining);
        ic_cdk::println!("  Needs Top-up: {}", status.needs_top_up);
        
        if status.needs_top_up {
            ic_cdk::println!("  ⚠️  WARNING: This canister needs cycle top-up!");
        }
    }
    
    cycle_status
}
```

## 5. Configuration Management

### Update Treasury Configuration
```rust
pub fn update_treasury_config() -> Result<String, String> {
    // Update treasury parameters
    set_treasury_configuration(
        Some(200_000),  // Min balance: 0.002 BTC
        Some(25),       // Emergency reserve: 25%
        Some(200),      // Auto top-up: 200%
        Some(1800)      // Monitoring interval: 30 minutes
    )
}
```

### Update Canister Configuration
```rust
pub fn update_oracle_canister_config() -> Result<String, String> {
    // Update specific canister settings
    update_canister_config(
        "oracle_canister".to_string(),
        Some(2_000_000_000_000),  // 2T cycles threshold
        Some(20_000_000_000_000), // 20T cycles limit
        Some(2),                  // Priority 2
        Some(true)                // Auto top-up enabled
    )
}
```

## 6. Emergency Operations

### Emergency Withdrawal
```rust
pub async fn emergency_treasury_withdrawal(
    amount: u64,
    destination: Principal,
    reason: String
) -> Result<String, String> {
    // Only super admin can perform this
    let result = emergency_withdraw(amount, destination, reason).await;
    
    match result {
        Ok(tx_id) => {
            ic_cdk::println!("Emergency withdrawal completed: {}", tx_id);
            Ok(tx_id)
        },
        Err(e) => {
            ic_cdk::println!("Emergency withdrawal failed: {}", e);
            Err(e)
        }
    }
}
```

## 7. Automated Operations (Heartbeat)

### Setup Heartbeat untuk Auto Management
```rust
#[ic_cdk_macros::heartbeat]
pub async fn heartbeat() {
    // Treasury heartbeat akan otomatis:
    // 1. Check semua canister cycles
    // 2. Top-up canister yang membutuhkan
    // 3. Update metrics
    treasury_management::treasury_heartbeat().await;
}
```

## 8. Revenue Tracking dan Analysis

### Get Revenue Log dengan Filter
```rust
pub fn analyze_revenue_trends() -> Vec<RevenueEntry> {
    // Get last 30 days revenue
    let thirty_days_ago = ic_cdk::api::time() - (30 * 24 * 60 * 60 * 1_000_000_000);
    
    let revenue_log = get_revenue_log(
        Some(100),      // Limit 100 entries
        None,           // All revenue types
        Some(thirty_days_ago), // From 30 days ago
        None            // Until now
    );
    
    // Analyze revenue by type
    let mut admin_fee_total = 0u64;
    let mut interest_share_total = 0u64;
    let mut liquidation_penalty_total = 0u64;
    
    for entry in &revenue_log {
        match entry.revenue_type {
            RevenueType::AdminFee => admin_fee_total += entry.amount,
            RevenueType::InterestShare => interest_share_total += entry.amount,
            RevenueType::LiquidationPenalty => liquidation_penalty_total += entry.amount,
            _ => {}
        }
    }
    
    ic_cdk::println!("=== REVENUE ANALYSIS (30 days) ===");
    ic_cdk::println!("Admin Fees: {} satoshi", admin_fee_total);
    ic_cdk::println!("Interest Share: {} satoshi", interest_share_total);
    ic_cdk::println!("Liquidation Penalties: {} satoshi", liquidation_penalty_total);
    ic_cdk::println!("Total Entries: {}", revenue_log.len());
    
    revenue_log
}
```

### Get Cycle Transaction History
```rust
pub fn analyze_cycle_transactions() -> Vec<CycleTransaction> {
    // Get recent cycle transactions
    let cycle_transactions = get_cycle_transactions(
        Some(50),       // Limit 50 entries
        None,           // All time
        None,           // All time
        None            // All canisters
    );
    
    let mut total_cycles_distributed = 0u64;
    let mut total_ckbtc_spent = 0u64;
    let mut successful_transactions = 0u32;
    
    for tx in &cycle_transactions {
        match tx.status {
            TransactionStatus::Completed => {
                total_cycles_distributed += tx.cycles_amount;
                total_ckbtc_spent += tx.ckbtc_cost;
                successful_transactions += 1;
            },
            _ => {}
        }
    }
    
    ic_cdk::println!("=== CYCLE TRANSACTION ANALYSIS ===");
    ic_cdk::println!("Total Cycles Distributed: {}", total_cycles_distributed);
    ic_cdk::println!("Total ckBTC Spent: {} satoshi", total_ckbtc_spent);
    ic_cdk::println!("Successful Transactions: {}", successful_transactions);
    ic_cdk::println!("Total Transactions: {}", cycle_transactions.len());
    
    if total_cycles_distributed > 0 {
        let avg_exchange_rate = total_ckbtc_spent as f64 / total_cycles_distributed as f64;
        ic_cdk::println!("Average Exchange Rate: {:.6} satoshi/cycle", avg_exchange_rate);
    }
    
    cycle_transactions
}
```

## 9. Integration Examples

### Frontend Integration (Query Calls)
```typescript
// TypeScript/JavaScript frontend code
import { Actor, HttpAgent } from "@dfinity/agent";

// Get treasury stats for dashboard
async function getTreasuryStats() {
    const stats = await backendCanister.get_treasury_stats();
    
    document.getElementById('balance').textContent = 
        `${stats.current_balance} satoshi`;
    document.getElementById('revenue').textContent = 
        `${stats.total_revenue_collected} satoshi`;
    document.getElementById('runway').textContent = 
        `${stats.projected_runway_days} days`;
}

// Monitor canister health
async function monitorCanisterHealth() {
    const healthReport = await backendCanister.get_treasury_health_report();
    
    const statusElement = document.getElementById('treasury-status');
    statusElement.textContent = healthReport.overall_status;
    statusElement.className = `status-${healthReport.overall_status.toLowerCase()}`;
    
    const recommendationsElement = document.getElementById('recommendations');
    recommendationsElement.innerHTML = healthReport.recommendations
        .map(rec => `<li>${rec}</li>`)
        .join('');
}
```

### CLI Integration
```bash
# dfx CLI commands untuk Treasury Management

# Get treasury statistics
dfx canister call agrilends_backend get_treasury_stats

# Get health report
dfx canister call agrilends_backend get_treasury_health_report

# Manual cycle top-up
dfx canister call agrilends_backend top_up_canister_cycles '("oracle_canister")'

# Register new canister
dfx canister call agrilends_backend register_canister '("new_canister", principal "rdmx6-jaaaa-aaaah-qcaiq-cai", variant {Core}, 3)'

# Get canister cycle status
dfx canister call agrilends_backend get_canister_cycle_status

# Get revenue log
dfx canister call agrilends_backend get_revenue_log '(opt 50, null, null, null)'
```

## 10. Best Practices

### Error Handling
```rust
// Selalu handle error dengan proper logging
match collect_fees(loan_id, amount, RevenueType::AdminFee).await {
    Ok(result) => {
        log_audit_action(
            caller,
            "FEE_COLLECTION_SUCCESS".to_string(),
            format!("Successfully collected {} fees", amount),
            true,
        );
        Ok(result)
    },
    Err(e) => {
        log_audit_action(
            caller,
            "FEE_COLLECTION_FAILED".to_string(),
            format!("Failed to collect fees: {}", e),
            false,
        );
        Err(format!("Fee collection failed: {}", e))
    }
}
```

### Regular Monitoring
```rust
// Setup regular monitoring schedule
pub async fn daily_treasury_check() {
    let health_report = get_treasury_health_report();
    
    // Alert if runway is less than 30 days
    if health_report.projected_runway_days < 30 {
        // Send alert to admin
        ic_cdk::println!("🚨 ALERT: Treasury runway critical!");
    }
    
    // Auto top-up critical canisters
    let cycle_status = get_canister_cycle_status().await;
    for status in cycle_status {
        if status.needs_top_up && status.canister_info.priority <= 2 {
            let _ = top_up_canister_cycles(status.canister_info.name).await;
        }
    }
}
```

### Security Considerations
```rust
// Selalu verify caller sebelum operasi sensitif
pub async fn secure_treasury_operation() -> Result<String, String> {
    let caller = ic_cdk::caller();
    
    // Check if caller is authorized
    if !is_admin(&caller) {
        log_audit_action(
            caller,
            "UNAUTHORIZED_TREASURY_ACCESS".to_string(),
            format!("Unauthorized access attempt by {}", caller.to_text()),
            false,
        );
        return Err("Unauthorized access".to_string());
    }
    
    // Proceed with operation
    Ok("Operation completed".to_string())
}
```

## Kesimpulan

Treasury Management system telah diimplementasikan dengan lengkap dan siap untuk production. Contoh-contoh di atas menunjukkan:

1. **Integration**: Mudah diintegrasikan dengan module lain
2. **Monitoring**: Comprehensive monitoring dan alerting
3. **Security**: Multi-layer security dengan audit logging
4. **Flexibility**: Konfigurasi yang fleksibel dan update-able
5. **Automation**: Full automation dengan manual override
6. **Transparency**: Complete visibility dan reporting

System ini dapat mengelola treasury dengan minimal manual intervention sambil memberikan full control dan visibility kepada admin.
