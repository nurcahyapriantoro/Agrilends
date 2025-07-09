pub mod rwa_nft_tests;
pub mod loan_lifecycle_tests;

pub use loan_lifecycle_tests::*;

// Basic test untuk memastikan user creation bekerja
#[cfg(test)]
mod basic_tests {
    use candid::Principal;
    use crate::user_management::*;
    
    // Helper function to create test principal
    fn create_test_principal(id: u8) -> Principal {
        Principal::from_slice(&[id; 29])
    }
    
    // Helper function to create mock time
    fn mock_time() -> u64 {
        1234567890_u64
    }
    
    #[test]
    fn test_user_creation_structure() {
        let principal = create_test_principal(1);
        
        // Create user with mock time instead of using ic_cdk::api::time()
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
            UserResult::Ok(u) => assert_eq!(u.id, principal),
            UserResult::Err(_) => panic!("Expected Ok result"),
        }
        
        match error_result {
            UserResult::Ok(_) => panic!("Expected Err result"),
            UserResult::Err(msg) => assert_eq!(msg, "Test error"),
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
    }
}

// Remove problematic tests that use ic_cdk::api::set_caller since it doesn't exist
// These tests would need to be rewritten for proper IC testing environment

pub mod user_management_tests;

/*
#[cfg(test)]
mod user_management_integration_tests {
    use super::*;

            fn create_test_principal(id: u8) -> Principal {
                let mut bytes = [0u8; 29];
                bytes[0] = id;
                Principal::from_slice(&bytes)
            }

            #[test]
            fn test_register_farmer_complete_flow() {
                let principal = create_test_principal(50);
                
                // Mock the caller
                ic_cdk::api::set_caller(principal);
                
                // Register as farmer
                let result = register_as_farmer();
                assert!(matches!(result, UserResult::Ok(_)));
                
                // Verify user exists and has correct role
                assert!(user_exists(&principal));
                assert!(is_farmer(principal));
                assert!(!is_investor(principal));
                assert!(is_user_active(principal));
                
                // Get user and verify details
                let user = get_user_by_principal(&principal).unwrap();
                assert_eq!(user.role, Role::Farmer);
                assert!(user.is_active);
                assert!(!user.profile_completed);
            }

            #[test]
            fn test_register_investor_complete_flow() {
                let principal = create_test_principal(51);
                
                ic_cdk::api::set_caller(principal);
                
                let result = register_as_investor();
                assert!(matches!(result, UserResult::Ok(_)));
                
                assert!(user_exists(&principal));
                assert!(is_investor(principal));
                assert!(!is_farmer(principal));
                
                let user = get_user_by_principal(&principal).unwrap();
                assert_eq!(user.role, Role::Investor);
            }

            #[test]
            fn test_duplicate_registration_prevention() {
                let principal = create_test_principal(52);
                ic_cdk::api::set_caller(principal);
                
                // First registration should succeed
                let first_result = register_as_farmer();
                assert!(matches!(first_result, UserResult::Ok(_)));
                
                // Second registration should fail
                let second_result = register_as_farmer();
                assert!(matches!(second_result, UserResult::Err(_)));
                
                // Different role registration should also fail
                let investor_result = register_as_investor();
                assert!(matches!(investor_result, UserResult::Err(_)));
            }

            #[test]
            fn test_user_profile_update_complete() {
                let principal = create_test_principal(53);
                ic_cdk::api::set_caller(principal);
                
                // Register user first
                let _ = register_as_farmer();
                
                // Update profile
                let update_request = UserUpdateRequest {
                    btc_address: Some("bc1qar0srrr7xfkvy5l643lydnw9re59gtzzwf5mdq".to_string()),
                    email: Some("farmer@example.com".to_string()),
                    phone: Some("+1234567890".to_string()),
                };
                
                let result = update_user_profile(update_request);
                assert!(matches!(result, UserResult::Ok(_)));
                
                // Verify updates
                let user = get_user_by_principal(&principal).unwrap();
                assert_eq!(user.btc_address, Some("bc1qar0srrr7xfkvy5l643lydnw9re59gtzzwf5mdq".to_string()));
                assert_eq!(user.email, Some("farmer@example.com".to_string()));
                assert_eq!(user.phone, Some("+1234567890".to_string()));
                assert!(user.profile_completed);
            }

            #[test]
            fn test_btc_address_update_validation() {
                let principal = create_test_principal(54);
                ic_cdk::api::set_caller(principal);
                let _ = register_as_investor();
                
                // Valid BTC address should work
                let valid_result = update_btc_address("bc1qar0srrr7xfkvy5l643lydnw9re59gtzzwf5mdq".to_string());
                assert!(matches!(valid_result, UserResult::Ok(_)));
                
                // Invalid BTC address should fail
                let invalid_result = update_btc_address("invalid_btc_address".to_string());
                assert!(matches!(invalid_result, UserResult::Err(_)));
            }

            #[test]
            fn test_user_activation_deactivation_cycle() {
                let principal = create_test_principal(55);
                ic_cdk::api::set_caller(principal);
                let _ = register_as_farmer();
                
                // Initially active
                assert!(is_user_active(principal));
                
                // Deactivate
                let deactivate_result = deactivate_user();
                assert!(matches!(deactivate_result, UserResult::Ok(_)));
                assert!(!is_user_active(principal));
                
                // Reactivate
                let reactivate_result = reactivate_user();
                assert!(matches!(reactivate_result, UserResult::Ok(_)));
                assert!(is_user_active(principal));
            }

            #[test]
            fn test_user_statistics_accuracy() {
                // Clear any existing users and register specific counts
                let farmer_count = 3;
                let investor_count = 2;
                
                // Register farmers
                for i in 60..60 + farmer_count {
                    let principal = create_test_principal(i);
                    ic_cdk::api::set_caller(principal);
                    let _ = register_as_farmer();
                }
                
                // Register investors
                for i in 70..70 + investor_count {
                    let principal = create_test_principal(i);
                    ic_cdk::api::set_caller(principal);
                    let _ = register_as_investor();
                }
                
                let stats = get_user_stats();
                assert!(stats.total_farmers >= farmer_count as u64);
                assert!(stats.total_investors >= investor_count as u64);
                assert!(stats.total_users >= (farmer_count + investor_count) as u64);
            }

            #[test]
            fn test_get_users_by_role_functionality() {
                // Register test users
                let farmer_principal = create_test_principal(80);
                let investor_principal = create_test_principal(81);
                
                ic_cdk::api::set_caller(farmer_principal);
                let _ = register_as_farmer();
                
                ic_cdk::api::set_caller(investor_principal);
                let _ = register_as_investor();
                
                // Test get farmers
                let farmers = get_users_by_role(Role::Farmer);
                assert!(farmers.iter().any(|u| u.id == farmer_principal));
                
                // Test get investors
                let investors = get_users_by_role(Role::Investor);
                assert!(investors.iter().any(|u| u.id == investor_principal));
            }

            #[test]
            fn test_user_operations_without_registration() {
                let unregistered_principal = create_test_principal(90);
                ic_cdk::api::set_caller(unregistered_principal);
                
                // Operations should fail for unregistered user
                let get_result = get_user();
                assert!(matches!(get_result, UserResult::Err(_)));
                
                let update_result = update_user_profile(UserUpdateRequest {
                    btc_address: Some("bc1qar0srrr7xfkvy5l643lydnw9re59gtzzwf5mdq".to_string()),
                    email: None,
                    phone: None,
                });
                assert!(matches!(update_result, UserResult::Err(_)));
                
                let deactivate_result = deactivate_user();
                assert!(matches!(deactivate_result, UserResult::Err(_)));
            }

            #[test]
            fn test_email_validation_comprehensive() {
                let principal = create_test_principal(91);
                ic_cdk::api::set_caller(principal);
                let _ = register_as_farmer();
                
                let valid_emails = vec![
                    "test@example.com",
                    "user.name@domain.org",
                    "user+tag@example.co.uk",
                    "123@domain.com",
                ];
                
                for email in valid_emails {
                    let update_request = UserUpdateRequest {
                        btc_address: None,
                        email: Some(email.to_string()),
                        phone: None,
                    };
                    let result = update_user_profile(update_request);
                    assert!(matches!(result, UserResult::Ok(_)), "Email {} should be valid", email);
                }
                
                let invalid_emails = vec![
                    "invalid",
                    "test@",
                    "@domain.com",
                    "test..test@domain.com",
                    "test@domain",
                ];
                
                for email in invalid_emails {
                    let update_request = UserUpdateRequest {
                        btc_address: None,
                        email: Some(email.to_string()),
                        phone: None,
                    };
                    let result = update_user_profile(update_request);
                    assert!(matches!(result, UserResult::Err(_)), "Email {} should be invalid", email);
                }
            }

            #[test]
            fn test_phone_validation_comprehensive() {
                let principal = create_test_principal(92);
                ic_cdk::api::set_caller(principal);
                let _ = register_as_investor();
                
                let valid_phones = vec![
                    "+1234567890",
                    "1234567890",
                    "+44 20 7946 0958",
                    "(555) 123-4567",
                ];
                
                for phone in valid_phones {
                    let update_request = UserUpdateRequest {
                        btc_address: None,
                        email: None,
                        phone: Some(phone.to_string()),
                    };
                    let result = update_user_profile(update_request);
                    assert!(matches!(result, UserResult::Ok(_)), "Phone {} should be valid", phone);
                }
            }

            #[test]
            fn test_get_active_users_filter() {
                let active_principal = create_test_principal(93);
                let inactive_principal = create_test_principal(94);
                
                // Register both users
                ic_cdk::api::set_caller(active_principal);
                let _ = register_as_farmer();
                
                ic_cdk::api::set_caller(inactive_principal);
                let _ = register_as_investor();
                
                // Deactivate one user
                let _ = deactivate_user();
                
                // Get active users
                let active_users = get_active_users();
                
                // Verify active user is in list, inactive is not
                assert!(active_users.iter().any(|u| u.id == active_principal));
                assert!(!active_users.iter().any(|u| u.id == inactive_principal));
            }

            #[test]
            fn test_profile_completion_status() {
                let principal = create_test_principal(95);
                ic_cdk::api::set_caller(principal);
                let _ = register_as_farmer();
                
                // Initially incomplete
                let user = get_user_by_principal(&principal).unwrap();
                assert!(!user.profile_completed);
                
                // Update with partial info
                let partial_update = UserUpdateRequest {
                    btc_address: Some("bc1qar0srrr7xfkvy5l643lydnw9re59gtzzwf5mdq".to_string()),
                    email: None,
                    phone: None,
                };
                let _ = update_user_profile(partial_update);
                
                let user = get_user_by_principal(&principal).unwrap();
                assert!(!user.profile_completed); // Still incomplete
                
                // Update with all info
                let complete_update = UserUpdateRequest {
                    btc_address: None, // Keep existing
                    email: Some("test@example.com".to_string()),
                    phone: Some("+1234567890".to_string()),
                };
                let _ = update_user_profile(complete_update);
                
                let user = get_user_by_principal(&principal).unwrap();
                assert!(user.profile_completed); // Now complete
            }
}
*/

#[cfg(test)]
mod validation_tests {
    use crate::user_management::{validate_btc_address, validate_email, validate_phone};
    
    #[test]
    fn test_btc_address_validation() {
        // Test valid BTC addresses
        assert!(validate_btc_address("1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa")); // Legacy format
        assert!(validate_btc_address("3J98t1WpEZ73CNmQviecrnyiWrnqRhWNLy")); // Script hash
        assert!(validate_btc_address("bc1qar0srrr7xfkvy5l643lydnw9re59gtzzwf5mdq")); // Bech32
        
        // Test invalid BTC addresses
        assert!(!validate_btc_address("invalid")); // Too short
        assert!(!validate_btc_address("2A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa")); // Wrong prefix
        assert!(!validate_btc_address("")); // Empty
    }
    
    #[test]
    fn test_email_validation() {
        // Test valid emails
        assert!(validate_email("test@example.com"));
        assert!(validate_email("user.name@domain.org"));
        assert!(validate_email("a@b.co"));
        
        // Test invalid emails
        assert!(!validate_email("invalid")); // No @
        assert!(!validate_email("test@")); // No domain
        assert!(!validate_email("")); // Empty
        
        // Add more specific invalid cases
        assert!(!validate_email("test@domain")); // No TLD
        assert!(!validate_email("test.domain.com")); // No @
    }
    
    #[test]
    fn test_phone_validation() {
        // Test valid phone numbers
        assert!(validate_phone("1234567890"));
        assert!(validate_phone("+1234567890"));
        assert!(validate_phone("123-456-7890"));
        assert!(validate_phone("123 456 7890"));
        
        // Test invalid phone numbers
        assert!(!validate_phone("123")); // Too short
        assert!(!validate_phone("abc123def")); // Contains letters
        assert!(!validate_phone("")); // Empty
    }
}