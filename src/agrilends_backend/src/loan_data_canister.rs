// ========== LOAN DATA CANISTER MODULE ==========
// Lightweight data storage canister for sharded loan data
// This canister is created dynamically by the factory pattern
// Handles CRUD operations for loan data within a single shard

use ic_cdk::{caller, api::time};
use ic_cdk_macros::{query, update, init, pre_upgrade, post_upgrade};
use candid::{CandidType, Deserialize, Principal};
use ic_stable_structures::{StableBTreeMap, memory::MemoryId};
use ic_stable_structures::memory::VirtualMemory;
use ic_stable_structures::DefaultMemoryImpl;
use std::cell::RefCell;
use std::collections::HashMap;

use crate::types::*;
use crate::storage::get_memory_by_id;

// ========== DATA CANISTER TYPES ==========

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct DataCanisterInfo {
    pub canister_id: Principal,
    pub shard_id: u32,
    pub created_at: u64,
    pub max_loans: u64,
    pub current_loan_count: u64,
    pub authorized_callers: Vec<Principal>,
    pub is_read_only: bool,
    pub last_backup_time: u64,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct ShardStatistics {
    pub total_loans: u64,
    pub active_loans: u64,
    pub completed_loans: u64,
    pub defaulted_loans: u64,
    pub total_volume_satoshi: u64,
    pub avg_loan_amount: u64,
    pub storage_used_bytes: u64,
    pub performance_metrics: ShardPerformanceMetrics,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct ShardPerformanceMetrics {
    pub avg_query_time_ms: u64,
    pub total_queries: u64,
    pub total_updates: u64,
    pub error_count: u64,
    pub last_activity: u64,
}

// ========== STORAGE MANAGEMENT ==========

thread_local! {
    static SHARD_LOANS: RefCell<StableBTreeMap<u64, Loan, VirtualMemory<DefaultMemoryImpl>>> = 
        RefCell::new(StableBTreeMap::init(get_memory_by_id(MemoryId::new(30))));
    
    static USER_LOAN_INDEX: RefCell<StableBTreeMap<Principal, Vec<u64>, VirtualMemory<DefaultMemoryImpl>>> = 
        RefCell::new(StableBTreeMap::init(get_memory_by_id(MemoryId::new(31))));
    
    static SHARD_INFO: RefCell<DataCanisterInfo> = RefCell::new(DataCanisterInfo {
        canister_id: Principal::anonymous(),
        shard_id: 0,
        created_at: 0,
        max_loans: 100_000,
        current_loan_count: 0,
        authorized_callers: vec![],
        is_read_only: false,
        last_backup_time: 0,
    });
    
    static PERFORMANCE_METRICS: RefCell<ShardPerformanceMetrics> = RefCell::new(ShardPerformanceMetrics {
        avg_query_time_ms: 0,
        total_queries: 0,
        total_updates: 0,
        error_count: 0,
        last_activity: 0,
    });
}

// ========== INITIALIZATION ==========

#[init]
fn init(shard_id: u32, max_loans: u64, authorized_callers: Vec<Principal>) {
    let current_time = time();
    let canister_id = ic_cdk::api::id();
    
    SHARD_INFO.with(|info| {
        *info.borrow_mut() = DataCanisterInfo {
            canister_id,
            shard_id,
            created_at: current_time,
            max_loans,
            current_loan_count: 0,
            authorized_callers,
            is_read_only: false,
            last_backup_time: current_time,
        };
    });
}

// ========== ACCESS CONTROL ==========

fn is_authorized_caller() -> Result<(), String> {
    let caller = caller();
    
    SHARD_INFO.with(|info| {
        let info_ref = info.borrow();
        if info_ref.authorized_callers.contains(&caller) {
            Ok(())
        } else {
            Err("Unauthorized caller".to_string())
        }
    })
}

fn check_read_only() -> Result<(), String> {
    SHARD_INFO.with(|info| {
        if info.borrow().is_read_only {
            Err("Shard is in read-only mode".to_string())
        } else {
            Ok(())
        }
    })
}

fn update_performance_metrics(operation_type: &str, duration_ms: u64) {
    PERFORMANCE_METRICS.with(|metrics| {
        let mut metrics_ref = metrics.borrow_mut();
        metrics_ref.last_activity = time();
        
        match operation_type {
            "query" => {
                metrics_ref.total_queries += 1;
                // Update average query time
                let total_time = metrics_ref.avg_query_time_ms * (metrics_ref.total_queries - 1) + duration_ms;
                metrics_ref.avg_query_time_ms = total_time / metrics_ref.total_queries;
            },
            "update" => {
                metrics_ref.total_updates += 1;
            },
            "error" => {
                metrics_ref.error_count += 1;
            },
            _ => {}
        }
    });
}

// ========== LOAN CRUD OPERATIONS ==========

/// Store a new loan in this shard
#[update]
pub fn store_loan(loan: Loan) -> Result<(), String> {
    let start_time = time();
    
    // Access control
    is_authorized_caller()?;
    check_read_only()?;
    
    // Check capacity
    let current_count = SHARD_INFO.with(|info| info.borrow().current_loan_count);
    let max_loans = SHARD_INFO.with(|info| info.borrow().max_loans);
    
    if current_count >= max_loans {
        update_performance_metrics("error", 0);
        return Err("Shard at maximum capacity".to_string());
    }
    
    let loan_id = loan.id;
    let borrower = loan.borrower;
    
    // Store loan
    SHARD_LOANS.with(|loans| {
        loans.borrow_mut().insert(loan_id, loan);
    });
    
    // Update user index
    USER_LOAN_INDEX.with(|index| {
        let mut index_ref = index.borrow_mut();
        let mut user_loans = index_ref.get(&borrower).unwrap_or_default();
        user_loans.push(loan_id);
        index_ref.insert(borrower, user_loans);
    });
    
    // Update shard info
    SHARD_INFO.with(|info| {
        let mut info_ref = info.borrow_mut();
        info_ref.current_loan_count += 1;
    });
    
    let duration = time() - start_time;
    update_performance_metrics("update", duration);
    
    Ok(())
}

/// Get a loan by ID
#[query]
pub fn get_loan(loan_id: u64) -> Result<Loan, String> {
    let start_time = time();
    
    // Access control
    is_authorized_caller()?;
    
    let result = SHARD_LOANS.with(|loans| {
        loans.borrow()
            .get(&loan_id)
            .ok_or_else(|| "Loan not found in this shard".to_string())
    });
    
    let duration = time() - start_time;
    match &result {
        Ok(_) => update_performance_metrics("query", duration),
        Err(_) => update_performance_metrics("error", duration),
    }
    
    result
}

/// Update an existing loan
#[update]
pub fn update_loan(loan_id: u64, updated_loan: Loan) -> Result<(), String> {
    let start_time = time();
    
    // Access control
    is_authorized_caller()?;
    check_read_only()?;
    
    // Verify loan exists
    let exists = SHARD_LOANS.with(|loans| {
        loans.borrow().contains_key(&loan_id)
    });
    
    if !exists {
        update_performance_metrics("error", 0);
        return Err("Loan not found in this shard".to_string());
    }
    
    // Update loan
    SHARD_LOANS.with(|loans| {
        loans.borrow_mut().insert(loan_id, updated_loan);
    });
    
    let duration = time() - start_time;
    update_performance_metrics("update", duration);
    
    Ok(())
}

/// Get all loans for a specific user
#[query]
pub fn get_user_loans(user_id: Principal) -> Result<Vec<Loan>, String> {
    let start_time = time();
    
    // Access control
    is_authorized_caller()?;
    
    let result = USER_LOAN_INDEX.with(|index| {
        let loan_ids = index.borrow().get(&user_id).unwrap_or_default();
        
        SHARD_LOANS.with(|loans| {
            let loans_ref = loans.borrow();
            let mut user_loans = Vec::new();
            
            for loan_id in loan_ids {
                if let Some(loan) = loans_ref.get(&loan_id) {
                    user_loans.push(loan);
                }
            }
            
            user_loans
        })
    });
    
    let duration = time() - start_time;
    update_performance_metrics("query", duration);
    
    Ok(result)
}

/// Get loans by status
#[query]
pub fn get_loans_by_status(status: LoanStatus) -> Result<Vec<Loan>, String> {
    let start_time = time();
    
    // Access control
    is_authorized_caller()?;
    
    let result = SHARD_LOANS.with(|loans| {
        loans.borrow()
            .iter()
            .filter(|(_, loan)| loan.status == status)
            .map(|(_, loan)| loan.clone())
            .collect()
    });
    
    let duration = time() - start_time;
    update_performance_metrics("query", duration);
    
    Ok(result)
}

/// Get paginated loans
#[query]
pub fn get_loans_paginated(
    offset: u64,
    limit: u64,
    filter_status: Option<LoanStatus>
) -> Result<PaginatedLoans, String> {
    let start_time = time();
    
    // Access control
    is_authorized_caller()?;
    
    if limit > 100 {
        update_performance_metrics("error", 0);
        return Err("Limit cannot exceed 100".to_string());
    }
    
    let result = SHARD_LOANS.with(|loans| {
        let loans_ref = loans.borrow();
        let mut filtered_loans: Vec<_> = loans_ref
            .iter()
            .filter(|(_, loan)| {
                filter_status.map_or(true, |status| loan.status == status)
            })
            .collect();
        
        // Sort by loan ID for consistent pagination
        filtered_loans.sort_by_key(|(id, _)| *id);
        
        let total_count = filtered_loans.len() as u64;
        let loans: Vec<Loan> = filtered_loans
            .into_iter()
            .skip(offset as usize)
            .take(limit as usize)
            .map(|(_, loan)| loan.clone())
            .collect();
        
        PaginatedLoans {
            loans,
            total_count,
            offset,
            limit,
            has_more: offset + loans.len() as u64 < total_count,
        }
    });
    
    let duration = time() - start_time;
    update_performance_metrics("query", duration);
    
    Ok(result)
}

// ========== SHARD MANAGEMENT ==========

/// Get shard information and statistics
#[query]
pub fn get_shard_info() -> Result<(DataCanisterInfo, ShardStatistics), String> {
    let start_time = time();
    
    // Access control
    is_authorized_caller()?;
    
    let shard_info = SHARD_INFO.with(|info| info.borrow().clone());
    
    let statistics = SHARD_LOANS.with(|loans| {
        let loans_ref = loans.borrow();
        let total_loans = loans_ref.len() as u64;
        
        let mut active_loans = 0;
        let mut completed_loans = 0;
        let mut defaulted_loans = 0;
        let mut total_volume_satoshi = 0;
        
        for (_, loan) in loans_ref.iter() {
            total_volume_satoshi += loan.amount_requested;
            
            match loan.status {
                LoanStatus::Active => active_loans += 1,
                LoanStatus::Completed => completed_loans += 1,
                LoanStatus::DefaultedLiquidated => defaulted_loans += 1,
                _ => {}
            }
        }
        
        let avg_loan_amount = if total_loans > 0 {
            total_volume_satoshi / total_loans
        } else { 0 };
        
        ShardStatistics {
            total_loans,
            active_loans,
            completed_loans,
            defaulted_loans,
            total_volume_satoshi,
            avg_loan_amount,
            storage_used_bytes: 0, // Would be calculated from canister status
            performance_metrics: PERFORMANCE_METRICS.with(|m| m.borrow().clone()),
        }
    });
    
    let duration = time() - start_time;
    update_performance_metrics("query", duration);
    
    Ok((shard_info, statistics))
}

/// Set shard to read-only mode
#[update]
pub fn set_read_only(read_only: bool) -> Result<(), String> {
    // Access control
    is_authorized_caller()?;
    
    SHARD_INFO.with(|info| {
        let mut info_ref = info.borrow_mut();
        info_ref.is_read_only = read_only;
    });
    
    Ok(())
}

/// Add authorized caller
#[update]
pub fn add_authorized_caller(caller_principal: Principal) -> Result<(), String> {
    // Access control - only existing authorized callers can add new ones
    is_authorized_caller()?;
    
    SHARD_INFO.with(|info| {
        let mut info_ref = info.borrow_mut();
        if !info_ref.authorized_callers.contains(&caller_principal) {
            info_ref.authorized_callers.push(caller_principal);
        }
    });
    
    Ok(())
}

/// Remove authorized caller
#[update]
pub fn remove_authorized_caller(caller_principal: Principal) -> Result<(), String> {
    // Access control
    is_authorized_caller()?;
    
    SHARD_INFO.with(|info| {
        let mut info_ref = info.borrow_mut();
        info_ref.authorized_callers.retain(|&p| p != caller_principal);
    });
    
    Ok(())
}

// ========== DATA MIGRATION SUPPORT ==========

/// Export loans for migration (authorized callers only)
#[query]
pub fn export_loans(loan_ids: Vec<u64>) -> Result<Vec<Loan>, String> {
    let start_time = time();
    
    // Access control
    is_authorized_caller()?;
    
    if loan_ids.len() > 1000 {
        update_performance_metrics("error", 0);
        return Err("Cannot export more than 1000 loans at once".to_string());
    }
    
    let result = SHARD_LOANS.with(|loans| {
        let loans_ref = loans.borrow();
        let mut exported_loans = Vec::new();
        
        for loan_id in loan_ids {
            if let Some(loan) = loans_ref.get(&loan_id) {
                exported_loans.push(loan.clone());
            }
        }
        
        exported_loans
    });
    
    let duration = time() - start_time;
    update_performance_metrics("query", duration);
    
    Ok(result)
}

/// Import loans from migration (authorized callers only)
#[update]
pub fn import_loans(loans: Vec<Loan>) -> Result<u64, String> {
    let start_time = time();
    
    // Access control
    is_authorized_caller()?;
    check_read_only()?;
    
    if loans.len() > 1000 {
        update_performance_metrics("error", 0);
        return Err("Cannot import more than 1000 loans at once".to_string());
    }
    
    // Check capacity
    let current_count = SHARD_INFO.with(|info| info.borrow().current_loan_count);
    let max_loans = SHARD_INFO.with(|info| info.borrow().max_loans);
    
    if current_count + loans.len() as u64 > max_loans {
        update_performance_metrics("error", 0);
        return Err("Not enough capacity for import".to_string());
    }
    
    let mut imported_count = 0;
    
    for loan in loans {
        let loan_id = loan.id;
        let borrower = loan.borrower;
        
        // Store loan
        SHARD_LOANS.with(|shard_loans| {
            shard_loans.borrow_mut().insert(loan_id, loan);
        });
        
        // Update user index
        USER_LOAN_INDEX.with(|index| {
            let mut index_ref = index.borrow_mut();
            let mut user_loans = index_ref.get(&borrower).unwrap_or_default();
            if !user_loans.contains(&loan_id) {
                user_loans.push(loan_id);
                index_ref.insert(borrower, user_loans);
            }
        });
        
        imported_count += 1;
    }
    
    // Update shard info
    SHARD_INFO.with(|info| {
        let mut info_ref = info.borrow_mut();
        info_ref.current_loan_count += imported_count;
    });
    
    let duration = time() - start_time;
    update_performance_metrics("update", duration);
    
    Ok(imported_count)
}

/// Delete loans (for cleanup after migration)
#[update]
pub fn delete_loans(loan_ids: Vec<u64>) -> Result<u64, String> {
    let start_time = time();
    
    // Access control
    is_authorized_caller()?;
    check_read_only()?;
    
    if loan_ids.len() > 1000 {
        update_performance_metrics("error", 0);
        return Err("Cannot delete more than 1000 loans at once".to_string());
    }
    
    let mut deleted_count = 0;
    
    for loan_id in loan_ids {
        // Get loan info before deletion for user index cleanup
        let loan = SHARD_LOANS.with(|loans| {
            loans.borrow().get(&loan_id)
        });
        
        if let Some(loan) = loan {
            let borrower = loan.borrower;
            
            // Delete from main storage
            SHARD_LOANS.with(|loans| {
                loans.borrow_mut().remove(&loan_id);
            });
            
            // Update user index
            USER_LOAN_INDEX.with(|index| {
                let mut index_ref = index.borrow_mut();
                if let Some(mut user_loans) = index_ref.get(&borrower) {
                    user_loans.retain(|&id| id != loan_id);
                    if user_loans.is_empty() {
                        index_ref.remove(&borrower);
                    } else {
                        index_ref.insert(borrower, user_loans);
                    }
                }
            });
            
            deleted_count += 1;
        }
    }
    
    // Update shard info
    SHARD_INFO.with(|info| {
        let mut info_ref = info.borrow_mut();
        info_ref.current_loan_count = info_ref.current_loan_count.saturating_sub(deleted_count);
    });
    
    let duration = time() - start_time;
    update_performance_metrics("update", duration);
    
    Ok(deleted_count)
}

// ========== HEALTH CHECK & MONITORING ==========

/// Health check endpoint
#[query]
pub fn health_check() -> Result<ShardHealthStatus, String> {
    let shard_info = SHARD_INFO.with(|info| info.borrow().clone());
    let performance_metrics = PERFORMANCE_METRICS.with(|m| m.borrow().clone());
    
    let health_status = if performance_metrics.error_count > 100 {
        "unhealthy".to_string()
    } else if shard_info.current_loan_count > shard_info.max_loans * 90 / 100 {
        "warning".to_string()
    } else {
        "healthy".to_string()
    };
    
    Ok(ShardHealthStatus {
        status: health_status,
        current_load: shard_info.current_loan_count,
        max_capacity: shard_info.max_loans,
        utilization_percentage: (shard_info.current_loan_count as f64 / shard_info.max_loans as f64) * 100.0,
        performance_metrics,
        last_check: time(),
    })
}

// ========== UPGRADE HOOKS ==========

#[pre_upgrade]
fn pre_upgrade() {
    // Data is automatically preserved in stable memory
}

#[post_upgrade]
fn post_upgrade() {
    // Data is automatically restored from stable memory
}

// ========== ADDITIONAL TYPES ==========

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct PaginatedLoans {
    pub loans: Vec<Loan>,
    pub total_count: u64,
    pub offset: u64,
    pub limit: u64,
    pub has_more: bool,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct ShardHealthStatus {
    pub status: String,
    pub current_load: u64,
    pub max_capacity: u64,
    pub utilization_percentage: f64,
    pub performance_metrics: ShardPerformanceMetrics,
    pub last_check: u64,
}
