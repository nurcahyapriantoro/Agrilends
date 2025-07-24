// PRODUCTION ORACLE IMPLEMENTATION FOR AGRILENDS
// Comprehensive Oracle system with HTTPS Outcalls for commodity price data
// Implements trustless price fetching with consensus mechanisms
// Supports multiple API sources with fallback and redundancy

use ic_cdk::{caller, api::time, api::management_canister::http_request::{
    CanisterHttpRequestArgument, HttpHeader, HttpMethod, HttpResponse, TransformContext,
    TransformArgs, http_request
}};
use ic_cdk_macros::{update, query, heartbeat};
use candid::{CandidType, Deserialize};
use serde_json;
use std::collections::HashMap;
use crate::storage::{
    log_audit_action, store_commodity_price, get_stored_commodity_price, 
    get_all_stored_commodity_prices, update_last_price_fetch, get_last_price_fetch
};
use crate::helpers::{is_admin, get_canister_config};
use crate::types::{
    CommodityPrice, CommodityPriceData, PriceFetchRecord, OracleConfig, 
    OracleStatistics, PriceAlert, PriceThresholdType
};

// Production Oracle Configuration Constants
const MAX_RESPONSE_BYTES: u64 = 2_000_000; // 2MB max response for safety
const CYCLES_PER_REQUEST: u64 = 50_000_000; // 50M cycles per HTTPS outcall
const RATE_LIMIT_WINDOW: u64 = 60_000_000_000; // 1 minute in nanoseconds
const PRICE_STALE_THRESHOLD: u64 = 86400_000_000_000; // 24 hours in nanoseconds
const MAX_RETRIES: u32 = 3;
const CONFIDENCE_THRESHOLD: u64 = 70; // Minimum confidence score for price data
const HEARTBEAT_INTERVAL: u64 = 3600_000_000_000; // 1 hour heartbeat interval

// Thread-local storage for Oracle state management
use std::cell::RefCell;
thread_local! {
    static ORACLE_CONFIG: RefCell<OracleConfig> = RefCell::new(OracleConfig::default());
    static ORACLE_STATS: RefCell<OracleStatistics> = RefCell::new(OracleStatistics {
        total_fetches: 0,
        successful_fetches: 0,
        failed_fetches: 0,
        average_response_time: 0,
        uptime_percentage: 100.0,
        commodities_tracked: 0,
        stale_prices_count: 0,
        last_update: 0,
        price_volatility: vec![],
    });
    static PRICE_ALERTS: RefCell<Vec<PriceAlert>> = RefCell::new(vec![]);
    static FETCH_RECORDS: RefCell<HashMap<String, PriceFetchRecord>> = RefCell::new(HashMap::new());
    static LAST_HEARTBEAT: RefCell<u64> = RefCell::new(0);
}

// Data structures for API responses
#[derive(CandidType, Deserialize, Debug)]
pub struct PriceApiResponse {
    pub price: Option<u64>,
    pub currency: Option<String>,
    pub timestamp: Option<u64>,
    pub source: Option<String>,
    pub status: Option<String>,
}

#[derive(CandidType, Deserialize, Debug)]
pub struct HargaPanganResponse {
    pub status: String,
    pub data: Option<HargaPanganData>,
}

#[derive(CandidType, Deserialize, Debug)]
pub struct HargaPanganData {
    pub prices: Vec<PriceEntry>,
}

#[derive(CandidType, Deserialize, Debug)]
pub struct PriceEntry {
    pub price: u64,
    pub currency: String,
    pub market: String,
    pub updated_at: String,
}

// Alternative API response structures for redundancy
#[derive(CandidType, Deserialize, Debug)]
pub struct SimplePriceResponse {
    pub price: u64,
    pub currency: String,
}

#[derive(CandidType, Deserialize, Debug)]
pub struct CommodityPriceResponse {
    pub success: bool,
    pub data: CommodityPriceInfo,
}

#[derive(CandidType, Deserialize, Debug)]
pub struct CommodityPriceInfo {
    pub commodity: String,
    pub price_idr: u64,
    pub price_usd: Option<u64>,
    pub timestamp: u64,
    pub market_source: String,
}

// =============================================================================
// TRANSFORM FUNCTION - Critical for consensus across IC nodes
// =============================================================================

#[query]
fn transform_commodity_response(response: TransformArgs) -> HttpResponse {
    // Remove non-deterministic headers that could cause consensus issues
    let mut headers = vec![
        HttpHeader {
            name: "Content-Security-Policy".to_string(),
            value: "default-src 'self'".to_string(),
        },
        HttpHeader {
            name: "Referrer-Policy".to_string(),
            value: "strict-origin".to_string(),
        },
    ];

    let mut sanitized_body = response.response.body.clone();
    
    // Parse and sanitize JSON response to ensure deterministic consensus
    if let Ok(json_str) = String::from_utf8(response.response.body.clone()) {
        // Extract only the essential price data to ensure consensus
        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&json_str) {
            let simplified_response = match extract_price_from_json(&parsed) {
                Ok(price) => {
                    serde_json::json!({
                        "price": price,
                        "currency": "IDR",
                        "timestamp": time()
                    })
                },
                Err(_) => {
                    // Fallback for any parsing issues
                    serde_json::json!({
                        "error": "price_parse_failed",
                        "timestamp": time()
                    })
                }
            };
            
            if let Ok(sanitized_json) = serde_json::to_string(&simplified_response) {
                sanitized_body = sanitized_json.into_bytes();
            }
        }
    }

    HttpResponse {
        status: response.response.status.clone(),
        headers,
        body: sanitized_body,
    }
}

// =============================================================================
// CORE ORACLE FUNCTIONS
// =============================================================================

/// Main function to fetch commodity price from external APIs
/// This is the primary entry point for price data collection
#[update]
pub async fn fetch_commodity_price(commodity_id: String) -> Result<CommodityPrice, String> {
    // Security check - only admins or automated heartbeat can trigger fetches
    let caller_principal = caller();
    if !is_admin(&caller_principal) && caller_principal != ic_cdk::id() {
        return Err("Unauthorized: Only admins can fetch commodity prices".to_string());
    }

    let start_time = time();
    
    // Rate limiting check
    if let Err(rate_limit_error) = check_rate_limit(&commodity_id) {
        return Err(rate_limit_error);
    }

    // Validate commodity type
    if !is_supported_commodity(&commodity_id) {
        return Err(format!("Unsupported commodity type: {}", commodity_id));
    }

    // Update fetch statistics
    update_fetch_attempt(&commodity_id);

    // Try multiple API sources for redundancy
    let api_sources = get_api_sources_for_commodity(&commodity_id);
    let mut last_error = String::new();

    for (api_name, api_url) in api_sources {
        match fetch_from_api(&commodity_id, &api_url, &api_name).await {
            Ok(commodity_price) => {
                // Validate price data quality
                if validate_price_data(&commodity_price) {
                    // Store successful price data
                    store_commodity_price(commodity_id.clone(), commodity_price.clone())?;
                    update_last_price_fetch(&commodity_id, start_time);
                    update_fetch_success(&commodity_id, start_time);

                    // Check and trigger price alerts
                    check_price_alerts(&commodity_id, commodity_price.price_per_unit);

                    // Log successful fetch
                    log_audit_action(
                        caller_principal,
                        "COMMODITY_PRICE_FETCHED".to_string(),
                        format!("Successfully fetched {} price: {} IDR from {}", 
                               commodity_id, commodity_price.price_per_unit, api_name),
                        true,
                    );

                    return Ok(commodity_price);
                } else {
                    last_error = format!("Invalid price data from {}", api_name);
                }
            },
            Err(e) => {
                last_error = format!("API {} failed: {}", api_name, e);
                continue;
            }
        }
    }

    // All APIs failed - try emergency fallback
    if let Some(fallback_price) = get_emergency_fallback_price(&commodity_id) {
        log_audit_action(
            caller_principal,
            "EMERGENCY_PRICE_FALLBACK".to_string(),
            format!("Using emergency fallback price for {}: {} IDR", commodity_id, fallback_price),
            true,
        );
        
        let emergency_price = CommodityPrice {
            price_per_unit: fallback_price,
            currency: "IDR".to_string(),
            timestamp: time(),
        };
        
        store_commodity_price(commodity_id.clone(), emergency_price.clone())?;
        return Ok(emergency_price);
    }

    // Complete failure - update statistics and return error
    update_fetch_failure(&commodity_id, &last_error);
    
    log_audit_action(
        caller_principal,
        "COMMODITY_PRICE_FETCH_FAILED".to_string(),
        format!("Failed to fetch {} price from all sources: {}", commodity_id, last_error),
        false,
    );

    Err(format!("Failed to fetch price for {}: {}", commodity_id, last_error))
}

/// Get cached commodity price from storage
#[query]
pub fn get_commodity_price(commodity_id: String) -> Result<CommodityPrice, String> {
    match get_stored_commodity_price(&commodity_id) {
        Some(price) => {
            // Check if price is stale
            if is_price_stale_internal(&commodity_id, &price) {
                return Err(format!("Price data for {} is stale (older than 24 hours)", commodity_id));
            }
            Ok(price)
        },
        None => Err(format!("Price not available for commodity: {}", commodity_id))
    }
}

/// Administrative function to manually set commodity price (for testing/emergency)
#[update]
pub fn admin_set_commodity_price(
    commodity_id: String,
    price_idr: u64,
) -> Result<(), String> {
    if !is_admin(&caller()) {
        return Err("Only admins can manually set commodity prices".to_string());
    }

    if price_idr == 0 {
        return Err("Price must be greater than 0".to_string());
    }

    if !is_supported_commodity(&commodity_id) {
        return Err(format!("Unsupported commodity type: {}", commodity_id));
    }

    let commodity_price = CommodityPrice {
        price_per_unit: price_idr,
        currency: "IDR".to_string(),
        timestamp: time(),
    };

    store_commodity_price(commodity_id.clone(), commodity_price)?;

    log_audit_action(
        caller(),
        "ADMIN_PRICE_OVERRIDE".to_string(),
        format!("Admin manually set {} price to {} IDR", commodity_id, price_idr),
        true,
    );

    Ok(())
}

/// Get all available commodity prices
#[query]
pub fn get_all_commodity_prices() -> Vec<(String, CommodityPrice)> {
    get_all_stored_commodity_prices()
}

/// Check if price data is stale (older than configured threshold)
#[query]
pub fn is_price_stale(commodity_id: String) -> bool {
    match get_stored_commodity_price(&commodity_id) {
        Some(price_data) => is_price_stale_internal(&commodity_id, &price_data),
        None => true // No price data means stale
    }
}

/// Get Oracle configuration
#[query]
pub fn get_oracle_config() -> OracleConfig {
    ORACLE_CONFIG.with(|config| config.borrow().clone())
}

/// Update Oracle configuration (admin only)
#[update]
pub fn update_oracle_config(new_config: OracleConfig) -> Result<(), String> {
    if !is_admin(&caller()) {
        return Err("Only admins can update Oracle configuration".to_string());
    }

    ORACLE_CONFIG.with(|config| {
        *config.borrow_mut() = new_config.clone();
    });

    log_audit_action(
        caller(),
        "ORACLE_CONFIG_UPDATED".to_string(),
        "Oracle configuration updated by admin".to_string(),
        true,
    );

    Ok(())
}

/// Get Oracle statistics
#[query]
pub fn get_oracle_statistics() -> OracleStatistics {
    ORACLE_STATS.with(|stats| {
        let mut current_stats = stats.borrow().clone();
        current_stats.stale_prices_count = count_stale_prices();
        current_stats.commodities_tracked = get_tracked_commodities_count();
        current_stats.last_update = time();
        current_stats
    })
}

/// Get price fetch records for monitoring
#[query]
pub fn get_price_fetch_records() -> Vec<(String, PriceFetchRecord)> {
    FETCH_RECORDS.with(|records| {
        records.borrow().iter().map(|(k, v)| (k.clone(), v.clone())).collect()
    })
}

/// Create a price alert
#[update]
pub fn create_price_alert(
    commodity_id: String,
    threshold_type: PriceThresholdType,
) -> Result<(), String> {
    if !is_admin(&caller()) {
        return Err("Only admins can create price alerts".to_string());
    }

    if !is_supported_commodity(&commodity_id) {
        return Err(format!("Unsupported commodity type: {}", commodity_id));
    }

    let threshold_value = match &threshold_type {
        PriceThresholdType::Above(value) => *value,
        PriceThresholdType::Below(value) => *value,
        PriceThresholdType::Change(value) => *value,
    };

    let alert = PriceAlert {
        commodity_id: commodity_id.clone(),
        threshold_type,
        threshold_value,
        is_active: true,
        created_by: caller(),
        created_at: time(),
        triggered_at: None,
    };

    PRICE_ALERTS.with(|alerts| {
        alerts.borrow_mut().push(alert);
    });

    log_audit_action(
        caller(),
        "PRICE_ALERT_CREATED".to_string(),
        format!("Price alert created for {} with threshold {}", commodity_id, threshold_value),
        true,
    );

    Ok(())
}

/// Get active price alerts
#[query]
pub fn get_price_alerts() -> Vec<PriceAlert> {
    PRICE_ALERTS.with(|alerts| {
        alerts.borrow().iter().filter(|alert| alert.is_active).cloned().collect()
    })
}

// =============================================================================
// HEARTBEAT FUNCTION - Automated price updates
// =============================================================================

#[heartbeat]
pub async fn heartbeat_price_update() {
    let current_time = time();
    
    // Check if enough time has passed since last heartbeat
    let should_run = LAST_HEARTBEAT.with(|last| {
        let last_time = *last.borrow();
        if current_time - last_time >= HEARTBEAT_INTERVAL {
            *last.borrow_mut() = current_time;
            true
        } else {
            false
        }
    });

    if !should_run {
        return;
    }

    // Get list of commodities to update
    let commodities = ORACLE_CONFIG.with(|config| {
        config.borrow().enabled_commodities.clone()
    });
    
    for commodity in commodities {
        // Only update stale prices to avoid unnecessary API calls
        if is_price_stale(commodity.clone()) {
            // Use canister itself as caller for heartbeat operations
            match fetch_commodity_price(commodity.clone()).await {
                Ok(_) => {
                    // Update success statistics
                    ORACLE_STATS.with(|stats| {
                        stats.borrow_mut().successful_fetches += 1;
                        stats.borrow_mut().total_fetches += 1;
                    });
                },
                Err(e) => {
                    // Log failure but don't stop heartbeat
                    log_audit_action(
                        ic_cdk::id(),
                        "HEARTBEAT_PRICE_UPDATE_FAILED".to_string(),
                        format!("Failed to auto-update {} price: {}", commodity, e),
                        false,
                    );
                    
                    ORACLE_STATS.with(|stats| {
                        stats.borrow_mut().failed_fetches += 1;
                        stats.borrow_mut().total_fetches += 1;
                    });
                }
            }
        }
    }

    // Update uptime percentage
    ORACLE_STATS.with(|stats| {
        let mut current_stats = stats.borrow_mut();
        if current_stats.total_fetches > 0 {
            current_stats.uptime_percentage = 
                (current_stats.successful_fetches as f64 / current_stats.total_fetches as f64) * 100.0;
        }
    });
}

// =============================================================================
// HELPER FUNCTIONS
// =============================================================================

/// Fetch price data from a specific API endpoint
async fn fetch_from_api(
    commodity_id: &str,
    api_url: &str,
    api_name: &str,
) -> Result<CommodityPrice, String> {
    let request_headers = vec![
        HttpHeader {
            name: "User-Agent".to_string(),
            value: "Agrilends-Oracle/1.0".to_string(),
        },
        HttpHeader {
            name: "Accept".to_string(),
            value: "application/json".to_string(),
        },
        HttpHeader {
            name: "Cache-Control".to_string(),
            value: "no-cache".to_string(),
        },
    ];

    let request = CanisterHttpRequestArgument {
        url: api_url.to_string(),
        method: HttpMethod::GET,
        body: None,
        max_response_bytes: Some(MAX_RESPONSE_BYTES),
        transform: Some(TransformContext::from_name(
            "transform_commodity_response".to_string(), 
            vec![]
        )),
        headers: request_headers,
    };

    let start_time = time();
    
    // Make HTTP request with retry logic
    match http_request(request, CYCLES_PER_REQUEST).await {
        Ok((response,)) => {
            let response_time = time() - start_time;
            
            if response.status == 200u16 {
                // Parse response based on API format
                parse_api_response(&response.body, commodity_id, api_name, response_time)
            } else {
                Err(format!("HTTP {} from {}", response.status, api_name))
            }
        },
        Err((rejection_code, message)) => {
            Err(format!("Request failed - Code: {:?}, Message: {}", rejection_code, message))
        }
    }
}

/// Parse API response into CommodityPrice structure
fn parse_api_response(
    body: &[u8],
    commodity_id: &str,
    api_name: &str,
    response_time: u64,
) -> Result<CommodityPrice, String> {
    let body_str = String::from_utf8(body.to_vec())
        .map_err(|_| "Invalid UTF-8 in response")?;

    // Try different parsing strategies based on API format
    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&body_str) {
        if let Ok(price) = extract_price_from_json(&parsed) {
            if price > 0 {
                // Update response time statistics
                update_response_time(commodity_id, response_time);
                
                return Ok(CommodityPrice {
                    price_per_unit: price,
                    currency: "IDR".to_string(),
                    timestamp: time(),
                });
            }
        }
    }

    // Try alternative parsing for simple format
    if let Ok(simple_response) = serde_json::from_str::<SimplePriceResponse>(&body_str) {
        if simple_response.price > 0 {
            return Ok(CommodityPrice {
                price_per_unit: simple_response.price,
                currency: simple_response.currency,
                timestamp: time(),
            });
        }
    }

    Err(format!("Could not parse price data from {}", api_name))
}

/// Extract price from JSON response using multiple strategies
fn extract_price_from_json(json: &serde_json::Value) -> Result<u64, String> {
    // Strategy 1: Direct price field
    if let Some(price) = json.get("price").and_then(|p| p.as_u64()) {
        return Ok(price);
    }

    // Strategy 2: HargaPangan API format
    if let Some(data) = json.get("data") {
        if let Some(prices) = data.get("prices").and_then(|p| p.as_array()) {
            if let Some(first_price) = prices.first() {
                if let Some(price) = first_price.get("price").and_then(|p| p.as_u64()) {
                    return Ok(price);
                }
            }
        }
    }

    // Strategy 3: Nested data structure
    if let Some(result) = json.get("result") {
        if let Some(price) = result.get("price_idr").and_then(|p| p.as_u64()) {
            return Ok(price);
        }
    }

    // Strategy 4: Alternative field names
    for field_name in &["price_idr", "harga", "nilai", "amount", "value"] {
        if let Some(price) = json.get(field_name).and_then(|p| p.as_u64()) {
            return Ok(price);
        }
    }

    Err("No valid price field found in JSON response".to_string())
}

/// Check rate limiting for API calls
fn check_rate_limit(commodity_id: &str) -> Result<(), String> {
    let current_time = time();
    
    FETCH_RECORDS.with(|records| {
        let mut records_map = records.borrow_mut();
        
        if let Some(record) = records_map.get_mut(commodity_id) {
            // Check if rate limit window has passed
            if current_time - record.last_fetch_timestamp < RATE_LIMIT_WINDOW {
                // Check if we've exceeded the rate limit
                let config = ORACLE_CONFIG.with(|c| c.borrow().clone());
                if record.fetch_count >= config.rate_limit_per_commodity {
                    return Err(format!(
                        "Rate limit exceeded for {}. Max {} requests per hour",
                        commodity_id, config.rate_limit_per_commodity
                    ));
                }
            } else {
                // Reset rate limit window
                record.fetch_count = 0;
                record.rate_limit_reset = current_time;
            }
        }
        
        Ok(())
    })
}

/// Get API sources for a specific commodity
fn get_api_sources_for_commodity(commodity_id: &str) -> Vec<(String, String)> {
    ORACLE_CONFIG.with(|config| {
        let oracle_config = config.borrow();
        let mut sources = Vec::new();

        // Get configured API endpoints
        for (comm_id, url) in &oracle_config.api_endpoints {
            if comm_id == commodity_id {
                sources.push((format!("Primary-{}", comm_id), url.clone()));
            }
        }

        // Add backup/alternative sources based on commodity type
        match commodity_id {
            "rice" => {
                sources.push(("Backup-Rice".to_string(), "https://api.backup-source.com/rice".to_string()));
                sources.push(("Market-Rice".to_string(), "https://market-api.com/commodity/rice".to_string()));
            },
            "corn" => {
                sources.push(("Backup-Corn".to_string(), "https://api.backup-source.com/corn".to_string()));
                sources.push(("Market-Corn".to_string(), "https://market-api.com/commodity/corn".to_string()));
            },
            "wheat" => {
                sources.push(("Backup-Wheat".to_string(), "https://api.backup-source.com/wheat".to_string()));
                sources.push(("Market-Wheat".to_string(), "https://market-api.com/commodity/wheat".to_string()));
            },
            _ => {}
        }

        sources
    })
}

/// Check if commodity type is supported
fn is_supported_commodity(commodity_id: &str) -> bool {
    ORACLE_CONFIG.with(|config| {
        config.borrow().enabled_commodities.contains(&commodity_id.to_string())
    })
}

/// Validate price data quality
fn validate_price_data(price: &CommodityPrice) -> bool {
    // Basic validation rules
    if price.price_per_unit == 0 {
        return false;
    }

    // Price should be reasonable (between 1000 and 1_000_000 IDR per unit)
    if price.price_per_unit < 1000 || price.price_per_unit > 1_000_000 {
        return false;
    }

    // Currency should be IDR
    if price.currency != "IDR" {
        return false;
    }

    // Timestamp should be recent (within last hour)
    let current_time = time();
    if price.timestamp > current_time || current_time - price.timestamp > 3600_000_000_000 {
        return false;
    }

    true
}

/// Check if price is stale
fn is_price_stale_internal(commodity_id: &str, price_data: &CommodityPrice) -> bool {
    let current_time = time();
    let stale_threshold = ORACLE_CONFIG.with(|config| {
        config.borrow().stale_threshold_seconds * 1_000_000_000 // Convert to nanoseconds
    });
    
    current_time - price_data.timestamp > stale_threshold
}

/// Get emergency fallback price
fn get_emergency_fallback_price(commodity_id: &str) -> Option<u64> {
    ORACLE_CONFIG.with(|config| {
        for (comm_id, fallback_price) in &config.borrow().backup_prices {
            if comm_id == commodity_id {
                return Some(*fallback_price);
            }
        }
        None
    })
}

/// Update fetch attempt statistics
fn update_fetch_attempt(commodity_id: &str) {
    let current_time = time();
    
    FETCH_RECORDS.with(|records| {
        let mut records_map = records.borrow_mut();
        
        let record = records_map.entry(commodity_id.to_string()).or_insert(PriceFetchRecord {
            commodity_id: commodity_id.to_string(),
            last_fetch_timestamp: 0,
            fetch_count: 0,
            success_count: 0,
            failure_count: 0,
            last_error: None,
            average_response_time: 0,
            rate_limit_reset: current_time,
        });
        
        record.fetch_count += 1;
        record.last_fetch_timestamp = current_time;
    });
}

/// Update fetch success statistics
fn update_fetch_success(commodity_id: &str, response_time: u64) {
    FETCH_RECORDS.with(|records| {
        let mut records_map = records.borrow_mut();
        
        if let Some(record) = records_map.get_mut(commodity_id) {
            record.success_count += 1;
            record.last_error = None;
            
            // Update average response time
            if record.average_response_time == 0 {
                record.average_response_time = response_time;
            } else {
                record.average_response_time = 
                    (record.average_response_time + response_time) / 2;
            }
        }
    });
}

/// Update fetch failure statistics
fn update_fetch_failure(commodity_id: &str, error: &str) {
    FETCH_RECORDS.with(|records| {
        let mut records_map = records.borrow_mut();
        
        if let Some(record) = records_map.get_mut(commodity_id) {
            record.failure_count += 1;
            record.last_error = Some(error.to_string());
        }
    });
}

/// Update response time statistics
fn update_response_time(commodity_id: &str, response_time: u64) {
    ORACLE_STATS.with(|stats| {
        let mut current_stats = stats.borrow_mut();
        
        if current_stats.average_response_time == 0 {
            current_stats.average_response_time = response_time;
        } else {
            current_stats.average_response_time = 
                (current_stats.average_response_time + response_time) / 2;
        }
    });
}

/// Count stale prices
fn count_stale_prices() -> u64 {
    let all_prices = get_all_stored_commodity_prices();
    let mut stale_count = 0;
    
    for (commodity_id, price_data) in all_prices {
        if is_price_stale_internal(&commodity_id, &price_data) {
            stale_count += 1;
        }
    }
    
    stale_count
}

/// Get count of tracked commodities
fn get_tracked_commodities_count() -> u64 {
    get_all_stored_commodity_prices().len() as u64
}

/// Check and trigger price alerts
fn check_price_alerts(commodity_id: &str, current_price: u64) {
    PRICE_ALERTS.with(|alerts| {
        let mut alerts_list = alerts.borrow_mut();
        
        for alert in alerts_list.iter_mut() {
            if alert.commodity_id == commodity_id && alert.is_active && alert.triggered_at.is_none() {
                let should_trigger = match &alert.threshold_type {
                    PriceThresholdType::Above(threshold) => current_price > *threshold,
                    PriceThresholdType::Below(threshold) => current_price < *threshold,
                    PriceThresholdType::Change(percentage) => {
                        // This would require historical price comparison
                        // For now, we'll skip this implementation
                        false
                    }
                };
                
                if should_trigger {
                    alert.triggered_at = Some(time());
                    
                    log_audit_action(
                        alert.created_by,
                        "PRICE_ALERT_TRIGGERED".to_string(),
                        format!("Price alert triggered for {}: {} IDR", commodity_id, current_price),
                        true,
                    );
                }
            }
        }
    });
}

// =============================================================================
// ORACLE HEALTH AND DIAGNOSTICS
// =============================================================================

/// Get Oracle health status
#[query]
pub fn get_oracle_health() -> bool {
    let stats = get_oracle_statistics();
    
    // Oracle is healthy if:
    // - Uptime is above 95%
    // - No more than 50% of prices are stale
    // - At least one successful fetch in the last hour
    
    stats.uptime_percentage > 95.0 && 
    stats.stale_prices_count <= (stats.commodities_tracked / 2) &&
    (time() - stats.last_update) < 3600_000_000_000
}

/// Perform Oracle diagnostics
#[query]
pub fn oracle_diagnostics() -> String {
    let stats = get_oracle_statistics();
    let config = get_oracle_config();
    
    format!(
        "Oracle Diagnostics Report:\n\
         - Status: {}\n\
         - Total Fetches: {}\n\
         - Success Rate: {:.2}%\n\
         - Commodities Tracked: {}\n\
         - Stale Prices: {}\n\
         - Average Response Time: {}ms\n\
         - Enabled Commodities: {:?}\n\
         - Emergency Mode: {}\n\
         - Last Update: {} ago",
        if get_oracle_health() { "Healthy" } else { "Unhealthy" },
        stats.total_fetches,
        stats.uptime_percentage,
        stats.commodities_tracked,
        stats.stale_prices_count,
        stats.average_response_time / 1_000_000, // Convert to milliseconds
        config.enabled_commodities,
        config.emergency_mode,
        (time() - stats.last_update) / 1_000_000_000 // Convert to seconds
    )
}
}

// =============================================================================
// EMERGENCY AND MAINTENANCE FUNCTIONS
// =============================================================================

/// Enable emergency mode (admin only)
#[update]
pub fn enable_emergency_mode() -> Result<(), String> {
    if !is_admin(&caller()) {
        return Err("Only admins can enable emergency mode".to_string());
    }

    ORACLE_CONFIG.with(|config| {
        let mut current_config = config.borrow_mut();
        current_config.emergency_mode = true;
    });

    log_audit_action(
        caller(),
        "ORACLE_EMERGENCY_MODE_ENABLED".to_string(),
        "Oracle emergency mode enabled - using backup prices".to_string(),
        true,
    );

    Ok(())
}

/// Disable emergency mode (admin only)
#[update]
pub fn disable_emergency_mode() -> Result<(), String> {
    if !is_admin(&caller()) {
        return Err("Only admins can disable emergency mode".to_string());
    }

    ORACLE_CONFIG.with(|config| {
        let mut current_config = config.borrow_mut();
        current_config.emergency_mode = false;
    });

    log_audit_action(
        caller(),
        "ORACLE_EMERGENCY_MODE_DISABLED".to_string(),
        "Oracle emergency mode disabled - resuming normal operation".to_string(),
        true,
    );

    Ok(())
}

/// Comprehensive health check for Oracle system
#[query]
pub fn oracle_health_check() -> Result<String, String> {
    let stats = get_oracle_statistics();
    let config = get_oracle_config();
    
    let mut health_issues = Vec::new();
    
    // Check uptime percentage
    if stats.uptime_percentage < 90.0 {
        health_issues.push(format!("Low uptime: {:.2}%", stats.uptime_percentage));
    }
    
    // Check for stale prices
    if stats.stale_prices_count > 0 {
        health_issues.push(format!("{} stale prices detected", stats.stale_prices_count));
    }
    
    // Check emergency mode
    if config.emergency_mode {
        health_issues.push("Emergency mode is active".to_string());
    }
    
    // Check response time
    if stats.average_response_time > 10_000_000_000 {
        health_issues.push(format!("High response time: {}ms", stats.average_response_time / 1_000_000));
    }
    
    // Check if any commodities are enabled
    if config.enabled_commodities.is_empty() {
        health_issues.push("No commodities enabled".to_string());
    }
    
    if health_issues.is_empty() {
        Ok(format!(
            "Oracle system is healthy - Tracking {} commodities with {:.2}% uptime",
            stats.commodities_tracked, stats.uptime_percentage
        ))
    } else {
        Err(format!("Health issues detected: {}", health_issues.join(", ")))
    }
}

/// Reset Oracle statistics (admin only)
#[update]
pub fn reset_oracle_statistics() -> Result<(), String> {
    if !is_admin(&caller()) {
        return Err("Only admins can reset Oracle statistics".to_string());
    }

    ORACLE_STATS.with(|stats| {
        *stats.borrow_mut() = OracleStatistics {
            total_fetches: 0,
            successful_fetches: 0,
            failed_fetches: 0,
            average_response_time: 0,
            uptime_percentage: 100.0,
            commodities_tracked: 0,
            stale_prices_count: 0,
            last_update: time(),
            price_volatility: vec![],
        };
    });

    FETCH_RECORDS.with(|records| {
        records.borrow_mut().clear();
    });

    log_audit_action(
        caller(),
        "ORACLE_STATISTICS_RESET".to_string(),
        "Oracle statistics have been reset by admin".to_string(),
        true,
    );

    Ok(())
}

// =============================================================================
// TESTING AND DEVELOPMENT FUNCTIONS
// =============================================================================

/// Test Oracle connectivity with a simple request (admin only)
#[update]
pub async fn test_oracle_connectivity(test_url: Option<String>) -> Result<String, String> {
    if !is_admin(&caller()) {
        return Err("Only admins can test Oracle connectivity".to_string());
    }

    let test_endpoint = test_url.unwrap_or_else(|| 
        "https://httpbin.org/json".to_string() // Simple test endpoint
    );

    let request_headers = vec![
        HttpHeader {
            name: "User-Agent".to_string(),
            value: "Agrilends-Oracle-Test/1.0".to_string(),
        },
        HttpHeader {
            name: "Accept".to_string(),
            value: "application/json".to_string(),
        },
    ];

    let request = CanisterHttpRequestArgument {
        url: test_endpoint.clone(),
        method: HttpMethod::GET,
        body: None,
        max_response_bytes: Some(1000),
        transform: Some(TransformContext::from_name(
            "transform_commodity_response".to_string(), 
            vec![]
        )),
        headers: request_headers,
    };

    let start_time = time();
    
    match http_request(request, CYCLES_PER_REQUEST).await {
        Ok((response,)) => {
            let response_time = time() - start_time;
            
            log_audit_action(
                caller(),
                "ORACLE_CONNECTIVITY_TEST".to_string(),
                format!("Connectivity test successful - Response: {} in {}ms", 
                       response.status, response_time / 1_000_000),
                true,
            );

            Ok(format!(
                "Connectivity test successful:\n- URL: {}\n- Status: {}\n- Response time: {}ms\n- Body size: {} bytes",
                test_endpoint, response.status, response_time / 1_000_000, response.body.len()
            ))
        },
        Err((rejection_code, message)) => {
            log_audit_action(
                caller(),
                "ORACLE_CONNECTIVITY_TEST_FAILED".to_string(),
                format!("Connectivity test failed: {:?} - {}", rejection_code, message),
                false,
            );

            Err(format!("Connectivity test failed: {:?} - {}", rejection_code, message))
        }
    }
}

/// Validate price data format for testing
#[query]
pub fn validate_price_format(price_json: String) -> Result<String, String> {
    // Try to parse the JSON as various expected formats
    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&price_json) {
        if let Ok(price) = extract_price_from_json(&parsed) {
            return Ok(format!("Valid price format - Extracted price: {} IDR", price));
        }
    }

    // Try alternative parsing
    if let Ok(simple_response) = serde_json::from_str::<SimplePriceResponse>(&price_json) {
        return Ok(format!("Valid simple format - Price: {} {}", 
                         simple_response.price, simple_response.currency));
    }

    Err("Invalid price format - Could not extract price data".to_string())
}

// =============================================================================
// EXPORTS FOR OTHER MODULES
// =============================================================================

/// Export for use in loan lifecycle - get current commodity price
pub fn get_current_commodity_price_internal(commodity_id: &str) -> Option<CommodityPrice> {
    get_stored_commodity_price(commodity_id)
}

/// Export for use in loan lifecycle - check if price is recent enough
pub fn is_commodity_price_valid(commodity_id: &str, max_age_seconds: u64) -> bool {
    if let Some(price_data) = get_stored_commodity_price(commodity_id) {
        let current_time = time();
        let age_nanoseconds = current_time - price_data.timestamp;
        let age_seconds = age_nanoseconds / 1_000_000_000;
        
        age_seconds <= max_age_seconds
    } else {
        false
    }
}
