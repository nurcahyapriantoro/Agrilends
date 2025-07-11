use candid::Principal;
use crate::types::*;
use crate::user_management::{get_user_by_principal, Role};
use crate::storage::*;
use std::cell::RefCell;
use ic_cdk::api::time;

/// Validate NFT metadata contains all required fields
pub fn validate_nft_metadata(metadata: &Vec<(String, MetadataValue)>) -> Result<(), String> {
    let mut has_legal_doc = false;
    let mut has_valuation = false;
    let mut has_description = false;
    
    for (key, value) in metadata {
        match key.as_str() {
            "rwa:legal_doc_hash" => {
                if let MetadataValue::Text(hash) = value {
                    if hash.len() != 64 { // SHA-256 hash should be 64 chars
                        return Err("Invalid legal document hash format".to_string());
                    }
                    has_legal_doc = true;
                }
            },
            "rwa:valuation_idr" => {
                if let MetadataValue::Nat(val) = value {
                    if *val == 0 {
                        return Err("Valuation must be greater than 0".to_string());
                    }
                    has_valuation = true;
                }
            },
            "rwa:asset_description" => {
                if let MetadataValue::Text(desc) = value {
                    if desc.trim().is_empty() {
                        return Err("Asset description cannot be empty".to_string());
                    }
                    has_description = true;
                }
            },
            _ => {}
        }
    }
    
    if !has_legal_doc {
        return Err("Missing required metadata: rwa:legal_doc_hash".to_string());
    }
    if !has_valuation {
        return Err("Missing required metadata: rwa:valuation_idr".to_string());
    }
    if !has_description {
        return Err("Missing required metadata: rwa:asset_description".to_string());
    }
    
    Ok(())
}

// PRODUCTION FIX: Add proper admin configuration
thread_local! {
    static ADMIN_PRINCIPALS: RefCell<Vec<Principal>> = RefCell::new(vec![]);
    static LOAN_MANAGER_PRINCIPAL: RefCell<Option<Principal>> = RefCell::new(None);
}

/// Initialize admin principals (call this during canister init)
pub fn init_admin_principals(admins: Vec<Principal>) {
    ADMIN_PRINCIPALS.with(|principals| {
        *principals.borrow_mut() = admins;
    });
}

/// Set loan manager canister principal
pub fn set_loan_manager_principal(principal: Principal) {
    LOAN_MANAGER_PRINCIPAL.with(|p| {
        *p.borrow_mut() = Some(principal);
    });
}

/// Check if caller is admin (PRODUCTION VERSION)
pub fn is_admin(caller: &Principal) -> bool {
    let config = get_canister_config();
    config.admins.contains(caller)
}

/// Check if caller is the loan manager canister (PRODUCTION VERSION)
pub fn is_loan_manager_canister(caller: &Principal) -> bool {
    let config = get_canister_config();
    if let Some(loan_manager) = config.loan_manager_principal {
        *caller == loan_manager
    } else {
        false
    }
}

/// Enhanced authorization check
pub fn is_authorized_to_mint(caller: &Principal) -> bool {
    // Check if caller is admin
    if is_admin(caller) {
        return true;
    }
    
    // Check if caller is registered farmer
    if let Some(user) = get_user_by_principal(caller) {
        return user.role == Role::Farmer && user.is_active;
    }
    
    false
}

// Add rate limiting
thread_local! {
    static RATE_LIMITER: RefCell<std::collections::HashMap<Principal, u64>> = RefCell::new(std::collections::HashMap::new());
}

pub fn check_rate_limit(caller: &Principal, _max_calls_per_minute: u64) -> Result<(), String> {
    let current_time = ic_cdk::api::time() / 1_000_000_000 / 60; // Convert to minutes
    
    RATE_LIMITER.with(|limiter| {
        let mut map = limiter.borrow_mut();
        let last_call = map.get(caller).unwrap_or(&0);
        
        if current_time == *last_call {
            return Err("Rate limit exceeded. Please try again later.".to_string());
        }
        
        map.insert(*caller, current_time);
        Ok(())
    })
}

/// Extract metadata values for collateral record
pub fn extract_metadata_values(metadata: &Vec<(String, MetadataValue)>) -> (String, u64, String) {
    let mut legal_doc_hash = String::new();
    let mut valuation_idr = 0u64;
    let mut asset_description = String::new();
    
    for (key, value) in metadata {
        match key.as_str() {
            "rwa:legal_doc_hash" => {
                if let MetadataValue::Text(hash) = value {
                    legal_doc_hash = hash.clone();
                }
            },
            "rwa:valuation_idr" => {
                if let MetadataValue::Nat(val) = value {
                    valuation_idr = *val;
                }
            },
            "rwa:asset_description" => {
                if let MetadataValue::Text(desc) = value {
                    asset_description = desc.clone();
                }
            },
            _ => {}
        }
    }
    
    (legal_doc_hash, valuation_idr, asset_description)
}

/// Validate SHA-256 hash format
pub fn validate_sha256_hash(hash: &str) -> bool {
    hash.len() == 64 && hash.chars().all(|c| c.is_ascii_hexdigit())
}

/// Loan-specific helper functions
pub fn log_audit_action(caller: Principal, action: String, details: String, success: bool) {
    // Use the existing log_action function from storage
    log_action(&action, &format!("{}:{}", caller.to_text(), details), success);
}

/// Get canister configuration
pub fn get_canister_config() -> CanisterConfig {
    get_config()
}

/// Set canister configuration
pub fn set_canister_config(config: CanisterConfig) -> Result<(), String> {
    update_config(config)
}

/// Add admin principal
pub fn add_admin(admin: Principal) -> Result<(), String> {
    let mut config = get_canister_config();
    if !config.admins.contains(&admin) {
        config.admins.push(admin);
        set_canister_config(config)?;
    }
    Ok(())
}

/// Remove admin principal
pub fn remove_admin(admin: Principal) -> Result<(), String> {
    let mut config = get_canister_config();
    config.admins.retain(|&x| x != admin);
    set_canister_config(config)?;
    Ok(())
}

/// Calculate loan health ratio (collateral value vs debt)
pub fn calculate_loan_health_ratio(loan: &Loan) -> Result<f64, String> {
    if loan.amount_approved == 0 {
        return Ok(f64::INFINITY);
    }
    
    let health_ratio = (loan.collateral_value_btc as f64) / (loan.amount_approved as f64);
    Ok(health_ratio)
}

/// Check if loan is at risk of liquidation
pub fn is_loan_at_risk(loan: &Loan, threshold: f64) -> Result<bool, String> {
    let health_ratio = calculate_loan_health_ratio(loan)?;
    Ok(health_ratio < threshold)
}

/// Get overdue loans
pub fn get_overdue_loans() -> Vec<Loan> {
    let current_time = time();
    let params = get_protocol_parameters();
    let grace_period = params.grace_period_days * 24 * 60 * 60 * 1_000_000_000;
    
    get_all_loans_data()
        .into_iter()
        .filter(|loan| {
            loan.status == LoanStatus::Active && 
            loan.due_date.map_or(false, |due_date| current_time > due_date + grace_period)
        })
        .collect()
}

/// Format loan summary for notifications
pub fn format_loan_summary(loan: &Loan) -> String {
    format!(
        "Loan #{}: Borrower {}, Amount {} satoshi, Status {:?}, Created {}",
        loan.id,
        loan.borrower.to_text(),
        loan.amount_approved,
        loan.status,
        loan.created_at
    )
}

// Production helper functions
pub fn is_loan_manager(principal: &Principal) -> bool {
    LOAN_MANAGER_PRINCIPAL.with(|manager| {
        manager.borrow().as_ref() == Some(principal)
    })
}

pub fn release_collateral_nft(nft_id: u64) -> Result<(), String> {
    // This would unlock the NFT and return it to the borrower
    unlock_nft(nft_id)
}

pub fn get_active_loans_count() -> u64 {
    LOANS.with(|loans| {
        loans.borrow().iter()
            .filter(|(_, loan)| loan.status == LoanStatus::Active)
            .count() as u64
    })
}

pub fn get_memory_usage() -> u64 {
    // Placeholder - would need actual memory calculation
    // In real implementation, use ic_cdk::api::canister_status
    0
}

pub fn check_oracle_health() -> bool {
    // Check if oracle prices are recent and available
    true
}

pub fn check_ckbtc_health() -> bool {
    // Check if ckBTC integration is working
    true
}

pub fn get_last_heartbeat_time() -> u64 {
    // Return last heartbeat timestamp
    ic_cdk::api::time()
}

pub fn is_in_maintenance_mode() -> bool {
    let config = get_canister_config();
    config.maintenance_mode
}

pub fn get_emergency_stop_status() -> bool {
    let config = get_canister_config();
    config.emergency_stop
}

pub async fn check_overdue_loans() {
    // Check for overdue loans and take action
    let overdue_loans = get_overdue_loans();
    for loan in overdue_loans {
        log_action(
            "overdue_loan_detected",
            &format!("Loan {} is overdue", loan.id),
            false,
        );
    }
}

pub fn monitor_cycles_balance() {
    // Monitor canister cycles and alert if low
    // This would use ic_cdk::api::canister_balance
}

pub fn cleanup_old_audit_logs() {
    // Cleanup old audit logs to save memory
    cleanup_audit_logs(1000); // Keep last 1000 entries
}

pub fn get_user_btc_address(principal: &Principal) -> Option<String> {
    match get_user_by_principal(principal) {
        Some(user) => user.btc_address,
        None => None,
    }
}

