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
    Bool(bool),
    Principal(Principal),
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

// Storage Statistics
#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct StorageStats {
    pub total_nfts: u64,
    pub total_collateral_records: u64,
    pub total_audit_logs: u64,
    pub nft_token_counter: u64,
    pub collateral_counter: u64,
    pub audit_log_counter: u64,
}

// Audit Log structure
#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct AuditLog {
    pub timestamp: u64,
    pub caller: Principal,
    pub action: String,
    pub details: String,
    pub success: bool,
}

// Canister Configuration
#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct CanisterConfig {
    pub admin_principals: Vec<Principal>,
    pub loan_manager_principal: Option<Principal>,
    pub max_nft_per_user: u64,
    pub min_collateral_value: u64,
    pub max_collateral_value: u64,
    pub emergency_stop: bool,
    pub maintenance_mode: bool,
}

impl Default for CanisterConfig {
    fn default() -> Self {
        Self {
            admin_principals: vec![],
            loan_manager_principal: None,
            max_nft_per_user: 100,
            min_collateral_value: 100_000_000, // 100M IDR
            max_collateral_value: 10_000_000_000, // 10B IDR
            emergency_stop: false,
            maintenance_mode: false,
        }
    }
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

// Implement Storable for AuditLog
impl Storable for AuditLog {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(candid::encode_one(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        candid::decode_one(&bytes).unwrap()
    }

    const BOUND: Bound = Bound::Unbounded;
}

// Implement Storable for CanisterConfig
impl Storable for CanisterConfig {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(candid::encode_one(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        candid::decode_one(&bytes).unwrap()
    }

    const BOUND: Bound = Bound::Unbounded;
}
