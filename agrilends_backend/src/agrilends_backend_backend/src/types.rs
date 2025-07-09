use candid::{CandidType, Deserialize, Principal};
use ic_stable_structures::{Storable, storable::Bound};
use std::borrow::Cow;

// Account type for ICRC-7 compliance
#[derive(CandidType, Deserialize, Clone, Debug, PartialEq)]
pub struct Account {
    pub owner: Principal,
    pub subaccount: Option<Vec<u8>>,
}

// Metadata value type for NFT metadata
#[derive(CandidType, Deserialize, Clone, Debug, PartialEq)]
pub enum MetadataValue {
    Text(String),
    Blob(Vec<u8>),
    Nat(u64),
    Int(i64),
}

// Transfer request for ICRC-7 compliance
#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct TransferRequest {
    pub from: Option<Account>,
    pub to: Account,
    pub token_id: u64,
    pub memo: Option<Vec<u8>>,
    pub created_at_time: Option<u64>,
}

// Transfer result for ICRC-7 compliance
#[derive(CandidType, Deserialize, Clone, Debug)]
pub enum TransferResult {
    Ok,
    Err(String),
}

// RWA NFT Data structure
#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct RWANFTData {
    pub token_id: u64,
    pub owner: Principal,
    pub metadata: Vec<(String, MetadataValue)>,
    pub created_at: u64,
    pub updated_at: u64,
    pub is_locked: bool,
    pub loan_id: Option<u64>,
}

// RWA NFT Result type
#[derive(CandidType, Deserialize)]
pub enum RWANFTResult {
    Ok(RWANFTData),
    Err(String),
}

// Collateral Status
#[derive(CandidType, Deserialize, Clone, Debug, PartialEq)]
pub enum CollateralStatus {
    Available,
    Locked,
    Released,
    Liquidated,
}

// Collateral Record
#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct CollateralRecord {
    pub collateral_id: u64,
    pub nft_token_id: u64,
    pub owner: Principal,
    pub loan_id: Option<u64>,
    pub valuation_idr: u64,
    pub asset_description: String,
    pub legal_doc_hash: String,
    pub status: CollateralStatus,
    pub created_at: u64,
    pub updated_at: u64,
}

// NFT Statistics
#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct NFTStats {
    pub total_nfts: u64,
    pub locked_nfts: u64,
    pub available_collateral: u64,
    pub liquidated_collateral: u64,
}

// Implement Storable for RWANFTData
impl Storable for RWANFTData {
    const BOUND: Bound = Bound::Unbounded;
    
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(candid::encode_one(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        candid::decode_one(&bytes).unwrap()
    }
}

// Implement Storable for CollateralRecord
impl Storable for CollateralRecord {
    const BOUND: Bound = Bound::Unbounded;
    
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(candid::encode_one(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        candid::decode_one(&bytes).unwrap()
    }
}
