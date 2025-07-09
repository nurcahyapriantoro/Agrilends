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

// Token ID counters
thread_local! {
    static NFT_TOKEN_COUNTER: RefCell<u64> = RefCell::new(0);
    static COLLATERAL_COUNTER: RefCell<u64> = RefCell::new(0);
    static AUDIT_LOG_COUNTER: RefCell<u64> = RefCell::new(0);
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

// Helper function to get NFT by token ID
pub fn get_nft_by_token_id(token_id: u64) -> Option<RWANFTData> {
    RWA_NFTS.with(|nfts| nfts.borrow().get(&token_id))
}

// Helper function to get collateral by ID
pub fn get_collateral_by_id(collateral_id: u64) -> Option<CollateralRecord> {
    COLLATERAL_RECORDS.with(|records| records.borrow().get(&collateral_id))
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
        let total_logs = logs_map.len() as u64;
        
        if total_logs > keep_recent {
            // Get all log IDs and sort them
            let mut log_ids: Vec<u64> = logs_map.iter().map(|(id, _)| id).collect();
            log_ids.sort();
            
            // Remove oldest logs
            let to_remove = total_logs - keep_recent;
            for i in 0..to_remove {
                if let Some(id) = log_ids.get(i as usize) {
                    logs_map.remove(id);
                }
            }
        }
    });
}

/// Get storage statistics
pub fn get_storage_stats() -> StorageStats {
    let nft_count = RWA_NFTS.with(|nfts| nfts.borrow().len());
    let collateral_count = COLLATERAL_RECORDS.with(|records| records.borrow().len());
    let audit_log_count = AUDIT_LOGS.with(|logs| logs.borrow().len());
    
    StorageStats {
        total_nfts: nft_count as u64,
        total_collateral_records: collateral_count as u64,
        total_audit_logs: audit_log_count as u64,
        nft_token_counter: NFT_TOKEN_COUNTER.with(|c| *c.borrow()),
        collateral_counter: COLLATERAL_COUNTER.with(|c| *c.borrow()),
        audit_log_counter: AUDIT_LOG_COUNTER.with(|c| *c.borrow()),
    }
}
