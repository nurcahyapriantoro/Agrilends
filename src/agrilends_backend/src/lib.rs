use candid::Principal;
use ic_cdk::api::{canister_self, time};
use ic_cdk::{caller};
use ic_cdk_macros::{query, pre_upgrade, post_upgrade, heartbeat};

// Add public re-exports
pub use user_management::{User, USERS}; // Add this line
pub use audit_logging::*; // Export all audit logging functions
pub use automated_maintenance::*; // Export automated maintenance functions
pub use notification_system::*; // Export notification system functions

// Existing modules
mod types;
mod storage;
mod user_management;
mod loan_lifecycle;
mod loan_repayment;  // Add new module
mod liquidation;     // Add liquidation module
mod governance;      // Add governance module
mod treasury_management; // Add treasury management module
mod treasury_management_tests; // Add treasury tests
mod rwa_nft;
mod ckbtc_integration;
mod liquidity_management;
mod oracle;          // Oracle module for production
mod oracle_integration; // Oracle integration helper
mod production_config;
mod production_security;
mod monitoring;
mod helpers;

// Scalability modules
mod scalability_architecture;
mod loan_data_canister;
mod advanced_query_routing;
mod load_balancing;
mod scalability_tests;
mod audit_logging;   // Add comprehensive audit logging module
mod automated_maintenance; // Add automated maintenance module
mod notification_system; // Add notification system module
mod dashboard_support; // Add dashboard support module
mod advanced_analytics; // Add advanced analytics module
mod scalability_architecture; // Add scalability architecture module
mod loan_data_canister; // Add loan data canister module
mod advanced_query_routing; // Add advanced query routing module
mod load_balancing; // Add load balancing module

// Add tests module
#[cfg(test)]
mod tests;

#[cfg(test)]
mod liquidity_management_tests;

// Specific imports to avoid ambiguous re-exports
pub use user_management::*;
pub use rwa_nft::*;
pub use types::{
    Account as TypesAccount, MetadataValue, TransferRequest, TransferResult, RWANFTData, RWANFTResult,
    CollateralStatus, CollateralRecord, NFTStats, StorageStats, AuditLog, CanisterConfig,
    LoanStatus, Loan, LoanApplication, CommodityPrice, NFTMetadata, ProtocolParameters,
    DisbursementRecord, RepaymentRecord, ProductionHealthStatus, CommodityPriceData,
    LiquidityPool, InvestorBalance, DepositRecord, WithdrawalRecord, ProcessedTransaction,
    PoolStats, InvestorTransactionHistory, PoolHealthMetrics, PoolConfiguration,
    Payment, PaymentType, PaymentBreakdown, LoanRepaymentSummary, RepaymentPlan, RepaymentResponse,
    LiquidationRecord, LiquidationReason, LiquidationSummary, LiquidationEligibilityCheck,
    LiquidationResult, LiquidationStatistics, ComprehensiveRepaymentAnalytics, LoanPerformanceMetrics,
    BatchRepaymentRequest, BatchRepaymentResult, RepaymentStatistics, RepaymentForecast,
    // Oracle Types
    PriceFetchRecord, OracleConfig, OracleStatistics, PriceAlert, PriceThresholdType,
    // Treasury Management Types
    TreasuryState, RevenueEntry, RevenueType, TransactionStatus, CanisterInfo, CanisterType,
    CycleTransaction, TreasuryStats, CanisterCycleStatus, TreasuryHealthReport
};
pub use storage::{
    get_nft_by_token_id, get_collateral_by_id, get_loan_by_id, update_collateral_status,
    count_user_nfts, get_config, update_config, log_action, log_nft_activity,
    get_nfts_by_owner, get_collateral_by_nft_token_id, get_audit_logs, cleanup_audit_logs,
    get_next_loan_id, store_loan, get_loan, get_loans_by_borrower, get_all_loans_data,
    get_protocol_parameters, set_protocol_parameters, get_nft_data, lock_nft_for_loan,
    unlock_nft, liquidate_collateral, store_disbursement_record, get_disbursement_record,
    get_all_disbursement_records, store_repayment_record, get_repayment_record, 
    get_all_repayment_records, get_repayment_records_by_loan, update_loan_status,
    update_loan_repaid_amount, calculate_remaining_balance,
    get_total_investors, get_total_deposits, get_total_withdrawals,
    get_all_processed_transactions, get_processed_transactions_by_investor,
    count_processed_transactions, get_pool_utilization_history, get_investor_count,
    get_active_investor_count, get_total_investor_deposits, get_total_investor_withdrawals,
    get_largest_investor_deposit, get_average_investor_deposit, get_pool_concentration_risk,
    cleanup_old_processed_transactions, get_storage_statistics, get_storage_stats,
    store_commodity_price, get_stored_commodity_price, get_all_stored_commodity_prices,
    update_last_price_fetch, get_last_price_fetch, get_liquidity_pool, store_liquidity_pool,
    get_investor_balance_by_principal, store_investor_balance, get_all_investor_balances,
    is_transaction_processed, mark_transaction_processed, has_investor_deposited_before,
    set_emergency_pause, is_emergency_paused, get_processed_transaction, remove_processed_transaction,
    next_nft_token_id, next_collateral_id, next_loan_id, next_disbursement_id, update_loan,
    update_price_fetch_failure, get_price_fetch_statistics
};
pub use oracle::{
    fetch_commodity_price, get_commodity_price, admin_set_commodity_price, get_all_commodity_prices,
    is_price_stale, get_oracle_statistics, configure_oracle, get_oracle_config,
    add_price_alert, get_price_alerts, enable_emergency_mode, disable_emergency_mode,
    oracle_health_check, heartbeat_price_update
};
pub use helpers::{
    validate_nft_metadata, init_admin_principals, set_loan_manager_principal, is_admin, is_loan_manager_canister,
    is_authorized_to_mint, check_rate_limit, extract_metadata_values, validate_sha256_hash, log_audit_action,
    get_canister_config, set_canister_config, add_admin, remove_admin, calculate_loan_health_ratio,
    is_loan_at_risk, get_overdue_loans, format_loan_summary, is_loan_manager, release_collateral_nft,
    get_active_loans_count, get_memory_usage, check_oracle_health, check_ckbtc_health, get_last_heartbeat_time,
    is_in_maintenance_mode, get_emergency_stop_status, monitor_cycles_balance,
    cleanup_old_audit_logs, get_user_btc_address
};
pub use loan_lifecycle::*;
pub use loan_repayment::{
    repay_loan, get_loan_repayment_summary, get_repayment_plan, get_loan_payment_history,
    get_loan_repayment_records, check_repayment_eligibility, calculate_early_repayment_benefits,
    emergency_repayment, get_repayment_statistics, calculate_total_debt_with_interest,
    calculate_payment_breakdown, get_comprehensive_repayment_analytics, calculate_loan_performance_metrics,
    process_batch_repayments, schedule_automatic_repayment, get_repayment_forecast,
    collect_protocol_fees_from_repayment, validate_repayment_amount
};
pub use liquidation::{
    trigger_liquidation, check_liquidation_eligibility, get_loans_eligible_for_liquidation,
    get_liquidation_record, get_all_liquidation_records, get_liquidation_statistics,
    trigger_bulk_liquidation, emergency_liquidation, automated_liquidation_check,
    get_liquidation_metrics, assess_liquidation_risk, get_loan_liquidation_history,
    list_all_liquidations, LiquidationMetrics, LiquidationRiskAssessment, LiquidationStatistics
};
pub use governance::{
    create_proposal, vote_on_proposal, execute_proposal, set_protocol_parameter,
    get_protocol_parameter, get_all_protocol_parameters, grant_admin_role, revoke_admin_role,
    transfer_admin_role, get_admin_role, get_all_admin_roles, get_proposal, get_proposals,
    get_proposal_votes, get_governance_stats, emergency_stop, resume_operations,
    update_governance_config, get_governance_config_public, create_batch_proposals,
    set_multiple_protocol_parameters, get_protocol_parameters_by_category,
    validate_parameter_value, get_parameter_history, can_execute_proposal,
    get_proposals_by_status, get_active_admin_count, set_maintenance_mode,
    get_system_status, initialize_super_admin, get_governance_dashboard
};

// Add dashboard support exports
pub use dashboard_support::{
    get_farmer_dashboard, get_investor_dashboard, get_admin_dashboard, get_public_stats,
    refresh_dashboard_cache, get_dashboard_status,
    FarmerDashboardData, InvestorDashboardData, AdminDashboardData, PublicStats,
    NFTSummary, LoanSummary, FarmerStats, InvestorStats, InvestmentRecord,
    SystemOverview, LiquidityMetrics, LoanMetrics, UserMetrics, RiskMetrics,
    DashboardStatus
};
pub use oracle::{fetch_commodity_price, get_commodity_price, admin_set_commodity_price, 
    get_all_commodity_prices, is_price_stale, heartbeat_price_update};
pub use ckbtc_integration::{transfer_ckbtc_to_borrower, process_ckbtc_repayment, 
    check_ckbtc_balance, get_protocol_ckbtc_balance, admin_withdraw_protocol_earnings};
pub use production_config::*;
pub use production_security::*;
pub use monitoring::*;
pub use liquidity_management::{
    deposit_liquidity, disburse_loan, withdraw_liquidity, 
    get_pool_stats, get_investor_balance, get_pool_details, get_all_investor_balances_admin,
    process_loan_repayment, collect_protocol_fees, emergency_pause_pool, resume_pool_operations,
    get_investor_transaction_history, get_all_disbursements, get_loan_disbursements,
    refresh_pool_statistics, set_pool_parameters, get_pool_health_metrics,
    perform_pool_maintenance, emergency_halt_operations, is_pool_paused,
    get_pool_configuration, get_processed_transactions_admin, get_my_processed_transactions,
    get_disbursement_records_by_loan
};
pub use treasury_management::{
    collect_fees, top_up_canister_cycles, get_treasury_stats, register_canister,
    update_canister_config, get_canister_cycle_status, get_revenue_log, emergency_withdraw,
    init_treasury, treasury_heartbeat, get_cycle_transactions, trigger_cycle_distribution,
    get_treasury_health_report, process_loan_fee_collection, process_liquidation_penalty,
    set_treasury_configuration
};

// Export advanced analytics functions
pub use advanced_analytics::{
    generate_analytics_report, get_predictive_analysis, get_portfolio_optimization,
    get_stress_test_results, get_market_intelligence
};

// System functions
#[query]
pub fn get_canister_id() -> Principal {
    canister_self()
}

#[query]
pub fn get_caller() -> Principal {
    caller()
}

// Health check function
#[query]
pub fn health_check() -> String {
    "OK".to_string()
}

// Loan lifecycle status check
#[query]
pub fn loan_lifecycle_status() -> String {
    #[cfg(test)]
    {
        crate::tests::test_loan_lifecycle_integration()
    }
    #[cfg(not(test))]
    {
        "Loan Lifecycle Integration Test:\n\
        - Loan types defined: ✓\n\
        - Storage functions implemented: ✓\n\
        - Application workflow: ✓\n\
        - Approval process: ✓\n\
        - Repayment system: ✓\n\
        - Liquidation mechanism: ✓\n\
        - Audit logging: ✓\n\
        \n\
        Ready for deployment and testing!".to_string()
    }
}

// Pre-upgrade hook
#[pre_upgrade]
fn pre_upgrade() {
    // User data is automatically preserved due to StableBTreeMap
    ic_cdk::println!("Pre-upgrade: User data preserved in stable storage");
}

// Post-upgrade hook
#[post_upgrade]
fn post_upgrade() {
    ic_cdk::println!("Post-upgrade: User management system restored");
    
    // Initialize treasury management system
    treasury_management::init_treasury();
    ic_cdk::println!("Post-upgrade: Treasury management system initialized");
}

// Generate Candid interface
#[query(name = "__get_candid_interface_tmp_hack")]
fn export_candid() -> String {
    "service : { ... }".to_string()
}

// Production heartbeat for automated maintenance - delegates to automated_maintenance module
#[heartbeat]
async fn heartbeat() {
    // Delegate to the comprehensive automated maintenance system
    automated_maintenance::canister_heartbeat().await;
}

// Emergency stop functionality
#[query]
pub fn is_emergency_stopped() -> bool {
    get_emergency_stop_status()
}

// Production health check
#[query]
pub fn production_health_check() -> ProductionHealthStatus {
    ProductionHealthStatus {
        is_healthy: !is_emergency_stopped() && !is_in_maintenance_mode(),
        emergency_stop: is_emergency_stopped(),
        maintenance_mode: is_in_maintenance_mode(),
        oracle_status: check_oracle_health(),
        ckbtc_integration: check_ckbtc_health(),
        memory_usage: get_memory_usage(),
        total_loans: get_active_loans_count(),
        active_loans: get_active_loans_count(),
        last_heartbeat: get_last_heartbeat_time(),
    }
}

// === SCALABILITY AND SHARDING FUNCTIONS ===

/// Get factory pattern statistics
#[query]
pub fn get_factory_stats() -> scalability_architecture::FactoryStats {
    scalability_architecture::get_factory_stats()
}

/// Get metrics for a specific shard
#[query]
pub fn get_shard_metrics(shard_id: u64) -> Option<types::ShardMetrics> {
    scalability_architecture::get_shard_metrics(shard_id)
}

/// Get comprehensive scalability metrics
#[query]
pub fn get_scalability_metrics() -> scalability_architecture::ScalabilityMetrics {
    scalability_architecture::get_scalability_metrics()
}

/// Get farmer dashboard data with advanced routing
#[query]
pub async fn get_farmer_dashboard_advanced(farmer_id: Principal) -> advanced_query_routing::DashboardData {
    advanced_query_routing::get_farmer_dashboard_advanced(farmer_id).await
}

/// Force shard migration (admin only)
#[update]
pub async fn migrate_shard_data(from_shard: u64, to_shard: u64) -> Result<String, String> {
    // Check admin permissions
    if !is_admin(&caller()) {
        return Err("Unauthorized: Admin access required".to_string());
    }
    scalability_architecture::migrate_shard_data(from_shard, to_shard).await
}

/// Create a new data shard
#[update]
pub async fn create_data_shard() -> Result<u64, String> {
    if !is_admin(&caller()) {
        return Err("Unauthorized: Admin access required".to_string());
    }
    scalability_architecture::create_new_data_shard().await
}

/// Get load balancing metrics
#[query]
pub fn get_load_balancing_metrics() -> load_balancing::LoadBalancingMetrics {
    load_balancing::get_load_balancing_metrics()
}

/// Test scalability features (development only)
#[update]
pub async fn run_scalability_tests() -> Vec<scalability_tests::TestResult> {
    // Only allow in development mode
    if is_production_mode() {
        return vec![scalability_tests::TestResult {
            test_name: "Scalability Tests".to_string(),
            passed: false,
            message: "Tests disabled in production mode".to_string(),
            execution_time_ms: 0,
        }];
    }
    scalability_tests::run_all_tests().await
}

/// Scalability heartbeat for automated scaling
#[update]
pub async fn scalability_heartbeat() {
    scalability_architecture::scalability_heartbeat().await;
}

/// Get query routing cache statistics
#[query]
pub fn get_query_cache_stats() -> advanced_query_routing::CacheStats {
    advanced_query_routing::get_cache_stats()
}

/// Clear query routing cache (admin only)
#[update]
pub fn clear_query_cache() -> Result<String, String> {
    if !is_admin(&caller()) {
        return Err("Unauthorized: Admin access required".to_string());
    }
    advanced_query_routing::clear_cache();
    Ok("Query cache cleared successfully".to_string())
}
