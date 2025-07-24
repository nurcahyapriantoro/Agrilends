// ========== LOAD BALANCING & CIRCUIT BREAKER MODULE ==========
// Production-grade load balancing and fault tolerance system
// Implements circuit breakers, health checks, and intelligent traffic distribution
// Ensures system reliability and optimal performance under high load

use ic_cdk::{caller, api::time, call};
use ic_cdk_macros::{query, update, heartbeat};
use candid::{CandidType, Deserialize, Principal};
use ic_stable_structures::{StableBTreeMap, memory::MemoryId};
use ic_stable_structures::memory::VirtualMemory;
use ic_stable_structures::DefaultMemoryImpl;
use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};

use crate::types::*;
use crate::storage::{get_memory_by_id, log_audit_action};
use crate::helpers::{is_admin, get_canister_config};
use crate::scalability_architecture::ShardInfo;

// ========== LOAD BALANCING TYPES ==========

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct LoadBalancer {
    pub balancer_id: String,
    pub algorithm: LoadBalancingAlgorithm,
    pub active_shards: Vec<ShardEndpoint>,
    pub health_check_config: HealthCheckConfig,
    pub circuit_breaker_config: CircuitBreakerConfig,
    pub traffic_distribution: TrafficDistribution,
    pub created_at: u64,
    pub last_updated: u64,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub enum LoadBalancingAlgorithm {
    RoundRobin,
    WeightedRoundRobin(HashMap<u32, f64>),
    LeastConnections,
    ResourceBased,
    ResponseTimeBased,
    ConsistentHashing,
    Geographic,
    Custom(String),
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct ShardEndpoint {
    pub shard_id: u32,
    pub canister_id: Principal,
    pub weight: f64,
    pub current_connections: u64,
    pub max_connections: u64,
    pub health_status: HealthStatus,
    pub circuit_breaker: CircuitBreakerState,
    pub performance_metrics: EndpointMetrics,
    pub geographic_region: Option<String>,
    pub last_health_check: u64,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub enum HealthStatus {
    Healthy,
    Warning,
    Unhealthy,
    Maintenance,
    Unknown,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct HealthCheckConfig {
    pub interval_seconds: u64,
    pub timeout_ms: u64,
    pub failure_threshold: u32,
    pub success_threshold: u32,
    pub health_check_path: String,
    pub enabled: bool,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct CircuitBreakerConfig {
    pub failure_threshold: u32,
    pub success_threshold: u32,
    pub timeout_duration_ms: u64,
    pub half_open_max_calls: u32,
    pub failure_rate_threshold: f64,
    pub minimum_throughput: u64,
    pub enabled: bool,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub enum CircuitBreakerState {
    Closed,    // Normal operation
    Open,      // Circuit is open, failing fast
    HalfOpen,  // Testing if service has recovered
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct EndpointMetrics {
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub avg_response_time_ms: u64,
    pub last_request_time: u64,
    pub current_load: f64,
    pub error_rate: f64,
    pub throughput_per_second: f64,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct TrafficDistribution {
    pub total_requests: u64,
    pub shard_distribution: HashMap<u32, u64>,
    pub algorithm_effectiveness: f64,
    pub load_variance: f64,
    pub last_calculated: u64,
}

// ========== CIRCUIT BREAKER IMPLEMENTATION ==========

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct CircuitBreaker {
    pub shard_id: u32,
    pub state: CircuitBreakerState,
    pub failure_count: u32,
    pub success_count: u32,
    pub last_failure_time: u64,
    pub next_attempt_time: u64,
    pub config: CircuitBreakerConfig,
    pub recent_calls: VecDeque<CallResult>,
    pub statistics: CircuitBreakerStats,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct CallResult {
    pub timestamp: u64,
    pub success: bool,
    pub response_time_ms: u64,
    pub error_type: Option<String>,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct CircuitBreakerStats {
    pub total_calls: u64,
    pub successful_calls: u64,
    pub failed_calls: u64,
    pub times_opened: u64,
    pub times_half_opened: u64,
    pub avg_failure_duration_ms: u64,
    pub current_failure_rate: f64,
}

// ========== STORAGE MANAGEMENT ==========

thread_local! {
    static LOAD_BALANCER: RefCell<LoadBalancer> = RefCell::new(LoadBalancer {
        balancer_id: "main_balancer".to_string(),
        algorithm: LoadBalancingAlgorithm::WeightedRoundRobin(HashMap::new()),
        active_shards: Vec::new(),
        health_check_config: HealthCheckConfig {
            interval_seconds: 30,
            timeout_ms: 5000,
            failure_threshold: 3,
            success_threshold: 2,
            health_check_path: "health_check".to_string(),
            enabled: true,
        },
        circuit_breaker_config: CircuitBreakerConfig {
            failure_threshold: 5,
            success_threshold: 3,
            timeout_duration_ms: 60000, // 1 minute
            half_open_max_calls: 10,
            failure_rate_threshold: 50.0, // 50%
            minimum_throughput: 10,
            enabled: true,
        },
        traffic_distribution: TrafficDistribution {
            total_requests: 0,
            shard_distribution: HashMap::new(),
            algorithm_effectiveness: 0.0,
            load_variance: 0.0,
            last_calculated: 0,
        },
        created_at: 0,
        last_updated: 0,
    });
    
    static CIRCUIT_BREAKERS: RefCell<StableBTreeMap<u32, CircuitBreaker, VirtualMemory<DefaultMemoryImpl>>> = 
        RefCell::new(StableBTreeMap::init(get_memory_by_id(MemoryId::new(40))));
    
    static REQUEST_COUNTERS: RefCell<StableBTreeMap<u32, u64, VirtualMemory<DefaultMemoryImpl>>> = 
        RefCell::new(StableBTreeMap::init(get_memory_by_id(MemoryId::new(41))));
    
    static RESPONSE_TIMES: RefCell<StableBTreeMap<u32, VecDeque<u64>, VirtualMemory<DefaultMemoryImpl>>> = 
        RefCell::new(StableBTreeMap::init(get_memory_by_id(MemoryId::new(42))));
    
    static ROUND_ROBIN_COUNTER: RefCell<u32> = RefCell::new(0);
}

// ========== LOAD BALANCING IMPLEMENTATION ==========

/// Get the best available shard based on load balancing algorithm
#[query]
pub fn get_optimal_shard(request_type: RequestType, user_context: Option<UserContext>) -> Result<ShardEndpoint, String> {
    let load_balancer = LOAD_BALANCER.with(|lb| lb.borrow().clone());
    
    // Filter healthy shards
    let healthy_shards: Vec<_> = load_balancer.active_shards
        .into_iter()
        .filter(|shard| {
            shard.health_status == HealthStatus::Healthy &&
            shard.circuit_breaker == CircuitBreakerState::Closed &&
            shard.current_connections < shard.max_connections
        })
        .collect();
    
    if healthy_shards.is_empty() {
        return Err("No healthy shards available".to_string());
    }
    
    let selected_shard = match load_balancer.algorithm {
        LoadBalancingAlgorithm::RoundRobin => {
            select_round_robin(&healthy_shards)?
        },
        LoadBalancingAlgorithm::WeightedRoundRobin(weights) => {
            select_weighted_round_robin(&healthy_shards, &weights)?
        },
        LoadBalancingAlgorithm::LeastConnections => {
            select_least_connections(&healthy_shards)?
        },
        LoadBalancingAlgorithm::ResourceBased => {
            select_resource_based(&healthy_shards)?
        },
        LoadBalancingAlgorithm::ResponseTimeBased => {
            select_response_time_based(&healthy_shards)?
        },
        LoadBalancingAlgorithm::ConsistentHashing => {
            select_consistent_hashing(&healthy_shards, user_context)?
        },
        LoadBalancingAlgorithm::Geographic => {
            select_geographic(&healthy_shards, user_context)?
        },
        LoadBalancingAlgorithm::Custom(algorithm) => {
            select_custom(&healthy_shards, &algorithm)?
        },
    };
    
    // Update request counter
    REQUEST_COUNTERS.with(|counters| {
        let mut counters_ref = counters.borrow_mut();
        let current_count = counters_ref.get(&selected_shard.shard_id).unwrap_or(0);
        counters_ref.insert(selected_shard.shard_id, current_count + 1);
    });
    
    Ok(selected_shard)
}

/// Round robin selection
fn select_round_robin(shards: &[ShardEndpoint]) -> Result<ShardEndpoint, String> {
    if shards.is_empty() {
        return Err("No shards available".to_string());
    }
    
    let index = ROUND_ROBIN_COUNTER.with(|counter| {
        let mut counter_ref = counter.borrow_mut();
        let current_index = *counter_ref % (shards.len() as u32);
        *counter_ref = (*counter_ref + 1) % (shards.len() as u32);
        current_index as usize
    });
    
    Ok(shards[index].clone())
}

/// Weighted round robin selection
fn select_weighted_round_robin(shards: &[ShardEndpoint], weights: &HashMap<u32, f64>) -> Result<ShardEndpoint, String> {
    if shards.is_empty() {
        return Err("No shards available".to_string());
    }
    
    // Calculate weighted selection
    let total_weight: f64 = shards.iter()
        .map(|shard| weights.get(&shard.shard_id).unwrap_or(&1.0))
        .sum();
    
    if total_weight <= 0.0 {
        return select_round_robin(shards);
    }
    
    // Generate random number based on current time (pseudo-random)
    let random_value = (time() % 1000) as f64 / 1000.0 * total_weight;
    let mut cumulative_weight = 0.0;
    
    for shard in shards {
        cumulative_weight += weights.get(&shard.shard_id).unwrap_or(&1.0);
        if random_value <= cumulative_weight {
            return Ok(shard.clone());
        }
    }
    
    // Fallback to last shard
    Ok(shards[shards.len() - 1].clone())
}

/// Least connections selection
fn select_least_connections(shards: &[ShardEndpoint]) -> Result<ShardEndpoint, String> {
    shards.iter()
        .min_by_key(|shard| shard.current_connections)
        .map(|shard| shard.clone())
        .ok_or_else(|| "No shards available".to_string())
}

/// Resource-based selection (lowest CPU/memory usage)
fn select_resource_based(shards: &[ShardEndpoint]) -> Result<ShardEndpoint, String> {
    shards.iter()
        .min_by(|a, b| a.performance_metrics.current_load.partial_cmp(&b.performance_metrics.current_load).unwrap_or(std::cmp::Ordering::Equal))
        .map(|shard| shard.clone())
        .ok_or_else(|| "No shards available".to_string())
}

/// Response time-based selection
fn select_response_time_based(shards: &[ShardEndpoint]) -> Result<ShardEndpoint, String> {
    shards.iter()
        .min_by_key(|shard| shard.performance_metrics.avg_response_time_ms)
        .map(|shard| shard.clone())
        .ok_or_else(|| "No shards available".to_string())
}

/// Consistent hashing selection
fn select_consistent_hashing(shards: &[ShardEndpoint], user_context: Option<UserContext>) -> Result<ShardEndpoint, String> {
    let hash_key = match user_context {
        Some(context) => context.user_id.to_text(),
        None => time().to_string(),
    };
    
    let hash = calculate_hash(&hash_key);
    let shard_index = (hash % shards.len() as u64) as usize;
    
    Ok(shards[shard_index].clone())
}

/// Geographic selection
fn select_geographic(shards: &[ShardEndpoint], user_context: Option<UserContext>) -> Result<ShardEndpoint, String> {
    if let Some(context) = user_context {
        if let Some(user_region) = &context.geographic_region {
            // Find shard in same region
            if let Some(regional_shard) = shards.iter()
                .find(|shard| shard.geographic_region.as_ref() == Some(user_region)) {
                return Ok(regional_shard.clone());
            }
        }
    }
    
    // Fallback to least connections
    select_least_connections(shards)
}

/// Custom algorithm selection
fn select_custom(shards: &[ShardEndpoint], algorithm: &str) -> Result<ShardEndpoint, String> {
    match algorithm {
        "performance_hybrid" => {
            // Hybrid algorithm considering multiple factors
            let mut best_shard = None;
            let mut best_score = f64::MAX;
            
            for shard in shards {
                let load_score = shard.performance_metrics.current_load * 0.4;
                let response_time_score = (shard.performance_metrics.avg_response_time_ms as f64) * 0.3;
                let connection_score = (shard.current_connections as f64 / shard.max_connections as f64) * 0.3;
                
                let total_score = load_score + response_time_score + connection_score;
                
                if total_score < best_score {
                    best_score = total_score;
                    best_shard = Some(shard.clone());
                }
            }
            
            best_shard.ok_or_else(|| "No suitable shard found".to_string())
        },
        _ => select_round_robin(shards)
    }
}

// ========== CIRCUIT BREAKER IMPLEMENTATION ==========

/// Check if circuit breaker allows the request
#[query]
pub fn can_execute_request(shard_id: u32) -> bool {
    CIRCUIT_BREAKERS.with(|breakers| {
        if let Some(breaker) = breakers.borrow().get(&shard_id) {
            match breaker.state {
                CircuitBreakerState::Closed => true,
                CircuitBreakerState::Open => {
                    // Check if timeout period has passed
                    time() >= breaker.next_attempt_time
                },
                CircuitBreakerState::HalfOpen => {
                    // Allow limited requests in half-open state
                    breaker.success_count < breaker.config.half_open_max_calls
                },
            }
        } else {
            true // No circuit breaker configured, allow request
        }
    })
}

/// Record request result and update circuit breaker state
#[update]
pub fn record_request_result(shard_id: u32, success: bool, response_time_ms: u64, error_type: Option<String>) -> Result<(), String> {
    let current_time = time();
    
    CIRCUIT_BREAKERS.with(|breakers| {
        let mut breakers_ref = breakers.borrow_mut();
        
        let mut breaker = breakers_ref.get(&shard_id).unwrap_or_else(|| {
            create_default_circuit_breaker(shard_id)
        });
        
        // Record the call result
        let call_result = CallResult {
            timestamp: current_time,
            success,
            response_time_ms,
            error_type,
        };
        
        breaker.recent_calls.push_back(call_result);
        
        // Keep only recent calls (last 100)
        while breaker.recent_calls.len() > 100 {
            breaker.recent_calls.pop_front();
        }
        
        // Update statistics
        breaker.statistics.total_calls += 1;
        if success {
            breaker.statistics.successful_calls += 1;
            breaker.success_count += 1;
            breaker.failure_count = 0; // Reset failure count on success
        } else {
            breaker.statistics.failed_calls += 1;
            breaker.failure_count += 1;
            breaker.success_count = 0; // Reset success count on failure
            breaker.last_failure_time = current_time;
        }
        
        // Update failure rate
        breaker.statistics.current_failure_rate = 
            (breaker.statistics.failed_calls as f64 / breaker.statistics.total_calls as f64) * 100.0;
        
        // Update circuit breaker state
        breaker.state = calculate_circuit_breaker_state(&breaker);
        
        // Set next attempt time if circuit is opened
        if breaker.state == CircuitBreakerState::Open {
            breaker.next_attempt_time = current_time + (breaker.config.timeout_duration_ms * 1_000_000); // Convert to nanoseconds
            breaker.statistics.times_opened += 1;
        } else if breaker.state == CircuitBreakerState::HalfOpen && 
                 breakers_ref.get(&shard_id).map_or(true, |old| old.state != CircuitBreakerState::HalfOpen) {
            breaker.statistics.times_half_opened += 1;
        }
        
        breakers_ref.insert(shard_id, breaker);
    });
    
    Ok(())
}

/// Calculate circuit breaker state based on current metrics
fn calculate_circuit_breaker_state(breaker: &CircuitBreaker) -> CircuitBreakerState {
    match breaker.state {
        CircuitBreakerState::Closed => {
            // Check if we should open the circuit
            if breaker.failure_count >= breaker.config.failure_threshold ||
               (breaker.statistics.current_failure_rate >= breaker.config.failure_rate_threshold &&
                breaker.statistics.total_calls >= breaker.config.minimum_throughput) {
                CircuitBreakerState::Open
            } else {
                CircuitBreakerState::Closed
            }
        },
        CircuitBreakerState::Open => {
            // Check if timeout period has passed
            if time() >= breaker.next_attempt_time {
                CircuitBreakerState::HalfOpen
            } else {
                CircuitBreakerState::Open
            }
        },
        CircuitBreakerState::HalfOpen => {
            // Check if we should close or re-open the circuit
            if breaker.success_count >= breaker.config.success_threshold {
                CircuitBreakerState::Closed
            } else if breaker.failure_count > 0 {
                CircuitBreakerState::Open
            } else {
                CircuitBreakerState::HalfOpen
            }
        },
    }
}

/// Create default circuit breaker for a shard
fn create_default_circuit_breaker(shard_id: u32) -> CircuitBreaker {
    CircuitBreaker {
        shard_id,
        state: CircuitBreakerState::Closed,
        failure_count: 0,
        success_count: 0,
        last_failure_time: 0,
        next_attempt_time: 0,
        config: CircuitBreakerConfig {
            failure_threshold: 5,
            success_threshold: 3,
            timeout_duration_ms: 60000,
            half_open_max_calls: 10,
            failure_rate_threshold: 50.0,
            minimum_throughput: 10,
            enabled: true,
        },
        recent_calls: VecDeque::new(),
        statistics: CircuitBreakerStats {
            total_calls: 0,
            successful_calls: 0,
            failed_calls: 0,
            times_opened: 0,
            times_half_opened: 0,
            avg_failure_duration_ms: 0,
            current_failure_rate: 0.0,
        },
    }
}

// ========== HEALTH CHECK SYSTEM ==========

/// Perform health check on all shards
#[heartbeat]
pub async fn health_check_heartbeat() {
    let config = LOAD_BALANCER.with(|lb| lb.borrow().health_check_config.clone());
    
    if !config.enabled {
        return;
    }
    
    let current_time = time();
    let check_interval_ns = config.interval_seconds * 1_000_000_000; // Convert to nanoseconds
    
    // Get all shards that need health check
    let shards_to_check: Vec<ShardEndpoint> = LOAD_BALANCER.with(|lb| {
        lb.borrow().active_shards
            .iter()
            .filter(|shard| current_time - shard.last_health_check >= check_interval_ns)
            .cloned()
            .collect()
    });
    
    // Perform health checks
    for shard in shards_to_check {
        match perform_health_check(&shard, &config).await {
            Ok(health_status) => {
                update_shard_health(shard.shard_id, health_status);
            },
            Err(e) => {
                log_audit_action(
                    "HEALTH_CHECK_FAILED".to_string(),
                    format!("Health check failed for shard {}: {}", shard.shard_id, e),
                    caller(),
                    Some(format!("shard_id:{}", shard.shard_id)),
                );
                update_shard_health(shard.shard_id, HealthStatus::Unhealthy);
            }
        }
    }
}

/// Perform health check on a specific shard
async fn perform_health_check(shard: &ShardEndpoint, config: &HealthCheckConfig) -> Result<HealthStatus, String> {
    let start_time = time();
    
    // Simulate inter-canister call for health check
    // In production, this would be an actual call to the shard's health_check endpoint
    match call_shard_health_check(shard.canister_id, config.timeout_ms).await {
        Ok(response) => {
            let response_time = time() - start_time;
            let response_time_ms = response_time / 1_000_000;
            
            // Update response time metrics
            update_shard_response_time(shard.shard_id, response_time_ms);
            
            // Determine health status based on response
            if response.status == "healthy" && response_time_ms < config.timeout_ms {
                Ok(HealthStatus::Healthy)
            } else if response.status == "warning" {
                Ok(HealthStatus::Warning)
            } else {
                Ok(HealthStatus::Unhealthy)
            }
        },
        Err(_) => Ok(HealthStatus::Unhealthy)
    }
}

/// Mock health check call (placeholder for actual inter-canister call)
async fn call_shard_health_check(canister_id: Principal, timeout_ms: u64) -> Result<HealthResponse, String> {
    // This would be an actual inter-canister call in production
    Ok(HealthResponse {
        status: "healthy".to_string(),
        timestamp: time(),
        metrics: HashMap::new(),
    })
}

/// Update shard health status
fn update_shard_health(shard_id: u32, health_status: HealthStatus) {
    LOAD_BALANCER.with(|lb| {
        let mut lb_ref = lb.borrow_mut();
        
        for shard in &mut lb_ref.active_shards {
            if shard.shard_id == shard_id {
                shard.health_status = health_status;
                shard.last_health_check = time();
                break;
            }
        }
        
        lb_ref.last_updated = time();
    });
}

/// Update shard response time metrics
fn update_shard_response_time(shard_id: u32, response_time_ms: u64) {
    RESPONSE_TIMES.with(|response_times| {
        let mut times_ref = response_times.borrow_mut();
        let mut times = times_ref.get(&shard_id).unwrap_or_default();
        
        times.push_back(response_time_ms);
        
        // Keep only last 100 response times
        while times.len() > 100 {
            times.pop_front();
        }
        
        times_ref.insert(shard_id, times);
    });
    
    // Update load balancer metrics
    LOAD_BALANCER.with(|lb| {
        let mut lb_ref = lb.borrow_mut();
        
        for shard in &mut lb_ref.active_shards {
            if shard.shard_id == shard_id {
                // Calculate new average response time
                let times = RESPONSE_TIMES.with(|rt| {
                    rt.borrow().get(&shard_id).unwrap_or_default()
                });
                
                if !times.is_empty() {
                    let sum: u64 = times.iter().sum();
                    shard.performance_metrics.avg_response_time_ms = sum / times.len() as u64;
                }
                break;
            }
        }
    });
}

// ========== ADMIN FUNCTIONS ==========

/// Add a new shard to the load balancer
#[update]
pub fn add_shard_to_balancer(shard_info: ShardInfo, weight: f64, max_connections: u64) -> Result<(), String> {
    let caller = caller();
    if !is_admin(&caller) {
        return Err("Only admin can add shards to load balancer".to_string());
    }
    
    let endpoint = ShardEndpoint {
        shard_id: shard_info.shard_id,
        canister_id: shard_info.canister_id,
        weight,
        current_connections: 0,
        max_connections,
        health_status: HealthStatus::Unknown,
        circuit_breaker: CircuitBreakerState::Closed,
        performance_metrics: EndpointMetrics {
            total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
            avg_response_time_ms: 0,
            last_request_time: 0,
            current_load: 0.0,
            error_rate: 0.0,
            throughput_per_second: 0.0,
        },
        geographic_region: None,
        last_health_check: 0,
    };
    
    LOAD_BALANCER.with(|lb| {
        let mut lb_ref = lb.borrow_mut();
        lb_ref.active_shards.push(endpoint);
        lb_ref.last_updated = time();
    });
    
    log_audit_action(
        "SHARD_ADDED_TO_BALANCER".to_string(),
        format!("Added shard {} to load balancer", shard_info.shard_id),
        caller,
        Some(format!("shard_id:{}", shard_info.shard_id)),
    );
    
    Ok(())
}

/// Remove a shard from the load balancer
#[update]
pub fn remove_shard_from_balancer(shard_id: u32) -> Result<(), String> {
    let caller = caller();
    if !is_admin(&caller) {
        return Err("Only admin can remove shards from load balancer".to_string());
    }
    
    LOAD_BALANCER.with(|lb| {
        let mut lb_ref = lb.borrow_mut();
        lb_ref.active_shards.retain(|shard| shard.shard_id != shard_id);
        lb_ref.last_updated = time();
    });
    
    // Clean up circuit breaker
    CIRCUIT_BREAKERS.with(|breakers| {
        breakers.borrow_mut().remove(&shard_id);
    });
    
    log_audit_action(
        "SHARD_REMOVED_FROM_BALANCER".to_string(),
        format!("Removed shard {} from load balancer", shard_id),
        caller,
        Some(format!("shard_id:{}", shard_id)),
    );
    
    Ok(())
}

/// Update load balancing algorithm
#[update]
pub fn update_load_balancing_algorithm(algorithm: LoadBalancingAlgorithm) -> Result<(), String> {
    let caller = caller();
    if !is_admin(&caller) {
        return Err("Only admin can update load balancing algorithm".to_string());
    }
    
    LOAD_BALANCER.with(|lb| {
        let mut lb_ref = lb.borrow_mut();
        lb_ref.algorithm = algorithm;
        lb_ref.last_updated = time();
    });
    
    log_audit_action(
        "LOAD_BALANCING_ALGORITHM_UPDATED".to_string(),
        "Load balancing algorithm updated".to_string(),
        caller,
        None,
    );
    
    Ok(())
}

// ========== MONITORING & STATISTICS ==========

/// Get load balancer statistics
#[query]
pub fn get_load_balancer_stats() -> LoadBalancerStats {
    let load_balancer = LOAD_BALANCER.with(|lb| lb.borrow().clone());
    let current_time = time();
    
    let total_requests: u64 = load_balancer.active_shards
        .iter()
        .map(|shard| shard.performance_metrics.total_requests)
        .sum();
    
    let healthy_shards = load_balancer.active_shards
        .iter()
        .filter(|shard| shard.health_status == HealthStatus::Healthy)
        .count();
    
    let avg_response_time = if !load_balancer.active_shards.is_empty() {
        load_balancer.active_shards
            .iter()
            .map(|shard| shard.performance_metrics.avg_response_time_ms)
            .sum::<u64>() / load_balancer.active_shards.len() as u64
    } else { 0 };
    
    // Calculate load variance
    let loads: Vec<f64> = load_balancer.active_shards
        .iter()
        .map(|shard| shard.performance_metrics.current_load)
        .collect();
    
    let avg_load = if !loads.is_empty() {
        loads.iter().sum::<f64>() / loads.len() as f64
    } else { 0.0 };
    
    let load_variance = if loads.len() > 1 {
        let variance_sum: f64 = loads.iter()
            .map(|load| (load - avg_load).powi(2))
            .sum();
        variance_sum / (loads.len() - 1) as f64
    } else { 0.0 };
    
    LoadBalancerStats {
        total_shards: load_balancer.active_shards.len() as u32,
        healthy_shards: healthy_shards as u32,
        total_requests,
        avg_response_time_ms: avg_response_time,
        load_variance,
        algorithm_effectiveness: calculate_algorithm_effectiveness(&load_balancer),
        circuit_breaker_stats: get_all_circuit_breaker_stats(),
        uptime_percentage: calculate_uptime_percentage(&load_balancer),
        last_updated: current_time,
    }
}

/// Get circuit breaker statistics for all shards
fn get_all_circuit_breaker_stats() -> HashMap<u32, CircuitBreakerStats> {
    CIRCUIT_BREAKERS.with(|breakers| {
        breakers.borrow()
            .iter()
            .map(|(shard_id, breaker)| (shard_id, breaker.statistics.clone()))
            .collect()
    })
}

/// Calculate algorithm effectiveness
fn calculate_algorithm_effectiveness(load_balancer: &LoadBalancer) -> f64 {
    // Algorithm effectiveness based on load distribution and response times
    let loads: Vec<f64> = load_balancer.active_shards
        .iter()
        .map(|shard| shard.performance_metrics.current_load)
        .collect();
    
    if loads.len() < 2 {
        return 100.0;
    }
    
    let avg_load = loads.iter().sum::<f64>() / loads.len() as f64;
    let max_deviation = loads.iter()
        .map(|load| (load - avg_load).abs())
        .fold(0.0, f64::max);
    
    // Lower deviation means better effectiveness
    let effectiveness = ((1.0 - (max_deviation / (avg_load + 1.0))) * 100.0).max(0.0);
    effectiveness
}

/// Calculate uptime percentage
fn calculate_uptime_percentage(load_balancer: &LoadBalancer) -> f64 {
    if load_balancer.active_shards.is_empty() {
        return 0.0;
    }
    
    let healthy_count = load_balancer.active_shards
        .iter()
        .filter(|shard| shard.health_status == HealthStatus::Healthy)
        .count();
    
    (healthy_count as f64 / load_balancer.active_shards.len() as f64) * 100.0
}

// ========== HELPER FUNCTIONS ==========

/// Calculate simple hash for consistent hashing
fn calculate_hash(input: &str) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    
    let mut hasher = DefaultHasher::new();
    input.hash(&mut hasher);
    hasher.finish()
}

// ========== ADDITIONAL TYPES ==========

#[derive(CandidType, Deserialize, Clone, Debug)]
pub enum RequestType {
    UserQuery,
    DataWrite,
    Analytics,
    HealthCheck,
    Migration,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct UserContext {
    pub user_id: Principal,
    pub geographic_region: Option<String>,
    pub request_priority: RequestPriority,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub enum RequestPriority {
    Critical,
    High,
    Medium,
    Low,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct HealthResponse {
    pub status: String,
    pub timestamp: u64,
    pub metrics: HashMap<String, f64>,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct LoadBalancerStats {
    pub total_shards: u32,
    pub healthy_shards: u32,
    pub total_requests: u64,
    pub avg_response_time_ms: u64,
    pub load_variance: f64,
    pub algorithm_effectiveness: f64,
    pub circuit_breaker_stats: HashMap<u32, CircuitBreakerStats>,
    pub uptime_percentage: f64,
    pub last_updated: u64,
}
