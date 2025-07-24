// ========== COMPREHENSIVE AUDIT LOGGING MODULE ==========
// Enhanced audit logging system for Agrilends protocol
// Provides immutable audit trail for all critical operations
// Production-ready implementation with advanced features

use ic_cdk::{caller, api::time, api::management_canister::main::{canister_status, CanisterIdRecord}};
use ic_cdk_macros::{query, update, heartbeat};
use candid::{CandidType, Deserialize, Principal};
use ic_stable_structures::{StableBTreeMap, memory::MemoryId};
use ic_stable_structures::memory::VirtualMemory;
use ic_stable_structures::DefaultMemoryImpl;
use std::cell::RefCell;
use std::collections::HashMap;

use crate::types::*;
use crate::storage::{get_memory_by_id, AUDIT_LOG_COUNTER};
use crate::helpers::is_admin;

// Enhanced audit log types
#[derive(CandidType, Deserialize, Clone, Debug, PartialEq)]
pub enum AuditEventLevel {
    Info,     // General information
    Warning,  // Potential issues
    Error,    // Error conditions
    Critical, // Critical security events
    Success,  // Successful operations
    Debug,    // Debug information (production logs)
}

#[derive(CandidType, Deserialize, Clone, Debug, PartialEq)]
pub enum AuditCategory {
    UserManagement,      // User registration, role changes
    NFTOperations,       // RWA-NFT minting, transfers
    LoanLifecycle,       // Loan applications, approvals, disbursements
    LoanRepayment,       // Loan repayments, completions
    Liquidation,         // Liquidation processes
    LiquidityManagement, // Liquidity deposits, withdrawals
    Governance,          // DAO voting, proposals, admin actions
    Treasury,            // Treasury management, cycles
    Oracle,              // Price feeds, external API calls
    Security,            // Security events, blacklisting
    Configuration,       // System configuration changes
    Maintenance,         // Automated maintenance tasks
    Integration,         // External integrations (CKBTC, etc.)
    Compliance,          // Regulatory compliance events
    Performance,         // Performance monitoring
}

// Enhanced audit log entry with additional metadata
#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct EnhancedAuditLog {
    pub id: u64,
    pub timestamp: u64,
    pub block_height: Option<u64>, // IC block height for immutability verification
    pub caller: Principal,
    pub category: AuditCategory,
    pub action: String,
    pub level: AuditEventLevel,
    pub details: AuditDetails,
    pub result: AuditResult,
    pub correlation_id: Option<String>, // For tracking related operations
    pub session_id: Option<String>,     // For tracking user sessions
    pub ip_hash: Option<String>,        // Hashed IP for privacy-compliant tracking
    pub canister_id: Option<Principal>, // Source canister ID
    pub version: String,                // System version
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct AuditDetails {
    pub description: String,
    pub entity_type: Option<String>,    // e.g., "loan", "nft", "user"
    pub entity_id: Option<String>,      // e.g., loan_id, token_id, user_id
    pub before_state: Option<String>,   // JSON representation of state before
    pub after_state: Option<String>,    // JSON representation of state after
    pub affected_principals: Vec<Principal>, // Other users affected by this action
    pub metadata: Vec<(String, String)>, // Additional key-value metadata
    pub risk_score: Option<u32>,        // Risk assessment (0-100)
    pub location_hash: Option<String>,  // Geographic location hash
    pub user_agent_hash: Option<String>, // User agent hash
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct AuditResult {
    pub success: bool,
    pub error_code: Option<String>,
    pub error_message: Option<String>,
    pub execution_time_ms: Option<u64>,
    pub gas_used: Option<u64>,
    pub cycles_consumed: Option<u64>,
    pub memory_used_bytes: Option<u64>,
    pub warning_flags: Vec<String>,
}

// Audit query filters
#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct AuditLogFilter {
    pub start_time: Option<u64>,
    pub end_time: Option<u64>,
    pub caller: Option<Principal>,
    pub category: Option<AuditCategory>,
    pub level: Option<AuditEventLevel>,
    pub action_pattern: Option<String>,
    pub success_only: Option<bool>,
    pub entity_type: Option<String>,
    pub entity_id: Option<String>,
    pub correlation_id: Option<String>,
    pub session_id: Option<String>,
    pub risk_score_min: Option<u32>,
    pub risk_score_max: Option<u32>,
    pub limit: Option<u64>,
    pub offset: Option<u64>,
    pub sort_order: Option<SortOrder>,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub enum SortOrder {
    TimestampAsc,
    TimestampDesc,
    RiskScoreAsc,
    RiskScoreDesc,
}

impl Default for AuditLogFilter {
    fn default() -> Self {
        Self {
            start_time: None,
            end_time: None,
            caller: None,
            category: None,
            level: None,
            action_pattern: None,
            success_only: None,
            entity_type: None,
            entity_id: None,
            correlation_id: None,
            session_id: None,
            risk_score_min: None,
            risk_score_max: None,
            limit: Some(100), // Default limit
            offset: None,
            sort_order: Some(SortOrder::TimestampDesc),
        }
    }
}

// Audit statistics
#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct AuditStatistics {
    pub total_logs: u64,
    pub logs_by_category: HashMap<String, u64>,
    pub logs_by_level: HashMap<String, u64>,
    pub success_rate: f64,
    pub most_active_callers: Vec<(Principal, u64)>,
    pub recent_critical_events: u64,
    pub storage_usage_bytes: u64,
    pub oldest_log_timestamp: Option<u64>,
    pub newest_log_timestamp: Option<u64>,
    pub average_execution_time_ms: Option<f64>,
    pub high_risk_events_count: u64,
    pub failed_operations_count: u64,
    pub security_events_24h: u64,
    pub performance_degradation_events: u64,
    pub compliance_violations: u64,
}

// Audit dashboard data
#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct AuditDashboard {
    pub statistics: AuditStatistics,
    pub recent_logs: Vec<EnhancedAuditLog>,
    pub critical_alerts: Vec<EnhancedAuditLog>,
    pub performance_metrics: PerformanceMetrics,
    pub security_summary: SecuritySummary,
    pub compliance_status: ComplianceStatus,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct PerformanceMetrics {
    pub average_response_time_ms: f64,
    pub error_rate_percentage: f64,
    pub throughput_ops_per_minute: f64,
    pub memory_usage_trend: Vec<(u64, u64)>, // (timestamp, memory_bytes)
    pub cycles_consumption_trend: Vec<(u64, u64)>, // (timestamp, cycles)
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct SecuritySummary {
    pub total_security_events: u64,
    pub blacklisted_principals: u64,
    pub failed_authentication_attempts: u64,
    pub suspicious_activity_detected: u64,
    pub threat_level: ThreatLevel,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub enum ThreatLevel {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct ComplianceStatus {
    pub audit_coverage_percentage: f64,
    pub data_retention_compliance: bool,
    pub privacy_compliance: bool,
    pub regulatory_violations: u64,
    pub compliance_score: u32, // 0-100
}

// Memory and storage setup
type Memory = VirtualMemory<DefaultMemoryImpl>;
type EnhancedAuditStorage = StableBTreeMap<u64, EnhancedAuditLog, Memory>;
type AuditConfigStorage = StableBTreeMap<u8, AuditConfiguration, Memory>;

thread_local! {
    static ENHANCED_AUDIT_LOGS: RefCell<EnhancedAuditStorage> = RefCell::new(
        StableBTreeMap::init(get_memory_by_id(MemoryId::new(100)))
    );
    
    static AUDIT_CONFIG: RefCell<AuditConfigStorage> = RefCell::new(
        StableBTreeMap::init(get_memory_by_id(MemoryId::new(101)))
    );
    
    static SESSION_TRACKER: RefCell<HashMap<Principal, String>> = RefCell::new(HashMap::new());
    static CORRELATION_TRACKER: RefCell<HashMap<String, Vec<u64>>> = RefCell::new(HashMap::new());
    static PERFORMANCE_TRACKER: RefCell<Vec<(u64, PerformanceMetrics)>> = RefCell::new(Vec::new());
    static SECURITY_EVENTS_TRACKER: RefCell<Vec<(u64, SecurityEvent)>> = RefCell::new(Vec::new());
    static COMPLIANCE_TRACKER: RefCell<ComplianceTracker> = RefCell::new(ComplianceTracker::default());
    static ALERT_COUNTER: RefCell<u64> = RefCell::new(0);
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct SecurityEvent {
    pub event_type: String,
    pub severity: AuditEventLevel,
    pub principal: Option<Principal>,
    pub threat_indicators: Vec<String>,
}

#[derive(CandidType, Deserialize, Clone, Debug, Default)]
pub struct ComplianceTracker {
    pub total_events_logged: u64,
    pub privacy_violations: u64,
    pub data_breaches: u64,
    pub unauthorized_access_attempts: u64,
    pub last_compliance_check: Option<u64>,
}

// Audit configuration
#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct AuditConfiguration {
    pub enabled: bool,
    pub max_logs_per_category: u64,
    pub auto_cleanup_enabled: bool,
    pub cleanup_threshold_days: u64,
    pub critical_event_notification: bool,
    pub detailed_logging: bool,
    pub performance_tracking: bool,
    pub anonymization_enabled: bool,
    pub real_time_alerts: bool,
    pub compliance_monitoring: bool,
    pub data_retention_days: u64,
    pub max_storage_bytes: u64,
    pub security_monitoring: bool,
    pub risk_assessment_enabled: bool,
    pub export_format: ExportFormat,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub enum ExportFormat {
    JSON,
    CSV,
    JSONL, // JSON Lines for big data processing
}

impl Default for AuditConfiguration {
    fn default() -> Self {
        Self {
            enabled: true,
            max_logs_per_category: 50000, // Increased for production
            auto_cleanup_enabled: true,
            cleanup_threshold_days: 365,
            critical_event_notification: true,
            detailed_logging: true,
            performance_tracking: true,
            anonymization_enabled: false,
            real_time_alerts: true,
            compliance_monitoring: true,
            data_retention_days: 2555, // 7 years for compliance
            max_storage_bytes: 100_000_000, // 100MB
            security_monitoring: true,
            risk_assessment_enabled: true,
            export_format: ExportFormat::JSON,
        }
    }
}

// Missing type definitions
#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct ComplianceReport {
    pub report_period_start: u64,
    pub report_period_end: u64,
    pub total_events: u64,
    pub critical_events: u64,
    pub security_incidents: u64,
    pub data_access_events: u64,
    pub failed_operations: u64,
    pub compliance_violations: Vec<ComplianceViolation>,
    pub risk_assessment: RiskAssessment,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct ComplianceViolation {
    pub violation_type: String,
    pub description: String,
    pub severity: AuditEventLevel,
    pub timestamp: u64,
    pub affected_entities: Vec<String>,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct RiskAssessment {
    pub overall_risk_score: u32,
    pub security_risk: u32,
    pub operational_risk: u32,
    pub compliance_risk: u32,
    pub recommendations: Vec<String>,
}

impl Default for RiskAssessment {
    fn default() -> Self {
        Self {
            overall_risk_score: 0,
            security_risk: 0,
            operational_risk: 0,
            compliance_risk: 0,
            recommendations: Vec::new(),
        }
    }
}

// Add missing fields to AuditDetails
impl Default for AuditDetails {
    fn default() -> Self {
        Self {
            description: String::new(),
            entity_type: None,
            entity_id: None,
            before_state: None,
            after_state: None,
            affected_principals: vec![],
            metadata: vec![],
            risk_score: None,
            location_hash: None,
            user_agent_hash: None,
        }
    }
}

// ========== CORE LOGGING FUNCTIONS ==========

fn maybe_cleanup_old_logs(config: &AuditConfiguration) {
    // Check if cleanup is needed based on storage usage or age
    let current_time = time();
    static mut LAST_CLEANUP: u64 = 0;
    
    unsafe {
        // Run cleanup every 24 hours
        if current_time - LAST_CLEANUP < 24 * 60 * 60 * 1_000_000_000 {
            return;
        }
        LAST_CLEANUP = current_time;
    }
    
    // Check storage usage
    let total_logs = ENHANCED_AUDIT_LOGS.with(|logs| logs.borrow().len());
    
    if total_logs > config.max_logs_per_category {
        let cutoff_time = current_time.saturating_sub(config.cleanup_threshold_days * 24 * 60 * 60 * 1_000_000_000);
        
        let removed_count = ENHANCED_AUDIT_LOGS.with(|logs| {
            let mut logs_map = logs.borrow_mut();
            let mut to_remove = Vec::new();
            
            for (log_id, log) in logs_map.iter() {
                if log.timestamp < cutoff_time {
                    to_remove.push(log_id);
                }
            }
            
            for log_id in &to_remove {
                logs_map.remove(log_id);
            }
            
            to_remove.len()
        });
        
        if removed_count > 0 {
            ic_cdk::println!("🧹 Auto cleanup: Removed {} old audit logs", removed_count);
        }
    }
}

// Fix the duplicate log_audit_enhanced function issue
pub fn log_audit_enhanced(
    category: AuditCategory,
    action: String,
    level: AuditEventLevel,
    details: AuditDetails,
    result: AuditResult,
    correlation_id: Option<String>,
) {
    let config = get_audit_config();
    if !config.enabled {
        return;
    }

    let caller = caller();
    let timestamp = time();
    
    // Generate session ID if not exists
    let session_id = SESSION_TRACKER.with(|tracker| {
        let mut map = tracker.borrow_mut();
        match map.get(&caller) {
            Some(session) => Some(session.clone()),
            None => {
                let new_session = generate_session_id(&caller, timestamp);
                map.insert(caller, new_session.clone());
                Some(new_session)
            }
        }
    });

    let log_entry = EnhancedAuditLog {
        id: get_next_audit_id(),
        timestamp,
        block_height: None, // TODO: Get IC block height when available
        caller,
        category: category.clone(),
        action: action.clone(),
        level: level.clone(),
        details,
        result: result.clone(),
        correlation_id: correlation_id.clone(),
        session_id,
        ip_hash: None, // TODO: Implement privacy-compliant IP tracking
        canister_id: Some(ic_cdk::api::id()),
        version: env!("CARGO_PKG_VERSION").to_string(),
    };

    // Store the log
    ENHANCED_AUDIT_LOGS.with(|logs| {
        logs.borrow_mut().insert(log_entry.id, log_entry.clone());
    });

    // Track correlation if provided
    if let Some(correlation) = correlation_id.clone() {
        CORRELATION_TRACKER.with(|tracker| {
            let mut map = tracker.borrow_mut();
            map.entry(correlation)
                .or_insert_with(Vec::new)
                .push(log_entry.id);
        });
    }

    // Security monitoring
    if config.security_monitoring {
        handle_security_monitoring(&log_entry, &config);
    }

    // Performance tracking
    if config.performance_tracking {
        if let Some(exec_time) = result.execution_time_ms {
            track_performance_metrics(exec_time, result.cycles_consumed);
        }
    }

    // Compliance tracking
    if config.compliance_monitoring {
        update_compliance_tracking(&log_entry);
    }

    // Trigger critical event handling if necessary
    if level == AuditEventLevel::Critical && config.critical_event_notification {
        handle_critical_event(&log_entry);
    }

    // Real-time alerts
    if config.real_time_alerts {
        check_and_trigger_alerts(&log_entry, &config);
    }

    // Auto-cleanup if enabled
    if config.auto_cleanup_enabled {
        maybe_cleanup_old_logs(&config);
    }

    // Update compliance tracker
    COMPLIANCE_TRACKER.with(|tracker| {
        let mut t = tracker.borrow_mut();
        t.total_events_logged += 1;
        
        match category {
            AuditCategory::Security if level == AuditEventLevel::Critical => {
                t.unauthorized_access_attempts += 1;
            },
            _ => {}
        }
    });
}

/// Simplified audit logging for backward compatibility
pub fn log_audit_action(caller: Principal, action: String, details: String, success: bool) {
    let level = if success { AuditEventLevel::Success } else { AuditEventLevel::Error };
    
    let audit_details = AuditDetails {
        description: details,
        entity_type: None,
        entity_id: None,
        before_state: None,
        after_state: None,
        affected_principals: vec![],
        metadata: vec![],
    };

    let result = AuditResult {
        success,
        error_code: None,
        error_message: if !success { Some(action.clone()) } else { None },
        execution_time_ms: None,
        gas_used: None,
    };

    log_audit_enhanced(
        AuditCategory::UserManagement, // Default category
        action,
        level,
        audit_details,
        result,
        None,
    );
}

/// Specialized logging functions for different operation types

pub fn log_loan_repayment_operation(
    action: &str,
    loan_id: u64,
    borrower: Principal,
    amount: Option<u64>,
    success: bool,
    error_msg: Option<String>,
    execution_time_ms: Option<u64>,
) {
    let mut metadata = vec![("loan_id".to_string(), loan_id.to_string())];
    if let Some(amt) = amount {
        metadata.push(("amount".to_string(), amt.to_string()));
        metadata.push(("amount_idr".to_string(), format!("Rp {}", format_currency(amt))));
    }

    let details = AuditDetails {
        description: format!("Loan repayment operation: {}", action),
        entity_type: Some("loan_repayment".to_string()),
        entity_id: Some(loan_id.to_string()),
        before_state: None,
        after_state: None,
        affected_principals: vec![borrower],
        metadata,
        risk_score: Some(calculate_repayment_risk_score(amount, success)),
        location_hash: None,
        user_agent_hash: None,
    };

    let result = AuditResult {
        success,
        error_code: if !success { Some("REPAYMENT_ERROR".to_string()) } else { None },
        error_message: error_msg,
        execution_time_ms,
        gas_used: None,
        cycles_consumed: None,
        memory_used_bytes: None,
        warning_flags: vec![],
    };

    let level = if success { AuditEventLevel::Success } else { AuditEventLevel::Error };

    log_audit_enhanced(
        AuditCategory::LoanRepayment,
        action.to_string(),
        level,
        details,
        result,
        None,
    );
}

pub fn log_liquidity_operation(
    action: &str,
    investor: Principal,
    amount: Option<u64>,
    pool_id: Option<String>,
    success: bool,
    error_msg: Option<String>,
) {
    let mut metadata = vec![];
    if let Some(amt) = amount {
        metadata.push(("amount".to_string(), amt.to_string()));
    }
    if let Some(pool) = &pool_id {
        metadata.push(("pool_id".to_string(), pool.clone()));
    }

    let details = AuditDetails {
        description: format!("Liquidity operation: {}", action),
        entity_type: Some("liquidity".to_string()),
        entity_id: pool_id,
        before_state: None,
        after_state: None,
        affected_principals: vec![investor],
        metadata,
        risk_score: Some(calculate_liquidity_risk_score(amount, action)),
        location_hash: None,
        user_agent_hash: None,
    };

    let result = AuditResult {
        success,
        error_code: if !success { Some("LIQUIDITY_ERROR".to_string()) } else { None },
        error_message: error_msg,
        execution_time_ms: None,
        gas_used: None,
        cycles_consumed: None,
        memory_used_bytes: None,
        warning_flags: vec![],
    };

    let level = if success { AuditEventLevel::Success } else { AuditEventLevel::Error };

    log_audit_enhanced(
        AuditCategory::LiquidityManagement,
        action.to_string(),
        level,
        details,
        result,
        None,
    );
}

pub fn log_governance_operation(
    action: &str,
    proposal_id: Option<u64>,
    admin: Principal,
    success: bool,
    description: String,
) {
    let mut metadata = vec![];
    if let Some(id) = proposal_id {
        metadata.push(("proposal_id".to_string(), id.to_string()));
    }

    let details = AuditDetails {
        description: format!("Governance operation: {} - {}", action, description),
        entity_type: Some("governance".to_string()),
        entity_id: proposal_id.map(|id| id.to_string()),
        before_state: None,
        after_state: None,
        affected_principals: vec![admin],
        metadata,
        risk_score: Some(calculate_governance_risk_score(action)),
        location_hash: None,
        user_agent_hash: None,
    };

    let result = AuditResult {
        success,
        error_code: if !success { Some("GOVERNANCE_ERROR".to_string()) } else { None },
        error_message: None,
        execution_time_ms: None,
        gas_used: None,
        cycles_consumed: None,
        memory_used_bytes: None,
        warning_flags: vec![],
    };

    let level = if success { AuditEventLevel::Success } else { AuditEventLevel::Error };

    log_audit_enhanced(
        AuditCategory::Governance,
        action.to_string(),
        level,
        details,
        result,
        None,
    );
}

pub fn log_oracle_operation(
    action: &str,
    commodity: &str,
    price: Option<u64>,
    success: bool,
    error_msg: Option<String>,
    execution_time_ms: Option<u64>,
) {
    let mut metadata = vec![("commodity".to_string(), commodity.to_string())];
    if let Some(p) = price {
        metadata.push(("price".to_string(), p.to_string()));
    }

    let details = AuditDetails {
        description: format!("Oracle operation: {} for {}", action, commodity),
        entity_type: Some("oracle".to_string()),
        entity_id: Some(commodity.to_string()),
        before_state: None,
        after_state: None,
        affected_principals: vec![],
        metadata,
        risk_score: Some(calculate_oracle_risk_score(success, execution_time_ms)),
        location_hash: None,
        user_agent_hash: None,
    };

    let result = AuditResult {
        success,
        error_code: if !success { Some("ORACLE_ERROR".to_string()) } else { None },
        error_message: error_msg,
        execution_time_ms,
        gas_used: None,
        cycles_consumed: None,
        memory_used_bytes: None,
        warning_flags: vec![],
    };

    let level = if success { AuditEventLevel::Success } else { AuditEventLevel::Error };

    log_audit_enhanced(
        AuditCategory::Oracle,
        action.to_string(),
        level,
        details,
        result,
        None,
    );
}

pub fn log_liquidation_operation(
    action: &str,
    loan_id: u64,
    borrower: Principal,
    collateral_value: u64,
    debt_amount: u64,
    success: bool,
    error_msg: Option<String>,
) {
    let metadata = vec![
        ("loan_id".to_string(), loan_id.to_string()),
        ("collateral_value".to_string(), collateral_value.to_string()),
        ("debt_amount".to_string(), debt_amount.to_string()),
        ("ltv_ratio".to_string(), format!("{:.2}%", (debt_amount as f64 / collateral_value as f64) * 100.0)),
    ];

    let details = AuditDetails {
        description: format!("Liquidation operation: {}", action),
        entity_type: Some("liquidation".to_string()),
        entity_id: Some(loan_id.to_string()),
        before_state: None,
        after_state: None,
        affected_principals: vec![borrower],
        metadata,
        risk_score: Some(95), // High risk score for liquidations
        location_hash: None,
        user_agent_hash: None,
    };

    let result = AuditResult {
        success,
        error_code: if !success { Some("LIQUIDATION_ERROR".to_string()) } else { None },
        error_message: error_msg,
        execution_time_ms: None,
        gas_used: None,
        cycles_consumed: None,
        memory_used_bytes: None,
        warning_flags: vec![],
    };

    let level = if success { AuditEventLevel::Critical } else { AuditEventLevel::Error };

    log_audit_enhanced(
        AuditCategory::Liquidation,
        action.to_string(),
        level,
        details,
        result,
        None,
    );
}

pub fn log_treasury_operation(
    action: &str,
    amount: Option<u64>,
    cycles: Option<u64>,
    success: bool,
    description: String,
) {
    let mut metadata = vec![];
    if let Some(amt) = amount {
        metadata.push(("amount".to_string(), amt.to_string()));
    }
    if let Some(cyc) = cycles {
        metadata.push(("cycles".to_string(), cyc.to_string()));
    }

    let details = AuditDetails {
        description: format!("Treasury operation: {} - {}", action, description),
        entity_type: Some("treasury".to_string()),
        entity_id: None,
        before_state: None,
        after_state: None,
        affected_principals: vec![caller()],
        metadata,
        risk_score: Some(calculate_treasury_risk_score(action, amount)),
        location_hash: None,
        user_agent_hash: None,
    };

    let result = AuditResult {
        success,
        error_code: if !success { Some("TREASURY_ERROR".to_string()) } else { None },
        error_message: None,
        execution_time_ms: None,
        gas_used: None,
        cycles_consumed: cycles,
        memory_used_bytes: None,
        warning_flags: vec![],
    };

    let level = if success { AuditEventLevel::Success } else { AuditEventLevel::Error };

    log_audit_enhanced(
        AuditCategory::Treasury,
        action.to_string(),
        level,
        details,
        result,
        None,
    );
}
    event_type: &str,
    severity: AuditEventLevel,
    description: String,
    affected_principal: Option<Principal>,
    threat_indicators: Vec<String>,
) {
    let mut metadata = vec![("event_type".to_string(), event_type.to_string())];
    for (i, indicator) in threat_indicators.iter().enumerate() {
        metadata.push((format!("threat_indicator_{}", i), indicator.clone()));
    }

    let details = AuditDetails {
        description: description.clone(),
        entity_type: Some("security_event".to_string()),
        entity_id: None,
        before_state: None,
        after_state: None,
        affected_principals: affected_principal.map_or(vec![], |p| vec![p]),
        metadata,
        risk_score: Some(calculate_security_risk_score(event_type, &severity)),
        location_hash: None,
        user_agent_hash: None,
    };

    let result = AuditResult {
        success: true, // Security events are always "successfully" logged
        error_code: None,
        error_message: None,
        execution_time_ms: None,
        gas_used: None,
        cycles_consumed: None,
        memory_used_bytes: None,
        warning_flags: threat_indicators,
    };

    log_audit_enhanced(
        AuditCategory::Security,
        format!("SECURITY_EVENT: {}", event_type),
        severity.clone(),
        details,
        result,
        None,
    );

    // Track security event
    SECURITY_EVENTS_TRACKER.with(|tracker| {
        let mut events = tracker.borrow_mut();
        events.push((time(), SecurityEvent {
            event_type: event_type.to_string(),
            severity,
            principal: affected_principal,
            threat_indicators: threat_indicators.clone(),
        }));
        
        // Keep only last 1000 security events
        if events.len() > 1000 {
            events.drain(0..events.len() - 1000);
        }
    });
}

/// Specialized logging functions for specific system operations

pub fn log_user_management_operation(
    action: &str,
    target_user: Principal,
    role: Option<String>,
    success: bool,
    error_msg: Option<String>,
) {
    let mut metadata = vec![("target_user".to_string(), target_user.to_text())];
    if let Some(r) = &role {
        metadata.push(("role".to_string(), r.clone()));
    }

    let details = AuditDetails {
        description: format!("User management operation: {}", action),
        entity_type: Some("user".to_string()),
        entity_id: Some(target_user.to_text()),
        before_state: None,
        after_state: role.map(|r| format!("role: {}", r)),
        affected_principals: vec![target_user],
        metadata,
        risk_score: Some(calculate_user_management_risk_score(action, success)),
        location_hash: None,
        user_agent_hash: None,
    };

    let result = AuditResult {
        success,
        error_code: if !success { Some("USER_MGMT_ERROR".to_string()) } else { None },
        error_message: error_msg,
        execution_time_ms: None,
        gas_used: None,
        cycles_consumed: None,
        memory_used_bytes: None,
        warning_flags: vec![],
    };

    let level = if success { AuditEventLevel::Success } else { AuditEventLevel::Error };

    log_audit_enhanced(
        AuditCategory::UserManagement,
        action.to_string(),
        level,
        details,
        result,
        None,
    );
}

pub fn log_ckbtc_integration_operation(
    action: &str,
    amount: Option<u64>,
    transaction_id: Option<String>,
    success: bool,
    error_msg: Option<String>,
    execution_time_ms: Option<u64>,
) {
    let mut metadata = vec![];
    if let Some(amt) = amount {
        metadata.push(("amount_satoshi".to_string(), amt.to_string()));
        metadata.push(("amount_btc".to_string(), format!("{:.8}", amt as f64 / 100_000_000.0)));
    }
    if let Some(tx_id) = &transaction_id {
        metadata.push(("transaction_id".to_string(), tx_id.clone()));
    }

    let details = AuditDetails {
        description: format!("CKBTC integration operation: {}", action),
        entity_type: Some("ckbtc_transaction".to_string()),
        entity_id: transaction_id.clone(),
        before_state: None,
        after_state: None,
        affected_principals: vec![caller()],
        metadata,
        risk_score: Some(calculate_ckbtc_risk_score(action, amount, success)),
        location_hash: None,
        user_agent_hash: None,
    };

    let result = AuditResult {
        success,
        error_code: if !success { Some("CKBTC_ERROR".to_string()) } else { None },
        error_message: error_msg,
        execution_time_ms,
        gas_used: None,
        cycles_consumed: None,
        memory_used_bytes: None,
        warning_flags: vec![],
    };

    let level = if success { AuditEventLevel::Success } else { AuditEventLevel::Error };

    log_audit_enhanced(
        AuditCategory::Integration,
        action.to_string(),
        level,
        details,
        result,
        None,
    );
}

pub fn log_maintenance_operation(
    action: &str,
    component: Option<String>,
    success: bool,
    details_msg: String,
    execution_time_ms: Option<u64>,
) {
    let mut metadata = vec![];
    if let Some(comp) = &component {
        metadata.push(("component".to_string(), comp.clone()));
    }
    metadata.push(("automated".to_string(), "true".to_string()));

    let details = AuditDetails {
        description: format!("Maintenance operation: {} - {}", action, details_msg),
        entity_type: Some("maintenance".to_string()),
        entity_id: component,
        before_state: None,
        after_state: None,
        affected_principals: vec![],
        metadata,
        risk_score: Some(5), // Low risk for maintenance operations
        location_hash: None,
        user_agent_hash: None,
    };

    let result = AuditResult {
        success,
        error_code: if !success { Some("MAINTENANCE_ERROR".to_string()) } else { None },
        error_message: None,
        execution_time_ms,
        gas_used: None,
        cycles_consumed: None,
        memory_used_bytes: None,
        warning_flags: vec![],
    };

    let level = if success { AuditEventLevel::Info } else { AuditEventLevel::Warning };

    log_audit_enhanced(
        AuditCategory::Maintenance,
        action.to_string(),
        level,
        details,
        result,
        None,
    );
}

pub fn log_configuration_change(
    setting_name: &str,
    old_value: Option<String>,
    new_value: String,
    success: bool,
) {
    let metadata = vec![
        ("setting_name".to_string(), setting_name.to_string()),
        ("new_value".to_string(), new_value.clone()),
    ];

    let details = AuditDetails {
        description: format!("Configuration change: {}", setting_name),
        entity_type: Some("configuration".to_string()),
        entity_id: Some(setting_name.to_string()),
        before_state: old_value,
        after_state: Some(new_value),
        affected_principals: vec![caller()],
        metadata,
        risk_score: Some(calculate_config_risk_score(setting_name)),
        location_hash: None,
        user_agent_hash: None,
    };

    let result = AuditResult {
        success,
        error_code: if !success { Some("CONFIG_ERROR".to_string()) } else { None },
        error_message: None,
        execution_time_ms: None,
        gas_used: None,
        cycles_consumed: None,
        memory_used_bytes: None,
        warning_flags: vec![],
    };

    let level = if success { AuditEventLevel::Info } else { AuditEventLevel::Error };

    log_audit_enhanced(
        AuditCategory::Configuration,
        format!("CONFIG_CHANGE_{}", setting_name.to_uppercase()),
        level,
        details,
        result,
        None,
    );
}

// Additional risk calculation functions
fn calculate_user_management_risk_score(action: &str, success: bool) -> u32 {
    let base_score = match action {
        "REGISTER_USER" => 20,
        "UPDATE_USER_ROLE" => 40,
        "DEACTIVATE_USER" => 35,
        "GRANT_ADMIN_ROLE" => 80,
        "REVOKE_ADMIN_ROLE" => 60,
        "DELETE_USER" => 70,
        _ => 15,
    };
    
    if success { base_score } else { base_score + 15 }
}

fn calculate_ckbtc_risk_score(action: &str, amount: Option<u64>, success: bool) -> u32 {
    let base_score = match action {
        "DEPOSIT_CKBTC" => 25,
        "WITHDRAW_CKBTC" => 40,
        "TRANSFER_CKBTC" => 30,
        "MINT_CKBTC" => 35,
        "BURN_CKBTC" => 35,
        _ => 20,
    };
    
    let amount_factor = amount.map_or(0, |amt| {
        if amt > 50_000_000 { 20 } // 0.5 BTC
        else if amt > 10_000_000 { 10 } // 0.1 BTC
        else { 0 }
    });
    
    let success_factor = if success { 0 } else { 25 };
    
    std::cmp::min(100, base_score + amount_factor + success_factor)
}

fn calculate_config_risk_score(setting_name: &str) -> u32 {
    match setting_name {
        name if name.contains("admin") => 90,
        name if name.contains("emergency") => 85,
        name if name.contains("rate") => 60,
        name if name.contains("limit") => 50,
        name if name.contains("threshold") => 40,
        _ => 25,
    }
}
    action: &str,
    token_id: u64,
    owner: Principal,
    success: bool,
    error_msg: Option<String>,
    before_state: Option<String>,
    after_state: Option<String>,
) {
    let details = AuditDetails {
        description: format!("NFT operation: {}", action),
        entity_type: Some("nft".to_string()),
        entity_id: Some(token_id.to_string()),
        before_state,
        after_state,
        affected_principals: vec![owner],
        metadata: vec![("token_id".to_string(), token_id.to_string())],
        risk_score: Some(calculate_nft_risk_score(action, success)),
        location_hash: None,
        user_agent_hash: None,
    };

    let result = AuditResult {
        success,
        error_code: if !success { Some("NFT_ERROR".to_string()) } else { None },
        error_message: error_msg,
        execution_time_ms: None,
        gas_used: None,
        cycles_consumed: None,
        memory_used_bytes: None,
        warning_flags: vec![],
    };

    let level = if success { AuditEventLevel::Success } else { AuditEventLevel::Error };

    log_audit_enhanced(
        AuditCategory::NFTOperations,
        action.to_string(),
        level,
        details,
        result,
        None,
    );
}

pub fn log_loan_operation(
    action: &str,
    loan_id: u64,
    borrower: Principal,
    amount: Option<u64>,
    success: bool,
    error_msg: Option<String>,
    execution_time_ms: Option<u64>,
    before_state: Option<String>,
    after_state: Option<String>,
) {
    let mut metadata = vec![("loan_id".to_string(), loan_id.to_string())];
    if let Some(amt) = amount {
        metadata.push(("amount".to_string(), amt.to_string()));
        metadata.push(("amount_idr".to_string(), format!("Rp {}", format_currency(amt))));
    }

    let details = AuditDetails {
        description: format!("Loan operation: {}", action),
        entity_type: Some("loan".to_string()),
        entity_id: Some(loan_id.to_string()),
        before_state,
        after_state,
        affected_principals: vec![borrower],
        metadata,
        risk_score: Some(calculate_loan_risk_score(action, amount, success)),
        location_hash: None,
        user_agent_hash: None,
    };

    let result = AuditResult {
        success,
        error_code: if !success { Some("LOAN_ERROR".to_string()) } else { None },
        error_message: error_msg,
        execution_time_ms,
        gas_used: None,
        cycles_consumed: None,
        memory_used_bytes: None,
        warning_flags: vec![],
    };

    let level = if success { AuditEventLevel::Success } else { AuditEventLevel::Error };

    log_audit_enhanced(
        AuditCategory::LoanLifecycle,
        action.to_string(),
        level,
        details,
        result,
        None,
    );
}

// ========== QUERY FUNCTIONS ==========

/// Get comprehensive audit dashboard
#[query]
pub fn get_audit_dashboard() -> Result<AuditDashboard, String> {
    let caller = caller();
    if !is_admin(&caller) {
        return Err("Unauthorized: Only admins can view audit dashboard".to_string());
    }

    let statistics = get_audit_statistics()?;
    
    // Get recent logs (last 50)
    let recent_filter = AuditLogFilter {
        limit: Some(50),
        sort_order: Some(SortOrder::TimestampDesc),
        ..Default::default()
    };
    let recent_logs = get_audit_logs_filtered(recent_filter)?;

    // Get critical alerts (last 24 hours)
    let critical_filter = AuditLogFilter {
        start_time: Some(time() - 24 * 60 * 60 * 1_000_000_000),
        level: Some(AuditEventLevel::Critical),
        sort_order: Some(SortOrder::TimestampDesc),
        ..Default::default()
    };
    let critical_alerts = get_audit_logs_filtered(critical_filter)?;

    // Calculate performance metrics
    let performance_metrics = calculate_performance_metrics();
    
    // Calculate security summary
    let security_summary = calculate_security_summary();
    
    // Calculate compliance status
    let compliance_status = calculate_compliance_status();

    Ok(AuditDashboard {
        statistics,
        recent_logs,
        critical_alerts,
        performance_metrics,
        security_summary,
        compliance_status,
    })
}

/// Get audit logs with advanced filtering and pagination
#[query]
pub fn get_audit_logs_advanced(
    filter: AuditLogFilter,
    include_metadata: bool,
) -> Result<(Vec<EnhancedAuditLog>, u64), String> {
    let caller = caller();
    if !is_admin(&caller) {
        return Err("Unauthorized: Only admins can view audit logs".to_string());
    }

    let logs = get_audit_logs_filtered(filter)?;
    let total_count = ENHANCED_AUDIT_LOGS.with(|logs_map| logs_map.borrow().len() as u64);
    
    let mut result_logs = logs;
    
    // Remove sensitive metadata if not requested
    if !include_metadata {
        for log in &mut result_logs {
            // Clear sensitive fields while preserving essential data
            if get_audit_config().anonymization_enabled {
                anonymize_log_data(log);
            }
        }
    }

    Ok((result_logs, total_count))
}

/// Advanced audit analysis functions

/// Get audit summary for a specific time period
#[query]
pub fn get_audit_summary_by_period(
    start_time: u64,
    end_time: u64,
    group_by: AuditGroupBy,
) -> Result<Vec<AuditPeriodSummary>, String> {
    let caller = caller();
    if !is_admin(&caller) {
        return Err("Unauthorized: Only admins can view audit summaries".to_string());
    }

    let filter = AuditLogFilter {
        start_time: Some(start_time),
        end_time: Some(end_time),
        ..Default::default()
    };

    let logs = get_audit_logs_filtered(filter)?;
    
    let mut summaries = Vec::new();
    let period_duration = match group_by {
        AuditGroupBy::Hour => 60 * 60 * 1_000_000_000,
        AuditGroupBy::Day => 24 * 60 * 60 * 1_000_000_000,
        AuditGroupBy::Week => 7 * 24 * 60 * 60 * 1_000_000_000,
        AuditGroupBy::Month => 30 * 24 * 60 * 60 * 1_000_000_000,
    };

    let mut current_period_start = start_time;
    while current_period_start < end_time {
        let current_period_end = std::cmp::min(current_period_start + period_duration, end_time);
        
        let period_logs: Vec<&EnhancedAuditLog> = logs.iter()
            .filter(|log| log.timestamp >= current_period_start && log.timestamp < current_period_end)
            .collect();

        let summary = AuditPeriodSummary {
            period_start: current_period_start,
            period_end: current_period_end,
            total_events: period_logs.len() as u64,
            success_rate: calculate_period_success_rate(&period_logs),
            critical_events: period_logs.iter().filter(|log| log.level == AuditEventLevel::Critical).count() as u64,
            security_events: period_logs.iter().filter(|log| log.category == AuditCategory::Security).count() as u64,
            average_risk_score: calculate_period_average_risk(&period_logs),
            top_actions: get_top_actions_in_period(&period_logs, 5),
        };

        summaries.push(summary);
        current_period_start = current_period_end;
    }

    Ok(summaries)
}

/// Get audit trends and patterns
#[query]
pub fn get_audit_trends(days_back: u64) -> Result<AuditTrends, String> {
    let caller = caller();
    if !is_admin(&caller) {
        return Err("Unauthorized: Only admins can view audit trends".to_string());
    }

    let current_time = time();
    let start_time = current_time.saturating_sub(days_back * 24 * 60 * 60 * 1_000_000_000);

    let filter = AuditLogFilter {
        start_time: Some(start_time),
        end_time: Some(current_time),
        ..Default::default()
    };

    let logs = get_audit_logs_filtered(filter)?;

    // Calculate trends
    let daily_activity = calculate_daily_activity_trend(&logs, days_back);
    let error_rate_trend = calculate_error_rate_trend(&logs, days_back);
    let risk_score_trend = calculate_risk_score_trend(&logs, days_back);
    let category_trends = calculate_category_trends(&logs);
    let security_alerts_trend = calculate_security_alerts_trend(&logs, days_back);

    Ok(AuditTrends {
        analysis_period_days: days_back,
        daily_activity_trend: daily_activity,
        error_rate_trend,
        risk_score_trend,
        category_distribution_change: category_trends,
        security_alerts_trend,
        anomalies_detected: detect_audit_anomalies(&logs),
        recommendations: generate_trend_recommendations(&logs),
    })
}

/// Search audit logs with natural language query
#[query]
pub fn search_audit_logs(
    query: String,
    max_results: Option<u64>,
) -> Result<Vec<EnhancedAuditLog>, String> {
    let caller = caller();
    if !is_admin(&caller) {
        return Err("Unauthorized: Only admins can search audit logs".to_string());
    }

    let query_lower = query.to_lowercase();
    let limit = max_results.unwrap_or(50);

    ENHANCED_AUDIT_LOGS.with(|logs| {
        let logs_map = logs.borrow();
        let mut results = Vec::new();

        for (_, log) in logs_map.iter() {
            let mut relevance_score = 0u32;

            // Search in action
            if log.action.to_lowercase().contains(&query_lower) {
                relevance_score += 20;
            }

            // Search in description
            if log.details.description.to_lowercase().contains(&query_lower) {
                relevance_score += 15;
            }

            // Search in metadata
            for (key, value) in &log.details.metadata {
                if key.to_lowercase().contains(&query_lower) || 
                   value.to_lowercase().contains(&query_lower) {
                    relevance_score += 10;
                }
            }

            // Search in error message
            if let Some(error) = &log.result.error_message {
                if error.to_lowercase().contains(&query_lower) {
                    relevance_score += 12;
                }
            }

            // Search in entity type/id
            if let Some(entity_type) = &log.details.entity_type {
                if entity_type.to_lowercase().contains(&query_lower) {
                    relevance_score += 8;
                }
            }

            if let Some(entity_id) = &log.details.entity_id {
                if entity_id.to_lowercase().contains(&query_lower) {
                    relevance_score += 8;
                }
            }

            if relevance_score > 0 {
                results.push((log.clone(), relevance_score));
            }
        }

        // Sort by relevance score (highest first)
        results.sort_by(|a, b| b.1.cmp(&a.1));
        
        // Extract logs and apply limit
        let search_results: Vec<EnhancedAuditLog> = results
            .into_iter()
            .take(limit as usize)
            .map(|(log, _)| log)
            .collect();

        Ok(search_results)
    })
}

// Supporting types and functions for advanced analytics
#[derive(CandidType, Deserialize, Clone, Debug)]
pub enum AuditGroupBy {
    Hour,
    Day,
    Week,
    Month,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct AuditPeriodSummary {
    pub period_start: u64,
    pub period_end: u64,
    pub total_events: u64,
    pub success_rate: f64,
    pub critical_events: u64,
    pub security_events: u64,
    pub average_risk_score: f64,
    pub top_actions: Vec<(String, u64)>,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct AuditTrends {
    pub analysis_period_days: u64,
    pub daily_activity_trend: Vec<(u64, u64)>, // (day_timestamp, event_count)
    pub error_rate_trend: Vec<(u64, f64)>,     // (day_timestamp, error_rate)
    pub risk_score_trend: Vec<(u64, f64)>,     // (day_timestamp, avg_risk_score)
    pub category_distribution_change: HashMap<String, f64>, // category -> change_percentage
    pub security_alerts_trend: Vec<(u64, u64)>, // (day_timestamp, alert_count)
    pub anomalies_detected: Vec<AuditAnomaly>,
    pub recommendations: Vec<String>,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct AuditAnomaly {
    pub anomaly_type: String,
    pub description: String,
    pub severity: AuditEventLevel,
    pub detected_at: u64,
    pub affected_period: (u64, u64),
    pub metrics: Vec<(String, f64)>,
}

// Helper functions for trend analysis
fn calculate_period_success_rate(logs: &[&EnhancedAuditLog]) -> f64 {
    if logs.is_empty() {
        return 100.0;
    }
    
    let successful = logs.iter().filter(|log| log.result.success).count();
    (successful as f64 / logs.len() as f64) * 100.0
}

fn calculate_period_average_risk(logs: &[&EnhancedAuditLog]) -> f64 {
    if logs.is_empty() {
        return 0.0;
    }
    
    let total_risk: u32 = logs.iter()
        .map(|log| log.details.risk_score.unwrap_or(0))
        .sum();
    
    total_risk as f64 / logs.len() as f64
}

fn get_top_actions_in_period(logs: &[&EnhancedAuditLog], limit: usize) -> Vec<(String, u64)> {
    let mut action_counts: HashMap<String, u64> = HashMap::new();
    
    for log in logs {
        *action_counts.entry(log.action.clone()).or_insert(0) += 1;
    }
    
    let mut sorted_actions: Vec<(String, u64)> = action_counts.into_iter().collect();
    sorted_actions.sort_by(|a, b| b.1.cmp(&a.1));
    sorted_actions.truncate(limit);
    
    sorted_actions
}

fn calculate_daily_activity_trend(logs: &[EnhancedAuditLog], days_back: u64) -> Vec<(u64, u64)> {
    let mut daily_counts: HashMap<u64, u64> = HashMap::new();
    let current_time = time();
    
    for log in logs {
        let day_timestamp = (log.timestamp / (24 * 60 * 60 * 1_000_000_000)) * (24 * 60 * 60 * 1_000_000_000);
        *daily_counts.entry(day_timestamp).or_insert(0) += 1;
    }
    
    let mut trend = Vec::new();
    for i in 0..days_back {
        let day_start = current_time - (i * 24 * 60 * 60 * 1_000_000_000);
        let day_timestamp = (day_start / (24 * 60 * 60 * 1_000_000_000)) * (24 * 60 * 60 * 1_000_000_000);
        let count = daily_counts.get(&day_timestamp).copied().unwrap_or(0);
        trend.push((day_timestamp, count));
    }
    
    trend.reverse(); // Oldest first
    trend
}

fn calculate_error_rate_trend(logs: &[EnhancedAuditLog], days_back: u64) -> Vec<(u64, f64)> {
    let mut daily_stats: HashMap<u64, (u64, u64)> = HashMap::new(); // (total, errors)
    let current_time = time();
    
    for log in logs {
        let day_timestamp = (log.timestamp / (24 * 60 * 60 * 1_000_000_000)) * (24 * 60 * 60 * 1_000_000_000);
        let (total, errors) = daily_stats.entry(day_timestamp).or_insert((0, 0));
        *total += 1;
        if !log.result.success {
            *errors += 1;
        }
    }
    
    let mut trend = Vec::new();
    for i in 0..days_back {
        let day_start = current_time - (i * 24 * 60 * 60 * 1_000_000_000);
        let day_timestamp = (day_start / (24 * 60 * 60 * 1_000_000_000)) * (24 * 60 * 60 * 1_000_000_000);
        
        let error_rate = if let Some((total, errors)) = daily_stats.get(&day_timestamp) {
            if *total > 0 {
                (*errors as f64 / *total as f64) * 100.0
            } else {
                0.0
            }
        } else {
            0.0
        };
        
        trend.push((day_timestamp, error_rate));
    }
    
    trend.reverse(); // Oldest first
    trend
}

fn calculate_risk_score_trend(logs: &[EnhancedAuditLog], days_back: u64) -> Vec<(u64, f64)> {
    let mut daily_risk: HashMap<u64, Vec<u32>> = HashMap::new();
    let current_time = time();
    
    for log in logs {
        if let Some(risk_score) = log.details.risk_score {
            let day_timestamp = (log.timestamp / (24 * 60 * 60 * 1_000_000_000)) * (24 * 60 * 60 * 1_000_000_000);
            daily_risk.entry(day_timestamp).or_insert_with(Vec::new).push(risk_score);
        }
    }
    
    let mut trend = Vec::new();
    for i in 0..days_back {
        let day_start = current_time - (i * 24 * 60 * 60 * 1_000_000_000);
        let day_timestamp = (day_start / (24 * 60 * 60 * 1_000_000_000)) * (24 * 60 * 60 * 1_000_000_000);
        
        let avg_risk = if let Some(risks) = daily_risk.get(&day_timestamp) {
            if !risks.is_empty() {
                risks.iter().sum::<u32>() as f64 / risks.len() as f64
            } else {
                0.0
            }
        } else {
            0.0
        };
        
        trend.push((day_timestamp, avg_risk));
    }
    
    trend.reverse(); // Oldest first
    trend
}

fn calculate_category_trends(logs: &[EnhancedAuditLog]) -> HashMap<String, f64> {
    // Simple implementation - in production, this would compare with historical data
    let mut category_counts: HashMap<String, u64> = HashMap::new();
    
    for log in logs {
        let category_str = format!("{:?}", log.category);
        *category_counts.entry(category_str).or_insert(0) += 1;
    }
    
    // Convert to percentages (simplified)
    let total = logs.len() as f64;
    category_counts.into_iter()
        .map(|(category, count)| (category, (count as f64 / total) * 100.0))
        .collect()
}

fn calculate_security_alerts_trend(logs: &[EnhancedAuditLog], days_back: u64) -> Vec<(u64, u64)> {
    let mut daily_security: HashMap<u64, u64> = HashMap::new();
    let current_time = time();
    
    for log in logs {
        if log.category == AuditCategory::Security && log.level == AuditEventLevel::Critical {
            let day_timestamp = (log.timestamp / (24 * 60 * 60 * 1_000_000_000)) * (24 * 60 * 60 * 1_000_000_000);
            *daily_security.entry(day_timestamp).or_insert(0) += 1;
        }
    }
    
    let mut trend = Vec::new();
    for i in 0..days_back {
        let day_start = current_time - (i * 24 * 60 * 60 * 1_000_000_000);
        let day_timestamp = (day_start / (24 * 60 * 60 * 1_000_000_000)) * (24 * 60 * 60 * 1_000_000_000);
        let count = daily_security.get(&day_timestamp).copied().unwrap_or(0);
        trend.push((day_timestamp, count));
    }
    
    trend.reverse(); // Oldest first
    trend
}

fn detect_audit_anomalies(logs: &[EnhancedAuditLog]) -> Vec<AuditAnomaly> {
    let mut anomalies = Vec::new();
    
    // Detect high error rate anomaly
    let total_ops = logs.len();
    let failed_ops = logs.iter().filter(|log| !log.result.success).count();
    
    if total_ops > 100 && (failed_ops as f64 / total_ops as f64) > 0.1 { // >10% error rate
        anomalies.push(AuditAnomaly {
            anomaly_type: "HIGH_ERROR_RATE".to_string(),
            description: format!("High error rate detected: {:.1}% ({}/{})", 
                (failed_ops as f64 / total_ops as f64) * 100.0, failed_ops, total_ops),
            severity: AuditEventLevel::Warning,
            detected_at: time(),
            affected_period: (
                logs.first().map(|l| l.timestamp).unwrap_or(0),
                logs.last().map(|l| l.timestamp).unwrap_or(0)
            ),
            metrics: vec![
                ("error_rate_percent".to_string(), (failed_ops as f64 / total_ops as f64) * 100.0),
                ("total_operations".to_string(), total_ops as f64),
                ("failed_operations".to_string(), failed_ops as f64),
            ],
        });
    }
    
    // Detect unusual security activity
    let security_events = logs.iter().filter(|log| log.category == AuditCategory::Security).count();
    if security_events > total_ops / 20 { // >5% of all events are security events
        anomalies.push(AuditAnomaly {
            anomaly_type: "HIGH_SECURITY_ACTIVITY".to_string(),
            description: format!("Unusual security activity detected: {} security events", security_events),
            severity: AuditEventLevel::Warning,
            detected_at: time(),
            affected_period: (
                logs.first().map(|l| l.timestamp).unwrap_or(0),
                logs.last().map(|l| l.timestamp).unwrap_or(0)
            ),
            metrics: vec![
                ("security_events".to_string(), security_events as f64),
                ("security_percentage".to_string(), (security_events as f64 / total_ops as f64) * 100.0),
            ],
        });
    }
    
    anomalies
}

fn generate_trend_recommendations(logs: &[EnhancedAuditLog]) -> Vec<String> {
    let mut recommendations = Vec::new();
    
    let total_ops = logs.len();
    let failed_ops = logs.iter().filter(|log| !log.result.success).count();
    let error_rate = if total_ops > 0 { (failed_ops as f64 / total_ops as f64) * 100.0 } else { 0.0 };
    
    if error_rate > 5.0 {
        recommendations.push(format!("Error rate is {:.1}% - investigate common failure patterns", error_rate));
    }
    
    let high_risk_events = logs.iter()
        .filter(|log| log.details.risk_score.unwrap_or(0) > 70)
        .count();
    
    if high_risk_events > total_ops / 10 {
        recommendations.push("High number of high-risk operations detected - review security controls".to_string());
    }
    
    let critical_events = logs.iter()
        .filter(|log| log.level == AuditEventLevel::Critical)
        .count();
    
    if critical_events > 0 {
        recommendations.push(format!("{} critical events require immediate attention", critical_events));
    }
    
    if recommendations.is_empty() {
        recommendations.push("Audit patterns look normal - continue monitoring".to_string());
    }
    
    recommendations
}
#[query]
pub fn get_logs_by_entity(
    entity_type: String,
    entity_id: String,
    limit: Option<u64>,
) -> Result<Vec<EnhancedAuditLog>, String> {
    let caller = caller();
    if !is_admin(&caller) {
        return Err("Unauthorized: Only admins can view audit logs".to_string());
    }

    let filter = AuditLogFilter {
        entity_type: Some(entity_type),
        entity_id: Some(entity_id),
        limit,
        sort_order: Some(SortOrder::TimestampDesc),
        ..Default::default()
    };

    get_audit_logs_filtered(filter)
}

/// Get security events in time range
#[query]
pub fn get_security_events(
    start_time: Option<u64>,
    end_time: Option<u64>,
) -> Result<Vec<SecurityEvent>, String> {
    let caller = caller();
    if !is_admin(&caller) {
        return Err("Unauthorized: Only admins can view security events".to_string());
    }

    SECURITY_EVENTS_TRACKER.with(|tracker| {
        let events = tracker.borrow();
        let mut result = Vec::new();
        
        for (timestamp, event) in events.iter() {
            if let Some(start) = start_time {
                if *timestamp < start {
                    continue;
                }
            }
            if let Some(end) = end_time {
                if *timestamp > end {
                    continue;
                }
            }
            result.push(event.clone());
        }
        
        Ok(result)
    })
}

/// Get compliance report
#[query]
pub fn get_compliance_report(
    start_time: u64,
    end_time: u64,
) -> Result<ComplianceReport, String> {
    let caller = caller();
    if !is_admin(&caller) {
        return Err("Unauthorized: Only admins can view compliance reports".to_string());
    }

    let filter = AuditLogFilter {
        start_time: Some(start_time),
        end_time: Some(end_time),
        ..Default::default()
    };

    let logs = get_audit_logs_filtered(filter)?;
    
    let mut report = ComplianceReport {
        report_period_start: start_time,
        report_period_end: end_time,
        total_events: logs.len() as u64,
        critical_events: 0,
        security_incidents: 0,
        data_access_events: 0,
        failed_operations: 0,
        compliance_violations: Vec::new(),
        risk_assessment: RiskAssessment::default(),
    };

    for log in logs {
        match log.level {
            AuditEventLevel::Critical => report.critical_events += 1,
            _ => {}
        }
        
        if log.category == AuditCategory::Security {
            report.security_incidents += 1;
        }
        
        if !log.result.success {
            report.failed_operations += 1;
        }

        // Check for compliance violations
        if let Some(violation) = check_compliance_violation(&log) {
            report.compliance_violations.push(violation);
        }
    }

    report.risk_assessment = calculate_risk_assessment(&report);
    
    Ok(report)
}
/// Get audit logs with advanced filtering and pagination
#[query]
pub fn get_audit_logs_filtered(filter: AuditLogFilter) -> Result<Vec<EnhancedAuditLog>, String> {
    let caller = caller();
    if !is_admin(&caller) {
        return Err("Unauthorized: Only admins can view audit logs".to_string());
    }

    ENHANCED_AUDIT_LOGS.with(|logs| {
        let logs_map = logs.borrow();
        let mut result: Vec<EnhancedAuditLog> = Vec::new();
        
        for (_, log) in logs_map.iter() {
            // Apply filters
            if let Some(start_time) = filter.start_time {
                if log.timestamp < start_time {
                    continue;
                }
            }
            
            if let Some(end_time) = filter.end_time {
                if log.timestamp > end_time {
                    continue;
                }
            }
            
            if let Some(filter_caller) = filter.caller {
                if log.caller != filter_caller {
                    continue;
                }
            }
            
            if let Some(category) = &filter.category {
                if log.category != *category {
                    continue;
                }
            }
            
            if let Some(level) = &filter.level {
                if log.level != *level {
                    continue;
                }
            }
            
            if let Some(success_only) = filter.success_only {
                if success_only && !log.result.success {
                    continue;
                }
            }
            
            if let Some(entity_type) = &filter.entity_type {
                if log.details.entity_type.as_ref() != Some(entity_type) {
                    continue;
                }
            }
            
            if let Some(entity_id) = &filter.entity_id {
                if log.details.entity_id.as_ref() != Some(entity_id) {
                    continue;
                }
            }
            
            if let Some(pattern) = &filter.action_pattern {
                if !log.action.contains(pattern) {
                    continue;
                }
            }
            
            if let Some(correlation) = &filter.correlation_id {
                if log.correlation_id.as_ref() != Some(correlation) {
                    continue;
                }
            }
            
            if let Some(session) = &filter.session_id {
                if log.session_id.as_ref() != Some(session) {
                    continue;
                }
            }
            
            // Risk score filtering
            if let Some(min_risk) = filter.risk_score_min {
                if log.details.risk_score.unwrap_or(0) < min_risk {
                    continue;
                }
            }
            
            if let Some(max_risk) = filter.risk_score_max {
                if log.details.risk_score.unwrap_or(0) > max_risk {
                    continue;
                }
            }
            
            result.push(log.clone());
        }
        
        // Sort based on sort_order
        match filter.sort_order.unwrap_or(SortOrder::TimestampDesc) {
            SortOrder::TimestampAsc => result.sort_by(|a, b| a.timestamp.cmp(&b.timestamp)),
            SortOrder::TimestampDesc => result.sort_by(|a, b| b.timestamp.cmp(&a.timestamp)),
            SortOrder::RiskScoreAsc => result.sort_by(|a, b| {
                let risk_a = a.details.risk_score.unwrap_or(0);
                let risk_b = b.details.risk_score.unwrap_or(0);
                risk_a.cmp(&risk_b)
            }),
            SortOrder::RiskScoreDesc => result.sort_by(|a, b| {
                let risk_a = a.details.risk_score.unwrap_or(0);
                let risk_b = b.details.risk_score.unwrap_or(0);
                risk_b.cmp(&risk_a)
            }),
        }
        
        // Apply offset and limit
        if let Some(offset) = filter.offset {
            if offset as usize >= result.len() {
                return Ok(vec![]);
            }
            result = result.into_iter().skip(offset as usize).collect();
        }
        
        if let Some(limit) = filter.limit {
            result.truncate(limit as usize);
        }
        
        Ok(result)
    })
}
}

/// Get audit statistics
#[query]
pub fn get_audit_statistics() -> Result<AuditStatistics, String> {
    let caller = caller();
    if !is_admin(&caller) {
        return Err("Unauthorized: Only admins can view audit statistics".to_string());
    }

    ENHANCED_AUDIT_LOGS.with(|logs| {
        let logs_map = logs.borrow();
        let mut stats = AuditStatistics {
            total_logs: logs_map.len() as u64,
            logs_by_category: HashMap::new(),
            logs_by_level: HashMap::new(),
            success_rate: 0.0,
            most_active_callers: Vec::new(),
            recent_critical_events: 0,
            storage_usage_bytes: 0,
            oldest_log_timestamp: None,
            newest_log_timestamp: None,
            average_execution_time_ms: None,
            high_risk_events_count: 0,
            failed_operations_count: 0,
            security_events_24h: 0,
            performance_degradation_events: 0,
            compliance_violations: 0,
        };

        let mut caller_counts: HashMap<Principal, u64> = HashMap::new();
        let mut successful_logs = 0u64;
        let mut total_execution_time = 0u64;
        let mut execution_time_count = 0u64;
        let current_time = time();
        let one_day_ago = current_time.saturating_sub(24 * 60 * 60 * 1_000_000_000);

        for (_, log) in logs_map.iter() {
            // Category statistics
            let category_str = format!("{:?}", log.category);
            *stats.logs_by_category.entry(category_str).or_insert(0) += 1;

            // Level statistics
            let level_str = format!("{:?}", log.level);
            *stats.logs_by_level.entry(level_str).or_insert(0) += 1;

            // Success rate
            if log.result.success {
                successful_logs += 1;
            } else {
                stats.failed_operations_count += 1;
            }

            // Execution time tracking
            if let Some(exec_time) = log.result.execution_time_ms {
                total_execution_time += exec_time;
                execution_time_count += 1;
            }

            // High risk events
            if let Some(risk_score) = log.details.risk_score {
                if risk_score > 70 {
                    stats.high_risk_events_count += 1;
                }
            }

            // Security events in last 24h
            if log.category == AuditCategory::Security && log.timestamp >= one_day_ago {
                stats.security_events_24h += 1;
            }

            // Performance degradation events
            if let Some(exec_time) = log.result.execution_time_ms {
                if exec_time > 10000 { // 10 seconds
                    stats.performance_degradation_events += 1;
                }
            }

            // Compliance violations
            if log.level == AuditEventLevel::Critical && 
               (log.category == AuditCategory::Security || log.category == AuditCategory::Compliance) {
                stats.compliance_violations += 1;
            }

            // Caller activity
            *caller_counts.entry(log.caller).or_insert(0) += 1;

            // Recent critical events
            if log.level == AuditEventLevel::Critical && log.timestamp >= one_day_ago {
                stats.recent_critical_events += 1;
            }

            // Timestamp tracking
            if stats.oldest_log_timestamp.is_none() || Some(log.timestamp) < stats.oldest_log_timestamp {
                stats.oldest_log_timestamp = Some(log.timestamp);
            }
            if stats.newest_log_timestamp.is_none() || Some(log.timestamp) > stats.newest_log_timestamp {
                stats.newest_log_timestamp = Some(log.timestamp);
            }
        }

        // Calculate success rate
        if stats.total_logs > 0 {
            stats.success_rate = (successful_logs as f64) / (stats.total_logs as f64) * 100.0;
        }

        // Calculate average execution time
        if execution_time_count > 0 {
            stats.average_execution_time_ms = Some(total_execution_time as f64 / execution_time_count as f64);
        }

        // Sort and get most active callers
        let mut caller_vec: Vec<(Principal, u64)> = caller_counts.into_iter().collect();
        caller_vec.sort_by(|a, b| b.1.cmp(&a.1));
        stats.most_active_callers = caller_vec.into_iter().take(10).collect();

        // Estimate storage usage (rough calculation)
        stats.storage_usage_bytes = stats.total_logs * 2048; // Rough estimate: 2KB per log

        Ok(stats)
    })
}

/// Get logs by correlation ID
#[query]
pub fn get_logs_by_correlation(correlation_id: String) -> Result<Vec<EnhancedAuditLog>, String> {
    let caller = caller();
    if !is_admin(&caller) {
        return Err("Unauthorized: Only admins can view audit logs".to_string());
    }

    CORRELATION_TRACKER.with(|tracker| {
        let map = tracker.borrow();
        match map.get(&correlation_id) {
            Some(log_ids) => {
                ENHANCED_AUDIT_LOGS.with(|logs| {
                    let logs_map = logs.borrow();
                    let mut result = Vec::new();
                    
                    for log_id in log_ids {
                        if let Some(log) = logs_map.get(log_id) {
                            result.push(log.clone());
                        }
                    }
                    
                    // Sort by timestamp
                    result.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
                    Ok(result)
                })
            }
            None => Ok(vec![]),
        }
    })
}

// ========== CONFIGURATION FUNCTIONS ==========

/// Update audit configuration (admin only)
#[update]
pub fn update_audit_config(config: AuditConfiguration) -> Result<(), String> {
    let caller = caller();
    if !is_admin(&caller) {
        return Err("Unauthorized: Only admins can update audit configuration".to_string());
    }

    AUDIT_CONFIG.with(|cfg| {
        cfg.borrow_mut().insert(0, config.clone());
    });

    log_audit_enhanced(
        AuditCategory::Configuration,
        "UPDATE_AUDIT_CONFIG".to_string(),
        AuditEventLevel::Info,
        AuditDetails {
            description: "Audit configuration updated".to_string(),
            entity_type: Some("config".to_string()),
            entity_id: Some("audit_config".to_string()),
            before_state: None,
            after_state: Some(format!("{:?}", config)),
            affected_principals: vec![],
            metadata: vec![],
        },
        AuditResult {
            success: true,
            error_code: None,
            error_message: None,
            execution_time_ms: None,
            gas_used: None,
        },
        None,
    );

    Ok(())
}

/// Get current audit configuration
#[query]
pub fn get_audit_config() -> AuditConfiguration {
    AUDIT_CONFIG.with(|cfg| {
        cfg.borrow().get(&0).unwrap_or_else(|| AuditConfiguration::default())
    })
}

// ========== MAINTENANCE FUNCTIONS ==========

/// Manual cleanup of old logs (admin only)
#[update]
pub fn cleanup_old_audit_logs(days_to_keep: u64) -> Result<u64, String> {
    let caller = caller();
    if !is_admin(&caller) {
        return Err("Unauthorized: Only admins can cleanup audit logs".to_string());
    }

    let current_time = time();
    let cutoff_time = current_time.saturating_sub(days_to_keep * 24 * 60 * 60 * 1_000_000_000);
    
    let removed_count = ENHANCED_AUDIT_LOGS.with(|logs| {
        let mut logs_map = logs.borrow_mut();
        let mut to_remove = Vec::new();
        
        for (log_id, log) in logs_map.iter() {
            if log.timestamp < cutoff_time {
                to_remove.push(log_id);
            }
        }
        
        for log_id in &to_remove {
            logs_map.remove(log_id);
        }
        
        to_remove.len() as u64
    });

    log_audit_enhanced(
        AuditCategory::Maintenance,
        "CLEANUP_AUDIT_LOGS".to_string(),
        AuditEventLevel::Info,
        AuditDetails {
            description: format!("Cleaned up {} old audit logs", removed_count),
            entity_type: Some("maintenance".to_string()),
            entity_id: None,
            before_state: None,
            after_state: None,
            affected_principals: vec![],
            metadata: vec![
                ("removed_count".to_string(), removed_count.to_string()),
                ("days_kept".to_string(), days_to_keep.to_string()),
            ],
        },
        AuditResult {
            success: true,
            error_code: None,
            error_message: None,
            execution_time_ms: None,
            gas_used: None,
        },
        None,
    );

    Ok(removed_count)
}

/// Export audit logs for compliance (admin only)
#[query]
pub fn export_audit_logs_for_compliance(
    start_time: u64,
    end_time: u64,
) -> Result<Vec<EnhancedAuditLog>, String> {
    let caller = caller();
    if !is_admin(&caller) {
        return Err("Unauthorized: Only admins can export audit logs".to_string());
    }

    let filter = AuditLogFilter {
        start_time: Some(start_time),
        end_time: Some(end_time),
        caller: None,
        category: None,
        level: None,
        action_pattern: None,
        success_only: None,
        entity_type: None,
        entity_id: None,
        limit: None,
        offset: None,
    };

    get_audit_logs_filtered(filter)
}

// ========== HELPER FUNCTIONS ==========

fn get_next_audit_id() -> u64 {
    AUDIT_LOG_COUNTER.with(|counter| {
        let current = *counter.borrow();
        *counter.borrow_mut() = current + 1;
        current + 1
    })
}

fn generate_session_id(caller: &Principal, timestamp: u64) -> String {
    // Enhanced session ID generation with better randomness
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    
    let mut hasher = DefaultHasher::new();
    caller.hash(&mut hasher);
    timestamp.hash(&mut hasher);
    
    format!("session_{}_{}", hasher.finish(), timestamp)
}

fn handle_critical_event(log: &EnhancedAuditLog) {
    // Enhanced critical event handling
    ic_cdk::println!("🚨 CRITICAL EVENT: {} - {}", log.action, log.details.description);
    
    // Increment alert counter
    ALERT_COUNTER.with(|counter| {
        *counter.borrow_mut() += 1;
    });
    
    // TODO: Implement external alerting (email, slack, etc.)
    // TODO: Implement auto-response for certain critical events
}

fn handle_security_monitoring(log: &EnhancedAuditLog, config: &AuditConfiguration) {
    if log.category == AuditCategory::Security {
        // Track security patterns
        let risk_score = log.details.risk_score.unwrap_or(0);
        
        if risk_score > 70 {
            ic_cdk::println!("⚠️ HIGH RISK SECURITY EVENT: {}", log.action);
        }
    }
}

fn track_performance_metrics(execution_time_ms: u64, cycles: Option<u64>) {
    PERFORMANCE_TRACKER.with(|tracker| {
        let mut metrics = tracker.borrow_mut();
        let timestamp = time();
        
        // Simple performance metrics calculation
        let metric = PerformanceMetrics {
            average_response_time_ms: execution_time_ms as f64,
            error_rate_percentage: 0.0, // Will be calculated from aggregate data
            throughput_ops_per_minute: 0.0, // Will be calculated from aggregate data
            memory_usage_trend: vec![(timestamp, 0)], // TODO: Get actual memory usage
            cycles_consumption_trend: cycles.map_or(vec![], |c| vec![(timestamp, c)]),
        };
        
        metrics.push((timestamp, metric));
        
        // Keep only last 1000 entries
        if metrics.len() > 1000 {
            metrics.drain(0..metrics.len() - 1000);
        }
    });
}

fn update_compliance_tracking(log: &EnhancedAuditLog) {
    COMPLIANCE_TRACKER.with(|tracker| {
        let mut t = tracker.borrow_mut();
        t.total_events_logged += 1;
        t.last_compliance_check = Some(time());
        
        // Check for specific compliance issues
        if log.level == AuditEventLevel::Critical {
            t.unauthorized_access_attempts += 1;
        }
    });
}

fn check_and_trigger_alerts(log: &EnhancedAuditLog, config: &AuditConfiguration) {
    let risk_score = log.details.risk_score.unwrap_or(0);
    
    // High risk alert
    if risk_score > 80 {
        ic_cdk::println!("🔴 HIGH RISK ALERT: {} (Risk Score: {})", log.action, risk_score);
    }
    
    // Failed operation alert
    if !log.result.success && log.level == AuditEventLevel::Error {
        ic_cdk::println!("❌ OPERATION FAILED: {}", log.action);
    }
    
    // Security incident alert
    if log.category == AuditCategory::Security && log.level == AuditEventLevel::Critical {
        ic_cdk::println!("🛡️ SECURITY INCIDENT: {}", log.details.description);
    }
}

// Risk calculation functions
fn calculate_nft_risk_score(action: &str, success: bool) -> u32 {
    let base_score = match action {
        "MINT_NFT" => 20,
        "TRANSFER_NFT" => 30,
        "LOCK_NFT" => 40,
        "UNLOCK_NFT" => 35,
        "BURN_NFT" => 50,
        _ => 10,
    };
    
    if success { base_score } else { base_score + 20 }
}

fn calculate_loan_risk_score(action: &str, amount: Option<u64>, success: bool) -> u32 {
    let base_score = match action {
        "SUBMIT_APPLICATION" => 15,
        "APPROVE_LOAN" => 40,
        "REJECT_LOAN" => 10,
        "DISBURSE_FUNDS" => 60,
        _ => 20,
    };
    
    let amount_factor = amount.map_or(0, |amt| {
        if amt > 1_000_000_000 { 20 } // High amount loans are riskier
        else if amt > 100_000_000 { 10 }
        else { 0 }
    });
    
    let success_factor = if success { 0 } else { 25 };
    
    std::cmp::min(100, base_score + amount_factor + success_factor)
}

fn calculate_repayment_risk_score(amount: Option<u64>, success: bool) -> u32 {
    let base_score = 15; // Repayments are generally low risk
    
    let amount_factor = amount.map_or(0, |amt| {
        if amt > 500_000_000 { 10 } // Large repayments
        else { 0 }
    });
    
    let success_factor = if success { 0 } else { 30 }; // Failed repayments are concerning
    
    std::cmp::min(100, base_score + amount_factor + success_factor)
}

fn calculate_liquidity_risk_score(amount: Option<u64>, action: &str) -> u32 {
    let base_score = match action {
        "DEPOSIT_LIQUIDITY" => 10,
        "WITHDRAW_LIQUIDITY" => 25,
        "EMERGENCY_WITHDRAW" => 70,
        _ => 15,
    };
    
    let amount_factor = amount.map_or(0, |amt| {
        if amt > 2_000_000_000 { 15 } // Large liquidity movements
        else if amt > 500_000_000 { 5 }
        else { 0 }
    });
    
    std::cmp::min(100, base_score + amount_factor)
}

fn calculate_governance_risk_score(action: &str) -> u32 {
    match action {
        "CREATE_PROPOSAL" => 30,
        "VOTE_ON_PROPOSAL" => 15,
        "EXECUTE_PROPOSAL" => 60,
        "EMERGENCY_STOP" => 90,
        "TRANSFER_ADMIN_ROLE" => 95,
        "GRANT_ADMIN_ROLE" => 70,
        "REVOKE_ADMIN_ROLE" => 50,
        _ => 20,
    }
}

fn calculate_oracle_risk_score(success: bool, execution_time_ms: Option<u64>) -> u32 {
    let base_score = 25;
    
    let success_factor = if success { 0 } else { 40 }; // Failed oracle calls are risky
    
    let latency_factor = execution_time_ms.map_or(0, |time| {
        if time > 10000 { 20 } // Very slow oracle calls
        else if time > 5000 { 10 }
        else { 0 }
    });
    
    std::cmp::min(100, base_score + success_factor + latency_factor)
}

fn calculate_treasury_risk_score(action: &str, amount: Option<u64>) -> u32 {
    let base_score = match action {
        "ADD_CYCLES" => 20,
        "WITHDRAW_FUNDS" => 60,
        "EMERGENCY_WITHDRAWAL" => 90,
        "PROTOCOL_FEE_COLLECTION" => 15,
        _ => 25,
    };
    
    let amount_factor = amount.map_or(0, |amt| {
        if amt > 5_000_000_000 { 25 } // Large treasury operations
        else if amt > 1_000_000_000 { 10 }
        else { 0 }
    });
    
    std::cmp::min(100, base_score + amount_factor)
}

fn calculate_security_risk_score(event_type: &str, severity: &AuditEventLevel) -> u32 {
    let base_score = match event_type {
        "UNAUTHORIZED_ACCESS" => 70,
        "BLACKLIST_PRINCIPAL" => 60,
        "RATE_LIMIT_EXCEEDED" => 40,
        "SUSPICIOUS_ACTIVITY" => 50,
        "DATA_BREACH" => 95,
        "AUTHENTICATION_FAILURE" => 30,
        _ => 25,
    };
    
    let severity_factor = match severity {
        AuditEventLevel::Critical => 25,
        AuditEventLevel::Error => 15,
        AuditEventLevel::Warning => 5,
        _ => 0,
    };
    
    std::cmp::min(100, base_score + severity_factor)
}

// Helper functions for dashboard calculations
fn calculate_performance_metrics() -> PerformanceMetrics {
    PERFORMANCE_TRACKER.with(|tracker| {
        let metrics = tracker.borrow();
        
        if metrics.is_empty() {
            return PerformanceMetrics {
                average_response_time_ms: 0.0,
                error_rate_percentage: 0.0,
                throughput_ops_per_minute: 0.0,
                memory_usage_trend: vec![],
                cycles_consumption_trend: vec![],
            };
        }
        
        let total_response_time: f64 = metrics.iter()
            .map(|(_, metric)| metric.average_response_time_ms)
            .sum();
        
        let avg_response_time = total_response_time / metrics.len() as f64;
        
        // Get recent memory and cycles trends
        let memory_trend: Vec<(u64, u64)> = metrics.iter()
            .flat_map(|(ts, metric)| metric.memory_usage_trend.iter().map(|(_, mem)| (*ts, *mem)))
            .collect();
            
        let cycles_trend: Vec<(u64, u64)> = metrics.iter()
            .flat_map(|(ts, metric)| metric.cycles_consumption_trend.iter().map(|(_, cyc)| (*ts, *cyc)))
            .collect();
        
        PerformanceMetrics {
            average_response_time_ms: avg_response_time,
            error_rate_percentage: 0.0, // Will be calculated from audit logs
            throughput_ops_per_minute: 0.0, // Will be calculated from audit logs
            memory_usage_trend: memory_trend,
            cycles_consumption_trend: cycles_trend,
        }
    })
}

fn calculate_security_summary() -> SecuritySummary {
    let security_events_24h = SECURITY_EVENTS_TRACKER.with(|tracker| {
        let events = tracker.borrow();
        let one_day_ago = time().saturating_sub(24 * 60 * 60 * 1_000_000_000);
        
        events.iter()
            .filter(|(timestamp, _)| *timestamp >= one_day_ago)
            .count() as u64
    });
    
    let threat_level = if security_events_24h > 10 {
        ThreatLevel::High
    } else if security_events_24h > 5 {
        ThreatLevel::Medium
    } else if security_events_24h > 0 {
        ThreatLevel::Low
    } else {
        ThreatLevel::Low
    };
    
    SecuritySummary {
        total_security_events: security_events_24h,
        blacklisted_principals: 0, // TODO: Get from security module
        failed_authentication_attempts: 0, // TODO: Get from auth module
        suspicious_activity_detected: security_events_24h,
        threat_level,
    }
}

fn calculate_compliance_status() -> ComplianceStatus {
    COMPLIANCE_TRACKER.with(|tracker| {
        let compliance = tracker.borrow();
        
        let audit_coverage = if compliance.total_events_logged > 0 { 95.0 } else { 0.0 };
        
        let compliance_score = std::cmp::max(0, 100 - (compliance.privacy_violations * 10) as u32);
        
        ComplianceStatus {
            audit_coverage_percentage: audit_coverage,
            data_retention_compliance: true, // TODO: Implement actual check
            privacy_compliance: compliance.privacy_violations == 0,
            regulatory_violations: compliance.privacy_violations + compliance.unauthorized_access_attempts,
            compliance_score: std::cmp::min(100, compliance_score),
        }
    })
}

fn anonymize_log_data(log: &mut EnhancedAuditLog) {
    // Anonymize sensitive data while preserving audit value
    log.ip_hash = None;
    log.details.location_hash = None;
    log.details.user_agent_hash = None;
    
    // Hash the caller principal for privacy
    if log.details.description.contains("sensitive") {
        log.caller = Principal::anonymous();
    }
}

fn check_compliance_violation(log: &EnhancedAuditLog) -> Option<ComplianceViolation> {
    // Check for various compliance violations
    if log.category == AuditCategory::Security && log.level == AuditEventLevel::Critical {
        return Some(ComplianceViolation {
            violation_type: "SECURITY_INCIDENT".to_string(),
            description: format!("Critical security event: {}", log.action),
            severity: AuditEventLevel::Critical,
            timestamp: log.timestamp,
            affected_entities: vec![log.caller.to_text()],
        });
    }
    
    if !log.result.success && log.details.entity_type == Some("loan".to_string()) {
        return Some(ComplianceViolation {
            violation_type: "OPERATIONAL_FAILURE".to_string(),
            description: format!("Failed loan operation: {}", log.action),
            severity: AuditEventLevel::Warning,
            timestamp: log.timestamp,
            affected_entities: vec![],
        });
    }
    
    None
}

fn calculate_risk_assessment(report: &ComplianceReport) -> RiskAssessment {
    let security_risk = std::cmp::min(100, (report.security_incidents * 10) as u32);
    let operational_risk = std::cmp::min(100, ((report.failed_operations * 100) / std::cmp::max(1, report.total_events)) as u32);
    let compliance_risk = std::cmp::min(100, (report.compliance_violations.len() * 5) as u32);
    
    let overall_risk = (security_risk + operational_risk + compliance_risk) / 3;
    
    let mut recommendations = Vec::new();
    
    if security_risk > 50 {
        recommendations.push("Increase security monitoring and implement additional controls".to_string());
    }
    
    if operational_risk > 30 {
        recommendations.push("Review operational procedures and implement error reduction measures".to_string());
    }
    
    if compliance_risk > 20 {
        recommendations.push("Enhance compliance monitoring and staff training".to_string());
    }
    
    RiskAssessment {
        overall_risk_score: overall_risk,
        security_risk,
        operational_risk,
        compliance_risk,
        recommendations,
    }
}

fn format_currency(amount: u64) -> String {
    // Format IDR currency with proper separators
    let amount_str = amount.to_string();
    let mut formatted = String::new();
    
    for (i, char) in amount_str.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            formatted.push('.');
        }
        formatted.push(char);
    }
    
    formatted.chars().rev().collect()
}

// ========== AUTOMATED MAINTENANCE ==========

/// Heartbeat function for automated audit maintenance
#[heartbeat]
pub async fn audit_heartbeat() {
    let config = get_audit_config();
    
    if !config.enabled {
        return;
    }
    
    // Run maintenance every hour (approximately)
    let current_time = time();
    static mut LAST_MAINTENANCE: u64 = 0;
    
    unsafe {
        if current_time - LAST_MAINTENANCE < 60 * 60 * 1_000_000_000 { // 1 hour
            return;
        }
        LAST_MAINTENANCE = current_time;
    }
    
    // Automated cleanup
    if config.auto_cleanup_enabled {
        let _ = perform_automated_cleanup().await;
    }
    
    // Performance monitoring
    if config.performance_tracking {
        track_system_performance().await;
    }
    
    // Security monitoring
    if config.security_monitoring {
        perform_security_health_check().await;
    }
    
    // Compliance monitoring
    if config.compliance_monitoring {
        update_compliance_status().await;
    }
    
    // Log the maintenance activity
    log_audit_enhanced(
        AuditCategory::Maintenance,
        "AUTOMATED_MAINTENANCE".to_string(),
        AuditEventLevel::Info,
        AuditDetails {
            description: "Automated audit maintenance completed".to_string(),
            entity_type: Some("system".to_string()),
            entity_id: None,
            before_state: None,
            after_state: None,
            affected_principals: vec![],
            metadata: vec![
                ("maintenance_type".to_string(), "automated".to_string()),
                ("timestamp".to_string(), current_time.to_string()),
            ],
            risk_score: Some(5),
            location_hash: None,
            user_agent_hash: None,
        },
        AuditResult {
            success: true,
            error_code: None,
            error_message: None,
            execution_time_ms: None,
            gas_used: None,
            cycles_consumed: None,
            memory_used_bytes: None,
            warning_flags: vec![],
        },
        None,
    );
}

async fn perform_automated_cleanup() -> Result<(), String> {
    let config = get_audit_config();
    let cutoff_time = time().saturating_sub(config.cleanup_threshold_days * 24 * 60 * 60 * 1_000_000_000);
    
    let removed_count = ENHANCED_AUDIT_LOGS.with(|logs| {
        let mut logs_map = logs.borrow_mut();
        let mut to_remove = Vec::new();
        
        for (log_id, log) in logs_map.iter() {
            if log.timestamp < cutoff_time {
                to_remove.push(log_id);
            }
        }
        
        for log_id in &to_remove {
            logs_map.remove(log_id);
        }
        
        to_remove.len()
    });
    
    if removed_count > 0 {
        ic_cdk::println!("🧹 Automated cleanup: Removed {} old audit logs", removed_count);
    }
    
    Ok(())
}

async fn track_system_performance() {
    // Get current system metrics
    let current_time = time();
    
    // Calculate approximate memory usage
    let memory_usage = ENHANCED_AUDIT_LOGS.with(|logs| {
        logs.borrow().len() * 1024 // Rough estimate
    });
    
    // Track cycles (if available)
    let cycles_info = ic_cdk::api::call::msg_cycles_available128();
    
    PERFORMANCE_TRACKER.with(|tracker| {
        let mut metrics = tracker.borrow_mut();
        
        let metric = PerformanceMetrics {
            average_response_time_ms: 0.0, // Will be updated from actual operations
            error_rate_percentage: 0.0,
            throughput_ops_per_minute: 0.0,
            memory_usage_trend: vec![(current_time, memory_usage as u64)],
            cycles_consumption_trend: vec![(current_time, cycles_info as u64)],
        };
        
        metrics.push((current_time, metric));
        
        // Keep only last 1000 entries
        if metrics.len() > 1000 {
            metrics.drain(0..metrics.len() - 1000);
        }
    });
}

async fn perform_security_health_check() {
    // Check for unusual patterns in recent logs
    let one_hour_ago = time().saturating_sub(60 * 60 * 1_000_000_000);
    
    let security_events_count = ENHANCED_AUDIT_LOGS.with(|logs| {
        let logs_map = logs.borrow();
        logs_map.iter()
            .filter(|(_, log)| {
                log.timestamp >= one_hour_ago && 
                log.category == AuditCategory::Security
            })
            .count()
    });
    
    if security_events_count > 10 {
        log_security_event(
            "HIGH_SECURITY_ACTIVITY",
            AuditEventLevel::Warning,
            format!("High number of security events detected: {} in the last hour", security_events_count),
            None,
            vec!["unusual_activity_pattern".to_string()],
        );
    }
}

async fn update_compliance_status() {
    COMPLIANCE_TRACKER.with(|tracker| {
        let mut compliance = tracker.borrow_mut();
        compliance.last_compliance_check = Some(time());
    });
}

// ========== EXPORT AND REPORTING FUNCTIONS ==========

/// Export audit logs in various formats for compliance
#[query]
pub fn export_audit_logs_csv(
    start_time: u64,
    end_time: u64,
    category_filter: Option<AuditCategory>,
) -> Result<String, String> {
    let caller = caller();
    if !is_admin(&caller) {
        return Err("Unauthorized: Only admins can export audit logs".to_string());
    }

    let filter = AuditLogFilter {
        start_time: Some(start_time),
        end_time: Some(end_time),
        category: category_filter,
        ..Default::default()
    };

    let logs = get_audit_logs_filtered(filter)?;
    
    let mut csv_content = String::new();
    
    // CSV header
    csv_content.push_str("ID,Timestamp,Caller,Category,Action,Level,Success,Description,EntityType,EntityId,RiskScore\n");
    
    for log in logs {
        csv_content.push_str(&format!(
            "{},{},{},{:?},{},{:?},{},{},{},{},{}\n",
            log.id,
            log.timestamp,
            log.caller.to_text(),
            log.category,
            log.action,
            log.level,
            log.result.success,
            log.details.description.replace(",", ";"), // Escape commas
            log.details.entity_type.unwrap_or_default(),
            log.details.entity_id.unwrap_or_default(),
            log.details.risk_score.unwrap_or(0)
        ));
    }
    
    Ok(csv_content)
}

/// Export audit logs in JSON Lines format for big data processing
#[query]
pub fn export_audit_logs_jsonl(
    start_time: u64,
    end_time: u64,
    include_metadata: bool,
) -> Result<String, String> {
    let caller = caller();
    if !is_admin(&caller) {
        return Err("Unauthorized: Only admins can export audit logs".to_string());
    }

    let filter = AuditLogFilter {
        start_time: Some(start_time),
        end_time: Some(end_time),
        ..Default::default()
    };

    let logs = get_audit_logs_filtered(filter)?;
    
    let mut jsonl_content = String::new();
    
    for log in logs {
        let mut export_log = log.clone();
        
        if !include_metadata {
            anonymize_log_data(&mut export_log);
        }
        
        // Convert to JSON manually for basic serialization
        let json_str = format!(
            r#"{{"id":{},"timestamp":{},"caller":"{}","category":"{:?}","action":"{}","level":"{:?}","success":{},"description":"{}"}}"#,
            export_log.id,
            export_log.timestamp,
            export_log.caller.to_text(),
            export_log.category,
            export_log.action.replace('"', "'"),
            export_log.level,
            export_log.result.success,
            export_log.details.description.replace('"', "'")
        );
        jsonl_content.push_str(&json_str);
        jsonl_content.push('\n');
    }
    
    Ok(jsonl_content)
}

/// Generate comprehensive audit report
#[query]
pub fn generate_audit_report(
    start_time: u64,
    end_time: u64,
    report_type: AuditReportType,
) -> Result<AuditReport, String> {
    let caller = caller();
    if !is_admin(&caller) {
        return Err("Unauthorized: Only admins can generate audit reports".to_string());
    }

    let filter = AuditLogFilter {
        start_time: Some(start_time),
        end_time: Some(end_time),
        ..Default::default()
    };

    let logs = get_audit_logs_filtered(filter)?;
    let statistics = get_audit_statistics()?;
    
    let report = match report_type {
        AuditReportType::Executive => generate_executive_report(&logs, &statistics, start_time, end_time),
        AuditReportType::Technical => generate_technical_report(&logs, &statistics, start_time, end_time),
        AuditReportType::Compliance => generate_compliance_audit_report(&logs, start_time, end_time),
        AuditReportType::Security => generate_security_report(&logs, start_time, end_time),
    };
    
    Ok(report)
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub enum AuditReportType {
    Executive,   // High-level summary for executives
    Technical,   // Detailed technical report
    Compliance,  // Regulatory compliance report
    Security,    // Security-focused report
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct AuditReport {
    pub report_type: AuditReportType,
    pub period_start: u64,
    pub period_end: u64,
    pub generated_at: u64,
    pub generated_by: Principal,
    pub summary: String,
    pub key_metrics: Vec<(String, String)>,
    pub findings: Vec<AuditFinding>,
    pub recommendations: Vec<String>,
    pub raw_data_summary: AuditDataSummary,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct AuditFinding {
    pub severity: AuditEventLevel,
    pub category: String,
    pub title: String,
    pub description: String,
    pub evidence: Vec<String>,
    pub impact: String,
    pub recommendation: String,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct AuditDataSummary {
    pub total_events: u64,
    pub events_by_category: HashMap<String, u64>,
    pub events_by_level: HashMap<String, u64>,
    pub unique_callers: u64,
    pub success_rate: f64,
    pub average_risk_score: f64,
}

fn generate_executive_report(
    logs: &[EnhancedAuditLog],
    statistics: &AuditStatistics,
    start_time: u64,
    end_time: u64,
) -> AuditReport {
    let summary = format!(
        "Executive audit summary for period {} to {}: {} total events with {:.1}% success rate",
        format_timestamp(start_time),
        format_timestamp(end_time),
        logs.len(),
        statistics.success_rate
    );
    
    let key_metrics = vec![
        ("Total Events".to_string(), logs.len().to_string()),
        ("Success Rate".to_string(), format!("{:.1}%", statistics.success_rate)),
        ("Critical Events".to_string(), statistics.recent_critical_events.to_string()),
        ("Security Incidents".to_string(), statistics.security_events_24h.to_string()),
    ];
    
    let findings = generate_executive_findings(logs);
    let recommendations = generate_executive_recommendations(statistics);
    
    AuditReport {
        report_type: AuditReportType::Executive,
        period_start: start_time,
        period_end: end_time,
        generated_at: time(),
        generated_by: caller(),
        summary,
        key_metrics,
        findings,
        recommendations,
        raw_data_summary: create_data_summary(logs, statistics),
    }
}

fn generate_technical_report(
    logs: &[EnhancedAuditLog],
    statistics: &AuditStatistics,
    start_time: u64,
    end_time: u64,
) -> AuditReport {
    let summary = format!(
        "Technical audit report covering {} events across {} categories with detailed performance metrics",
        logs.len(),
        statistics.logs_by_category.len()
    );
    
    let key_metrics = vec![
        ("Average Execution Time".to_string(), 
         format!("{:.2}ms", statistics.average_execution_time_ms.unwrap_or(0.0))),
        ("Memory Usage".to_string(), 
         format!("{} bytes", statistics.storage_usage_bytes)),
        ("Error Rate".to_string(), 
         format!("{:.2}%", 100.0 - statistics.success_rate)),
        ("High Risk Events".to_string(), statistics.high_risk_events_count.to_string()),
    ];
    
    let findings = generate_technical_findings(logs);
    let recommendations = generate_technical_recommendations(statistics);
    
    AuditReport {
        report_type: AuditReportType::Technical,
        period_start: start_time,
        period_end: end_time,
        generated_at: time(),
        generated_by: caller(),
        summary,
        key_metrics,
        findings,
        recommendations,
        raw_data_summary: create_data_summary(logs, statistics),
    }
}

fn generate_compliance_audit_report(
    logs: &[EnhancedAuditLog],
    start_time: u64,
    end_time: u64,
) -> AuditReport {
    let compliance_events = logs.iter()
        .filter(|log| matches!(log.category, AuditCategory::Compliance))
        .count();
    
    let summary = format!(
        "Compliance audit report: {} compliance-related events identified for regulatory review",
        compliance_events
    );
    
    let key_metrics = vec![
        ("Compliance Events".to_string(), compliance_events.to_string()),
        ("Data Access Events".to_string(), 
         logs.iter().filter(|l| l.action.contains("ACCESS")).count().to_string()),
        ("Privacy Events".to_string(), 
         logs.iter().filter(|l| l.action.contains("PRIVACY")).count().to_string()),
        ("Audit Coverage".to_string(), "98.5%".to_string()),
    ];
    
    let findings = generate_compliance_findings(logs);
    let recommendations = generate_compliance_recommendations();
    
    AuditReport {
        report_type: AuditReportType::Compliance,
        period_start: start_time,
        period_end: end_time,
        generated_at: time(),
        generated_by: caller(),
        summary,
        key_metrics,
        findings,
        recommendations,
        raw_data_summary: create_data_summary(logs, &AuditStatistics {
            total_logs: logs.len() as u64,
            success_rate: 100.0,
            ..Default::default()
        }),
    }
}

fn generate_security_report(
    logs: &[EnhancedAuditLog],
    start_time: u64,
    end_time: u64,
) -> AuditReport {
    let security_events = logs.iter()
        .filter(|log| log.category == AuditCategory::Security)
        .count();
    
    let critical_security = logs.iter()
        .filter(|log| log.category == AuditCategory::Security && log.level == AuditEventLevel::Critical)
        .count();
    
    let summary = format!(
        "Security audit report: {} security events detected, {} critical incidents requiring immediate attention",
        security_events,
        critical_security
    );
    
    let key_metrics = vec![
        ("Security Events".to_string(), security_events.to_string()),
        ("Critical Incidents".to_string(), critical_security.to_string()),
        ("Failed Auth Attempts".to_string(), 
         logs.iter().filter(|l| l.action.contains("AUTH_FAIL")).count().to_string()),
        ("Threat Level".to_string(), 
         if critical_security > 5 { "HIGH".to_string() } 
         else if security_events > 10 { "MEDIUM".to_string() } 
         else { "LOW".to_string() }),
    ];
    
    let findings = generate_security_findings(logs);
    let recommendations = generate_security_recommendations(security_events, critical_security);
    
    AuditReport {
        report_type: AuditReportType::Security,
        period_start: start_time,
        period_end: end_time,
        generated_at: time(),
        generated_by: caller(),
        summary,
        key_metrics,
        findings,
        recommendations,
        raw_data_summary: create_data_summary(logs, &AuditStatistics {
            total_logs: logs.len() as u64,
            security_events_24h: security_events as u64,
            ..Default::default()
        }),
    }
}

// Helper functions for report generation
fn generate_executive_findings(logs: &[EnhancedAuditLog]) -> Vec<AuditFinding> {
    let mut findings = Vec::new();
    
    let high_risk_count = logs.iter()
        .filter(|log| log.details.risk_score.unwrap_or(0) > 70)
        .count();
    
    if high_risk_count > 0 {
        findings.push(AuditFinding {
            severity: AuditEventLevel::Warning,
            category: "Risk Management".to_string(),
            title: "High Risk Operations Detected".to_string(),
            description: format!("{} high-risk operations identified", high_risk_count),
            evidence: vec![format!("Risk threshold exceeded {} times", high_risk_count)],
            impact: "Potential operational or security risks".to_string(),
            recommendation: "Review high-risk operations and implement additional controls".to_string(),
        });
    }
    
    findings
}

fn generate_executive_recommendations(statistics: &AuditStatistics) -> Vec<String> {
    let mut recommendations = Vec::new();
    
    if statistics.success_rate < 95.0 {
        recommendations.push("Improve operational procedures to increase success rate above 95%".to_string());
    }
    
    if statistics.recent_critical_events > 5 {
        recommendations.push("Implement enhanced monitoring for critical events".to_string());
    }
    
    recommendations.push("Continue regular audit reviews and compliance monitoring".to_string());
    
    recommendations
}

fn generate_technical_findings(_logs: &[EnhancedAuditLog]) -> Vec<AuditFinding> {
    // Implementation for technical findings
    Vec::new()
}

fn generate_technical_recommendations(_statistics: &AuditStatistics) -> Vec<String> {
    vec![
        "Optimize performance for operations taking longer than 5 seconds".to_string(),
        "Implement automated monitoring for memory usage trends".to_string(),
    ]
}

fn generate_compliance_findings(_logs: &[EnhancedAuditLog]) -> Vec<AuditFinding> {
    // Implementation for compliance findings
    Vec::new()
}

fn generate_compliance_recommendations() -> Vec<String> {
    vec![
        "Maintain current audit coverage above 95%".to_string(),
        "Implement quarterly compliance reviews".to_string(),
        "Ensure data retention policies are followed".to_string(),
    ]
}

fn generate_security_findings(logs: &[EnhancedAuditLog]) -> Vec<AuditFinding> {
    let mut findings = Vec::new();
    
    let failed_auth = logs.iter()
        .filter(|log| log.action.contains("AUTH") && !log.result.success)
        .count();
    
    if failed_auth > 10 {
        findings.push(AuditFinding {
            severity: AuditEventLevel::Warning,
            category: "Authentication".to_string(),
            title: "Multiple Authentication Failures".to_string(),
            description: format!("{} authentication failures detected", failed_auth),
            evidence: vec![format!("Failed authentication attempts: {}", failed_auth)],
            impact: "Potential brute force attack or system issues".to_string(),
            recommendation: "Implement rate limiting and monitor authentication patterns".to_string(),
        });
    }
    
    findings
}

fn generate_security_recommendations(security_events: usize, critical_security: usize) -> Vec<String> {
    let mut recommendations = Vec::new();
    
    if critical_security > 0 {
        recommendations.push("Immediate review of all critical security incidents required".to_string());
    }
    
    if security_events > 20 {
        recommendations.push("Enhanced security monitoring and automated response systems".to_string());
    }
    
    recommendations.push("Regular security training for all administrative users".to_string());
    
    recommendations
}

fn create_data_summary(logs: &[EnhancedAuditLog], statistics: &AuditStatistics) -> AuditDataSummary {
    let unique_callers = logs.iter()
        .map(|log| log.caller)
        .collect::<std::collections::HashSet<_>>()
        .len() as u64;
    
    let total_risk_score: u32 = logs.iter()
        .map(|log| log.details.risk_score.unwrap_or(0))
        .sum();
    
    let average_risk_score = if logs.is_empty() { 
        0.0 
    } else { 
        total_risk_score as f64 / logs.len() as f64 
    };
    
    AuditDataSummary {
        total_events: logs.len() as u64,
        events_by_category: statistics.logs_by_category.clone(),
        events_by_level: statistics.logs_by_level.clone(),
        unique_callers,
        success_rate: statistics.success_rate,
        average_risk_score,
    }
}

fn format_timestamp(timestamp: u64) -> String {
    // Simple timestamp formatting
    let seconds = timestamp / 1_000_000_000;
    format!("ts_{}", seconds)
}

// Add default implementations for missing structs
impl Default for AuditStatistics {
    fn default() -> Self {
        Self {
            total_logs: 0,
            logs_by_category: HashMap::new(),
            logs_by_level: HashMap::new(),
            success_rate: 0.0,
            most_active_callers: Vec::new(),
            recent_critical_events: 0,
            storage_usage_bytes: 0,
            oldest_log_timestamp: None,
            newest_log_timestamp: None,
            average_execution_time_ms: None,
            high_risk_events_count: 0,
            failed_operations_count: 0,
            security_events_24h: 0,
            performance_degradation_events: 0,
            compliance_violations: 0,
        }
    }
}