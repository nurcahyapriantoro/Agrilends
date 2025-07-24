// EXAMPLE: Integration dengan Liquidity Management Module
// File: src/liquidity_management.rs (contoh integrasi)

use ic_cdk::{caller, api::time, CallResult};
use ic_cdk_macros::{query, update};
use candid::{Nat, Principal};
use crate::types::*;
use crate::storage::*;
use crate::helpers::{is_admin, log_audit_action, get_canister_config};

// Import notification system
use crate::notification_system::{
    notify_liquidity_deposited,
    notify_liquidity_withdrawn,
    notify_investment_returns,
    create_batch_notifications,
    NotificationEvent,
    NotificationPriority,
};

/// Enhanced deposit liquidity with notifications
#[update]
pub async fn deposit_liquidity_with_notifications(amount: u64, tx_id: u64) -> Result<String, String> {
    let caller = caller();
    
    // Check if emergency pause is active
    if is_emergency_paused() {
        return Err("Pool operations are currently paused".to_string());
    }

    // Validate input parameters
    if amount == 0 {
        return Err("Amount must be greater than zero".to_string());
    }

    // Check minimum deposit amount (0.001 BTC = 100,000 satoshi)
    if amount < 100_000 {
        return Err("Amount must be at least 0.001 BTC (100,000 satoshi)".to_string());
    }

    // Check for idempotency - prevent duplicate transactions
    if is_transaction_processed(tx_id) {
        let processed_tx = get_processed_transaction(tx_id)
            .ok_or("Transaction processed but details not found")?;
        
        // Verify the processor is the same as current caller
        if processed_tx.processor != caller {
            return Err("Transaction ID already used by different account".to_string());
        }
        
        return Ok("Transaction already processed".to_string());
    }
    
    // Verify caller is registered as investor
    match get_user_by_principal(&caller) {
        Some(user) => {
            if !user.is_active {
                return Err("Account is not active".to_string());
            }
            if user.role != crate::user_management::Role::Investor {
                return Err("Only investors can deposit liquidity".to_string());
            }
        }
        None => return Err("User not registered. Please register first".to_string()),
    }
    
    // Rate limiting check
    check_rate_limit(&caller, 10)?; // Max 10 calls per minute
    
    // Prepare ckBTC transfer from caller to this canister
    let ckbtc_ledger = Principal::from_text(CKBTC_LEDGER_PRINCIPAL)
        .map_err(|_| "Invalid ckBTC ledger principal")?;
    
    let canister_account = Account {
        owner: ic_cdk::id(),
        subaccount: None,
    };
    
    let from_account = Account {
        owner: caller,
        subaccount: None,
    };
    
    let transfer_args = TransferFromArgs {
        spender_subaccount: None,
        from: from_account,
        to: canister_account,
        amount: Nat::from(amount),
        fee: None,
        memo: Some(format!("Liquidity deposit - tx_id: {}", tx_id).as_bytes().to_vec()),
        created_at_time: Some(time()),
    };
    
    // Execute the transfer
    let call_result: CallResult<(Result<Nat, TransferFromError>,)> = 
        ic_cdk::call(ckbtc_ledger, "icrc2_transfer_from", (transfer_args,)).await;
    
    match call_result {
        Ok((Ok(block_index),)) => {
            // Transfer successful, update pool state
            let block_idx = block_index.0.try_into().unwrap_or(0u64);
            
            // Update total liquidity
            let mut pool = get_liquidity_pool();
            pool.total_liquidity += amount;
            pool.available_liquidity += amount;
            pool.updated_at = time();
            
            // Update investor count if this is first deposit
            let is_first_deposit = !has_investor_deposited_before(caller);
            if is_first_deposit {
                pool.total_investors += 1;
            }
            
            store_liquidity_pool(pool)?;
            
            // Update investor balance
            let mut investor_balance = get_investor_balance_for_principal(caller).unwrap_or(InvestorBalance {
                investor: caller,
                balance: 0,
                deposits: Vec::new(),
                withdrawals: Vec::new(),
                total_deposited: 0,
                total_withdrawn: 0,
                first_deposit_at: time(),
                last_activity_at: time(),
            });
            
            // Add deposit record
            let deposit_record = DepositRecord {
                investor: caller,
                amount,
                ckbtc_block_index: block_idx,
                timestamp: time(),
            };
            
            investor_balance.balance += amount;
            investor_balance.total_deposited += amount;
            investor_balance.deposits.push(deposit_record);
            investor_balance.last_activity_at = time();
            
            // If this is the first deposit, set the first_deposit_at
            if is_first_deposit {
                investor_balance.first_deposit_at = time();
            }
            
            // Store updated investor balance
            store_investor_balance(investor_balance)?;
            
            // Mark transaction as processed
            mark_transaction_processed(tx_id)?;
            
            // Log audit action
            log_audit_action(
                caller,
                "LIQUIDITY_DEPOSIT".to_string(),
                format!("Deposited {} ckBTC satoshi, tx_id: {}, block: {}", amount, tx_id, block_idx),
                true,
            );
            
            // 🔔 KIRIM NOTIFIKASI: Likuiditas berhasil didepositkan
            match notify_liquidity_deposited(caller, amount) {
                Ok(notification_id) => {
                    log_audit_action(
                        caller,
                        "NOTIFICATION_SENT".to_string(),
                        format!("Liquidity deposit notification {} sent for amount {} satoshi", notification_id, amount),
                        true,
                    );
                }
                Err(e) => {
                    log_audit_action(
                        caller,
                        "NOTIFICATION_FAILED".to_string(),
                        format!("Failed to send liquidity deposit notification: {}", e),
                        false,
                    );
                }
            }
            
            // Check if this is a milestone deposit and send bonus notification
            if amount >= 10_000_000 { // 0.1 BTC or more
                let milestone_event = NotificationEvent::Custom {
                    event_type: "milestone_deposit".to_string(),
                    data: {
                        let mut data = std::collections::HashMap::new();
                        data.insert("amount".to_string(), amount.to_string());
                        data.insert("milestone".to_string(), "large_deposit".to_string());
                        data
                    },
                };
                
                match crate::notification_system::create_notification(
                    caller,
                    milestone_event,
                    Some(format!("Congratulations! You've made a significant deposit of {} satoshi. Thank you for supporting the Agrilends ecosystem!", amount)),
                    Some(NotificationPriority::Normal)
                ) {
                    Ok(notification_id) => {
                        log_audit_action(
                            caller,
                            "MILESTONE_NOTIFICATION_SENT".to_string(),
                            format!("Milestone deposit notification {} sent", notification_id),
                            true,
                        );
                    }
                    Err(e) => {
                        log_audit_action(
                            caller,
                            "MILESTONE_NOTIFICATION_FAILED".to_string(),
                            format!("Failed to send milestone notification: {}", e),
                            false,
                        );
                    }
                }
            }
            
            Ok("Deposit successful".to_string())
        }
        Ok((Err(transfer_error),)) => {
            let error_msg = format!("Transfer failed: {:?}", transfer_error);
            log_audit_action(
                caller,
                "LIQUIDITY_DEPOSIT_FAILED".to_string(),
                format!("Failed to deposit {} ckBTC satoshi: {}", amount, error_msg),
                false,
            );
            Err(error_msg)
        }
        Err(call_error) => {
            let error_msg = format!("Call to ckBTC ledger failed: {:?}", call_error);
            log_audit_action(
                caller,
                "LIQUIDITY_DEPOSIT_FAILED".to_string(),
                format!("Failed to deposit {} ckBTC satoshi: {}", amount, error_msg),
                false,
            );
            Err(error_msg)
        }
    }
}

/// Enhanced withdraw liquidity with notifications
#[update]
pub async fn withdraw_liquidity_with_notifications(amount: u64) -> Result<String, String> {
    let caller = caller();
    
    // Validate input
    if amount == 0 {
        return Err("Amount must be greater than zero".to_string());
    }
    
    // Get investor balance
    let investor_balance = get_investor_balance_for_principal(caller)?;
    
    // Check if investor has sufficient balance
    if investor_balance.balance < amount {
        return Err("Withdrawal amount exceeds your balance".to_string());
    }
    
    // Check if pool has sufficient available liquidity
    let pool = get_liquidity_pool();
    if pool.available_liquidity < amount {
        return Err("Withdrawal failed due to insufficient available liquidity".to_string());
    }
    
    // Prepare ckBTC transfer from canister to investor
    let ckbtc_ledger = Principal::from_text(CKBTC_LEDGER_PRINCIPAL)
        .map_err(|_| "Invalid ckBTC ledger principal")?;
    
    let investor_account = Account {
        owner: caller,
        subaccount: None,
    };
    
    let transfer_args = TransferArgs {
        from_subaccount: None,
        to: investor_account,
        amount: Nat::from(amount),
        fee: None,
        memo: Some(format!("Liquidity withdrawal for investor").as_bytes().to_vec()),
        created_at_time: Some(time()),
    };
    
    // Execute the transfer
    let call_result: CallResult<(Result<Nat, TransferError>,)> = 
        ic_cdk::call(ckbtc_ledger, "icrc1_transfer", (transfer_args,)).await;
    
    match call_result {
        Ok((Ok(block_index),)) => {
            // Transfer successful, update states
            let block_idx = block_index.0.try_into().unwrap_or(0u64);
            
            // Update pool state
            let mut pool = get_liquidity_pool();
            pool.total_liquidity -= amount;
            pool.available_liquidity -= amount;
            pool.updated_at = time();
            store_liquidity_pool(pool)?;
            
            // Update investor balance
            let mut investor_balance = investor_balance;
            investor_balance.balance -= amount;
            investor_balance.total_withdrawn += amount;
            investor_balance.last_activity_at = time();
            
            // Add withdrawal record
            let withdrawal_record = WithdrawalRecord {
                investor: caller,
                amount,
                ckbtc_block_index: block_idx,
                timestamp: time(),
            };
            investor_balance.withdrawals.push(withdrawal_record);
            
            // Store updated investor balance
            store_investor_balance(investor_balance)?;
            
            // Log audit action
            log_audit_action(
                caller,
                "LIQUIDITY_WITHDRAWAL".to_string(),
                format!("Withdrew {} ckBTC satoshi, block: {}", amount, block_idx),
                true,
            );
            
            // 🔔 KIRIM NOTIFIKASI: Likuiditas berhasil ditarik
            match notify_liquidity_withdrawn(caller, amount) {
                Ok(notification_id) => {
                    log_audit_action(
                        caller,
                        "NOTIFICATION_SENT".to_string(),
                        format!("Liquidity withdrawal notification {} sent for amount {} satoshi", notification_id, amount),
                        true,
                    );
                }
                Err(e) => {
                    log_audit_action(
                        caller,
                        "NOTIFICATION_FAILED".to_string(),
                        format!("Failed to send liquidity withdrawal notification: {}", e),
                        false,
                    );
                }
            }
            
            Ok("Withdrawal successful".to_string())
        }
        Ok((Err(transfer_error),)) => {
            let error_msg = format!("Transfer failed: {:?}", transfer_error);
            log_audit_action(
                caller,
                "LIQUIDITY_WITHDRAWAL_FAILED".to_string(),
                format!("Failed to withdraw {} ckBTC satoshi: {}", amount, error_msg),
                false,
            );
            Err(error_msg)
        }
        Err(call_error) => {
            let error_msg = format!("Call to ckBTC ledger failed: {:?}", call_error);
            log_audit_action(
                caller,
                "LIQUIDITY_WITHDRAWAL_FAILED".to_string(),
                format!("Failed to withdraw {} ckBTC satoshi: {}", amount, error_msg),
                false,
            );
            Err(error_msg)
        }
    }
}

/// Distribute investment returns with notifications
#[update]
pub async fn distribute_investment_returns(period: String) -> Result<String, String> {
    let caller = caller();
    
    // Only admin can distribute returns
    if !is_admin(&caller) {
        return Err("Unauthorized: Only admin can distribute investment returns".to_string());
    }
    
    // Get all investor balances
    let all_investors = get_all_investor_balances()?;
    let pool = get_liquidity_pool();
    
    // Calculate total returns to distribute (example: 5% of total pool value)
    let total_returns = (pool.total_liquidity * 5) / 100; // 5% returns
    let mut distributed_count = 0u64;
    let mut total_distributed = 0u64;
    let mut notification_ids = Vec::new();
    
    for investor_balance in all_investors {
        if investor_balance.balance > 0 {
            // Calculate proportional returns
            let investor_share = (investor_balance.balance * total_returns) / pool.total_liquidity;
            
            if investor_share > 0 {
                // Update investor balance
                let mut updated_balance = investor_balance.clone();
                updated_balance.balance += investor_share;
                updated_balance.last_activity_at = time();
                
                // Add return record
                let return_record = ReturnRecord {
                    investor: investor_balance.investor,
                    amount: investor_share,
                    period: period.clone(),
                    timestamp: time(),
                };
                
                // Store updated balance (this would need proper implementation)
                match store_investor_balance(updated_balance) {
                    Ok(_) => {
                        distributed_count += 1;
                        total_distributed += investor_share;
                        
                        // 🔔 KIRIM NOTIFIKASI: Investment returns
                        match notify_investment_returns(investor_balance.investor, investor_share, &period) {
                            Ok(notification_id) => {
                                notification_ids.push(notification_id);
                                log_audit_action(
                                    caller,
                                    "RETURN_NOTIFICATION_SENT".to_string(),
                                    format!("Investment return notification {} sent to {} for {} satoshi", 
                                           notification_id, investor_balance.investor.to_text(), investor_share),
                                    true,
                                );
                            }
                            Err(e) => {
                                log_audit_action(
                                    caller,
                                    "RETURN_NOTIFICATION_FAILED".to_string(),
                                    format!("Failed to send return notification to {}: {}", 
                                           investor_balance.investor.to_text(), e),
                                    false,
                                );
                            }
                        }
                    }
                    Err(e) => {
                        log_audit_action(
                            caller,
                            "RETURN_DISTRIBUTION_FAILED".to_string(),
                            format!("Failed to update balance for {}: {}", 
                                   investor_balance.investor.to_text(), e),
                            false,
                        );
                    }
                }
            }
        }
    }
    
    // Update pool state
    let mut updated_pool = get_liquidity_pool();
    updated_pool.total_liquidity += total_distributed;
    updated_pool.updated_at = time();
    store_liquidity_pool(updated_pool)?;
    
    // Log distribution summary
    log_audit_action(
        caller,
        "INVESTMENT_RETURNS_DISTRIBUTED".to_string(),
        format!("Distributed {} satoshi to {} investors for period {}", 
               total_distributed, distributed_count, period),
        true,
    );
    
    Ok(format!("Successfully distributed {} satoshi to {} investors. Sent {} notifications.", 
              total_distributed, distributed_count, notification_ids.len()))
}

/// Send pool update notifications to all investors
#[update]
pub async fn broadcast_pool_update(message: String, priority: String) -> Result<u64, String> {
    let caller = caller();
    
    // Only admin can broadcast
    if !is_admin(&caller) {
        return Err("Unauthorized: Only admin can broadcast pool updates".to_string());
    }
    
    // Get all investors
    let all_investors = get_all_investor_balances()?;
    let mut recipients = Vec::new();
    
    for investor_balance in all_investors {
        recipients.push(investor_balance.investor);
    }
    
    // Create custom pool update event
    let event = NotificationEvent::Custom {
        event_type: "pool_update".to_string(),
        data: {
            let mut data = std::collections::HashMap::new();
            data.insert("message".to_string(), message.clone());
            data.insert("pool_stats".to_string(), format!("{:?}", get_pool_stats()));
            data
        },
    };
    
    // Determine priority
    let notification_priority = match priority.as_str() {
        "high" => NotificationPriority::High,
        "critical" => NotificationPriority::Critical,
        "emergency" => NotificationPriority::Emergency,
        _ => NotificationPriority::Normal,
    };
    
    // Send batch notifications
    match create_batch_notifications(recipients, event, Some(message.clone()), Some(notification_priority)) {
        Ok(notification_ids) => {
            log_audit_action(
                caller,
                "POOL_UPDATE_BROADCAST".to_string(),
                format!("Broadcast pool update to {} investors: {}", notification_ids.len(), message),
                true,
            );
            
            Ok(notification_ids.len() as u64)
        }
        Err(e) => {
            log_audit_action(
                caller,
                "POOL_UPDATE_BROADCAST_FAILED".to_string(),
                format!("Failed to broadcast pool update: {}", e),
                false,
            );
            
            Err(e)
        }
    }
}

/// Send emergency pool notifications
#[update]
pub async fn send_emergency_pool_notification(emergency_type: String, details: String) -> Result<u64, String> {
    let caller = caller();
    
    // Only admin can send emergency notifications
    if !is_admin(&caller) {
        return Err("Unauthorized: Only admin can send emergency notifications".to_string());
    }
    
    // Get all users (both farmers and investors)
    let all_users = crate::user_management::get_all_users();
    let mut recipients = Vec::new();
    
    for user in all_users {
        recipients.push(user.id);
    }
    
    // Create emergency event
    let event = match emergency_type.as_str() {
        "emergency_stop" => NotificationEvent::EmergencyStop { reason: details.clone() },
        "system_resumed" => NotificationEvent::SystemResumed,
        _ => NotificationEvent::Custom {
            event_type: format!("emergency_{}", emergency_type),
            data: {
                let mut data = std::collections::HashMap::new();
                data.insert("details".to_string(), details.clone());
                data.insert("timestamp".to_string(), time().to_string());
                data
            },
        },
    };
    
    // Send emergency notifications
    match create_batch_notifications(recipients, event, Some(details.clone()), Some(NotificationPriority::Emergency)) {
        Ok(notification_ids) => {
            log_audit_action(
                caller,
                "EMERGENCY_NOTIFICATION_BROADCAST".to_string(),
                format!("Broadcast emergency notification ({}) to {} users: {}", 
                       emergency_type, notification_ids.len(), details),
                true,
            );
            
            Ok(notification_ids.len() as u64)
        }
        Err(e) => {
            log_audit_action(
                caller,
                "EMERGENCY_NOTIFICATION_FAILED".to_string(),
                format!("Failed to broadcast emergency notification: {}", e),
                false,
            );
            
            Err(e)
        }
    }
}

/// Process loan repayment with enhanced notifications
#[update]
pub fn process_loan_repayment_with_notifications(loan_id: u64, amount: u64) -> Result<String, String> {
    let caller = caller();
    
    // Only loan management canister can process repayments
    if !is_loan_manager(&caller) {
        return Err("Unauthorized: Only loan manager can process repayments".to_string());
    }
    
    // Update pool state
    let mut pool = get_liquidity_pool();
    let old_available = pool.available_liquidity;
    
    pool.available_liquidity += amount;
    pool.total_repaid += amount;
    pool.updated_at = time();
    store_liquidity_pool(pool.clone())?;
    
    // Log audit action
    log_audit_action(
        caller,
        "LOAN_REPAYMENT_PROCESSED".to_string(),
        format!("Processed repayment of {} ckBTC satoshi for loan #{}", amount, loan_id),
        true,
    );
    
    // Check if this repayment significantly improves pool liquidity
    let liquidity_improvement = ((pool.available_liquidity - old_available) as f64 / old_available as f64) * 100.0;
    
    if liquidity_improvement > 10.0 { // If liquidity improved by more than 10%
        // Notify all investors about improved liquidity
        let investors = get_all_investor_balances().unwrap_or_default();
        
        for investor_balance in investors {
            let event = NotificationEvent::Custom {
                event_type: "liquidity_improved".to_string(),
                data: {
                    let mut data = std::collections::HashMap::new();
                    data.insert("improvement_percentage".to_string(), format!("{:.2}%", liquidity_improvement));
                    data.insert("new_available_liquidity".to_string(), pool.available_liquidity.to_string());
                    data.insert("repayment_amount".to_string(), amount.to_string());
                    data
                },
            };
            
            match crate::notification_system::create_notification(
                investor_balance.investor,
                event,
                Some(format!("Good news! A loan repayment of {} satoshi has improved pool liquidity by {:.2}%. More opportunities available!", amount, liquidity_improvement)),
                Some(NotificationPriority::Normal)
            ) {
                Ok(notification_id) => {
                    log_audit_action(
                        caller,
                        "LIQUIDITY_IMPROVEMENT_NOTIFICATION_SENT".to_string(),
                        format!("Liquidity improvement notification {} sent to {}", notification_id, investor_balance.investor.to_text()),
                        true,
                    );
                }
                Err(e) => {
                    log_audit_action(
                        caller,
                        "LIQUIDITY_IMPROVEMENT_NOTIFICATION_FAILED".to_string(),
                        format!("Failed to send liquidity improvement notification: {}", e),
                        false,
                    );
                }
            }
        }
    }
    
    Ok("Repayment processed successfully".to_string())
}

// Helper functions for return distribution
#[derive(Debug, Clone)]
struct ReturnRecord {
    investor: Principal,
    amount: u64,
    period: String,
    timestamp: u64,
}

fn get_all_investor_balances() -> Result<Vec<InvestorBalance>, String> {
    // This would fetch all investor balances from storage
    // Implementation depends on storage structure
    Ok(Vec::new()) // Placeholder
}

/// Send weekly/monthly pool performance reports
pub async fn send_pool_performance_report(period: String) -> Result<u64, String> {
    let caller = caller();
    
    // Only admin or automated system can send reports
    if !is_admin(&caller) && !crate::helpers::is_maintenance_canister(&caller) {
        return Err("Unauthorized: Only admin or maintenance canister can send reports".to_string());
    }
    
    let pool_stats = get_pool_stats();
    let all_investors = get_all_investor_balances().unwrap_or_default();
    
    let mut notification_count = 0u64;
    
    for investor_balance in all_investors {
        let event = NotificationEvent::Custom {
            event_type: "pool_performance_report".to_string(),
            data: {
                let mut data = std::collections::HashMap::new();
                data.insert("period".to_string(), period.clone());
                data.insert("total_liquidity".to_string(), pool_stats.total_liquidity.to_string());
                data.insert("available_liquidity".to_string(), pool_stats.available_liquidity.to_string());
                data.insert("utilization_rate".to_string(), pool_stats.utilization_rate.to_string());
                data.insert("apy".to_string(), pool_stats.apy.to_string());
                data.insert("your_balance".to_string(), investor_balance.balance.to_string());
                data
            },
        };
        
        let performance_message = format!(
            "Pool Performance Report for {}: Total Liquidity: {} satoshi, APY: {}%, Your Balance: {} satoshi. Pool utilization: {}%",
            period, pool_stats.total_liquidity, pool_stats.apy, investor_balance.balance, pool_stats.utilization_rate
        );
        
        match crate::notification_system::create_notification(
            investor_balance.investor,
            event,
            Some(performance_message),
            Some(NotificationPriority::Low)
        ) {
            Ok(notification_id) => {
                notification_count += 1;
                log_audit_action(
                    caller,
                    "PERFORMANCE_REPORT_SENT".to_string(),
                    format!("Performance report notification {} sent to {}", notification_id, investor_balance.investor.to_text()),
                    true,
                );
            }
            Err(e) => {
                log_audit_action(
                    caller,
                    "PERFORMANCE_REPORT_FAILED".to_string(),
                    format!("Failed to send performance report to {}: {}", investor_balance.investor.to_text(), e),
                    false,
                );
            }
        }
    }
    
    log_audit_action(
        caller,
        "POOL_PERFORMANCE_REPORT_COMPLETED".to_string(),
        format!("Sent {} performance reports for period {}", notification_count, period),
        true,
    );
    
    Ok(notification_count)
}
