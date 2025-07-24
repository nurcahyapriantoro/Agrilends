// ========== GOVERNANCE MODULE TESTS ==========
// Comprehensive test suite for governance and administration features
// Tests all README specifications including parameter updates and admin role management

#[cfg(test)]
mod governance_tests {
    use super::*;
    use candid::Principal;
    use crate::governance::*;
    use crate::types::*;
    
    // Test helper functions
    fn get_test_admin() -> Principal {
        Principal::from_text("rdmx6-jaaaa-aaaah-qcaiq-cai").unwrap()
    }
    
    fn get_test_user() -> Principal {
        Principal::from_text("rrkah-fqaaa-aaaah-qcaiq-cai").unwrap()
    }
    
    // Initialize test environment
    fn setup_governance_test() {
        init_governance();
        
        // Grant super admin role to test admin
        let admin = get_test_admin();
        let admin_role = AdminRole {
            principal: admin,
            role_type: AdminRoleType::SuperAdmin,
            granted_at: ic_cdk::api::time(),
            granted_by: admin,
            expires_at: None,
            permissions: vec![
                Permission::ManageParameters,
                Permission::ManageAdmins,
                Permission::EmergencyStop,
                Permission::ManageTreasury,
                Permission::ManageLiquidation,
                Permission::ManageOracle,
                Permission::ViewMetrics,
                Permission::ExecuteProposals,
            ],
            is_active: true,
        };
        
        // Manually insert admin role for testing
        ADMIN_ROLES.with(|roles| {
            roles.borrow_mut().insert(admin, admin_role);
        });
    }
    
    #[test]
    fn test_set_protocol_parameter_success_by_admin() {
        setup_governance_test();
        
        // Mock caller as admin
        ic_cdk::api::set_caller(get_test_admin());
        
        // Test updating LTV ratio
        let result = set_protocol_parameter("loan_to_value_ratio".to_string(), 7000);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Parameter loan_to_value_ratio updated successfully");
        
        // Verify parameter was updated
        let param = get_protocol_parameter("loan_to_value_ratio".to_string());
        assert!(param.is_ok());
        assert_eq!(param.unwrap().current_value, 7000);
    }
    
    #[test]
    fn test_set_protocol_parameter_failure_by_user() {
        setup_governance_test();
        
        // Mock caller as regular user
        ic_cdk::api::set_caller(get_test_user());
        
        // Test updating LTV ratio as non-admin (should fail)
        let result = set_protocol_parameter("loan_to_value_ratio".to_string(), 7000);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Unauthorized: Only admins can set parameters directly");
    }
    
    #[test]
    fn test_update_base_apr_parameter() {
        setup_governance_test();
        
        // Mock caller as admin
        ic_cdk::api::set_caller(get_test_admin());
        
        // Test updating base APR
        let result = set_protocol_parameter("base_interest_rate".to_string(), 1200);
        assert!(result.is_ok());
        
        // Verify parameter was updated
        let param = get_protocol_parameter("base_interest_rate".to_string());
        assert!(param.is_ok());
        assert_eq!(param.unwrap().current_value, 1200);
    }
    
    #[test]
    fn test_parameter_validation_bounds() {
        setup_governance_test();
        
        // Mock caller as admin
        ic_cdk::api::set_caller(get_test_admin());
        
        // Test setting LTV below minimum (should fail)
        let result = set_protocol_parameter("loan_to_value_ratio".to_string(), 2000);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("below minimum"));
        
        // Test setting LTV above maximum (should fail)
        let result = set_protocol_parameter("loan_to_value_ratio".to_string(), 9000);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("above maximum"));
    }
    
    #[test]
    fn test_transfer_admin_role_success() {
        setup_governance_test();
        
        // Mock caller as current admin
        ic_cdk::api::set_caller(get_test_admin());
        
        let new_admin = get_test_user();
        
        // Transfer admin role
        let result = transfer_admin_role(new_admin);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Admin role transferred successfully");
        
        // Verify new admin has super admin role
        let new_admin_role = get_admin_role(new_admin);
        assert!(new_admin_role.is_some());
        assert!(matches!(new_admin_role.unwrap().role_type, AdminRoleType::SuperAdmin));
        
        // Verify old admin role is deactivated
        let old_admin_role = get_admin_role(get_test_admin());
        assert!(old_admin_role.is_some());
        assert!(!old_admin_role.unwrap().is_active);
    }
    
    #[test]
    fn test_transfer_admin_role_unauthorized() {
        setup_governance_test();
        
        // Mock caller as regular user
        ic_cdk::api::set_caller(get_test_user());
        
        let new_admin = Principal::from_text("rjump-6iaaa-aaaah-qcaiq-cai").unwrap();
        
        // Attempt to transfer admin role (should fail)
        let result = transfer_admin_role(new_admin);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Unauthorized: Only super admins can transfer ownership");
    }
    
    #[test]
    fn test_grant_and_revoke_admin_roles() {
        setup_governance_test();
        
        // Mock caller as super admin
        ic_cdk::api::set_caller(get_test_admin());
        
        let new_admin = get_test_user();
        
        // Grant protocol admin role
        let result = grant_admin_role(
            new_admin,
            AdminRoleType::ProtocolAdmin,
            vec![Permission::ManageParameters, Permission::ViewMetrics],
            None
        );
        assert!(result.is_ok());
        
        // Verify role was granted
        let admin_role = get_admin_role(new_admin);
        assert!(admin_role.is_some());
        assert!(admin_role.unwrap().is_active);
        
        // Revoke admin role
        let result = revoke_admin_role(new_admin);
        assert!(result.is_ok());
        
        // Verify role was revoked
        let admin_role = get_admin_role(new_admin);
        assert!(admin_role.is_some());
        assert!(!admin_role.unwrap().is_active);
    }
    
    #[test]
    fn test_governance_proposal_creation() {
        setup_governance_test();
        
        // Mock caller as admin
        ic_cdk::api::set_caller(get_test_admin());
        
        // Create parameter update proposal
        let result = create_proposal(
            ProposalType::ProtocolParameterUpdate,
            "Update LTV Ratio".to_string(),
            "Proposal to update loan-to-value ratio to 65%".to_string(),
            Some("loan_to_value_ratio:6500".as_bytes().to_vec())
        );
        assert!(result.is_ok());
        
        let proposal_id = result.unwrap();
        assert!(proposal_id > 0);
        
        // Verify proposal was created
        let proposal = get_proposal(proposal_id);
        assert!(proposal.is_some());
        let prop = proposal.unwrap();
        assert_eq!(prop.title, "Update LTV Ratio");
        assert_eq!(prop.status, ProposalStatus::Active);
    }
    
    #[test]
    fn test_proposal_voting_process() {
        setup_governance_test();
        
        // Mock caller as admin
        ic_cdk::api::set_caller(get_test_admin());
        
        // Create proposal
        let proposal_id = create_proposal(
            ProposalType::ProtocolParameterUpdate,
            "Test Proposal".to_string(),
            "Test proposal for voting".to_string(),
            Some("base_interest_rate:1500".as_bytes().to_vec())
        ).unwrap();
        
        // Vote on proposal
        let result = vote_on_proposal(
            proposal_id,
            VoteChoice::Yes,
            Some("Approved for testing".to_string())
        );
        assert!(result.is_ok());
        
        // Verify vote was recorded
        let votes = get_proposal_votes(proposal_id);
        assert_eq!(votes.len(), 1);
        assert_eq!(votes[0].choice, VoteChoice::Yes);
        assert_eq!(votes[0].voter, get_test_admin());
    }
    
    #[test]
    fn test_emergency_stop_functionality() {
        setup_governance_test();
        
        // Mock caller as admin with emergency permission
        ic_cdk::api::set_caller(get_test_admin());
        
        // Activate emergency stop
        let result = emergency_stop();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Emergency stop activated");
        
        // Verify emergency stop parameter is set
        let param = get_protocol_parameter("emergency_stop".to_string());
        assert!(param.is_ok());
        assert_eq!(param.unwrap().current_value, 1);
    }
    
    #[test]
    fn test_resume_operations_functionality() {
        setup_governance_test();
        
        // Mock caller as super admin
        ic_cdk::api::set_caller(get_test_admin());
        
        // First activate emergency stop
        let _ = emergency_stop();
        
        // Then resume operations
        let result = resume_operations();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Operations resumed successfully");
        
        // Verify emergency stop and maintenance mode are disabled
        let emergency_param = get_protocol_parameter("emergency_stop".to_string());
        assert!(emergency_param.is_ok());
        assert_eq!(emergency_param.unwrap().current_value, 0);
        
        let maintenance_param = get_protocol_parameter("maintenance_mode".to_string());
        assert!(maintenance_param.is_ok());
        assert_eq!(maintenance_param.unwrap().current_value, 0);
    }
    
    #[test]
    fn test_get_all_protocol_parameters() {
        setup_governance_test();
        
        let parameters = get_all_protocol_parameters();
        assert!(!parameters.is_empty());
        
        // Verify default parameters are present
        let ltv_param = parameters.iter().find(|p| p.key == "loan_to_value_ratio");
        assert!(ltv_param.is_some());
        assert_eq!(ltv_param.unwrap().current_value, 6000);
        
        let apr_param = parameters.iter().find(|p| p.key == "base_interest_rate");
        assert!(apr_param.is_some());
        assert_eq!(apr_param.unwrap().current_value, 1000);
    }
    
    #[test]
    fn test_governance_statistics() {
        setup_governance_test();
        
        // Mock caller as admin
        ic_cdk::api::set_caller(get_test_admin());
        
        // Create some proposals
        let _proposal1 = create_proposal(
            ProposalType::ProtocolParameterUpdate,
            "Proposal 1".to_string(),
            "First test proposal".to_string(),
            None
        ).unwrap();
        
        let _proposal2 = create_proposal(
            ProposalType::AdminRoleUpdate,
            "Proposal 2".to_string(),
            "Second test proposal".to_string(),
            None
        ).unwrap();
        
        // Get governance stats
        let stats = get_governance_stats();
        assert_eq!(stats.total_proposals, 2);
        assert_eq!(stats.active_proposals, 2);
        assert_eq!(stats.executed_proposals, 0);
        assert!(stats.total_voting_power > 0);
    }
    
    #[test]
    fn test_admin_role_permissions() {
        setup_governance_test();
        
        // Mock caller as super admin
        ic_cdk::api::set_caller(get_test_admin());
        
        let treasury_admin = Principal::from_text("rjump-6iaaa-aaaah-qcaiq-cai").unwrap();
        
        // Grant treasury admin role with specific permissions
        let result = grant_admin_role(
            treasury_admin,
            AdminRoleType::TreasuryAdmin,
            vec![Permission::ManageTreasury, Permission::ViewMetrics],
            Some(ic_cdk::api::time() + 365 * 24 * 60 * 60 * 1_000_000_000) // 1 year expiry
        );
        assert!(result.is_ok());
        
        // Verify role has correct permissions
        let role = get_admin_role(treasury_admin);
        assert!(role.is_some());
        let admin_role = role.unwrap();
        assert!(admin_role.permissions.contains(&Permission::ManageTreasury));
        assert!(admin_role.permissions.contains(&Permission::ViewMetrics));
        assert!(!admin_role.permissions.contains(&Permission::ManageParameters));
        assert!(admin_role.expires_at.is_some());
    }
    
    #[test]
    fn test_parameter_history_tracking() {
        setup_governance_test();
        
        // Mock caller as admin
        ic_cdk::api::set_caller(get_test_admin());
        
        // Update parameter multiple times
        let _ = set_protocol_parameter("loan_to_value_ratio".to_string(), 6500);
        let first_update_time = ic_cdk::api::time();
        
        // Simulate time passage
        std::thread::sleep(std::time::Duration::from_millis(10));
        
        let _ = set_protocol_parameter("loan_to_value_ratio".to_string(), 7000);
        
        // Verify latest update is tracked
        let param = get_protocol_parameter("loan_to_value_ratio".to_string()).unwrap();
        assert_eq!(param.current_value, 7000);
        assert_eq!(param.updated_by, get_test_admin());
        assert!(param.last_updated >= first_update_time);
    }
    
    #[test]
    fn test_custom_parameter_creation() {
        setup_governance_test();
        
        // Mock caller as admin
        ic_cdk::api::set_caller(get_test_admin());
        
        // Create new custom parameter
        let result = set_protocol_parameter("custom_fee_rate".to_string(), 250);
        assert!(result.is_ok());
        
        // Verify custom parameter was created
        let param = get_protocol_parameter("custom_fee_rate".to_string());
        assert!(param.is_ok());
        let custom_param = param.unwrap();
        assert_eq!(custom_param.key, "custom_fee_rate");
        assert_eq!(custom_param.current_value, 250);
        assert_eq!(custom_param.description, "Custom parameter");
    }
    
    #[test]
    fn test_multiple_admin_roles() {
        setup_governance_test();
        
        // Mock caller as super admin
        ic_cdk::api::set_caller(get_test_admin());
        
        // Create multiple admin roles
        let protocol_admin = Principal::from_text("rjump-6iaaa-aaaah-qcaiq-cai").unwrap();
        let risk_admin = Principal::from_text("rekah-fqaaa-aaaah-qcaiq-cai").unwrap();
        let oracle_admin = Principal::from_text("repah-gqaaa-aaaah-qcaiq-cai").unwrap();
        
        // Grant different admin roles
        let _ = grant_admin_role(protocol_admin, AdminRoleType::ProtocolAdmin, vec![Permission::ManageParameters], None);
        let _ = grant_admin_role(risk_admin, AdminRoleType::RiskAdmin, vec![Permission::ManageLiquidation], None);
        let _ = grant_admin_role(oracle_admin, AdminRoleType::OracleAdmin, vec![Permission::ManageOracle], None);
        
        // Verify all roles are active
        let all_roles = get_all_admin_roles();
        assert!(all_roles.len() >= 4); // Including super admin
        
        let active_roles: Vec<_> = all_roles.iter().filter(|r| r.is_active).collect();
        assert!(active_roles.len() >= 4);
    }
    
    #[test]
    fn test_proposal_execution_with_parameter_update() {
        setup_governance_test();
        
        // Mock caller as admin
        ic_cdk::api::set_caller(get_test_admin());
        
        // Create parameter update proposal
        let proposal_id = create_proposal(
            ProposalType::ProtocolParameterUpdate,
            "Update Liquidation Threshold".to_string(),
            "Proposal to update liquidation threshold to 85%".to_string(),
            Some("liquidation_threshold:8500".as_bytes().to_vec())
        ).unwrap();
        
        // Vote yes on proposal
        let _ = vote_on_proposal(proposal_id, VoteChoice::Yes, Some("Approved".to_string()));
        
        // Wait for voting period to end (simulate by modifying proposal deadline)
        PROPOSALS.with(|proposals| {
            if let Some(mut proposal) = proposals.borrow().get(&proposal_id) {
                proposal.voting_deadline = ic_cdk::api::time() - 1000; // Past deadline
                proposals.borrow_mut().insert(proposal_id, proposal);
            }
        });
        
        // Execute proposal
        let result = execute_proposal(proposal_id);
        assert!(result.is_ok());
        
        // Verify parameter was updated through proposal execution
        let param = get_protocol_parameter("liquidation_threshold".to_string());
        assert!(param.is_ok());
        assert_eq!(param.unwrap().current_value, 8500);
    }
}

// Integration tests for governance system
#[cfg(test)]
mod governance_integration_tests {
    use super::*;
    
    #[test]
    fn test_complete_governance_workflow() {
        // This test demonstrates a complete governance workflow
        // from proposal creation to execution
        
        governance_tests::setup_governance_test();
        
        // Mock caller as admin
        ic_cdk::api::set_caller(governance_tests::get_test_admin());
        
        // 1. Create a parameter update proposal
        let proposal_id = create_proposal(
            ProposalType::ProtocolParameterUpdate,
            "Comprehensive Parameter Update".to_string(),
            "Update multiple system parameters for better protocol management".to_string(),
            Some("protocol_fee_rate:600".as_bytes().to_vec())
        ).unwrap();
        
        // 2. Vote on the proposal
        let vote_result = vote_on_proposal(
            proposal_id,
            VoteChoice::Yes,
            Some("Supports better fee structure".to_string())
        );
        assert!(vote_result.is_ok());
        
        // 3. Fast-forward time to end voting period
        PROPOSALS.with(|proposals| {
            if let Some(mut proposal) = proposals.borrow().get(&proposal_id) {
                proposal.voting_deadline = ic_cdk::api::time() - 1000;
                proposals.borrow_mut().insert(proposal_id, proposal);
            }
        });
        
        // 4. Execute the proposal
        let execution_result = execute_proposal(proposal_id);
        assert!(execution_result.is_ok());
        
        // 5. Verify the parameter was updated
        let updated_param = get_protocol_parameter("protocol_fee_rate".to_string());
        assert!(updated_param.is_ok());
        assert_eq!(updated_param.unwrap().current_value, 600);
        
        // 6. Check governance statistics
        let stats = get_governance_stats();
        assert_eq!(stats.executed_proposals, 1);
        assert_eq!(stats.total_votes_cast, 1);
        
        println!("✅ Complete governance workflow test passed successfully!");
    }
}
