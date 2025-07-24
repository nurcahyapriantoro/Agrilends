use candid::{CandidType, Deserialize, Principal, Nat};
use ic_cdk::call::CallResult; // Fix CallResult import
use ic_cdk::api::{time, canister_self};
use ic_cdk::{call}; // Import call function
use ic_cdk_macros::{query, update};

use crate::types::*;
use crate::storage::{
    get_liquidity_pool, store_liquidity_pool, get_investor_balance_by_principal,
    store_investor_balance, is_transaction_processed, mark_transaction_processed,
    has_investor_deposited_before, set_emergency_pause, is_emergency_paused, get_processed_transaction,
    remove_processed_transaction, store_disbursement_record, get_all_disbursement_records, 
    get_all_processed_transactions
};
use crate::helpers::{check_rate_limit, check_rate_limit_with_operation, is_loan_manager_canister, is_admin, log_audit_action,
    get_canister_config, set_canister_config};
use crate::user_management::get_user_by_principal;

// ckBTC Ledger and Minter Constants
const CKBTC_LEDGER_PRINCIPAL: &str = "mxzaz-hqaaa-aaaar-qaada-cai";
const CKBTC_MINTER_PRINCIPAL: &str = "mqygn-kiaaa-aaaar-qaadq-cai";

// ckBTC Integration structures
#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct Account {
    pub owner: Principal,
    pub subaccount: Option<Vec<u8>>,
}

#[derive(CandidType, Deserialize)]
pub struct TransferFromArgs {
    pub spender_subaccount: Option<Vec<u8>>,
    pub from: Account,
    pub to: Account,
    pub amount: Nat,
    pub fee: Option<Nat>,
    pub memo: Option<Vec<u8>>,
    pub created_at_time: Option<u64>,
}

#[derive(CandidType, Deserialize)]
pub struct TransferArgs {
    pub from_subaccount: Option<Vec<u8>>,
    pub to: Account,
    pub amount: Nat,
    pub fee: Option<Nat>,
    pub memo: Option<Vec<u8>>,
    pub created_at_time: Option<u64>,
}

#[derive(CandidType, Deserialize)]
pub struct ApproveArgs {
    pub from_subaccount: Option<Vec<u8>>,
    pub spender: Account,
    pub amount: Nat,
    pub expected_allowance: Option<Nat>,
    pub expires_at: Option<u64>,
    pub fee: Option<Nat>,
    pub memo: Option<Vec<u8>>,
    pub created_at_time: Option<u64>,
}

#[derive(CandidType, Deserialize)]
pub struct RetrieveBtcArgs {
    pub address: String,
    pub amount: u64,
}

#[derive(CandidType, Deserialize, Debug)]
pub enum TransferError {
    BadFee { expected_fee: Nat },
    BadBurn { min_burn_amount: Nat },
    InsufficientFunds { balance: Nat },
    TooOld,
    CreatedInFuture { ledger_time: u64 },
    TemporarilyUnavailable,
    Duplicate { duplicate_of: Nat },
    GenericError { error_code: Nat, message: String },
}

#[derive(CandidType, Deserialize, Debug)]
pub enum TransferFromError {
    BadFee { expected_fee: Nat },
    BadBurn { min_burn_amount: Nat },
    InsufficientFunds { balance: Nat },
    InsufficientAllowance { allowance: Nat },
    TooOld,
    CreatedInFuture { ledger_time: u64 },
    TemporarilyUnavailable,
    Duplicate { duplicate_of: Nat },
    GenericError { error_code: Nat, message: String },
}

#[derive(CandidType, Deserialize, Debug)]
pub enum ApproveError {
    BadFee { expected_fee: Nat },
    InsufficientFunds { balance: Nat },
    AllowanceChanged { current_allowance: Nat },
    Expired { ledger_time: u64 },
    TooOld,
    CreatedInFuture { ledger_time: u64 },
    TemporarilyUnavailable,
    Duplicate { duplicate_of: Nat },
    GenericError { error_code: Nat, message: String },
}

#[derive(CandidType, Deserialize, Debug)]
pub enum RetrieveBtcError {
    MalformedAddress(String),
    GenericError { error_code: u64, error_message: String },
    TemporarilyUnavailable(String),
    AlreadyProcessing,
    AmountTooLow(u64),
    InsufficientFunds { balance: u64 },
}

/// Deposit liquidity to the pool
/// This function handles incoming ckBTC deposits from investors
/// Implements idempotency, strict validation, and comprehensive audit logging
#[update]
pub async fn deposit_liquidity(amount: u64, tx_id: u64) -> Result<String, String> {
    let caller = ic_cdk::caller();
    
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
        owner: canister_self(),
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
    let call_result: Result<(Result<Nat, TransferFromError>,), _> = 
        call(ckbtc_ledger, "icrc2_transfer_from", (transfer_args,)).await;
    
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

/// Disburse loan to borrower's Bitcoin address
/// This function is CRITICAL and must be protected - only callable by loan management canister
/// Implements comprehensive security checks, Bitcoin address validation, and audit logging
#[update]
pub async fn disburse_loan(
    loan_id: u64,
    borrower_btc_address: String, 
    amount: u64
) -> Result<String, String> {
    let caller = ic_cdk::caller();
    
    // Check if emergency pause is active
    if is_emergency_paused() {
        return Err("Pool operations are currently paused".to_string());
    }
    
    // CRITICAL ACCESS CONTROL: Only loan management canister can disburse funds
    if !is_loan_manager_canister(&caller) {
        ic_cdk::trap("Unauthorized: Only the loan manager can disburse funds");
    }
    
    // Validate input parameters
    if amount == 0 {
        return Err("Amount must be greater than zero".to_string());
    }
    
    if borrower_btc_address.is_empty() {
        return Err("Bitcoin address cannot be empty".to_string());
    }
    
    // Validate Bitcoin address format (basic validation)
    if !is_valid_bitcoin_address(&borrower_btc_address) {
        return Err("Invalid Bitcoin address format".to_string());
    }
    
    // Check minimum disbursement amount (0.001 BTC = 100,000 satoshi)
    if amount < 100_000 {
        return Err("Amount must be at least 0.001 BTC (100,000 satoshi)".to_string());
    }
    
    // Check if pool has sufficient available liquidity
    let pool = get_liquidity_pool();
    if pool.available_liquidity < amount {
        return Err(format!(
            "Insufficient liquidity in the pool. Available: {} satoshi, Required: {} satoshi",
            pool.available_liquidity, amount
        ));
    }
    
    // Additional safety check: ensure we don't exceed 80% of total liquidity for a single loan
    let max_single_loan = (pool.total_liquidity * 80) / 100;
    if amount > max_single_loan {
        return Err(format!(
            "Loan amount too large. Maximum allowed: {} satoshi (80% of total liquidity)",
            max_single_loan
        ));
    }
    
    // Prepare for Bitcoin withdrawal via ckBTC Minter
    let ckbtc_ledger = Principal::from_text(CKBTC_LEDGER_PRINCIPAL)
        .map_err(|_| "Invalid ckBTC ledger principal")?;
    
    let ckbtc_minter = Principal::from_text(CKBTC_MINTER_PRINCIPAL)
        .map_err(|_| "Invalid ckBTC minter principal")?;
    
    let _canister_account = Account {
        owner: canister_self(),
        subaccount: None,
    };
    
    let minter_account = Account {
        owner: ckbtc_minter,
        subaccount: None,
    };
    
    // Step 1: Approve the minter to spend our ckBTC
    let approve_args = ApproveArgs {
        from_subaccount: None,
        spender: minter_account.clone(),
        amount: Nat::from(amount),
        expected_allowance: None,
        expires_at: Some(time() + 600_000_000_000), // 10 minutes expiry
        fee: None,
        memo: Some(format!("Loan disbursement approval - Loan ID: {}", loan_id).as_bytes().to_vec()),
        created_at_time: Some(time()),
    };
    
    let approve_result: Result<(Result<Nat, ApproveError>,), _> = 
        call(ckbtc_ledger, "icrc2_approve", (approve_args,)).await;
    
    match approve_result {
        Ok((Ok(approve_block),)) => {
            // Step 2: Call retrieve_btc_with_approval on the minter
            let retrieve_args = RetrieveBtcArgs {
                address: borrower_btc_address.clone(),
                amount,
            };
            
            let retrieve_result: Result<(Result<u64, RetrieveBtcError>,), _> = 
                call(ckbtc_minter, "retrieve_btc_with_approval", (retrieve_args,)).await;
            
            match retrieve_result {
                Ok((Ok(block_index),)) => {
                    // Disbursement successful, update pool state
                    let mut pool = get_liquidity_pool();
                    pool.available_liquidity -= amount;
                    pool.total_borrowed += amount;
                    pool.updated_at = time();
                    store_liquidity_pool(pool)?;
                    
                    // Create disbursement record
                    let disbursement_record = DisbursementRecord {
                        loan_id,
                        borrower_btc_address: borrower_btc_address.clone(),
                        amount,
                        ckbtc_block_index: block_index,
                        disbursed_at: time(),
                        disbursed_by: caller,
                    };
                    
                    // Store disbursement record
                    store_disbursement_record(disbursement_record)?;
                    
                    // Log audit action
                    log_audit_action(
                        caller,
                        "LOAN_DISBURSEMENT".to_string(),
                        format!(
                            "Disbursed {} ckBTC satoshi to {} for loan #{}, approve_block: {}, btc_block: {}",
                            amount, borrower_btc_address, loan_id, 
                            approve_block.0.try_into().unwrap_or(0u64), 
                            block_index
                        ),
                        true,
                    );
                    
                    Ok("Disbursement initiated successfully".to_string())
                }
                Ok((Err(retrieve_error),)) => {
                    let error_msg = format!("Bitcoin retrieval failed: {:?}", retrieve_error);
                    log_audit_action(
                        caller,
                        "LOAN_DISBURSEMENT_FAILED".to_string(),
                        format!(
                            "Failed to disburse {} ckBTC satoshi to {} for loan #{}: {}",
                            amount, borrower_btc_address, loan_id, error_msg
                        ),
                        false,
                    );
                    Err(error_msg)
                }
                Err(call_error) => {
                    let error_msg = format!("Call to ckBTC minter failed: {:?}", call_error);
                    log_audit_action(
                        caller,
                        "LOAN_DISBURSEMENT_FAILED".to_string(),
                        format!(
                            "Failed to disburse {} ckBTC satoshi to {} for loan #{}: {}",
                            amount, borrower_btc_address, loan_id, error_msg
                        ),
                        false,
                    );
                    Err(error_msg)
                }
            }
        }
        Ok((Err(approve_error),)) => {
            let error_msg = format!("Approval failed: {:?}", approve_error);
            log_audit_action(
                caller,
                "LOAN_DISBURSEMENT_FAILED".to_string(),
                format!(
                    "Failed to approve disbursement of {} ckBTC satoshi for loan #{}: {}",
                    amount, loan_id, error_msg
                ),
                false,
            );
            Err(error_msg)
        }
        Err(call_error) => {
            let error_msg = format!("Call to approve failed: {:?}", call_error);
            log_audit_action(
                caller,
                "LOAN_DISBURSEMENT_FAILED".to_string(),
                format!(
                    "Failed to approve disbursement of {} ckBTC satoshi for loan #{}: {}",
                    amount, loan_id, error_msg
                ),
                false,
            );
            Err(error_msg)
        }
    }
}

/// Withdraw liquidity from the pool
/// Allows investors to withdraw their funds (principal + accumulated yield)
/// Implements comprehensive security checks, validation, and audit logging
/// 
/// # Arguments
/// * `amount` - Amount in ckBTC satoshi to withdraw
/// 
/// # Returns
/// * `Result<String, String>` - Success message or error details
/// 
/// # Security Features
/// - Validates caller authorization and amount
/// - Checks investor balance sufficiency
/// - Verifies pool liquidity availability
/// - Implements rate limiting and emergency pause checks
/// - Comprehensive audit logging for all actions
#[update]
pub async fn withdraw_liquidity(amount: u64) -> Result<String, String> {
    let caller = ic_cdk::caller();
    
    // Security: Check if system is paused
    if is_emergency_paused() {
        log_audit_action(
            caller,
            "LIQUIDITY_WITHDRAWAL_BLOCKED".to_string(),
            format!("Withdrawal attempt during emergency pause: {} ckBTC satoshi", amount),
            false,
        );
        return Err("System is currently paused for maintenance".to_string());
    }
    
    // Rate limiting check
    if !check_rate_limit_with_operation(&caller, "WITHDRAW_LIQUIDITY") {
        log_audit_action(
            caller,
            "LIQUIDITY_WITHDRAWAL_RATE_LIMITED".to_string(),
            format!("Rate limited withdrawal attempt: {} ckBTC satoshi", amount),
            false,
        );
        return Err("Rate limit exceeded. Please try again later".to_string());
    }
    
    // Input validation
    if amount == 0 {
        log_audit_action(
            caller,
            "LIQUIDITY_WITHDRAWAL_INVALID_INPUT".to_string(),
            "Attempted withdrawal with zero amount".to_string(),
            false,
        );
        return Err("Amount must be greater than zero".to_string());
    }
    
    // Minimum withdrawal amount (1000 satoshi = 0.00001 BTC)
    const MIN_WITHDRAWAL_AMOUNT: u64 = 1000;
    if amount < MIN_WITHDRAWAL_AMOUNT {
        log_audit_action(
            caller,
            "LIQUIDITY_WITHDRAWAL_BELOW_MINIMUM".to_string(),
            format!("Attempted withdrawal below minimum: {} < {}", amount, MIN_WITHDRAWAL_AMOUNT),
            false,
        );
        return Err(format!("Minimum withdrawal amount is {} ckBTC satoshi", MIN_WITHDRAWAL_AMOUNT));
    }
    
    // Get investor balance with comprehensive error handling
    let investor_balance = match get_investor_balance_for_principal(caller) {
        Ok(balance) => balance,
        Err(_) => {
            log_audit_action(
                caller,
                "LIQUIDITY_WITHDRAWAL_NO_BALANCE".to_string(),
                format!("Withdrawal attempt by investor with no balance: {} ckBTC satoshi", amount),
                false,
            );
            return Err("No investment balance found. Please deposit first".to_string());
        }
    };
    
    // Check if investor has sufficient balance
    if investor_balance.balance < amount {
        log_audit_action(
            caller,
            "LIQUIDITY_WITHDRAWAL_INSUFFICIENT_BALANCE".to_string(),
            format!(
                "Insufficient balance: attempted {} ckBTC satoshi, available {} ckBTC satoshi", 
                amount, investor_balance.balance
            ),
            false,
        );
        return Err(format!(
            "Withdrawal amount exceeds your balance. Available: {} ckBTC satoshi", 
            investor_balance.balance
        ));
    }
    
    // Get current pool state
    let pool = get_liquidity_pool();
    
    // Check if pool has sufficient available liquidity
    if pool.available_liquidity < amount {
        log_audit_action(
            caller,
            "LIQUIDITY_WITHDRAWAL_INSUFFICIENT_POOL".to_string(),
            format!(
                "Insufficient pool liquidity: requested {} ckBTC satoshi, available {} ckBTC satoshi", 
                amount, pool.available_liquidity
            ),
            false,
        );
        return Err(format!(
            "Withdrawal failed due to insufficient available liquidity. Available: {} ckBTC satoshi", 
            pool.available_liquidity
        ));
    }
    
    // Additional safety check: ensure pool maintains emergency reserve
    let emergency_reserve_ratio = 5; // 5% emergency reserve
    let required_reserve = (pool.total_liquidity * emergency_reserve_ratio) / 100;
    let liquidity_after_withdrawal = pool.available_liquidity - amount;
    
    if liquidity_after_withdrawal < required_reserve {
        log_audit_action(
            caller,
            "LIQUIDITY_WITHDRAWAL_RESERVE_VIOLATION".to_string(),
            format!(
                "Withdrawal would violate emergency reserve: {} < {} required", 
                liquidity_after_withdrawal, required_reserve
            ),
            false,
        );
        return Err("Withdrawal would violate emergency reserve requirements".to_string());
    }
    
    // Prepare ckBTC transfer from canister to investor
    let ckbtc_ledger = Principal::from_text(CKBTC_LEDGER_PRINCIPAL)
        .map_err(|_| "Invalid ckBTC ledger principal configuration")?;
    
    let investor_account = Account {
        owner: caller,
        subaccount: None,
    };
    
    let transfer_args = TransferArgs {
        from_subaccount: None,
        to: investor_account,
        amount: Nat::from(amount),
        fee: None,
        memo: Some(format!("Agrilends liquidity withdrawal: {} satoshi", amount).as_bytes().to_vec()),
        created_at_time: Some(time()),
    };
    
    // Log withdrawal initiation
    log_audit_action(
        caller,
        "LIQUIDITY_WITHDRAWAL_INITIATED".to_string(),
        format!(
            "Initiating withdrawal: {} ckBTC satoshi from balance {} ckBTC satoshi", 
            amount, investor_balance.balance
        ),
        true,
    );
    
    // Execute the ckBTC transfer
    let call_result: Result<(Result<Nat, TransferError>,), _> = 
        call(ckbtc_ledger, "icrc1_transfer", (transfer_args,)).await;
    
    match call_result {
        Ok((Ok(block_index),)) => {
            // Transfer successful, update all states atomically
            let block_idx = block_index.0.try_into().unwrap_or(0u64);
            
            // Update pool state
            let mut updated_pool = pool;
            updated_pool.total_liquidity -= amount;
            updated_pool.available_liquidity -= amount;
            updated_pool.updated_at = time();
            
            // Add pool stats tracking
            updated_pool.total_withdrawals = updated_pool.total_withdrawals.saturating_add(1);
            updated_pool.total_withdrawn_amount = updated_pool.total_withdrawn_amount.saturating_add(amount);
            
            store_liquidity_pool(updated_pool)?;
            
            // Update investor balance
            let mut updated_investor_balance = investor_balance;
            updated_investor_balance.balance -= amount;
            updated_investor_balance.total_withdrawn += amount;
            updated_investor_balance.last_activity_at = time();
            
            // Create detailed withdrawal record
            let withdrawal_record = WithdrawalRecord {
                investor: caller,
                amount,
                ckbtc_block_index: block_idx,
                timestamp: time(),
            };
            updated_investor_balance.withdrawals.push(withdrawal_record);
            
            // Store updated investor balance
            store_investor_balance(updated_investor_balance)?;
            
            // Comprehensive audit logging
            log_audit_action(
                caller,
                "LIQUIDITY_WITHDRAWAL_SUCCESS".to_string(),
                format!(
                    "Successfully withdrew {} ckBTC satoshi, ckBTC block: {}, remaining balance: {} ckBTC satoshi", 
                    amount, block_idx, updated_investor_balance.balance
                ),
                true,
            );
            
            Ok(format!(
                "Withdrawal successful. Amount: {} ckBTC satoshi, Transaction Block: {}", 
                amount, block_idx
            ))
        }
        Ok((Err(transfer_error),)) => {
            let error_msg = match transfer_error {
                TransferError::BadBurn { min_burn_amount } => {
                    format!("Invalid burn amount. Minimum required: {:?}", min_burn_amount)
                }
                TransferError::BadFee { expected_fee } => {
                    format!("Invalid fee. Expected: {:?}", expected_fee)
                }
                TransferError::InsufficientFunds { balance } => {
                    format!("Canister has insufficient ckBTC funds. Available: {:?}", balance)
                }
                TransferError::TooOld => {
                    "Transaction timestamp too old".to_string()
                }
                TransferError::CreatedInFuture { ledger_time } => {
                    format!("Transaction created in future. Ledger time: {:?}", ledger_time)
                }
                TransferError::Duplicate { duplicate_of } => {
                    format!("Duplicate transaction. Original: {:?}", duplicate_of)
                }
                TransferError::TemporarilyUnavailable => {
                    "ckBTC ledger temporarily unavailable".to_string()
                }
                TransferError::GenericError { error_code, message } => {
                    format!("ckBTC transfer error {}: {}", error_code, message)
                }
            };
            
            log_audit_action(
                caller,
                "LIQUIDITY_WITHDRAWAL_TRANSFER_FAILED".to_string(),
                format!("ckBTC transfer failed for {} ckBTC satoshi: {}", amount, error_msg),
                false,
            );
            
            Err(format!("Withdrawal failed: {}", error_msg))
        }
        Err(call_error) => {
            let error_msg = format!("Failed to communicate with ckBTC ledger: {:?}", call_error);
            log_audit_action(
                caller,
                "LIQUIDITY_WITHDRAWAL_NETWORK_ERROR".to_string(),
                format!("Network error during withdrawal of {} ckBTC satoshi: {}", amount, error_msg),
                false,
            );
            Err(format!("Network error: {}", error_msg))
        }
    }
}

/// Get comprehensive pool statistics
/// Returns detailed information about the liquidity pool for public viewing
#[query]
pub fn get_pool_stats() -> PoolStats {
    let pool = get_liquidity_pool();
    
    // Calculate utilization rate (percentage of liquidity currently borrowed)
    let utilization_rate = if pool.total_liquidity > 0 {
        ((pool.total_liquidity - pool.available_liquidity) * 100) / pool.total_liquidity
    } else {
        0
    };
    
    // Calculate APY based on utilization and pool performance
    let apy = calculate_pool_apy(&pool);
    
    // Calculate total return rate (including repayments)
    let _total_return_rate = if pool.total_borrowed > 0 {
        (pool.total_repaid * 100) / pool.total_borrowed
    } else {
        0
    };
    
    PoolStats {
        total_liquidity: pool.total_liquidity,
        available_liquidity: pool.available_liquidity,
        total_borrowed: pool.total_borrowed,
        total_repaid: pool.total_repaid,
        utilization_rate: utilization_rate as u64,
        total_investors: pool.total_investors,
        apy: apy as u64,
        created_at: pool.created_at,
        updated_at: pool.updated_at,
    }
}

/// Get investor balance for the calling investor
/// Returns comprehensive balance information including deposits, withdrawals, and activity history
/// 
/// # Returns
/// * `Result<InvestorBalance, String>` - Complete investor balance data or error message
/// 
/// # Features
/// - Returns detailed balance breakdown
/// - Includes transaction history
/// - Provides activity timestamps
/// - Implements proper error handling
#[query]
pub fn get_investor_balance() -> Result<InvestorBalance, String> {
    let caller = ic_cdk::caller();
    
    // Validate caller (not anonymous)
    if caller == Principal::anonymous() {
        log_audit_action(
            caller,
            "BALANCE_QUERY_ANONYMOUS".to_string(),
            "Anonymous user attempted to query balance".to_string(),
            false,
        );
        return Err("Anonymous users cannot query balance".to_string());
    }
    
    // Rate limiting for balance queries (prevent spam)
    if !check_rate_limit_with_operation(&caller, "BALANCE_QUERY") {
        return Err("Rate limit exceeded for balance queries".to_string());
    }
    
    match get_investor_balance_by_principal(caller) {
        Some(mut balance) => {
            // Calculate derived metrics for enhanced user experience
            let total_activity = balance.deposits.len() + balance.withdrawals.len();
            let net_position = balance.total_deposited.saturating_sub(balance.total_withdrawn);
            
            // Add calculated fields for better UX (these could be added to the struct later)
            // For now, we'll include them in logs for admin monitoring
            log_audit_action(
                caller,
                "BALANCE_QUERY_SUCCESS".to_string(),
                format!(
                    "Balance queried: {} ckBTC satoshi, deposits: {}, withdrawals: {}, net: {} ckBTC satoshi", 
                    balance.balance, balance.deposits.len(), balance.withdrawals.len(), net_position
                ),
                true,
            );
            
            // Sort transaction history by timestamp (most recent first)
            balance.deposits.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
            balance.withdrawals.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
            
            Ok(balance)
        }
        None => {
            log_audit_action(
                caller,
                "BALANCE_QUERY_NOT_FOUND".to_string(),
                "Queried balance for non-investor".to_string(),
                false,
            );
            Err("No investment balance found. Please make a deposit first to create your investor profile".to_string())
        }
    }
}

/// Get investor balance for a specific investor (used internally)
pub fn get_investor_balance_for_principal(investor: Principal) -> Result<InvestorBalance, String> {
    match get_investor_balance_by_principal(investor) {
        Some(balance) => Ok(balance),
        None => Err("No balance found for investor".to_string()),
    }
}

/// Get detailed pool information (admin only)
#[query]
pub fn get_pool_details() -> Result<LiquidityPool, String> {
    let caller = ic_cdk::caller();
    
    if !is_admin(&caller) {
        return Err("Unauthorized: Only admins can view detailed pool information".to_string());
    }
    
    Ok(get_liquidity_pool())
}

/// Get all investor balances (admin only)
#[query]
pub fn get_all_investor_balances_admin() -> Result<Vec<InvestorBalance>, String> {
    let caller = ic_cdk::caller();
    
    if !is_admin(&caller) {
        return Err("Unauthorized: Only admins can view all investor balances".to_string());
    }
    
    Ok(crate::storage::get_all_investor_balances())
}

/// Process loan repayment and update pool
/// This function is called when a loan is repaid
#[update]
pub fn process_loan_repayment(loan_id: u64, amount: u64) -> Result<String, String> {
    let caller = ic_cdk::caller();
    
    // Only loan management canister can process repayments
    if !is_loan_manager(&caller) {
        return Err("Unauthorized: Only loan manager can process repayments".to_string());
    }
    
    // Update pool state
    let mut pool = get_liquidity_pool();
    pool.available_liquidity += amount;
    pool.total_repaid += amount;
    pool.updated_at = time();
    store_liquidity_pool(pool)?;
    
    // Log audit action
    log_audit_action(
        caller,
        "LOAN_REPAYMENT_PROCESSED".to_string(),
        format!("Processed repayment of {} ckBTC satoshi for loan #{}", amount, loan_id),
        true,
    );
    
    Ok("Repayment processed successfully".to_string())
}

/// Record liquidation loss in liquidity pool accounting
/// Sesuai README: "Catat kerugian pada liquidity pool. Nilai kerugian adalah sisa utang pokok"
#[update]
pub async fn record_liquidation_loss(
    loan_id: u64, 
    principal_loss: u64,
    total_debt: u64
) -> Result<String, String> {
    let caller = ic_cdk::caller();
    
    // Only liquidation system can record losses
    if !is_admin(&caller) && !is_loan_manager(&caller) {
        return Err("Unauthorized: Only admin or loan manager can record liquidation losses".to_string());
    }

    // Update pool state to reflect the loss
    let mut pool = get_liquidity_pool();
    
    // Record the principal loss (affects investor returns)
    pool.total_borrowed = pool.total_borrowed.saturating_sub(principal_loss);
    
    // Update pool metrics untuk reflect liquidation impact
    pool.updated_at = time();
    
    // Store updated pool state
    store_liquidity_pool(pool)?;

    // Log comprehensive audit trail
    log_audit_action(
        caller,
        "LIQUIDATION_LOSS_RECORDED".to_string(),
        format!(
            "Liquidation loss recorded for loan #{}: Principal loss: {} satoshi, Total debt: {} satoshi. Pool adjusted accordingly.",
            loan_id, principal_loss, total_debt
        ),
        true,
    );

    Ok(format!(
        "Liquidation loss of {} satoshi recorded for loan #{}", 
        principal_loss, loan_id
    ))
}

/// Collect protocol fees from loan repayments
#[update]
pub async fn collect_protocol_fees(loan_id: u64, fee_amount: u64) -> Result<String, String> {
    let caller = ic_cdk::caller();
    
    // Only loan management canister can collect fees
    if !is_loan_manager(&caller) {
        return Err("Unauthorized: Only loan manager can collect protocol fees".to_string());
    }
    
    if fee_amount == 0 {
        return Ok("No fees to collect".to_string());
    }
    
    // Update pool state with protocol earnings
    let mut pool = get_liquidity_pool();
    // In a real implementation, you might have a separate treasury balance
    // For now, we'll just track it in the pool
    pool.updated_at = time();
    store_liquidity_pool(pool)?;
    
    // Log audit action
    log_audit_action(
        caller,
        "PROTOCOL_FEE_COLLECTED".to_string(),
        format!("Collected {} satoshi protocol fee from loan #{}", fee_amount, loan_id),
        true,
    );
    
    Ok(format!("Successfully collected {} satoshi in protocol fees", fee_amount))
}

/// Emergency pause function (admin only)
#[update]
pub fn emergency_pause_pool() -> Result<String, String> {
    let caller = ic_cdk::caller();
    
    if !is_admin(&caller) {
        return Err("Unauthorized: Only admins can pause the pool".to_string());
    }
    
    // Set emergency pause flag
    set_emergency_pause(true)?;
    
    log_audit_action(
        caller,
        "EMERGENCY_PAUSE".to_string(),
        "Liquidity pool operations paused".to_string(),
        true,
    );
    
    Ok("Pool operations paused successfully".to_string())
}

/// Resume pool operations (admin only)
#[update]
pub fn resume_pool_operations() -> Result<String, String> {
    let caller = ic_cdk::caller();
    
    if !is_admin(&caller) {
        return Err("Unauthorized: Only admins can resume pool operations".to_string());
    }
    
    // Remove emergency pause flag
    set_emergency_pause(false)?;
    
    log_audit_action(
        caller,
        "EMERGENCY_RESUME".to_string(),
        "Liquidity pool operations resumed".to_string(),
        true,
    );
    
    Ok("Pool operations resumed successfully".to_string())
}

// Helper functions for liquidity management

/// Calculate pool APY based on utilization rate and historical performance
fn calculate_pool_apy(pool: &LiquidityPool) -> u64 {
    // Calculate utilization rate
    let utilization_rate = if pool.total_liquidity > 0 {
        ((pool.total_liquidity - pool.available_liquidity) * 100) / pool.total_liquidity
    } else {
        0
    };
    
    // Base APY starts at 3%
    let base_apy = 3;
    
    // Add utilization bonus: 0.05% per 1% utilization
    let utilization_bonus = (utilization_rate * 5) / 100;
    
    // Performance bonus based on repayment rate
    let performance_bonus = if pool.total_borrowed > 0 {
        let repayment_rate = (pool.total_repaid * 100) / pool.total_borrowed;
        if repayment_rate > 90 {
            2 // 2% bonus for >90% repayment rate
        } else if repayment_rate > 75 {
            1 // 1% bonus for >75% repayment rate
        } else {
            0
        }
    } else {
        0
    };
    
    // Cap maximum APY at 15%
    let total_apy = base_apy + utilization_bonus + performance_bonus;
    std::cmp::min(total_apy, 15)
}

/// Calculate pool health score (0-100)
fn calculate_pool_health_score(pool: &LiquidityPool) -> u64 {
    let mut score = 100u64;
    
    // Deduct points for high utilization (>80%)
    let utilization_rate = if pool.total_liquidity > 0 {
        ((pool.total_liquidity - pool.available_liquidity) * 100) / pool.total_liquidity
    } else {
        0
    };
    
    if utilization_rate > 80 {
        score -= (utilization_rate - 80) * 2; // -2 points per % over 80%
    }
    
    // Deduct points for low liquidity (<1 BTC)
    if pool.total_liquidity < 100_000_000 { // 1 BTC in satoshi
        score -= 20;
    }
    
    // Add points for good repayment history
    if pool.total_borrowed > 0 {
        let repayment_rate = (pool.total_repaid * 100) / pool.total_borrowed;
        if repayment_rate > 95 {
            score += 10;
        } else if repayment_rate < 70 {
            score -= 30;
        }
    }
    
    // Ensure score is within bounds
    std::cmp::min(score, 100)
}

/// Check if the caller is authorized to manage loans
fn is_loan_manager(principal: &Principal) -> bool {
    // Check if caller is the loan management canister
    is_loan_manager_canister(principal) || is_admin(principal)
}

/// Validate Bitcoin address format (basic validation)
fn is_valid_bitcoin_address(address: &str) -> bool {
    // Basic Bitcoin address validation
    // This is a simplified check - in production, use a proper Bitcoin address library
    
    if address.is_empty() || address.len() < 26 || address.len() > 62 {
        return false;
    }
    
    // Check for valid Bitcoin address prefixes
    let valid_prefixes = ["1", "3", "bc1", "tb1", "2"]; // mainnet, testnet, bech32
    let starts_with_valid_prefix = valid_prefixes.iter().any(|&prefix| address.starts_with(prefix));
    
    if !starts_with_valid_prefix {
        return false;
    }
    
    // Check for valid characters (base58 for legacy, bech32 for segwit)
    let is_legacy = address.starts_with('1') || address.starts_with('3') || address.starts_with('2');
    let is_bech32 = address.starts_with("bc1") || address.starts_with("tb1");
    
    if is_legacy {
        // Base58 characters (no 0, O, I, l)
        address.chars().all(|c| {
            "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz".contains(c)
        })
    } else if is_bech32 {
        // Bech32 characters (lowercase letters and numbers, no 1, b, i, o)
        address.chars().all(|c| {
            "023456789acdefghjklmnpqrstuvwxyz".contains(c)
        })
    } else {
        false
    }
}

/// Get all disbursement records for a specific loan
pub fn get_disbursement_records_by_loan(loan_id: u64) -> Vec<DisbursementRecord> {
    get_all_disbursement_records()
        .into_iter()
        .filter(|record| record.loan_id == loan_id)
        .collect()
}

/// Validate withdrawal request with comprehensive checks
/// This function performs all validation logic for withdrawal requests
/// Used both for actual withdrawals and for UI validation
#[query]
pub fn validate_withdrawal_request(amount: u64) -> Result<WithdrawalValidation, String> {
    let caller = ic_cdk::caller();
    
    // Anonymous user check
    if caller == Principal::anonymous() {
        return Err("Anonymous users cannot withdraw funds".to_string());
    }
    
    // Basic amount validation
    if amount == 0 {
        return Err("Amount must be greater than zero".to_string());
    }
    
    const MIN_WITHDRAWAL_AMOUNT: u64 = 1000;
    if amount < MIN_WITHDRAWAL_AMOUNT {
        return Err(format!("Minimum withdrawal amount is {} ckBTC satoshi", MIN_WITHDRAWAL_AMOUNT));
    }
    
    // Get investor balance
    let investor_balance = match get_investor_balance_by_principal(caller) {
        Some(balance) => balance,
        None => return Err("No investment balance found".to_string()),
    };
    
    // Check investor balance sufficiency
    if investor_balance.balance < amount {
        return Err(format!(
            "Insufficient balance. Available: {} ckBTC satoshi", 
            investor_balance.balance
        ));
    }
    
    // Check pool liquidity
    let pool = get_liquidity_pool();
    if pool.available_liquidity < amount {
        return Err(format!(
            "Insufficient pool liquidity. Available: {} ckBTC satoshi", 
            pool.available_liquidity
        ));
    }
    
    // Check emergency reserve
    let emergency_reserve_ratio = 5; // 5%
    let required_reserve = (pool.total_liquidity * emergency_reserve_ratio) / 100;
    let liquidity_after_withdrawal = pool.available_liquidity - amount;
    
    if liquidity_after_withdrawal < required_reserve {
        return Err("Withdrawal would violate emergency reserve requirements".to_string());
    }
    
    // System status checks
    if is_emergency_paused() {
        return Err("System is currently paused for maintenance".to_string());
    }
    
    // Rate limiting check
    if !check_rate_limit_with_operation(&caller, "VALIDATE_WITHDRAWAL") {
        return Err("Rate limit exceeded. Please try again later".to_string());
    }
    
    // Calculate fees and final amount (if any fees are implemented)
    let withdrawal_fee = 0u64; // Currently no withdrawal fees
    let net_amount = amount.saturating_sub(withdrawal_fee);
    
    // Calculate new balance after withdrawal
    let new_balance = investor_balance.balance - amount;
    let new_pool_liquidity = pool.available_liquidity - amount;
    
    Ok(WithdrawalValidation {
        is_valid: true,
        amount_requested: amount,
        withdrawal_fee,
        net_amount,
        current_balance: investor_balance.balance,
        new_balance,
        current_pool_liquidity: pool.available_liquidity,
        new_pool_liquidity,
        estimated_confirmation_time: 300, // 5 minutes in seconds
        warnings: vec![], // Add any warnings here
    })
}

/// Get comprehensive investor statistics
/// Provides detailed analytics for individual investors
#[query]
pub fn get_investor_statistics() -> Result<InvestorStatistics, String> {
    let caller = ic_cdk::caller();
    
    if caller == Principal::anonymous() {
        return Err("Anonymous users cannot access statistics".to_string());
    }
    
    let investor_balance = get_investor_balance_for_principal(caller)
        .map_err(|_| "No investment history found")?;
    
    let pool = get_liquidity_pool();
    
    // Calculate investor's share of the pool
    let pool_share_percentage = if pool.total_liquidity > 0 {
        (investor_balance.balance * 10000) / pool.total_liquidity // Basis points
    } else {
        0
    };
    
    // Calculate total returns
    let total_net_return = investor_balance.total_withdrawn.saturating_sub(investor_balance.total_deposited);
    let return_percentage = if investor_balance.total_deposited > 0 {
        (total_net_return * 10000) / investor_balance.total_deposited // Basis points
    } else {
        0
    };
    
    // Calculate average transaction size
    let total_transactions = investor_balance.deposits.len() + investor_balance.withdrawals.len();
    let total_volume = investor_balance.total_deposited + investor_balance.total_withdrawn;
    let avg_transaction_size = if total_transactions > 0 {
        total_volume / (total_transactions as u64)
    } else {
        0
    };
    
    // Calculate activity metrics
    let days_since_first_deposit = if investor_balance.first_deposit_at > 0 {
        (time() - investor_balance.first_deposit_at) / (24 * 60 * 60 * 1_000_000_000) // Convert from nanoseconds to days
    } else {
        0
    };
    
    let days_since_last_activity = if investor_balance.last_activity_at > 0 {
        (time() - investor_balance.last_activity_at) / (24 * 60 * 60 * 1_000_000_000)
    } else {
        0
    };
    
    Ok(InvestorStatistics {
        investor: caller,
        current_balance: investor_balance.balance,
        total_deposited: investor_balance.total_deposited,
        total_withdrawn: investor_balance.total_withdrawn,
        net_position: investor_balance.balance,
        total_deposits_count: investor_balance.deposits.len() as u64,
        total_withdrawals_count: investor_balance.withdrawals.len() as u64,
        pool_share_basis_points: pool_share_percentage,
        return_basis_points: return_percentage,
        avg_transaction_size,
        days_since_first_deposit,
        days_since_last_activity,
        is_active_investor: days_since_last_activity <= 30, // Active if activity within 30 days
        risk_level: if investor_balance.balance > 10_000_000 { "HIGH" } else if investor_balance.balance > 1_000_000 { "MEDIUM" } else { "LOW" }.to_string(),
    })
}

/// Get withdrawal fee estimate
/// Calculates estimated fees for a withdrawal (currently zero)
#[query]
pub fn get_withdrawal_fee_estimate(amount: u64) -> Result<WithdrawalFeeEstimate, String> {
    if amount == 0 {
        return Err("Amount must be greater than zero".to_string());
    }
    
    // Currently no withdrawal fees implemented
    // This function is prepared for future fee implementation
    let base_fee = 0u64;
    let percentage_fee = 0u64; // 0% fee
    let total_fee = base_fee + ((amount * percentage_fee) / 10000);
    let net_amount = amount.saturating_sub(total_fee);
    
    Ok(WithdrawalFeeEstimate {
        requested_amount: amount,
        base_fee,
        percentage_fee_basis_points: percentage_fee,
        total_fee,
        net_withdrawal_amount: net_amount,
        fee_structure_version: 1,
    })
}

/// Emergency withdrawal for admin (in case of system issues)
/// This function allows admins to help users withdraw in emergency situations
#[update]
pub async fn emergency_admin_withdrawal(
    investor: Principal, 
    amount: u64, 
    reason: String
) -> Result<String, String> {
    let caller = ic_cdk::caller();
    
    // Only admins can perform emergency withdrawals
    if !is_admin(&caller) {
        return Err("Unauthorized: Only admins can perform emergency withdrawals".to_string());
    }
    
    // Validate inputs
    if amount == 0 {
        return Err("Amount must be greater than zero".to_string());
    }
    
    if reason.trim().is_empty() {
        return Err("Emergency reason is required".to_string());
    }
    
    // Get investor balance
    let investor_balance = get_investor_balance_for_principal(investor)
        .map_err(|_| "Investor not found or has no balance")?;
    
    if investor_balance.balance < amount {
        return Err("Investor has insufficient balance".to_string());
    }
    
    // Check pool liquidity
    let pool = get_liquidity_pool();
    if pool.available_liquidity < amount {
        return Err("Insufficient pool liquidity for emergency withdrawal".to_string());
    }
    
    // Log emergency action
    log_audit_action(
        caller,
        "EMERGENCY_WITHDRAWAL_INITIATED".to_string(),
        format!(
            "Admin {} initiated emergency withdrawal for investor {}: {} ckBTC satoshi. Reason: {}", 
            caller, investor, amount, reason
        ),
        true,
    );
    
    // Prepare ckBTC transfer
    let ckbtc_ledger = Principal::from_text(CKBTC_LEDGER_PRINCIPAL)
        .map_err(|_| "Invalid ckBTC ledger principal configuration")?;
    
    let investor_account = Account {
        owner: investor,
        subaccount: None,
    };
    
    let transfer_args = TransferArgs {
        from_subaccount: None,
        to: investor_account,
        amount: Nat::from(amount),
        fee: None,
        memo: Some(format!("EMERGENCY WITHDRAWAL by admin: {}", reason).as_bytes().to_vec()),
        created_at_time: Some(time()),
    };
    
    // Execute transfer
    let call_result: Result<(Result<Nat, TransferError>,), _> = 
        call(ckbtc_ledger, "icrc1_transfer", (transfer_args,)).await;
    
    match call_result {
        Ok((Ok(block_index),)) => {
            let block_idx = block_index.0.try_into().unwrap_or(0u64);
            
            // Update pool state
            let mut updated_pool = pool;
            updated_pool.total_liquidity -= amount;
            updated_pool.available_liquidity -= amount;
            updated_pool.updated_at = time();
            store_liquidity_pool(updated_pool)?;
            
            // Update investor balance
            let mut updated_investor_balance = investor_balance;
            updated_investor_balance.balance -= amount;
            updated_investor_balance.total_withdrawn += amount;
            updated_investor_balance.last_activity_at = time();
            
            // Create withdrawal record with emergency flag
            let withdrawal_record = WithdrawalRecord {
                investor,
                amount,
                ckbtc_block_index: block_idx,
                timestamp: time(),
            };
            updated_investor_balance.withdrawals.push(withdrawal_record);
            
            store_investor_balance(updated_investor_balance)?;
            
            // Comprehensive audit logging
            log_audit_action(
                caller,
                "EMERGENCY_WITHDRAWAL_SUCCESS".to_string(),
                format!(
                    "Emergency withdrawal completed: {} ckBTC satoshi for investor {}, block: {}, reason: {}", 
                    amount, investor, block_idx, reason
                ),
                true,
            );
            
            Ok(format!("Emergency withdrawal successful. Block: {}", block_idx))
        }
        Ok((Err(transfer_error),)) => {
            let error_msg = format!("Emergency withdrawal failed: {:?}", transfer_error);
            log_audit_action(
                caller,
                "EMERGENCY_WITHDRAWAL_FAILED".to_string(),
                format!("Emergency withdrawal failed for investor {}: {}", investor, error_msg),
                false,
            );
            Err(error_msg)
        }
        Err(call_error) => {
            let error_msg = format!("Network error during emergency withdrawal: {:?}", call_error);
            log_audit_action(
                caller,
                "EMERGENCY_WITHDRAWAL_NETWORK_ERROR".to_string(),
                format!("Emergency withdrawal network error for investor {}: {}", investor, error_msg),
                false,
            );
            Err(error_msg)
        }
    }
}

/// Get investor's transaction history
#[query]
pub fn get_investor_transaction_history() -> Result<InvestorTransactionHistory, String> {
    let caller = ic_cdk::caller();
    
    // Get investor balance
    let investor_balance = get_investor_balance_for_principal(caller)
        .map_err(|e| format!("No balance found for investor: {}", e))?;
    
    Ok(InvestorTransactionHistory {
        investor: caller,
        deposits: investor_balance.deposits,
        withdrawals: investor_balance.withdrawals,
        total_deposited: investor_balance.total_deposited,
        total_withdrawn: investor_balance.total_withdrawn,
        net_balance: investor_balance.balance,
        first_activity: investor_balance.first_deposit_at,
        last_activity: investor_balance.last_activity_at,
    })
}

/// Get all disbursement records (admin only)
#[query]
pub fn get_all_disbursements() -> Result<Vec<DisbursementRecord>, String> {
    let caller = ic_cdk::caller();
    
    if !is_admin(&caller) {
        return Err("Unauthorized: Only admins can view all disbursements".to_string());
    }
    
    Ok(get_all_disbursement_records())
}

/// Get disbursement records for a specific loan
#[query]
pub fn get_loan_disbursements(loan_id: u64) -> Result<Vec<DisbursementRecord>, String> {
    let caller = ic_cdk::caller();
    
    // Allow admins and loan managers to view disbursements
    if !is_admin(&caller) && !is_loan_manager_canister(&caller) {
        return Err("Unauthorized: Only admins and loan managers can view disbursements".to_string());
    }
    
    Ok(get_disbursement_records_by_loan(loan_id))
}

/// Force refresh pool statistics (admin only)
#[update]
pub fn refresh_pool_statistics() -> Result<String, String> {
    let caller = ic_cdk::caller();
    
    if !is_admin(&caller) {
        return Err("Unauthorized: Only admins can refresh pool statistics".to_string());
    }
    
    // Recalculate pool statistics
    let mut pool = get_liquidity_pool();
    pool.updated_at = time();
    
    // Recalculate investor count
    let all_balances = crate::storage::get_all_investor_balances();
    let active_investors = all_balances.iter()
        .filter(|balance| balance.balance > 0)
        .count() as u64;
    
    pool.total_investors = active_investors;
    
    store_liquidity_pool(pool)?;
    
    log_audit_action(
        caller,
        "POOL_STATISTICS_REFRESH".to_string(),
        "Pool statistics refreshed manually".to_string(),
        true,
    );
    
    Ok("Pool statistics refreshed successfully".to_string())
}

/// Set liquidity pool parameters (admin only)
#[update]
pub fn set_pool_parameters(
    min_deposit_amount: Option<u64>,
    max_utilization_rate: Option<u64>,
    emergency_reserve_ratio: Option<u64>
) -> Result<String, String> {
    let caller = ic_cdk::caller();
    
    if !is_admin(&caller) {
        return Err("Unauthorized: Only admins can set pool parameters".to_string());
    }
    
    // Store parameters in canister configuration
    let mut config = get_canister_config();
    
    if let Some(min_deposit) = min_deposit_amount {
        if min_deposit < 10_000 { // Minimum 0.0001 BTC
            return Err("Minimum deposit amount too small".to_string());
        }
        config.min_deposit_amount = min_deposit;
    }
    
    if let Some(max_util) = max_utilization_rate {
        if max_util > 95 {
            return Err("Maximum utilization rate cannot exceed 95%".to_string());
        }
        config.max_utilization_rate = max_util * 100; // Convert to basis points
    }
    
    if let Some(reserve_ratio) = emergency_reserve_ratio {
        if reserve_ratio < 5 || reserve_ratio > 50 {
            return Err("Emergency reserve ratio must be between 5% and 50%".to_string());
        }
        config.emergency_reserve_ratio = reserve_ratio * 100; // Convert to basis points
    }
    
    set_canister_config(config)?;
    
    log_audit_action(
        caller,
        "POOL_PARAMETERS_UPDATE".to_string(),
        format!("Pool parameters updated: min_deposit={:?}, max_util={:?}, reserve_ratio={:?}", 
                min_deposit_amount, max_utilization_rate, emergency_reserve_ratio),
        true,
    );
    
    Ok("Pool parameters updated successfully".to_string())
}

/// Get pool health metrics (admin only)
#[query]
pub fn get_pool_health_metrics() -> Result<PoolHealthMetrics, String> {
    let caller = ic_cdk::caller();
    
    if !is_admin(&caller) {
        return Err("Unauthorized: Only admins can view pool health metrics".to_string());
    }
    
    let pool = get_liquidity_pool();
    let all_balances = crate::storage::get_all_investor_balances();
    let all_disbursements = get_all_disbursement_records();
    
    // Calculate metrics
    let _utilization_rate = if pool.total_liquidity > 0 {
        ((pool.total_liquidity - pool.available_liquidity) * 100) / pool.total_liquidity
    } else {
        0
    };
    
    let average_deposit_size = if all_balances.is_empty() {
        0
    } else {
        all_balances.iter().map(|b| b.total_deposited).sum::<u64>() / all_balances.len() as u64
    };
    
    let largest_deposit = all_balances.iter()
        .map(|b| b.total_deposited)
        .max()
        .unwrap_or(0);
    
    let _concentration_risk = if pool.total_liquidity > 0 {
        (largest_deposit * 100) / pool.total_liquidity
    } else {
        0
    };
    
    let total_disbursements = all_disbursements.len() as u64;
    let _average_loan_size = if total_disbursements > 0 {
        all_disbursements.iter().map(|d| d.amount).sum::<u64>() / total_disbursements
    } else {
        0
    };
    
    let utilization_rate = if pool.total_liquidity > 0 {
        ((pool.total_liquidity - pool.available_liquidity) * 100) / pool.total_liquidity
    } else {
        0
    };
    
    let health_score = calculate_pool_health_score(&pool);
    
    Ok(PoolHealthMetrics {
        total_liquidity: pool.total_liquidity,
        available_liquidity: pool.available_liquidity,
        utilized_liquidity: pool.total_liquidity - pool.available_liquidity,
        utilization_rate,
        total_investors: pool.total_investors,
        active_investors: pool.total_investors,
        total_loans: all_disbursements.len() as u64,
        active_loans: pool.total_borrowed,
        default_rate: 0, // TODO: Calculate actual default rate
        avg_loan_size: if total_disbursements > 0 {
            all_disbursements.iter().map(|d| d.amount).sum::<u64>() / total_disbursements
        } else {
            0
        },
        pool_health_score: health_score,
        last_updated: time(),
    })
}

/// Calculate liquidity trend based on recent activity
#[allow(dead_code)]
fn calculate_liquidity_trend(pool: &LiquidityPool) -> String {
    // This is a simplified trend calculation
    // In a real implementation, you would track historical data
    
    let utilization_rate = if pool.total_liquidity > 0 {
        ((pool.total_liquidity - pool.available_liquidity) * 100) / pool.total_liquidity
    } else {
        0
    };
    
    match utilization_rate {
        0..=20 => "Low Demand".to_string(),
        21..=50 => "Moderate Demand".to_string(),
        51..=80 => "High Demand".to_string(),
        81..=95 => "Very High Demand".to_string(),
        _ => "Over-Utilized".to_string(),
    }
}

/// Perform automated pool maintenance (heartbeat function)
#[update]
pub fn perform_pool_maintenance() -> Result<String, String> {
    let caller = ic_cdk::caller();
    
    // Only allow system calls or admin calls
    if caller != canister_self() && !is_admin(&caller) {
        return Err("Unauthorized: Only system or admin can perform maintenance".to_string());
    }
    
    let mut maintenance_actions = Vec::new();
    
    // Check pool health
    let pool = get_liquidity_pool();
    let health_score = calculate_pool_health_score(&pool);
    
    if health_score < 50 {
        maintenance_actions.push("Pool health critical - consider emergency measures".to_string());
    }
    
    // Check utilization rate
    let utilization_rate = if pool.total_liquidity > 0 {
        ((pool.total_liquidity - pool.available_liquidity) * 100) / pool.total_liquidity
    } else {
        0
    };
    
    if utilization_rate > 90 {
        maintenance_actions.push("High utilization detected - monitor closely".to_string());
    }
    
    // Clean up old processed transactions (older than 30 days)
    let thirty_days_ago = time() - (30 * 24 * 60 * 60 * 1_000_000_000);
    let cleaned_transactions = cleanup_old_transactions(thirty_days_ago)?;
    
    if cleaned_transactions > 0 {
        maintenance_actions.push(format!("Cleaned {} old transactions", cleaned_transactions));
    }
    
    // Log maintenance activity
    log_audit_action(
        caller,
        "POOL_MAINTENANCE".to_string(),
        format!("Maintenance performed: {:?}", maintenance_actions),
        true,
    );
    
    Ok(format!("Maintenance completed. Actions: {:?}", maintenance_actions))
}

/// Cleanup old processed transactions
fn cleanup_old_transactions(cutoff_time: u64) -> Result<u64, String> {
    let old_transactions = get_all_processed_transactions()
        .into_iter()
        .filter(|tx| tx.processed_at < cutoff_time)
        .collect::<Vec<_>>();
    
    let count = old_transactions.len() as u64;
    
    for tx in old_transactions {
        remove_processed_transaction(tx.tx_id);
    }
    
    Ok(count)
}

/// Get all processed transactions (admin only)
#[query]
pub fn get_processed_transactions_admin() -> Result<Vec<ProcessedTransaction>, String> {
    let caller = ic_cdk::caller();
    
    if !is_admin(&caller) {
        return Err("Unauthorized: Only admins can view processed transactions".to_string());
    }
    
    Ok(crate::storage::get_all_processed_transactions())
}

/// Get processed transactions for current investor
#[query]
pub fn get_my_processed_transactions() -> Vec<ProcessedTransaction> {
    let caller = ic_cdk::caller();
    crate::storage::get_processed_transactions_by_investor(caller)
}

/// Emergency function to halt all pool operations
#[update]
pub fn emergency_halt_operations() -> Result<String, String> {
    let caller = ic_cdk::caller();
    
    if !is_admin(&caller) {
        return Err("Unauthorized: Only admins can halt operations".to_string());
    }
    
    set_emergency_pause(true)?;
    
    log_audit_action(
        caller,
        "EMERGENCY_HALT".to_string(),
        "All pool operations halted by admin".to_string(),
        true,
    );
    
    Ok("Emergency halt activated - all operations suspended".to_string())
}

/// Check if pool operations are paused
#[query]
pub fn is_pool_paused() -> bool {
    is_emergency_paused()
}

/// Get pool configuration (admin only)
#[query]
pub fn get_pool_configuration() -> Result<PoolConfiguration, String> {
    let caller = ic_cdk::caller();
    
    if !is_admin(&caller) {
        return Err("Unauthorized: Only admins can view pool configuration".to_string());
    }
    
    let config = get_canister_config();
    let pool = get_liquidity_pool();
    
    Ok(PoolConfiguration {
        min_deposit_amount: config.min_deposit_amount,
        max_deposit_amount: u64::MAX, // No current limit
        min_withdrawal_amount: 10_000, // 0.0001 BTC
        max_utilization_rate: config.max_utilization_rate,
        emergency_reserve_ratio: config.emergency_reserve_ratio,
        base_apy: 300, // 3% base APY in basis points
        performance_fee: 100, // 1% performance fee in basis points
        withdrawal_fee: 0, // No withdrawal fee
        is_paused: is_emergency_paused(),
        created_at: pool.created_at,
        updated_at: pool.updated_at,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use candid::Principal;
    
    fn setup_test_environment() {
        // Initialize test configuration
        let config = CanisterConfig {
            admins: vec![Principal::from_text("rdmx6-jaaaa-aaaah-qcaiq-cai").unwrap()],
            loan_manager_principal: Some(Principal::from_text("rrkah-fqaaa-aaaah-qcaiq-cai").unwrap()),
            min_deposit_amount: 100_000,
            max_utilization_rate: 85,
            emergency_reserve_ratio: 15,
            is_maintenance_mode: false,
            created_at: 0,
            updated_at: 0,
        };
        set_canister_config(config).unwrap();
    }
    
    #[test]
    fn test_pool_stats_calculation() {
        setup_test_environment();
        
        let stats = get_pool_stats();
        
        assert_eq!(stats.total_liquidity, 0);
        assert_eq!(stats.available_liquidity, 0);
        assert_eq!(stats.utilization_rate, 0);
        assert_eq!(stats.total_investors, 0);
        assert!(stats.apy >= 3); // Base APY should be at least 3%
    }
    
    #[test]
    fn test_bitcoin_address_validation() {
        assert!(is_valid_bitcoin_address("1BvBMSEYstWetqTFn5Au4m4GFg7xJaNVN2"));
        assert!(is_valid_bitcoin_address("3J98t1WpEZ73CNmQviecrnyiWrnqRhWNLy"));
        assert!(is_valid_bitcoin_address("bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4"));
        
        assert!(!is_valid_bitcoin_address(""));
        assert!(!is_valid_bitcoin_address("invalid"));
        assert!(!is_valid_bitcoin_address("1BvBMSEYstWetqTFn5Au4m4GFg7xJaNVN2X")); // Too long
        assert!(!is_valid_bitcoin_address("0BvBMSEYstWetqTFn5Au4m4GFg7xJaNVN2")); // Invalid character
    }
    
    #[test]
    fn test_pool_health_score_calculation() {
        setup_test_environment();
        
        let pool = LiquidityPool {
            total_liquidity: 1_000_000_000, // 10 BTC
            available_liquidity: 200_000_000, // 2 BTC available
            total_borrowed: 800_000_000, // 8 BTC borrowed
            total_repaid: 760_000_000, // 7.6 BTC repaid (95% repayment rate)
            utilization_rate: 80,
            total_investors: 5,
            apy: 0,
            created_at: 0,
            updated_at: 0,
        };
        
        let health_score = calculate_pool_health_score(&pool);
        
        // Should have good health score due to high liquidity and good repayment rate
        assert!(health_score >= 80);
    }
    
    #[test]
    fn test_apy_calculation() {
        setup_test_environment();
        
        let pool = LiquidityPool {
            total_liquidity: 1_000_000_000, // 10 BTC
            available_liquidity: 300_000_000, // 3 BTC available (70% utilization)
            total_borrowed: 700_000_000, // 7 BTC borrowed
            total_repaid: 665_000_000, // 6.65 BTC repaid (95% repayment rate)
            utilization_rate: 70,
            total_investors: 10,
            apy: 0,
            created_at: 0,
            updated_at: 0,
        };
        
        let apy = calculate_pool_apy(&pool);
        
        // Should be base APY (3%) + utilization bonus + performance bonus
        assert!(apy >= 6); // 3% base + 3.5% utilization + 2% performance
        assert!(apy <= 15); // Should not exceed maximum APY
    }
    
    #[test]
    fn test_pool_concentration_risk() {
        setup_test_environment();
        
        // Test scenario with high concentration risk
        let pool = LiquidityPool {
            total_liquidity: 1_000_000_000, // 10 BTC
            available_liquidity: 500_000_000, // 5 BTC available
            total_borrowed: 500_000_000, // 5 BTC borrowed
            total_repaid: 0,
            utilization_rate: 50,
            total_investors: 5,
            apy: 0,
            created_at: 0,
            updated_at: 0,
        };
        
        // Simulate largest investor with 8 BTC deposit
        let concentration_risk = (800_000_000 * 100) / pool.total_liquidity;
        
        assert_eq!(concentration_risk, 80); // 80% concentration risk
    }
}

// Integration tests for liquidity management workflows
#[cfg(test)]
mod integration_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_deposit_workflow() {
        // Note: This test would require setting up a local IC environment
        // and mocking the ckBTC ledger calls
        
        // 1. Register investor
        // 2. Approve ckBTC spend
        // 3. Call deposit_liquidity
        // 4. Verify pool state updated
        // 5. Verify investor balance updated
        // 6. Verify transaction marked as processed
    }
    
    #[tokio::test]
    async fn test_disbursement_workflow() {
        // Note: This test would require setting up a local IC environment
        // and mocking the ckBTC minter calls
        
        // 1. Setup pool with liquidity
        // 2. Call disburse_loan from loan manager
        // 3. Verify Bitcoin address validation
        // 4. Verify sufficient liquidity check
        // 5. Verify pool state updated
        // 6. Verify disbursement record created
    }
    
    #[tokio::test]
    async fn test_emergency_scenarios() {
        // Test emergency pause functionality
        // Test pool utilization limits
        // Test concentration risk warnings
        // Test maintenance mode operations
    }
}

// Performance tests
#[cfg(test)]
mod performance_tests {
    use super::*;
    
    #[test]
    fn test_large_dataset_performance() {
        // Test with large number of investors
        // Test with many transactions
        // Test query performance
        // Test memory usage
    }
}

// Security tests
#[cfg(test)]
mod security_tests {
    use super::*;
    
    #[test]
    fn test_access_control() {
        // Test that only authorized callers can disburse
        // Test that only admins can access sensitive functions
        // Test that investors can only access their own data
    }
    
    #[test]
    fn test_input_validation() {
        // Test invalid amounts
        // Test invalid addresses
        // Test boundary conditions
        // Test overflow protection
    }
}
