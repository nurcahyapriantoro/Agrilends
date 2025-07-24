// EXAMPLE: Integration dengan Loan Lifecycle Module
// File: src/loan_lifecycle.rs (contoh integrasi)

use ic_cdk::{caller, api::time};
use ic_cdk_macros::{query, update};
use crate::types::*;
use crate::storage::*;
use crate::user_management::{get_user, Role, UserResult};
use crate::helpers::{get_user_btc_address, log_audit_action};

// Import notification system
use crate::notification_system::{
    notify_loan_application_submitted,
    notify_loan_offer_ready,
    notify_loan_approved,
    notify_loan_disbursed,
    notify_loan_repayment_received,
    notify_loan_fully_repaid,
    notify_loan_overdue,
    notify_loan_liquidated,
    notify_collateral_escrowed,
    notify_collateral_released,
};

/// Submit loan application dengan notifikasi otomatis
#[update]
pub async fn submit_loan_application_with_notifications(
    nft_id: u64,
    amount_requested: u64,
) -> Result<Loan, String> {
    let caller = caller();
    
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
        return Err("NFT is already locked as collateral".to_string());
    }

    // 4. Dapatkan informasi komoditas dari metadata NFT
    let commodity_info = extract_commodity_info(&nft_data.metadata)?;
    
    // 5. Dapatkan harga komoditas dari oracle
    let commodity_price_data = get_stored_commodity_price(&commodity_info.commodity_type)
        .ok_or_else(|| "Price data not available for this commodity".to_string())?;

    // 6. Hitung nilai agunan dalam ckBTC
    let collateral_value_btc = calculate_collateral_value_btc(
        commodity_info.valuation_idr,
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
    };

    // 11. Simpan loan
    store_loan(loan.clone())?;

    // 12. Log audit
    log_audit_action(
        caller,
        "LOAN_APPLICATION_SUBMITTED".to_string(),
        format!("Loan #{} submitted for NFT #{} with amount {}", loan_id, nft_id, amount_requested),
        true,
    );

    // 13. 🔔 KIRIM NOTIFIKASI: Aplikasi pinjaman telah disubmit
    match notify_loan_application_submitted(caller, loan_id) {
        Ok(notification_id) => {
            log_audit_action(
                caller,
                "NOTIFICATION_SENT".to_string(),
                format!("Loan application notification {} sent for loan #{}", notification_id, loan_id),
                true,
            );
        }
        Err(e) => {
            // Log error tapi jangan gagalkan seluruh proses
            log_audit_action(
                caller,
                "NOTIFICATION_FAILED".to_string(),
                format!("Failed to send loan application notification for loan #{}: {}", loan_id, e),
                false,
            );
        }
    }

    // 14. 🔔 KIRIM NOTIFIKASI: Penawaran pinjaman siap
    match notify_loan_offer_ready(caller, loan_id, amount_approved) {
        Ok(notification_id) => {
            log_audit_action(
                caller,
                "NOTIFICATION_SENT".to_string(),
                format!("Loan offer ready notification {} sent for loan #{}", notification_id, loan_id),
                true,
            );
        }
        Err(e) => {
            log_audit_action(
                caller,
                "NOTIFICATION_FAILED".to_string(),
                format!("Failed to send loan offer notification for loan #{}: {}", loan_id, e),
                false,
            );
        }
    }

    Ok(loan)
}

/// Accept loan offer dengan notifikasi otomatis
#[update]
pub async fn accept_loan_offer_with_notifications(loan_id: u64) -> Result<String, String> {
    let caller = caller();

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
            
            // 🔔 KIRIM NOTIFIKASI: Pinjaman disetujui
            match notify_loan_approved(caller, loan_id) {
                Ok(notification_id) => {
                    log_audit_action(
                        caller,
                        "NOTIFICATION_SENT".to_string(),
                        format!("Loan approved notification {} sent for loan #{}", notification_id, loan_id),
                        true,
                    );
                }
                Err(e) => {
                    log_audit_action(
                        caller,
                        "NOTIFICATION_FAILED".to_string(),
                        format!("Failed to send loan approved notification for loan #{}: {}", loan_id, e),
                        false,
                    );
                }
            }

            // 🔔 KIRIM NOTIFIKASI: Agunan ditempatkan di escrow
            match notify_collateral_escrowed(caller, loan.nft_id, loan_id) {
                Ok(notification_id) => {
                    log_audit_action(
                        caller,
                        "NOTIFICATION_SENT".to_string(),
                        format!("Collateral escrowed notification {} sent for NFT #{}", notification_id, loan.nft_id),
                        true,
                    );
                }
                Err(e) => {
                    log_audit_action(
                        caller,
                        "NOTIFICATION_FAILED".to_string(),
                        format!("Failed to send collateral escrowed notification: {}", e),
                        false,
                    );
                }
            }
        }
        Err(e) => return Err(format!("Failed to lock NFT as collateral: {}", e)),
    }

    // 5. Set tanggal jatuh tempo
    let params = get_protocol_parameters();
    loan.due_date = Some(
        time() + (params.max_loan_duration_days * 24 * 60 * 60 * 1_000_000_000)
    );

    // 6. Coba cairkan dana via liquidity management
    let borrower_btc_address = get_user_btc_address(&caller)
        .ok_or("Borrower Bitcoin address not found. Please update your profile.".to_string())?;
    
    match crate::liquidity_management::disburse_loan(loan_id, borrower_btc_address, loan.amount_approved).await {
        Ok(_) => {
            loan.status = LoanStatus::Active;
            
            // Simpan perubahan loan
            store_loan(loan.clone())?;

            // 🔔 KIRIM NOTIFIKASI: Pinjaman dicairkan
            match notify_loan_disbursed(caller, loan_id, loan.amount_approved) {
                Ok(notification_id) => {
                    log_audit_action(
                        caller,
                        "NOTIFICATION_SENT".to_string(),
                        format!("Loan disbursed notification {} sent for loan #{}", notification_id, loan_id),
                        true,
                    );
                }
                Err(e) => {
                    log_audit_action(
                        caller,
                        "NOTIFICATION_FAILED".to_string(),
                        format!("Failed to send loan disbursed notification for loan #{}: {}", loan_id, e),
                        false,
                    );
                }
            }

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

/// Repay loan dengan notifikasi otomatis
#[update]
pub async fn repay_loan_with_notifications(loan_id: u64, amount: u64) -> Result<String, String> {
    let caller = caller();

    // 1. Ambil data pinjaman
    let mut loan = get_loan(loan_id).ok_or_else(|| "Loan not found".to_string())?;

    // 2. Verifikasi caller adalah peminjam
    if loan.borrower != caller {
        return Err("Unauthorized: You are not the borrower of this loan".to_string());
    }

    // 3. Verifikasi status pinjaman
    if loan.status != LoanStatus::Active {
        return Err("Loan is not active".to_string());
    }

    // 4. Hitung total utang (pokok + bunga)
    let total_debt = calculate_total_debt(&loan)?;
    let remaining_debt = total_debt.saturating_sub(loan.total_repaid);

    if amount > remaining_debt {
        return Err(format!(
            "Payment amount {} exceeds remaining debt {}",
            amount, remaining_debt
        ));
    }

    // 5. Proses pembayaran via ckBTC
    match process_ckbtc_repayment(caller, amount, loan_id).await {
        Ok(_) => {
            // Update loan data
            loan.total_repaid += amount;
            let new_remaining = total_debt.saturating_sub(loan.total_repaid);
            
            if new_remaining == 0 {
                // Loan fully repaid
                loan.status = LoanStatus::Completed;
                
                // Release collateral
                match unlock_nft(loan.nft_id) {
                    Ok(_) => {
                        // 🔔 KIRIM NOTIFIKASI: Pinjaman lunas
                        match notify_loan_fully_repaid(caller, loan_id) {
                            Ok(notification_id) => {
                                log_audit_action(
                                    caller,
                                    "NOTIFICATION_SENT".to_string(),
                                    format!("Loan fully repaid notification {} sent for loan #{}", notification_id, loan_id),
                                    true,
                                );
                            }
                            Err(e) => {
                                log_audit_action(
                                    caller,
                                    "NOTIFICATION_FAILED".to_string(),
                                    format!("Failed to send loan fully repaid notification: {}", e),
                                    false,
                                );
                            }
                        }

                        // 🔔 KIRIM NOTIFIKASI: Agunan dilepaskan
                        match notify_collateral_released(caller, loan.nft_id, loan_id) {
                            Ok(notification_id) => {
                                log_audit_action(
                                    caller,
                                    "NOTIFICATION_SENT".to_string(),
                                    format!("Collateral released notification {} sent for NFT #{}", notification_id, loan.nft_id),
                                    true,
                                );
                            }
                            Err(e) => {
                                log_audit_action(
                                    caller,
                                    "NOTIFICATION_FAILED".to_string(),
                                    format!("Failed to send collateral released notification: {}", e),
                                    false,
                                );
                            }
                        }
                    }
                    Err(e) => {
                        log_audit_action(
                            caller,
                            "COLLATERAL_RELEASE_FAILED".to_string(),
                            format!("Failed to release collateral NFT #{}: {}", loan.nft_id, e),
                            false,
                        );
                    }
                }
            } else {
                // Partial repayment
                // 🔔 KIRIM NOTIFIKASI: Pembayaran diterima
                match notify_loan_repayment_received(caller, loan_id, amount, new_remaining) {
                    Ok(notification_id) => {
                        log_audit_action(
                            caller,
                            "NOTIFICATION_SENT".to_string(),
                            format!("Loan repayment notification {} sent for loan #{}", notification_id, loan_id),
                            true,
                        );
                    }
                    Err(e) => {
                        log_audit_action(
                            caller,
                            "NOTIFICATION_FAILED".to_string(),
                            format!("Failed to send loan repayment notification: {}", e),
                            false,
                        );
                    }
                }
            }

            // Update loan record
            store_loan(loan.clone())?;

            // Update pool liquidity
            let _ = crate::liquidity_management::process_loan_repayment(loan_id, amount);

            Ok(format!(
                "Repayment successful. Loan status: {:?}. Remaining debt: {}",
                loan.status,
                total_debt.saturating_sub(loan.total_repaid)
            ))
        }
        Err(e) => Err(format!("Payment transfer failed: {}", e)),
    }
}

/// Trigger liquidation dengan notifikasi otomatis
#[update]
pub async fn trigger_liquidation_with_notifications(loan_id: u64) -> Result<String, String> {
    // Verifikasi admin access atau automated process
    verify_liquidation_access()?;

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

            // 🔔 KIRIM NOTIFIKASI: Pinjaman dilikuidasi
            match notify_loan_liquidated(loan.borrower, loan_id, vec![loan.nft_id]) {
                Ok(notification_id) => {
                    log_audit_action(
                        caller(),
                        "NOTIFICATION_SENT".to_string(),
                        format!("Loan liquidated notification {} sent for loan #{}", notification_id, loan_id),
                        true,
                    );
                }
                Err(e) => {
                    log_audit_action(
                        caller(),
                        "NOTIFICATION_FAILED".to_string(),
                        format!("Failed to send loan liquidated notification: {}", e),
                        false,
                    );
                }
            }

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

// Helper functions for verification
fn verify_liquidation_access() -> Result<(), String> {
    let caller = caller();
    
    // Allow admin or automated maintenance canister
    if crate::helpers::is_admin(&caller) || crate::helpers::is_maintenance_canister(&caller) {
        Ok(())
    } else {
        Err("Unauthorized: Only admin or maintenance canister can trigger liquidation".to_string())
    }
}

fn calculate_total_debt(loan: &Loan) -> Result<u64, String> {
    let principal = loan.amount_approved;
    
    // Calculate interest based on APR and loan duration
    let loan_duration_days = if let Some(created_at) = Some(loan.created_at) {
        (time() - created_at) / (24 * 60 * 60 * 1_000_000_000)
    } else {
        0
    };
    
    let interest = (principal * loan.apr * loan_duration_days) / (365 * 100);
    
    Ok(principal + interest)
}

/// Get loans that are overdue (for automated maintenance)
pub fn get_overdue_loans() -> Vec<Loan> {
    let current_time = time();
    let mut overdue_loans = Vec::new();
    
    // This would iterate through all active loans and check due dates
    let all_loans = get_all_loans_data();
    
    for loan in all_loans {
        if loan.status == LoanStatus::Active {
            if let Some(due_date) = loan.due_date {
                if current_time > due_date {
                    overdue_loans.push(loan);
                }
            }
        }
    }
    
    overdue_loans
}

/// Send overdue notifications (called by automated maintenance)
pub async fn send_overdue_notifications() -> Result<u64, String> {
    let overdue_loans = get_overdue_loans();
    let mut notification_count = 0u64;
    
    for loan in overdue_loans {
        let days_overdue = if let Some(due_date) = loan.due_date {
            (time() - due_date) / (24 * 60 * 60 * 1_000_000_000)
        } else {
            0
        };
        
        // Send overdue notification
        match notify_loan_overdue(loan.borrower, loan.id, days_overdue) {
            Ok(_) => {
                notification_count += 1;
                log_audit_action(
                    ic_cdk::id(), // System principal
                    "OVERDUE_NOTIFICATION_SENT".to_string(),
                    format!("Sent overdue notification for loan #{} ({} days overdue)", loan.id, days_overdue),
                    true,
                );
            }
            Err(e) => {
                log_audit_action(
                    ic_cdk::id(),
                    "OVERDUE_NOTIFICATION_FAILED".to_string(),
                    format!("Failed to send overdue notification for loan #{}: {}", loan.id, e),
                    false,
                );
            }
        }
    }
    
    Ok(notification_count)
}
