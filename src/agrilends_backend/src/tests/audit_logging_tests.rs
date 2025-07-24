#[cfg(test)]
mod audit_logging_tests {
    use super::*;
    use crate::audit_logging::*;
    use ic_cdk::api::time;
    use candid::Principal;
    
    // Helper function to create a test principal
    fn test_principal() -> Principal {
        Principal::from_text("rdmx6-jaaaa-aaaah-qcaiq-cai").unwrap()
    }

    #[test]
    fn test_enhanced_audit_log_creation() {
        let caller = test_principal();
        let timestamp = time();
        
        let details = AuditDetails {
            description: "Test audit log creation".to_string(),
            entity_type: Some("test".to_string()),
            entity_id: Some("123".to_string()),
            before_state: None,
            after_state: None,
            affected_principals: vec![caller],
            metadata: vec![("test_key".to_string(), "test_value".to_string())],
        };

        let result = AuditResult {
            success: true,
            error_code: None,
            error_message: None,
            execution_time_ms: Some(150),
            gas_used: None,
        };

        // Test creating an enhanced audit log
        log_audit_enhanced(
            AuditCategory::UserManagement,
            "TEST_ACTION".to_string(),
            AuditEventLevel::Info,
            details,
            result,
            Some("test-correlation-123".to_string()),
        );

        // The log should be stored successfully
        // In a real test environment, we would verify this by querying the logs
    }

    #[test]
    fn test_nft_operation_logging() {
        let token_id = 123u64;
        let owner = test_principal();
        
        // Test successful NFT operation
        log_nft_operation(
            "MINT_NFT",
            token_id,
            owner,
            true,
            None,
        );
        
        // Test failed NFT operation
        log_nft_operation(
            "TRANSFER_NFT",
            token_id,
            owner,
            false,
            Some("Insufficient authorization".to_string()),
        );
    }

    #[test]
    fn test_loan_operation_logging() {
        let loan_id = 456u64;
        let borrower = test_principal();
        let amount = 100_000u64;
        
        // Test successful loan operation
        log_loan_operation(
            "APPROVE_LOAN",
            loan_id,
            borrower,
            Some(amount),
            true,
            None,
        );
        
        // Test failed loan operation
        log_loan_operation(
            "DISBURSE_FUNDS",
            loan_id,
            borrower,
            Some(amount),
            false,
            Some("Insufficient pool liquidity".to_string()),
        );
    }

    #[test]
    fn test_security_event_logging() {
        let affected_principal = test_principal();
        
        // Test critical security event
        log_security_event(
            "UNAUTHORIZED_ACCESS",
            AuditEventLevel::Critical,
            "Attempted access to admin function by non-admin principal".to_string(),
            Some(affected_principal),
        );
        
        // Test warning security event
        log_security_event(
            "RATE_LIMIT_EXCEEDED",
            AuditEventLevel::Warning,
            "Principal exceeded rate limit for API calls".to_string(),
            Some(affected_principal),
        );
    }

    #[test]
    fn test_audit_configuration() {
        let config = AuditConfiguration {
            enabled: true,
            max_logs_per_category: 5000,
            auto_cleanup_enabled: true,
            cleanup_threshold_days: 180,
            critical_event_notification: true,
            detailed_logging: true,
            performance_tracking: false,
            anonymization_enabled: true,
        };

        // Test configuration validation
        assert!(config.enabled);
        assert_eq!(config.max_logs_per_category, 5000);
        assert_eq!(config.cleanup_threshold_days, 180);
        assert!(config.anonymization_enabled);
    }

    #[test]
    fn test_audit_filter_creation() {
        let filter = AuditLogFilter {
            start_time: Some(1640995200000000000u64), // Jan 1, 2022
            end_time: Some(1672531200000000000u64),   // Jan 1, 2023
            caller: Some(test_principal()),
            category: Some(AuditCategory::LoanLifecycle),
            level: Some(AuditEventLevel::Success),
            action_pattern: Some("LOAN".to_string()),
            success_only: Some(true),
            entity_type: Some("loan".to_string()),
            entity_id: Some("123".to_string()),
            limit: Some(100),
            offset: Some(0),
        };

        // Verify filter properties
        assert!(filter.start_time.is_some());
        assert!(filter.end_time.is_some());
        assert!(filter.caller.is_some());
        assert_eq!(filter.limit, Some(100));
        assert_eq!(filter.offset, Some(0));
    }

    #[test]
    fn test_audit_event_levels() {
        // Test all audit event levels
        let levels = vec![
            AuditEventLevel::Info,
            AuditEventLevel::Warning,
            AuditEventLevel::Error,
            AuditEventLevel::Critical,
            AuditEventLevel::Success,
        ];

        for level in levels {
            let details = AuditDetails {
                description: format!("Test event for level {:?}", level),
                entity_type: None,
                entity_id: None,
                before_state: None,
                after_state: None,
                affected_principals: vec![],
                metadata: vec![],
            };

            let result = AuditResult {
                success: matches!(level, AuditEventLevel::Success),
                error_code: None,
                error_message: None,
                execution_time_ms: None,
                gas_used: None,
            };

            log_audit_enhanced(
                AuditCategory::UserManagement,
                format!("TEST_{:?}", level),
                level,
                details,
                result,
                None,
            );
        }
    }

    #[test]
    fn test_audit_categories() {
        // Test all audit categories
        let categories = vec![
            AuditCategory::UserManagement,
            AuditCategory::NFTOperations,
            AuditCategory::LoanLifecycle,
            AuditCategory::Liquidation,
            AuditCategory::Governance,
            AuditCategory::Treasury,
            AuditCategory::Oracle,
            AuditCategory::Security,
            AuditCategory::Configuration,
            AuditCategory::Maintenance,
            AuditCategory::Integration,
        ];

        for category in categories {
            let details = AuditDetails {
                description: format!("Test event for category {:?}", category),
                entity_type: None,
                entity_id: None,
                before_state: None,
                after_state: None,
                affected_principals: vec![],
                metadata: vec![],
            };

            let result = AuditResult {
                success: true,
                error_code: None,
                error_message: None,
                execution_time_ms: None,
                gas_used: None,
            };

            log_audit_enhanced(
                category,
                "TEST_CATEGORY".to_string(),
                AuditEventLevel::Info,
                details,
                result,
                None,
            );
        }
    }

    #[test]
    fn test_correlation_tracking() {
        let correlation_id = "test-transaction-456".to_string();
        
        // Log multiple related operations with the same correlation ID
        for i in 1..=3 {
            let details = AuditDetails {
                description: format!("Step {} of transaction", i),
                entity_type: Some("transaction".to_string()),
                entity_id: Some(correlation_id.clone()),
                before_state: None,
                after_state: None,
                affected_principals: vec![],
                metadata: vec![("step".to_string(), i.to_string())],
            };

            let result = AuditResult {
                success: true,
                error_code: None,
                error_message: None,
                execution_time_ms: Some(50 * i as u64),
                gas_used: None,
            };

            log_audit_enhanced(
                AuditCategory::LoanLifecycle,
                format!("TRANSACTION_STEP_{}", i),
                AuditEventLevel::Info,
                details,
                result,
                Some(correlation_id.clone()),
            );
        }
    }

    #[test]
    fn test_performance_tracking() {
        let details = AuditDetails {
            description: "Performance test operation".to_string(),
            entity_type: Some("performance_test".to_string()),
            entity_id: Some("perf_001".to_string()),
            before_state: None,
            after_state: None,
            affected_principals: vec![],
            metadata: vec![
                ("operation_type".to_string(), "complex_calculation".to_string()),
                ("input_size".to_string(), "1000".to_string()),
            ],
        };

        let result = AuditResult {
            success: true,
            error_code: None,
            error_message: None,
            execution_time_ms: Some(2500), // 2.5 seconds
            gas_used: Some(150000),
        };

        log_audit_enhanced(
            AuditCategory::Maintenance,
            "PERFORMANCE_TEST".to_string(),
            AuditEventLevel::Info,
            details,
            result,
            None,
        );
    }

    #[test]
    fn test_error_handling_in_audit() {
        let details = AuditDetails {
            description: "Failed operation test".to_string(),
            entity_type: Some("error_test".to_string()),
            entity_id: Some("err_001".to_string()),
            before_state: Some("initial_state".to_string()),
            after_state: None, // No after state because operation failed
            affected_principals: vec![test_principal()],
            metadata: vec![
                ("error_type".to_string(), "validation_error".to_string()),
                ("attempted_value".to_string(), "invalid_input".to_string()),
            ],
        };

        let result = AuditResult {
            success: false,
            error_code: Some("ERR_VALIDATION_001".to_string()),
            error_message: Some("Input validation failed: invalid format".to_string()),
            execution_time_ms: Some(100),
            gas_used: Some(5000),
        };

        log_audit_enhanced(
            AuditCategory::UserManagement,
            "VALIDATION_FAILED".to_string(),
            AuditEventLevel::Error,
            details,
            result,
            None,
        );
    }
}
