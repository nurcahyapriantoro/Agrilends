use crate::types::*;
use crate::user_management::{User, USERS};
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{DefaultMemoryImpl, StableBTreeMap};
use std::cell::RefCell;
use ic_cdk::api::time;
use ic_cdk::caller;
use candid::{CandidType, Deserialize, Principal};

// Memory types
type Memory = VirtualMemory<DefaultMemoryImpl>;
type NFTStorage = StableBTreeMap<u64, RWANFTData, Memory>;
type CollateralStorage = StableBTreeMap<u64, CollateralRecord, Memory>;
type AuditLogStorage = StableBTreeMap<u64, AuditLog, Memory>;
type ConfigStorage = StableBTreeMap<u8, CanisterConfig, Memory>;
type LoanStorage = StableBTreeMap<u64, Loan, Memory>;
type ProtocolParamsStorage = StableBTreeMap<u8, ProtocolParameters, Memory>;
type OraclePriceStorage = StableBTreeMap<String, CommodityPrice, Memory>;
type RepaymentStorage = StableBTreeMap<u64, RepaymentRecord, Memory>;

// Liquidity Management Storage Types
type LiquidityPoolStorage = StableBTreeMap<u8, LiquidityPool, Memory>;
type InvestorBalanceStorage = StableBTreeMap<Principal, InvestorBalance, Memory>;
type ProcessedTransactionStorage = StableBTreeMap<u64, ProcessedTransaction, Memory>;
type EmergencyPauseStorage = StableBTreeMap<u8, bool, Memory>;
type DisbursementRecordStorage = StableBTreeMap<u64, DisbursementRecord, Memory>;
type PriceFetchTracker = StableBTreeMap<String, PriceFetchRecord, Memory>;

// Memory Manager
thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(
        MemoryManager::init(DefaultMemoryImpl::default())
    );
}

/// Helper function to get memory by ID for governance module
pub fn get_memory_by_id(id: MemoryId) -> Memory {
    MEMORY_MANAGER.with(|m| m.borrow().get(id))
}

// Storage for RWA NFTs
thread_local! {
    pub static RWA_NFTS: RefCell<NFTStorage> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(1)))
        )
    );
}

// Storage for collateral records
thread_local! {
    pub static COLLATERAL_RECORDS: RefCell<CollateralStorage> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(2)))
        )
    );
}

// Storage for audit logs
thread_local! {
    pub static AUDIT_LOGS: RefCell<AuditLogStorage> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(3)))
        )
    );
}

// Storage for configuration
thread_local! {
    pub static CONFIG_STORAGE: RefCell<ConfigStorage> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(4)))
        )
    );
}

// Storage for loans
thread_local! {
    pub static LOANS: RefCell<LoanStorage> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(5)))
        )
    );
}

// Storage for protocol parameters
thread_local! {
    pub static PROTOCOL_PARAMS: RefCell<ProtocolParamsStorage> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(6)))
        )
    );
}

// Storage for Oracle prices
thread_local! {
    pub static ORACLE_PRICES: RefCell<OraclePriceStorage> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(7)))
        )
    );
}

// Storage for loan disbursements
thread_local! {
    pub static DISBURSEMENTS: RefCell<DisbursementRecordStorage> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(8)))
        )
    );
}

// Storage for loan repayments
thread_local! {
    pub static REPAYMENTS: RefCell<RepaymentStorage> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(9)))
        )
    );
}

// Storage for liquidity management
thread_local! {
    pub static LIQUIDITY_POOL: RefCell<LiquidityPoolStorage> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(10)))
        )
    );
}

thread_local! {
    pub static INVESTOR_BALANCES: RefCell<InvestorBalanceStorage> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(11)))
        )
    );
}

thread_local! {
    pub static PROCESSED_TRANSACTIONS: RefCell<ProcessedTransactionStorage> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(12)))
        )
    );
}

thread_local! {
    pub static EMERGENCY_PAUSE: RefCell<EmergencyPauseStorage> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(13)))
        )
    );
}

// Storage for disbursement records
thread_local! {
    pub static DISBURSEMENT_RECORDS: RefCell<DisbursementRecordStorage> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(14)))
        )
    );
}

// CONFIG_STORAGE is already defined above, removing duplicate

// Remove duplicated storage aliases - these are redundant and causing confusion
// Keep only the main storage definitions above

// Storage for tracking last price fetch times
thread_local! {
    pub static PRICE_FETCH_TRACKER: RefCell<StableBTreeMap<String, PriceFetchRecord, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(15)))
        )
    );
}

// Token ID counters
thread_local! {
    static NFT_TOKEN_COUNTER: RefCell<u64> = RefCell::new(0);
    static COLLATERAL_COUNTER: RefCell<u64> = RefCell::new(0);
    pub static AUDIT_LOG_COUNTER: RefCell<u64> = RefCell::new(0);
    static LOAN_COUNTER: RefCell<u64> = RefCell::new(0);
    static DISBURSEMENT_COUNTER: RefCell<u64> = RefCell::new(0);
}

// Helper functions for token ID generation
pub fn next_nft_token_id() -> u64 {
    NFT_TOKEN_COUNTER.with(|counter| {
        let current = *counter.borrow();
        *counter.borrow_mut() = current + 1;
        current + 1
    })
}

pub fn next_collateral_id() -> u64 {
    COLLATERAL_COUNTER.with(|counter| {
        let current = *counter.borrow();
        *counter.borrow_mut() = current + 1;
        current + 1
    })
}

pub fn next_loan_id() -> u64 {
    LOAN_COUNTER.with(|counter| {
        let current = *counter.borrow();
        *counter.borrow_mut() = current + 1;
        current + 1
    })
}

pub fn next_disbursement_id() -> u64 {
    DISBURSEMENT_COUNTER.with(|counter| {
        let current = *counter.borrow();
        *counter.borrow_mut() = current + 1;
        current + 1
    })
}

// Helper function to get NFT by token ID
pub fn get_nft_by_token_id(token_id: u64) -> Option<RWANFTData> {
    RWA_NFTS.with(|nfts| nfts.borrow().get(&token_id))
}

// Helper function to get collateral by ID
pub fn get_collateral_by_id(collateral_id: u64) -> Option<CollateralRecord> {
    COLLATERAL_RECORDS.with(|records| records.borrow().get(&collateral_id))
}

// Helper function to get loan by ID
pub fn get_loan_by_id(loan_id: u64) -> Option<Loan> {
    LOANS.with(|loans| loans.borrow().get(&loan_id))
}

// Update collateral status
pub fn update_collateral_status(token_id: u64, status: CollateralStatus, loan_id: Option<u64>) {
    COLLATERAL_RECORDS.with(|records| {
        let mut records_map = records.borrow_mut();
        for (collateral_id, record) in records_map.iter() {
            if record.nft_token_id == token_id {
                let mut updated_record = record.clone();
                updated_record.status = status;
                updated_record.loan_id = loan_id;
                updated_record.updated_at = time();
                records_map.insert(collateral_id, updated_record);
                break;
            }
        }
    });
}

// Count NFTs owned by a user
pub fn count_user_nfts(owner: &Principal) -> u64 {
    RWA_NFTS.with(|nfts| {
        nfts.borrow()
            .iter()
            .filter(|(_, nft_data)| nft_data.owner == *owner)
            .count() as u64
    })
}

// Configuration management
pub fn get_config() -> CanisterConfig {
    CONFIG_STORAGE.with(|config| {
        config.borrow()
            .get(&0) // Use key 0 for main config
            .unwrap_or_default()
    })
}

pub fn update_config(new_config: CanisterConfig) -> Result<(), String> {
    CONFIG_STORAGE.with(|config| {
        config.borrow_mut().insert(0, new_config);
        Ok(())
    })
}

// Audit logging functions
pub fn log_action(action: &str, details: &str, success: bool) {
    let log_entry = AuditLog {
        timestamp: time(),
        caller: ic_cdk::api::caller(),
        action: action.to_string(),
        details: details.to_string(),
        success,
    };
    
    AUDIT_LOGS.with(|logs| {
        let next_id = AUDIT_LOG_COUNTER.with(|counter| {
            let current = *counter.borrow();
            *counter.borrow_mut() = current + 1;
            current
        });
        
        logs.borrow_mut().insert(next_id, log_entry);
    });
}

pub fn log_nft_activity(activity: &str, token_id: u64, caller: Principal) {
    let details = format!("Token ID: {}, Caller: {}", token_id, caller.to_text());
    log_action(activity, &details, true);
}

// Additional helper functions for better storage management

/// Get all NFTs for a specific owner
pub fn get_nfts_by_owner(owner: &Principal) -> Vec<RWANFTData> {
    RWA_NFTS.with(|nfts| {
        nfts.borrow()
            .iter()
            .filter(|(_, nft_data)| nft_data.owner == *owner)
            .map(|(_, nft_data)| nft_data.clone())
            .collect()
    })
}

/// Get collateral record by NFT token ID
pub fn get_collateral_by_nft_token_id(token_id: u64) -> Option<CollateralRecord> {
    COLLATERAL_RECORDS.with(|records| {
        records.borrow()
            .iter()
            .find(|(_, record)| record.nft_token_id == token_id)
            .map(|(_, record)| record.clone())
    })
}

/// Get all audit logs for debugging (admin only in production)
pub fn get_audit_logs(limit: Option<u64>) -> Vec<AuditLog> {
    AUDIT_LOGS.with(|logs| {
        let logs_map = logs.borrow();
        let mut result: Vec<AuditLog> = logs_map.iter()
            .map(|(_, log)| log.clone())
            .collect();
        
        // Sort by timestamp (newest first)
        result.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        
        // Apply limit if specified
        if let Some(limit) = limit {
            result.truncate(limit as usize);
        }
        
        result
    })
}

/// Clear old audit logs (keep only recent ones)
pub fn cleanup_audit_logs(keep_recent: u64) {
    AUDIT_LOGS.with(|logs| {
        let mut logs_map = logs.borrow_mut();
        
        // Get all logs sorted by timestamp
        let mut all_logs: Vec<(u64, AuditLog)> = logs_map.iter().collect();
        all_logs.sort_by(|a, b| b.1.timestamp.cmp(&a.1.timestamp));
        
        // Remove old logs (keep only recent ones)
        if all_logs.len() > keep_recent as usize {
            for (log_id, _) in all_logs.iter().skip(keep_recent as usize) {
                logs_map.remove(log_id);
            }
        }
    });
}

// Loan management functions
pub fn get_next_loan_id() -> u64 {
    LOAN_COUNTER.with(|counter| {
        let current = *counter.borrow();
        *counter.borrow_mut() = current + 1;
        current + 1
    })
}

pub fn store_loan(loan: Loan) -> Result<(), String> {
    LOANS.with(|loans| {
        loans.borrow_mut().insert(loan.id, loan);
        Ok(())
    })
}

pub fn update_loan(loan: Loan) -> Result<(), String> {
    LOANS.with(|loans| {
        loans.borrow_mut().insert(loan.id, loan);
        Ok(())
    })
}

pub fn get_loan(loan_id: u64) -> Option<Loan> {
    LOANS.with(|loans| loans.borrow().get(&loan_id))
}

pub fn get_loans_by_borrower(borrower: Principal) -> Vec<Loan> {
    LOANS.with(|loans| {
        loans.borrow()
            .iter()
            .filter(|(_, loan)| loan.borrower == borrower)
            .map(|(_, loan)| loan.clone())
            .collect()
    })
}

pub fn get_all_loans_data() -> Vec<Loan> {
    LOANS.with(|loans| {
        loans.borrow()
            .iter()
            .map(|(_, loan)| loan.clone())
            .collect()
    })
}

pub fn get_protocol_parameters() -> ProtocolParameters {
    PROTOCOL_PARAMS.with(|params| {
        params.borrow()
            .get(&0)
            .unwrap_or_else(|| ProtocolParameters::default())
    })
}

pub fn set_protocol_parameters(params: ProtocolParameters) -> Result<(), String> {
    PROTOCOL_PARAMS.with(|storage| {
        storage.borrow_mut().insert(0, params);
        Ok(())
    })
}

pub fn get_nft_data(token_id: u64) -> Option<RWANFTData> {
    get_nft_by_token_id(token_id)
}

pub fn lock_nft_for_loan(token_id: u64, loan_id: u64) -> Result<(), String> {
    RWA_NFTS.with(|nfts| {
        let mut nfts_map = nfts.borrow_mut();
        if let Some(mut nft_data) = nfts_map.get(&token_id) {
            if nft_data.is_locked {
                return Err("NFT is already locked".to_string());
            }
            
            nft_data.is_locked = true;
            nft_data.loan_id = Some(loan_id);
            nft_data.updated_at = time();
            
            nfts_map.insert(token_id, nft_data);
            
            // Update collateral record status
            update_collateral_status(token_id, CollateralStatus::Locked, Some(loan_id));
            
            Ok(())
        } else {
            Err("NFT not found".to_string())
        }
    })
}

pub fn unlock_nft(token_id: u64) -> Result<(), String> {
    RWA_NFTS.with(|nfts| {
        let mut nfts_map = nfts.borrow_mut();
        if let Some(mut nft_data) = nfts_map.get(&token_id) {
            nft_data.is_locked = false;
            nft_data.loan_id = None;
            nft_data.updated_at = time();
            
            nfts_map.insert(token_id, nft_data);
            
            // Update collateral record status
            update_collateral_status(token_id, CollateralStatus::Available, None);
            
            Ok(())
        } else {
            Err("NFT not found".to_string())
        }
    })
}

pub fn liquidate_collateral(token_id: u64, loan_id: u64) -> Result<(), String> {
    RWA_NFTS.with(|nfts| {
        let mut nfts_map = nfts.borrow_mut();
        if let Some(mut nft_data) = nfts_map.get(&token_id) {
            // Transfer ownership to system (represented by IC's management canister)
            nft_data.owner = Principal::management_canister();
            nft_data.is_locked = true;
            nft_data.loan_id = Some(loan_id);
            nft_data.updated_at = time();
            
            nfts_map.insert(token_id, nft_data);
            
            // Update collateral record status
            update_collateral_status(token_id, CollateralStatus::Liquidated, Some(loan_id));
            
            Ok(())
        } else {
            Err("NFT not found".to_string())
        }
    })
}

// Storage functions for production features
pub fn store_disbursement_record(record: DisbursementRecord) -> Result<(), String> {
    DISBURSEMENT_RECORDS.with(|records| {
        records.borrow_mut().insert(record.loan_id, record);
        Ok(())
    })
}

pub fn get_disbursement_record(loan_id: u64) -> Option<DisbursementRecord> {
    DISBURSEMENT_RECORDS.with(|records| records.borrow().get(&loan_id))
}

pub fn get_all_disbursement_records() -> Vec<DisbursementRecord> {
    DISBURSEMENT_RECORDS.with(|records| {
        records.borrow().iter().map(|(_, record)| record).collect()
    })
}

pub fn store_repayment_record(record: RepaymentRecord) -> Result<(), String> {
    REPAYMENTS.with(|repayments| {
        let mut repayments_map = repayments.borrow_mut();
        let key = format!("{}_{}", record.loan_id, record.timestamp);
        repayments_map.insert(record.loan_id, record);
        Ok(())
    })
}

pub fn get_repayment_record(loan_id: u64) -> Option<RepaymentRecord> {
    REPAYMENTS.with(|repayments| repayments.borrow().get(&loan_id))
}

pub fn get_all_repayment_records() -> Vec<RepaymentRecord> {
    REPAYMENTS.with(|repayments| {
        repayments.borrow().iter().map(|(_, record)| record).collect()
    })
}

pub fn get_repayment_records_by_loan(loan_id: u64) -> Vec<RepaymentRecord> {
    // Since we're using a simple storage, this will return the single record for the loan
    // In a production system, you'd want to store multiple records per loan
    get_all_repayment_records()
        .into_iter()
        .filter(|record| record.loan_id == loan_id)
        .collect()
}

pub fn update_loan_status(loan_id: u64, status: LoanStatus) -> Result<(), String> {
    LOANS.with(|loans| {
        let mut loans_map = loans.borrow_mut();
        if let Some(mut loan) = loans_map.get(&loan_id) {
            loan.status = status;
            loans_map.insert(loan_id, loan);
            Ok(())
        } else {
            Err("Loan not found".to_string())
        }
    })
}

pub fn update_loan_repaid_amount(loan_id: u64, amount: u64) -> Result<(), String> {
    LOANS.with(|loans| {
        let mut loans_map = loans.borrow_mut();
        if let Some(mut loan) = loans_map.get(&loan_id) {
            loan.total_repaid += amount;
            loans_map.insert(loan_id, loan);
            Ok(())
        } else {
            Err("Loan not found".to_string())
        }
    })
}

pub fn calculate_remaining_balance(loan_id: u64) -> Result<u64, String> {
    LOANS.with(|loans| {
        if let Some(loan) = loans.borrow().get(&loan_id) {
            Ok(loan.amount_approved.saturating_sub(loan.total_repaid))
        } else {
            Err("Loan not found".to_string())
        }
    })
}

pub fn release_collateral_nft(nft_id: u64) -> Result<(), String> {
    update_collateral_status(nft_id, CollateralStatus::Released, None);
    Ok(())
}

// Statistics functions
pub fn get_total_investors() -> u64 {
    INVESTOR_BALANCES.with(|balances| {
        balances.borrow().len() as u64
    })
}

pub fn get_total_deposits() -> u64 {
    INVESTOR_BALANCES.with(|balances| {
        balances.borrow().iter()
            .map(|(_, balance)| balance.total_deposited)
            .sum()
    })
}

pub fn get_total_withdrawals() -> u64 {
    INVESTOR_BALANCES.with(|balances| {
        balances.borrow().iter()
            .map(|(_, balance)| balance.total_withdrawn)
            .sum()
    })
}

// Enhanced Processed Transaction Functions

pub fn get_all_processed_transactions() -> Vec<ProcessedTransaction> {
    PROCESSED_TRANSACTIONS.with(|transactions| {
        transactions.borrow().iter().map(|(_, tx)| tx).collect()
    })
}

pub fn get_processed_transactions_by_investor(investor: Principal) -> Vec<ProcessedTransaction> {
    PROCESSED_TRANSACTIONS.with(|transactions| {
        transactions.borrow()
            .iter()
            .filter(|(_, tx)| tx.processor == investor)
            .map(|(_, tx)| tx)
            .collect()
    })
}

pub fn count_processed_transactions() -> u64 {
    PROCESSED_TRANSACTIONS.with(|transactions| {
        transactions.borrow().len() as u64
    })
}

// Enhanced configuration functions

// Enhanced pool management functions

pub fn get_pool_utilization_history() -> Vec<(u64, u64)> {
    // This would typically store historical utilization data
    // For now, return current utilization
    let pool = get_liquidity_pool();
    let utilization = if pool.total_liquidity > 0 {
        ((pool.total_liquidity - pool.available_liquidity) * 100) / pool.total_liquidity
    } else {
        0
    };
    vec![(time(), utilization)]
}

pub fn get_investor_count() -> u64 {
    INVESTOR_BALANCES.with(|balances| {
        balances.borrow().len() as u64
    })
}

pub fn get_active_investor_count() -> u64 {
    INVESTOR_BALANCES.with(|balances| {
        balances.borrow()
            .iter()
            .filter(|(_, balance)| balance.balance > 0)
            .count() as u64
    })
}

pub fn get_total_investor_deposits() -> u64 {
    INVESTOR_BALANCES.with(|balances| {
        balances.borrow()
            .iter()
            .map(|(_, balance)| balance.total_deposited)
            .sum()
    })
}

pub fn get_total_investor_withdrawals() -> u64 {
    INVESTOR_BALANCES.with(|balances| {
        balances.borrow()
            .iter()
            .map(|(_, balance)| balance.total_withdrawn)
            .sum()
    })
}

// Pool analytics functions

pub fn get_largest_investor_deposit() -> u64 {
    INVESTOR_BALANCES.with(|balances| {
        balances.borrow()
            .iter()
            .map(|(_, balance)| balance.total_deposited)
            .max()
            .unwrap_or(0)
    })
}

pub fn get_average_investor_deposit() -> u64 {
    let total_investors = get_investor_count();
    if total_investors > 0 {
        get_total_investor_deposits() / total_investors
    } else {
        0
    }
}

pub fn get_pool_concentration_risk() -> u64 {
    let pool = get_liquidity_pool();
    let largest_deposit = get_largest_investor_deposit();
    
    if pool.total_liquidity > 0 {
        (largest_deposit * 100) / pool.total_liquidity
    } else {
        0
    }
}

// Cleanup and maintenance functions

pub fn cleanup_old_processed_transactions(cutoff_time: u64) -> u64 {
    let mut cleaned_count = 0;
    
    PROCESSED_TRANSACTIONS.with(|transactions| {
        let keys_to_remove: Vec<u64> = transactions.borrow()
            .iter()
            .filter(|(_, tx)| tx.processed_at < cutoff_time)
            .map(|(tx_id, _)| tx_id)
            .collect();
        
        cleaned_count = keys_to_remove.len() as u64;
        
        for key in keys_to_remove {
            transactions.borrow_mut().remove(&key);
        }
    });
    
    cleaned_count
}

pub fn get_storage_statistics() -> StorageStats {
    let total_nfts = RWA_NFTS.with(|nfts| nfts.borrow().len() as u64);
    let total_loans = LOANS.with(|loans| loans.borrow().len() as u64);
    let total_users = INVESTOR_BALANCES.with(|balances| balances.borrow().len() as u64);
    
    // Get liquidity data
    let pool = get_liquidity_pool();
    let total_collateral = COLLATERAL_RECORDS.with(|records| {
        records.borrow().iter()
            .map(|(_, record)| record.valuation_idr)
            .sum::<u64>()
    });
    
    // Estimate memory usage (simplified)
    let estimated_memory = (total_nfts * 1024) + (total_loans * 2048) + (total_users * 512);
    
    StorageStats {
        total_nfts,
        total_loans,
        total_users,
        memory_usage_bytes: estimated_memory,
        total_collateral,
        total_liquidity: pool.total_liquidity,
    }
}

pub fn get_storage_stats() -> StorageStats {
    get_storage_statistics()
}

// Oracle and Price Management Storage Functions
pub fn store_commodity_price(commodity_id: String, price: CommodityPriceData) -> Result<(), String> {
    ORACLE_PRICES.with(|prices| {
        // Convert CommodityPriceData to CommodityPrice for compatibility
        let legacy_price = CommodityPrice {
            price_per_unit: price.price_per_unit,
            currency: price.currency.clone(),
            timestamp: price.timestamp,
        };
        prices.borrow_mut().insert(commodity_id, legacy_price);
    });
    Ok(())
}

pub fn get_stored_commodity_price(commodity_id: &str) -> Option<CommodityPriceData> {
    ORACLE_PRICES.with(|prices| {
        prices.borrow().get(&commodity_id.to_string()).map(|legacy_price| {
            // Convert CommodityPrice to CommodityPriceData
            CommodityPriceData {
                commodity_type: commodity_id.to_string(),
                price_per_unit: legacy_price.price_per_unit,
                currency: legacy_price.currency.clone(),
                timestamp: legacy_price.timestamp,
                source: "legacy_storage".to_string(),
                confidence_score: 80, // Default confidence
                is_stale: false, // Will be calculated by caller
                fetch_attempt_count: 1,
                last_successful_fetch: legacy_price.timestamp,
            }
        })
    })
}

pub fn get_all_stored_commodity_prices() -> Vec<(String, CommodityPriceData)> {
    ORACLE_PRICES.with(|prices| {
        prices.borrow().iter()
            .map(|(key, legacy_price)| {
                let price_data = CommodityPriceData {
                    commodity_type: key.clone(),
                    price_per_unit: legacy_price.price_per_unit,
                    currency: legacy_price.currency.clone(),
                    timestamp: legacy_price.timestamp,
                    source: "legacy_storage".to_string(),
                    confidence_score: 80,
                    is_stale: false,
                    fetch_attempt_count: 1,
                    last_successful_fetch: legacy_price.timestamp,
                };
                (key.clone(), price_data)
            })
            .collect()
    })
}

pub fn update_last_price_fetch(commodity_id: &str, timestamp: u64) {
    PRICE_FETCH_TRACKER.with(|tracker| {
        let mut fetch_record = tracker.borrow().get(&commodity_id.to_string())
            .unwrap_or(PriceFetchRecord {
                commodity_id: commodity_id.to_string(),
                last_fetch_timestamp: 0,
                fetch_count: 0,
                success_count: 0,
                failure_count: 0,
                last_error: None,
                average_response_time: 0,
                rate_limit_reset: 0,
            });
        
        fetch_record.last_fetch_timestamp = timestamp;
        fetch_record.fetch_count += 1;
        fetch_record.success_count += 1;
        
        tracker.borrow_mut().insert(commodity_id.to_string(), fetch_record);
    });
}

pub fn get_last_price_fetch(commodity_id: &str) -> Option<u64> {
    PRICE_FETCH_TRACKER.with(|tracker| {
        tracker.borrow().get(&commodity_id.to_string())
            .map(|record| record.last_fetch_timestamp)
    })
}

pub fn update_price_fetch_failure(commodity_id: &str, error: String) {
    PRICE_FETCH_TRACKER.with(|tracker| {
        let mut fetch_record = tracker.borrow().get(&commodity_id.to_string())
            .unwrap_or(PriceFetchRecord {
                commodity_id: commodity_id.to_string(),
                last_fetch_timestamp: 0,
                fetch_count: 0,
                success_count: 0,
                failure_count: 0,
                last_error: None,
                average_response_time: 0,
                rate_limit_reset: 0,
            });
        
        fetch_record.fetch_count += 1;
        fetch_record.failure_count += 1;
        fetch_record.last_error = Some(error);
        fetch_record.last_fetch_timestamp = time();
        
        tracker.borrow_mut().insert(commodity_id.to_string(), fetch_record);
    });
}

pub fn get_price_fetch_statistics(commodity_id: &str) -> Option<PriceFetchRecord> {
    PRICE_FETCH_TRACKER.with(|tracker| {
        tracker.borrow().get(&commodity_id.to_string())
    })
}

// Liquidity Management Storage Functions

pub fn get_liquidity_pool() -> LiquidityPool {
    LIQUIDITY_POOL.with(|pool| {
        pool.borrow().get(&0).unwrap_or(LiquidityPool {
            total_liquidity: 0,
            available_liquidity: 0,
            total_borrowed: 0,
            total_repaid: 0,
            utilization_rate: 0,
            total_investors: 0,
            apy: 0,
            created_at: time(),
            updated_at: time(),
        })
    })
}

pub fn store_liquidity_pool(pool: LiquidityPool) -> Result<(), String> {
    LIQUIDITY_POOL.with(|p| {
        p.borrow_mut().insert(0, pool);
    });
    Ok(())
}

pub fn get_investor_balance_by_principal(investor: Principal) -> Option<InvestorBalance> {
    INVESTOR_BALANCES.with(|balances| {
        balances.borrow().get(&investor)
    })
}

pub fn store_investor_balance(balance: InvestorBalance) -> Result<(), String> {
    INVESTOR_BALANCES.with(|balances| {
        balances.borrow_mut().insert(balance.investor, balance);
    });
    Ok(())
}

pub fn get_all_investor_balances() -> Vec<InvestorBalance> {
    INVESTOR_BALANCES.with(|balances| {
        balances.borrow().iter().map(|(_, balance)| balance).collect()
    })
}

pub fn is_transaction_processed(tx_id: u64) -> bool {
    PROCESSED_TRANSACTIONS.with(|transactions| {
        transactions.borrow().contains_key(&tx_id)
    })
}

pub fn mark_transaction_processed(tx_id: u64) -> Result<(), String> {
    PROCESSED_TRANSACTIONS.with(|transactions| {
        let processed_tx = ProcessedTransaction {
            tx_id,
            processed_at: time(),
            processor: caller(),
        };
        transactions.borrow_mut().insert(tx_id, processed_tx);
    });
    Ok(())
}

pub fn has_investor_deposited_before(investor: Principal) -> bool {
    INVESTOR_BALANCES.with(|balances| {
        balances.borrow().contains_key(&investor)
    })
}

pub fn set_emergency_pause(paused: bool) -> Result<(), String> {
    EMERGENCY_PAUSE.with(|pause| {
        pause.borrow_mut().insert(0, paused);
    });
    Ok(())
}

pub fn is_emergency_paused() -> bool {
    EMERGENCY_PAUSE.with(|pause| {
        pause.borrow().get(&0).unwrap_or(false)
    })
}

pub fn get_processed_transaction(tx_id: u64) -> Option<ProcessedTransaction> {
    PROCESSED_TRANSACTIONS.with(|transactions| {
        transactions.borrow().get(&tx_id)
    })
}

pub fn remove_processed_transaction(tx_id: u64) -> Option<ProcessedTransaction> {
    PROCESSED_TRANSACTIONS.with(|transactions| {
        transactions.borrow_mut().remove(&tx_id)
    })
}

/// Get all users (for admin or debugging purposes)
pub fn get_all_users() -> Vec<User> {
    USERS.with(|users| {
        users.borrow().iter()
            .map(|(_, user)| user.clone())  // Use .clone() to get owned values
            .collect()
    })
}

