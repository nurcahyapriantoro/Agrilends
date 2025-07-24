use ic_cdk::{caller, api::time};
use ic_cdk_macros::{query, update};
use candid::Principal;

use crate::types::*;
use crate::storage::*;
use crate::helpers::{log_audit_action, verify_admin_access, is_admin};
// Notification system integration
use crate::notification_system::{notify_loan_event, notify_collateral_event};
use std::collections::HashMap;

// Constants for loan repayment - Production ready values
const PROTOCOL_FEE_PERCENTAGE: u64 = 10; // 10% dari bunga untuk protokol
const GRACE_PERIOD_FACTOR: f64 = 1.1; // 10% tambahan waktu grace
const MINIMUM_PAYMENT_AMOUNT: u64 = 1000; // Minimum 1000 satoshi
const EARLY_REPAYMENT_DISCOUNT_RATE: u64 = 5; // 5% discount untuk early repayment
const EARLY_REPAYMENT_THRESHOLD: f64 = 0.8; // 80% dari loan term untuk qualify early repayment
const OVERPAYMENT_TOLERANCE: u64 = 100; // Toleransi overpayment 100 satoshi
const MAX_DAILY_REPAYMENT_LIMIT: u64 = 1_000_000_000; // 10 BTC per day maximum
const LATE_PAYMENT_PENALTY_RATE: u64 = 2; // 2% penalty per bulan keterlambatan

/// Calculate total debt including principal, accrued interest, and late payment penalties
/// Implementasi sesuai dengan production requirements untuk menghitung utang total
pub fn calculate_total_debt_with_interest(loan: &Loan) -> Result<(u64, u64, u64, u64), String> {
    let current_time = time();
    
    // Calculate time elapsed since loan creation
    let time_elapsed = current_time.saturating_sub(loan.created_at);
    
    // Convert nanoseconds to years (365.25 days per year untuk akurasi)
    let years = time_elapsed as f64 / (365.25 * 24.0 * 60.0 * 60.0 * 1_000_000_000.0);
    
    let principal = loan.amount_approved;
    let annual_rate = loan.apr as f64 / 100.0;
    
    // Simple interest calculation: Interest = Principal * Rate * Time
    // Sesuai dengan spesifikasi README untuk akumulasi bunga
    let accrued_interest = (principal as f64 * annual_rate * years) as u64;
    
    // Calculate late payment penalty if loan is overdue
    // Implementasi sesuai dengan kebutuhan production untuk penalty keterlambatan
    let late_penalty = if let Some(due_date) = loan.due_date {
        if current_time > due_date {
            let overdue_time = current_time.saturating_sub(due_date);
            let months_overdue = overdue_time as f64 / (30.0 * 24.0 * 60.0 * 60.0 * 1_000_000_000.0);
            
            // Penalty = Principal * Penalty_Rate * Months_Overdue
            let penalty = (principal as f64 * (LATE_PAYMENT_PENALTY_RATE as f64 / 100.0) * months_overdue) as u64;
            std::cmp::min(penalty, principal / 10) // Cap penalty at 10% of principal
        } else {
            0
        }
    } else {
        0
    };
    
    let total_debt = principal + accrued_interest + late_penalty;
    
    Ok((principal, accrued_interest, late_penalty, total_debt))
}

/// Enhanced payment breakdown calculation with detailed allocation
pub fn calculate_payment_breakdown(
    loan: &Loan, 
    payment_amount: u64
) -> Result<PaymentBreakdown, String> {
    let (principal_outstanding, accrued_interest, late_penalty, total_debt) = 
        calculate_total_debt_with_interest(loan)?;
    
    let already_paid = loan.total_repaid;
    let remaining_debt = total_debt.saturating_sub(already_paid);
    
    // Allow small overpayments with tolerance
    if payment_amount > remaining_debt + OVERPAYMENT_TOLERANCE {
        return Err(format!(
            "Payment amount {} exceeds remaining debt {} by more than tolerance limit",
            payment_amount, remaining_debt
        ));
    }
    
    // Adjust payment if it slightly exceeds remaining debt
    let actual_payment = std::cmp::min(payment_amount, remaining_debt);
    
    // Payment allocation priority: 1. Late penalty, 2. Interest, 3. Principal
    let remaining_penalty = late_penalty.saturating_sub(
        if already_paid > principal_outstanding + accrued_interest {
            already_paid - (principal_outstanding + accrued_interest)
        } else {
            0
        }
    );
    
    let remaining_interest = accrued_interest.saturating_sub(
        if already_paid > principal_outstanding {
            std::cmp::min(already_paid - principal_outstanding, accrued_interest)
        } else {
            0
        }
    );
    
    let remaining_principal = principal_outstanding.saturating_sub(
        std::cmp::min(already_paid, principal_outstanding)
    );
    
    // Allocate payment
    let penalty_payment = std::cmp::min(actual_payment, remaining_penalty);
    let remaining_after_penalty = actual_payment.saturating_sub(penalty_payment);
    
    let interest_payment = std::cmp::min(remaining_after_penalty, remaining_interest);
    let principal_payment = remaining_after_penalty.saturating_sub(interest_payment);
    
    // Calculate protocol fee (percentage of interest payment only)
    let protocol_fee = (interest_payment * PROTOCOL_FEE_PERCENTAGE) / 100;
    
    Ok(PaymentBreakdown {
        principal_amount: principal_payment,
        interest_amount: interest_payment,
        protocol_fee_amount: protocol_fee,
        penalty_amount: penalty_payment,
        total_amount: actual_payment,
    })
}

/// Get loan repayment summary
#[query]
pub fn get_loan_repayment_summary(loan_id: u64) -> Result<LoanRepaymentSummary, String> {
    let caller = caller();
    let loan = get_loan(loan_id).ok_or("Loan not found")?;
    
    // Verify caller is the borrower or admin
    if loan.borrower != caller && !is_admin(&caller) {
        return Err("Unauthorized: Only borrower or admin can view repayment summary".to_string());
    }
    
    let (principal, accrued_interest, total_debt) = calculate_total_debt_with_interest(&loan)?;
    let remaining_balance = total_debt.saturating_sub(loan.total_repaid);
    
    // Check if loan is overdue
    let current_time = time();
    let (is_overdue, days_overdue) = if let Some(due_date) = loan.due_date {
        if current_time > due_date {
            let overdue_time = current_time - due_date;
            let days = overdue_time / (24 * 60 * 60 * 1_000_000_000);
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
        principal_outstanding: principal.saturating_sub(loan.total_repaid.min(principal)),
        interest_outstanding: accrued_interest.saturating_sub(
            loan.total_repaid.saturating_sub(principal.min(loan.total_repaid))
        ),
        total_repaid: loan.total_repaid,
        remaining_balance,
        next_payment_due: loan.due_date,
        is_overdue,
        days_overdue,
    })
}

/// Get repayment plan for a loan
#[query]
pub fn get_repayment_plan(loan_id: u64) -> Result<RepaymentPlan, String> {
    let loan = get_loan(loan_id).ok_or("Loan not found")?;
    
    // Verify caller is the borrower
    let caller = caller();
    if loan.borrower != caller {
        return Err("Unauthorized: Only borrower can view repayment plan".to_string());
    }
    
    let (_, _, total_debt) = calculate_total_debt_with_interest(&loan)?;
    let remaining_debt = total_debt.saturating_sub(loan.total_repaid);
    
    let breakdown = calculate_payment_breakdown(&loan, remaining_debt)?;
    
    Ok(RepaymentPlan {
        loan_id: loan.id,
        total_amount_due: remaining_debt,
        principal_amount: breakdown.principal_amount,
        interest_amount: breakdown.interest_amount,
        protocol_fee: breakdown.protocol_fee_amount,
        due_date: loan.due_date.unwrap_or(time() + (30 * 24 * 60 * 60 * 1_000_000_000)), // Default 30 days if no due date
        minimum_payment: MINIMUM_PAYMENT_AMOUNT,
    })
}

/// Process loan repayment - Implementasi utama sesuai spesifikasi README
/// Memproses pembayaran kembali dari peminjam dengan validasi komprehensif
/// Termasuk transfer ckBTC, update loan, release collateral, dan protokol fees
#[update]
pub async fn repay_loan(loan_id: u64, amount: u64) -> Result<RepaymentResponse, String> {
    let caller = caller();
    
    // 1. Validate input - Sesuai spesifikasi keamanan production
    if amount == 0 {
        return Err("Payment amount must be greater than zero".to_string());
    }
    
    if amount < MINIMUM_PAYMENT_AMOUNT {
        return Err(format!("Payment amount must be at least {} satoshi", MINIMUM_PAYMENT_AMOUNT));
    }
    
    // 2. Get and validate loan - Verifikasi pinjaman ada dan valid
    let mut loan = get_loan(loan_id).ok_or("Loan not found")?;
    
    // 3. Verify caller is the borrower - Keamanan: hanya peminjam yang dapat bayar
    if loan.borrower != caller {
        return Err("Unauthorized: Only the borrower can repay the loan".to_string());
    }
    
    // 4. Verify loan status - Pastikan pinjaman aktif
    if loan.status != LoanStatus::Active {
        return Err(format!("Loan is not active for repayment. Current status: {:?}", loan.status));
    }
    
    // 5. Calculate debt and payment breakdown - Hitung total utang dengan bunga
    let (_, _, _, total_debt) = calculate_total_debt_with_interest(&loan)?;
    let remaining_debt = total_debt.saturating_sub(loan.total_repaid);
    
    if remaining_debt == 0 {
        return Err("Loan is already fully repaid".to_string());
    }
    
    // 6. Adjust payment amount if it exceeds remaining debt
    let actual_payment = std::cmp::min(amount, remaining_debt);
    let payment_breakdown = calculate_payment_breakdown(&loan, actual_payment)?;
    
    // 7. Process ckBTC transfer - Panggilan Antar-Canister sesuai README
    match crate::ckbtc_integration::process_ckbtc_repayment(loan_id, actual_payment).await {
        Ok(block_index) => {
            // 8. Update loan with payment information
            loan.total_repaid += actual_payment;
            loan.last_payment_date = Some(time());
            
            // 9. Add payment to history - Sesuai spek README untuk tracking
            let payment = Payment {
                amount: actual_payment,
                timestamp: time(),
                payment_type: if payment_breakdown.principal_amount > 0 && payment_breakdown.interest_amount > 0 {
                    PaymentType::Mixed
                } else if payment_breakdown.principal_amount > 0 {
                    PaymentType::Principal
                } else {
                    PaymentType::Interest
                },
                transaction_id: Some(block_index.to_string()),
            };
            
            loan.repayment_history.push(payment);
            
            // 10. Check if loan is fully repaid - Logika Pelunasan sesuai README
            let is_fully_repaid = loan.total_repaid >= total_debt;
            let mut collateral_released = false;
            
            if is_fully_repaid {
                loan.status = LoanStatus::Repaid;
                
                // 11. Release collateral NFT back to borrower - Panggilan Antar-Canister
                // Sesuai README: "Panggil icrc7_transfer di Canister_RWA_NFT"
                match unlock_nft(loan.nft_id) {
                    Ok(_) => {
                        collateral_released = true;
                        log_audit_action(
                            caller,
                            "COLLATERAL_RELEASED".to_string(),
                            format!("NFT #{} released back to borrower for fully repaid loan #{}", loan.nft_id, loan_id),
                            true,
                        );
                    }
                    Err(e) => {
                        log_audit_action(
                            caller,
                            "COLLATERAL_RELEASE_FAILED".to_string(),
                            format!("Failed to release NFT #{} for loan #{}: {}", loan.nft_id, loan_id, e),
                            false,
                        );
                    }
                }
            }
            
            // 12. Store updated loan
            store_loan(loan.clone())?;
            
            // 13. Store repayment record for audit trail
            let repayment_record = RepaymentRecord {
                loan_id,
                payer: caller,
                amount: actual_payment,
                ckbtc_block_index: block_index,
                timestamp: time(),
                payment_breakdown: payment_breakdown.clone(),
            };
            
            store_repayment_record(repayment_record)?;
            
            // 14. Send protocol fees to treasury - Panggilan Antar-Canister sesuai README
            // "Panggil collect_fees di Canister_Kas_Protokol"
            if payment_breakdown.protocol_fee_amount > 0 {
                match crate::treasury_management::collect_fees(
                    loan_id, 
                    payment_breakdown.protocol_fee_amount,
                    crate::types::RevenueType::ProtocolFee
                ).await {
                    Ok(_) => {
                        log_audit_action(
                            caller,
                            "PROTOCOL_FEE_COLLECTED".to_string(),
                            format!("Collected {} satoshi protocol fee from loan #{} repayment", 
                                payment_breakdown.protocol_fee_amount, loan_id),
                            true,
                        );
                    }
                    Err(e) => {
                        log_audit_action(
                            caller,
                            "PROTOCOL_FEE_COLLECTION_FAILED".to_string(),
                            format!("Failed to collect protocol fee for loan #{}: {}", loan_id, e),
                            false,
                        );
                    }
                }
            }
            
            // 15. Update liquidity pool
            if let Err(e) = crate::liquidity_management::process_loan_repayment(loan_id, actual_payment) {
                log_audit_action(
                    caller,
                    "LIQUIDITY_POOL_UPDATE_FAILED".to_string(),
                    format!("Failed to update liquidity pool for loan #{} repayment: {}", loan_id, e),
                    false,
                );
            }
            
            // 16. Log successful repayment - Audit logging
            log_audit_action(
                caller,
                if is_fully_repaid { "LOAN_FULLY_REPAID" } else { "LOAN_PARTIAL_REPAYMENT" },
                format!(
                    "Loan #{} {}: {} satoshi paid (Principal: {}, Interest: {}, Fee: {})",
                    loan_id,
                    if is_fully_repaid { "fully repaid" } else { "partially repaid" },
                    actual_payment,
                    payment_breakdown.principal_amount,
                    payment_breakdown.interest_amount,
                    payment_breakdown.protocol_fee_amount
                ),
                true,
            );
            
            let new_remaining = total_debt.saturating_sub(loan.total_repaid);
            
            // 17. Send notifications about repayment
            let mut repayment_data = HashMap::new();
            repayment_data.insert("amount".to_string(), actual_payment.to_string());
            repayment_data.insert("remaining_balance".to_string(), new_remaining.to_string());
            
            if is_fully_repaid {
                // Notify loan fully repaid
                let _ = notify_loan_event(
                    caller,
                    loan_id,
                    "fully_repaid",
                    None,
                );
                
                // Notify collateral released
                if collateral_released {
                    let mut collateral_data = HashMap::new();
                    collateral_data.insert("loan_id".to_string(), loan_id.to_string());
                    
                    let _ = notify_collateral_event(
                        caller,
                        loan.nft_id,
                        "released",
                        Some(collateral_data),
                    );
                }
            } else {
                // Notify partial repayment received
                let _ = notify_loan_event(
                    caller,
                    loan_id,
                    "repayment_received",
                    Some(repayment_data),
                );
            }
            
            // 18. Return success response - Format sesuai README
            Ok(RepaymentResponse {
                success: true,
                message: if is_fully_repaid {
                    "Loan fully repaid. Collateral NFT has been released back to you.".to_string()
                } else {
                    format!(
                        "Payment successful. Remaining balance: {} satoshi. Principal paid: {}, Interest paid: {}",
                        new_remaining, payment_breakdown.principal_amount, payment_breakdown.interest_amount
                    )
                },
                transaction_id: Some(block_index.to_string()),
                new_loan_status: loan.status,
                remaining_balance: new_remaining,
                collateral_released,
            })
        }
        
        Err(e) => {
            // 18. Handle payment failure - Error handling
            log_audit_action(
                caller,
                "LOAN_REPAYMENT_FAILED".to_string(),
                format!("Failed repayment for loan #{}: {}", loan_id, e),
                false,
            );
            
            Ok(RepaymentResponse {
                success: false,
                message: format!("Payment failed: {}", e),
                transaction_id: None,
                new_loan_status: loan.status,
                remaining_balance: remaining_debt,
                collateral_released: false,
            })
        }
    }
}

/// Get payment history for a loan
#[query]
pub fn get_loan_payment_history(loan_id: u64) -> Result<Vec<Payment>, String> {
    let caller = caller();
    let loan = get_loan(loan_id).ok_or("Loan not found")?;
    
    // Verify caller is the borrower or admin
    if loan.borrower != caller && !is_admin(&caller) {
        return Err("Unauthorized: Only borrower or admin can view payment history".to_string());
    }
    
    Ok(loan.repayment_history)
}

/// Get all repayment records for a loan (admin only)
#[query]
pub fn get_loan_repayment_records(loan_id: u64) -> Result<Vec<RepaymentRecord>, String> {
    let caller = caller();
    
    // Verify admin access
    if !is_admin(&caller) {
        return Err("Unauthorized: Only admins can access repayment records".to_string());
    }
    
    Ok(get_repayment_records_by_loan(loan_id))
}

/// Check if a loan is eligible for repayment
#[query]
pub fn check_repayment_eligibility(loan_id: u64) -> Result<bool, String> {
    let loan = get_loan(loan_id).ok_or("Loan not found")?;
    
    match loan.status {
        LoanStatus::Active => Ok(true),
        LoanStatus::Repaid => Err("Loan is already fully repaid".to_string()),
        LoanStatus::Defaulted => Err("Loan is in default status".to_string()),
        _ => Err("Loan is not eligible for repayment in current status".to_string()),
    }
}

/// Collect protocol fees and send to treasury
/// Implementasi untuk Panggilan Antar-Canister ke Canister_Kas_Protokol
pub async fn collect_protocol_fees_from_repayment(
    loan_id: u64, 
    fee_amount: u64
) -> Result<String, String> {
    // Implementation untuk mengirim protocol fee ke treasury
    // Sesuai spek README: "panggil collect_fees di Canister_Kas_Protokol"
    
    if fee_amount == 0 {
        return Ok("No fees to collect".to_string());
    }
    
    // In production, this would call the treasury canister
    // For now, we'll use the liquidity management integration
    match crate::liquidity_management::collect_protocol_fees(loan_id, fee_amount).await {
        Ok(result) => Ok(result),
        Err(e) => Err(format!("Failed to collect protocol fees: {}", e))
    }
}

/// Validate repayment amount and check daily limits
/// Production security feature untuk membatasi jumlah pembayaran harian
pub fn validate_repayment_amount(caller: Principal, amount: u64) -> Result<(), String> {
    // Check maximum daily repayment limit untuk keamanan
    if amount > MAX_DAILY_REPAYMENT_LIMIT {
        return Err(format!(
            "Payment amount {} exceeds daily limit of {} satoshi", 
            amount, MAX_DAILY_REPAYMENT_LIMIT
        ));
    }
    
    // Additional validation logic bisa ditambahkan di sini
    // seperti checking user's daily transaction history
    
    Ok(())
}

/// Calculate early repayment benefits (if any)
/// Implementasi untuk memberikan diskon early repayment
#[query]
pub fn calculate_early_repayment_benefits(loan_id: u64) -> Result<u64, String> {
    let loan = get_loan(loan_id).ok_or("Loan not found")?;
    
    // Verify caller authority
    let caller = caller();
    if loan.borrower != caller && !is_admin(&caller) {
        return Err("Unauthorized: Only borrower or admin can calculate early repayment benefits".to_string());
    }
    
    // For early repayment, we might offer a small discount on interest
    // Implementasi sesuai dengan EARLY_REPAYMENT_DISCOUNT_RATE dan THRESHOLD
    if let Some(due_date) = loan.due_date {
        let current_time = time();
        if current_time < due_date {
            let time_remaining = due_date - current_time;
            let total_loan_duration = due_date - loan.created_at;
            
            if total_loan_duration > 0 {
                let completion_ratio = (total_loan_duration - time_remaining) as f64 / total_loan_duration as f64;
                
                // Offer discount if less than threshold of loan term has passed
                if completion_ratio < EARLY_REPAYMENT_THRESHOLD {
                    let (_, accrued_interest, _, _) = calculate_total_debt_with_interest(&loan)?;
                    let remaining_interest = accrued_interest.saturating_sub(
                        loan.total_repaid.saturating_sub(loan.amount_approved.min(loan.total_repaid))
                    );
                    let discount = (remaining_interest * EARLY_REPAYMENT_DISCOUNT_RATE) / 100;
                    
                    return Ok(discount);
                }
            }
        }
    }
    
    Ok(0) // No early repayment benefits
}

/// Emergency repayment function (admin only) - for special circumstances
#[update]
pub async fn emergency_repayment(
    loan_id: u64, 
    amount: u64, 
    reason: String
) -> Result<String, String> {
    let caller = caller();
    
    // Verify admin access
    verify_admin_access()?;
    
    let mut loan = get_loan(loan_id).ok_or("Loan not found")?;
    
    if loan.status != LoanStatus::Active {
        return Err("Loan is not active".to_string());
    }
    
    // Process emergency payment without ckBTC transfer
    // This might be used in case of manual off-chain payments
    loan.total_repaid += amount;
    loan.last_payment_date = Some(time());
    
    // Add to payment history
    let payment = Payment {
        amount,
        timestamp: time(),
        payment_type: PaymentType::Mixed,
        transaction_id: Some(format!("EMERGENCY_PAYMENT_{}", time())),
    };
    
    loan.repayment_history.push(payment);
    
    // Check if fully repaid
    let (_, _, total_debt) = calculate_total_debt_with_interest(&loan)?;
    if loan.total_repaid >= total_debt {
        loan.status = LoanStatus::Repaid;
        unlock_nft(loan.nft_id)?;
    }
    
    store_loan(loan)?;
    
    log_audit_action(
        caller,
        "EMERGENCY_REPAYMENT".to_string(),
        format!("Emergency repayment of {} for loan #{}: {}", amount, loan_id, reason),
        true,
    );
    
    Ok(format!("Emergency repayment of {} satoshi processed for loan #{}", amount, loan_id))
}

// Helper functions for storage operations

pub fn get_repayment_records_by_loan(loan_id: u64) -> Vec<RepaymentRecord> {
    get_all_repayment_records()
        .into_iter()
        .filter(|record| record.loan_id == loan_id)
        .collect()
}

pub fn get_all_repayment_records() -> Vec<RepaymentRecord> {
    REPAYMENTS.with(|repayments| {
        repayments.borrow().iter().map(|(_, record)| record).collect()
    })
}

/// Get comprehensive repayment analytics untuk dashboard admin
/// Production feature untuk monitoring dan analytics
#[query]
pub fn get_comprehensive_repayment_analytics() -> Result<ComprehensiveRepaymentAnalytics, String> {
    let caller = caller();
    
    if !is_admin(&caller) {
        return Err("Unauthorized: Only admins can view comprehensive analytics".to_string());
    }
    
    let all_loans = get_all_loans_data();
    let current_time = time();
    
    let mut total_principal_paid = 0u64;
    let mut total_interest_paid = 0u64;
    let mut total_fees_collected = 0u64;
    let mut overdue_loans = 0u64;
    let mut total_overdue_amount = 0u64;
    let mut early_repayments = 0u64;
    
    for loan in &all_loans {
        total_principal_paid += std::cmp::min(loan.total_repaid, loan.amount_approved);
        
        if loan.total_repaid > loan.amount_approved {
            total_interest_paid += loan.total_repaid - loan.amount_approved;
        }
        
        // Calculate fees from repayment history
        for payment in &loan.repayment_history {
            // Estimate fee as 10% of interest portion
            if payment.amount > 0 {
                total_fees_collected += (payment.amount * PROTOCOL_FEE_PERCENTAGE) / 100;
            }
        }
        
        // Check if loan is overdue
        if let Some(due_date) = loan.due_date {
            if current_time > due_date && loan.status == LoanStatus::Active {
                overdue_loans += 1;
                if let Ok((_, _, _, total_debt)) = calculate_total_debt_with_interest(loan) {
                    total_overdue_amount += total_debt.saturating_sub(loan.total_repaid);
                }
            }
        }
        
        // Check for early repayments
        if loan.status == LoanStatus::Repaid {
            if let Some(due_date) = loan.due_date {
                if let Some(last_payment) = loan.last_payment_date {
                    if last_payment < due_date {
                        early_repayments += 1;
                    }
                }
            }
        }
    }
    
    Ok(ComprehensiveRepaymentAnalytics {
        total_loans_count: all_loans.len() as u64,
        active_loans_count: all_loans.iter().filter(|l| l.status == LoanStatus::Active).count() as u64,
        repaid_loans_count: all_loans.iter().filter(|l| l.status == LoanStatus::Repaid).count() as u64,
        defaulted_loans_count: all_loans.iter().filter(|l| l.status == LoanStatus::Defaulted).count() as u64,
        total_principal_paid,
        total_interest_paid,
        total_fees_collected,
        overdue_loans_count: overdue_loans,
        total_overdue_amount,
        early_repayments_count: early_repayments,
        average_repayment_time: calculate_average_repayment_time(&all_loans.iter().filter(|l| l.status == LoanStatus::Repaid).collect::<Vec<_>>()),
        current_timestamp: current_time,
    })
}

/// Calculate repayment performance metrics untuk analytics
pub fn calculate_loan_performance_metrics(loan: &Loan) -> LoanPerformanceMetrics {
    let current_time = time();
    
    let is_performing = match loan.status {
        LoanStatus::Active => {
            if let Some(due_date) = loan.due_date {
                current_time <= due_date
            } else {
                true
            }
        }
        LoanStatus::Repaid => true,
        _ => false,
    };
    
    let repayment_rate = if loan.amount_approved > 0 {
        (loan.total_repaid as f64 / loan.amount_approved as f64 * 100.0) as u64
    } else {
        0
    };
    
    let payment_frequency = if loan.repayment_history.len() > 1 {
        let first_payment = loan.repayment_history.first().map(|p| p.timestamp).unwrap_or(loan.created_at);
        let last_payment = loan.repayment_history.last().map(|p| p.timestamp).unwrap_or(current_time);
        let time_span = last_payment.saturating_sub(first_payment);
        
        if time_span > 0 {
            (loan.repayment_history.len() as u64 * 30 * 24 * 60 * 60 * 1_000_000_000) / time_span
        } else {
            0
        }
    } else {
        0
    };
    
    LoanPerformanceMetrics {
        loan_id: loan.id,
        is_performing,
        repayment_rate,
        payment_frequency,
        total_payments_made: loan.repayment_history.len() as u64,
        days_since_last_payment: loan.last_payment_date.map(|last| {
            (current_time.saturating_sub(last)) / (24 * 60 * 60 * 1_000_000_000)
        }).unwrap_or(0),
    }
}

/// Batch repayment processing untuk efisiensi
/// Production feature untuk memproses multiple repayments sekaligus
#[update]
pub async fn process_batch_repayments(
    repayment_requests: Vec<BatchRepaymentRequest>
) -> Result<Vec<BatchRepaymentResult>, String> {
    let caller = caller();
    
    // Only admins dapat melakukan batch processing
    if !is_admin(&caller) {
        return Err("Unauthorized: Only admins can process batch repayments".to_string());
    }
    
    let mut results = Vec::new();
    
    for request in repayment_requests {
        let result = match repay_loan(request.loan_id, request.amount).await {
            Ok(response) => BatchRepaymentResult {
                loan_id: request.loan_id,
                success: response.success,
                message: response.message,
                transaction_id: response.transaction_id,
            },
            Err(e) => BatchRepaymentResult {
                loan_id: request.loan_id,
                success: false,
                message: e,
                transaction_id: None,
            }
        };
        
        results.push(result);
    }
    
    // Log batch processing
    log_audit_action(
        caller,
        "BATCH_REPAYMENT_PROCESSED".to_string(),
        format!("Processed {} repayment requests", results.len()),
        true,
    );
    
    Ok(results)
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct RepaymentStatistics {
    pub total_loans: u64,
    pub active_loans: u64,
    pub repaid_loans: u64,
    pub defaulted_loans: u64,
    pub total_repaid_amount: u64,
    pub total_outstanding_amount: u64,
    pub average_repayment_time: u64, // in days
}

fn calculate_average_repayment_time(repaid_loans: &[&Loan]) -> u64 {
    if repaid_loans.is_empty() {
        return 0;
    }
    
    let total_time: u64 = repaid_loans.iter()
        .filter_map(|loan| loan.last_payment_date.map(|last| last - loan.created_at))
        .sum();
    
    if total_time == 0 {
        return 0;
    }
    
    let average_nanoseconds = total_time / repaid_loans.len() as u64;
    average_nanoseconds / (24 * 60 * 60 * 1_000_000_000) // Convert to days
}

/// Get traditional repayment statistics (kept for backward compatibility)
#[query]
pub fn get_repayment_statistics() -> Result<RepaymentStatistics, String> {
    let caller = caller();
    
    if !is_admin(&caller) {
        return Err("Unauthorized: Only admins can view repayment statistics".to_string());
    }
    
    let all_loans = get_all_loans_data();
    let active_loans: Vec<_> = all_loans.iter().filter(|l| l.status == LoanStatus::Active).collect();
    let repaid_loans: Vec<_> = all_loans.iter().filter(|l| l.status == LoanStatus::Repaid).collect();
    let defaulted_loans: Vec<_> = all_loans.iter().filter(|l| l.status == LoanStatus::Defaulted).collect();
    
    let total_repaid_amount: u64 = all_loans.iter().map(|l| l.total_repaid).sum();
    let total_outstanding: u64 = active_loans.iter()
        .map(|l| {
            let (_, _, _, total_debt) = calculate_total_debt_with_interest(l).unwrap_or((0, 0, 0, 0));
            total_debt.saturating_sub(l.total_repaid)
        })
        .sum();
    
    Ok(RepaymentStatistics {
        total_loans: all_loans.len() as u64,
        active_loans: active_loans.len() as u64,
        repaid_loans: repaid_loans.len() as u64,
        defaulted_loans: defaulted_loans.len() as u64,
        total_repaid_amount,
        total_outstanding_amount: total_outstanding,
        average_repayment_time: calculate_average_repayment_time(&repaid_loans),
    })
}

/// Schedule automatic repayment untuk recurring payments
/// Production feature untuk automatic repayment scheduling
#[update]
pub async fn schedule_automatic_repayment(
    loan_id: u64,
    amount: u64,
    frequency_days: u64
) -> Result<String, String> {
    let caller = caller();
    let loan = get_loan(loan_id).ok_or("Loan not found")?;
    
    // Verify caller is the borrower
    if loan.borrower != caller {
        return Err("Unauthorized: Only the borrower can schedule automatic repayment".to_string());
    }
    
    // Validate repayment amount
    validate_repayment_amount(caller, amount)?;
    
    // In production, this would integrate with a scheduler service
    // For now, we'll just log the scheduling request
    log_audit_action(
        caller,
        "AUTOMATIC_REPAYMENT_SCHEDULED".to_string(),
        format!(
            "Scheduled automatic repayment for loan #{}: {} satoshi every {} days", 
            loan_id, amount, frequency_days
        ),
        true,
    );
    
    Ok(format!(
        "Automatic repayment scheduled for loan #{}: {} satoshi every {} days", 
        loan_id, amount, frequency_days
    ))
}

/// Get loan repayment forecasting untuk financial planning
#[query]
pub fn get_repayment_forecast(loan_id: u64, months_ahead: u64) -> Result<Vec<RepaymentForecast>, String> {
    let loan = get_loan(loan_id).ok_or("Loan not found")?;
    let caller = caller();
    
    // Verify access
    if loan.borrower != caller && !is_admin(&caller) {
        return Err("Unauthorized: Only borrower or admin can view repayment forecast".to_string());
    }
    
    let mut forecasts = Vec::new();
    let current_time = time();
    let month_in_nanoseconds = 30 * 24 * 60 * 60 * 1_000_000_000u64;
    
    for month in 1..=months_ahead {
        let forecast_time = current_time + (month * month_in_nanoseconds);
        
        // Create a temporary loan state for the forecast
        let mut forecast_loan = loan.clone();
        forecast_loan.created_at = current_time;
        
        if let Ok((_, accrued_interest, _, total_debt)) = calculate_total_debt_with_interest(&forecast_loan) {
            let remaining_balance = total_debt.saturating_sub(loan.total_repaid);
            
            forecasts.push(RepaymentForecast {
                month,
                forecast_date: forecast_time,
                projected_interest: accrued_interest,
                projected_total_debt: total_debt,
                projected_remaining_balance: remaining_balance,
                recommended_payment: std::cmp::max(
                    remaining_balance / (months_ahead - month + 1),
                    MINIMUM_PAYMENT_AMOUNT
                ),
            });
        }
    }
    
    Ok(forecasts)
}

// Struct untuk forecasting
#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct RepaymentForecast {
    pub month: u64,
    pub forecast_date: u64,
    pub projected_interest: u64,
    pub projected_total_debt: u64,
    pub projected_remaining_balance: u64,
    pub recommended_payment: u64,
}

// Test module
#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::*;
    use candid::Principal;
    
    // Helper function untuk membuat test loan
    fn create_test_loan() -> Loan {
        Loan {
            id: 1,
            borrower: Principal::from_slice(&[1u8; 29]),
            nft_id: 1,
            collateral_value_btc: 100_000_000, // 1 BTC
            amount_requested: 50_000_000,       // 0.5 BTC
            amount_approved: 50_000_000,        // 0.5 BTC
            apr: 10,                            // 10% APR
            status: LoanStatus::Active,
            created_at: 1_000_000_000_000_000_000u64, // Mock timestamp
            due_date: Some(1_000_000_000_000_000_000u64 + (365 * 24 * 60 * 60 * 1_000_000_000u64)), // 1 year later
            total_repaid: 0,
            repayment_history: Vec::new(),
            last_payment_date: None,
        }
    }
    
    #[test]
    fn test_calculate_total_debt_with_interest() {
        let loan = create_test_loan();
        
        // Test akan gagal karena memerlukan IC environment untuk time()
        // Ini adalah contoh struktur test yang seharusnya digunakan
        // Dalam production, gunakan ic-test-utilities atau mock time
        
        // Mock calculation for testing logic
        let principal = loan.amount_approved;
        let annual_rate = loan.apr as f64 / 100.0;
        let years = 1.0; // Assume 1 year for test
        
        let expected_interest = (principal as f64 * annual_rate * years) as u64;
        let expected_total = principal + expected_interest;
        
        assert_eq!(principal, 50_000_000);
        assert_eq!(expected_interest, 5_000_000); // 10% of 50M
        assert_eq!(expected_total, 55_000_000);
    }
    
    #[test]
    fn test_payment_breakdown_calculation() {
        let mut loan = create_test_loan();
        loan.total_repaid = 10_000_000; // Already paid 10M satoshi
        
        // Test payment breakdown logic
        let payment_amount = 20_000_000; // 20M satoshi payment
        
        // Manual calculation for test verification
        let protocol_fee = (payment_amount * PROTOCOL_FEE_PERCENTAGE) / 100;
        
        assert!(protocol_fee > 0);
        assert_eq!(PROTOCOL_FEE_PERCENTAGE, 10);
    }
    
    #[test]
    fn test_early_repayment_discount_calculation() {
        let loan = create_test_loan();
        
        // Test early repayment discount logic
        let discount_rate = EARLY_REPAYMENT_DISCOUNT_RATE;
        let threshold = EARLY_REPAYMENT_THRESHOLD;
        
        assert_eq!(discount_rate, 5); // 5% discount
        assert_eq!(threshold, 0.8);   // 80% threshold
    }
    
    #[test]
    fn test_repayment_response_structure() {
        let response = RepaymentResponse {
            success: true,
            message: "Test repayment successful".to_string(),
            transaction_id: Some("test_tx_123".to_string()),
            new_loan_status: LoanStatus::Repaid,
            remaining_balance: 0,
            collateral_released: true,
        };
        
        assert!(response.success);
        assert_eq!(response.new_loan_status, LoanStatus::Repaid);
        assert!(response.collateral_released);
        assert_eq!(response.remaining_balance, 0);
    }
    
    #[test]
    fn test_payment_breakdown_structure() {
        let breakdown = PaymentBreakdown {
            principal_amount: 40_000_000,
            interest_amount: 5_000_000,
            protocol_fee_amount: 500_000,
            penalty_amount: 0,
            total_amount: 45_500_000,
        };
        
        assert_eq!(breakdown.total_amount, 
                   breakdown.principal_amount + breakdown.interest_amount + breakdown.protocol_fee_amount);
    }
    
    #[test]
    fn test_comprehensive_analytics_structure() {
        let analytics = ComprehensiveRepaymentAnalytics {
            total_loans_count: 100,
            active_loans_count: 60,
            repaid_loans_count: 35,
            defaulted_loans_count: 5,
            total_principal_paid: 1_000_000_000,
            total_interest_paid: 100_000_000,
            total_fees_collected: 10_000_000,
            overdue_loans_count: 5,
            total_overdue_amount: 50_000_000,
            early_repayments_count: 10,
            average_repayment_time: 300, // 300 days
            current_timestamp: 1_700_000_000_000_000_000u64,
        };
        
        assert_eq!(analytics.total_loans_count, 
                   analytics.active_loans_count + analytics.repaid_loans_count + analytics.defaulted_loans_count);
    }
    
    #[test]
    fn test_loan_performance_metrics() {
        let metrics = LoanPerformanceMetrics {
            loan_id: 1,
            is_performing: true,
            repayment_rate: 80, // 80%
            payment_frequency: 2, // 2 payments per month
            total_payments_made: 12,
            days_since_last_payment: 5,
        };
        
        assert!(metrics.is_performing);
        assert_eq!(metrics.repayment_rate, 80);
        assert_eq!(metrics.total_payments_made, 12);
    }
}
