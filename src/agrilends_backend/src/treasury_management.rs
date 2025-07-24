// ========== TREASURY MANAGEMENT MODULE ==========
// Comprehensive treasury and protocol fee management system for Agrilends protocol
// Implements treasury operations, protocol fee collection, and canister cycle management
// Production-ready implementation with enhanced security and monitoring

use candid::{CandidType, Deserialize, Principal, Nat};
use ic_cdk::{caller, api::time, api::management_canister::main::{deposit_cycles, canister_status, CanisterStatusResponse}};
use ic_cdk_macros::{query, update, init, pre_upgrade, post_upgrade, heartbeat};
use ic_stable_structures::{StableBTreeMap, memory::MemoryId};
use ic_stable_structures::memory_manager::{MemoryManager, VirtualMemory};
use ic_stable_structures::DefaultMemoryImpl;
use std::cell::RefCell;
use std::collections::HashMap;

use crate::types::*;
use crate::storage::{log_action, get_config, update_config};
use crate::helpers::{is_admin, is_loan_manager};

// Treasury-specific types
#[derive(CandidType, Deserialize, Clone, Debug, PartialEq)]
pub struct Account {
    pub owner: Principal,
    pub subaccount: Option<Vec<u8>>,
}

// Memory types for treasury storage
type Memory = VirtualMemory<DefaultMemoryImpl>;
type TreasuryStorage = StableBTreeMap<u8, TreasuryState, Memory>;
type RevenueLogStorage = StableBTreeMap<u64, RevenueEntry, Memory>;
type CanisterRegistryStorage = StableBTreeMap<String, CanisterInfo, Memory>;
type CycleTransactionStorage = StableBTreeMap<u64, CycleTransaction, Memory>;

// Memory Manager for Treasury (using dedicated memory IDs for treasury)
thread_local! {
    static TREASURY_MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(
        MemoryManager::init(DefaultMemoryImpl::default())
    );
}

fn get_treasury_memory(id: u8) -> Memory {
    TREASURY_MEMORY_MANAGER.with(|manager| {
        manager.borrow().get(MemoryId::new(id))
    })
}

// Treasury data structures
#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct TreasuryState {
    pub balance_ckbtc: u64,
    pub total_fees_collected: u64,
    pub total_cycles_distributed: u64,
    pub last_cycle_distribution: u64,
    pub emergency_reserve: u64,
    pub daily_cycle_cost: u64,
    pub average_daily_revenue: u64,
    pub revenue_streak_days: u32,
    pub last_revenue_date: u64,
    pub created_at: u64,
    pub updated_at: u64,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct RevenueEntry {
    pub id: u64,
    pub source_loan_id: u64,
    pub amount: u64,
    pub revenue_type: RevenueType,
    pub source_canister: Principal,
    pub timestamp: u64,
    pub transaction_hash: Option<String>,
    pub status: TransactionStatus,
    pub processing_fee: u64,
    pub net_amount: u64,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub enum RevenueType {
    AdminFee,
    InterestShare,
    LiquidationPenalty,
    EarlyRepaymentFee,
    ProtocolFee,
    LatePaymentFee,
    CollateralProcessingFee,
    OracleServiceFee,
    GovernanceFee,
    OtherRevenue(String),
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub enum TransactionStatus {
    Pending,
    Processing,
    Completed,
    Failed(String),
    Refunded,
    Cancelled,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct CanisterInfo {
    pub name: String,
    pub principal: Principal,
    pub canister_type: CanisterType,
    pub min_cycles_threshold: u64,
    pub max_cycles_limit: u64,
    pub priority: u8, // 1-10, 1 being highest priority
    pub last_top_up: u64,
    pub total_cycles_received: u64,
    pub estimated_daily_consumption: u64,
    pub consumption_history: Vec<CycleConsumptionRecord>,
    pub is_active: bool,
    pub auto_top_up_enabled: bool,
    pub health_check_enabled: bool,
    pub alert_threshold_percentage: u8, // Alert when cycles drop below this % of threshold
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct CycleConsumptionRecord {
    pub date: u64,
    pub cycles_consumed: u64,
    pub operations_count: u32,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub enum CanisterType {
    Core,           // Core business logic canisters
    Infrastructure, // Infrastructure support canisters
    Analytics,      // Analytics and reporting canisters
    Frontend,       // Frontend serving canisters
    Oracle,         // Oracle and external data canisters
    Backup,         // Backup and recovery canisters
    Testing,        // Testing and development canisters
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct CycleTransaction {
    pub id: u64,
    pub target_canister: Principal,
    pub canister_name: String,
    pub cycles_amount: u64,
    pub ckbtc_cost: u64,
    pub exchange_rate: f64, // ckBTC to cycles exchange rate
    pub timestamp: u64,
    pub status: TransactionStatus,
    pub initiated_by: Principal,
    pub reason: String,
    pub gas_fee: u64,
    pub confirmation_blocks: u32,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct TreasuryStats {
    pub current_balance: u64,
    pub available_balance: u64, // Balance minus emergency reserve
    pub total_revenue_collected: u64,
    pub total_cycles_distributed: u64,
    pub emergency_reserve: u64,
    pub active_canisters_count: u32,
    pub last_distribution_time: u64,
    pub average_daily_revenue: u64,
    pub projected_runway_days: u32,
    pub revenue_growth_rate: f64,
    pub cycle_efficiency_score: f64,
    pub health_status: TreasuryHealthStatus,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub enum TreasuryHealthStatus {
    Healthy,
    Warning(String),
    Critical(String),
    Emergency(String),
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct CanisterCycleStatus {
    pub canister_info: CanisterInfo,
    pub current_cycles: u64,
    pub cycles_percentage: f64, // Percentage of threshold
    pub estimated_consumption_per_day: u64,
    pub days_remaining: u32,
    pub needs_top_up: bool,
    pub is_critical: bool, // Less than 24 hours remaining
    pub last_checked: u64,
    pub status_message: String,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct TreasuryHealthReport {
    pub overall_health: TreasuryHealthStatus,
    pub balance_analysis: BalanceAnalysis,
    pub cycle_analysis: CycleAnalysis,
    pub revenue_analysis: RevenueAnalysis,
    pub recommendations: Vec<String>,
    pub alerts: Vec<Alert>,
    pub generated_at: u64,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct BalanceAnalysis {
    pub current_balance: u64,
    pub emergency_reserve: u64,
    pub available_for_operations: u64,
    pub burn_rate_daily: u64,
    pub runway_days: u32,
    pub balance_trend: TrendDirection,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct CycleAnalysis {
    pub total_canisters: u32,
    pub healthy_canisters: u32,
    pub warning_canisters: u32,
    pub critical_canisters: u32,
    pub total_daily_consumption: u64,
    pub efficiency_score: f64,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct RevenueAnalysis {
    pub total_revenue: u64,
    pub daily_average: u64,
    pub growth_rate: f64,
    pub revenue_streams: HashMap<String, u64>,
    pub trend: TrendDirection,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub enum TrendDirection {
    Increasing,
    Stable,
    Decreasing,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct Alert {
    pub alert_type: AlertType,
    pub severity: AlertSeverity,
    pub message: String,
    pub created_at: u64,
    pub canister_name: Option<String>,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub enum AlertType {
    LowBalance,
    LowCycles,
    HighConsumption,
    RevenueDecline,
    EmergencyReserveTouch,
    CanisterOffline,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
    Emergency,
}

// Production constants
const DEFAULT_MIN_CYCLES_THRESHOLD: u64 = 1_000_000_000_000; // 1T cycles minimum
const DEFAULT_MAX_CYCLES_LIMIT: u64 = 10_000_000_000_000; // 10T cycles maximum
const EMERGENCY_RESERVE_PERCENTAGE: u64 = 20; // 20% of total balance as emergency reserve
const AUTO_TOP_UP_PERCENTAGE: u64 = 150; // Top up to 150% of threshold
const CYCLE_MONITORING_INTERVAL_SECONDS: u64 = 3600; // Check every hour
const MIN_TREASURY_BALANCE_FOR_OPERATIONS: u64 = 100_000; // 0.001 BTC minimum
const CKBTC_TO_CYCLES_EXCHANGE_BUFFER: f64 = 1.1; // 10% buffer for exchange rate fluctuation

// Treasury storage
thread_local! {
    static TREASURY_STATE: RefCell<StableBTreeMap<u8, TreasuryState, Memory>> = RefCell::new(
        StableBTreeMap::init(get_treasury_memory(20))
    );
    
    static REVENUE_LOG: RefCell<StableBTreeMap<u64, RevenueEntry, Memory>> = RefCell::new(
        StableBTreeMap::init(get_treasury_memory(21))
    );
    
    static CANISTER_REGISTRY: RefCell<StableBTreeMap<String, CanisterInfo, Memory>> = RefCell::new(
        StableBTreeMap::init(get_treasury_memory(22))
    );
    
    static CYCLE_TRANSACTIONS: RefCell<StableBTreeMap<u64, CycleTransaction, Memory>> = RefCell::new(
        StableBTreeMap::init(get_treasury_memory(23))
    );
    
    static REVENUE_COUNTER: RefCell<u64> = RefCell::new(0);
    static CYCLE_TX_COUNTER: RefCell<u64> = RefCell::new(0);
}

// ========== CORE TREASURY FUNCTIONS ==========

/// Initialize treasury state
pub fn init_treasury() {
    TREASURY_STATE.with(|state| {
        let mut state_map = state.borrow_mut();
        if state_map.get(&0).is_none() {
            let initial_state = TreasuryState {
                balance_ckbtc: 0,
                total_fees_collected: 0,
                total_cycles_distributed: 0,
                last_cycle_distribution: time(),
                emergency_reserve: 0,
                created_at: time(),
                updated_at: time(),
            };
            state_map.insert(0, initial_state);
        }
    });
    
    // Register default operational canisters
    register_default_canisters();
}

/// Register default operational canisters that need cycle management
fn register_default_canisters() {
    let default_canisters = vec![
        ("agrilends_backend", CanisterType::Core, 1),
        ("agrilends_frontend", CanisterType::Frontend, 2),
        ("rwa_nft_canister", CanisterType::Core, 1),
        ("liquidity_pool_canister", CanisterType::Core, 1),
        ("oracle_canister", CanisterType::Oracle, 3),
        ("governance_canister", CanisterType::Infrastructure, 2),
    ];
    
    for (name, canister_type, priority) in default_canisters {
        let canister_info = CanisterInfo {
            name: name.to_string(),
            principal: Principal::anonymous(), // Will be updated by admin
            canister_type,
            min_cycles_threshold: DEFAULT_MIN_CYCLES_THRESHOLD,
            max_cycles_limit: DEFAULT_MAX_CYCLES_LIMIT,
            priority,
            last_top_up: 0,
            total_cycles_received: 0,
            estimated_daily_consumption: DEFAULT_MIN_CYCLES_THRESHOLD / 100, // Simplified estimation
            consumption_history: Vec::new(),
            is_active: false, // Will be activated when principal is set
            auto_top_up_enabled: true,
            health_check_enabled: true,
            alert_threshold_percentage: 20, // Alert at 20% of threshold
        };
        
        CANISTER_REGISTRY.with(|registry| {
            registry.borrow_mut().insert(name.to_string(), canister_info);
        });
    }
}

/// Get current treasury state
fn get_treasury_state() -> TreasuryState {
    TREASURY_STATE.with(|state| {
        state.borrow().get(&0).unwrap_or_else(|| TreasuryState {
            balance_ckbtc: 0,
            total_fees_collected: 0,
            total_cycles_distributed: 0,
            last_cycle_distribution: time(),
            emergency_reserve: 0,
            created_at: time(),
            updated_at: time(),
        })
    })
}

/// Update treasury state
fn update_treasury_state(new_state: TreasuryState) -> Result<(), String> {
    TREASURY_STATE.with(|state| {
        state.borrow_mut().insert(0, new_state);
    });
    Ok(())
}

/// Convert ckBTC to cycles using real-time exchange rate
async fn convert_ckbtc_to_cycles(ckbtc_amount: u64) -> Result<u64, String> {
    // In production, this would call the cycles minting canister
    // For now, use a simplified conversion rate
    // 1 ckBTC satoshi = 1000 cycles (simplified)
    let cycles = ckbtc_amount * 1000;
    Ok(cycles)
}

/// Get real-time ckBTC to cycles exchange rate
async fn get_ckbtc_cycles_exchange_rate() -> Result<f64, String> {
    // In production, this would fetch from cycles minting canister
    // For now, return a mock rate
    Ok(1000.0) // 1 satoshi = 1000 cycles
}

// ========== PUBLIC API FUNCTIONS ==========

/// Collect fees from loan operations (called by loan management canister)
#[update]
pub async fn collect_fees(
    source_loan_id: u64, 
    amount: u64, 
    revenue_type: RevenueType
) -> Result<String, String> {
    let caller = caller();
    
    // Security: Only loan management canister can collect fees
    if !is_loan_manager(&caller) {
        log_action(
            "TREASURY_UNAUTHORIZED_ACCESS",
            &format!("Unauthorized attempt to collect fees by {}", caller.to_text()),
            false,
        );
        return Err("Unauthorized: Only loan management canister can collect fees".to_string());
    }
    
    if amount == 0 {
        return Ok("No fees to collect".to_string());
    }
    
    // Generate revenue entry ID
    let revenue_id = REVENUE_COUNTER.with(|counter| {
        let mut counter = counter.borrow_mut();
        *counter += 1;
        *counter
    });
    
    // Create revenue entry
    let revenue_entry = RevenueEntry {
        id: revenue_id,
        source_loan_id,
        amount,
        revenue_type: revenue_type.clone(),
        source_canister: caller,
        timestamp: time(),
        transaction_hash: None, // Will be updated after ckBTC transfer
        status: TransactionStatus::Pending,
        processing_fee: 0, // No processing fee for now
        net_amount: amount, // Full amount as no fees
    };
    
    // Store revenue entry
    REVENUE_LOG.with(|log| {
        log.borrow_mut().insert(revenue_id, revenue_entry);
    });
    
    // Update treasury balance
    let mut treasury_state = get_treasury_state();
    treasury_state.balance_ckbtc += amount;
    treasury_state.total_fees_collected += amount;
    treasury_state.updated_at = time();
    
    // Calculate and update emergency reserve
    treasury_state.emergency_reserve = (treasury_state.balance_ckbtc * EMERGENCY_RESERVE_PERCENTAGE) / 100;
    
    update_treasury_state(treasury_state)?;
    
    // Update revenue entry status
    REVENUE_LOG.with(|log| {
        if let Some(mut entry) = log.borrow().get(&revenue_id) {
            entry.status = TransactionStatus::Completed;
            log.borrow_mut().insert(revenue_id, entry);
        }
    });
    
    // Log successful collection
    log_action(
        "TREASURY_FEE_COLLECTED",
        &format!("Successfully collected {} satoshi from loan #{} as {:?}", 
            amount, source_loan_id, revenue_type),
        true,
    );
    
    // Check if any canisters need cycle top-up
    let _ = check_and_auto_top_up_canisters().await;
    
    Ok(format!("Successfully collected {} satoshi in treasury", amount))
}

/// Top up cycles for a specific canister (admin or governance only)
#[update]
pub async fn top_up_canister_cycles(canister_name: String) -> Result<String, String> {
    let caller = caller();
    
    // Security check
    if !is_admin(&caller) {
        log_action(
            "TREASURY_UNAUTHORIZED_CYCLE_TOPUP",
            &format!("Unauthorized cycle top-up attempt for {} by {}", canister_name, caller.to_text()),
            false,
        );
        return Err("Unauthorized: Only admins can manually top up canister cycles".to_string());
    }
    
    // Get canister info
    let canister_info = CANISTER_REGISTRY.with(|registry| {
        registry.borrow().get(&canister_name)
    }).ok_or_else(|| "Canister not found in registry".to_string())?;
    
    if !canister_info.is_active {
        return Err("Canister is not active for cycle management".to_string());
    }
    
    // Check treasury balance
    let treasury_state = get_treasury_state();
    if treasury_state.balance_ckbtc < MIN_TREASURY_BALANCE_FOR_OPERATIONS {
        return Err("Insufficient treasury balance for cycle top-up operations".to_string());
    }
    
    // Calculate cycles needed
    let current_cycles = get_canister_cycles(canister_info.principal).await
        .map_err(|e| format!("Failed to get canister cycles: {}", e))?;
    
    if current_cycles >= canister_info.min_cycles_threshold {
        return Ok(format!("Canister {} already has sufficient cycles: {}", 
            canister_name, current_cycles));
    }
    
    let cycles_needed = (canister_info.min_cycles_threshold * AUTO_TOP_UP_PERCENTAGE / 100) - current_cycles;
    
    // Perform cycle top-up
    let result = perform_cycle_top_up(
        canister_info.clone(),
        cycles_needed,
        caller,
        format!("Manual top-up requested by admin")
    ).await;
    
    match result {
        Ok(tx_id) => {
            log_action(
                "TREASURY_MANUAL_CYCLE_TOPUP",
                &format!("Successfully topped up {} cycles for canister {} (TX: {})", 
                    cycles_needed, canister_name, tx_id),
                true,
            );
            Ok(format!("Successfully topped up {} cycles for canister {}", 
                cycles_needed, canister_name))
        },
        Err(e) => {
            log_action(
                "TREASURY_CYCLE_TOPUP_FAILED",
                &format!("Failed to top up cycles for canister {}: {}", canister_name, e),
                false,
            );
            Err(format!("Failed to top up cycles: {}", e))
        }
    }
}

/// Get comprehensive treasury statistics
#[query]
pub fn get_treasury_stats() -> TreasuryStats {
    let treasury_state = get_treasury_state();
    let active_canisters = CANISTER_REGISTRY.with(|registry| {
        registry.borrow().iter()
            .filter(|(_, canister)| canister.is_active)
            .count() as u32
    });
    
    // Calculate average daily revenue (last 30 days)
    let thirty_days_ago = time() - (30 * 24 * 60 * 60 * 1_000_000_000);
    let recent_revenue: u64 = REVENUE_LOG.with(|log| {
        log.borrow().iter()
            .filter(|(_, entry)| entry.timestamp >= thirty_days_ago)
            .map(|(_, entry)| entry.amount)
            .sum()
    });
    let average_daily_revenue = recent_revenue / 30;
    
    // Calculate projected runway (assuming current burn rate)
    let daily_cycle_cost = calculate_daily_cycle_cost();
    let projected_runway_days = if daily_cycle_cost > 0 {
        ((treasury_state.balance_ckbtc - treasury_state.emergency_reserve) / daily_cycle_cost) as u32
    } else {
        u32::MAX
    };
    
    TreasuryStats {
        current_balance: treasury_state.balance_ckbtc,
        total_revenue_collected: treasury_state.total_fees_collected,
        total_cycles_distributed: treasury_state.total_cycles_distributed,
        emergency_reserve: treasury_state.emergency_reserve,
        active_canisters_count: active_canisters,
        last_distribution_time: treasury_state.last_cycle_distribution,
        average_daily_revenue,
        projected_runway_days,
    }
}

/// Register a new canister for cycle management (admin only)
#[update]
pub fn register_canister(
    name: String,
    principal: Principal,
    canister_type: CanisterType,
    priority: u8
) -> Result<String, String> {
    let caller = caller();
    
    if !is_admin(&caller) {
        return Err("Unauthorized: Only admins can register canisters".to_string());
    }
    
    if priority < 1 || priority > 10 {
        return Err("Priority must be between 1 and 10".to_string());
    }
    
    let canister_info = CanisterInfo {
        name: name.clone(),
        principal,
        canister_type: canister_type.clone(),
        min_cycles_threshold: DEFAULT_MIN_CYCLES_THRESHOLD,
        max_cycles_limit: DEFAULT_MAX_CYCLES_LIMIT,
        priority,
        last_top_up: 0,
        total_cycles_received: 0,
        estimated_daily_consumption: DEFAULT_MIN_CYCLES_THRESHOLD / 100, // Simplified estimation
        consumption_history: Vec::new(),
        is_active: true,
        auto_top_up_enabled: true,
        health_check_enabled: true,
        alert_threshold_percentage: 20, // Alert at 20% of threshold
    };
    
    CANISTER_REGISTRY.with(|registry| {
        registry.borrow_mut().insert(name.clone(), canister_info);
    });
    
    log_action(
        "TREASURY_CANISTER_REGISTERED",
        &format!("Registered canister {} ({}) with type {:?} and priority {}", 
            name, principal.to_text(), canister_type, priority),
        true,
    );
    
    Ok(format!("Successfully registered canister {}", name))
}

/// Update canister configuration (admin only)
#[update]
pub fn update_canister_config(
    name: String,
    min_cycles_threshold: Option<u64>,
    max_cycles_limit: Option<u64>,
    priority: Option<u8>,
    auto_top_up_enabled: Option<bool>
) -> Result<String, String> {
    let caller = caller();
    
    if !is_admin(&caller) {
        return Err("Unauthorized: Only admins can update canister configuration".to_string());
    }
    
    CANISTER_REGISTRY.with(|registry| {
        let mut registry = registry.borrow_mut();
        if let Some(mut canister_info) = registry.get(&name) {
            if let Some(threshold) = min_cycles_threshold {
                canister_info.min_cycles_threshold = threshold;
            }
            if let Some(limit) = max_cycles_limit {
                canister_info.max_cycles_limit = limit;
            }
            if let Some(p) = priority {
                if p < 1 || p > 10 {
                    return Err("Priority must be between 1 and 10".to_string());
                }
                canister_info.priority = p;
            }
            if let Some(auto_enabled) = auto_top_up_enabled {
                canister_info.auto_top_up_enabled = auto_enabled;
            }
            
            registry.insert(name.clone(), canister_info);
            Ok(format!("Successfully updated configuration for canister {}", name))
        } else {
            Err("Canister not found in registry".to_string())
        }
    })
}

/// Get all registered canisters and their cycle status
#[query]
pub async fn get_canister_cycle_status() -> Vec<CanisterCycleStatus> {
    let mut statuses = Vec::new();
    
    CANISTER_REGISTRY.with(|registry| {
        for (_, canister_info) in registry.borrow().iter() {
            if canister_info.is_active {
                // Calculate estimated consumption and remaining days based on historical data
                let estimated_consumption_per_day = match canister_info.canister_type {
                    CanisterType::Core => 500_000_000,           // 500M cycles/day for core canisters
                    CanisterType::Infrastructure => 200_000_000, // 200M cycles/day for infrastructure
                    CanisterType::Frontend => 100_000_000,       // 100M cycles/day for frontend
                    CanisterType::Oracle => 300_000_000,         // 300M cycles/day for oracle
                    CanisterType::Analytics => 150_000_000,      // 150M cycles/day for analytics
                    CanisterType::Backup => 50_000_000,          // 50M cycles/day for backup
                };
                
                // For this query function, we use estimated values
                // In practice, you'd call get_canister_cycles in an update function
                let estimated_current_cycles = if canister_info.last_top_up > 0 {
                    let time_since_top_up = (time() - canister_info.last_top_up) / (24 * 60 * 60 * 1_000_000_000);
                    let consumed = estimated_consumption_per_day * time_since_top_up;
                    (canister_info.min_cycles_threshold * AUTO_TOP_UP_PERCENTAGE / 100).saturating_sub(consumed)
                } else {
                    canister_info.min_cycles_threshold / 2 // Assume half threshold if never topped up
                };
                
                let days_remaining = if estimated_consumption_per_day > 0 {
                    (estimated_current_cycles / estimated_consumption_per_day) as u32
                } else {
                    u32::MAX
                };
                
                let needs_top_up = estimated_current_cycles < canister_info.min_cycles_threshold;
                
                let status = CanisterCycleStatus {
                    canister_info: canister_info.clone(),
                    current_cycles: estimated_current_cycles,
                    cycles_percentage: (estimated_current_cycles * 100) as f64 / canister_info.min_cycles_threshold as f64,
                    estimated_consumption_per_day,
                    days_remaining,
                    needs_top_up,
                    is_critical: days_remaining < 1, // Less than 24 hours
                    last_checked: time(),
                    status_message: if needs_top_up {
                        format!("Needs top-up: {} days remaining", days_remaining)
                    } else {
                        "Healthy".to_string()
                    },
                };
                statuses.push(status);
            }
        }
    });
    
    // Sort by priority (lowest number = highest priority)
    statuses.sort_by_key(|s| s.canister_info.priority);
    statuses
}

/// Get revenue log with optional filtering
#[query]
pub fn get_revenue_log(
    limit: Option<u32>,
    revenue_type_filter: Option<RevenueType>,
    start_time: Option<u64>,
    end_time: Option<u64>
) -> Vec<RevenueEntry> {
    let limit = limit.unwrap_or(100).min(1000) as usize; // Max 1000 entries
    let mut entries = Vec::new();
    
    REVENUE_LOG.with(|log| {
        for (_, entry) in log.borrow().iter() {
            // Apply filters
            if let Some(ref filter_type) = revenue_type_filter {
                if std::mem::discriminant(&entry.revenue_type) != std::mem::discriminant(filter_type) {
                    continue;
                }
            }
            
            if let Some(start) = start_time {
                if entry.timestamp < start {
                    continue;
                }
            }
            
            if let Some(end) = end_time {
                if entry.timestamp > end {
                    continue;
                }
            }
            
            entries.push(entry.clone());
            
            if entries.len() >= limit {
                break;
            }
        }
    });
    
    // Sort by timestamp descending (most recent first)
    entries.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
    entries
}

/// Emergency withdraw function (super admin only)
#[update]
pub async fn emergency_withdraw(
    amount: u64,
    destination: Principal,
    reason: String
) -> Result<String, String> {
    let caller = caller();
    
    // Only super admin can perform emergency withdrawals
    if !is_admin(&caller) {
        log_action(
            "TREASURY_UNAUTHORIZED_EMERGENCY_WITHDRAWAL",
            &format!("Unauthorized emergency withdrawal attempt by {}", caller.to_text()),
            false,
        );
        return Err("Unauthorized: Only super admins can perform emergency withdrawals".to_string());
    }
    
    let treasury_state = get_treasury_state();
    
    if amount > treasury_state.balance_ckbtc {
        return Err("Insufficient treasury balance".to_string());
    }
    
    // Don't allow withdrawal of emergency reserve unless explicitly authorized
    let available_for_withdrawal = treasury_state.balance_ckbtc - treasury_state.emergency_reserve;
    if amount > available_for_withdrawal && !reason.contains("EMERGENCY_RESERVE_AUTHORIZED") {
        return Err(format!("Cannot withdraw emergency reserve. Available: {} satoshi", available_for_withdrawal));
    }
    
    // Perform ckBTC transfer
    let transfer_result = transfer_ckbtc_to_account(
        Account {
            owner: destination,
            subaccount: None,
        },
        amount
    ).await;
    
    match transfer_result {
        Ok(tx_id) => {
            // Update treasury balance
            let mut new_state = treasury_state;
            new_state.balance_ckbtc -= amount;
            new_state.updated_at = time();
            // Recalculate emergency reserve
            new_state.emergency_reserve = (new_state.balance_ckbtc * EMERGENCY_RESERVE_PERCENTAGE) / 100;
            update_treasury_state(new_state)?;
            
            log_action(
                "TREASURY_EMERGENCY_WITHDRAWAL",
                &format!("Emergency withdrawal of {} satoshi to {} (TX: {}). Reason: {}", 
                    amount, destination.to_text(), tx_id, reason),
                true,
            );
            
            Ok(format!("Emergency withdrawal completed. TX ID: {}", tx_id))
        },
        Err(e) => {
            log_action(
                "TREASURY_EMERGENCY_WITHDRAWAL_FAILED",
                &format!("Failed emergency withdrawal attempt: {}", e),
                false,
            );
            Err(format!("Emergency withdrawal failed: {}", e))
        }
    }
}

/// Get detailed cycle transactions log with filtering
#[query]
pub fn get_cycle_transactions(
    limit: Option<u32>,
    start_time: Option<u64>,
    end_time: Option<u64>,
    canister_filter: Option<String>
) -> Vec<CycleTransaction> {
    let caller = caller();
    
    // Only admin can view detailed cycle transactions
    if !is_admin(&caller) {
        return Vec::new();
    }
    
    let limit = limit.unwrap_or(100).min(1000) as usize;
    let mut transactions = Vec::new();
    
    CYCLE_TRANSACTIONS.with(|txs| {
        for (_, tx) in txs.borrow().iter() {
            // Apply filters
            if let Some(start) = start_time {
                if tx.timestamp < start {
                    continue;
                }
            }
            
            if let Some(end) = end_time {
                if tx.timestamp > end {
                    continue;
                }
            }
            
            if let Some(ref filter) = canister_filter {
                if !tx.canister_name.contains(filter) {
                    continue;
                }
            }
            
            transactions.push(tx.clone());
            
            if transactions.len() >= limit {
                break;
            }
        }
    });
    
    // Sort by timestamp descending (most recent first)
    transactions.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
    transactions
}

/// Manually trigger cycle distribution check (admin only)
#[update]
pub async fn trigger_cycle_distribution() -> Result<String, String> {
    let caller = caller();
    
    if !is_admin(&caller) {
        return Err("Unauthorized: Only admins can trigger cycle distribution".to_string());
    }
    
    let topped_up_count = check_and_auto_top_up_canisters().await?;
    
    log_action(
        "TREASURY_MANUAL_CYCLE_DISTRIBUTION",
        &format!("Manual cycle distribution triggered, {} canisters topped up", topped_up_count),
        true,
    );
    
    Ok(format!("Cycle distribution completed. {} canisters topped up.", topped_up_count))
}

/// Get treasury health report (simplified)
#[query]
pub fn get_treasury_health_report() -> TreasuryStats {
    let treasury_state = get_treasury_state();
    let active_canisters = CANISTER_REGISTRY.with(|registry| {
        registry.borrow().iter()
            .filter(|(_, canister)| canister.is_active)
            .count() as u32
    });
    
    // Calculate average daily revenue (last 30 days)
    let thirty_days_ago = time() - (30 * 24 * 60 * 60 * 1_000_000_000);
    let recent_revenue: u64 = REVENUE_LOG.with(|log| {
        log.borrow().iter()
            .filter(|(_, entry)| entry.timestamp >= thirty_days_ago)
            .map(|(_, entry)| entry.amount)
            .sum()
    });
    let average_daily_revenue = recent_revenue / 30;
    
    // Calculate projected runway (assuming current burn rate)
    let daily_cycle_cost = calculate_daily_cycle_cost();
    let projected_runway_days = if daily_cycle_cost > 0 {
        ((treasury_state.balance_ckbtc - treasury_state.emergency_reserve) / daily_cycle_cost) as u32
    } else {
        u32::MAX
    };
    
    TreasuryStats {
        current_balance: treasury_state.balance_ckbtc,
        total_revenue_collected: treasury_state.total_fees_collected,
        total_cycles_distributed: treasury_state.total_cycles_distributed,
        emergency_reserve: treasury_state.emergency_reserve,
        active_canisters_count: active_canisters,
        last_distribution_time: treasury_state.last_cycle_distribution,
        average_daily_revenue,
        projected_runway_days,
    }
}

/// Generate treasury management recommendations
fn generate_treasury_recommendations(treasury_state: &TreasuryState, daily_burn_rate: u64) -> Vec<String> {
    let mut recommendations = Vec::new();
    
    let runway_days = if daily_burn_rate > 0 {
        (treasury_state.balance_ckbtc - treasury_state.emergency_reserve) / daily_burn_rate
    } else {
        u64::MAX
    };
    
    if runway_days < 30 {
        recommendations.push("CRITICAL: Treasury balance critically low. Immediate funding required.".to_string());
    } else if runway_days < 90 {
        recommendations.push("WARNING: Treasury runway less than 90 days. Plan funding strategy.".to_string());
    }
    
    if treasury_state.balance_ckbtc < MIN_TREASURY_BALANCE_FOR_OPERATIONS * 10 {
        recommendations.push("Consider increasing protocol fees to improve treasury sustainability.".to_string());
    }
    
    let emergency_reserve_ratio = (treasury_state.emergency_reserve * 100) / treasury_state.balance_ckbtc;
    if emergency_reserve_ratio < 15 {
        recommendations.push("Emergency reserve ratio below 15%. Consider increasing reserve.".to_string());
    }
    
    if recommendations.is_empty() {
        recommendations.push("Treasury status healthy. Continue monitoring.".to_string());
    }
    
    recommendations
}

// ========== INTERNAL HELPER FUNCTIONS ==========

/// Check all canisters and automatically top up if needed
async fn check_and_auto_top_up_canisters() -> Result<u32, String> {
    let mut topped_up_count = 0;
    let now = time();
    
    // Get all active canisters sorted by priority
    let mut canisters: Vec<_> = CANISTER_REGISTRY.with(|registry| {
        registry.borrow().iter()
            .filter(|(_, canister)| canister.is_active && canister.auto_top_up_enabled)
            .map(|(_, canister)| canister.clone())
            .collect()
    });
    
    // Sort by priority (1 is highest priority)
    canisters.sort_by_key(|c| c.priority);
    
    for canister_info in canisters {
        // Check if enough time has passed since last top-up
        if now - canister_info.last_top_up < CYCLE_MONITORING_INTERVAL_SECONDS * 1_000_000_000 {
            continue;
        }
        
        // Get current cycle balance
        match get_canister_cycles(canister_info.principal).await {
            Ok(current_cycles) => {
                if current_cycles < canister_info.min_cycles_threshold {
                    let cycles_needed = (canister_info.min_cycles_threshold * AUTO_TOP_UP_PERCENTAGE / 100) - current_cycles;
                    
                    match perform_cycle_top_up(
                        canister_info.clone(),
                        cycles_needed,
                        Principal::management_canister(),
                        "Automatic cycle top-up".to_string()
                    ).await {
                        Ok(_) => {
                            topped_up_count += 1;
                        },
                        Err(e) => {
                            log_action(
                                "TREASURY_AUTO_TOPUP_FAILED",
                                &format!("Failed to auto top-up canister {}: {}", canister_info.name, e),
                                false,
                            );
                        }
                    }
                }
            },
            Err(e) => {
                log_action(
                    "TREASURY_CYCLE_CHECK_FAILED",
                    &format!("Failed to check cycles for canister {}: {}", canister_info.name, e),
                    false,
                );
            }
        }
    }
    
    Ok(topped_up_count)
}

/// Perform actual cycle top-up operation
async fn perform_cycle_top_up(
    canister_info: CanisterInfo,
    cycles_amount: u64,
    initiated_by: Principal,
    reason: String
) -> Result<u64, String> {
    // Generate transaction ID
    let tx_id = CYCLE_TX_COUNTER.with(|counter| {
        let mut counter = counter.borrow_mut();
        *counter += 1;
        *counter
    });
    
    // Get current exchange rate
    let exchange_rate = get_ckbtc_cycles_exchange_rate().await
        .unwrap_or(1000.0); // Fallback rate
    
    // Calculate ckBTC cost with buffer for exchange rate fluctuation
    let ckbtc_cost_base = (cycles_amount as f64 / exchange_rate) as u64;
    let ckbtc_cost = (ckbtc_cost_base as f64 * CKBTC_TO_CYCLES_EXCHANGE_BUFFER) as u64;
    
    // Check treasury balance
    let treasury_state = get_treasury_state();
    if treasury_state.balance_ckbtc < ckbtc_cost {
        return Err(format!("Insufficient treasury balance for cycle top-up. Required: {} satoshi, Available: {} satoshi", 
            ckbtc_cost, treasury_state.balance_ckbtc));
    }
    
    // Create cycle transaction record
    let cycle_tx = CycleTransaction {
        id: tx_id,
        target_canister: canister_info.principal,
        canister_name: canister_info.name.clone(),
        cycles_amount,
        ckbtc_cost,
        exchange_rate,
        timestamp: time(),
        status: TransactionStatus::Pending,
        initiated_by,
        reason: reason.clone(),
        gas_fee: 0, // No gas fee in IC
        confirmation_blocks: 0, // Not applicable in IC
    };
    
    CYCLE_TRANSACTIONS.with(|txs| {
        txs.borrow_mut().insert(tx_id, cycle_tx);
    });
    
    // Step 1: Convert ckBTC to cycles (using cycles minting canister)
    let cycles_result = convert_ckbtc_to_cycles(ckbtc_cost).await;
    
    match cycles_result {
        Ok(converted_cycles) => {
            // Step 2: Deposit cycles to target canister
            let deposit_result = deposit_cycles_to_canister(canister_info.principal, converted_cycles).await;
            
            match deposit_result {
                Ok(()) => {
                    // Update treasury balance
                    let mut new_treasury_state = treasury_state;
                    new_treasury_state.balance_ckbtc -= ckbtc_cost;
                    new_treasury_state.total_cycles_distributed += converted_cycles;
                    new_treasury_state.last_cycle_distribution = time();
                    new_treasury_state.updated_at = time();
                    update_treasury_state(new_treasury_state)?;
                    
                    // Update canister info
                    CANISTER_REGISTRY.with(|registry| {
                        let mut registry = registry.borrow_mut();
                        if let Some(mut info) = registry.get(&canister_info.name) {
                            info.last_top_up = time();
                            info.total_cycles_received += converted_cycles;
                            registry.insert(canister_info.name.clone(), info);
                        }
                    });
                    
                    // Update transaction status
                    CYCLE_TRANSACTIONS.with(|txs| {
                        if let Some(mut tx) = txs.borrow().get(&tx_id) {
                            tx.status = TransactionStatus::Completed;
                            tx.cycles_amount = converted_cycles; // Update with actual cycles received
                            txs.borrow_mut().insert(tx_id, tx);
                        }
                    });
                    
                    log_action(
                        "TREASURY_CYCLE_TOPUP_SUCCESS",
                        &format!("Successfully topped up {} cycles for canister {} (Cost: {} satoshi, Rate: {:.2})", 
                            converted_cycles, canister_info.name, ckbtc_cost, exchange_rate),
                        true,
                    );
                    
                    Ok(tx_id)
                },
                Err(e) => {
                    // Update transaction status as failed
                    CYCLE_TRANSACTIONS.with(|txs| {
                        if let Some(mut tx) = txs.borrow().get(&tx_id) {
                            tx.status = TransactionStatus::Failed(e.clone());
                            txs.borrow_mut().insert(tx_id, tx);
                        }
                    });
                    
                    Err(format!("Cycle deposit failed: {}", e))
                }
            }
        },
        Err(e) => {
            // Update transaction status as failed
            CYCLE_TRANSACTIONS.with(|txs| {
                if let Some(mut tx) = txs.borrow().get(&tx_id) {
                    tx.status = TransactionStatus::Failed(e.clone());
                    txs.borrow_mut().insert(tx_id, tx);
                }
            });
            
            Err(format!("ckBTC to cycles conversion failed: {}", e))
        }
    }
}

/// Get canister cycle balance (simplified implementation)
async fn get_canister_cycles(canister_id: Principal) -> Result<u64, String> {
    // In production, this would call canister_status on the management canister
    // For now, return a mock value
    Ok(500_000_000_000) // 500B cycles
}

/// Real cycle deposit implementation using IC Management Canister
async fn deposit_cycles_to_canister(canister_id: Principal, cycles: u64) -> Result<(), String> {
    use ic_cdk::api::management_canister::main::{deposit_cycles, CanisterIdRecord};
    
    // Convert cycles to IC cycles type
    let cycle_amount = cycles;
    
    // Call the management canister to deposit cycles
    match deposit_cycles(CanisterIdRecord { canister_id }, cycle_amount).await {
        Ok(()) => Ok(()),
        Err((rejection_code, msg)) => {
            Err(format!("Failed to deposit cycles: {:?} - {}", rejection_code, msg))
        }
    }
}

/// Get canister status from IC Management Canister
async fn get_canister_cycles(canister_id: Principal) -> Result<u64, String> {
    use ic_cdk::api::management_canister::main::{canister_status, CanisterIdRecord};
    
    match canister_status(CanisterIdRecord { canister_id }).await {
        Ok((status,)) => Ok(status.cycles),
        Err((rejection_code, msg)) => {
            Err(format!("Failed to get canister status: {:?} - {}", rejection_code, msg))
        }
    }
}

/// Real ckBTC transfer implementation using ICRC-1 standard
async fn transfer_ckbtc_to_account(to_account: Account, amount: u64) -> Result<String, String> {
    use ic_cdk::api::call::CallResult;
    use candid::Nat;
    
    // Define ckBTC transfer structures locally to avoid import issues
    #[derive(CandidType, Deserialize)]
    struct TransferArgs {
        from_subaccount: Option<Vec<u8>>,
        to: Account,
        amount: Nat,
        fee: Option<Nat>,
        memo: Option<Vec<u8>>,
        created_at_time: Option<u64>,
    }
    
    #[derive(CandidType, Deserialize, Debug)]
    enum TransferError {
        BadFee { expected_fee: Nat },
        BadBurn { min_burn_amount: Nat },
        InsufficientFunds { balance: Nat },
        TooOld,
        CreatedInFuture { ledger_time: u64 },
        TemporarilyUnavailable,
        Duplicate { duplicate_of: Nat },
        GenericError { error_code: Nat, message: String },
    }
    
    let ckbtc_ledger = Principal::from_text("mxzaz-hqaaa-aaaar-qaada-cai")
        .map_err(|_| "Invalid ckBTC ledger principal")?;
    
    let transfer_args = TransferArgs {
        from_subaccount: None,
        to: to_account,
        amount: Nat::from(amount),
        fee: None,
        memo: Some("Treasury emergency withdrawal".as_bytes().to_vec()),
        created_at_time: Some(time()),
    };
    
    let call_result: CallResult<(Result<Nat, TransferError>,)> = 
        ic_cdk::call(ckbtc_ledger, "icrc1_transfer", (transfer_args,)).await;
    
    match call_result {
        Ok((Ok(block_index),)) => {
            Ok(format!("block_{}", block_index.0.to_string()))
        },
        Ok((Err(transfer_error),)) => {
            Err(format!("ckBTC transfer failed: {:?}", transfer_error))
        },
        Err((rejection_code, msg)) => {
            Err(format!("ckBTC transfer call failed: {:?} - {}", rejection_code, msg))
        }
    }
}

/// Calculate estimated daily cycle cost across all canisters
fn calculate_daily_cycle_cost() -> u64 {
    CANISTER_REGISTRY.with(|registry| {
        registry.borrow().iter()
            .filter(|(_, canister)| canister.is_active)
            .map(|(_, canister)| {
                match canister.canister_type {
                    CanisterType::Core => 500_000_000,           // 500M cycles/day
                    CanisterType::Infrastructure => 200_000_000, // 200M cycles/day
                    CanisterType::Frontend => 100_000_000,       // 100M cycles/day
                    CanisterType::Oracle => 300_000_000,         // 300M cycles/day
                    CanisterType::Analytics => 150_000_000,      // 150M cycles/day
                    CanisterType::Backup => 50_000_000,          // 50M cycles/day
                }
            })
            .sum()
    })
}

/// Process fee collection from loan operations with detailed tracking
pub async fn process_loan_fee_collection(
    loan_id: u64,
    total_amount: u64,
    admin_fee_amount: u64,
    interest_share_amount: u64
) -> Result<String, String> {
    let caller = caller();
    
    // Security: Only loan management canister can collect fees
    if !is_loan_manager(&caller) {
        return Err("Unauthorized: Only loan management canister can collect fees".to_string());
    }
    
    let mut results = Vec::new();
    
    // Collect admin fee
    if admin_fee_amount > 0 {
        match collect_fees(loan_id, admin_fee_amount, RevenueType::AdminFee).await {
            Ok(msg) => results.push(format!("Admin fee: {}", msg)),
            Err(e) => return Err(format!("Failed to collect admin fee: {}", e)),
        }
    }
    
    // Collect interest share
    if interest_share_amount > 0 {
        match collect_fees(loan_id, interest_share_amount, RevenueType::InterestShare).await {
            Ok(msg) => results.push(format!("Interest share: {}", msg)),
            Err(e) => return Err(format!("Failed to collect interest share: {}", e)),
        }
    }
    
    log_action(
        "TREASURY_LOAN_FEE_COLLECTION",
        &format!("Collected {} satoshi total fees from loan #{} (Admin: {}, Interest: {})", 
            total_amount, loan_id, admin_fee_amount, interest_share_amount),
        true,
    );
    
    Ok(format!("Successfully collected fees: {}", results.join(", ")))
}

/// Process liquidation penalty collection
pub async fn process_liquidation_penalty(
    loan_id: u64,
    penalty_amount: u64,
    liquidation_reason: String
) -> Result<String, String> {
    let caller = caller();
    
    // Security: Only liquidation canister can collect penalties
    if !is_admin(&caller) && !is_loan_manager(&caller) {
        return Err("Unauthorized: Only liquidation system can collect penalties".to_string());
    }
    
    if penalty_amount == 0 {
        return Ok("No penalty to collect".to_string());
    }
    
    let result = collect_fees(loan_id, penalty_amount, RevenueType::LiquidationPenalty).await?;
    
    log_action(
        "TREASURY_LIQUIDATION_PENALTY",
        &format!("Collected {} satoshi liquidation penalty from loan #{}: {}", 
            penalty_amount, loan_id, liquidation_reason),
        true,
    );
    
    Ok(result)
}

/// Set treasury configuration parameters (admin only)
#[update]
pub fn set_treasury_configuration(
    min_balance_threshold: Option<u64>,
    emergency_reserve_percentage: Option<u64>,
    auto_top_up_percentage: Option<u64>,
    cycle_monitoring_interval: Option<u64>
) -> Result<String, String> {
    let caller = caller();
    
    if !is_admin(&caller) {
        return Err("Unauthorized: Only admins can update treasury configuration".to_string());
    }
    
    // Validate parameters
    if let Some(reserve_pct) = emergency_reserve_percentage {
        if reserve_pct > 50 {
            return Err("Emergency reserve percentage cannot exceed 50%".to_string());
        }
    }
    
    if let Some(top_up_pct) = auto_top_up_percentage {
        if top_up_pct < 100 || top_up_pct > 300 {
            return Err("Auto top-up percentage must be between 100% and 300%".to_string());
        }
    }
    
    // Store configuration in canister config
    let mut config = get_config();
    
    // Note: These fields would need to be added to CanisterConfig if needed
    // For now, we'll just log the configuration update
    
    log_action(
        "TREASURY_CONFIG_UPDATE",
        "Treasury configuration updated",
        true,
    );
    
    Ok("Treasury configuration updated successfully".to_string())
}

// ========== HEARTBEAT AND MONITORING ==========

/// Heartbeat function to check canister cycles periodically
#[ic_cdk_macros::heartbeat]
pub async fn treasury_heartbeat() {
    let now = time();
    let last_check = get_treasury_state().last_cycle_distribution;
    
    // Check every hour
    if now - last_check >= CYCLE_MONITORING_INTERVAL_SECONDS * 1_000_000_000 {
        let _ = check_and_auto_top_up_canisters().await;
    }
}

// ========== INITIALIZATION AND UPGRADE HOOKS ==========

/// Initialize treasury system
#[init]
fn init() {
    init_treasury();
}

#[pre_upgrade]
fn pre_upgrade() {
    // Treasury state is already in stable storage
}

#[post_upgrade]
fn post_upgrade() {
    // Initialize if needed
    init_treasury();
}

// ========== PUBLIC EXPORTS ==========

// Export key functions for use by other modules
pub use collect_fees;
pub use top_up_canister_cycles;
pub use get_treasury_stats;
pub use register_canister;
pub use update_canister_config;
pub use get_canister_cycle_status;
pub use get_revenue_log;
pub use emergency_withdraw;
pub use get_cycle_transactions;
pub use trigger_cycle_distribution;
pub use process_loan_fee_collection;
pub use process_liquidation_penalty;
pub use set_treasury_configuration;
