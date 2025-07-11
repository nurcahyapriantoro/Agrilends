use candid::{CandidType, Deserialize, Principal, Encode, Decode};
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
    pub total_loans: u64,
    pub total_users: u64,
    pub memory_usage_bytes: u64,
}

impl Default for StorageStats {
    fn default() -> Self {
        Self {
            total_nfts: 0,
            total_loans: 0,
            total_users: 0,
            memory_usage_bytes: 0,
        }
    }
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
    pub admins: Vec<Principal>, // Changed from admin_principals to match helpers.rs
    pub loan_manager_principal: Option<Principal>,
    pub max_nft_per_user: u64,
    pub min_collateral_value: u64,
    pub max_collateral_value: u64,
    pub emergency_stop: bool,
    pub maintenance_mode: bool, // Keep only one maintenance mode field
    pub min_deposit_amount: u64,
    pub max_utilization_rate: f64,
    pub emergency_reserve_ratio: f64,
    pub created_at: u64,
    pub updated_at: u64,
}

impl Default for CanisterConfig {
    fn default() -> Self {
        Self {
            admins: vec![],
            loan_manager_principal: None,
            max_nft_per_user: 100,
            min_collateral_value: 100_000_000, // 100M IDR
            max_collateral_value: 10_000_000_000, // 10B IDR
            emergency_stop: false,
            maintenance_mode: false,
            min_deposit_amount: 1_000_000, // 1M satoshi
            max_utilization_rate: 0.8, // 80%
            emergency_reserve_ratio: 0.2, // 20%
            created_at: 0,
            updated_at: 0,
        }
    }
}

// Loan Lifecycle Types
#[derive(CandidType, Deserialize, Clone, Debug, PartialEq)]
pub enum LoanStatus {
    PendingApplication, // Menunggu data agunan dan valuasi
    PendingApproval,    // Menunggu persetujuan dari peminjam
    Approved,           // Disetujui, menunggu pencairan dana
    Active,             // Dana sudah cair, pinjaman aktif
    Repaid,             // Lunas
    Defaulted,          // Gagal bayar
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct Loan {
    pub id: u64,
    pub borrower: Principal,
    pub nft_id: u64,
    pub collateral_value_btc: u64, // Nilai agunan dalam satoshi ckBTC
    pub amount_requested: u64,      // Jumlah yang diminta dalam satoshi
    pub amount_approved: u64,       // Jumlah yang disetujui (mis. 60% dari nilai agunan)
    pub apr: u64,                   // Suku bunga per tahun, mis. 10
    pub status: LoanStatus,
    pub created_at: u64,
    pub due_date: Option<u64>,      // Tanggal jatuh tempo
    pub total_repaid: u64,          // Total yang sudah dibayar
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct LoanApplication {
    pub nft_id: u64,
    pub amount_requested: u64,
    pub commodity_type: String,
    pub quantity: u64,
    pub grade: String,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct CommodityPrice {
    pub price_per_unit: u64, // Harga per unit dalam satoshi
    pub currency: String,
    pub timestamp: u64,
}

impl Storable for CommodityPrice {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
    
    const BOUND: Bound = Bound::Unbounded;
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct NFTMetadata {
    pub valuation_idr: u64,
    pub commodity_type: String,
    pub quantity: u64,
    pub grade: String,
    pub warehouse_receipt_hash: String,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct ProtocolParameters {
    pub loan_to_value_ratio: u64, // Default 60%
    pub base_apr: u64,            // Default 10%
    pub max_loan_duration_days: u64, // Default 365 days
    pub grace_period_days: u64,   // Default 30 days
}

impl Default for ProtocolParameters {
    fn default() -> Self {
        Self {
            loan_to_value_ratio: 60,
            base_apr: 10,
            max_loan_duration_days: 365,
            grace_period_days: 30,
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

// Implement Storable for Loan
impl Storable for Loan {
    const BOUND: Bound = Bound::Unbounded;
    
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(candid::encode_one(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        candid::decode_one(&bytes).unwrap()
    }
}

// Implement Storable for ProtocolParameters
impl Storable for ProtocolParameters {
    const BOUND: Bound = Bound::Unbounded;
    
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(candid::encode_one(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        candid::decode_one(&bytes).unwrap()
    }
}

// Additional types for production features
#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct DisbursementRecord {
    pub loan_id: u64,
    pub borrower_btc_address: String,
    pub amount: u64,
    pub ckbtc_block_index: u64,
    pub disbursed_at: u64,
    pub disbursed_by: Principal,
}

impl Storable for DisbursementRecord {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
    
    const BOUND: Bound = Bound::Unbounded;
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct RepaymentRecord {
    pub loan_id: u64,
    pub payer: Principal,
    pub amount: u64,
    pub ckbtc_block_index: u64,
    pub timestamp: u64,
}

impl Storable for RepaymentRecord {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
    
    const BOUND: Bound = Bound::Unbounded;
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct LiquidityPool {
    pub total_liquidity: u64,
    pub available_liquidity: u64,
    pub total_borrowed: u64,
    pub total_repaid: u64,
    pub utilization_rate: u64,
    pub total_investors: u64,
    pub apy: u64,
    pub created_at: u64,
    pub updated_at: u64,
}

impl Storable for LiquidityPool {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
    
    const BOUND: Bound = Bound::Unbounded;
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct InvestorBalance {
    pub investor: Principal,
    pub balance: u64,
    pub total_deposited: u64,
    pub total_withdrawn: u64,
    pub deposits: Vec<DepositRecord>,
    pub withdrawals: Vec<WithdrawalRecord>,
    pub first_deposit_at: u64,
    pub last_activity_at: u64,
}

impl Storable for InvestorBalance {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
    
    const BOUND: Bound = Bound::Unbounded;
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct DepositRecord {
    pub investor: Principal,
    pub amount: u64,
    pub ckbtc_block_index: u64,
    pub timestamp: u64,
}

impl Storable for DepositRecord {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
    
    const BOUND: Bound = Bound::Unbounded;
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct WithdrawalRecord {
    pub investor: Principal,
    pub amount: u64,
    pub ckbtc_block_index: u64,
    pub timestamp: u64,
}

impl Storable for WithdrawalRecord {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
    
    const BOUND: Bound = Bound::Unbounded;
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct ProcessedTransaction {
    pub tx_id: u64,
    pub processed_at: u64,
    pub processor: Principal,
}

impl Storable for ProcessedTransaction {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
    
    const BOUND: Bound = Bound::Unbounded;
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct ProductionHealthStatus {
    pub is_healthy: bool,
    pub emergency_stop: bool,
    pub maintenance_mode: bool,
    pub oracle_status: bool,
    pub ckbtc_integration: bool,
    pub memory_usage: u64,
    pub total_loans: u64,
    pub active_loans: u64,
    pub last_heartbeat: u64,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct CommodityPriceData {
    pub commodity_type: String,
    pub price_per_unit: u64,
    pub currency: String,
    pub timestamp: u64,
    pub source: String,
}

#[derive(CandidType, Deserialize, Default, Clone, Debug)]
pub struct PoolStats {
    pub total_liquidity: u64,
    pub available_liquidity: u64,
    pub total_borrowed: u64,
    pub total_repaid: u64,
    pub utilization_rate: f64,
    pub total_investors: u64,
    pub apy: f64,
}

#[derive(CandidType, Deserialize, Default, Clone, Debug)]
pub struct InvestorTransactionHistory {
    pub deposits: Vec<DepositRecord>,
    pub withdrawals: Vec<WithdrawalRecord>,
}

#[derive(CandidType, Deserialize, Default, Clone, Debug)]
pub struct PoolHealthMetrics {
    pub total_value_locked: u64,
    pub active_loans: u64,
    pub defaulted_loans: u64,
    pub average_deposit_size: u64,
    pub active_investors: u64,
}

#[derive(CandidType, Deserialize, Default, Clone, Debug)]
pub struct PoolConfiguration {
    pub min_deposit_amount: u64,
    pub max_utilization_rate: f64,
    pub emergency_reserve_ratio: f64,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct PriceFetchRecord {
    pub commodity_type: String,
    pub last_fetch_time: u64,
    pub fetch_count: u64,
}

impl Storable for PriceFetchRecord {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
    
    const BOUND: Bound = Bound::Unbounded;
}

// StorageStats already defined above at line 94 - removing duplicate

// Add missing emergency configuration
#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct EmergencyConfig {
    pub max_single_loan_percentage: u64, // Max percentage of total liquidity for single loan
    pub max_utilization_rate: u64,       // Max utilization rate before emergency
    pub min_reserve_ratio: u64,          // Minimum reserve ratio to maintain
    pub emergency_contact: Option<Principal>, // Emergency contact principal
}

impl Default for EmergencyConfig {
    fn default() -> Self {
        Self {
            max_single_loan_percentage: 80, // 80% max
            max_utilization_rate: 95,       // 95% max
            min_reserve_ratio: 15,          // 15% min reserve
            emergency_contact: None,
        }
    }
}
