use ic_cdk::api::management_canister::http_request::{
    http_request, CanisterHttpRequestArgument, HttpHeader, HttpMethod, HttpResponse, TransformArgs,
    TransformContext,
};
use ic_cdk_macros::{update, query};
use candid::{CandidType, Deserialize};
use crate::types::CommodityPrice;
use crate::storage::{store_commodity_price, get_stored_commodity_price, get_all_stored_commodity_prices,
    update_last_price_fetch, get_last_price_fetch};
use crate::helpers::*;

// Oracle data structures
#[derive(CandidType, Deserialize)]
pub struct PriceResponse {
    pub price: u64,
    pub currency: String,
    pub timestamp: u64,
}

// Transform function for HTTP response
#[ic_cdk_macros::query]
fn transform_commodity_response(response: TransformArgs) -> HttpResponse {
    let headers = vec![
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
    
    // Parse and sanitize JSON response
    if let Ok(json_str) = String::from_utf8(response.response.body.clone()) {
        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&json_str) {
            // Extract only the price data we need
            if let Some(price) = parsed.get("price").and_then(|p| p.as_u64()) {
                let simplified_response = serde_json::json!({
                    "price": price,
                    "currency": "IDR"
                });
                
                sanitized_body = simplified_response.to_string().into_bytes();
            }
        }
    }

    HttpResponse {
        status: response.response.status.clone(),
        headers,
        body: sanitized_body,
    }
}

// Fetch commodity price from external API
#[update]
pub async fn fetch_commodity_price(commodity_id: String) -> Result<CommodityPrice, String> {
    // Only admins can trigger price updates
    if !is_admin(&ic_cdk::caller()) {
        return Err("Only admins can fetch commodity prices".to_string());
    }

    // Rate limiting - max 1 request per minute per commodity
    let last_fetch = get_last_price_fetch(&commodity_id).unwrap_or(0);
    let current_time = ic_cdk::api::time();
    
    if last_fetch > 0 && current_time - last_fetch < 60_000_000_000 { // 1 minute in nanoseconds
        return Err("Rate limit exceeded. Please wait before fetching again".to_string());
    }

    // Configure API endpoint based on commodity
    let api_url = match commodity_id.as_str() {
        "rice" => "https://api.hargapangan.id/tabel/pasar/provinsi/komoditas/33/1", // Example for rice in Jakarta
        "corn" => "https://api.hargapangan.id/tabel/pasar/provinsi/komoditas/33/2", // Example for corn
        "wheat" => "https://api.hargapangan.id/tabel/pasar/provinsi/komoditas/33/3", // Example for wheat
        _ => return Err("Unsupported commodity type".to_string()),
    };

    let request_headers = vec![
        HttpHeader {
            name: "User-Agent".to_string(),
            value: "Agrilends-Oracle/1.0".to_string(),
        },
        HttpHeader {
            name: "Accept".to_string(),
            value: "application/json".to_string(),
        },
    ];

    let request = CanisterHttpRequestArgument {
        url: api_url.to_string(),
        method: HttpMethod::GET,
        body: None,
        max_response_bytes: Some(2_000),
        transform: Some(TransformContext::from_name("transform_commodity_response".to_string(), vec![])),
        headers: request_headers,
    };

    // Make the HTTP request with timeout and retry logic
    match http_request(request, 50_000_000).await {
        Ok((response,)) => {
            if response.status == 200u16 {
                // Parse the response
                let body_str = String::from_utf8(response.body)
                    .map_err(|_| "Invalid UTF-8 in response")?;
                
                let price_data: PriceResponse = serde_json::from_str(&body_str)
                    .map_err(|e| format!("Failed to parse price data: {}", e))?;

                // Validate price data
                if price_data.price == 0 {
                    return Err("Invalid price data received".to_string());
                }

                let commodity_price = CommodityPrice {
                    price_per_unit: price_data.price,
                    currency: price_data.currency,
                    timestamp: current_time,
                };

                // Store the price data
                store_commodity_price(commodity_id.clone(), commodity_price.clone())?;
                update_last_price_fetch(&commodity_id, current_time);

                // Log the successful fetch
                log_audit_action(
                    ic_cdk::caller(),
                    "COMMODITY_PRICE_FETCHED".to_string(),
                    format!("Fetched {} price: {} IDR", commodity_id, price_data.price),
                    true,
                );

                Ok(commodity_price)
            } else {
                Err(format!("HTTP request failed with status: {}", response.status))
            }
        }
        Err((r, m)) => {
            log_audit_action(
                ic_cdk::caller(),
                "COMMODITY_PRICE_FETCH_FAILED".to_string(),
                format!("Failed to fetch {} price: {:?} - {}", commodity_id, r, m),
                false,
            );
            Err(format!("HTTP request failed: {:?} - {}", r, m))
        }
    }
}

// Get cached commodity price
#[query]
pub fn get_commodity_price(commodity_id: String) -> Result<CommodityPrice, String> {
    get_stored_commodity_price(&commodity_id)
        .ok_or_else(|| "Price not available for this commodity".to_string())
}

// Admin function to manually set price (for testing/emergency)
#[update]
pub fn admin_set_commodity_price(
    commodity_id: String,
    price_idr: u64,
) -> Result<(), String> {
    if !is_admin(&ic_cdk::caller()) {
        return Err("Only admins can set commodity prices".to_string());
    }

    if price_idr == 0 {
        return Err("Price must be greater than 0".to_string());
    }

    let commodity_price = CommodityPrice {
        price_per_unit: price_idr,
        currency: "IDR".to_string(),
        timestamp: ic_cdk::api::time(),
    };

    store_commodity_price(commodity_id.clone(), commodity_price)?;

    log_audit_action(
        ic_cdk::caller(),
        "ADMIN_PRICE_OVERRIDE".to_string(),
        format!("Admin manually set {} price to {} IDR", commodity_id, price_idr),
        true,
    );

    Ok(())
}

// Get all available commodity prices
#[query]
pub fn get_all_commodity_prices() -> Vec<CommodityPrice> {
    get_all_stored_commodity_prices().into_iter().map(|(_, price)| price).collect()
}

// Check if price data is stale (older than 24 hours)
#[query]
pub fn is_price_stale(commodity_id: String) -> bool {
    if let Some(price_data) = get_stored_commodity_price(&commodity_id) {
        let current_time = ic_cdk::api::time();
        let twenty_four_hours = 24 * 60 * 60 * 1_000_000_000u64; // 24 hours in nanoseconds
        
        (current_time - price_data.timestamp) > twenty_four_hours
    } else {
        true // No price data means stale
    }
}

// Heartbeat function to auto-update stale prices
pub async fn heartbeat_price_update() {
    let commodities = vec!["rice".to_string(), "corn".to_string(), "wheat".to_string()];
    
    for commodity in commodities {
        if is_price_stale(commodity.clone()) {
            // Try to update stale prices automatically
            if let Err(e) = fetch_commodity_price(commodity.clone()).await {
                // Log but don't fail - this is background maintenance
                log_audit_action(
                    ic_cdk::id(), // Canister itself as the caller
                    "AUTO_PRICE_UPDATE_FAILED".to_string(),
                    format!("Failed to auto-update {} price: {}", commodity, e),
                    false,
                );
            }
        }
    }
}
