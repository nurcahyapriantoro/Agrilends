use ic_cdk_macros::{update, query};
use crate::types::CanisterConfig;
use crate::storage::{get_config, update_config, log_action};
use crate::helpers::is_admin;

/// Production-ready configuration update with validation
#[update]
pub fn update_canister_config(new_config: CanisterConfig) -> Result<(), String> {
    let caller = ic_cdk::api::caller();
    
    // Security check
    if !is_admin(&caller) {
        return Err("Unauthorized: Only admins can update configuration".to_string());
    }
    
    // Validate configuration
    if new_config.max_nft_per_user == 0 {
        return Err("Invalid config: max_nft_per_user must be greater than 0".to_string());
    }
    
    if new_config.min_collateral_value == 0 {
        return Err("Invalid config: min_collateral_value must be greater than 0".to_string());
    }
    
    if new_config.min_collateral_value >= new_config.max_collateral_value {
        return Err("Invalid config: min_collateral_value must be less than max_collateral_value".to_string());
    }
    
    // Update configuration
    update_config(new_config.clone())?;
    
    log_action("config_update", &format!("Configuration updated by admin: {}", caller.to_text()), true);
    Ok(())
}

/// Emergency stop function
#[update]
pub fn emergency_stop() -> Result<(), String> {
    let caller = ic_cdk::api::caller();
    
    if !is_admin(&caller) {
        return Err("Unauthorized: Only admins can trigger emergency stop".to_string());
    }
    
    let mut config = get_config();
    config.emergency_stop = true;
    update_config(config)?;
    
    log_action("emergency_stop", &format!("Emergency stop activated by: {}", caller.to_text()), true);
    Ok(())
}

/// Resume operations after emergency stop
#[update]
pub fn resume_operations() -> Result<(), String> {
    let caller = ic_cdk::api::caller();
    
    if !is_admin(&caller) {
        return Err("Unauthorized: Only admins can resume operations".to_string());
    }
    
    let mut config = get_config();
    config.emergency_stop = false;
    config.maintenance_mode = false;
    update_config(config)?;
    
    log_action("resume_operations", &format!("Operations resumed by: {}", caller.to_text()), true);
    Ok(())
}

/// Get system health status
#[query]
pub fn get_system_health() -> SystemHealth {
    let config = get_config();
    let stats = crate::storage::get_storage_stats();
    
    SystemHealth {
        emergency_stop: config.emergency_stop,
        maintenance_mode: config.maintenance_mode,
        total_nfts: stats.total_nfts,
        total_users: crate::user_management::get_user_stats().total_users,
        system_uptime: ic_cdk::api::time(),
    }
}

#[derive(candid::CandidType, candid::Deserialize, Clone, Debug)]
pub struct SystemHealth {
    pub emergency_stop: bool,
    pub maintenance_mode: bool,
    pub total_nfts: u64,
    pub total_users: u64,
    pub system_uptime: u64,
}