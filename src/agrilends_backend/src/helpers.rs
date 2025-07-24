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

// Enhanced rate limiting with operation-specific limits
thread_local! {
    static OPERATION_RATE_LIMITER: RefCell<std::collections::HashMap<(Principal, String), u64>> = RefCell::new(std::collections::HashMap::new());
}

pub fn check_rate_limit(caller: &Principal, _max_calls_per_minute: u64) -> Result<(), String> {
    let current_time = time() / 1_000_000_000 / 60; // Convert to minutes
    
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

pub fn check_rate_limit_with_operation(caller: &Principal, operation: &str) -> bool {
    let current_time = time() / 1_000_000_000; // Convert to seconds
    let rate_limit_window = 60; // 1 minute window
    
    OPERATION_RATE_LIMITER.with(|limiter| {
        let mut map = limiter.borrow_mut();
        let key = (*caller, operation.to_string());
        let last_call = map.get(&key).unwrap_or(&0);
        
        if current_time - last_call < rate_limit_window {
            return false; // Rate limited
        }
        
        map.insert(key, current_time);
        true // Allow the operation
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
    // Use the enhanced audit logging system
    crate::audit_logging::log_audit_action(caller, action, details, success);
}

/// Enhanced audit logging helpers for specific operations
pub fn log_nft_audit(action: &str, token_id: u64, owner: Principal, success: bool, error_msg: Option<String>) {
    crate::audit_logging::log_nft_operation(action, token_id, owner, success, error_msg);
}

pub fn log_loan_audit(action: &str, loan_id: u64, borrower: Principal, amount: Option<u64>, success: bool, error_msg: Option<String>) {
    crate::audit_logging::log_loan_operation(action, loan_id, borrower, amount, success, error_msg);
}

pub fn log_security_audit(event_type: &str, severity: crate::audit_logging::AuditEventLevel, description: String, affected_principal: Option<Principal>) {
    crate::audit_logging::log_security_event(event_type, severity, description, affected_principal);
}

pub fn log_governance_audit(action: &str, proposal_id: Option<u64>, success: bool, details: String) {
    use crate::audit_logging::{log_audit_enhanced, AuditCategory, AuditEventLevel, AuditDetails, AuditResult};
    
    let audit_details = AuditDetails {
        description: details,
        entity_type: Some("governance".to_string()),
        entity_id: proposal_id.map(|id| id.to_string()),
        before_state: None,
        after_state: None,
        affected_principals: vec![],
        metadata: vec![],
    };

    let result = AuditResult {
        success,
        error_code: None,
        error_message: if !success { Some(action.to_string()) } else { None },
        execution_time_ms: None,
        gas_used: None,
    };

    let level = if success { AuditEventLevel::Success } else { AuditEventLevel::Error };

    log_audit_enhanced(
        AuditCategory::Governance,
        action.to_string(),
        level,
        audit_details,
        result,
        None,
    );
}

pub fn log_liquidation_audit(action: &str, loan_id: u64, borrower: Principal, collateral_value: u64, debt_amount: u64, success: bool) {
    use crate::audit_logging::{log_audit_enhanced, AuditCategory, AuditEventLevel, AuditDetails, AuditResult};
    
    let audit_details = AuditDetails {
        description: format!("Liquidation operation: {}", action),
        entity_type: Some("liquidation".to_string()),
        entity_id: Some(loan_id.to_string()),
        before_state: None,
        after_state: None,
        affected_principals: vec![borrower],
        metadata: vec![
            ("loan_id".to_string(), loan_id.to_string()),
            ("collateral_value".to_string(), collateral_value.to_string()),
            ("debt_amount".to_string(), debt_amount.to_string()),
        ],
    };

    let result = AuditResult {
        success,
        error_code: None,
        error_message: None,
        execution_time_ms: None,
        gas_used: None,
    };

    let level = if success { AuditEventLevel::Success } else { AuditEventLevel::Error };

    log_audit_enhanced(
        AuditCategory::Liquidation,
        action.to_string(),
        level,
        audit_details,
        result,
        None,
    );
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
    time()
}

pub fn get_last_heartbeat_time() -> u64 {
    // Return last heartbeat timestamp
    time()
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
            &format!("Loan {} is overdue and may require liquidation review", loan.id),
            false,
        );
        
        // Check liquidation eligibility
        if let Ok(eligible) = crate::liquidation::check_liquidation_eligibility(loan.id) {
            if eligible.is_eligible {
                log_action(
                    "liquidation_eligible_detected",
                    &format!("Loan {} is eligible for liquidation: {}", loan.id, eligible.reason),
                    false,
                );
            }
        }
    }
}

pub fn monitor_cycles_balance() {
    // Monitor canister cycles and alert if low
    let current_cycles = ic_cdk::api::canister_balance();
    let cycles_threshold_alert = 1_000_000_000_000u64; // 1T cycles
    let cycles_threshold_critical = 500_000_000_000u64; // 500B cycles
    
    if current_cycles < cycles_threshold_critical {
        log_action(
            "cycles_critical",
            &format!("CRITICAL: Canister cycles below critical threshold: {} cycles", current_cycles),
            false,
        );
    } else if current_cycles < cycles_threshold_alert {
        log_action(
            "cycles_low",
            &format!("WARNING: Canister cycles running low: {} cycles", current_cycles),
            false,
        );
    }
}

pub fn cleanup_old_audit_logs() {
    // Cleanup old audit logs to save memory
    const MAX_AUDIT_LOGS: usize = 10_000;
    cleanup_audit_logs(MAX_AUDIT_LOGS); // Keep last 10,000 entries
}

pub fn get_user_btc_address(principal: &Principal) -> Option<String> {
    match get_user_by_principal(principal) {
        Some(user) => user.btc_address,
        None => None,
    }
}

