# Agrilends Audit Logging - Integration Guide

## Overview
This guide provides step-by-step instructions for integrating the comprehensive audit logging system into existing and new Agrilends modules.

## Quick Start

### 1. Import Required Modules
```rust
use crate::helpers::{
    log_audit_action,      // Basic audit logging
    log_nft_audit,         // NFT-specific logging
    log_loan_audit,        // Loan-specific logging
    log_security_audit,    // Security event logging
    log_governance_audit,  // Governance logging
    log_liquidation_audit, // Liquidation logging
};
use crate::audit_logging::{
    AuditEventLevel,       // Event severity levels
    AuditCategory,         // Event categories
};
```

### 2. Basic Audit Logging Pattern
```rust
pub fn my_function() -> Result<String, String> {
    let caller = ic_cdk::caller();
    let start_time = ic_cdk::api::time();
    
    // Your business logic here
    let result = perform_operation();
    
    match result {
        Ok(success_data) => {
            log_audit_action(
                caller,
                "MY_OPERATION".to_string(),
                format!("Successfully performed operation: {}", success_data),
                true
            );
            Ok(success_data)
        },
        Err(error) => {
            log_audit_action(
                caller,
                "MY_OPERATION".to_string(),
                format!("Operation failed: {}", error),
                false
            );
            Err(error)
        }
    }
}
```

## Integration Examples by Module

### User Management Integration

```rust
// In user_management.rs
use crate::helpers::log_audit_action;

#[update]
pub fn register_user(user_data: User) -> UserResult {
    let caller = ic_cdk::caller();
    
    // Existing validation logic...
    
    match register_user_logic(user_data.clone()) {
        UserResult::Ok(user) => {
            log_audit_action(
                caller,
                "REGISTER_USER".to_string(),
                format!("User registered with role: {:?}, Principal: {}", 
                       user.role, caller.to_text()),
                true
            );
            UserResult::Ok(user)
        },
        UserResult::Err(error) => {
            log_audit_action(
                caller,
                "REGISTER_USER".to_string(),
                format!("User registration failed: {}", error),
                false
            );
            UserResult::Err(error)
        }
    }
}

#[update]
pub fn update_user_role(target_principal: Principal, new_role: Role) -> Result<(), String> {
    let caller = ic_cdk::caller();
    
    // Authorization check
    if !is_admin(&caller) {
        log_security_audit(
            "UNAUTHORIZED_ROLE_UPDATE",
            AuditEventLevel::Warning,
            format!("Non-admin {} attempted to update role for {}", 
                   caller.to_text(), target_principal.to_text()),
            Some(caller)
        );
        return Err("Unauthorized".to_string());
    }
    
    // Perform role update
    match update_role_logic(target_principal, new_role.clone()) {
        Ok(_) => {
            log_audit_action(
                caller,
                "UPDATE_USER_ROLE".to_string(),
                format!("Role updated for {} to {:?}", target_principal.to_text(), new_role),
                true
            );
            Ok(())
        },
        Err(error) => {
            log_audit_action(
                caller,
                "UPDATE_USER_ROLE".to_string(),
                format!("Failed to update role for {}: {}", target_principal.to_text(), error),
                false
            );
            Err(error)
        }
    }
}
```

### NFT Operations Integration

```rust
// In rwa_nft.rs
use crate::helpers::log_nft_audit;

#[update]
pub fn mint_nft(owner: Principal, metadata: Vec<(String, MetadataValue)>) -> RWANFTResult {
    let caller = ic_cdk::caller();
    
    // Existing validation and minting logic...
    
    match mint_nft_logic(owner, metadata.clone()) {
        RWANFTResult::Ok(nft) => {
            log_nft_audit(
                "MINT_NFT",
                nft.token_id,
                owner,
                true,
                None
            );
            RWANFTResult::Ok(nft)
        },
        RWANFTResult::Err(error) => {
            log_nft_audit(
                "MINT_NFT",
                0, // No token ID for failed mint
                owner,
                false,
                Some(error.clone())
            );
            RWANFTResult::Err(error)
        }
    }
}

#[update]
pub fn transfer_nft(from: Principal, to: Principal, token_id: u64) -> Result<(), String> {
    let caller = ic_cdk::caller();
    
    // Authorization and transfer logic...
    
    match transfer_nft_logic(from, to, token_id) {
        Ok(_) => {
            log_nft_audit(
                "TRANSFER_NFT",
                token_id,
                from,
                true,
                None
            );
            Ok(())
        },
        Err(error) => {
            log_nft_audit(
                "TRANSFER_NFT",
                token_id,
                from,
                false,
                Some(error.clone())
            );
            Err(error)
        }
    }
}
```

### Loan Lifecycle Integration

```rust
// In loan_lifecycle.rs
use crate::helpers::log_loan_audit;

#[update]
pub async fn submit_loan_application(nft_id: u64, amount_requested: u64) -> Result<Loan, String> {
    let caller = ic_cdk::caller();
    
    // Application logic...
    
    match create_loan_application(caller, nft_id, amount_requested) {
        Ok(loan) => {
            log_loan_audit(
                "SUBMIT_APPLICATION",
                loan.id,
                loan.borrower,
                Some(loan.amount_requested),
                true,
                None
            );
            Ok(loan)
        },
        Err(error) => {
            log_loan_audit(
                "SUBMIT_APPLICATION",
                0, // No loan ID for failed application
                caller,
                Some(amount_requested),
                false,
                Some(error.clone())
            );
            Err(error)
        }
    }
}

#[update]
pub fn approve_loan(loan_id: u64, approved_amount: u64) -> Result<(), String> {
    let caller = ic_cdk::caller();
    
    // Only loan managers can approve
    if !is_loan_manager(&caller) {
        log_security_audit(
            "UNAUTHORIZED_LOAN_APPROVAL",
            AuditEventLevel::Critical,
            format!("Non-manager {} attempted to approve loan {}", caller.to_text(), loan_id),
            Some(caller)
        );
        return Err("Unauthorized".to_string());
    }
    
    match approve_loan_logic(loan_id, approved_amount) {
        Ok(loan) => {
            log_loan_audit(
                "APPROVE_LOAN",
                loan_id,
                loan.borrower,
                Some(approved_amount),
                true,
                None
            );
            Ok(())
        },
        Err(error) => {
            log_loan_audit(
                "APPROVE_LOAN",
                loan_id,
                Principal::anonymous(), // Unknown borrower at this point
                Some(approved_amount),
                false,
                Some(error.clone())
            );
            Err(error)
        }
    }
}
```

### Security Integration

```rust
// In production_security.rs
use crate::helpers::log_security_audit;
use crate::audit_logging::AuditEventLevel;

pub fn security_check(principal: &Principal) -> Result<(), String> {
    // Check blacklist
    if is_blacklisted(principal) {
        log_security_audit(
            "BLACKLISTED_ACCESS_ATTEMPT",
            AuditEventLevel::Critical,
            format!("Blacklisted principal {} attempted access", principal.to_text()),
            Some(*principal)
        );
        return Err("Access denied: Principal is blacklisted".to_string());
    }
    
    // Check rate limiting
    if is_rate_limited(principal) {
        log_security_audit(
            "RATE_LIMIT_EXCEEDED",
            AuditEventLevel::Warning,
            format!("Principal {} exceeded rate limit", principal.to_text()),
            Some(*principal)
        );
        return Err("Rate limit exceeded".to_string());
    }
    
    Ok(())
}

pub fn admin_blacklist_principal(principal: Principal) -> Result<(), String> {
    let caller = ic_cdk::caller();
    
    if !is_admin(&caller) {
        log_security_audit(
            "UNAUTHORIZED_BLACKLIST_ATTEMPT",
            AuditEventLevel::Critical,
            format!("Non-admin {} attempted to blacklist {}", caller.to_text(), principal.to_text()),
            Some(caller)
        );
        return Err("Unauthorized".to_string());
    }
    
    // Perform blacklisting
    add_to_blacklist(principal);
    
    log_security_audit(
        "PRINCIPAL_BLACKLISTED",
        AuditEventLevel::Critical,
        format!("Principal {} blacklisted by admin {}", principal.to_text(), caller.to_text()),
        Some(principal)
    );
    
    Ok(())
}
```

### Governance Integration

```rust
// In governance.rs
use crate::helpers::log_governance_audit;

#[update]
pub fn create_proposal(proposal_data: CreateProposalArgs) -> Result<u64, String> {
    let caller = ic_cdk::caller();
    
    // Validation logic...
    
    match create_proposal_logic(caller, proposal_data.clone()) {
        Ok(proposal_id) => {
            log_governance_audit(
                "CREATE_PROPOSAL",
                Some(proposal_id),
                true,
                format!("Proposal created: {} by {}", proposal_data.title, caller.to_text())
            );
            Ok(proposal_id)
        },
        Err(error) => {
            log_governance_audit(
                "CREATE_PROPOSAL",
                None,
                false,
                format!("Failed to create proposal: {}", error)
            );
            Err(error)
        }
    }
}

#[update]
pub fn vote_on_proposal(proposal_id: u64, vote: VoteOption) -> Result<(), String> {
    let caller = ic_cdk::caller();
    
    match cast_vote_logic(caller, proposal_id, vote.clone()) {
        Ok(_) => {
            log_governance_audit(
                "CAST_VOTE",
                Some(proposal_id),
                true,
                format!("Vote cast: {:?} on proposal {} by {}", vote, proposal_id, caller.to_text())
            );
            Ok(())
        },
        Err(error) => {
            log_governance_audit(
                "CAST_VOTE",
                Some(proposal_id),
                false,
                format!("Failed to cast vote: {}", error)
            );
            Err(error)
        }
    }
}
```

### Liquidation Integration

```rust
// In liquidation.rs
use crate::helpers::log_liquidation_audit;

#[update]
pub async fn trigger_liquidation(loan_id: u64) -> Result<String, String> {
    let caller = ic_cdk::caller();
    
    // Get loan details
    let loan = get_loan(loan_id).ok_or("Loan not found")?;
    
    // Check eligibility
    let eligibility = check_liquidation_eligibility(loan_id)?;
    if !eligibility.is_eligible {
        log_liquidation_audit(
            "LIQUIDATION_INELIGIBLE",
            loan_id,
            loan.borrower,
            loan.collateral_value,
            eligibility.outstanding_debt,
            false
        );
        return Err("Loan not eligible for liquidation".to_string());
    }
    
    // Perform liquidation
    match perform_liquidation_logic(&loan) {
        Ok(liquidation_result) => {
            log_liquidation_audit(
                "LIQUIDATION_COMPLETED",
                loan_id,
                loan.borrower,
                loan.collateral_value,
                liquidation_result.debt_recovered,
                true
            );
            Ok(format!("Liquidation completed. Debt recovered: {}", liquidation_result.debt_recovered))
        },
        Err(error) => {
            log_liquidation_audit(
                "LIQUIDATION_FAILED",
                loan_id,
                loan.borrower,
                loan.collateral_value,
                eligibility.outstanding_debt,
                false
            );
            Err(error)
        }
    }
}
```

## Advanced Audit Logging

### Using Enhanced Audit Logging

```rust
use crate::audit_logging::{
    log_audit_enhanced, AuditCategory, AuditEventLevel, 
    AuditDetails, AuditResult
};

pub fn complex_operation() -> Result<String, String> {
    let caller = ic_cdk::caller();
    let correlation_id = format!("operation-{}-{}", caller.to_text(), ic_cdk::api::time());
    
    // Step 1: Validation
    let validation_details = AuditDetails {
        description: "Input validation for complex operation".to_string(),
        entity_type: Some("operation".to_string()),
        entity_id: Some(correlation_id.clone()),
        before_state: Some("unvalidated".to_string()),
        after_state: Some("validated".to_string()),
        affected_principals: vec![caller],
        metadata: vec![
            ("step".to_string(), "validation".to_string()),
            ("input_size".to_string(), "large".to_string()),
        ],
    };
    
    let validation_result = AuditResult {
        success: true,
        error_code: None,
        error_message: None,
        execution_time_ms: Some(150),
        gas_used: Some(10000),
    };
    
    log_audit_enhanced(
        AuditCategory::UserManagement,
        "VALIDATE_INPUT".to_string(),
        AuditEventLevel::Info,
        validation_details,
        validation_result,
        Some(correlation_id.clone()),
    );
    
    // Step 2: Processing
    let processing_details = AuditDetails {
        description: "Main processing of complex operation".to_string(),
        entity_type: Some("operation".to_string()),
        entity_id: Some(correlation_id.clone()),
        before_state: Some("validated".to_string()),
        after_state: Some("processed".to_string()),
        affected_principals: vec![caller],
        metadata: vec![
            ("step".to_string(), "processing".to_string()),
            ("algorithm".to_string(), "advanced".to_string()),
        ],
    };
    
    let processing_result = AuditResult {
        success: true,
        error_code: None,
        error_message: None,
        execution_time_ms: Some(2500),
        gas_used: Some(50000),
    };
    
    log_audit_enhanced(
        AuditCategory::UserManagement,
        "PROCESS_OPERATION".to_string(),
        AuditEventLevel::Success,
        processing_details,
        processing_result,
        Some(correlation_id.clone()),
    );
    
    Ok("Operation completed successfully".to_string())
}
```

### Performance Monitoring

```rust
pub fn monitored_function() -> Result<String, String> {
    let start_time = ic_cdk::api::time();
    
    // Your function logic here
    let result = perform_work();
    
    let end_time = ic_cdk::api::time();
    let execution_time = (end_time - start_time) / 1_000_000; // Convert to milliseconds
    
    let details = AuditDetails {
        description: "Performance monitored function".to_string(),
        entity_type: Some("performance".to_string()),
        entity_id: None,
        before_state: None,
        after_state: None,
        affected_principals: vec![],
        metadata: vec![
            ("function_name".to_string(), "monitored_function".to_string()),
            ("execution_time_ms".to_string(), execution_time.to_string()),
        ],
    };
    
    let audit_result = AuditResult {
        success: result.is_ok(),
        error_code: if result.is_err() { Some("PERF_001".to_string()) } else { None },
        error_message: result.as_ref().err().cloned(),
        execution_time_ms: Some(execution_time),
        gas_used: None, // Could estimate gas usage here
    };
    
    log_audit_enhanced(
        AuditCategory::Maintenance,
        "PERFORMANCE_MONITOR".to_string(),
        if execution_time > 5000 { AuditEventLevel::Warning } else { AuditEventLevel::Info },
        details,
        audit_result,
        None,
    );
    
    result
}
```

## Best Practices

### 1. Consistent Action Naming
```rust
// Good - Use consistent, descriptive action names
"MINT_NFT"
"APPROVE_LOAN"
"TRANSFER_FUNDS"
"UPDATE_CONFIG"

// Avoid - Vague or inconsistent names
"do_something"
"process"
"action1"
```

### 2. Meaningful Context
```rust
// Good - Provide meaningful context
log_audit_action(
    caller,
    "APPROVE_LOAN".to_string(),
    format!("Loan {} approved for {} satoshi to borrower {}", 
           loan_id, amount, borrower.to_text()),
    true
);

// Avoid - Minimal context
log_audit_action(caller, "APPROVE".to_string(), "OK".to_string(), true);
```

### 3. Error Handling
```rust
// Always log both success and failure cases
match operation() {
    Ok(result) => {
        log_audit_action(caller, "OPERATION".to_string(), 
                        format!("Success: {}", result), true);
        Ok(result)
    },
    Err(error) => {
        log_audit_action(caller, "OPERATION".to_string(), 
                        format!("Failed: {}", error), false);
        Err(error)
    }
}
```

### 4. Security Events
```rust
// Always log security-relevant events
if !is_authorized(&caller) {
    log_security_audit(
        "UNAUTHORIZED_ACCESS",
        AuditEventLevel::Warning,
        format!("Principal {} attempted unauthorized access to {}", 
               caller.to_text(), function_name),
        Some(caller)
    );
    return Err("Unauthorized".to_string());
}
```

## Testing Your Integration

```rust
#[cfg(test)]
mod audit_integration_tests {
    use super::*;
    
    #[test]
    fn test_audit_logging_integration() {
        // Test your function that includes audit logging
        let result = my_audited_function();
        
        // Verify the function worked
        assert!(result.is_ok());
        
        // In a full test environment, you would also verify
        // that the appropriate audit logs were created
    }
}
```

## Troubleshooting

### Common Issues

1. **Memory Errors**: If you encounter memory allocation errors, check that you're using unique memory IDs for your audit storage.

2. **Performance Impact**: If audit logging is impacting performance, consider:
   - Using async logging for non-critical operations
   - Reducing the level of detail in logs
   - Implementing log batching

3. **Storage Limits**: Monitor storage usage and implement cleanup policies:
   ```rust
   // Regular cleanup of old logs
   if should_cleanup() {
       cleanup_old_audit_logs(365).await?; // Keep 1 year of logs
   }
   ```

## Migration from Existing Logging

If you have existing logging code, migrate gradually:

1. **Phase 1**: Add new audit logging alongside existing logging
2. **Phase 2**: Verify new logging captures all necessary information
3. **Phase 3**: Remove old logging code
4. **Phase 4**: Optimize and tune the new system

```rust
// During migration - run both systems
old_log_function(data);
log_audit_action(caller, action, details, success); // New system
```

---

This integration guide provides comprehensive examples for implementing audit logging across all modules of the Agrilends system. The audit logging system is designed to be both powerful and easy to use, providing valuable insights into system operations while maintaining security and compliance requirements.
