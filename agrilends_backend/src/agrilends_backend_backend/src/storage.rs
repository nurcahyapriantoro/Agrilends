use crate::types::*;
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{DefaultMemoryImpl, StableBTreeMap};
use std::cell::RefCell;
use ic_cdk::api::time;
use candid::Principal;

// Memory types
type Memory = VirtualMemory<DefaultMemoryImpl>;
type NFTStorage = StableBTreeMap<u64, RWANFTData, Memory>;
type CollateralStorage = StableBTreeMap<u64, CollateralRecord, Memory>;
type AuditLogStorage = StableBTreeMap<u64, AuditLog, Memory>;
type ConfigStorage = StableBTreeMap<u8, CanisterConfig, Memory>;
type LoanStorage = StableBTreeMap<u64, Loan, Memory>;
type ProtocolParamsStorage = StableBTreeMap<u8, ProtocolParameters, Memory>;

// Memory Manager
thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(
        MemoryManager::init(DefaultMemoryImpl::default())
    );
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

// Token ID counters
thread_local! {
    static NFT_TOKEN_COUNTER: RefCell<u64> = RefCell::new(0);
    static COLLATERAL_COUNTER: RefCell<u64> = RefCell::new(0);
    pub static AUDIT_LOG_COUNTER: RefCell<u64> = RefCell::new(0);
    static LOAN_COUNTER: RefCell<u64> = RefCell::new(0);
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

pub fn get_loan(loan_id: u64) -> Option<Loan> {
    LOANS.with(|loans| {
        loans.borrow().get(&loan_id)
    })
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
