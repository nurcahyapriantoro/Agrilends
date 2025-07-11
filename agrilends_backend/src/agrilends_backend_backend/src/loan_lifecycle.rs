use ic_cdk::{caller, api::time};
use ic_cdk_macros::{query, update};
use crate::types::*;
use crate::storage::{
    get_loan, store_loan, get_next_loan_id, get_loans_by_borrower,
    get_all_loans_data, get_nft_data, lock_nft_for_loan, get_stored_commodity_price,
    get_protocol_parameters, liquidate_collateral, unlock_nft
};
use crate::user_management::{get_user, Role, UserResult};
use crate::helpers::{get_user_btc_address, log_audit_action, get_canister_config};
// Production integrations  
use crate::oracle::{is_price_stale};
use crate::ckbtc_integration::{process_ckbtc_repayment};

// Submit loan application
#[update]
pub async fn submit_loan_application(
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

    Ok(loan)
}

// Accept loan offer
#[update]
pub async fn accept_loan_offer(loan_id: u64) -> Result<String, String> {
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
    let caller = caller();
    get_loans_by_borrower(caller)
}

// Get all loans (admin only)
#[query]
pub fn get_all_loans() -> Vec<Loan> {
    // Dalam implementasi nyata, tambahkan verifikasi admin
    get_all_loans_data()
}

// Repay loan
#[update]
pub async fn repay_loan(loan_id: u64, amount: u64) -> Result<String, String> {
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
        return Err(format!("Payment amount {} exceeds remaining debt {}", amount, remaining_debt));
    }

    // 5. Proses pembayaran melalui integrasi ckBTC
    match process_ckbtc_repayment(loan_id, amount).await {
        Ok(_) => {
            // Update loan
            loan.total_repaid += amount;

            // Cek apakah sudah lunas
            if loan.total_repaid >= total_debt {
                loan.status = LoanStatus::Repaid;

                // Unlock dan kembalikan NFT ke peminjam
                unlock_nft(loan.nft_id)?;

                // Log audit
                log_audit_action(
                    caller,
                    "LOAN_REPAID".to_string(),
                    format!("Loan #{} fully repaid and NFT returned", loan_id),
                    true,
                );
            }

            // Simpan perubahan loan
            store_loan(loan.clone())?;

            Ok(format!(
                "Repayment successful. Loan status: {:?}. Remaining debt: {}",
                loan.status,
                total_debt.saturating_sub(loan.total_repaid)
            ))
        }
        Err(e) => Err(format!("Payment transfer failed: {}", e)),
    }
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
    let caller = caller();
    let config = get_canister_config();
    
    if config.admins.contains(&caller) {
        Ok(())
    } else {
        Err("Unauthorized: Admin access required".to_string())
    }
}
