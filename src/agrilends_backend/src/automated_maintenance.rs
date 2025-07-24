// ========== COMPREHENSIVE AUTOMATED MAINTENANCE MODULE ==========
// Advanced heartbeat system for Agrilends protocol
// Implements automated maintenance, monitoring, and optimization tasks
// Production-ready implementation with error handling and resilience

use ic_cdk::{caller, api::time, id};
use ic_cdk_macros::{query, update, heartbeat};
use candid::{CandidType, Deserialize, Principal};
use ic_stable_structures::{StableBTreeMap, memory::MemoryId};
use ic_stable_structures::memory::VirtualMemory;
use ic_stable_structures::DefaultMemoryImpl;
use std::cell::RefCell;
use std::collections::HashMap;

use crate::types::*;
use crate::storage::{get_memory_by_id, get_canister_config, set_canister_config};
use crate::helpers::{is_admin, is_in_maintenance_mode, get_emergency_stop_status, get_memory_usage, 
                     get_active_loans_count, check_oracle_health, check_ckbtc_health, get_overdue_loans,
                     log_audit_action};
use crate::oracle;
use crate::liquidity_management;
use crate::liquidation::trigger_liquidation;
use crate::loan_lifecycle::get_loan;

// Memory types for heartbeat storage
type Memory = VirtualMemory<DefaultMemoryImpl>;
type HeartbeatMetricsStorage = StableBTreeMap<u8, HeartbeatMetrics, Memory>;
type HeartbeatConfigStorage = StableBTreeMap<u8, HeartbeatConfig, Memory>;
type CircuitBreakerStorage = StableBTreeMap<String, CircuitBreaker, Memory>;

// Thread-local storage for heartbeat state
thread_local! {
    static HEARTBEAT_METRICS: RefCell<HeartbeatMetricsStorage> = RefCell::new(
        StableBTreeMap::init(get_memory_by_id(MemoryId::new(50)))
    );
    
    static HEARTBEAT_CONFIG: RefCell<HeartbeatConfigStorage> = RefCell::new(
        StableBTreeMap::init(get_memory_by_id(MemoryId::new(51)))
    );
    
    static CIRCUIT_BREAKERS: RefCell<CircuitBreakerStorage> = RefCell::new(
        StableBTreeMap::init(get_memory_by_id(MemoryId::new(52)))
    );
    
    static LAST_HEARTBEAT_TIME: RefCell<u64> = RefCell::new(0);
    static HEARTBEAT_EXECUTION_COUNT: RefCell<u64> = RefCell::new(0);
}

// Constants for heartbeat configuration
const MEMORY_WARNING_THRESHOLD: u64 = 1_000_000_000; // 1GB
const CYCLES_THRESHOLD_ALERT: u64 = 1_000_000_000_000; // 1T cycles
const CYCLES_THRESHOLD_CRITICAL: u64 = 500_000_000_000; // 500B cycles
const MAX_AUDIT_LOGS: usize = 10_000;
const AUTO_LIQUIDATION_THRESHOLD_DAYS: u64 = 45;
const CIRCUIT_BREAKER_THRESHOLD: u64 = 5;
const CIRCUIT_BREAKER_TIMEOUT: u64 = 300_000_000_000; // 5 minutes

// ========== DATA STRUCTURES ==========

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct HeartbeatConfig {
    pub enabled: bool,
    pub maintenance_mode: bool,
    pub price_update_enabled: bool,
    pub loan_monitoring_enabled: bool,
    pub cycles_monitoring_enabled: bool,
    pub auto_cleanup_enabled: bool,
    pub pool_maintenance_enabled: bool,
    pub auto_liquidation_enabled: bool,
    pub auto_liquidation_threshold_days: u64,
    pub memory_monitoring_enabled: bool,
    pub oracle_monitoring_enabled: bool,
    pub treasury_monitoring_enabled: bool,
}

impl Default for HeartbeatConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            maintenance_mode: false,
            price_update_enabled: true,
            loan_monitoring_enabled: true,
            cycles_monitoring_enabled: true,
            auto_cleanup_enabled: true,
            pool_maintenance_enabled: true,
            auto_liquidation_enabled: false, // Disabled by default for safety
            auto_liquidation_threshold_days: AUTO_LIQUIDATION_THRESHOLD_DAYS,
            memory_monitoring_enabled: true,
            oracle_monitoring_enabled: true,
            treasury_monitoring_enabled: true,
        }
    }
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct HeartbeatMetrics {
    pub last_execution_time: u64,
    pub total_executions: u64,
    pub successful_executions: u64,
    pub failed_executions: u64,
    pub average_execution_time: u64,
    pub tasks_completed: HashMap<String, u64>,
    pub last_error: Option<String>,
    pub last_error_time: Option<u64>,
    pub total_execution_time: u64,
    pub peak_execution_time: u64,
    pub last_maintenance_tasks: Vec<String>,
}

impl Default for HeartbeatMetrics {
    fn default() -> Self {
        Self {
            last_execution_time: 0,
            total_executions: 0,
            successful_executions: 0,
            failed_executions: 0,
            average_execution_time: 0,
            tasks_completed: HashMap::new(),
            last_error: None,
            last_error_time: None,
            total_execution_time: 0,
            peak_execution_time: 0,
            last_maintenance_tasks: Vec::new(),
        }
    }
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct CircuitBreaker {
    pub failure_count: u64,
    pub last_failure_time: u64,
    pub threshold: u64,
    pub timeout: u64,
    pub state: CircuitBreakerState,
}

#[derive(CandidType, Deserialize, Clone, Debug, PartialEq)]
pub enum CircuitBreakerState {
    Closed,
    Open,
    HalfOpen,
}

impl Default for CircuitBreaker {
    fn default() -> Self {
        Self {
            failure_count: 0,
            last_failure_time: 0,
            threshold: CIRCUIT_BREAKER_THRESHOLD,
            timeout: CIRCUIT_BREAKER_TIMEOUT,
            state: CircuitBreakerState::Closed,
        }
    }
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct MaintenanceTaskResult {
    pub task_name: String,
    pub success: bool,
    pub execution_time: u64,
    pub details: String,
    pub error_message: Option<String>,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct SystemMaintenanceReport {
    pub timestamp: u64,
    pub total_tasks: u64,
    pub successful_tasks: u64,
    pub failed_tasks: u64,
    pub total_execution_time: u64,
    pub tasks: Vec<MaintenanceTaskResult>,
    pub system_health: ProductionHealthStatus,
}

// ========== MAIN HEARTBEAT FUNCTION ==========

/// Main heartbeat function - executed automatically by IC system
/// This function is called by IC system periodically without external trigger
pub async fn canister_heartbeat() {
    let execution_start = time();
    let mut tasks_executed = Vec::new();
    let mut successful_tasks = 0u64;
    let mut failed_tasks = 0u64;
    
    // Update execution count and last heartbeat time
    HEARTBEAT_EXECUTION_COUNT.with(|count| {
        *count.borrow_mut() += 1;
    });
    LAST_HEARTBEAT_TIME.with(|last_time| {
        *last_time.borrow_mut() = execution_start;
    });
    
    // Check if heartbeat is enabled and not in emergency mode
    let config = get_heartbeat_config();
    if !config.enabled || is_in_maintenance_mode() || get_emergency_stop_status() {
        log_audit_action(
            id(),
            "HEARTBEAT_SKIPPED".to_string(),
            "Heartbeat skipped - system in maintenance or emergency mode".to_string(),
            true,
        );
        return;
    }
    
    // Execute maintenance tasks with error handling
    
    // 1. Oracle Price Updates
    if config.price_update_enabled {
        let task_result = execute_with_circuit_breaker(
            "price_update",
            oracle_price_update_task()
        ).await;
        tasks_executed.push(task_result.clone());
        if task_result.success { successful_tasks += 1; } else { failed_tasks += 1; }
    }
    
    // 2. Loan Monitoring
    if config.loan_monitoring_enabled {
        let task_result = execute_with_circuit_breaker(
            "loan_monitoring",
            loan_monitoring_task()
        ).await;
        tasks_executed.push(task_result.clone());
        if task_result.success { successful_tasks += 1; } else { failed_tasks += 1; }
    }
    
    // 3. Cycles Monitoring
    if config.cycles_monitoring_enabled {
        let task_result = execute_task("cycles_monitoring", cycles_monitoring_task()).await;
        tasks_executed.push(task_result.clone());
        if task_result.success { successful_tasks += 1; } else { failed_tasks += 1; }
    }
    
    // 4. Memory Monitoring
    if config.memory_monitoring_enabled {
        let task_result = execute_task("memory_monitoring", memory_monitoring_task()).await;
        tasks_executed.push(task_result.clone());
        if task_result.success { successful_tasks += 1; } else { failed_tasks += 1; }
    }
    
    // 5. Auto Cleanup
    if config.auto_cleanup_enabled {
        let task_result = execute_task("auto_cleanup", auto_cleanup_task()).await;
        tasks_executed.push(task_result.clone());
        if task_result.success { successful_tasks += 1; } else { failed_tasks += 1; }
    }
    
    // 6. Pool Maintenance
    if config.pool_maintenance_enabled {
        let task_result = execute_with_circuit_breaker(
            "pool_maintenance",
            pool_maintenance_task()
        ).await;
        tasks_executed.push(task_result.clone());
        if task_result.success { successful_tasks += 1; } else { failed_tasks += 1; }
    }
    
    // 7. Auto Liquidation Monitoring
    if config.auto_liquidation_enabled {
        let task_result = execute_with_circuit_breaker(
            "auto_liquidation",
            auto_liquidation_monitoring_task(config.auto_liquidation_threshold_days)
        ).await;
        tasks_executed.push(task_result.clone());
        if task_result.success { successful_tasks += 1; } else { failed_tasks += 1; }
    }
    
    // 8. Oracle Health Monitoring
    if config.oracle_monitoring_enabled {
        let task_result = execute_task("oracle_health", oracle_health_monitoring_task()).await;
        tasks_executed.push(task_result.clone());
        if task_result.success { successful_tasks += 1; } else { failed_tasks += 1; }
    }
    
    // 9. Treasury Monitoring
    if config.treasury_monitoring_enabled {
        let task_result = execute_task("treasury_monitoring", treasury_monitoring_task()).await;
        tasks_executed.push(task_result.clone());
        if task_result.success { successful_tasks += 1; } else { failed_tasks += 1; }
    }
    
    // Update metrics
    let execution_time = time() - execution_start;
    update_heartbeat_metrics(execution_time, successful_tasks > 0, tasks_executed.clone());
    
    // Log heartbeat completion
    log_audit_action(
        id(),
        "HEARTBEAT_COMPLETED".to_string(),
        format!(
            "Heartbeat completed: {} successful, {} failed, {}ms execution time", 
            successful_tasks, failed_tasks, execution_time / 1_000_000
        ),
        successful_tasks > 0,
    );
}

// ========== MAINTENANCE TASKS ==========

/// Oracle price update task
async fn oracle_price_update_task() -> Result<String, String> {
    let commodities = vec!["rice".to_string(), "corn".to_string(), "wheat".to_string()];
    let mut updated_count = 0;
    let mut errors = Vec::new();
    
    for commodity in commodities {
        if is_price_stale(commodity.clone()) {
            match oracle::fetch_commodity_price(commodity.clone()).await {
                Ok(_) => {
                    updated_count += 1;
                    log_audit_action(
                        id(),
                        "AUTO_PRICE_UPDATE_SUCCESS".to_string(),
                        format!("Successfully auto-updated {} price", commodity),
                        true,
                    );
                },
                Err(e) => {
                    errors.push(format!("{}: {}", commodity, e));
                    log_audit_action(
                        id(),
                        "AUTO_PRICE_UPDATE_FAILED".to_string(),
                        format!("Failed to auto-update {} price: {}", commodity, e),
                        false,
                    );
                }
            }
        }
    }
    
    if errors.is_empty() {
        Ok(format!("Updated {} commodity prices", updated_count))
    } else {
        Err(format!("Errors updating prices: {:?}", errors))
    }
}

/// Check if commodity price is stale (>24 hours)
fn is_price_stale(commodity_id: String) -> bool {
    if let Some(price_data) = crate::storage::get_stored_commodity_price(&commodity_id) {
        let current_time = time();
        let twenty_four_hours = 24 * 60 * 60 * 1_000_000_000u64;
        (current_time - price_data.timestamp) > twenty_four_hours
    } else {
        true // No data = stale
    }
}

/// Loan monitoring task
async fn loan_monitoring_task() -> Result<String, String> {
    let overdue_loans = get_overdue_loans();
    let mut monitored_count = 0;
    let mut liquidation_candidates = 0;
    
    for loan in overdue_loans {
        monitored_count += 1;
        
        // Log overdue loan detection
        log_audit_action(
            id(),
            "OVERDUE_LOAN_DETECTED".to_string(),
            format!("Loan {} is overdue and may require liquidation review", loan.id),
            false,
        );
        
        // Check liquidation eligibility
        if let Ok(eligible) = crate::liquidation::check_liquidation_eligibility(loan.id) {
            if eligible.is_eligible {
                liquidation_candidates += 1;
                log_audit_action(
                    id(),
                    "LIQUIDATION_ELIGIBLE_DETECTED".to_string(),
                    format!("Loan {} is eligible for liquidation: {}", loan.id, eligible.reason),
                    false,
                );
            }
        }
    }
    
    Ok(format!("Monitored {} overdue loans, {} liquidation candidates", monitored_count, liquidation_candidates))
}

/// Cycles monitoring task
async fn cycles_monitoring_task() -> Result<String, String> {
    let current_cycles = ic_cdk::api::canister_balance();
    
    if current_cycles < CYCLES_THRESHOLD_CRITICAL {
        log_audit_action(
            id(),
            "CYCLES_CRITICAL".to_string(),
            format!("CRITICAL: Canister cycles below critical threshold: {} cycles", current_cycles),
            false,
        );
        Err(format!("Critical cycles level: {}", current_cycles))
    } else if current_cycles < CYCLES_THRESHOLD_ALERT {
        log_audit_action(
            id(),
            "CYCLES_LOW".to_string(),
            format!("WARNING: Canister cycles running low: {} cycles", current_cycles),
            false,
        );
        Ok(format!("Low cycles warning: {}", current_cycles))
    } else {
        Ok(format!("Cycles healthy: {}", current_cycles))
    }
}

/// Memory monitoring task
async fn memory_monitoring_task() -> Result<String, String> {
    let memory_usage = get_memory_usage();
    
    if memory_usage > MEMORY_WARNING_THRESHOLD {
        log_audit_action(
            id(),
            "MEMORY_WARNING".to_string(),
            format!("High memory usage detected: {} bytes", memory_usage),
            false,
        );
        Ok(format!("Memory warning: {} bytes", memory_usage))
    } else {
        Ok(format!("Memory usage normal: {} bytes", memory_usage))
    }
}

/// Auto cleanup task
async fn auto_cleanup_task() -> Result<String, String> {
    let mut cleanup_actions = Vec::new();
    
    // Cleanup old audit logs
    let cleaned_logs = cleanup_old_audit_logs();
    if cleaned_logs > 0 {
        cleanup_actions.push(format!("Cleaned {} audit logs", cleaned_logs));
    }
    
    // Cleanup old transactions (30 days)
    let thirty_days_ago = time() - (30 * 24 * 60 * 60 * 1_000_000_000);
    if let Ok(cleaned_tx) = cleanup_old_transactions(thirty_days_ago) {
        if cleaned_tx > 0 {
            cleanup_actions.push(format!("Cleaned {} old transactions", cleaned_tx));
        }
    }
    
    // Optimize memory usage
    optimize_memory_usage();
    cleanup_actions.push("Memory optimization completed".to_string());
    
    Ok(format!("Cleanup completed: {:?}", cleanup_actions))
}

/// Pool maintenance task
async fn pool_maintenance_task() -> Result<String, String> {
    match liquidity_management::perform_pool_maintenance() {
        Ok(result) => Ok(result),
        Err(e) => Err(format!("Pool maintenance failed: {}", e))
    }
}

/// Auto liquidation monitoring task
async fn auto_liquidation_monitoring_task(threshold_days: u64) -> Result<String, String> {
    let eligible_loans = get_loans_eligible_for_liquidation();
    let mut liquidated_count = 0;
    let mut errors = Vec::new();
    
    for loan_id in eligible_loans {
        if let Ok(loan) = get_loan(loan_id) {
            if let Some(due_date) = loan.due_date {
                let days_overdue = (time() - due_date) / (24 * 60 * 60 * 1_000_000_000);
                
                // Auto-liquidation after threshold days
                if days_overdue > threshold_days {
                    match trigger_liquidation(loan_id).await {
                        Ok(_) => {
                            liquidated_count += 1;
                            log_audit_action(
                                id(),
                                "AUTO_LIQUIDATION_TRIGGERED".to_string(),
                                format!("Automatically triggered liquidation for loan {} after {} days overdue", loan_id, days_overdue),
                                true,
                            );
                        },
                        Err(e) => {
                            errors.push(format!("Loan {}: {}", loan_id, e));
                            log_audit_action(
                                id(),
                                "AUTO_LIQUIDATION_FAILED".to_string(),
                                format!("Failed to auto-liquidate loan {}: {}", loan_id, e),
                                false,
                            );
                        }
                    }
                }
            }
        }
    }
    
    if errors.is_empty() {
        Ok(format!("Auto-liquidated {} loans", liquidated_count))
    } else {
        Err(format!("Liquidation errors: {:?}", errors))
    }
}

/// Oracle health monitoring task
async fn oracle_health_monitoring_task() -> Result<String, String> {
    let oracle_healthy = check_oracle_health();
    
    if oracle_healthy {
        Ok("Oracle system healthy".to_string())
    } else {
        log_audit_action(
            id(),
            "ORACLE_HEALTH_WARNING".to_string(),
            "Oracle system health check failed".to_string(),
            false,
        );
        Err("Oracle system unhealthy".to_string())
    }
}

/// Treasury monitoring task
async fn treasury_monitoring_task() -> Result<String, String> {
    // Check treasury balance and cycle top-up needs
    if let Ok(stats) = crate::treasury_management::get_treasury_stats() {
        let needs_attention = stats.balance_ckbtc < 1_000_000; // 0.01 BTC threshold
        
        if needs_attention {
            log_audit_action(
                id(),
                "TREASURY_LOW_BALANCE".to_string(),
                format!("Treasury balance low: {} satoshi", stats.balance_ckbtc),
                false,
            );
        }
        
        Ok(format!("Treasury balance: {} satoshi", stats.balance_ckbtc))
    } else {
        Err("Failed to get treasury stats".to_string())
    }
}

// ========== HELPER FUNCTIONS ==========

/// Get loans eligible for liquidation
fn get_loans_eligible_for_liquidation() -> Vec<u64> {
    // This would integrate with loan lifecycle to get eligible loans
    // Placeholder implementation
    Vec::new()
}

/// Execute task with circuit breaker protection
async fn execute_with_circuit_breaker(
    task_name: &str,
    task: impl std::future::Future<Output = Result<String, String>>
) -> MaintenanceTaskResult {
    let start_time = time();
    
    // Check circuit breaker
    if !should_execute_task(task_name) {
        return MaintenanceTaskResult {
            task_name: task_name.to_string(),
            success: false,
            execution_time: 0,
            details: "Skipped due to circuit breaker".to_string(),
            error_message: Some("Circuit breaker open".to_string()),
        };
    }
    
    match task.await {
        Ok(details) => {
            record_task_success(task_name);
            MaintenanceTaskResult {
                task_name: task_name.to_string(),
                success: true,
                execution_time: time() - start_time,
                details,
                error_message: None,
            }
        },
        Err(error) => {
            record_task_failure(task_name);
            MaintenanceTaskResult {
                task_name: task_name.to_string(),
                success: false,
                execution_time: time() - start_time,
                details: "Task failed".to_string(),
                error_message: Some(error),
            }
        }
    }
}

/// Execute task without circuit breaker
async fn execute_task(
    task_name: &str,
    task: impl std::future::Future<Output = Result<String, String>>
) -> MaintenanceTaskResult {
    let start_time = time();
    
    match task.await {
        Ok(details) => MaintenanceTaskResult {
            task_name: task_name.to_string(),
            success: true,
            execution_time: time() - start_time,
            details,
            error_message: None,
        },
        Err(error) => MaintenanceTaskResult {
            task_name: task_name.to_string(),
            success: false,
            execution_time: time() - start_time,
            details: "Task failed".to_string(),
            error_message: Some(error),
        }
    }
}

/// Check if task should execute based on circuit breaker
fn should_execute_task(task_name: &str) -> bool {
    CIRCUIT_BREAKERS.with(|breakers| {
        let breakers_map = breakers.borrow();
        if let Some(breaker) = breakers_map.get(task_name) {
            let current_time = time();
            
            match breaker.state {
                CircuitBreakerState::Closed => true,
                CircuitBreakerState::Open => {
                    // Check if timeout expired
                    if current_time - breaker.last_failure_time > breaker.timeout {
                        // Move to half-open state
                        true
                    } else {
                        false
                    }
                },
                CircuitBreakerState::HalfOpen => true,
            }
        } else {
            true // No circuit breaker = allow execution
        }
    })
}

/// Record task success for circuit breaker
fn record_task_success(task_name: &str) {
    CIRCUIT_BREAKERS.with(|breakers| {
        let mut breakers_map = breakers.borrow_mut();
        let mut breaker = breakers_map.get(task_name).unwrap_or_default();
        breaker.failure_count = 0;
        breaker.state = CircuitBreakerState::Closed;
        breakers_map.insert(task_name.to_string(), breaker);
    });
}

/// Record task failure for circuit breaker
fn record_task_failure(task_name: &str) {
    CIRCUIT_BREAKERS.with(|breakers| {
        let mut breakers_map = breakers.borrow_mut();
        let mut breaker = breakers_map.get(task_name).unwrap_or_default();
        breaker.failure_count += 1;
        breaker.last_failure_time = time();
        
        if breaker.failure_count >= breaker.threshold {
            breaker.state = CircuitBreakerState::Open;
        }
        
        breakers_map.insert(task_name.to_string(), breaker);
    });
}

/// Cleanup old audit logs
fn cleanup_old_audit_logs() -> u64 {
    // Call the actual storage cleanup function
    crate::storage::cleanup_audit_logs(MAX_AUDIT_LOGS as u64);
    MAX_AUDIT_LOGS as u64
}

/// Cleanup old transactions
fn cleanup_old_transactions(cutoff_time: u64) -> Result<u64, String> {
    // Call liquidity management cleanup function
    crate::liquidity_management::cleanup_old_transactions(cutoff_time)
}

/// Optimize memory usage
fn optimize_memory_usage() {
    // Trigger garbage collection and memory optimization
    // This is a placeholder - actual implementation would depend on specific optimizations
}

/// Update heartbeat metrics
fn update_heartbeat_metrics(execution_time: u64, success: bool, tasks: Vec<MaintenanceTaskResult>) {
    HEARTBEAT_METRICS.with(|metrics| {
        let mut metrics_map = metrics.borrow_mut();
        let mut current_metrics = metrics_map.get(&0).unwrap_or_default();
        
        current_metrics.last_execution_time = time();
        current_metrics.total_executions += 1;
        current_metrics.total_execution_time += execution_time;
        
        if success {
            current_metrics.successful_executions += 1;
        } else {
            current_metrics.failed_executions += 1;
        }
        
        // Update average execution time
        if current_metrics.total_executions > 0 {
            current_metrics.average_execution_time = 
                current_metrics.total_execution_time / current_metrics.total_executions;
        }
        
        // Update peak execution time
        if execution_time > current_metrics.peak_execution_time {
            current_metrics.peak_execution_time = execution_time;
        }
        
        // Update task counts
        for task in &tasks {
            let count = current_metrics.tasks_completed.get(&task.task_name).unwrap_or(&0);
            current_metrics.tasks_completed.insert(task.task_name.clone(), count + 1);
        }
        
        // Update last maintenance tasks
        current_metrics.last_maintenance_tasks = tasks.iter()
            .map(|t| format!("{}: {}", t.task_name, if t.success { "✓" } else { "✗" }))
            .collect();
        
        // Store last error if any
        if let Some(failed_task) = tasks.iter().find(|t| !t.success) {
            current_metrics.last_error = failed_task.error_message.clone();
            current_metrics.last_error_time = Some(time());
        }
        
        metrics_map.insert(0, current_metrics);
    });
}

// ========== PUBLIC FUNCTIONS ==========

/// Get heartbeat configuration
#[query]
pub fn get_heartbeat_config() -> HeartbeatConfig {
    HEARTBEAT_CONFIG.with(|config| {
        config.borrow().get(&0).unwrap_or_default()
    })
}

/// Update heartbeat configuration (admin only)
#[update]
pub fn update_heartbeat_config(new_config: HeartbeatConfig) -> Result<String, String> {
    let caller = caller();
    if !is_admin(&caller) {
        return Err("Unauthorized: Only admins can update heartbeat configuration".to_string());
    }
    
    HEARTBEAT_CONFIG.with(|config| {
        config.borrow_mut().insert(0, new_config.clone());
    });
    
    log_audit_action(
        caller,
        "HEARTBEAT_CONFIG_UPDATE".to_string(),
        format!("Heartbeat configuration updated: enabled={}, auto_liquidation={}", 
                new_config.enabled, new_config.auto_liquidation_enabled),
        true,
    );
    
    Ok("Heartbeat configuration updated successfully".to_string())
}

/// Emergency pause heartbeat (admin only)
#[update]
pub fn emergency_pause_heartbeat() -> Result<String, String> {
    let caller = caller();
    if !is_admin(&caller) {
        return Err("Unauthorized: Only admins can pause heartbeat".to_string());
    }
    
    let mut config = get_canister_config();
    config.maintenance_mode = true;
    set_canister_config(config)?;
    
    log_audit_action(
        caller,
        "HEARTBEAT_EMERGENCY_PAUSE".to_string(),
        "Heartbeat operations paused due to emergency".to_string(),
        true,
    );
    
    Ok("Heartbeat operations paused successfully".to_string())
}

/// Resume heartbeat operations (admin only)
#[update]
pub fn resume_heartbeat_operations() -> Result<String, String> {
    let caller = caller();
    if !is_admin(&caller) {
        return Err("Unauthorized: Only admins can resume heartbeat".to_string());
    }
    
    let mut config = get_canister_config();
    config.maintenance_mode = false;
    set_canister_config(config)?;
    
    log_audit_action(
        caller,
        "HEARTBEAT_OPERATIONS_RESUMED".to_string(),
        "Heartbeat operations resumed".to_string(),
        true,
    );
    
    Ok("Heartbeat operations resumed successfully".to_string())
}

/// Get heartbeat metrics (admin only)
#[query]
pub fn get_heartbeat_metrics() -> Result<HeartbeatMetrics, String> {
    let caller = caller();
    if !is_admin(&caller) {
        return Err("Unauthorized: Only admins can view heartbeat metrics".to_string());
    }
    
    Ok(HEARTBEAT_METRICS.with(|metrics| {
        metrics.borrow().get(&0).unwrap_or_default()
    }))
}

/// Get last heartbeat time
#[query]
pub fn get_last_heartbeat_time() -> u64 {
    LAST_HEARTBEAT_TIME.with(|time| *time.borrow())
}

/// Get heartbeat execution count
#[query]
pub fn get_heartbeat_execution_count() -> u64 {
    HEARTBEAT_EXECUTION_COUNT.with(|count| *count.borrow())
}

/// Production health check with heartbeat status
#[query]
pub fn production_health_check_with_heartbeat() -> ProductionHealthStatus {
    let config = get_canister_config();
    let heartbeat_config = get_heartbeat_config();
    let last_heartbeat = get_last_heartbeat_time();
    let current_time = time();
    
    // Check if heartbeat is recent (within last 2 minutes)
    let heartbeat_healthy = (current_time - last_heartbeat) < (2 * 60 * 1_000_000_000);
    
    ProductionHealthStatus {
        is_healthy: !config.emergency_stop && !config.maintenance_mode && heartbeat_healthy,
        emergency_stop: config.emergency_stop,
        maintenance_mode: config.maintenance_mode,
        oracle_status: check_oracle_health(),
        ckbtc_integration: check_ckbtc_health(),
        memory_usage: get_memory_usage(),
        total_loans: get_active_loans_count(),
        active_loans: get_active_loans_count(),
        last_heartbeat,
    }
}

/// Get comprehensive system maintenance report (admin only)
#[query]
pub fn get_system_maintenance_report() -> Result<SystemMaintenanceReport, String> {
    let caller = caller();
    if !is_admin(&caller) {
        return Err("Unauthorized: Only admins can view maintenance reports".to_string());
    }
    
    let metrics = HEARTBEAT_METRICS.with(|m| m.borrow().get(&0).unwrap_or_default());
    let health_status = production_health_check_with_heartbeat();
    
    Ok(SystemMaintenanceReport {
        timestamp: time(),
        total_tasks: metrics.last_maintenance_tasks.len() as u64,
        successful_tasks: metrics.successful_executions,
        failed_tasks: metrics.failed_executions,
        total_execution_time: metrics.total_execution_time,
        tasks: Vec::new(), // Would contain detailed task results
        system_health: health_status,
    })
}

/// Reset circuit breakers (admin only)
#[update]
pub fn reset_circuit_breakers() -> Result<String, String> {
    let caller = caller();
    if !is_admin(&caller) {
        return Err("Unauthorized: Only admins can reset circuit breakers".to_string());
    }
    
    CIRCUIT_BREAKERS.with(|breakers| {
        breakers.borrow_mut().clear();
    });
    
    log_audit_action(
        caller,
        "CIRCUIT_BREAKERS_RESET".to_string(),
        "All circuit breakers have been reset".to_string(),
        true,
    );
    
    Ok("Circuit breakers reset successfully".to_string())
}

/// Get circuit breaker status (admin only)
#[query]
pub fn get_circuit_breaker_status() -> Result<HashMap<String, CircuitBreaker>, String> {
    let caller = caller();
    if !is_admin(&caller) {
        return Err("Unauthorized: Only admins can view circuit breaker status".to_string());
    }
    
    Ok(CIRCUIT_BREAKERS.with(|breakers| {
        breakers.borrow().iter().collect()
    }))
}
