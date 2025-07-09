use crate::types::*;
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{DefaultMemoryImpl, StableBTreeMap};
use std::cell::RefCell;
use candid::Principal;

// Memory types
type Memory = VirtualMemory<DefaultMemoryImpl>;
type NFTStorage = StableBTreeMap<u64, RWANFTData, Memory>;
type CollateralStorage = StableBTreeMap<u64, CollateralRecord, Memory>;

// Memory Manager
thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(
        MemoryManager::init(DefaultMemoryImpl::default())
    );
}

// Storage for RWA NFTs
thread_local! {
    pub static RWA_NFTS: RefCell<NFTStorage> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(1)))
        )
    );
}

// Storage for collateral records
thread_local! {
    pub static COLLATERAL_RECORDS: RefCell<CollateralStorage> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(2)))
        )
    );
}

// Token ID counters
thread_local! {
    static NFT_TOKEN_COUNTER: RefCell<u64> = RefCell::new(0);
    static COLLATERAL_COUNTER: RefCell<u64> = RefCell::new(0);
}

// Helper functions for token ID generation
pub fn next_nft_token_id() -> u64 {
    NFT_TOKEN_COUNTER.with(|counter| {
        let current = *counter.borrow();
        *counter.borrow_mut() = current + 1;
        current + 1
    })
}

pub fn next_collateral_id() -> u64 {
    COLLATERAL_COUNTER.with(|counter| {
        let current = *counter.borrow();
        *counter.borrow_mut() = current + 1;
        current + 1
    })
}

// Helper function to get NFT by token ID
pub fn get_nft_by_token_id(token_id: u64) -> Option<RWANFTData> {
    RWA_NFTS.with(|nfts| nfts.borrow().get(&token_id))
}

// Helper function to get collateral by ID
pub fn get_collateral_by_id(collateral_id: u64) -> Option<CollateralRecord> {
    COLLATERAL_RECORDS.with(|records| records.borrow().get(&collateral_id))
}
