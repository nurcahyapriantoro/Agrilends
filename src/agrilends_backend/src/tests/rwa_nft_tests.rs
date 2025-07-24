#[cfg(test)]
mod rwa_nft_tests {
    use candid::Principal;
    use crate::types::*;
    use crate::helpers::*;

    // Helper function to create test principal
    fn create_test_principal(id: u8) -> Principal {
        Principal::from_slice(&[id; 29])
    }

    // Helper function to create mock time (since ic_cdk::api::time() doesn't work in tests)
    fn mock_time() -> u64 {
        1234567890_u64
    }

    // Helper function to create valid NFT metadata
    fn create_valid_metadata() -> Vec<(String, MetadataValue)> {
        vec![
            ("rwa:legal_doc_hash".to_string(), MetadataValue::Text("a".repeat(64))),
            ("rwa:valuation_idr".to_string(), MetadataValue::Nat(300_000_000)),
            ("rwa:asset_description".to_string(), MetadataValue::Text("Gabah, 20 Ton, Kualitas A".to_string())),
            ("immutable".to_string(), MetadataValue::Bool(true)),
        ]
    }

    // Helper function to create invalid metadata (missing required fields)
    fn create_invalid_metadata() -> Vec<(String, MetadataValue)> {
        vec![
            ("rwa:valuation_idr".to_string(), MetadataValue::Nat(300_000_000)),
            ("custom_field".to_string(), MetadataValue::Text("custom value".to_string())),
        ]
    }

    #[test]
    fn test_validate_nft_metadata_valid() {
        let metadata = create_valid_metadata();
        assert!(validate_nft_metadata(&metadata).is_ok());
    }

    #[test]
    fn test_validate_nft_metadata_missing_legal_doc() {
        let metadata = vec![
            ("rwa:valuation_idr".to_string(), MetadataValue::Nat(300_000_000)),
            ("rwa:asset_description".to_string(), MetadataValue::Text("Gabah, 20 Ton, Kualitas A".to_string())),
        ];
        
        let result = validate_nft_metadata(&metadata);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Missing required metadata: rwa:legal_doc_hash"));
    }

    #[test]
    fn test_validate_nft_metadata_missing_valuation() {
        let metadata = vec![
            ("rwa:legal_doc_hash".to_string(), MetadataValue::Text("a".repeat(64))),
            ("rwa:asset_description".to_string(), MetadataValue::Text("Gabah, 20 Ton, Kualitas A".to_string())),
        ];
        
        let result = validate_nft_metadata(&metadata);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Missing required metadata: rwa:valuation_idr"));
    }

    #[test]
    fn test_validate_nft_metadata_missing_description() {
        let metadata = vec![
            ("rwa:legal_doc_hash".to_string(), MetadataValue::Text("a".repeat(64))),
            ("rwa:valuation_idr".to_string(), MetadataValue::Nat(300_000_000)),
        ];
        
        let result = validate_nft_metadata(&metadata);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Missing required metadata: rwa:asset_description"));
    }

    #[test]
    fn test_validate_nft_metadata_invalid_hash_length() {
        let metadata = vec![
            ("rwa:legal_doc_hash".to_string(), MetadataValue::Text("invalid_hash".to_string())),
            ("rwa:valuation_idr".to_string(), MetadataValue::Nat(300_000_000)),
            ("rwa:asset_description".to_string(), MetadataValue::Text("Gabah, 20 Ton, Kualitas A".to_string())),
        ];
        
        let result = validate_nft_metadata(&metadata);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid legal document hash format"));
    }

    #[test]
    fn test_validate_nft_metadata_zero_valuation() {
        let metadata = vec![
            ("rwa:legal_doc_hash".to_string(), MetadataValue::Text("a".repeat(64))),
            ("rwa:valuation_idr".to_string(), MetadataValue::Nat(0)),
            ("rwa:asset_description".to_string(), MetadataValue::Text("Gabah, 20 Ton, Kualitas A".to_string())),
        ];
        
        let result = validate_nft_metadata(&metadata);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Valuation must be greater than 0"));
    }

    #[test]
    fn test_validate_nft_metadata_empty_description() {
        let metadata = vec![
            ("rwa:legal_doc_hash".to_string(), MetadataValue::Text("a".repeat(64))),
            ("rwa:valuation_idr".to_string(), MetadataValue::Nat(300_000_000)),
            ("rwa:asset_description".to_string(), MetadataValue::Text("   ".to_string())),
        ];
        
        let result = validate_nft_metadata(&metadata);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Asset description cannot be empty"));
    }

    #[test]
    fn test_extract_metadata_values() {
        let metadata = create_valid_metadata();
        let (legal_doc_hash, valuation_idr, asset_description) = extract_metadata_values(&metadata);
        
        assert_eq!(legal_doc_hash, "a".repeat(64));
        assert_eq!(valuation_idr, 300_000_000);
        assert_eq!(asset_description, "Gabah, 20 Ton, Kualitas A");
    }

    #[test]
    fn test_extract_metadata_values_partial() {
        let metadata = vec![
            ("rwa:valuation_idr".to_string(), MetadataValue::Nat(500_000_000)),
            ("custom_field".to_string(), MetadataValue::Text("custom value".to_string())),
        ];
        
        let (legal_doc_hash, valuation_idr, asset_description) = extract_metadata_values(&metadata);
        
        assert_eq!(legal_doc_hash, "");
        assert_eq!(valuation_idr, 500_000_000);
        assert_eq!(asset_description, "");
    }

    #[test]
    fn test_validate_sha256_hash_valid() {
        let valid_hash = "a".repeat(64);
        assert!(validate_sha256_hash(&valid_hash));
    }

    #[test]
    fn test_validate_sha256_hash_invalid_length() {
        let invalid_hash = "a".repeat(63);
        assert!(!validate_sha256_hash(&invalid_hash));
    }

    #[test]
    fn test_validate_sha256_hash_invalid_characters() {
        let invalid_hash = "g".repeat(64);
        assert!(!validate_sha256_hash(&invalid_hash));
    }

    #[test]
    fn test_is_authorized_to_mint_non_farmer() {
        let non_farmer_principal = create_test_principal(2);
        
        // Unregistered user should not be authorized
        assert!(!is_authorized_to_mint(&non_farmer_principal));
    }

    #[test]
    fn test_is_loan_manager_canister() {
        // Test with mock principals - adjust expectations based on actual implementation
        let admin_principal = create_test_principal(1);
        let regular_principal = create_test_principal(3);
        
        // Since we're using test principals, both should return false
        // unless specifically configured as loan manager
        assert!(!is_loan_manager_canister(&admin_principal));
        assert!(!is_loan_manager_canister(&regular_principal));
        
        // Test with actual loan manager principal if available
        if let Ok(loan_manager) = Principal::from_text("rdmx6-jaaaa-aaaah-qcaiq-cai") {
            // This would be true if the principal is configured as loan manager
            // For now, we'll just test that the function doesn't panic
            let _result = is_loan_manager_canister(&loan_manager);
        }
    }

    #[test]
    fn test_mint_nft_invalid_metadata() {
        let metadata = create_invalid_metadata();
        
        // Test invalid metadata
        assert!(validate_nft_metadata(&metadata).is_err());
    }

    #[test]
    fn test_transfer_request_validation() {
        let owner = create_test_principal(1);
        let recipient = create_test_principal(2);
        let mock_time = mock_time();
        
        let transfer_request = TransferRequest {
            from: Some(Account {
                owner,
                subaccount: None,
            }),
            to: Account {
                owner: recipient,
                subaccount: None,
            },
            token_id: 1,
            memo: None,
            created_at_time: Some(mock_time),
        };
        
        // Test transfer request structure
        assert_eq!(transfer_request.from.unwrap().owner, owner);
        assert_eq!(transfer_request.to.owner, recipient);
        assert_eq!(transfer_request.token_id, 1);
        assert_eq!(transfer_request.created_at_time, Some(mock_time));
    }

    #[test]
    fn test_collateral_status_transitions() {
        // Test valid status transitions
        let available = CollateralStatus::Available;
        let locked = CollateralStatus::Locked;
        let released = CollateralStatus::Released;
        let liquidated = CollateralStatus::Liquidated;
        
        assert_eq!(available, CollateralStatus::Available);
        assert_eq!(locked, CollateralStatus::Locked);
        assert_eq!(released, CollateralStatus::Released);
        assert_eq!(liquidated, CollateralStatus::Liquidated);
        
        // Test inequality
        assert_ne!(available, locked);
        assert_ne!(locked, released);
        assert_ne!(released, liquidated);
    }

    #[test]
    fn test_account_structure() {
        let owner = create_test_principal(1);
        let account = Account {
            owner,
            subaccount: None,
        };
        
        assert_eq!(account.owner, owner);
        assert_eq!(account.subaccount, None);
        
        // Test with subaccount
        let subaccount = vec![1, 2, 3, 4];
        let account_with_sub = Account {
            owner,
            subaccount: Some(subaccount.clone()),
        };
        
        assert_eq!(account_with_sub.owner, owner);
        assert_eq!(account_with_sub.subaccount, Some(subaccount));
    }

    #[test]
    fn test_metadata_value_types() {
        let text_value = MetadataValue::Text("test".to_string());
        let nat_value = MetadataValue::Nat(123);
        let bool_value = MetadataValue::Bool(true);
        let principal_value = MetadataValue::Principal(create_test_principal(1));
        let blob_value = MetadataValue::Blob(vec![1, 2, 3]);
        let int_value = MetadataValue::Int(-456);
        
        // Test pattern matching
        match text_value {
            MetadataValue::Text(s) => assert_eq!(s, "test"),
            _ => panic!("Expected Text value"),
        }
        
        match nat_value {
            MetadataValue::Nat(n) => assert_eq!(n, 123),
            _ => panic!("Expected Nat value"),
        }
        
        match bool_value {
            MetadataValue::Bool(b) => assert!(b),
            _ => panic!("Expected Bool value"),
        }
        
        match principal_value {
            MetadataValue::Principal(p) => assert_eq!(p, create_test_principal(1)),
            _ => panic!("Expected Principal value"),
        }
        
        match blob_value {
            MetadataValue::Blob(b) => assert_eq!(b, vec![1, 2, 3]),
            _ => panic!("Expected Blob value"),
        }
        
        match int_value {
            MetadataValue::Int(i) => assert_eq!(i, -456),
            _ => panic!("Expected Int value"),
        }
    }

    #[test]
    fn test_nft_stats_structure() {
        let stats = NFTStats {
            total_nfts: 100,
            locked_nfts: 25,
            available_collateral: 75,
            liquidated_collateral: 5,
        };
        
        assert_eq!(stats.total_nfts, 100);
        assert_eq!(stats.locked_nfts, 25);
        assert_eq!(stats.available_collateral, 75);
        assert_eq!(stats.liquidated_collateral, 5);
    }

    #[test]
    fn test_collateral_record_structure() {
        let owner = create_test_principal(1);
        let mock_time = mock_time();
        
        let collateral_record = CollateralRecord {
            collateral_id: 1,
            nft_token_id: 1,
            owner,
            loan_id: Some(123),
            valuation_idr: 300_000_000,
            asset_description: "Gabah, 20 Ton, Kualitas A".to_string(),
            legal_doc_hash: "a".repeat(64),
            status: CollateralStatus::Locked,
            created_at: mock_time,
            updated_at: mock_time,
        };
        
        assert_eq!(collateral_record.collateral_id, 1);
        assert_eq!(collateral_record.nft_token_id, 1);
        assert_eq!(collateral_record.owner, owner);
        assert_eq!(collateral_record.loan_id, Some(123));
        assert_eq!(collateral_record.valuation_idr, 300_000_000);
        assert_eq!(collateral_record.asset_description, "Gabah, 20 Ton, Kualitas A");
        assert_eq!(collateral_record.legal_doc_hash, "a".repeat(64));
        assert_eq!(collateral_record.status, CollateralStatus::Locked);
        assert_eq!(collateral_record.created_at, mock_time);
        assert_eq!(collateral_record.updated_at, mock_time);
    }

    #[test]
    fn test_rwa_nft_data_structure() {
        let owner = create_test_principal(1);
        let mock_time = mock_time();
        let metadata = create_valid_metadata();
        
        let nft_data = RWANFTData {
            token_id: 1,
            owner,
            metadata: metadata.clone(),
            created_at: mock_time,
            updated_at: mock_time,
            is_locked: false,
            loan_id: None,
        };
        
        assert_eq!(nft_data.token_id, 1);
        assert_eq!(nft_data.owner, owner);
        assert_eq!(nft_data.metadata, metadata);
        assert_eq!(nft_data.created_at, mock_time);
        assert_eq!(nft_data.updated_at, mock_time);
        assert!(!nft_data.is_locked);
        assert_eq!(nft_data.loan_id, None);
    }

    // Edge case tests
    #[test]
    fn test_edge_case_empty_metadata() {
        let metadata = vec![];
        let result = validate_nft_metadata(&metadata);
        assert!(result.is_err());
    }

    #[test]
    fn test_edge_case_large_valuation() {
        let metadata = vec![
            ("rwa:legal_doc_hash".to_string(), MetadataValue::Text("a".repeat(64))),
            ("rwa:valuation_idr".to_string(), MetadataValue::Nat(u64::MAX)),
            ("rwa:asset_description".to_string(), MetadataValue::Text("Large valuation test".to_string())),
        ];
        
        let result = validate_nft_metadata(&metadata);
        assert!(result.is_ok());
        
        let (_, valuation_idr, _) = extract_metadata_values(&metadata);
        assert_eq!(valuation_idr, u64::MAX);
    }

    #[test]
    fn test_edge_case_very_long_description() {
        let long_description = "A".repeat(1000);
        let metadata = vec![
            ("rwa:legal_doc_hash".to_string(), MetadataValue::Text("a".repeat(64))),
            ("rwa:valuation_idr".to_string(), MetadataValue::Nat(300_000_000)),
            ("rwa:asset_description".to_string(), MetadataValue::Text(long_description.clone())),
        ];
        
        let result = validate_nft_metadata(&metadata);
        assert!(result.is_ok());
        
        let (_, _, asset_description) = extract_metadata_values(&metadata);
        assert_eq!(asset_description, long_description);
    }

    #[test]
    fn test_multiple_metadata_entries_same_key() {
        // Test that the last entry wins when duplicate keys exist
        let metadata = vec![
            ("rwa:legal_doc_hash".to_string(), MetadataValue::Text("a".repeat(64))),
            ("rwa:valuation_idr".to_string(), MetadataValue::Nat(300_000_000)),
            ("rwa:valuation_idr".to_string(), MetadataValue::Nat(500_000_000)), // Duplicate key
            ("rwa:asset_description".to_string(), MetadataValue::Text("Test description".to_string())),
        ];
        
        let (_, valuation_idr, _) = extract_metadata_values(&metadata);
        assert_eq!(valuation_idr, 500_000_000); // Should use the last value
    }
}

// Performance tests
#[cfg(test)]
mod performance_tests {
    use crate::types::*;
    use crate::helpers::*;
    
    #[test]
    fn test_metadata_validation_performance() {
        let metadata = vec![
            ("rwa:legal_doc_hash".to_string(), MetadataValue::Text("a".repeat(64))),
            ("rwa:valuation_idr".to_string(), MetadataValue::Nat(300_000_000)),
            ("rwa:asset_description".to_string(), MetadataValue::Text("Test".to_string())),
        ];
        
        // Run validation multiple times to test performance
        for _ in 0..1000 {
            let result = validate_nft_metadata(&metadata);
            assert!(result.is_ok());
        }
    }
    
    #[test]
    fn test_metadata_extraction_performance() {
        let metadata = vec![
            ("rwa:legal_doc_hash".to_string(), MetadataValue::Text("a".repeat(64))),
            ("rwa:valuation_idr".to_string(), MetadataValue::Nat(300_000_000)),
            ("rwa:asset_description".to_string(), MetadataValue::Text("Test".to_string())),
        ];
        
        // Run extraction multiple times to test performance
        for _ in 0..1000 {
            let (legal_doc_hash, valuation_idr, asset_description) = extract_metadata_values(&metadata);
            assert!(!legal_doc_hash.is_empty());
            assert!(valuation_idr > 0);
            assert!(!asset_description.is_empty());
        }
    }
}