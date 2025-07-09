pub mod rwa_nft_tests;

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
        assert_eq!(stats.total_investors, 40);
        assert_eq!(stats.active_users, 95);
        assert_eq!(stats.inactive_users, 5);
        assert_eq!(stats.users_with_btc_address, 30);
        assert_eq!(stats.completed_profiles, 25);
    }
    
    #[test]
    fn test_user_update_request_structure() {
        let update_request = UserUpdateRequest {
            btc_address: Some("bc1qar0srrr7xfkvy5l643lydnw9re59gtzzwf5mdq".to_string()),
            email: Some("test@example.com".to_string()),
            phone: Some("+1234567890".to_string()),
        };
        
        assert!(update_request.btc_address.is_some());
        assert!(update_request.email.is_some());
        assert!(update_request.phone.is_some());
    }
    
    #[test]
    fn test_btc_address_validation() {
        // Test valid BTC addresses
        assert!(validate_btc_address("1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa")); // Legacy
        assert!(validate_btc_address("3J98t1WpEZ73CNmQviecrnyiWrnqRhWNLy")); // P2SH
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