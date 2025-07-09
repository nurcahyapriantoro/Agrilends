// RWA NFT Management - Complete ICRC-7 compliant implementation
// This module handles Real World Asset tokenization as NFTs

use candid::Principal;
use ic_cdk::api::{time, caller};
use ic_cdk_macros::{query, update};
use crate::types::*;
use crate::storage::*;
use crate::helpers::*;

// ICRC-7 Standard Implementation

/// Emergency stop check
fn check_emergency_stop() -> Result<(), String> {
    let config = get_config();
    if config.emergency_stop {
        return Err("Emergency stop is active. All operations are suspended.".to_string());
    }
    if config.maintenance_mode {
        return Err("System is in maintenance mode. Please try again later.".to_string());
    }
    Ok(())
}

/// Mint new RWA NFT - CRITICAL SECURITY: Only authorized entities can mint
#[update]
pub fn mint_nft(owner: Principal, metadata: Vec<(String, MetadataValue)>) -> RWANFTResult {
    // Check emergency stop
    if let Err(e) = check_emergency_stop() {
        log_action("mint_nft", &e, false);
        return RWANFTResult::Err(e);
    }
    
    let caller = caller();
    
    // Rate limiting
    if let Err(e) = check_rate_limit(&caller, 10) { // Max 10 mints per minute
        log_action("mint_nft", &e, false);
        return RWANFTResult::Err(e);
    }
    
    // Authorization check
    if !is_authorized_to_mint(&caller) {
        let error = "Unauthorized: Only registered farmers can mint NFTs".to_string();
        log_action("mint_nft", &error, false);
        return RWANFTResult::Err(error);
    }
    
    // Validate metadata
    if let Err(e) = validate_nft_metadata(&metadata) {
        log_action("mint_nft", &format!("Metadata validation failed: {}", e), false);
        return RWANFTResult::Err(e);
    }
    
    // Check user limits
    let config = get_config();
    let user_nft_count = count_user_nfts(&owner);
    if user_nft_count >= config.max_nft_per_user {
        let error = format!("User has reached maximum NFT limit: {}", config.max_nft_per_user);
        log_action("mint_nft", &error, false);
        return RWANFTResult::Err(error);
    }
    
    // Validate valuation limits
    let (_, valuation_idr, _) = extract_metadata_values(&metadata);
    if valuation_idr < config.min_collateral_value || valuation_idr > config.max_collateral_value {
        let error = format!("Valuation {} is outside allowed range: {} - {}", 
                          valuation_idr, config.min_collateral_value, config.max_collateral_value);
        log_action("mint_nft", &error, false);
        return RWANFTResult::Err(error);
    }
    
    // Proceed with minting
    let result = do_mint_nft(owner, metadata);
    
    match &result {
        RWANFTResult::Ok(nft) => {
            log_nft_activity("mint_nft", nft.token_id, caller);
        },
        RWANFTResult::Err(e) => {
            log_action("mint_nft", e, false);
        }
    }
    
    result
}

fn do_mint_nft(owner: Principal, metadata: Vec<(String, MetadataValue)>) -> RWANFTResult {
    let token_id = next_nft_token_id();
    let current_time = time();
    
    let nft_data = RWANFTData {
        token_id,
        owner,
        metadata: metadata.clone(),
        created_at: current_time,
        updated_at: current_time,
        is_locked: false,
        loan_id: None,
    };
    
    // Store NFT
    RWA_NFTS.with(|nfts| {
        nfts.borrow_mut().insert(token_id, nft_data.clone());
    });
    
    // Create collateral record
    let (legal_doc_hash, valuation_idr, asset_description) = extract_metadata_values(&metadata);
    let collateral_record = CollateralRecord {
        collateral_id: next_collateral_id(),
        nft_token_id: token_id,
        owner,
        loan_id: None,
        valuation_idr,
        asset_description,
        legal_doc_hash,
        status: CollateralStatus::Available,
        created_at: current_time,
        updated_at: current_time,
    };
    
    // Store collateral record
    COLLATERAL_RECORDS.with(|records| {
        records.borrow_mut().insert(collateral_record.collateral_id, collateral_record);
    });
    
    RWANFTResult::Ok(nft_data)
}

/// Get NFT by token ID
#[query]
pub fn get_nft(token_id: u64) -> Option<RWANFTData> {
    get_nft_by_token_id(token_id)
}

/// Get all NFTs owned by a principal
#[query]
pub fn get_user_nfts(owner: Principal) -> Vec<RWANFTData> {
    RWA_NFTS.with(|nfts| {
        nfts.borrow()
            .iter()
            .filter(|(_, nft_data)| nft_data.owner == owner)
            .map(|(_, nft_data)| nft_data.clone())
            .collect()
    })
}

/// Get NFT statistics
#[query]
pub fn get_nft_stats() -> NFTStats {
    RWA_NFTS.with(|nfts| {
        let nfts_map = nfts.borrow();
        let total_nfts = nfts_map.len() as u64;
        let locked_nfts = nfts_map.iter()
            .filter(|(_, nft)| nft.is_locked)
            .count() as u64;
        
        NFTStats {
            total_nfts,
            locked_nfts,
            available_collateral: total_nfts - locked_nfts,
            liquidated_collateral: 0, // TODO: implement based on collateral status
        }
    })
}

/// Transfer NFT (ICRC-7 compliance)
#[update]
pub fn transfer(request: TransferRequest) -> TransferResult {
    let caller = caller();
    
    // Verify ownership
    if let Some(nft) = get_nft_by_token_id(request.token_id) {
        if nft.owner != caller {
            return TransferResult::Err("Unauthorized: You don't own this NFT".to_string());
        }
        
        if nft.is_locked {
            return TransferResult::Err("NFT is locked and cannot be transferred".to_string());
        }
        
        // Update ownership
        RWA_NFTS.with(|nfts| {
            let mut nfts_map = nfts.borrow_mut();
            if let Some(mut nft_data) = nfts_map.get(&request.token_id) {
                nft_data.owner = request.to.owner;
                nft_data.updated_at = time();
                nfts_map.insert(request.token_id, nft_data);
            }
        });
        
        log_nft_activity("transfer", request.token_id, caller);
        TransferResult::Ok
    } else {
        TransferResult::Err("NFT not found".to_string())
    }
}
