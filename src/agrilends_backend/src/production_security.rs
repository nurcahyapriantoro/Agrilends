use candid::Principal;
use ic_cdk::api::time;
use std::cell::RefCell;
use crate::storage::log_action;

// Enhanced security measures for production
thread_local! {
    static BLACKLISTED_PRINCIPALS: RefCell<std::collections::HashSet<Principal>> = RefCell::new(std::collections::HashSet::new());
    static ADMIN_ACTIONS_LOG: RefCell<Vec<(u64, Principal, String)>> = RefCell::new(Vec::new());
    static FAILED_AUTH_ATTEMPTS: RefCell<std::collections::HashMap<Principal, u64>> = RefCell::new(std::collections::HashMap::new());
}

/// Security middleware - check if principal is blacklisted
pub fn security_check(principal: &Principal) -> Result<(), String> {
    BLACKLISTED_PRINCIPALS.with(|blacklist| {
        if blacklist.borrow().contains(principal) {
            log_action("security_violation", &format!("Blacklisted principal attempted access: {}", principal.to_text()), false);
            return Err("Access denied: Principal is blacklisted".to_string());
        }
        Ok(())
    })
}

/// Track failed authentication attempts
pub fn track_failed_auth(principal: &Principal) {
    FAILED_AUTH_ATTEMPTS.with(|attempts| {
        let mut map = attempts.borrow_mut();
        let count = map.get(principal).unwrap_or(&0) + 1;
        map.insert(*principal, count);
        
        // Auto-blacklist after 10 failed attempts
        if count >= 10 {
            BLACKLISTED_PRINCIPALS.with(|blacklist| {
                blacklist.borrow_mut().insert(*principal);
            });
            log_action("auto_blacklist", &format!("Principal auto-blacklisted after {} failed attempts: {}", count, principal.to_text()), true);
        }
    });
}

/// Admin function to blacklist a principal
pub fn admin_blacklist_principal(principal: Principal) -> Result<(), String> {
    let caller = ic_cdk::caller();
    
    // Only allow admin to blacklist
    if !crate::helpers::is_admin(&caller) {
        return Err("Unauthorized: Only admins can blacklist principals".to_string());
    }
    
    BLACKLISTED_PRINCIPALS.with(|blacklist| {
        blacklist.borrow_mut().insert(principal);
    });
    
    ADMIN_ACTIONS_LOG.with(|log| {
        log.borrow_mut().push((time(), caller, format!("Blacklisted principal: {}", principal.to_text())));
    });
    
    log_action("admin_blacklist", &format!("Admin {} blacklisted principal: {}", caller.to_text(), principal.to_text()), true);
    Ok(())
}