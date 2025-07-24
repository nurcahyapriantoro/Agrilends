use ic_cdk::{caller, api::time}; // Add caller import
use ic_cdk_macros::{query, update};
use crate::types::*;
use crate::storage::{
    get_loan, store_loan, get_next_loan_id, get_loans_by_borrower,
    get_all_loans_data, get_nft_data, lock_nft_for_loan, get_stored_commodity_price,
    get_protocol_parameters, liquidate_collateral, unlock_nft, store_repayment_record,
    release_collateral_nft
};
use crate::user_management::{get_user, Role, UserResult};
use crate::helpers::{get_user_btc_address, log_audit_action, get_canister_config};
// Production integrations  
use crate::oracle::{is_price_stale};
use crate::ckbtc_integration::{process_ckbtc_repayment};
// Notification system integration
use crate::notification_system::{notify_loan_event, notify_collateral_event};
use std::collections::HashMap;

// Submit loan application
#[update]
pub async fn submit_loan_application(
    nft_id: u64,
    amount_requested: u64,
) -> Result<Loan, String> {
    let caller = ic_cdk::caller();
    
    // 1. Verifikasi pengguna terdaftar sebagai petani
    match get_user() {
        UserResult::Ok(user) => {
            if user.role != Role::Farmer {
                return Err("Only farmers can apply for loans".to_string());
            }
        }
        UserResult::Err(e) => return Err(format!("User verification failed: {}", e)),
    }

    // 2. Verifikasi kepemilikan NFT
    let nft_data = get_nft_data(nft_id).ok_or_else(|| "NFT not found".to_string())?;
    if nft_data.owner != caller {
        return Err("You don't own this NFT".to_string());
    }

    // 3. Verifikasi NFT tidak sedang terkunci
    if nft_data.is_locked {
        return Err("NFT is already locked in another loan".to_string());
    }

    // 4. Ambil metadata NFT untuk valuasi
    let valuation_idr = extract_valuation_from_metadata(&nft_data.metadata)?;
    let commodity_info = extract_commodity_info_from_metadata(&nft_data.metadata)?;

    // 5. Ambil harga komoditas real dari Oracle
    let commodity_price_data = get_stored_commodity_price(&commodity_info.commodity_type)
        .ok_or_else(|| "Commodity price not available. Please contact admin to update price feeds.".to_string())?;
    
    // Check if price is stale (older than 24 hours)
    if is_price_stale(commodity_info.commodity_type.clone()) {
        return Err("Commodity price data is stale. Please wait for price update.".to_string());
    }

    // 6. Hitung nilai agunan dalam ckBTC
    let collateral_value_btc = calculate_collateral_value_btc(
        valuation_idr,
        commodity_info.quantity,
        &commodity_price_data,
    )?;

    // 7. Ambil parameter protokol
    let params = get_protocol_parameters();
    
    // 8. Hitung jumlah yang disetujui (LTV ratio)
    let amount_approved = (collateral_value_btc * params.loan_to_value_ratio) / 100;

    // 9. Validasi jumlah yang diminta
    if amount_requested > amount_approved {
        return Err(format!(
            "Requested amount {} exceeds approved amount {} based on collateral value",
            amount_requested, amount_approved
        ));
    }

    // 10. Buat loan baru
    let loan_id = get_next_loan_id();

    let loan = Loan {
        id: loan_id,
        borrower: caller,
        nft_id,
        collateral_value_btc,
        amount_requested,
        amount_approved,
        apr: params.base_apr,
        status: LoanStatus::PendingApproval,
        created_at: time(),
        due_date: None,
        total_repaid: 0,
        repayment_history: Vec::new(),
        last_payment_date: None,
    };

    // 11. Simpan loan
    store_loan(loan.clone())?;

    // 12. Send notification to borrower about loan application
    let mut additional_data = HashMap::new();
    additional_data.insert("amount".to_string(), amount_approved.to_string());
    additional_data.insert("collateral_value".to_string(), collateral_value_btc.to_string());
    
    let _ = notify_loan_event(
        caller,
        loan_id,
        "application_submitted",
        Some(additional_data),
    ); // Don't fail if notification fails

    // 13. Log audit
    log_audit_action(
        caller,
        "LOAN_APPLICATION_SUBMITTED".to_string(),
        format!("Loan #{} submitted for NFT #{} with amount {}", loan_id, nft_id, amount_requested),
        true,
    );

    Ok(loan)
}

// Accept loan offer
#[update]
pub async fn accept_loan_offer(loan_id: u64) -> Result<String, String> {
    let caller = ic_cdk::caller();

    // 1. Ambil data pinjaman
    let mut loan = get_loan(loan_id).ok_or_else(|| "Loan not found".to_string())?;

    // 2. Verifikasi caller adalah peminjam
    if loan.borrower != caller {
        return Err("Unauthorized: You are not the borrower of this loan".to_string());
    }

    // 3. Verifikasi status pinjaman
    if loan.status != LoanStatus::PendingApproval {
        return Err("Loan is not in pending approval status".to_string());
    }

    // 4. Lock NFT sebagai escrow
    match lock_nft_for_loan(loan.nft_id, loan_id) {
        Ok(_) => {
            loan.status = LoanStatus::Approved;
        }
        Err(e) => return Err(format!("Failed to lock NFT as collateral: {}", e)),
    }

    // 5. Set tanggal jatuh tempo
    let params = get_protocol_parameters();
    loan.due_date = Some(
        time() + (params.max_loan_duration_days * 24 * 60 * 60 * 1_000_000_000)
    );

    // 6. Coba cairkan dana via liquidity management
    // First, get the borrower's Bitcoin address (this would need to be stored in user profile)
    let borrower_btc_address = get_user_btc_address(&caller)
        .ok_or("Borrower Bitcoin address not found. Please update your profile.".to_string())?;
    
    match crate::liquidity_management::disburse_loan(loan_id, borrower_btc_address, loan.amount_approved).await {
        Ok(_) => {
            loan.status = LoanStatus::Active;
            
            // Simpan perubahan loan
            store_loan(loan.clone())?;

            // Send notification about loan approval and disbursement
            let mut approval_data = HashMap::new();
            approval_data.insert("amount".to_string(), loan.amount_approved.to_string());
            
            let _ = notify_loan_event(
                caller,
                loan_id,
                "approved",
                Some(approval_data.clone()),
            );
            
            let _ = notify_loan_event(
                caller,
                loan_id,
                "disbursed",
                Some(approval_data),
            );

            // Send notification about collateral escrow
            let mut collateral_data = HashMap::new();
            collateral_data.insert("loan_id".to_string(), loan_id.to_string());
            
            let _ = notify_collateral_event(
                caller,
                loan.nft_id,
                "escrowed",
                Some(collateral_data),
            );

            // Log audit
            log_audit_action(
                caller,
                "LOAN_ACCEPTED".to_string(),
                format!("Loan #{} accepted and disbursed via liquidity pool", loan_id),
                true,
            );

            Ok("Loan approved, collateral secured, and disbursement completed.".to_string())
        }
        Err(e) => {
            // Rollback NFT lock jika pencairan gagal
            let _ = unlock_nft(loan.nft_id);
            loan.status = LoanStatus::PendingApproval;
            store_loan(loan)?;

            log_audit_action(
                caller,
                "LOAN_DISBURSEMENT_FAILED".to_string(),
                format!("Loan #{} disbursement failed: {}", loan_id, e),
                false,
            );

            Err(format!("Disbursement failed: {}", e))
        }
    }
}

// Get loan status
#[query]
pub fn get_loan_status(loan_id: u64) -> Option<Loan> {
    get_loan(loan_id)
}

// Get user loans
#[query]
pub fn get_user_loans() -> Vec<Loan> {
    let caller = ic_cdk::caller();
    get_loans_by_borrower(caller)
}

// Get all loans (admin only)
#[query]
pub fn get_all_loans() -> Vec<Loan> {
    // Dalam implementasi nyata, tambahkan verifikasi admin
    get_all_loans_data()
}

// Repay loan - Enhanced implementation with comprehensive payment tracking
#[update]
pub async fn repay_loan(loan_id: u64, amount: u64) -> Result<RepaymentResponse, String> {
    let caller = ic_cdk::caller();

    // 1. Validasi dasar
    if amount == 0 {
        return Err("Payment amount must be greater than zero".to_string());
    }

    // 2. Ambil data pinjaman
    let mut loan = get_loan(loan_id).ok_or_else(|| "Loan not found".to_string())?;

    // 3. Verifikasi caller adalah peminjam
    if loan.borrower != caller {
        return Err("Unauthorized: You are not the borrower of this loan".to_string());
    }

    // 4. Verifikasi status pinjaman
    if loan.status != LoanStatus::Active {
        return Err(format!("Loan is not active. Current status: {:?}", loan.status));
    }

    // 5. Hitung breakdown utang saat ini
    let repayment_summary = calculate_loan_repayment_summary(&loan)?;
    
    if amount > repayment_summary.remaining_balance {
        return Err(format!(
            "Payment amount {} exceeds remaining debt {}. Maximum payable: {}", 
            amount, 
            repayment_summary.remaining_balance,
            repayment_summary.remaining_balance
        ));
    }

    // 6. Hitung breakdown pembayaran
    let payment_breakdown = calculate_payment_breakdown(&loan, amount)?;

    // 7. Proses transfer ckBTC - panggil fungsi yang sudah ada
    let transaction_id = match process_ckbtc_repayment(loan_id, amount).await {
        Ok(tx_id) => Some(tx_id.to_string()),
        Err(e) => return Err(format!("ckBTC transfer failed: {}", e)),
    };

    // 8. Update loan dengan payment baru
    let payment = Payment {
        amount,
        timestamp: time(),
        payment_type: if payment_breakdown.principal_amount > 0 && payment_breakdown.interest_amount > 0 {
            PaymentType::Mixed
        } else if payment_breakdown.principal_amount > 0 {
            PaymentType::Principal
        } else {
            PaymentType::Interest
        },
        transaction_id: transaction_id.clone(),
    };

    loan.total_repaid += amount;
    loan.repayment_history.push(payment);
    loan.last_payment_date = Some(time());

    // 9. Cek apakah sudah lunas
    let mut collateral_released = false;
    let updated_summary = calculate_loan_repayment_summary(&loan)?;
    
    if updated_summary.remaining_balance == 0 || loan.total_repaid >= repayment_summary.total_debt {
        loan.status = LoanStatus::Repaid;
        
        // Kembalikan NFT ke peminjam
        match release_collateral_to_borrower(loan.nft_id, loan.borrower).await {
            Ok(_) => {
                collateral_released = true;
                log_audit_action(
                    caller,
                    "COLLATERAL_RELEASED".to_string(),
                    format!("NFT #{} returned to borrower after loan #{} full repayment", loan.nft_id, loan_id),
                    true,
                );
            }
            Err(e) => {
                // Log error tapi jangan gagalkan pembayaran
                log_audit_action(
                    caller,
                    "COLLATERAL_RELEASE_FAILED".to_string(),
                    format!("Failed to return NFT #{} after loan #{} repayment: {}", loan.nft_id, loan_id, e),
                    false,
                );
            }
        }

        log_audit_action(
            caller,
            "LOAN_FULLY_REPAID".to_string(),
            format!("Loan #{} fully repaid. Total amount: {}", loan_id, loan.total_repaid),
            true,
        );
    } else {
        log_audit_action(
            caller,
            "LOAN_PAYMENT_PROCESSED".to_string(),
            format!("Partial payment of {} processed for loan #{}. Remaining: {}", 
                   amount, loan_id, updated_summary.remaining_balance),
            true,
        );
    }

    // 10. Simpan repayment record
    let repayment_record = RepaymentRecord {
        loan_id,
        payer: caller,
        amount,
        ckbtc_block_index: transaction_id.as_ref().map(|s| s.parse().unwrap_or(0)).unwrap_or(0),
        timestamp: time(),
        payment_breakdown: payment_breakdown.clone(),
    };
    
    store_repayment_record(repayment_record)?;

    // 11. Simpan perubahan loan
    store_loan(loan.clone())?;

    // 12. Kirim fee ke protocol treasury jika ada
    if payment_breakdown.protocol_fee_amount > 0 {
        // Ini akan diimplementasikan ketika treasury management tersedia
        log_audit_action(
            caller,
            "PROTOCOL_FEE_COLLECTED".to_string(),
            format!("Protocol fee {} collected from loan #{} payment", 
                   payment_breakdown.protocol_fee_amount, loan_id),
            true,
        );
    }

    // 13. Buat response
    Ok(RepaymentResponse {
        success: true,
        message: if loan.status == LoanStatus::Repaid {
            format!("Loan fully repaid! Collateral NFT #{} will be returned to your account.", loan.nft_id)
        } else {
            format!("Payment processed successfully. Remaining balance: {}", updated_summary.remaining_balance)
        },
        transaction_id,
        new_loan_status: loan.status,
        remaining_balance: updated_summary.remaining_balance,
        collateral_released,
    })
}

// Trigger liquidation (admin only)
#[update]
pub async fn trigger_liquidation(loan_id: u64) -> Result<String, String> {
    // Verifikasi admin access
    verify_admin_access()?;

    let mut loan = get_loan(loan_id).ok_or_else(|| "Loan not found".to_string())?;

    // Verifikasi loan eligible untuk liquidation
    if loan.status != LoanStatus::Active {
        return Err("Loan is not eligible for liquidation".to_string());
    }

    let params = get_protocol_parameters();
    let grace_period = params.grace_period_days * 24 * 60 * 60 * 1_000_000_000;
    
    if let Some(due_date) = loan.due_date {
        if time() < due_date + grace_period {
            return Err("Loan is not overdue enough for liquidation".to_string());
        }
    } else {
        return Err("Loan has no due date set".to_string());
    }

    // Update status
    loan.status = LoanStatus::Defaulted;

    // Transfer NFT ke sistem (untuk liquidation)
    match liquidate_collateral(loan.nft_id, loan_id) {
        Ok(_) => {
            // Simpan perubahan loan
            store_loan(loan.clone())?;

            // Log audit
            log_audit_action(
                caller(),
                "LOAN_LIQUIDATED".to_string(),
                format!("Loan #{} liquidated due to default", loan_id),
                true,
            );

            Ok(format!("Liquidation process initiated for loan #{}", loan_id))
        }
        Err(e) => Err(format!("Failed to liquidate collateral: {}", e)),
    }
}

// Helper functions
pub fn extract_valuation_from_metadata(metadata: &Vec<(String, MetadataValue)>) -> Result<u64, String> {
    for (key, value) in metadata {
        if key == "rwa:valuation_idr" {
            if let MetadataValue::Nat(val) = value {
                return Ok(*val);
            }
        }
    }
    Err("Valuation not found in metadata".to_string())
}

#[derive(Clone, Debug)]
pub struct CommodityInfo {
    pub commodity_type: String,
    pub quantity: u64,
    pub grade: String,
}

pub fn extract_commodity_info_from_metadata(metadata: &Vec<(String, MetadataValue)>) -> Result<CommodityInfo, String> {
    let mut commodity_type = None;
    let mut quantity = None;
    let mut grade = None;

    for (key, value) in metadata {
        match key.as_str() {
            "rwa:commodity_type" => {
                if let MetadataValue::Text(val) = value {
                    commodity_type = Some(val.clone());
                }
            }
            "rwa:quantity" => {
                if let MetadataValue::Nat(val) = value {
                    quantity = Some(*val);
                }
            }
            "rwa:grade" => {
                if let MetadataValue::Text(val) = value {
                    grade = Some(val.clone());
                }
            }
            _ => {}
        }
    }

    match (commodity_type, quantity, grade) {
        (Some(ct), Some(q), Some(g)) => Ok(CommodityInfo {
            commodity_type: ct,
            quantity: q,
            grade: g,
        }),
        _ => Err("Incomplete commodity information in metadata".to_string()),
    }
}

pub fn calculate_collateral_value_btc(
    valuation_idr: u64,
    quantity: u64,
    commodity_price: &CommodityPrice,
) -> Result<u64, String> {
    // Hitung nilai total berdasarkan kuantitas dan harga pasar
    let market_value_idr = quantity * commodity_price.price_per_unit;
    
    // Gunakan nilai yang lebih konservatif (minimum antara valuasi dan harga pasar)
    let conservative_value_idr = std::cmp::min(valuation_idr, market_value_idr);
    
    // Konversi ke satoshi (asumsi 1 BTC = 600,000,000 IDR)
    let btc_price_idr = 600_000_000u64;
    let collateral_value_satoshi = (conservative_value_idr * 100_000_000) / btc_price_idr;
    
    Ok(collateral_value_satoshi)
}

pub fn calculate_total_debt(loan: &Loan) -> Result<u64, String> {
    // Hitung total utang = pokok + bunga berdasarkan APR dan waktu
    let current_time = time();
    let loan_duration = if let Some(due_date) = loan.due_date {
        if current_time > due_date {
            // Loan is overdue, calculate from creation to due date
            due_date.saturating_sub(loan.created_at)
        } else {
            // Loan is still active, calculate from creation to now
            current_time.saturating_sub(loan.created_at)
        }
    } else {
        // No due date set, calculate from creation to now
        current_time.saturating_sub(loan.created_at)
    };
    
    // Convert nanoseconds to years (approximate)
    let years = loan_duration as f64 / (365.25 * 24.0 * 60.0 * 60.0 * 1_000_000_000.0);
    
    // Calculate interest: principal * (apr/100) * years
    let interest = (loan.amount_approved as f64 * (loan.apr as f64 / 100.0) * years) as u64;
    
    let total_debt = loan.amount_approved + interest;
    Ok(total_debt)
}

fn verify_admin_access() -> Result<(), String> {
    let caller = ic_cdk::caller();
    let config = get_canister_config();
    
    if config.admins.contains(&caller) {
        Ok(())
    } else {
        Err("Unauthorized: Admin access required".to_string())
    }
}

// ========================== REPAYMENT HELPER FUNCTIONS ==========================

/// Calculate comprehensive loan repayment summary
pub fn calculate_loan_repayment_summary(loan: &Loan) -> Result<LoanRepaymentSummary, String> {
    let total_debt = calculate_total_debt(loan)?;
    let remaining_balance = total_debt.saturating_sub(loan.total_repaid);
    
    // Calculate principal and interest breakdown
    let principal_outstanding = if loan.total_repaid < loan.amount_approved {
        loan.amount_approved.saturating_sub(loan.total_repaid)
    } else {
        0
    };
    
    let interest_outstanding = if loan.total_repaid > loan.amount_approved {
        0
    } else {
        total_debt.saturating_sub(loan.amount_approved)
    };
    
    // Check if overdue
    let current_time = time();
    let (is_overdue, days_overdue) = if let Some(due_date) = loan.due_date {
        if current_time > due_date {
            let overdue_ns = current_time - due_date;
            let days = overdue_ns / (24 * 60 * 60 * 1_000_000_000);
            (true, days)
        } else {
            (false, 0)
        }
    } else {
        (false, 0)
    };
    
    Ok(LoanRepaymentSummary {
        loan_id: loan.id,
        borrower: loan.borrower,
        total_debt,
        principal_outstanding,
        interest_outstanding,
        total_repaid: loan.total_repaid,
        remaining_balance,
        next_payment_due: loan.due_date,
        is_overdue,
        days_overdue,
    })
}

/// Calculate how payment amount should be allocated between principal, interest, and fees
pub fn calculate_payment_breakdown(loan: &Loan, payment_amount: u64) -> Result<PaymentBreakdown, String> {
    let total_debt = calculate_total_debt(loan)?;
    let interest_accrued = total_debt.saturating_sub(loan.amount_approved);
    let principal_remaining = loan.amount_approved.saturating_sub(loan.total_repaid.min(loan.amount_approved));
    let interest_remaining = if loan.total_repaid >= loan.amount_approved {
        interest_accrued.saturating_sub(loan.total_repaid - loan.amount_approved)
    } else {
        interest_accrued
    };
    
    // Protocol fee (e.g., 2% of interest portion)
    let protocol_fee_rate = 200; // 2% in basis points (2/100 * 10000)
    
    let mut breakdown = PaymentBreakdown {
        principal_amount: 0,
        interest_amount: 0,
        protocol_fee_amount: 0,
        total_amount: payment_amount,
    };
    
    let mut remaining_payment = payment_amount;
    
    // First pay interest
    if interest_remaining > 0 && remaining_payment > 0 {
        let interest_payment = remaining_payment.min(interest_remaining);
        breakdown.interest_amount = interest_payment;
        remaining_payment = remaining_payment.saturating_sub(interest_payment);
        
        // Calculate protocol fee on interest
        breakdown.protocol_fee_amount = (interest_payment * protocol_fee_rate) / 10000;
    }
    
    // Then pay principal
    if principal_remaining > 0 && remaining_payment > 0 {
        breakdown.principal_amount = remaining_payment.min(principal_remaining);
    }
    
    Ok(breakdown)
}

/// Release collateral NFT back to borrower after full repayment
pub async fn release_collateral_to_borrower(nft_id: u64, borrower: Principal) -> Result<(), String> {
    // This would call the RWA NFT canister to transfer the NFT back
    // For now, we'll call the existing unlock function
    unlock_nft(nft_id)?;
    
    // In a full implementation, this would make an inter-canister call:
    // let transfer_result = ic_cdk::call::<(TransferRequest,), (TransferResult,)>(
    //     nft_canister_id,
    //     "icrc7_transfer",
    //     (TransferRequest {
    //         from: Account { owner: ic_cdk::id(), subaccount: None },
    //         to: Account { owner: borrower, subaccount: None },
    //         token_id: nft_id,
    //         memo: Some("Collateral release after loan repayment".as_bytes().to_vec()),
    //         created_at_time: Some(time()),
    //     },)
    // ).await;
    
    Ok(())
}

// ========================== QUERY FUNCTIONS FOR REPAYMENT ==========================

/// Get loan repayment summary for a specific loan
#[query]
pub fn get_loan_repayment_summary(loan_id: u64) -> Result<LoanRepaymentSummary, String> {
    let loan = get_loan(loan_id).ok_or_else(|| "Loan not found".to_string())?;
    calculate_loan_repayment_summary(&loan)
}

/// Get repayment plan for a loan (what the borrower needs to pay)
#[query]
pub fn get_repayment_plan(loan_id: u64) -> Result<RepaymentPlan, String> {
    let loan = get_loan(loan_id).ok_or_else(|| "Loan not found".to_string())?;
    let caller = ic_cdk::caller();
    
    // Verify caller is the borrower
    if loan.borrower != caller {
        return Err("Unauthorized: You can only view your own repayment plan".to_string());
    }
    
    let summary = calculate_loan_repayment_summary(&loan)?;
    let params = get_protocol_parameters();
    
    // Calculate minimum payment (e.g., 10% of remaining balance or $100 equivalent)
    let minimum_payment_threshold = 1_000_000; // 0.01 BTC in satoshi
    let minimum_payment = (summary.remaining_balance / 10).max(minimum_payment_threshold);
    
    Ok(RepaymentPlan {
        loan_id,
        total_amount_due: summary.remaining_balance,
        principal_amount: summary.principal_outstanding,
        interest_amount: summary.interest_outstanding,
        protocol_fee: (summary.interest_outstanding * 200) / 10000, // 2% protocol fee
        due_date: loan.due_date.unwrap_or(time() + (params.max_loan_duration_days * 24 * 60 * 60 * 1_000_000_000)),
        minimum_payment,
    })
}

/// Get payment history for a loan
#[query]
pub fn get_payment_history(loan_id: u64) -> Result<Vec<Payment>, String> {
    let loan = get_loan(loan_id).ok_or_else(|| "Loan not found".to_string())?;
    let caller = ic_cdk::caller();
    
    // Verify caller is the borrower or admin
    let config = get_canister_config();
    if loan.borrower != caller && !config.admins.contains(&caller) {
        return Err("Unauthorized: You can only view your own payment history".to_string());
    }
    
    Ok(loan.repayment_history)
}

/// Check if a borrower can make early full repayment (usually allowed)
#[query]
pub fn can_make_early_repayment(loan_id: u64) -> Result<bool, String> {
    let loan = get_loan(loan_id).ok_or_else(|| "Loan not found".to_string())?;
    
    match loan.status {
        LoanStatus::Active => Ok(true),
        _ => Ok(false),
    }
}

/// Calculate early repayment amount (may include early repayment discount)
#[query]
pub fn calculate_early_repayment_amount(loan_id: u64) -> Result<u64, String> {
    let loan = get_loan(loan_id).ok_or_else(|| "Loan not found".to_string())?;
    
    if loan.status != LoanStatus::Active {
        return Err("Loan is not active".to_string());
    }
    
    let summary = calculate_loan_repayment_summary(&loan)?;
    
    // For early repayment, we might offer a small discount on interest
    // For now, just return the full amount
    Ok(summary.remaining_balance)
}
