use candid::{CandidType, Deserialize, Principal};
use ic_cdk::{caller, api::time};
use ic_cdk_macros::{query, update};

use crate::types::*;
use crate::user_management::{get_user_by_principal, User, Role};
use crate::storage::{
    get_loans_by_borrower, get_all_loans_data, get_liquidity_pool, 
    get_investor_balance_by_principal, get_all_investor_balances
};
use crate::liquidity_management::{get_pool_stats, get_investor_balance};
use crate::helpers::{is_admin, calculate_loan_health_ratio};

// Dashboard Data Types
#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct NFTSummary {
    pub token_id: u64,
    pub owner: Principal,
    pub metadata_title: String,
    pub metadata_description: String,
    pub commodity_type: String,
    pub valuation_idr: u64,
    pub quantity: u64,
    pub is_locked: bool,
    pub loan_id: Option<u64>,
    pub created_at: u64,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct LoanSummary {
    pub id: u64,
    pub borrower: Principal,
    pub nft_id: u64,
    pub amount_requested: u64,
    pub amount_approved: u64,
    pub status: LoanStatus,
    pub interest_rate: u64,
    pub total_repaid: u64,
    pub remaining_balance: u64,
    pub health_ratio: f64,
    pub created_at: u64,
    pub due_date: Option<u64>,
    pub is_overdue: bool,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct FarmerDashboardData {
    pub user_details: User,
    pub active_loans: Vec<LoanSummary>,
    pub historical_loans: Vec<LoanSummary>,
    pub owned_nfts: Vec<NFTSummary>,
    pub dashboard_stats: FarmerStats,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct FarmerStats {
    pub total_loans_applied: u64,
    pub total_loans_active: u64,
    pub total_loans_completed: u64,
    pub total_amount_borrowed: u64,
    pub total_amount_repaid: u64,
    pub average_loan_health: f64,
    pub total_nfts_owned: u64,
    pub total_nfts_locked: u64,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct InvestorDashboardData {
    pub user_details: User,
    pub current_balance: u64,
    pub total_invested: u64,
    pub total_earnings: u64,
    pub estimated_annual_return: f64,
    pub pool_stats: PoolStats,
    pub investment_history: Vec<InvestmentRecord>,
    pub dashboard_stats: InvestorStats,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct InvestorStats {
    pub days_invested: u64,
    pub total_deposits: u64,
    pub total_withdrawals: u64,
    pub current_portfolio_value: u64,
    pub roi_percentage: f64,
    pub participation_percentage: f64, // Percentage of total pool
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct InvestmentRecord {
    pub transaction_type: String, // "DEPOSIT" or "WITHDRAWAL"
    pub amount: u64,
    pub timestamp: u64,
    pub pool_apy_at_time: f64,
    pub balance_after: u64,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct AdminDashboardData {
    pub system_overview: SystemOverview,
    pub liquidity_metrics: LiquidityMetrics,
    pub loan_metrics: LoanMetrics,
    pub user_metrics: UserMetrics,
    pub risk_metrics: RiskMetrics,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct SystemOverview {
    pub total_users: u64,
    pub total_farmers: u64,
    pub total_investors: u64,
    pub total_loans: u64,
    pub total_nfts: u64,
    pub platform_uptime_days: u64,
    pub last_updated: u64,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct LiquidityMetrics {
    pub total_pool_value: u64,
    pub available_liquidity: u64,
    pub total_borrowed: u64,
    pub utilization_rate: f64,
    pub current_apy: f64,
    pub total_investors: u64,
    pub average_investor_balance: u64,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct LoanMetrics {
    pub total_loans: u64,
    pub active_loans: u64,
    pub completed_loans: u64,
    pub defaulted_loans: u64,
    pub total_amount_disbursed: u64,
    pub total_amount_repaid: u64,
    pub average_loan_amount: u64,
    pub default_rate: f64,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct UserMetrics {
    pub total_registered_users: u64,
    pub active_users: u64,
    pub completed_profiles: u64,
    pub users_with_btc_address: u64,
    pub new_users_this_month: u64,
    pub user_retention_rate: f64,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct RiskMetrics {
    pub loans_at_risk: u64,
    pub total_collateral_value: u64,
    pub average_health_ratio: f64,
    pub concentration_risk_score: f64,
    pub liquidity_risk_score: f64,
    pub overdue_loans: u64,
}

// Dashboard Query Functions

/// Get comprehensive dashboard data for farmers
/// Aggregates user details, loans, NFTs, and statistics
#[query]
pub fn get_farmer_dashboard() -> Result<FarmerDashboardData, String> {
    let caller_principal = caller();

    // Get user details
    let user_details = get_user_by_principal(&caller_principal)
        .ok_or("User not found. Please register first.")?;

    // Verify user is a farmer
    if user_details.role != Role::Farmer {
        return Err("Access denied: This endpoint is only for farmers".to_string());
    }

    // Get all loans for this farmer
    let all_loans = get_loans_by_borrower(caller_principal);
    
    // Separate active and historical loans
    let mut active_loans = Vec::new();
    let mut historical_loans = Vec::new();
    let mut total_amount_borrowed = 0u64;
    let mut total_amount_repaid = 0u64;
    let mut health_ratios = Vec::new();

    for loan in all_loans {
        let remaining_balance = loan.amount_approved.saturating_sub(loan.total_repaid);
        let health_ratio = calculate_loan_health_ratio(&loan).unwrap_or(0.0);
        let is_overdue = is_loan_overdue(&loan);

        let loan_summary = LoanSummary {
            id: loan.id,
            borrower: loan.borrower,
            nft_id: loan.nft_id,
            amount_requested: loan.amount_requested,
            amount_approved: loan.amount_approved,
            status: loan.status.clone(),
            interest_rate: loan.interest_rate,
            total_repaid: loan.total_repaid,
            remaining_balance,
            health_ratio,
            created_at: loan.created_at,
            due_date: loan.due_date,
            is_overdue,
        };

        total_amount_borrowed += loan.amount_approved;
        total_amount_repaid += loan.total_repaid;
        
        if health_ratio > 0.0 {
            health_ratios.push(health_ratio);
        }

        match loan.status {
            LoanStatus::Active | LoanStatus::PendingApproval | LoanStatus::Approved => {
                active_loans.push(loan_summary);
            }
            _ => {
                historical_loans.push(loan_summary);
            }
        }
    }

    // Get owned NFTs
    let owned_nfts = get_farmer_nfts(caller_principal)?;
    let total_nfts_owned = owned_nfts.len() as u64;
    let total_nfts_locked = owned_nfts.iter().filter(|nft| nft.is_locked).count() as u64;

    // Calculate statistics
    let total_loans_applied = (active_loans.len() + historical_loans.len()) as u64;
    let total_loans_active = active_loans.len() as u64;
    let total_loans_completed = historical_loans.iter()
        .filter(|loan| loan.status == LoanStatus::Repaid)
        .count() as u64;

    let average_loan_health = if health_ratios.is_empty() {
        0.0
    } else {
        health_ratios.iter().sum::<f64>() / health_ratios.len() as f64
    };

    let dashboard_stats = FarmerStats {
        total_loans_applied,
        total_loans_active,
        total_loans_completed,
        total_amount_borrowed,
        total_amount_repaid,
        average_loan_health,
        total_nfts_owned,
        total_nfts_locked,
    };

    Ok(FarmerDashboardData {
        user_details,
        active_loans,
        historical_loans,
        owned_nfts,
        dashboard_stats,
    })
}

/// Get comprehensive dashboard data for investors
/// Aggregates user details, balance, earnings, and pool statistics
#[query]
pub fn get_investor_dashboard() -> Result<InvestorDashboardData, String> {
    let caller_principal = caller();

    // Get user details
    let user_details = get_user_by_principal(&caller_principal)
        .ok_or("User not found. Please register first.")?;

    // Verify user is an investor
    if user_details.role != Role::Investor {
        return Err("Access denied: This endpoint is only for investors".to_string());
    }

    // Get investor balance
    let investor_balance = get_investor_balance()
        .map_err(|e| format!("Failed to get investor balance: {}", e))?;

    // Get pool statistics
    let pool_stats = get_pool_stats();

    // Calculate investment metrics
    let current_balance = investor_balance.balance;
    let total_invested = investor_balance.total_deposited;
    let total_withdrawn = investor_balance.total_withdrawn;
    
    // Calculate earnings (simplified - in production would need more complex calculation)
    let total_earnings = if total_invested > 0 {
        current_balance + total_withdrawn - total_invested
    } else {
        0
    };

    // Calculate ROI percentage
    let roi_percentage = if total_invested > 0 {
        ((total_earnings as f64) / (total_invested as f64)) * 100.0
    } else {
        0.0
    };

    // Calculate participation percentage in pool
    let participation_percentage = if pool_stats.total_liquidity > 0 {
        (current_balance as f64 / pool_stats.total_liquidity as f64) * 100.0
    } else {
        0.0
    };

    // Calculate days invested
    let days_invested = if investor_balance.first_deposit_at > 0 {
        (time() - investor_balance.first_deposit_at) / (24 * 60 * 60 * 1_000_000_000)
    } else {
        0
    };

    // Build investment history
    let mut investment_history = Vec::new();
    
    // Add deposit records
    for deposit in &investor_balance.deposits {
        investment_history.push(InvestmentRecord {
            transaction_type: "DEPOSIT".to_string(),
            amount: deposit.amount,
            timestamp: deposit.timestamp,
            pool_apy_at_time: pool_stats.apy, // Simplified - would need historical APY
            balance_after: deposit.amount, // Simplified
        });
    }

    // Add withdrawal records
    for withdrawal in &investor_balance.withdrawals {
        investment_history.push(InvestmentRecord {
            transaction_type: "WITHDRAWAL".to_string(),
            amount: withdrawal.amount,
            timestamp: withdrawal.timestamp,
            pool_apy_at_time: pool_stats.apy,
            balance_after: withdrawal.amount, // Simplified
        });
    }

    // Sort by timestamp (most recent first)
    investment_history.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

    let dashboard_stats = InvestorStats {
        days_invested,
        total_deposits: total_invested,
        total_withdrawals: total_withdrawn,
        current_portfolio_value: current_balance,
        roi_percentage,
        participation_percentage,
    };

    let estimated_annual_return = pool_stats.apy;

    Ok(InvestorDashboardData {
        user_details,
        current_balance,
        total_invested,
        total_earnings,
        estimated_annual_return,
        pool_stats,
        investment_history,
        dashboard_stats,
    })
}

/// Get comprehensive admin dashboard data (admin only)
/// Provides system-wide metrics and insights
#[query]
pub fn get_admin_dashboard() -> Result<AdminDashboardData, String> {
    let caller_principal = caller();

    // Verify admin access
    if !is_admin(&caller_principal) {
        return Err("Access denied: Admin privileges required".to_string());
    }

    // Get system overview
    let user_stats = crate::user_management::get_user_stats();
    let all_loans = get_all_loans_data();
    let total_nfts = get_total_nfts_count();
    
    let system_overview = SystemOverview {
        total_users: user_stats.total_users,
        total_farmers: user_stats.total_farmers,
        total_investors: user_stats.total_investors,
        total_loans: all_loans.len() as u64,
        total_nfts,
        platform_uptime_days: calculate_platform_uptime_days(),
        last_updated: time(),
    };

    // Get liquidity metrics
    let pool_stats = get_pool_stats();
    let all_investor_balances = get_all_investor_balances();
    let average_investor_balance = if all_investor_balances.len() > 0 {
        all_investor_balances.iter().map(|b| b.balance).sum::<u64>() / all_investor_balances.len() as u64
    } else {
        0
    };

    let liquidity_metrics = LiquidityMetrics {
        total_pool_value: pool_stats.total_liquidity,
        available_liquidity: pool_stats.available_liquidity,
        total_borrowed: pool_stats.total_borrowed,
        utilization_rate: pool_stats.utilization_rate,
        current_apy: pool_stats.apy,
        total_investors: pool_stats.total_investors,
        average_investor_balance,
    };

    // Calculate loan metrics
    let active_loans = all_loans.iter().filter(|l| l.status == LoanStatus::Active).count() as u64;
    let completed_loans = all_loans.iter().filter(|l| l.status == LoanStatus::Repaid).count() as u64;
    let defaulted_loans = all_loans.iter().filter(|l| l.status == LoanStatus::Defaulted).count() as u64;
    
    let total_amount_disbursed = all_loans.iter().map(|l| l.amount_approved).sum::<u64>();
    let total_amount_repaid = all_loans.iter().map(|l| l.total_repaid).sum::<u64>();
    
    let average_loan_amount = if all_loans.len() > 0 {
        total_amount_disbursed / all_loans.len() as u64
    } else {
        0
    };

    let default_rate = if all_loans.len() > 0 {
        (defaulted_loans as f64 / all_loans.len() as f64) * 100.0
    } else {
        0.0
    };

    let loan_metrics = LoanMetrics {
        total_loans: all_loans.len() as u64,
        active_loans,
        completed_loans,
        defaulted_loans,
        total_amount_disbursed,
        total_amount_repaid,
        average_loan_amount,
        default_rate,
    };

    // User metrics (using existing stats)
    let user_metrics = UserMetrics {
        total_registered_users: user_stats.total_users,
        active_users: user_stats.active_users,
        completed_profiles: user_stats.completed_profiles,
        users_with_btc_address: user_stats.users_with_btc_address,
        new_users_this_month: calculate_new_users_this_month(),
        user_retention_rate: calculate_user_retention_rate(),
    };

    // Calculate risk metrics
    let loans_at_risk = calculate_loans_at_risk(&all_loans);
    let total_collateral_value = calculate_total_collateral_value(&all_loans);
    let average_health_ratio = calculate_average_health_ratio(&all_loans);
    let overdue_loans = all_loans.iter().filter(|l| is_loan_overdue(l)).count() as u64;

    let risk_metrics = RiskMetrics {
        loans_at_risk,
        total_collateral_value,
        average_health_ratio,
        concentration_risk_score: calculate_concentration_risk_score(&all_loans),
        liquidity_risk_score: calculate_liquidity_risk_score(&pool_stats),
        overdue_loans,
    };

    Ok(AdminDashboardData {
        system_overview,
        liquidity_metrics,
        loan_metrics,
        user_metrics,
        risk_metrics,
    })
}

/// Get quick statistics for homepage/public dashboard
#[query]
pub fn get_public_stats() -> PublicStats {
    let user_stats = crate::user_management::get_user_stats();
    let pool_stats = get_pool_stats();
    let all_loans = get_all_loans_data();

    PublicStats {
        total_users: user_stats.total_users,
        total_farmers: user_stats.total_farmers,
        total_investors: user_stats.total_investors,
        total_liquidity: pool_stats.total_liquidity,
        total_loans_disbursed: all_loans.len() as u64,
        current_apy: pool_stats.apy,
        platform_uptime_days: calculate_platform_uptime_days(),
    }
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct PublicStats {
    pub total_users: u64,
    pub total_farmers: u64,
    pub total_investors: u64,
    pub total_liquidity: u64,
    pub total_loans_disbursed: u64,
    pub current_apy: f64,
    pub platform_uptime_days: u64,
}

// Helper Functions

/// Get NFTs owned by a farmer (Inter-canister call simulation)
/// In production, this would call the RWA NFT canister
fn get_farmer_nfts(farmer_principal: Principal) -> Result<Vec<NFTSummary>, String> {
    // This is a placeholder - in production, you would make an inter-canister call
    // to the RWA NFT canister to get the actual NFT data
    
    // For now, we'll use the storage directly (this should be replaced with inter-canister call)
    use crate::storage::RWA_NFTS;
    
    let nfts = RWA_NFTS.with(|nfts| {
        let nfts_map = nfts.borrow();
        let mut owned_nfts = Vec::new();
        
        for (token_id, nft_data) in nfts_map.iter() {
            if nft_data.owner == farmer_principal {
                // Extract metadata values
                let (title, valuation_idr, commodity_type) = extract_nft_metadata(&nft_data.metadata);
                
                owned_nfts.push(NFTSummary {
                    token_id,
                    owner: nft_data.owner,
                    metadata_title: title,
                    metadata_description: format!("Agricultural asset #{}", token_id),
                    commodity_type,
                    valuation_idr,
                    quantity: 1, // Simplified
                    is_locked: nft_data.is_locked,
                    loan_id: nft_data.loan_id,
                    created_at: nft_data.created_at,
                });
            }
        }
        
        owned_nfts
    });
    
    Ok(nfts)
}

/// Extract metadata values from NFT metadata
fn extract_nft_metadata(metadata: &Vec<(String, MetadataValue)>) -> (String, u64, String) {
    let mut title = "Agricultural Asset".to_string();
    let mut valuation_idr = 0u64;
    let mut commodity_type = "Unknown".to_string();
    
    for (key, value) in metadata {
        match key.as_str() {
            "title" | "name" => {
                if let MetadataValue::Text(text) = value {
                    title = text.clone();
                }
            }
            "valuation_idr" => {
                if let MetadataValue::Nat(nat) = value {
                    valuation_idr = *nat;
                }
            }
            "commodity_type" => {
                if let MetadataValue::Text(text) = value {
                    commodity_type = text.clone();
                }
            }
            _ => {}
        }
    }
    
    (title, valuation_idr, commodity_type)
}

/// Check if a loan is overdue
fn is_loan_overdue(loan: &Loan) -> bool {
    if let Some(due_date) = loan.due_date {
        time() > due_date && loan.status == LoanStatus::Active
    } else {
        false
    }
}

/// Get total NFTs count (inter-canister call simulation)
fn get_total_nfts_count() -> u64 {
    use crate::storage::RWA_NFTS;
    
    RWA_NFTS.with(|nfts| nfts.borrow().len() as u64)
}

/// Calculate platform uptime in days
fn calculate_platform_uptime_days() -> u64 {
    // This is a simplified calculation
    // In production, you might want to track the actual deployment date
    let deployment_timestamp = 1_700_000_000_000_000_000u64; // Example timestamp
    let current_time = time();
    
    if current_time > deployment_timestamp {
        (current_time - deployment_timestamp) / (24 * 60 * 60 * 1_000_000_000)
    } else {
        0
    }
}

/// Calculate new users this month
fn calculate_new_users_this_month() -> u64 {
    // This is a placeholder - in production, you would track user registration timestamps
    // and count those from the current month
    0
}

/// Calculate user retention rate
fn calculate_user_retention_rate() -> f64 {
    // This is a placeholder - in production, you would implement proper retention tracking
    85.5 // Example value
}

/// Calculate loans at risk (health ratio below threshold)
fn calculate_loans_at_risk(loans: &[Loan]) -> u64 {
    loans.iter()
        .filter(|loan| {
            if loan.status == LoanStatus::Active {
                if let Ok(health_ratio) = calculate_loan_health_ratio(loan) {
                    health_ratio < 1.5 // Threshold for at-risk loans
                } else {
                    false
                }
            } else {
                false
            }
        })
        .count() as u64
}

/// Calculate total collateral value
fn calculate_total_collateral_value(loans: &[Loan]) -> u64 {
    loans.iter()
        .filter(|loan| loan.status == LoanStatus::Active)
        .map(|loan| loan.collateral_value_btc)
        .sum()
}

/// Calculate average health ratio
fn calculate_average_health_ratio(loans: &[Loan]) -> f64 {
    let active_loans: Vec<&Loan> = loans.iter()
        .filter(|loan| loan.status == LoanStatus::Active)
        .collect();
    
    if active_loans.is_empty() {
        return 0.0;
    }
    
    let total_health_ratio: f64 = active_loans.iter()
        .filter_map(|loan| calculate_loan_health_ratio(loan).ok())
        .sum();
    
    total_health_ratio / active_loans.len() as f64
}

/// Calculate concentration risk score
fn calculate_concentration_risk_score(loans: &[Loan]) -> f64 {
    // This is a simplified concentration risk calculation
    // In production, you would implement more sophisticated risk modeling
    
    if loans.is_empty() {
        return 0.0;
    }
    
    let total_exposure = loans.iter()
        .filter(|loan| loan.status == LoanStatus::Active)
        .map(|loan| loan.amount_approved)
        .sum::<u64>();
    
    if total_exposure == 0 {
        return 0.0;
    }
    
    let max_loan = loans.iter()
        .filter(|loan| loan.status == LoanStatus::Active)
        .map(|loan| loan.amount_approved)
        .max()
        .unwrap_or(0);
    
    // Risk score based on largest loan percentage of total exposure
    (max_loan as f64 / total_exposure as f64) * 100.0
}

/// Calculate liquidity risk score
fn calculate_liquidity_risk_score(pool_stats: &PoolStats) -> f64 {
    // Simple liquidity risk calculation based on utilization rate
    if pool_stats.total_liquidity == 0 {
        return 100.0; // Maximum risk if no liquidity
    }
    
    let utilization_rate = pool_stats.utilization_rate;
    
    // Risk increases exponentially after 80% utilization
    if utilization_rate > 80.0 {
        50.0 + ((utilization_rate - 80.0) * 2.5)
    } else {
        utilization_rate * 0.625 // Linear scaling up to 50% risk at 80% utilization
    }
}

/// Get all investor balances (internal helper)
fn get_all_investor_balances() -> Vec<InvestorBalance> {
    use crate::storage::INVESTOR_BALANCES;
    
    INVESTOR_BALANCES.with(|balances| {
        balances.borrow().iter().map(|(_, balance)| balance).collect()
    })
}

// Dashboard refresh functions for real-time updates

/// Refresh dashboard cache (admin only)
/// This would be useful for ensuring dashboard data is up-to-date
#[update]
pub fn refresh_dashboard_cache() -> Result<String, String> {
    let caller_principal = caller();
    
    if !is_admin(&caller_principal) {
        return Err("Access denied: Admin privileges required".to_string());
    }
    
    // In production, you might implement caching mechanisms here
    // For now, we'll just return success as data is always fresh
    
    Ok("Dashboard cache refreshed successfully".to_string())
}

/// Get dashboard loading status
/// Useful for frontend to show appropriate loading states
#[query]
pub fn get_dashboard_status() -> DashboardStatus {
    DashboardStatus {
        farmer_dashboard_available: true,
        investor_dashboard_available: true,
        admin_dashboard_available: true,
        last_updated: time(),
        system_healthy: true,
    }
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct DashboardStatus {
    pub farmer_dashboard_available: bool,
    pub investor_dashboard_available: bool,
    pub admin_dashboard_available: bool,
    pub last_updated: u64,
    pub system_healthy: bool,
}
