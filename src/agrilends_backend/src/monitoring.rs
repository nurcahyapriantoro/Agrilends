use ic_cdk_macros::query;
use ic_cdk::api::time;
use crate::storage::{get_storage_stats, get_config, get_audit_logs};
use crate::user_management::get_user_stats;
use crate::types::StorageStats;
use crate::production_config::SystemHealth;

#[derive(candid::CandidType, candid::Deserialize, Clone, Debug)]
pub struct SystemMetrics {
    pub storage_stats: StorageStats,
    pub user_stats: crate::user_management::UserStats,
    pub nft_stats: crate::types::NFTStats,
    pub system_health: SystemHealth,
    pub recent_audit_logs: Vec<crate::types::AuditLog>,
}

/// Get comprehensive system metrics (admin only)
#[query]
pub fn get_system_metrics() -> Result<SystemMetrics, String> {
    let caller = ic_cdk::api::caller();
    
    if !crate::helpers::is_admin(&caller) {
        return Err("Unauthorized: Only admins can view system metrics".to_string());
    }
    
    Ok(SystemMetrics {
        storage_stats: get_storage_stats(),
        user_stats: get_user_stats(),
        nft_stats: crate::rwa_nft::get_nft_stats(),
        system_health: crate::production_config::get_system_health(),
        recent_audit_logs: get_audit_logs(Some(100)), // Last 100 logs
    })
}

/// Health check endpoint for monitoring systems
#[query]
pub fn health_check_detailed() -> HealthCheckResult {
    let config = get_config();
    
    HealthCheckResult {
        status: if config.emergency_stop { "EMERGENCY_STOP".to_string() } 
               else if config.maintenance_mode { "MAINTENANCE".to_string() }
               else { "HEALTHY".to_string() },
        timestamp: time(),
        version: "1.0.0".to_string(),
        uptime: time(),
    }
}

#[derive(candid::CandidType, candid::Deserialize, Clone, Debug)]
pub struct HealthCheckResult {
    pub status: String,
    pub timestamp: u64,
    pub version: String,
    pub uptime: u64,
}
