// PRODUCTION ORACLE INTEGRATION
// Replace mock implementation in loan_lifecycle.rs

use ic_cdk::api::management_canister::http_request::{
    HttpRequest, HttpMethod, HttpResponse, TransformContext, TransformArgs,
    http_request
};
use crate::types::CommodityPrice;

// Real commodity price fetcher
async fn fetch_real_commodity_price(commodity_type: &str) -> Result<CommodityPrice, String> {
    let url = format!(
        "https://api.example-commodity.com/v1/prices/{}?currency=IDR", 
        commodity_type
    );
    
    let request = HttpRequest {
        url,
        method: HttpMethod::GET,
        headers: vec![
            ("User-Agent".to_string(), "Agrilends/1.0".to_string()),
            ("Accept".to_string(), "application/json".to_string()),
        ],
        body: None,
        transform: Some(TransformContext::new(transform_commodity_response, vec![])),
    };

    // Make HTTPS outcall (50M cycles max)
    let response = http_request(request, 50_000_000).await
        .map_err(|e| format!("HTTP request failed: {:?}", e))?;

    if response.status != 200 {
        return Err(format!("API error: HTTP {}", response.status));
    }

    parse_commodity_response(response.body)
}

// Response transformer (required for consensus)
fn transform_commodity_response(args: TransformArgs) -> HttpResponse {
    let mut response = args.response;
    
    // Remove non-deterministic headers
    response.headers.retain(|h| {
        h.name != "date" && 
        h.name != "x-request-id" &&
        h.name != "server"
    });
    
    response
}

// Parse JSON response
fn parse_commodity_response(body: Vec<u8>) -> Result<CommodityPrice, String> {
    let json_str = String::from_utf8(body)
        .map_err(|_| "Invalid UTF-8 response")?;
    
    // Simple JSON parsing (or use serde_json if available)
    // Expected format: {"price": 15000, "currency": "IDR", "timestamp": 1234567890}
    
    let price = extract_price_from_json(&json_str)?;
    
    Ok(CommodityPrice {
        price_per_unit: price,
        currency: "IDR".to_string(),
        timestamp: ic_cdk::api::time(),
    })
}

fn extract_price_from_json(json: &str) -> Result<u64, String> {
    // Simple JSON parser for price field
    if let Some(start) = json.find(r#""price":"#) {
        let price_start = start + 8;
        if let Some(end) = json[price_start..].find(',') {
            let price_str = &json[price_start..price_start + end];
            price_str.parse::<u64>()
                .map_err(|_| "Invalid price format".to_string())
        } else {
            Err("Price field not found".to_string())
        }
    } else {
        Err("JSON format invalid".to_string())
    }
}
