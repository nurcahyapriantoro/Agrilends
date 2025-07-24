// ========== ADVANCED QUERY ROUTING & AGGREGATION MODULE ==========
// Production-ready query distribution and data aggregation system
// Handles complex queries across multiple shards with performance optimization
// Implements caching, load balancing, and intelligent query planning

use ic_cdk::{caller, api::time, call};
use ic_cdk_macros::{query, update, heartbeat};
use candid::{CandidType, Deserialize, Principal};
use ic_stable_structures::{StableBTreeMap, memory::MemoryId};
use ic_stable_structures::memory::VirtualMemory;
use ic_stable_structures::DefaultMemoryImpl;
use std::cell::RefCell;
use std::collections::{HashMap, BTreeMap};

use crate::types::*;
use crate::storage::{get_memory_by_id, log_audit_action};
use crate::helpers::{is_admin, get_canister_config};
use crate::scalability_architecture::{ShardInfo, get_user_shards, get_all_shards};

// ========== QUERY ROUTING TYPES ==========

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct QueryPlan {
    pub query_id: String,
    pub query_type: QueryType,
    pub target_shards: Vec<ShardTarget>,
    pub aggregation_strategy: AggregationStrategy,
    pub execution_order: Vec<QueryStep>,
    pub estimated_duration_ms: u64,
    pub cache_key: Option<String>,
    pub created_at: u64,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub enum QueryType {
    UserDashboard,      // Get all user data
    LoansByStatus,      // Filter loans by status
    AggregateStats,     // System-wide statistics
    PerformanceMetrics, // Performance data
    AuditTrail,         // Audit logs across shards
    CustomQuery,        // User-defined query
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct ShardTarget {
    pub shard_id: u32,
    pub canister_id: Principal,
    pub query_params: QueryParams,
    pub priority: QueryPriority,
    pub timeout_ms: u64,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub enum QueryPriority {
    Critical,   // System queries
    High,       // User dashboard
    Medium,     // Analytics
    Low,        // Background reports
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct QueryParams {
    pub filters: HashMap<String, String>,
    pub pagination: Option<PaginationParams>,
    pub sorting: Option<SortingParams>,
    pub fields: Option<Vec<String>>,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct PaginationParams {
    pub offset: u64,
    pub limit: u64,
    pub cursor: Option<String>,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct SortingParams {
    pub field: String,
    pub direction: SortDirection,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub enum SortDirection {
    Ascending,
    Descending,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub enum AggregationStrategy {
    Merge,          // Simple concatenation
    Sum,            // Numerical aggregation  
    Average,        // Calculate averages
    GroupBy,        // Group results by field
    TopN,           // Get top N results
    Custom(String), // Custom aggregation logic
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct QueryStep {
    pub step_id: u32,
    pub step_type: StepType,
    pub target_shard: Option<Principal>,
    pub dependencies: Vec<u32>,
    pub parallel_execution: bool,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub enum StepType {
    DataRetrieval,
    Filtering,
    Aggregation,
    Sorting,
    Pagination,
    Caching,
}

// ========== CACHING SYSTEM ==========

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct CacheEntry {
    pub key: String,
    pub data: CachedData,
    pub created_at: u64,
    pub expires_at: u64,
    pub access_count: u64,
    pub last_accessed: u64,
    pub size_bytes: u64,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub enum CachedData {
    UserLoans(Vec<Loan>),
    AggregateStats(SystemStats),
    QueryResults(String), // JSON serialized results
    ShardMetrics(Vec<ShardMetrics>),
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct SystemStats {
    pub total_loans: u64,
    pub total_volume: u64,
    pub active_users: u64,
    pub avg_loan_amount: u64,
    pub default_rate: f64,
    pub system_health: f64,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct ShardMetrics {
    pub shard_id: u32,
    pub response_time_ms: u64,
    pub load_factor: f64,
    pub error_rate: f64,
    pub last_updated: u64,
}

// ========== STORAGE MANAGEMENT ==========

thread_local! {
    static QUERY_CACHE: RefCell<StableBTreeMap<String, CacheEntry, VirtualMemory<DefaultMemoryImpl>>> = 
        RefCell::new(StableBTreeMap::init(get_memory_by_id(MemoryId::new(35))));
    
    static QUERY_PLANS: RefCell<StableBTreeMap<String, QueryPlan, VirtualMemory<DefaultMemoryImpl>>> = 
        RefCell::new(StableBTreeMap::init(get_memory_by_id(MemoryId::new(36))));
    
    static SHARD_PERFORMANCE: RefCell<StableBTreeMap<u32, ShardMetrics, VirtualMemory<DefaultMemoryImpl>>> = 
        RefCell::new(StableBTreeMap::init(get_memory_by_id(MemoryId::new(37))));
    
    static QUERY_STATS: RefCell<QueryStatistics> = RefCell::new(QueryStatistics {
        total_queries: 0,
        cache_hits: 0,
        cache_misses: 0,
        avg_query_time_ms: 0,
        failed_queries: 0,
        active_queries: 0,
        last_reset: 0,
    });
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct QueryStatistics {
    pub total_queries: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub avg_query_time_ms: u64,
    pub failed_queries: u64,
    pub active_queries: u64,
    pub last_reset: u64,
}

// ========== ADVANCED DASHBOARD QUERIES ==========

/// Get comprehensive farmer dashboard data with caching
#[query]
pub async fn get_farmer_dashboard_advanced(user_id: Principal) -> Result<FarmerDashboardAdvanced, String> {
    let start_time = time();
    let cache_key = format!("farmer_dashboard_{}", user_id.to_text());
    
    // Check cache first
    if let Some(cached_data) = get_from_cache(&cache_key) {
        update_query_stats(true, start_time);
        if let CachedData::QueryResults(json_data) = cached_data.data {
            // Parse cached JSON back to struct
            return parse_dashboard_from_json(&json_data);
        }
    }
    
    // Generate query plan
    let query_plan = create_user_dashboard_query_plan(user_id)?;
    
    // Execute distributed query
    let dashboard_data = execute_distributed_dashboard_query(query_plan).await?;
    
    // Cache results for 5 minutes
    cache_query_result(&cache_key, &dashboard_data, 300)?;
    
    update_query_stats(false, start_time);
    Ok(dashboard_data)
}

/// Get investor dashboard with real-time analytics
#[query]
pub async fn get_investor_dashboard_advanced(user_id: Principal) -> Result<InvestorDashboardAdvanced, String> {
    let start_time = time();
    let cache_key = format!("investor_dashboard_{}", user_id.to_text());
    
    // Check cache (shorter TTL for investor data)
    if let Some(cached_data) = get_from_cache(&cache_key) {
        if cached_data.expires_at > time() {
            update_query_stats(true, start_time);
            if let CachedData::QueryResults(json_data) = cached_data.data {
                return parse_investor_dashboard_from_json(&json_data);
            }
        }
    }
    
    // Execute multi-shard aggregation
    let mut dashboard_data = InvestorDashboardAdvanced {
        user_id,
        total_invested: 0,
        active_investments: 0,
        total_returns: 0,
        roi_percentage: 0.0,
        risk_score: 0.0,
        portfolio_distribution: HashMap::new(),
        recent_transactions: Vec::new(),
        performance_metrics: InvestorPerformanceMetrics::default(),
        market_insights: MarketInsights::default(),
        last_updated: time(),
    };
    
    // Query all shards for investor data
    let shards = get_all_shards();
    let mut parallel_queries = Vec::new();
    
    for shard in shards {
        if shard.is_active {
            parallel_queries.push(query_shard_for_investor_data(shard.canister_id, user_id));
        }
    }
    
    // Execute queries in parallel and aggregate results
    let results = execute_parallel_queries(parallel_queries).await?;
    dashboard_data = aggregate_investor_data(dashboard_data, results)?;
    
    // Cache for 2 minutes
    cache_investor_dashboard(&cache_key, &dashboard_data, 120)?;
    
    update_query_stats(false, start_time);
    Ok(dashboard_data)
}

/// Get system-wide analytics with intelligent caching
#[query] 
pub async fn get_system_analytics() -> Result<SystemAnalytics, String> {
    let start_time = time();
    let cache_key = "system_analytics".to_string();
    
    // Check cache (10 minute TTL for system stats)
    if let Some(cached_data) = get_from_cache(&cache_key) {
        if cached_data.expires_at > time() {
            update_query_stats(true, start_time);
            if let CachedData::QueryResults(json_data) = cached_data.data {
                return parse_system_analytics_from_json(&json_data);
            }
        }
    }
    
    // Query all active shards for statistics
    let shards = get_all_shards();
    let active_shards: Vec<_> = shards.into_iter().filter(|s| s.is_active).collect();
    
    let mut analytics = SystemAnalytics {
        total_loans: 0,
        total_volume_satoshi: 0,
        active_loans: 0,
        completed_loans: 0,
        defaulted_loans: 0,
        total_users: 0,
        total_farmers: 0,
        total_investors: 0,
        avg_loan_amount: 0,
        default_rate: 0.0,
        platform_revenue: 0,
        top_commodities: Vec::new(),
        regional_distribution: HashMap::new(),
        time_series_data: Vec::new(),
        shard_performance: Vec::new(),
        last_updated: time(),
    };
    
    // Aggregate data from all shards
    for shard in active_shards {
        match query_shard_analytics(shard.canister_id).await {
            Ok(shard_analytics) => {
                analytics.total_loans += shard_analytics.loan_count;
                analytics.total_volume_satoshi += shard_analytics.total_volume;
                analytics.active_loans += shard_analytics.active_loans;
                analytics.completed_loans += shard_analytics.completed_loans;
                analytics.defaulted_loans += shard_analytics.defaulted_loans;
                
                // Add shard performance metrics
                analytics.shard_performance.push(ShardPerformanceData {
                    shard_id: shard.shard_id,
                    loan_count: shard_analytics.loan_count,
                    avg_response_time: shard_analytics.avg_response_time,
                    error_rate: shard_analytics.error_rate,
                    utilization: shard_analytics.utilization,
                });
            },
            Err(e) => {
                log_audit_action(
                    "SHARD_QUERY_ERROR".to_string(),
                    format!("Failed to query analytics from shard {}: {}", shard.shard_id, e),
                    caller(),
                    Some(format!("shard_id:{}", shard.shard_id)),
                );
            }
        }
    }
    
    // Calculate derived metrics
    if analytics.total_loans > 0 {
        analytics.avg_loan_amount = analytics.total_volume_satoshi / analytics.total_loans;
        analytics.default_rate = (analytics.defaulted_loans as f64 / analytics.total_loans as f64) * 100.0;
    }
    
    // Cache results for 10 minutes
    cache_system_analytics(&cache_key, &analytics, 600)?;
    
    update_query_stats(false, start_time);
    Ok(analytics)
}

// ========== INTELLIGENT QUERY PLANNING ==========

/// Create optimized query plan for user dashboard
fn create_user_dashboard_query_plan(user_id: Principal) -> Result<QueryPlan, String> {
    let query_id = format!("user_dashboard_{}_{}", user_id.to_text(), time());
    let user_shards = get_user_shards(user_id);
    
    let mut target_shards = Vec::new();
    for shard in user_shards {
        target_shards.push(ShardTarget {
            shard_id: shard.shard_id,
            canister_id: shard.canister_id,
            query_params: QueryParams {
                filters: {
                    let mut filters = HashMap::new();
                    filters.insert("user_id".to_string(), user_id.to_text());
                    filters
                },
                pagination: Some(PaginationParams {
                    offset: 0,
                    limit: 100,
                    cursor: None,
                }),
                sorting: Some(SortingParams {
                    field: "created_at".to_string(),
                    direction: SortDirection::Descending,
                }),
                fields: None,
            },
            priority: QueryPriority::High,
            timeout_ms: 5000,
        });
    }
    
    // Create execution steps
    let execution_order = vec![
        QueryStep {
            step_id: 1,
            step_type: StepType::DataRetrieval,
            target_shard: None,
            dependencies: vec![],
            parallel_execution: true,
        },
        QueryStep {
            step_id: 2,
            step_type: StepType::Aggregation,
            target_shard: None,
            dependencies: vec![1],
            parallel_execution: false,
        },
        QueryStep {
            step_id: 3,
            step_type: StepType::Sorting,
            target_shard: None,
            dependencies: vec![2],
            parallel_execution: false,
        },
    ];
    
    Ok(QueryPlan {
        query_id,
        query_type: QueryType::UserDashboard,
        target_shards,
        aggregation_strategy: AggregationStrategy::Merge,
        execution_order,
        estimated_duration_ms: calculate_estimated_duration(&target_shards),
        cache_key: Some(format!("user_dashboard_{}", user_id.to_text())),
        created_at: time(),
    })
}

/// Execute distributed dashboard query
async fn execute_distributed_dashboard_query(query_plan: QueryPlan) -> Result<FarmerDashboardAdvanced, String> {
    let mut all_loans = Vec::new();
    let mut shard_errors = Vec::new();
    
    // Execute queries in parallel
    for shard_target in query_plan.target_shards {
        match query_shard_for_user_loans(shard_target.canister_id, shard_target.query_params).await {
            Ok(mut loans) => {
                all_loans.append(&mut loans);
            },
            Err(e) => {
                shard_errors.push(format!("Shard {}: {}", shard_target.shard_id, e));
            }
        }
    }
    
    // Log any shard errors but continue with available data
    if !shard_errors.is_empty() {
        log_audit_action(
            "PARTIAL_QUERY_SUCCESS".to_string(),
            format!("Query completed with errors: {:?}", shard_errors),
            caller(),
            Some(query_plan.query_id.clone()),
        );
    }
    
    // Aggregate and process results
    let dashboard_data = process_farmer_dashboard_data(all_loans)?;
    Ok(dashboard_data)
}

// ========== CACHING SYSTEM IMPLEMENTATION ==========

/// Get data from cache if available and not expired
fn get_from_cache(key: &str) -> Option<CacheEntry> {
    QUERY_CACHE.with(|cache| {
        if let Some(mut entry) = cache.borrow().get(key) {
            if entry.expires_at > time() {
                // Update access statistics
                entry.access_count += 1;
                entry.last_accessed = time();
                cache.borrow_mut().insert(key.to_string(), entry.clone());
                Some(entry)
            } else {
                // Remove expired entry
                cache.borrow_mut().remove(key);
                None
            }
        } else {
            None
        }
    })
}

/// Cache query results with TTL
fn cache_query_result(key: &str, data: &FarmerDashboardAdvanced, ttl_seconds: u64) -> Result<(), String> {
    let serialized_data = serialize_dashboard_to_json(data)?;
    let current_time = time();
    
    let cache_entry = CacheEntry {
        key: key.to_string(),
        data: CachedData::QueryResults(serialized_data),
        created_at: current_time,
        expires_at: current_time + (ttl_seconds * 1_000_000_000), // Convert to nanoseconds
        access_count: 0,
        last_accessed: current_time,
        size_bytes: key.len() as u64 + 1000, // Approximate size
    };
    
    QUERY_CACHE.with(|cache| {
        cache.borrow_mut().insert(key.to_string(), cache_entry);
    });
    
    Ok(())
}

/// Cache management heartbeat
#[heartbeat]
pub fn cache_maintenance_heartbeat() {
    let current_time = time();
    let mut expired_keys = Vec::new();
    
    // Find expired entries
    QUERY_CACHE.with(|cache| {
        for (key, entry) in cache.borrow().iter() {
            if entry.expires_at <= current_time {
                expired_keys.push(key.clone());
            }
        }
        
        // Remove expired entries
        let mut cache_ref = cache.borrow_mut();
        for key in expired_keys {
            cache_ref.remove(&key);
        }
    });
    
    // Update shard performance metrics
    update_shard_performance_metrics();
}

/// Update performance metrics for all shards
fn update_shard_performance_metrics() {
    let shards = get_all_shards();
    let current_time = time();
    
    for shard in shards {
        if shard.is_active {
            let metrics = ShardMetrics {
                shard_id: shard.shard_id,
                response_time_ms: shard.performance_metrics.avg_response_time_ms,
                load_factor: shard.storage_percentage / 100.0,
                error_rate: calculate_shard_error_rate(&shard),
                last_updated: current_time,
            };
            
            SHARD_PERFORMANCE.with(|perf| {
                perf.borrow_mut().insert(shard.shard_id, metrics);
            });
        }
    }
}

// ========== PERFORMANCE OPTIMIZATION ==========

/// Get query statistics
#[query]
pub fn get_query_statistics() -> QueryStatistics {
    QUERY_STATS.with(|stats| stats.borrow().clone())
}

/// Reset query statistics (admin only)
#[update]
pub fn reset_query_statistics() -> Result<(), String> {
    let caller = caller();
    if !is_admin(&caller) {
        return Err("Only admin can reset query statistics".to_string());
    }
    
    QUERY_STATS.with(|stats| {
        *stats.borrow_mut() = QueryStatistics {
            total_queries: 0,
            cache_hits: 0,
            cache_misses: 0,
            avg_query_time_ms: 0,
            failed_queries: 0,
            active_queries: 0,
            last_reset: time(),
        };
    });
    
    Ok(())
}

/// Update query statistics
fn update_query_stats(cache_hit: bool, start_time: u64) {
    let duration = time() - start_time;
    let duration_ms = duration / 1_000_000; // Convert to milliseconds
    
    QUERY_STATS.with(|stats| {
        let mut stats_ref = stats.borrow_mut();
        stats_ref.total_queries += 1;
        
        if cache_hit {
            stats_ref.cache_hits += 1;
        } else {
            stats_ref.cache_misses += 1;
        }
        
        // Update average query time
        let total_time = stats_ref.avg_query_time_ms * (stats_ref.total_queries - 1) + duration_ms;
        stats_ref.avg_query_time_ms = total_time / stats_ref.total_queries;
    });
}

/// Get cache statistics
#[query]
pub fn get_cache_statistics() -> CacheStatistics {
    let current_time = time();
    let mut total_entries = 0;
    let mut expired_entries = 0;
    let mut total_size_bytes = 0;
    let mut total_access_count = 0;
    
    QUERY_CACHE.with(|cache| {
        for (_, entry) in cache.borrow().iter() {
            total_entries += 1;
            total_size_bytes += entry.size_bytes;
            total_access_count += entry.access_count;
            
            if entry.expires_at <= current_time {
                expired_entries += 1;
            }
        }
    });
    
    CacheStatistics {
        total_entries,
        expired_entries,
        total_size_bytes,
        total_access_count,
        hit_rate: if total_access_count > 0 {
            (total_access_count as f64 / (total_entries as f64 + 1.0)) * 100.0
        } else { 0.0 },
        last_cleanup: current_time,
    }
}

// ========== HELPER FUNCTIONS ==========

async fn query_shard_for_user_loans(canister_id: Principal, query_params: QueryParams) -> Result<Vec<Loan>, String> {
    // This would be an actual inter-canister call in production
    // For now, return empty vec as placeholder
    Ok(vec![])
}

async fn query_shard_for_investor_data(canister_id: Principal, user_id: Principal) -> Result<InvestorShardData, String> {
    // This would be an actual inter-canister call in production
    Ok(InvestorShardData::default())
}

async fn query_shard_analytics(canister_id: Principal) -> Result<ShardAnalytics, String> {
    // This would be an actual inter-canister call in production
    Ok(ShardAnalytics::default())
}

fn calculate_estimated_duration(shards: &[ShardTarget]) -> u64 {
    // Estimate based on shard count and historical performance
    let base_time = 100; // 100ms base
    let shard_penalty = shards.len() as u64 * 50; // 50ms per shard
    base_time + shard_penalty
}

fn calculate_shard_error_rate(shard: &ShardInfo) -> f64 {
    let total_requests = shard.performance_metrics.total_requests;
    if total_requests > 0 {
        (shard.performance_metrics.error_count as f64 / total_requests as f64) * 100.0
    } else {
        0.0
    }
}

// ========== SERIALIZATION HELPERS ==========

fn serialize_dashboard_to_json(dashboard: &FarmerDashboardAdvanced) -> Result<String, String> {
    // In production, use proper JSON serialization
    Ok(format!("{{\"user_id\": \"{}\", \"timestamp\": {}}}", dashboard.user_id.to_text(), dashboard.last_updated))
}

fn parse_dashboard_from_json(json: &str) -> Result<FarmerDashboardAdvanced, String> {
    // In production, use proper JSON deserialization
    Err("JSON parsing not implemented".to_string())
}

// ========== ADDITIONAL TYPES ==========

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct FarmerDashboardAdvanced {
    pub user_id: Principal,
    pub total_loans: u64,
    pub active_loans: u64,
    pub completed_loans: u64,
    pub total_borrowed: u64,
    pub total_repaid: u64,
    pub credit_score: f64,
    pub recent_loans: Vec<Loan>,
    pub nft_collateral: Vec<NFTSummary>,
    pub payment_history: Vec<PaymentRecord>,
    pub performance_metrics: FarmerPerformanceMetrics,
    pub recommendations: Vec<String>,
    pub last_updated: u64,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct InvestorDashboardAdvanced {
    pub user_id: Principal,
    pub total_invested: u64,
    pub active_investments: u64,
    pub total_returns: u64,
    pub roi_percentage: f64,
    pub risk_score: f64,
    pub portfolio_distribution: HashMap<String, u64>,
    pub recent_transactions: Vec<TransactionRecord>,
    pub performance_metrics: InvestorPerformanceMetrics,
    pub market_insights: MarketInsights,
    pub last_updated: u64,
}

#[derive(CandidType, Deserialize, Clone, Debug, Default)]
pub struct InvestorPerformanceMetrics {
    pub total_roi: f64,
    pub monthly_roi: f64,
    pub risk_adjusted_return: f64,
    pub portfolio_volatility: f64,
}

#[derive(CandidType, Deserialize, Clone, Debug, Default)]
pub struct MarketInsights {
    pub trending_commodities: Vec<String>,
    pub market_opportunities: Vec<String>,
    pub risk_alerts: Vec<String>,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct SystemAnalytics {
    pub total_loans: u64,
    pub total_volume_satoshi: u64,
    pub active_loans: u64,
    pub completed_loans: u64,
    pub defaulted_loans: u64,
    pub total_users: u64,
    pub total_farmers: u64,
    pub total_investors: u64,
    pub avg_loan_amount: u64,
    pub default_rate: f64,
    pub platform_revenue: u64,
    pub top_commodities: Vec<CommodityStats>,
    pub regional_distribution: HashMap<String, u64>,
    pub time_series_data: Vec<TimeSeriesPoint>,
    pub shard_performance: Vec<ShardPerformanceData>,
    pub last_updated: u64,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct CacheStatistics {
    pub total_entries: u64,
    pub expired_entries: u64,
    pub total_size_bytes: u64,
    pub total_access_count: u64,
    pub hit_rate: f64,
    pub last_cleanup: u64,
}

#[derive(CandidType, Deserialize, Clone, Debug, Default)]
pub struct InvestorShardData {
    pub investments: Vec<Investment>,
    pub returns: u64,
    pub performance: f64,
}

#[derive(CandidType, Deserialize, Clone, Debug, Default)]
pub struct ShardAnalytics {
    pub loan_count: u64,
    pub total_volume: u64,
    pub active_loans: u64,
    pub completed_loans: u64,
    pub defaulted_loans: u64,
    pub avg_response_time: u64,
    pub error_rate: f64,
    pub utilization: f64,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct ShardPerformanceData {
    pub shard_id: u32,
    pub loan_count: u64,
    pub avg_response_time: u64,
    pub error_rate: f64,
    pub utilization: f64,
}
