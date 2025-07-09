use crate::types::*;
use crate::user_management::{get_user_by_principal, Role};
use candid::Principal;

// Helper function to check if user is authorized to mint NFTs
pub fn is_authorized_to_mint(principal: &Principal) -> bool {
    match get_user_by_principal(principal) {
        Some(user) => user.role == Role::Farmer && user.is_active,
        None => false,
    }
}

// Helper function to validate NFT metadata
pub fn validate_nft_metadata(metadata: &Vec<(String, MetadataValue)>) -> Result<(), String> {
    // Check if required fields are present
    let mut has_asset_description = false;
    let mut has_valuation = false;
    let mut has_legal_doc = false;
    
    for (key, value) in metadata {
        match key.as_str() {
            "asset_description" => {
                if let MetadataValue::Text(desc) = value {
                    if !desc.is_empty() {
                        has_asset_description = true;
                    }
                }
            }
            "valuation_idr" => {
                if let MetadataValue::Nat(val) = value {
                    if *val > 0 {
                        has_valuation = true;
                    }
                }
            }
            "legal_doc_hash" => {
                if let MetadataValue::Text(hash) = value {
                    if !hash.is_empty() {
                        has_legal_doc = true;
                    }
                }
            }
            _ => {}
        }
    }
    
    if !has_asset_description {
        return Err("Missing required field: asset_description".to_string());
    }
    
    if !has_valuation {
        return Err("Missing required field: valuation_idr".to_string());
    }
    
    if !has_legal_doc {
        return Err("Missing required field: legal_doc_hash".to_string());
    }
    
    Ok(())
}

// Helper function to extract metadata values
pub fn extract_metadata_values(metadata: &Vec<(String, MetadataValue)>) -> (String, u64, String) {
    let mut legal_doc_hash = String::new();
    let mut valuation_idr = 0u64;
    let mut asset_description = String::new();
    
    for (key, value) in metadata {
        match key.as_str() {
            "legal_doc_hash" => {
                if let MetadataValue::Text(hash) = value {
                    legal_doc_hash = hash.clone();
                }
            }
            "valuation_idr" => {
                if let MetadataValue::Nat(val) = value {
                    valuation_idr = *val;
                }
            }
            "asset_description" => {
                if let MetadataValue::Text(desc) = value {
                    asset_description = desc.clone();
                }
            }
            _ => {}
        }
    }
    
    (legal_doc_hash, valuation_idr, asset_description)
}

// Helper function to validate BTC address
pub fn validate_btc_address(address: &str) -> bool {
    if address.len() < 26 || address.len() > 62 {
        return false;
    }
    
    // Basic validation - should start with 1, 3, or bc1
    address.starts_with('1') || address.starts_with('3') || address.starts_with("bc1")
}

// Helper function to validate email
pub fn validate_email(email: &str) -> bool {
    email.contains('@') && email.len() >= 5 && email.len() <= 254
}

// Helper function to validate phone number
pub fn validate_phone(phone: &str) -> bool {
    phone.len() >= 10 && phone.len() <= 20 && phone.chars().all(|c| c.is_numeric() || c == '+' || c == '-' || c == ' ')
}
