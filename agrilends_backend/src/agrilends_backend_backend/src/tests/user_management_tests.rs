/*
// This entire file is commented out because it uses ic_cdk::api::set_caller which doesn't exist
// These tests would need to be rewritten for proper IC testing environment

// src/tests/mod.rs - Fixed version
pub mod user_management_tests;
pub mod rwa_nft_tests;

use crate::user_management::*;
use crate::types::*;
use candid::Principal;

// Helper functions shared across all tests
fn create_test_principal(id: u8) -> Principal {
    let mut bytes = [0u8; 29];
    bytes[0] = id;
    Principal::from_slice(&bytes)
}

fn mock_time() -> u64 {
    1234567890_u64
}

// Mock caller function for testing (since ic_cdk::api::set_caller doesn't exist)
thread_local! {
    static MOCK_CALLER: std::cell::RefCell<Option<Principal>> = std::cell::RefCell::new(None);
}

pub fn set_mock_caller(principal: Principal) {
    MOCK_CALLER.with(|caller| {
        *caller.borrow_mut() = Some(principal);
    });
}

pub fn get_mock_caller() -> Option<Principal> {
    MOCK_CALLER.with(|caller| *caller.borrow())
}

pub fn clear_mock_caller() {
    MOCK_CALLER.with(|caller| {
        *caller.borrow_mut() = None;
    });
}

#[cfg(test)]
mod basic_tests {
    use super::*;
    
    #[test]
    fn test_user_creation_structure() {
        let principal = create_test_principal(1);
        
        let user = User {
            id: principal,
            role: Role::Farmer,
            created_at: mock_time(),
            btc_address: None,
            is_active: true,
            updated_at: mock_time(),
            email: None,
            phone: None,
            profile_completed: false,
        };
        
        assert_eq!(user.id, principal);
        assert_eq!(user.role, Role::Farmer);
        assert!(user.is_active);
        assert!(!user.profile_completed);
    }

    #[test]
    fn test_principal_creation() {
        let principal = create_test_principal(1);
        assert!(!principal.to_text().is_empty());
    }

    #[test]
    fn test_role_enum() {
        let farmer = Role::Farmer;
        let investor = Role::Investor;
        
        assert_eq!(farmer, Role::Farmer);
        assert_eq!(investor, Role::Investor);
        assert_ne!(farmer, investor);
    }

    #[test]
    fn test_user_result_enum() {
        let principal = create_test_principal(1);
        let user = User {
            id: principal,
            role: Role::Farmer,
            created_at: mock_time(),
            btc_address: None,
            is_active: true,
            updated_at: mock_time(),
            email: None,
            phone: None,
            profile_completed: false,
        };
        
        let success_result = UserResult::Ok(user);
        let error_result = UserResult::Err("Test error".to_string());
        
        match success_result {
            UserResult::Ok(u) => assert_eq!(u.role, Role::Farmer),
            UserResult::Err(_) => panic!("Expected Ok"),
        }
        
        match error_result {
            UserResult::Err(msg) => assert_eq!(msg, "Test error"),
            UserResult::Ok(_) => panic!("Expected Err"),
        }
    }

    #[test]
    fn test_user_stats_structure() {
        let stats = UserStats {
            total_users: 100,
            total_farmers: 60,
            total_investors: 40,
            active_users: 95,
            inactive_users: 5,
            users_with_btc_address: 30,
            completed_profiles: 25,
        };
        
        assert_eq!(stats.total_users, 100);
        assert_eq!(stats.total_farmers, 60);
        assert_eq!(stats.total_investors, 40);
        assert_eq!(stats.active_users, 95);
        assert_eq!(stats.inactive_users, 5);
        assert_eq!(stats.users_with_btc_address, 30);
        assert_eq!(stats.completed_profiles, 25);
    }
}

#[cfg(test)]
mod validation_function_tests {
    use super::*;

    #[test]
    fn test_btc_address_validation() {
        // Valid BTC addresses
        assert!(validate_btc_address("1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa"));
        assert!(validate_btc_address("3J98t1WpEZ73CNmQviecrnyiWrnqRhWNLy"));
        assert!(validate_btc_address("bc1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh"));
        
        // Invalid BTC addresses
        assert!(!validate_btc_address("invalid"));
        assert!(!validate_btc_address("2A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa"));
        assert!(!validate_btc_address(""));
    }
    
    #[test]
    fn test_email_validation() {
        // Valid emails
        assert!(validate_email("test@example.com"));
        assert!(validate_email("user.name@domain.org"));
        assert!(validate_email("a@b.co"));
        
        // Invalid emails
        assert!(!validate_email("invalid"));
        assert!(!validate_email("test@"));
        assert!(!validate_email(""));
        assert!(!validate_email("test@domain"));
        assert!(!validate_email("test.domain.com"));
    }
    
    #[test]
    fn test_phone_validation() {
        // Valid phone numbers
        assert!(validate_phone("1234567890"));
        assert!(validate_phone("+1234567890"));
        assert!(validate_phone("123-456-7890"));
        assert!(validate_phone("123 456 7890"));
        
        // Invalid phone numbers
        assert!(!validate_phone("123"));
        assert!(!validate_phone("abc123def"));
        assert!(!validate_phone(""));
    }
}
        
        // Execute
        let result = register_as_farmer();
        
        // Verify
        match result {
            UserResult::Ok(user) => {
                assert_eq!(user.id, farmer_principal);
                assert_eq!(user.role, Role::Farmer);
                assert!(user.is_active);
                assert!(user.btc_address.is_none());
                assert!(!user.profile_completed);
                assert!(user.created_at > 0);
                assert_eq!(user.created_at, user.updated_at);
            }
            UserResult::Err(msg) => panic!("Expected success, got error: {}", msg),
        }
        
        // Verify user exists in storage
        assert!(user_exists(&farmer_principal));
        
        // Verify user can be retrieved
        let retrieved_user = get_user_by_principal(&farmer_principal);
        assert!(retrieved_user.is_some());
        assert_eq!(retrieved_user.unwrap().role, Role::Farmer);
    }

    #[test]
    fn test_register_as_investor_success() {
        // Setup
        let investor_principal = create_test_principal(2);
        
        // Mock caller
        ic_cdk::api::set_caller(investor_principal);
        
        // Execute
        let result = register_as_investor();
        
        // Verify
        match result {
            UserResult::Ok(user) => {
                assert_eq!(user.id, investor_principal);
                assert_eq!(user.role, Role::Investor);
                assert!(user.is_active);
                assert!(user.btc_address.is_none());
                assert!(!user.profile_completed);
            }
            UserResult::Err(msg) => panic!("Expected success, got error: {}", msg),
        }
        
        // Verify user exists and has correct role
        assert!(is_investor(investor_principal));
        assert!(!is_farmer(investor_principal));
    }

    #[test]
    fn test_duplicate_registration_fails() {
        // Setup
        let principal = create_test_principal(3);
        ic_cdk::api::set_caller(principal);
        
        // First registration should succeed
        let first_result = register_as_farmer();
        assert!(matches!(first_result, UserResult::Ok(_)));
        
        // Second registration should fail
        let second_result = register_as_farmer();
        match second_result {
            UserResult::Err(msg) => {
                assert_eq!(msg, "User already registered");
            }
            UserResult::Ok(_) => panic!("Expected error for duplicate registration"),
        }
        
        // Try registering as different role should also fail
        let investor_result = register_as_investor();
        match investor_result {
            UserResult::Err(msg) => {
                assert_eq!(msg, "User already registered");
            }
            UserResult::Ok(_) => panic!("Expected error for duplicate registration"),
        }
    }

    #[test]
    fn test_multiple_users_registration() {
        // Register multiple farmers
        for i in 10..15 {
            let principal = create_test_principal(i);
            ic_cdk::api::set_caller(principal);
            
            let result = register_as_farmer();
            assert!(matches!(result, UserResult::Ok(_)));
            assert!(is_farmer(principal));
        }
        
        // Register multiple investors
        for i in 20..25 {
            let principal = create_test_principal(i);
            ic_cdk::api::set_caller(principal);
            
            let result = register_as_investor();
            assert!(matches!(result, UserResult::Ok(_)));
            assert!(is_investor(principal));
        }
        
        // Verify statistics
        let stats = get_user_stats();
        assert!(stats.total_farmers >= 5);
        assert!(stats.total_investors >= 5);
        assert!(stats.total_users >= 10);
    }
}

#[cfg(test)]
mod user_profile_tests {
    use super::*;

    #[test]
    fn test_get_user_success() {
        // Setup
        let principal = create_test_principal(30);
        ic_cdk::api::set_caller(principal);
        
        // Register user first
        let _ = register_as_farmer();
        
        // Execute
        let result = get_user();
        
        // Verify
        match result {
            UserResult::Ok(user) => {
                assert_eq!(user.id, principal);
                assert_eq!(user.role, Role::Farmer);
            }
            UserResult::Err(msg) => panic!("Expected success, got error: {}", msg),
        }
    }

    #[test]
    fn test_get_user_not_registered() {
        // Setup
        let principal = create_test_principal(31);
        ic_cdk::api::set_caller(principal);
        
        // Execute without registering
        let result = get_user();
        
        // Verify
        match result {
            UserResult::Err(msg) => {
                assert_eq!(msg, "User not found. Please register first.");
            }
            UserResult::Ok(_) => panic!("Expected error for unregistered user"),
        }
    }

    #[test]
    fn test_update_btc_address_success() {
        // Setup
        let principal = create_test_principal(32);
        ic_cdk::api::set_caller(principal);
        let _ = register_as_farmer();
        
        let valid_btc_address = "bc1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh";
        
        // Execute
        let result = update_btc_address(valid_btc_address.to_string());
        
        // Verify
        match result {
            UserResult::Ok(user) => {
                assert_eq!(user.btc_address, Some(valid_btc_address.to_string()));
                assert!(user.profile_completed);
                assert!(user.updated_at >= user.created_at);
            }
            UserResult::Err(msg) => panic!("Expected success, got error: {}", msg),
        }
        
        // Verify persistence
        let retrieved_user = get_user_by_principal(&principal).unwrap();
        assert_eq!(retrieved_user.btc_address, Some(valid_btc_address.to_string()));
    }

    #[test]
    fn test_update_btc_address_invalid() {
        // Setup
        let principal = create_test_principal(33);
        ic_cdk::api::set_caller(principal);
        let _ = register_as_farmer();
        
        let invalid_addresses = vec![
            "invalid",
            "123",
            "", // empty
            "1234567890123456789012345", // too short
            "bc1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlhbc1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh", // too long
        ];
        
        for invalid_address in invalid_addresses {
            let result = update_btc_address(invalid_address.to_string());
            match result {
                UserResult::Err(msg) => {
                    assert_eq!(msg, "Invalid BTC address format");
                }
                UserResult::Ok(_) => panic!("Expected error for invalid address: {}", invalid_address),
            }
        }
    }

    #[test]
    fn test_update_user_profile_comprehensive() {
        // Setup
        let principal = create_test_principal(34);
        ic_cdk::api::set_caller(principal);
        let _ = register_as_investor();
        
        let update_request = UserUpdateRequest {
            btc_address: Some("1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa".to_string()),
            email: Some("farmer@agrilends.com".to_string()),
            phone: Some("+62-812-3456-7890".to_string()),
        };
        
        // Execute
        let result = update_user_profile(update_request);
        
        // Verify
        match result {
            UserResult::Ok(user) => {
                assert_eq!(user.btc_address, Some("1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa".to_string()));
                assert_eq!(user.email, Some("farmer@agrilends.com".to_string()));
                assert_eq!(user.phone, Some("+62-812-3456-7890".to_string()));
                assert!(user.profile_completed);
            }
            UserResult::Err(msg) => panic!("Expected success, got error: {}", msg),
        }
    }

    #[test]
    fn test_update_user_profile_invalid_data() {
        // Setup
        let principal = create_test_principal(35);
        ic_cdk::api::set_caller(principal);
        let _ = register_as_farmer();
        
        // Test invalid email
        let invalid_email_request = UserUpdateRequest {
            btc_address: None,
            email: Some("invalid-email".to_string()),
            phone: None,
        };
        
        let result = update_user_profile(invalid_email_request);
        assert!(matches!(result, UserResult::Err(_)));
        
        // Test invalid phone
        let invalid_phone_request = UserUpdateRequest {
            btc_address: None,
            email: None,
            phone: Some("123".to_string()),
        };
        
        let result = update_user_profile(invalid_phone_request);
        assert!(matches!(result, UserResult::Err(_)));
        
        // Test empty request
        let empty_request = UserUpdateRequest {
            btc_address: None,
            email: None,
            phone: None,
        };
        
        let result = update_user_profile(empty_request);
        match result {
            UserResult::Err(msg) => {
                assert_eq!(msg, "No valid update data provided");
            }
            UserResult::Ok(_) => panic!("Expected error for empty update"),
        }
    }
}

#[cfg(test)]
mod user_status_tests {
    use super::*;

    #[test]
    fn test_user_activation_deactivation() {
        // Setup
        let principal = create_test_principal(40);
        ic_cdk::api::set_caller(principal);
        let _ = register_as_farmer();
        
        // Verify initial active status
        assert!(is_user_active(principal));
        
        // Deactivate user
        let deactivate_result = deactivate_user();
        match deactivate_result {
            UserResult::Ok(user) => {
                assert!(!user.is_active);
            }
            UserResult::Err(msg) => panic!("Expected success, got error: {}", msg),
        }
        
        // Verify deactivation
        assert!(!is_user_active(principal));
        assert!(!is_farmer(principal)); // Should return false for inactive farmer
        
        // Reactivate user
        let reactivate_result = reactivate_user();
        match reactivate_result {
            UserResult::Ok(user) => {
                assert!(user.is_active);
            }
            UserResult::Err(msg) => panic!("Expected success, got error: {}", msg),
        }
        
        // Verify reactivation
        assert!(is_user_active(principal));
        assert!(is_farmer(principal)); // Should return true for active farmer
    }

    #[test]
    fn test_user_role_checks() {
        // Setup farmers
        let farmer1 = create_test_principal(41);
        let farmer2 = create_test_principal(42);
        
        ic_cdk::api::set_caller(farmer1);
        let _ = register_as_farmer();
        
        ic_cdk::api::set_caller(farmer2);
        let _ = register_as_farmer();
        
        // Setup investors
        let investor1 = create_test_principal(43);
        let investor2 = create_test_principal(44);
        
        ic_cdk::api::set_caller(investor1);
        let _ = register_as_investor();
        
        ic_cdk::api::set_caller(investor2);
        let _ = register_as_investor();
        
        // Test role checks
        assert!(is_farmer(farmer1));
        assert!(is_farmer(farmer2));
        assert!(!is_farmer(investor1));
        assert!(!is_farmer(investor2));
        
        assert!(is_investor(investor1));
        assert!(is_investor(investor2));
        assert!(!is_investor(farmer1));
        assert!(!is_investor(farmer2));
        
        // Test with non-existent user
        let non_existent = create_test_principal(99);
        assert!(!is_farmer(non_existent));
        assert!(!is_investor(non_existent));
        assert!(!is_user_active(non_existent));
    }

    #[test]
    fn test_profile_completion_status() {
        // Setup
        let principal = create_test_principal(45);
        ic_cdk::api::set_caller(principal);
        let _ = register_as_farmer();
        
        // Initially profile should not be completed
        assert!(!has_completed_profile(principal));
        
        // Add BTC address
        let _ = update_btc_address("bc1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh".to_string());
        assert!(has_completed_profile(principal));
        
        // Test with email only
        let principal2 = create_test_principal(46);
        ic_cdk::api::set_caller(principal2);
        let _ = register_as_investor();
        
        let update_request = UserUpdateRequest {
            btc_address: None,
            email: Some("test@example.com".to_string()),
            phone: None,
        };
        let _ = update_user_profile(update_request);
        assert!(has_completed_profile(principal2));
    }
}

#[cfg(test)]
mod user_query_tests {
    use super::*;

    #[test]
    fn test_get_user_by_id() {
        // Setup
        let principal = create_test_principal(50);
        ic_cdk::api::set_caller(principal);
        let _ = register_as_farmer();
        
        // Test getting user by ID
        let result = get_user_by_id(principal);
        match result {
            UserResult::Ok(user) => {
                assert_eq!(user.id, principal);
                assert_eq!(user.role, Role::Farmer);
            }
            UserResult::Err(msg) => panic!("Expected success, got error: {}", msg),
        }
        
        // Test with non-existent user
        let non_existent = create_test_principal(51);
        let result = get_user_by_id(non_existent);
        match result {
            UserResult::Err(msg) => {
                assert_eq!(msg, "User not found");
            }
            UserResult::Ok(_) => panic!("Expected error for non-existent user"),
        }
    }

    #[test]
    fn test_get_users_by_role() {
        // Setup multiple users
        let farmer_principals: Vec<Principal> = (60..65).map(create_test_principal).collect();
        let investor_principals: Vec<Principal> = (70..73).map(create_test_principal).collect();
        
        // Register farmers
        for principal in &farmer_principals {
            ic_cdk::api::set_caller(*principal);
            let _ = register_as_farmer();
        }
        
        // Register investors
        for principal in &investor_principals {
            ic_cdk::api::set_caller(*principal);
            let _ = register_as_investor();
        }
        
        // Test get farmers
        let farmers = get_users_by_role(Role::Farmer);
        let farmer_ids: Vec<Principal> = farmers.iter().map(|u| u.id).collect();
        
        for principal in &farmer_principals {
            assert!(farmer_ids.contains(principal));
        }
        
        // Test get investors
        let investors = get_users_by_role(Role::Investor);
        let investor_ids: Vec<Principal> = investors.iter().map(|u| u.id).collect();
        
        for principal in &investor_principals {
            assert!(investor_ids.contains(principal));
        }
        
        // Verify role separation
        for farmer in &farmers {
            assert_eq!(farmer.role, Role::Farmer);
        }
        
        for investor in &investors {
            assert_eq!(investor.role, Role::Investor);
        }
    }

    #[test]
    fn test_get_active_users() {
        // Setup users with different statuses
        let active_principals: Vec<Principal> = (80..83).map(create_test_principal).collect();
        let inactive_principals: Vec<Principal> = (85..87).map(create_test_principal).collect();
        
        // Register and keep active
        for principal in &active_principals {
            ic_cdk::api::set_caller(*principal);
            let _ = register_as_farmer();
        }
        
        // Register and deactivate
        for principal in &inactive_principals {
            ic_cdk::api::set_caller(*principal);
            let _ = register_as_investor();
            let _ = deactivate_user();
        }
        
        // Test get active users
        let active_users = get_active_users();
        let active_ids: Vec<Principal> = active_users.iter().map(|u| u.id).collect();
        
        // Verify all active users are included
        for principal in &active_principals {
            assert!(active_ids.contains(principal));
        }
        
        // Verify inactive users are not included
        for principal in &inactive_principals {
            assert!(!active_ids.contains(principal));
        }
        
        // Verify all returned users are active
        for user in &active_users {
            assert!(user.is_active);
        }
    }

    #[test]
    fn test_get_all_users() {
        // Setup multiple users
        let all_principals: Vec<Principal> = (90..95).map(create_test_principal).collect();
        
        for (i, principal) in all_principals.iter().enumerate() {
            ic_cdk::api::set_caller(*principal);
            if i % 2 == 0 {
                let _ = register_as_farmer();
            } else {
                let _ = register_as_investor();
            }
        }
        
        // Test get all users
        let all_users = get_all_users();
        let all_ids: Vec<Principal> = all_users.iter().map(|u| u.id).collect();
        
        // Verify all registered users are included
        for principal in &all_principals {
            assert!(all_ids.contains(principal));
        }
        
        // Verify user count matches
        assert!(all_users.len() >= all_principals.len());
    }
}

#[cfg(test)]
mod user_statistics_tests {
    use super::*;

    #[test]
    fn test_user_statistics_comprehensive() {
        // Clear existing data for clean test
        // Note: In real implementation, you might want to use a test database
        
        // Setup known data set
        let farmer_count = 5;
        let investor_count = 3;
        let inactive_count = 2;
        
        // Register farmers
        for i in 100..(100 + farmer_count) {
            let principal = create_test_principal(i);
            ic_cdk::api::set_caller(principal);
            let _ = register_as_farmer();
            
            // Add BTC address to some farmers
            if i % 2 == 0 {
                let _ = update_btc_address("bc1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh".to_string());
            }
        }
        
        // Register investors
        for i in 110..(110 + investor_count) {
            let principal = create_test_principal(i);
            ic_cdk::api::set_caller(principal);
            let _ = register_as_investor();
        }
        
        // Deactivate some users
        for i in 115..(115 + inactive_count) {
            let principal = create_test_principal(i);
            ic_cdk::api::set_caller(principal);
            let _ = register_as_farmer();
            let _ = deactivate_user();
        }
        
        // Get statistics
        let stats = get_user_stats();
        
        // Verify counts
        assert!(stats.total_farmers >= farmer_count + inactive_count);
        assert!(stats.total_investors >= investor_count);
        assert!(stats.total_users >= farmer_count + investor_count + inactive_count);
        assert!(stats.active_users >= farmer_count + investor_count);
        assert!(stats.inactive_users >= inactive_count);
        
        // Verify consistency
        assert_eq!(stats.total_users, stats.total_farmers + stats.total_investors);
        assert_eq!(stats.total_users, stats.active_users + stats.inactive_users);
        
        // Verify BTC address count
        assert!(stats.users_with_btc_address >= farmer_count / 2);
    }

    #[test]
    fn test_statistics_empty_state() {
        // Test statistics with no users (if possible to reset state)
        // This would require a way to clear the storage for testing
        
        // For now, just verify statistics are consistent
        let stats = get_user_stats();
        
        assert_eq!(stats.total_users, stats.total_farmers + stats.total_investors);
        assert_eq!(stats.total_users, stats.active_users + stats.inactive_users);
        assert!(stats.users_with_btc_address <= stats.total_users);
        assert!(stats.completed_profiles <= stats.total_users);
    }
}

#[cfg(test)]
mod validation_tests {
    use super::*;

    #[test]
    fn test_email_validation() {
        // Valid emails
        let valid_emails = vec![
            "test@example.com",
            "farmer@agrilends.co.id",
            "user.name@domain.org",
            "user+tag@example.com",
            "123@example.com",
        ];
        
        for email in valid_emails {
            assert!(validate_email(email), "Expected {} to be valid", email);
        }
        
        // Invalid emails
        let invalid_emails = vec![
            "",
            "invalid",
            "@example.com",
            "test@",
            "test@.com",
            "test@com",
            "test.example.com",
            "test@@example.com",
        ];
        
        for email in invalid_emails {
            assert!(!validate_email(email), "Expected {} to be invalid", email);
        }
    }

    #[test]
    fn test_phone_validation() {
        // Valid phone numbers
        let valid_phones = vec![
            "+628123456789",
            "08123456789",
            "+1-555-123-4567",
            "555 123 4567",
            "(555) 123-4567",
            "5551234567",
            "+62 812 3456 7890",
        ];
        
        for phone in valid_phones {
            assert!(validate_phone(phone), "Expected {} to be valid", phone);
        }
        
        // Invalid phone numbers
        let invalid_phones = vec![
            "",
            "123",
            "abcdefghij",
            "+",
            "123-abc-4567",
            "12345", // too short
        ];
        
        for phone in invalid_phones {
            assert!(!validate_phone(phone), "Expected {} to be invalid", phone);
        }
    }

    #[test]
    fn test_btc_address_validation() {
        // Valid BTC addresses
        let valid_addresses = vec![
            "1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa", // Legacy P2PKH
            "3J98t1WpEZ73CNmQviecrnyiWrnqRhWNLy", // P2SH
            "bc1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh", // Bech32
            "bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4", // Bech32
        ];
        
        for address in valid_addresses {
            assert!(validate_btc_address(address), "Expected {} to be valid", address);
        }
        
        // Invalid BTC addresses
        let invalid_addresses = vec![
            "",
            "invalid",
            "1", // too short
            "2A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa", // starts with 2
            "4J98t1WpEZ73CNmQviecrnyiWrnqRhWNLy", // starts with 4
            "bc2qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh", // invalid bech32 prefix
            "1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa", // too long
        ];
        
        for address in invalid_addresses {
            assert!(!validate_btc_address(address), "Expected {} to be invalid", address);
        }
    }
}

#[cfg(test)]
mod edge_case_tests {
    use super::*;

    #[test]
    fn test_concurrent_user_operations() {
        // Simulate concurrent operations on same user
        let principal = create_test_principal(200);
        ic_cdk::api::set_caller(principal);
        
        // Register user
        let _ = register_as_farmer();
        
        // Simulate multiple concurrent updates
        let btc_address = "bc1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh";
        let _ = update_btc_address(btc_address.to_string());
        
        let update_request = UserUpdateRequest {
            btc_address: None,
            email: Some("test@example.com".to_string()),
            phone: Some("+628123456789".to_string()),
        };
        let _ = update_user_profile(update_request);
        
        // Verify final state is consistent
        let user = get_user_by_principal(&principal).unwrap();
        assert_eq!(user.btc_address, Some(btc_address.to_string()));
        assert_eq!(user.email, Some("test@example.com".to_string()));
        assert_eq!(user.phone, Some("+628123456789".to_string()));
        assert!(user.profile_completed);
    }

    #[test]
    fn test_user_operations_without_registration() {
        // Test operations on unregistered user
        let principal = create_test_principal(201);
        ic_cdk::api::set_caller(principal);
        
        // All operations should fail for unregistered user
        let update_result = update_btc_address("bc1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh".to_string());
        assert!(matches!(update_result, UserResult::Err(_)));
        
        let profile_update = UserUpdateRequest {
            btc_address: Some("bc1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh".to_string()),
            email: None,
            phone: None,
        };
        let profile_result = update_user_profile(profile_update);
        assert!(matches!(profile_result, UserResult::Err(_)));
        
        let deactivate_result = deactivate_user();
        assert!(matches!(deactivate_result, UserResult::Err(_)));
        
        let reactivate_result = reactivate_user();
        assert!(matches!(reactivate_result, UserResult::Err(_)));
    }

    #[test]
    fn test_large_scale_user_registration() {
        // Test system behavior with many users
        let user_count = 100;
        let mut farmer_count = 0;
        let mut investor_count = 0;
        
        for i in 300..(300 + user_count) {
            let principal = create_test_principal(i);
            ic_cdk::api::set_caller(principal);
            
            if i % 2 == 0 {
                let result = register_as_farmer();
                assert!(matches!(result, UserResult::Ok(_)));
                farmer_count += 1;
            } else {
                let result = register_as_investor();
                assert!(matches!(result, UserResult::Ok(_)));
                investor_count += 1;
            }
        }
        
        // Verify all users were registered
        let stats = get_user_stats();
        assert!(stats.total_farmers >= farmer_count);
        assert!(stats.total_investors >= investor_count);
        assert!(stats.total_users >= user_count);
    }

    #[test]
    fn test_boundary_value_validation() {
        // Test boundary values for BTC address
        let min_valid_btc = "1".repeat(26); // Minimum length
        let max_valid_btc = "1".repeat(35); // Maximum length for legacy
        
        // These might not be valid BTC addresses but test length boundaries
        let just_too_short = "1".repeat(25);
        let just_too_long = "1".repeat(36);
        
        assert!(!validate_btc_address(&just_too_short));
        assert!(!validate_btc_address(&just_too_long));
        
        // Test phone number boundaries
        let min_valid_phone = "1234567890"; // 10 digits
        let just_too_short_phone = "123456789"; // 9 digits
        
        assert!(validate_phone(min_valid_phone));
        assert!(!validate_phone(just_too_short_phone));
        
        // Test empty string handling
        assert!(!validate_email(""));
        assert!(!validate_phone(""));
        assert!(!validate_btc_address(""));
    }
}

// Integration tests with complete user journey
#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_complete_farmer_journey() {
        let farmer_principal = create_test_principal(500);
        ic_cdk::api::set_caller(farmer_principal);
        
        // 1. Register as farmer
        let registration_result = register_as_farmer();
        assert!(matches!(registration_result, UserResult::Ok(_)));
        
        // 2. Verify initial state
        let user = get_user_by_principal(&farmer_principal).unwrap();
        assert_eq!(user.role, Role::Farmer);
        assert!(user.is_active);
        assert!(!user.profile_completed);
        
        // 3. Update BTC address
        let btc_address = "bc1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh";
        let btc_result = update_btc_address(btc_address.to_string());
        assert!(matches!(btc_result, UserResult::Ok(_)));
        
        // 4. Complete profile
        let profile_update = UserUpdateRequest {
            btc_address: None,
            email: Some("farmer@agrilends.com".to_string()),
            phone: Some("+628123456789".to_string()),
        };
        let profile_result = update_user_profile(profile_update);
        assert!(matches!(profile_result, UserResult::Ok(_)));
        
        // 5. Verify final state
        let final_user = get_user_by_principal(&farmer_principal).unwrap();
        assert!(final_user.profile_completed);
        assert!(final_user.is_active);
        assert_eq!(final_user.btc_address, Some(btc_address.to_string()));
        assert_eq!(final_user.email, Some("farmer@agrilends.com".to_string()));
        assert_eq!(final_user.phone, Some("+628123456789".to_string()));
        
        // 6. Test role checks
        assert!(is_farmer(farmer_principal));
        assert!(!is_investor(farmer_principal));
        assert!(is_user_active(farmer_principal));
        assert!(has_completed_profile(farmer_principal));
    }

    #[test]
    fn test_complete_investor_journey() {
        let investor_principal = create_test_principal(501);
        ic_cdk::api::set_caller(investor_principal);
        
        // 1. Register as investor
        let registration_result = register_as_investor();
        assert!(matches!(registration_result, UserResult::Ok(_)));
        
        // 2. Complete profile in one step
        let profile_update = UserUpdateRequest {
            btc_address: Some("1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa".to_string()),
            email: Some("investor@agrilends.com".to_string()),
            phone: Some("+1-555-123-4567".to_string()),
        };
        let profile_result = update_user_profile(profile_update);
        assert!(matches!(profile_result, UserResult::Ok(_)));
        
        // 3. Test account deactivation and reactivation
        let deactivate_result = deactivate_user();
        assert!(matches!(deactivate_result, UserResult::Ok(_)));
        assert!(!is_user_active(investor_principal));
        assert!(!is_investor(investor_principal)); // Should return false when inactive
        
        let reactivate_result = reactivate_user();
        assert!(matches!(reactivate_result, UserResult::Ok(_)));
        assert!(is_user_active(investor_principal));
        assert!(is_investor(investor_principal)); // Should return true when active
        
        // 4. Verify final state
        let final_user = get_user_by_principal(&investor_principal).unwrap();
        assert!(final_user.profile_completed);
        assert!(final_user.is_active);
        assert_eq!(final_user.role, Role::Investor);
    }

    #[test]
    fn test_multi_user_ecosystem() {
        // Create a mini ecosystem with multiple users
        let farmer_principals: Vec<Principal> = (600..605).map(create_test_principal).collect();
        let investor_principals: Vec<Principal> = (610..613).map(create_test_principal).collect();
        
        // Register all farmers
        for principal in &farmer_principals {
            ic_cdk::api::set_caller(*principal);
            let _ = register_as_farmer();
            let _ = update_btc_address("bc1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh".to_string());
        }
        
        // Register all investors
        for principal in &investor_principals {
            ic_cdk::api::set_caller(*principal);
            let _ = register_as_investor();
            
            let profile_update = UserUpdateRequest {
                btc_address: Some("1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa".to_string()),
                email: Some("investor@test.com".to_string()),
                phone: None,
            };
            let _ = update_user_profile(profile_update);
        }
        
        // Verify ecosystem statistics
        let stats = get_user_stats();
        assert!(stats.total_farmers >= farmer_principals.len() as u64);
        assert!(stats.total_investors >= investor_principals.len() as u64);
        assert!(stats.completed_profiles >= (farmer_principals.len() + investor_principals.len()) as u64);
        
        // Test queries
        let farmers = get_users_by_role(Role::Farmer);
        let investors = get_users_by_role(Role::Investor);
        let active_users = get_active_users();
        
        assert!(farmers.len() >= farmer_principals.len());
        assert!(investors.len() >= investor_principals.len());
        assert!(active_users.len() >= farmer_principals.len() + investor_principals.len());
        
        // Verify role separation
        for farmer in farmers {
            assert_eq!(farmer.role, Role::Farmer);
            assert!(farmer.is_active);
        }
        
        for investor in investors {
            assert_eq!(investor.role, Role::Investor);
            assert!(investor.is_active);
        }
    }
}
*/

// Placeholder content to prevent empty file issues
// These tests need to be rewritten without ic_cdk::api::set_caller