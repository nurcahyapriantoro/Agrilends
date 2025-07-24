use candid::{CandidType, Deserialize, Principal, Encode, Decode};
use ic_stable_structures::{Storable, storable::Bound};
use ic_cdk::api::time;
use std::borrow::Cow;

// Constants
pub const MAX_METADATA_SIZE: usize = 1024;
pub const MAX_COMMODITY_NAME_LENGTH: usize = 50;
pub const MAX_GRADE_LENGTH: usize = 20;
pub const BASIS_POINTS_SCALE: u64 = 10000; // 100% = 10000 basis points
pub const DEFAULT_LOAN_DURATION_DAYS: u64 = 365;
pub const DEFAULT_GRACE_PERIOD_DAYS: u64 = 30;
pub const MIN_COLLATERAL_VALUE_SATOSHI: u64 = 100_000; // 0.001 BTC
pub const MAX_COLLATERAL_VALUE_SATOSHI: u64 = 100_000_000; // 1 BTC

// Standardized Result Types
pub type AgrilendsResult<T> = Result<T, AgrilendsError>;
pub type AsyncResult<T> = std::result::Result<T, String>;

// Scalability and Load Balancing Support Types
#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct ShardMetrics {
    pub avg_response_time_ms: u64,
    pub total_requests: u64,
    pub error_count: u64,
    pub last_request_time: u64,
    pub cpu_utilization: f64,
    pub memory_utilization: f64,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct CommodityStats {
    pub commodity_type: String,
    pub total_volume: u64,
    pub avg_price: u64,
    pub loan_count: u64,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct TimeSeriesPoint {
    pub timestamp: u64,
    pub value: f64,
    pub metric_type: String,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct PaymentRecord {
    pub payment_id: u64,
    pub loan_id: u64,
    pub amount: u64,
    pub payment_type: PaymentType,
    pub timestamp: u64,
    pub status: PaymentStatus,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub enum PaymentStatus {
    Pending,
    Completed,
    Failed,
    Cancelled,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct TransactionRecord {
    pub transaction_id: u64,
    pub user_id: Principal,
    pub transaction_type: TransactionType,
    pub amount: u64,
    pub timestamp: u64,
    pub status: TransactionStatus,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub enum TransactionType {
    Deposit,
    Withdrawal,
    LoanDisbursement,
    LoanRepayment,
    ProtocolFee,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub enum TransactionStatus {
    Pending,
    Completed,
    Failed,
    Cancelled,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct FarmerPerformanceMetrics {
    pub total_loans: u64,
    pub repayment_rate: f64,
    pub average_loan_duration: u64,
    pub credit_score: f64,
    pub risk_level: RiskLevel,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct Investment {
    pub investment_id: u64,
    pub investor_id: Principal,
    pub amount: u64,
    pub timestamp: u64,
    pub expected_return: f64,
    pub actual_return: Option<f64>,
}

// Advanced NFT Summary for dashboards
#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct NFTSummary {
    pub token_id: u64,
    pub owner: Principal,
    pub commodity_type: String,
    pub grade: String,
    pub quantity_kg: u64,
    pub valuation_satoshi: u64,
    pub status: CollateralStatus,
    pub created_at: u64,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub enum AgrilendsError {
    NotFound(String),
    Unauthorized,
    InvalidInput(String),
    SystemError(String),
    StorageError(String),
    NetworkError(String),
    InsufficientFunds,
    EmergencyStop,
    MaintenanceMode,
}

impl std::fmt::Display for AgrilendsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AgrilendsError::NotFound(msg) => write!(f, "Not found: {}", msg),
            AgrilendsError::Unauthorized => write!(f, "Unauthorized access"),
            AgrilendsError::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
            AgrilendsError::SystemError(msg) => write!(f, "System error: {}", msg),
            AgrilendsError::StorageError(msg) => write!(f, "Storage error: {}", msg),
            AgrilendsError::NetworkError(msg) => write!(f, "Network error: {}", msg),
            AgrilendsError::InsufficientFunds => write!(f, "Insufficient funds"),
            AgrilendsError::EmergencyStop => write!(f, "Emergency stop activated"),
            AgrilendsError::MaintenanceMode => write!(f, "System in maintenance mode"),
        }
    }
}

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
    pub total_collateral: u64,        // Tambahkan field yang mungkin diperlukan
    pub total_liquidity: u64,         // Tambahkan field yang mungkin diperlukan
}

impl Default for StorageStats {
    fn default() -> Self {
        Self {
            total_nfts: 0,
            total_loans: 0,
            total_users: 0,
            memory_usage_bytes: 0,
            total_collateral: 0,
            total_liquidity: 0,
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
    pub max_utilization_rate: u64,       // Ubah dari f64 ke u64
    pub emergency_reserve_ratio: u64,    // Ubah dari f64 ke u64
    pub created_at: u64,
    pub updated_at: u64,
    // Treasury configuration
    pub treasury_min_balance: u64,
    pub emergency_reserve_percentage: u64,
    pub auto_top_up_percentage: u64,
    pub cycle_monitoring_interval: u64,
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
            max_utilization_rate: 8000, // 80% in basis points - PERBAIKI dari 0.8
            emergency_reserve_ratio: 2000, // 20% in basis points - PERBAIKI dari 0.2
            created_at: 0,
            updated_at: 0,
            // Treasury defaults
            treasury_min_balance: 100_000, // 0.001 BTC minimum
            emergency_reserve_percentage: 20, // 20%
            auto_top_up_percentage: 150, // 150%
            cycle_monitoring_interval: 3600, // 1 hour
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
    pub repayment_history: Vec<Payment>, // Riwayat pembayaran
    pub last_payment_date: Option<u64>,  // Tanggal pembayaran terakhir
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
        Cow::Owned(candid::encode_one(self).unwrap())  // Ubah dari Encode!
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        candid::decode_one(&bytes).unwrap()            // Ubah dari Decode!
    }
    
    const BOUND: Bound = Bound::Unbounded;
}

// Struktur untuk repayment summary dan detail
#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct LoanRepaymentSummary {
    pub loan_id: u64,
    pub borrower: Principal,
    pub total_debt: u64,           // Total utang (pokok + bunga akumulasi)
    pub principal_outstanding: u64,  // Sisa pokok
    pub interest_outstanding: u64,   // Sisa bunga
    pub total_repaid: u64,          // Total yang sudah dibayar
    pub remaining_balance: u64,     // Sisa yang harus dibayar
    pub next_payment_due: Option<u64>, // Tanggal pembayaran berikutnya
    pub is_overdue: bool,           // Apakah terlambat
    pub days_overdue: u64,          // Jumlah hari terlambat
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct RepaymentPlan {
    pub loan_id: u64,
    pub total_amount_due: u64,
    pub principal_amount: u64,
    pub interest_amount: u64,
    pub protocol_fee: u64,
    pub due_date: u64,
    pub minimum_payment: u64,
}

// Response structure untuk repayment
#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct RepaymentResponse {
    pub success: bool,
    pub message: String,
    pub transaction_id: Option<String>,
    pub new_loan_status: LoanStatus,
    pub remaining_balance: u64,
    pub collateral_released: bool,
}

// Additional comprehensive types untuk production loan repayment features
#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct ComprehensiveRepaymentAnalytics {
    pub total_loans_count: u64,
    pub active_loans_count: u64,
    pub repaid_loans_count: u64,
    pub defaulted_loans_count: u64,
    pub total_principal_paid: u64,
    pub total_interest_paid: u64,
    pub total_fees_collected: u64,
    pub overdue_loans_count: u64,
    pub total_overdue_amount: u64,
    pub early_repayments_count: u64,
    pub average_repayment_time: u64, // in days
    pub current_timestamp: u64,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct LoanPerformanceMetrics {
    pub loan_id: u64,
    pub is_performing: bool,
    pub repayment_rate: u64, // percentage (0-100)
    pub payment_frequency: u64, // payments per month
    pub total_payments_made: u64,
    pub days_since_last_payment: u64,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct BatchRepaymentRequest {
    pub loan_id: u64,
    pub amount: u64,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct BatchRepaymentResult {
    pub loan_id: u64,
    pub success: bool,
    pub message: String,
    pub transaction_id: Option<String>,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct RepaymentStatistics {
    pub total_loans: u64,
    pub active_loans: u64,
    pub repaid_loans: u64,
    pub defaulted_loans: u64,
    pub total_repaid_amount: u64,
    pub total_outstanding_amount: u64,
    pub average_repayment_time: u64, // in days
}

// Repayment forecasting untuk financial planning
#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct RepaymentForecast {
    pub month: u64,
    pub forecast_date: u64,
    pub projected_interest: u64,
    pub projected_total_debt: u64,
    pub projected_remaining_balance: u64,
    pub recommended_payment: u64,
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
        Cow::Owned(candid::encode_one(self).unwrap())  // Ubah dari Encode!
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        candid::decode_one(&bytes).unwrap()            // Ubah dari Decode!
    }
    
    const BOUND: Bound = Bound::Unbounded;
}

// Payment structure untuk tracking individual payments
#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct Payment {
    pub amount: u64,
    pub timestamp: u64,
    pub payment_type: PaymentType,
    pub transaction_id: Option<String>,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub enum PaymentType {
    Principal,      // Pembayaran pokok
    Interest,       // Pembayaran bunga
    Mixed,          // Campuran pokok dan bunga
}

impl Storable for Payment {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(candid::encode_one(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        candid::decode_one(&bytes).unwrap()
    }
    
    const BOUND: Bound = Bound::Unbounded;
}

// Payment breakdown untuk menunjukkan alokasi pembayaran
#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct PaymentBreakdown {
    pub principal_amount: u64,
    pub interest_amount: u64,
    pub protocol_fee_amount: u64,
    pub penalty_amount: u64, // Late payment penalty
    pub total_amount: u64,
}

impl Default for PaymentBreakdown {
    fn default() -> Self {
        Self {
            principal_amount: 0,
            interest_amount: 0,
            protocol_fee_amount: 0,
            penalty_amount: 0,
            total_amount: 0,
        }
    }
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct RepaymentRecord {
    pub loan_id: u64,
    pub payer: Principal,
    pub amount: u64,
    pub ckbtc_block_index: u64,
    pub timestamp: u64,
    pub payment_breakdown: PaymentBreakdown,
}

impl Storable for RepaymentRecord {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(candid::encode_one(self).unwrap()) // Ubah dari Encode!(self).unwrap()
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        candid::decode_one(&bytes).unwrap() // Ubah dari Decode!(bytes.as_ref(), Self).unwrap()
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
        Cow::Owned(candid::encode_one(self).unwrap()) // Ubah dari Encode!(self).unwrap()
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        candid::decode_one(&bytes).unwrap() // Ubah dari Decode!(bytes.as_ref(), Self).unwrap()
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
        Cow::Owned(candid::encode_one(self).unwrap()) // Ubah dari Encode!(self).unwrap()
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        candid::decode_one(&bytes).unwrap() // Ubah dari Decode!(bytes.as_ref(), Self).unwrap()
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
        Cow::Owned(candid::encode_one(self).unwrap()) // Ubah dari Encode!(self).unwrap()
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        candid::decode_one(&bytes).unwrap() // Ubah dari Decode!(bytes.as_ref(), Self).unwrap()
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
        Cow::Owned(candid::encode_one(self).unwrap()) // Ubah dari Encode!(self).unwrap()
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        candid::decode_one(&bytes).unwrap() // Ubah dari Decode!(bytes.as_ref(), Self).unwrap()
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
        Cow::Owned(candid::encode_one(self).unwrap()) // Ubah dari Encode!(self).unwrap()
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        candid::decode_one(&bytes).unwrap() // Ubah dari Decode!(bytes.as_ref(), Self).unwrap()
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

// Oracle-related Types
#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct CommodityPriceData {
    pub commodity_type: String,
    pub price_per_unit: u64,
    pub currency: String,
    pub timestamp: u64,
    pub source: String,
    pub confidence_score: u64, // 0-100
    pub is_stale: bool,
    pub fetch_attempt_count: u32,
    pub last_successful_fetch: u64,
}

impl Storable for CommodityPriceData {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(candid::encode_one(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        candid::decode_one(&bytes).unwrap()
    }
    
    const BOUND: Bound = Bound::Unbounded;
}

// Price Fetch Record untuk tracking
#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct PriceFetchRecord {
    pub commodity_id: String,
    pub last_fetch_timestamp: u64,
    pub fetch_count: u32,
    pub success_count: u32,
    pub failure_count: u32,
    pub last_error: Option<String>,
    pub average_response_time: u64, // in milliseconds
    pub rate_limit_reset: u64,
}

impl Storable for PriceFetchRecord {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(candid::encode_one(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        candid::decode_one(&bytes).unwrap()
    }
    
    const BOUND: Bound = Bound::Unbounded;
}

// Oracle Configuration
#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct OracleConfig {
    pub enabled_commodities: Vec<String>,
    pub api_endpoints: Vec<(String, String)>, // (commodity_id, api_url)
    pub fetch_interval_seconds: u64,
    pub stale_threshold_seconds: u64,
    pub max_fetch_retries: u32,
    pub confidence_threshold: u64, // Minimum confidence score
    pub rate_limit_per_commodity: u32, // Max fetches per hour
    pub emergency_mode: bool,
    pub backup_prices: Vec<(String, u64)>, // Emergency fallback prices
}

impl Default for OracleConfig {
    fn default() -> Self {
        Self {
            enabled_commodities: vec![
                "rice".to_string(),
                "corn".to_string(), 
                "wheat".to_string(),
                "soybean".to_string(),
                "sugar".to_string(),
            ],
            api_endpoints: vec![
                ("rice".to_string(), "https://api.hargapangan.id/tabel/pasar/provinsi/komoditas/33/1".to_string()),
                ("corn".to_string(), "https://api.hargapangan.id/tabel/pasar/provinsi/komoditas/33/2".to_string()),
                ("wheat".to_string(), "https://api.hargapangan.id/tabel/pasar/provinsi/komoditas/33/3".to_string()),
            ],
            fetch_interval_seconds: 3600, // 1 hour
            stale_threshold_seconds: 86400, // 24 hours
            max_fetch_retries: 3,
            confidence_threshold: 70,
            rate_limit_per_commodity: 24, // Max 24 fetches per hour per commodity
            emergency_mode: false,
            backup_prices: vec![
                ("rice".to_string(), 15000), // IDR per kg - fallback price
                ("corn".to_string(), 8000),
                ("wheat".to_string(), 12000),
            ],
        }
    }
}

impl Storable for OracleConfig {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(candid::encode_one(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        candid::decode_one(&bytes).unwrap()
    }
    
    const BOUND: Bound = Bound::Unbounded;
}

// Oracle Statistics
#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct OracleStatistics {
    pub total_fetches: u64,
    pub successful_fetches: u64,
    pub failed_fetches: u64,
    pub average_response_time: u64,
    pub uptime_percentage: f64,
    pub commodities_tracked: u64,
    pub stale_prices_count: u64,
    pub last_update: u64,
    pub price_volatility: Vec<(String, f64)>, // (commodity, volatility_percentage)
}

impl Storable for OracleStatistics {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(candid::encode_one(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        candid::decode_one(&bytes).unwrap()
    }
    
    const BOUND: Bound = Bound::Unbounded;
}

// Price Alert Configuration
#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct PriceAlert {
    pub commodity_id: String,
    pub threshold_type: PriceThresholdType,
    pub threshold_value: u64,
    pub is_active: bool,
    pub created_by: Principal,
    pub created_at: u64,
    pub triggered_at: Option<u64>,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub enum PriceThresholdType {
    Above(u64),     // Alert when price goes above value
    Below(u64),     // Alert when price goes below value
    Change(u64),    // Alert when price changes by percentage (basis points)
}

impl Storable for PriceAlert {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(candid::encode_one(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        candid::decode_one(&bytes).unwrap()
    }
    
    const BOUND: Bound = Bound::Unbounded;
}

impl Storable for ProductionHealthStatus {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(candid::encode_one(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        candid::decode_one(&bytes).unwrap()
    }
    
    const BOUND: Bound = Bound::Unbounded;
}

impl Storable for NFTMetadata {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(candid::encode_one(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        candid::decode_one(&bytes).unwrap()
    }
    
    const BOUND: Bound = Bound::Unbounded;
}

impl Storable for LoanApplication {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(candid::encode_one(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        candid::decode_one(&bytes).unwrap()
    }
    
    const BOUND: Bound = Bound::Unbounded;
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct InvestorTransactionHistory {
    pub investor: Principal,
    pub deposits: Vec<DepositRecord>,
    pub withdrawals: Vec<WithdrawalRecord>,
    pub total_deposited: u64,
    pub total_withdrawn: u64,
    pub net_balance: u64,
    pub first_activity: u64,
    pub last_activity: u64,
}

impl Storable for InvestorTransactionHistory {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(candid::encode_one(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        candid::decode_one(&bytes).unwrap()
    }
    
    const BOUND: Bound = Bound::Unbounded;
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct PoolHealthMetrics {
    pub total_liquidity: u64,
    pub available_liquidity: u64,
    pub utilized_liquidity: u64,
    pub utilization_rate: u64, // Basis points
    pub total_investors: u64,
    pub active_investors: u64,
    pub total_loans: u64,
    pub active_loans: u64,
    pub default_rate: u64, // Basis points
    pub avg_loan_size: u64,
    pub pool_health_score: u64, // 0-100
    pub last_updated: u64,
}

impl Storable for PoolHealthMetrics {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(candid::encode_one(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        candid::decode_one(&bytes).unwrap()
    }
    
    const BOUND: Bound = Bound::Unbounded;
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct PoolConfiguration {
    pub min_deposit_amount: u64,
    pub max_deposit_amount: u64,
    pub min_withdrawal_amount: u64,
    pub max_utilization_rate: u64, // Basis points
    pub emergency_reserve_ratio: u64, // Basis points
    pub base_apy: u64, // Basis points
    pub performance_fee: u64, // Basis points
    pub withdrawal_fee: u64, // Basis points
    pub is_paused: bool,
    pub created_at: u64,
    pub updated_at: u64,
}

impl Storable for PoolConfiguration {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(candid::encode_one(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        candid::decode_one(&bytes).unwrap()
    }
    
    const BOUND: Bound = Bound::Unbounded;
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct PriceFetchRecord {
    pub commodity_type: String,
    pub last_fetch_time: u64,
    pub fetch_count: u64,
    pub last_successful_fetch: u64,
    pub consecutive_failures: u64,
}

impl Storable for PriceFetchRecord {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(candid::encode_one(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        candid::decode_one(&bytes).unwrap()
    }
    
    const BOUND: Bound = Bound::Unbounded;
}

impl Loan {
    pub fn is_active(&self) -> bool {
        self.status == LoanStatus::Active
    }
    
    pub fn is_overdue(&self) -> bool {
        if let Some(due_date) = self.due_date {
            time() > due_date
        } else {
            false
        }
    }
    
    pub fn remaining_balance(&self) -> u64 {
        self.amount_approved.saturating_sub(self.total_repaid)
    }
}

impl InvestorBalance {
    pub fn net_balance(&self) -> u64 {
        self.total_deposited.saturating_sub(self.total_withdrawn)
    }
    
    pub fn is_active(&self) -> bool {
        self.balance > 0
    }
}

impl LiquidityPool {
    pub fn calculate_utilization_rate(&self) -> u64 {
        if self.total_liquidity == 0 {
            0
        } else {
            (self.total_borrowed * 10000) / self.total_liquidity // Basis points
        }
    }
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct PoolStats {
    pub total_liquidity: u64,
    pub available_liquidity: u64,
    pub total_borrowed: u64,
    pub total_repaid: u64,
    pub utilization_rate: u64, // Basis points
    pub total_investors: u64,
    pub apy: u64, // Basis points
    pub created_at: u64,
    pub updated_at: u64,
}

impl Storable for PoolStats {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(candid::encode_one(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        candid::decode_one(&bytes).unwrap()
    }
    
    const BOUND: Bound = Bound::Unbounded;
}

impl PoolStats {
    pub fn calculate_apy(&self) -> u64 {
        // Implementasi kalkulasi APY berdasarkan utilization rate
        if self.utilization_rate > 8000 { // 80%
            1200 // 12% APY
        } else if self.utilization_rate > 5000 { // 50%
            1000 // 10% APY
        } else {
            800 // 8% APY
        }
    }
    
    pub fn is_healthy(&self) -> bool {
        self.utilization_rate < 9000 && self.available_liquidity > 0
    }
}

// Liquidation-specific types
#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct LiquidationRecord {
    pub loan_id: u64,
    pub liquidated_at: u64,
    pub liquidated_by: Principal,
    pub collateral_nft_id: u64,
    pub outstanding_debt: u64,
    pub principal_loss: u64,
    pub collateral_value: u64,
    pub liquidation_reason: LiquidationReason,
    pub ecdsa_signature: Option<String>,
    pub liquidation_wallet: Principal,
    pub processing_fee: u64,
    pub recovery_expected: u64,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub enum LiquidationReason {
    GracePeriodExpired,          // Loan overdue beyond grace period (sesuai README)
    LongTermDefault,             // Long-term default (> 90 days overdue)
    UndercollateralizationRisk,  // Collateral-to-debt ratio too low
    EmergencyLiquidation,        // Emergency liquidation by admin
    AutomatedLiquidation,        // Triggered by automated system
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct LiquidationEligibilityCheck {
    pub loan_id: u64,
    pub is_eligible: bool,
    pub reason: String,              // Detailed explanation
    pub days_overdue: u64,
    pub health_ratio: f64,          // Collateral value / Outstanding debt
    pub grace_period_expired: bool,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct LiquidationResult {
    pub loan_id: u64,
    pub success: bool,
    pub message: String,
    pub error: Option<String>,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct LiquidationStatistics {
    pub total_liquidations: u64,
    pub total_liquidated_debt: u64,
    pub total_liquidated_collateral_value: u64,
    pub liquidations_this_month: u64,
    pub recovery_rate: f64,
    pub loans_eligible_for_liquidation: u64,
    pub average_time_to_liquidation: u64,
    pub liquidation_success_rate: f64,
}

impl Storable for LiquidationRecord {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        std::borrow::Cow::Owned(candid::encode_one(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        candid::decode_one(&bytes).unwrap()
    }
    
    const BOUND: ic_stable_structures::storable::Bound = ic_stable_structures::storable::Bound::Bounded {
        max_size: 1000,
        is_fixed_size: false,
    };
}

// Enhanced liquidation analysis types
#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct LiquidationRiskAssessment {
    pub loan_id: u64,
    pub risk_level: String,
    pub health_ratio: f64,
    pub days_until_liquidation: u64,
    pub estimated_loss: u64,
    pub recommended_action: String,
    pub assessment_timestamp: u64,
    pub collateral_value: u64,
    pub outstanding_debt: u64,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct LiquidationMetrics {
    pub total_liquidations: u64,
    pub total_liquidated_debt: u64,
    pub total_liquidated_collateral_value: u64,
    pub liquidations_this_month: u64,
    pub recovery_rate: f64,
    pub loans_eligible_for_liquidation: u64,
    pub average_liquidation_time: u64,
    pub liquidation_success_rate: f64,
    pub total_processing_fees_collected: u64,
    pub timestamp: u64,
}

// ========== GOVERNANCE TYPES ==========

#[derive(CandidType, Deserialize, Clone, Debug)]
pub enum ProposalType {
    ProtocolParameterUpdate,
    AdminRoleUpdate,
    CanisterUpgrade,
    EmergencyAction,
    SystemConfiguration,
    TreasuryManagement,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub enum ProposalStatus {
    Pending,
    Active,
    Approved,
    Rejected,
    Executed,
    Expired,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub enum VoteChoice {
    Yes,
    No,
    Abstain,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct Proposal {
    pub id: u64,
    pub proposer: Principal,
    pub proposal_type: ProposalType,
    pub title: String,
    pub description: String,
    pub execution_payload: Option<Vec<u8>>, // Serialized execution data
    pub created_at: u64,
    pub voting_deadline: u64,
    pub execution_deadline: u64,
    pub status: ProposalStatus,
    pub yes_votes: u64,
    pub no_votes: u64,
    pub abstain_votes: u64,
    pub total_voting_power: u64,
    pub quorum_threshold: u64,
    pub approval_threshold: u64, // Percentage in basis points
    pub executed_at: Option<u64>,
    pub executed_by: Option<Principal>,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct Vote {
    pub voter: Principal,
    pub proposal_id: u64,
    pub choice: VoteChoice,
    pub voting_power: u64,
    pub voted_at: u64,
    pub reason: Option<String>,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct GovernanceConfig {
    pub voting_period_seconds: u64,
    pub execution_delay_seconds: u64,
    pub proposal_threshold: u64, // Minimum voting power to create proposal
    pub quorum_threshold: u64, // Minimum participation for valid vote
    pub approval_threshold: u64, // Percentage needed for approval (basis points)
    pub max_proposals_per_user: u64,
    pub governance_token_canister: Option<Principal>,
    pub emergency_action_threshold: u64, // Lower threshold for emergency actions
    pub treasury_action_threshold: u64, // Higher threshold for treasury actions
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct ProtocolParameter {
    pub key: String,
    pub current_value: u64,
    pub proposed_value: Option<u64>,
    pub value_type: ParameterType,
    pub min_value: Option<u64>,
    pub max_value: Option<u64>,
    pub description: String,
    pub last_updated: u64,
    pub updated_by: Principal,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub enum ParameterType {
    Percentage, // Stored in basis points
    Amount,     // Absolute value
    Duration,   // Time in seconds
    Boolean,    // 0 or 1
    Principal,  // Principal as u64 hash
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct AdminRole {
    pub admin_principal: Principal,
    pub role_type: AdminRoleType,
    pub granted_at: u64,
    pub granted_by: Principal,
    pub expires_at: Option<u64>,
    pub permissions: Vec<Permission>,
    pub is_active: bool,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub enum AdminRoleType {
    SuperAdmin,
    ProtocolAdmin,
    TreasuryAdmin,
    RiskAdmin,
    LiquidationAdmin,
    OracleAdmin,
    EmergencyAdmin,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub enum Permission {
    ManageParameters,
    ManageAdmins,
    EmergencyStop,
    ManageTreasury,
    ManageLiquidation,
    ManageOracle,
    ViewMetrics,
    ExecuteProposals,
}

// Governance Results
pub type GovernanceResult<T> = Result<T, GovernanceError>;

#[derive(CandidType, Deserialize, Clone, Debug)]
pub enum GovernanceError {
    Unauthorized,
    ProposalNotFound,
    InvalidProposal,
    VotingClosed,
    AlreadyVoted,
    InsufficientVotingPower,
    QuorumNotMet,
    ProposalExpired,
    ExecutionFailed,
    InvalidParameter,
}

// Governance Statistics
#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct GovernanceStats {
    pub total_proposals: u64,
    pub active_proposals: u64,
    pub executed_proposals: u64,
    pub total_votes_cast: u64,
    pub total_voting_power: u64,
    pub average_participation_rate: u64, // Basis points
    pub last_proposal_id: u64,
}

// Storable implementations for governance types
impl Storable for Proposal {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        std::borrow::Cow::Owned(candid::encode_one(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        candid::decode_one(&bytes).unwrap()
    }
    
    const BOUND: ic_stable_structures::storable::Bound = ic_stable_structures::storable::Bound::Unbounded;
}

impl Storable for Vote {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        std::borrow::Cow::Owned(candid::encode_one(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        candid::decode_one(&bytes).unwrap()
    }
    
    const BOUND: ic_stable_structures::storable::Bound = ic_stable_structures::storable::Bound::Unbounded;
}

impl Storable for ProtocolParameter {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        std::borrow::Cow::Owned(candid::encode_one(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        candid::decode_one(&bytes).unwrap()
    }
    
    const BOUND: ic_stable_structures::storable::Bound = ic_stable_structures::storable::Bound::Unbounded;
}

impl Storable for AdminRole {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        std::borrow::Cow::Owned(candid::encode_one(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        candid::decode_one(&bytes).unwrap()
    }
    
    const BOUND: ic_stable_structures::storable::Bound = ic_stable_structures::storable::Bound::Unbounded;
}

impl CommodityPriceData {
    pub fn is_stale(&self, max_age_seconds: u64) -> bool {
        time() > self.timestamp + (max_age_seconds * 1_000_000_000)
    }
}

impl PoolHealthMetrics {
    pub fn calculate_health_score(&self) -> u64 {
        let mut score = 100;
        
        // Penalti untuk utilization rate tinggi
        if self.utilization_rate > 8000 {
            score -= 20;
        }
        
        // Penalti untuk default rate tinggi
        if self.default_rate > 500 { // 5%
            score -= 30;
        }
        
        // Bonus untuk diversifikasi investor
        if self.total_investors > 100 {
            score += 10;
        }
        
        score.min(100)
    }
}

// ========== TREASURY MANAGEMENT TYPES ==========

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct TreasuryState {
    pub balance_ckbtc: u64,
    pub total_fees_collected: u64,
    pub total_cycles_distributed: u64,
    pub last_cycle_distribution: u64,
    pub emergency_reserve: u64,
    pub created_at: u64,
    pub updated_at: u64,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct RevenueEntry {
    pub id: u64,
    pub source_loan_id: u64,
    pub amount: u64,
    pub revenue_type: RevenueType,
    pub source_canister: Principal,
    pub timestamp: u64,
    pub transaction_hash: Option<String>,
    pub status: TransactionStatus,
}

#[derive(CandidType, Deserialize, Clone, Debug, PartialEq)]
pub enum RevenueType {
    AdminFee,
    InterestShare,
    LiquidationPenalty,
    EarlyRepaymentFee,
    ProtocolFee,
    OtherRevenue(String),
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub enum TransactionStatus {
    Pending,
    Completed,
    Failed(String),
    Refunded,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct CanisterInfo {
    pub name: String,
    pub principal: Principal,
    pub canister_type: CanisterType,
    pub min_cycles_threshold: u64,
    pub max_cycles_limit: u64,
    pub priority: u8, // 1-10, 1 being highest priority
    pub last_top_up: u64,
    pub total_cycles_received: u64,
    pub is_active: bool,
    pub auto_top_up_enabled: bool,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub enum CanisterType {
    Core,           // Core business logic canisters
    Infrastructure, // Infrastructure support canisters
    Analytics,      // Analytics and reporting canisters
    Frontend,       // Frontend serving canisters
    Oracle,         // Oracle and external data canisters
    Backup,         // Backup and recovery canisters
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct CycleTransaction {
    pub id: u64,
    pub target_canister: Principal,
    pub canister_name: String,
    pub cycles_amount: u64,
    pub ckbtc_cost: u64,
    pub exchange_rate: f64, // ckBTC to cycles exchange rate
    pub timestamp: u64,
    pub status: TransactionStatus,
    pub initiated_by: Principal,
    pub reason: String,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct TreasuryStats {
    pub current_balance: u64,
    pub total_revenue_collected: u64,
    pub total_cycles_distributed: u64,
    pub emergency_reserve: u64,
    pub active_canisters_count: u32,
    pub last_distribution_time: u64,
    pub average_daily_revenue: u64,
    pub projected_runway_days: u32,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct CanisterCycleStatus {
    pub canister_info: CanisterInfo,
    pub current_cycles: u64,
    pub estimated_consumption_per_day: u64,
    pub days_remaining: u32,
    pub needs_top_up: bool,
    pub last_checked: u64,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct TreasuryHealthReport {
    pub overall_status: String, // "Healthy", "Warning", "Critical"
    pub current_balance: u64,
    pub emergency_reserve: u64,
    pub available_balance: u64,
    pub daily_burn_rate: u64,
    pub projected_runway_days: u32,
    pub active_canisters_count: u32,
    pub last_distribution: u64,
    pub recommendations: Vec<String>,
}

// ========== TREASURY STORABLE IMPLEMENTATIONS ==========

impl Storable for TreasuryState {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        std::borrow::Cow::Owned(candid::encode_one(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        candid::decode_one(&bytes).unwrap()
    }
    
    const BOUND: ic_stable_structures::storable::Bound = ic_stable_structures::storable::Bound::Unbounded;
}

impl Storable for RevenueEntry {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        std::borrow::Cow::Owned(candid::encode_one(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        candid::decode_one(&bytes).unwrap()
    }
    
    const BOUND: ic_stable_structures::storable::Bound = ic_stable_structures::storable::Bound::Unbounded;
}

impl Storable for CanisterInfo {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        std::borrow::Cow::Owned(candid::encode_one(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        candid::decode_one(&bytes).unwrap()
    }
    
    const BOUND: ic_stable_structures::storable::Bound = ic_stable_structures::storable::Bound::Unbounded;
}

impl Storable for CycleTransaction {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        std::borrow::Cow::Owned(candid::encode_one(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        candid::decode_one(&bytes).unwrap()
    }
    
    const BOUND: ic_stable_structures::storable::Bound = ic_stable_structures::storable::Bound::Unbounded;
}

impl Storable for TreasuryHealthReport {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        std::borrow::Cow::Owned(candid::encode_one(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        candid::decode_one(&bytes).unwrap()
    }
    
    const BOUND: ic_stable_structures::storable::Bound = ic_stable_structures::storable::Bound::Unbounded;
}

// ========== NOTIFICATION SYSTEM TYPES ==========

#[derive(CandidType, Deserialize, Clone, Debug, PartialEq)]
pub enum NotificationEvent {
    // Loan lifecycle events
    LoanApplicationSubmitted { loan_id: u64 },
    LoanOfferReady { loan_id: u64, amount: u64 },
    LoanApproved { loan_id: u64 },
    LoanDisbursed { loan_id: u64, amount: u64 },
    LoanRepaymentReceived { loan_id: u64, amount: u64, remaining_balance: u64 },
    LoanFullyRepaid { loan_id: u64 },
    LoanOverdue { loan_id: u64, days_overdue: u64 },
    LoanLiquidated { loan_id: u64, collateral_seized: Vec<u64> },
    
    // Collateral events
    CollateralMinted { nft_id: u64, commodity_type: String },
    CollateralEscrowed { nft_id: u64, loan_id: u64 },
    CollateralReleased { nft_id: u64, loan_id: u64 },
    CollateralLiquidated { nft_id: u64, sale_price: u64 },
    
    // Investment events
    LiquidityDeposited { amount: u64 },
    LiquidityWithdrawn { amount: u64 },
    InvestmentReturns { amount: u64, period: String },
    
    // Oracle and price events
    PriceAlert { commodity: String, old_price: u64, new_price: u64, change_percentage: f64 },
    OracleFailure { commodity: String, error: String },
    
    // Governance events
    ProposalCreated { proposal_id: u64, title: String },
    ProposalVoted { proposal_id: u64, vote: String },
    ProposalExecuted { proposal_id: u64, outcome: String },
    
    // System events
    MaintenanceScheduled { start_time: u64, duration_hours: u64 },
    EmergencyStop { reason: String },
    SystemResumed,
    
    // Security events
    SecurityAlert { event_type: String, severity: NotificationPriority },
    UnusualActivity { description: String },
    
    // Custom events
    Custom { event_type: String, data: std::collections::HashMap<String, String> },
}

#[derive(CandidType, Deserialize, Clone, Debug, PartialEq)]
pub enum NotificationPriority {
    Low,
    Normal,
    High,
    Critical,
    Emergency,
}

#[derive(CandidType, Deserialize, Clone, Debug, PartialEq)]
pub enum NotificationStatus {
    Pending,
    Delivered,
    Read,
    Acknowledged,
    Expired,
    Failed,
}

#[derive(CandidType, Deserialize, Clone, Debug, PartialEq)]
pub enum NotificationChannel {
    OnChain,
    Email, // For future integration
    Push,  // For future mobile integration
    SMS,   // For future integration
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct NotificationRecord {
    pub id: u64,
    pub recipient: Principal,
    pub event: NotificationEvent,
    pub title: String,
    pub message: String,
    pub priority: NotificationPriority,
    pub status: NotificationStatus,
    pub channels: Vec<NotificationChannel>,
    pub created_at: u64,
    pub delivered_at: Option<u64>,
    pub read_at: Option<u64>,
    pub acknowledged_at: Option<u64>,
    pub expires_at: Option<u64>,
    pub metadata: std::collections::HashMap<String, String>,
    pub retry_count: u8,
    pub last_retry_at: Option<u64>,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct NotificationTemplate {
    pub event_type: String,
    pub title_template: String,
    pub message_template: String,
    pub default_priority: NotificationPriority,
    pub default_channels: Vec<NotificationChannel>,
    pub variables: Vec<String>, // Template variables like {loan_id}, {amount}
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct NotificationSettings {
    pub user_id: Principal,
    pub enabled: bool,
    pub preferred_channels: Vec<NotificationChannel>,
    pub event_preferences: std::collections::HashMap<String, bool>, // Event type -> enabled
    pub quiet_hours_start: Option<u8>, // Hour 0-23
    pub quiet_hours_end: Option<u8>,
    pub max_notifications_per_day: Option<u32>,
    pub language: String, // For future i18n support
    pub timezone: String,
    pub email_address: Option<String>,
    pub phone_number: Option<String>,
    pub push_token: Option<String>,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct NotificationStats {
    pub total_notifications: u64,
    pub notifications_by_status: std::collections::HashMap<String, u64>,
    pub notifications_by_priority: std::collections::HashMap<String, u64>,
    pub notifications_by_event_type: std::collections::HashMap<String, u64>,
    pub average_delivery_time_ms: f64,
    pub delivery_success_rate: f64,
    pub unread_notifications_count: u64,
    pub active_users_with_notifications: u64,
    pub last_cleanup_time: u64,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct NotificationFilter {
    pub status: Option<NotificationStatus>,
    pub priority: Option<NotificationPriority>,
    pub event_types: Option<Vec<String>>,
    pub from_date: Option<u64>,
    pub to_date: Option<u64>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

// Result types for notifications
pub type NotificationResult = Result<NotificationRecord, String>;
pub type NotificationListResult = Result<Vec<NotificationRecord>, String>;
pub type NotificationStatsResult = Result<NotificationStats, String>;

// ========= STORABLE IMPLEMENTATIONS FOR NOTIFICATIONS =========

impl Storable for NotificationRecord {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        std::borrow::Cow::Owned(candid::encode_one(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        candid::decode_one(&bytes).unwrap()
    }
    
    const BOUND: ic_stable_structures::storable::Bound = ic_stable_structures::storable::Bound::Unbounded;
}

impl Storable for NotificationTemplate {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        std::borrow::Cow::Owned(candid::encode_one(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        candid::decode_one(&bytes).unwrap()
    }
    
    const BOUND: ic_stable_structures::storable::Bound = ic_stable_structures::storable::Bound::Unbounded;
}

impl Storable for NotificationSettings {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        std::borrow::Cow::Owned(candid::encode_one(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        candid::decode_one(&bytes).unwrap()
    }
    
    const BOUND: ic_stable_structures::storable::Bound = ic_stable_structures::storable::Bound::Unbounded;
}

impl Storable for NotificationStats {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        std::borrow::Cow::Owned(candid::encode_one(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        candid::decode_one(&bytes).unwrap()
    }
    
    const BOUND: ic_stable_structures::storable::Bound = ic_stable_structures::storable::Bound::Unbounded;
}

// ========== GOVERNANCE DASHBOARD TYPES ==========

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct GovernanceDashboard {
    pub stats: GovernanceStats,
    pub active_proposals: Vec<Proposal>,
    pub recent_proposals: Vec<Proposal>,
    pub admin_count: u64,
    pub system_status: std::collections::HashMap<String, bool>,
    pub parameter_count: u64,
    pub last_updated: u64,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct ParameterUpdateRequest {
    pub key: String,
    pub value: u64,
    pub reason: String,
    pub effective_date: Option<u64>,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct BatchParameterUpdate {
    pub parameters: Vec<ParameterUpdateRequest>,
    pub requester: Principal,
    pub requested_at: u64,
    pub approval_required: bool,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct AdminAuditLog {
    pub timestamp: u64,
    pub admin: Principal,
    pub action: AdminAction,
    pub target: Option<Principal>,
    pub details: String,
    pub success: bool,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub enum AdminAction {
    ParameterUpdate,
    RoleGrant,
    RoleRevoke,
    EmergencyStop,
    MaintenanceMode,
    ProposalExecution,
    SystemConfiguration,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct SystemHealthCheck {
    pub governance_health: bool,
    pub parameters_valid: bool,
    pub admin_roles_active: bool,
    pub emergency_status: bool,
    pub maintenance_mode: bool,
    pub last_check: u64,
    pub issues: Vec<String>,
}

impl Storable for GovernanceDashboard {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        std::borrow::Cow::Owned(candid::encode_one(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        candid::decode_one(&bytes).unwrap()
    }
    
    const BOUND: ic_stable_structures::storable::Bound = ic_stable_structures::storable::Bound::Unbounded;
}

// Additional types for enhanced liquidity withdrawal features
#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct WithdrawalValidation {
    pub is_valid: bool,
    pub amount_requested: u64,
    pub withdrawal_fee: u64,
    pub net_amount: u64,
    pub current_balance: u64,
    pub new_balance: u64,
    pub current_pool_liquidity: u64,
    pub new_pool_liquidity: u64,
    pub estimated_confirmation_time: u64, // seconds
    pub warnings: Vec<String>,
}

impl Storable for WithdrawalValidation {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        std::borrow::Cow::Owned(candid::encode_one(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        candid::decode_one(&bytes).unwrap()
    }
    
    const BOUND: ic_stable_structures::storable::Bound = ic_stable_structures::storable::Bound::Unbounded;
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct InvestorStatistics {
    pub investor: Principal,
    pub current_balance: u64,
    pub total_deposited: u64,
    pub total_withdrawn: u64,
    pub net_position: u64,
    pub total_deposits_count: u64,
    pub total_withdrawals_count: u64,
    pub pool_share_basis_points: u64, // Share of total pool (basis points)
    pub return_basis_points: u64, // Return percentage (basis points)
    pub avg_transaction_size: u64,
    pub days_since_first_deposit: u64,
    pub days_since_last_activity: u64,
    pub is_active_investor: bool,
    pub risk_level: String, // "LOW", "MEDIUM", "HIGH"
}

impl Storable for InvestorStatistics {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        std::borrow::Cow::Owned(candid::encode_one(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        candid::decode_one(&bytes).unwrap()
    }
    
    const BOUND: ic_stable_structures::storable::Bound = ic_stable_structures::storable::Bound::Unbounded;
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct WithdrawalFeeEstimate {
    pub requested_amount: u64,
    pub base_fee: u64,
    pub percentage_fee_basis_points: u64,
    pub total_fee: u64,
    pub net_withdrawal_amount: u64,
    pub fee_structure_version: u64,
}

impl Storable for WithdrawalFeeEstimate {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        std::borrow::Cow::Owned(candid::encode_one(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        candid::decode_one(&bytes).unwrap()
    }
    
    const BOUND: ic_stable_structures::storable::Bound = ic_stable_structures::storable::Bound::Unbounded;
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct LiquidityWithdrawalRequest {
    pub id: u64,
    pub investor: Principal,
    pub amount: u64,
    pub requested_at: u64,
    pub status: WithdrawalStatus,
    pub processed_at: Option<u64>,
    pub ckbtc_block_index: Option<u64>,
    pub failure_reason: Option<String>,
    pub admin_notes: Option<String>,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub enum WithdrawalStatus {
    Pending,
    Processing,
    Completed,
    Failed,
    Cancelled,
}

impl Storable for LiquidityWithdrawalRequest {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        std::borrow::Cow::Owned(candid::encode_one(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        candid::decode_one(&bytes).unwrap()
    }
    
    const BOUND: ic_stable_structures::storable::Bound = ic_stable_structures::storable::Bound::Unbounded;
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct LiquidityPoolAnalytics {
    pub total_liquidity_history: Vec<(u64, u64)>, // (timestamp, amount)
    pub utilization_rate_history: Vec<(u64, u64)>, // (timestamp, rate_basis_points)
    pub investor_count_history: Vec<(u64, u64)>, // (timestamp, count)
    pub average_deposit_size: u64,
    pub average_withdrawal_size: u64,
    pub deposit_frequency: u64, // deposits per day
    pub withdrawal_frequency: u64, // withdrawals per day
    pub peak_liquidity: u64,
    pub peak_utilization: u64,
    pub total_volume_deposited: u64,
    pub total_volume_withdrawn: u64,
    pub analytics_updated_at: u64,
}

impl Storable for LiquidityPoolAnalytics {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        std::borrow::Cow::Owned(candid::encode_one(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        candid::decode_one(&bytes).unwrap()
    }
    
    const BOUND: ic_stable_structures::storable::Bound = ic_stable_structures::storable::Bound::Unbounded;
}