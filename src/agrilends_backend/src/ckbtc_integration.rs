use ic_cdk::api::{canister_self, time};
use ic_cdk::call::CallResult;
use ic_cdk::{call}; // Add call import
use ic_cdk_macros::update;
use candid::{CandidType, Deserialize, Principal, Nat};
use crate::types::*;
use crate::storage::{
    get_loan, update_loan_status, update_loan_repaid_amount, store_disbursement_record,
    store_repayment_record, get_disbursement_record
};
use crate::helpers::{log_audit_action, is_admin, is_loan_manager, get_user_btc_address};
use crate::storage::release_collateral_nft;

// ckBTC Ledger Principal (Mainnet)
const CKBTC_LEDGER_PRINCIPAL: &str = "mxzaz-hqaaa-aaaar-qaada-cai";

// ckBTC Integration structures
#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct Account {
    pub owner: Principal,
    pub subaccount: Option<Vec<u8>>,
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

#[derive(CandidType, Deserialize)]
pub struct BalanceArgs {
    pub account: Account,
}

// Real ckBTC transfer implementation
#[update]
pub async fn transfer_ckbtc_to_borrower(
    loan_id: u64,
    borrower: Principal,
    amount: u64,
) -> Result<u64, String> {
    // Verify caller is authorized (loan manager or admin)
    let caller = ic_cdk::caller();
    if !is_admin(&caller) && !is_loan_manager(&caller) {
        return Err("Unauthorized: Only loan manager or admin can transfer ckBTC".to_string());
    }

    // Verify loan exists and is in correct state
    let loan = get_loan(loan_id).ok_or("Loan not found")?;
    if loan.status != LoanStatus::Approved {
        return Err("Loan must be approved for disbursement".to_string());
    }

    if loan.borrower != borrower {
        return Err("Borrower mismatch".to_string());
    }

    // Check if already disbursed
    if get_disbursement_record(loan_id).is_some() {
        return Err("Loan already disbursed".to_string());
    }

    // Get borrower's BTC address
    let borrower_btc_address = get_user_btc_address(&borrower)
        .ok_or("Borrower BTC address not found")?;

    let ckbtc_ledger = Principal::from_text(CKBTC_LEDGER_PRINCIPAL)
        .map_err(|_| "Invalid ckBTC ledger principal")?;

    // Create transfer arguments
    let transfer_args = TransferArgs {
        from_subaccount: None, // Use default subaccount
        to: Account {
            owner: borrower,
            subaccount: None,
        },
        amount: Nat::from(amount),
        fee: None, // Let ledger determine fee
        memo: Some(format!("Loan disbursement #{}", loan_id).into_bytes()),
        created_at_time: Some(time()),
    };

    // Execute the transfer
    let call_result: CallResult<(Result<Nat, TransferError>,)> = 
        call(ckbtc_ledger, "icrc1_transfer", (transfer_args,)).await;

    match call_result {
        Ok((Ok(block_index),)) => {
            let block_index_u64 = block_index.0.try_into()
                .map_err(|_| "Block index too large")?;

            // Record the disbursement
            let disbursement = DisbursementRecord {
                loan_id,
                borrower_btc_address: borrower_btc_address.clone(),
                amount,
                ckbtc_block_index: block_index_u64,
                disbursed_at: time(),
                disbursed_by: caller,
            };

            store_disbursement_record(disbursement)?;

            // Update loan status
            update_loan_status(loan_id, LoanStatus::Active)?;

            // Log the successful transfer
            log_audit_action(
                caller,
                "CKBTC_TRANSFER_SUCCESS".to_string(),
                format!("Disbursed {} ckBTC to {} for loan #{}, block: {}", 
                    amount, borrower, loan_id, block_index_u64),
                true,
            );

            Ok(block_index_u64)
        }
        Ok((Err(transfer_error),)) => {
            let error_msg = format!("ckBTC transfer failed: {:?}", transfer_error);
            
            log_audit_action(
                caller,
                "CKBTC_TRANSFER_FAILED".to_string(),
                format!("Failed to disburse loan #{}: {}", loan_id, error_msg),
                false,
            );

            Err(error_msg)
        }
        Err((rejection_code, msg)) => {
            let error_msg = format!("ckBTC transfer call failed: {:?} - {}", rejection_code, msg);
            
            log_audit_action(
                caller,
                "CKBTC_CALL_FAILED".to_string(),
                format!("Failed to call ckBTC ledger for loan #{}: {}", loan_id, error_msg),
                false,
            );

            Err(error_msg)
        }
    }
}

// Process loan repayment via ckBTC
#[update]
pub async fn process_ckbtc_repayment(
    loan_id: u64,
    amount: u64,
) -> Result<u64, String> {
    let caller = ic_cdk::caller();
    
    // Verify loan exists
    let loan = get_loan(loan_id).ok_or("Loan not found")?;
    
    // Verify caller is the borrower
    if loan.borrower != caller {
        return Err("Only the borrower can repay the loan".to_string());
    }

    // Verify loan is active
    if loan.status != LoanStatus::Active {
        return Err("Loan is not active for repayment".to_string());
    }

    // Calculate remaining balance
    let remaining_balance = calculate_remaining_balance(loan_id)?;
    if amount > remaining_balance {
        return Err(format!(
            "Payment amount {} exceeds remaining balance {}", 
            amount, remaining_balance
        ));
    }

    let ckbtc_ledger = Principal::from_text(CKBTC_LEDGER_PRINCIPAL)
        .map_err(|_| "Invalid ckBTC ledger principal")?;

    // Create transfer arguments (from borrower to protocol)
    let transfer_args = TransferArgs {
        from_subaccount: None,
        to: Account {
            owner: canister_self(), // Transfer to this canister
            subaccount: None,
        },
        amount: Nat::from(amount),
        fee: None,
        memo: Some(format!("Loan repayment #{}", loan_id).into_bytes()),
        created_at_time: Some(time()),
    };

    // Note: In real implementation, borrower would need to approve the transfer first
    // This is a simplified version - actual implementation needs approval workflow

    let call_result: CallResult<(Result<Nat, TransferError>,)> = 
        call(ckbtc_ledger, "icrc1_transfer", (transfer_args,)).await;

    match call_result {
        Ok((Ok(block_index),)) => {
            let block_index_u64 = block_index.0.try_into()
                .map_err(|_| "Block index too large")?;

            // Record the repayment
            let repayment = RepaymentRecord {
                loan_id,
                payer: caller,
                amount,
                ckbtc_block_index: block_index_u64,
                timestamp: time(),
            };

            store_repayment_record(repayment)?;

            // Update loan's total repaid amount
            update_loan_repaid_amount(loan_id, amount)?;

            // Check if loan is fully repaid
            let new_remaining = remaining_balance - amount;
            if new_remaining == 0 {
                update_loan_status(loan_id, LoanStatus::Repaid)?;
                
                // Release the collateral NFT
                release_collateral_nft(loan.nft_id)?;
                
                log_audit_action(
                    caller,
                    "LOAN_FULLY_REPAID".to_string(),
                    format!("Loan #{} fully repaid, collateral released", loan_id),
                    true,
                );
            } else {
                log_audit_action(
                    caller,
                    "LOAN_PARTIAL_REPAYMENT".to_string(),
                    format!("Partial repayment of {} for loan #{}, remaining: {}", 
                        amount, loan_id, new_remaining),
                    true,
                );
            }

            Ok(block_index_u64)
        }
        Ok((Err(transfer_error),)) => {
            let error_msg = format!("ckBTC repayment failed: {:?}", transfer_error);
            
            log_audit_action(
                caller,
                "CKBTC_REPAYMENT_FAILED".to_string(),
                format!("Failed repayment for loan #{}: {}", loan_id, error_msg),
                false,
            );

            Err(error_msg)
        }
        Err((rejection_code, msg)) => {
            let error_msg = format!("ckBTC repayment call failed: {:?} - {}", rejection_code, msg);
            
            log_audit_action(
                caller,
                "CKBTC_REPAYMENT_CALL_FAILED".to_string(),
                format!("Failed to call ckBTC ledger for repayment #{}: {}", loan_id, error_msg),
                false,
            );

            Err(error_msg)
        }
    }
}

// Check ckBTC balance of an account
#[update]
pub async fn check_ckbtc_balance(account: Account) -> Result<u64, String> {
    let ckbtc_ledger = Principal::from_text(CKBTC_LEDGER_PRINCIPAL)
        .map_err(|_| "Invalid ckBTC ledger principal")?;

    let balance_args = BalanceArgs { account };

    let call_result: Result<(Nat,), _> = 
        call(ckbtc_ledger, "icrc1_balance_of", (balance_args,)).await;

    match call_result {
        Ok((balance,)) => {
            balance.0.try_into()
                .map_err(|_| "Balance too large".to_string())
        }
        Err((rejection_code, msg)) => {
            Err(format!("Failed to check balance: {:?} - {}", rejection_code, msg))
        }
    }
}

// Get canister's ckBTC balance
#[update]
pub async fn get_protocol_ckbtc_balance() -> Result<u64, String> {
    let account = Account {
        owner: canister_self(),
        subaccount: None,
    };
    
    check_ckbtc_balance(account).await
}

// Admin function to withdraw protocol earnings
#[update]
pub async fn admin_withdraw_protocol_earnings(
    to: Principal,
    amount: u64,
) -> Result<u64, String> {
    let caller = ic_cdk::caller();
    if !is_admin(&caller) {
        return Err("Only admins can withdraw protocol earnings".to_string());
    }

    // Check available balance
    let balance = get_protocol_ckbtc_balance().await?;
    if amount > balance {
        return Err("Insufficient protocol balance".to_string());
    }

    let ckbtc_ledger = Principal::from_text(CKBTC_LEDGER_PRINCIPAL)
        .map_err(|_| "Invalid ckBTC ledger principal")?;

    let transfer_args = TransferArgs {
        from_subaccount: None,
        to: Account {
            owner: to,
            subaccount: None,
        },
        amount: Nat::from(amount),
        fee: None,
        memo: Some("Protocol earnings withdrawal".as_bytes().to_vec()),
        created_at_time: Some(time()),
    };

    let call_result: CallResult<(Result<Nat, TransferError>,)> = 
        call(ckbtc_ledger, "icrc1_transfer", (transfer_args,)).await;

    match call_result {
        Ok((Ok(block_index),)) => {
            let block_index_u64 = block_index.0.try_into()
                .map_err(|_| "Block index too large")?;

            log_audit_action(
                caller,
                "PROTOCOL_EARNINGS_WITHDRAWAL".to_string(),
                format!("Admin {} withdrew {} ckBTC to {}, block: {}", 
                    caller, amount, to, block_index_u64),
                true,
            );

            Ok(block_index_u64)
        }
        Ok((Err(transfer_error),)) => {
            Err(format!("Protocol withdrawal failed: {:?}", transfer_error))
        }
        Err((rejection_code, msg)) => {
            Err(format!("Protocol withdrawal call failed: {:?} - {}", rejection_code, msg))
        }
    }
}

// Helper function to calculate remaining loan balance including interest
fn calculate_remaining_balance(loan_id: u64) -> Result<u64, String> {
    let loan = get_loan(loan_id).ok_or("Loan not found")?;
    
    // Simple interest calculation for MVP
    // In production, consider compound interest and more sophisticated models
    let elapsed_time = time() - loan.created_at;
    let elapsed_days = elapsed_time / (24 * 60 * 60 * 1_000_000_000u64);
    
    let principal = loan.amount_approved;
    let annual_rate = loan.apr as f64 / 100.0;
    let daily_rate = annual_rate / 365.0;
    
    let interest = (principal as f64 * daily_rate * elapsed_days as f64) as u64;
    let total_owed = principal + interest;
    
    // Subtract any payments already made
    Ok(total_owed.saturating_sub(loan.total_repaid))
}
