# Panduan Integrasi RWA-NFT dengan Sistem Agrilends

## 1. Integrasi dengan Sistem Pinjaman

### A. Saat Pengajuan Pinjaman
```rust
// Dalam canister loan management
#[update]
pub fn create_loan_application(
    borrower: Principal,
    amount: u64,
    collateral_token_id: u64
) -> LoanResult {
    // 1. Validasi bahwa pemohon adalah pemilik NFT
    let nft_data = call_canister_query(
        NFT_CANISTER_ID,
        "get_rwa_nft_data",
        (collateral_token_id,)
    ).await?;
    
    if nft_data.owner != borrower {
        return LoanResult::Err("You don't own this NFT".to_string());
    }
    
    // 2. Validasi bahwa NFT tersedia sebagai agunan
    if nft_data.is_locked {
        return LoanResult::Err("NFT is already locked as collateral".to_string());
    }
    
    // 3. Dapatkan nilai agunan dari metadata
    let collateral_record = call_canister_query(
        NFT_CANISTER_ID,
        "get_collateral_by_nft_token_id",
        (collateral_token_id,)
    ).await?;
    
    // 4. Validasi LTV (Loan-to-Value ratio)
    let ltv_ratio = (amount as f64) / (collateral_record.valuation_idr as f64);
    if ltv_ratio > MAX_LTV_RATIO {
        return LoanResult::Err("Loan amount exceeds maximum LTV ratio".to_string());
    }
    
    // 5. Buat aplikasi pinjaman
    let loan_id = create_loan_application_internal(borrower, amount, collateral_token_id);
    
    LoanResult::Ok(loan_id)
}
```

### B. Saat Persetujuan Pinjaman
```rust
#[update]
pub fn approve_loan(loan_id: u64) -> LoanResult {
    // 1. Dapatkan data pinjaman
    let loan = get_loan_by_id(loan_id)?;
    
    // 2. Kunci NFT sebagai agunan
    let lock_result = call_canister_update(
        NFT_CANISTER_ID,
        "lock_nft_as_collateral",
        (loan.collateral_token_id, loan_id)
    ).await?;
    
    if lock_result.is_err() {
        return LoanResult::Err("Failed to lock collateral".to_string());
    }
    
    // 3. Update status pinjaman
    update_loan_status(loan_id, LoanStatus::Approved);
    
    LoanResult::Ok(loan_id)
}
```

### C. Saat Pelunasan Pinjaman
```rust
#[update]
pub fn repay_loan(loan_id: u64) -> LoanResult {
    // 1. Validasi pembayaran
    let loan = get_loan_by_id(loan_id)?;
    validate_repayment(loan_id)?;
    
    // 2. Buka kunci NFT
    let unlock_result = call_canister_update(
        NFT_CANISTER_ID,
        "unlock_nft_from_collateral",
        (loan.collateral_token_id,)
    ).await?;
    
    if unlock_result.is_err() {
        return LoanResult::Err("Failed to unlock collateral".to_string());
    }
    
    // 3. Update status pinjaman
    update_loan_status(loan_id, LoanStatus::Repaid);
    
    LoanResult::Ok(loan_id)
}
```

## 2. Integrasi dengan Sistem Likuidasi

### A. Deteksi Gagal Bayar
```rust
#[update]
pub fn check_loan_defaults() -> Vec<u64> {
    let mut defaulted_loans = Vec::new();
    
    for loan in get_active_loans() {
        if is_loan_defaulted(loan.id) {
            // Mulai proses likuidasi
            initiate_liquidation(loan.id);
            defaulted_loans.push(loan.id);
        }
    }
    
    defaulted_loans
}
```

### B. Proses Likuidasi
```rust
#[update]
pub fn liquidate_loan(loan_id: u64) -> LiquidationResult {
    let loan = get_loan_by_id(loan_id)?;
    
    // 1. Tandai agunan sebagai terlikuidasi
    let liquidation_result = call_canister_update(
        NFT_CANISTER_ID,
        "liquidate_collateral",
        (loan.collateral_token_id,)
    ).await?;
    
    // 2. Transfer NFT ke pool likuidasi atau auction
    let transfer_result = call_canister_update(
        NFT_CANISTER_ID,
        "icrc7_transfer",
        (
            Account { owner: loan.borrower, subaccount: None },
            Account { owner: LIQUIDATION_POOL_PRINCIPAL, subaccount: None },
            loan.collateral_token_id
        )
    ).await?;
    
    // 3. Update status pinjaman
    update_loan_status(loan_id, LoanStatus::Liquidated);
    
    LiquidationResult::Ok(loan_id)
}
```

## 3. Integrasi dengan Oracle Harga

### A. Update Valuasi Real-time
```rust
#[update]
pub fn update_collateral_valuation(
    token_id: u64,
    new_valuation: u64,
    price_source: String
) -> Result<(), String> {
    // 1. Validasi caller adalah oracle yang terpercaya
    let caller = msg_caller();
    if !is_trusted_oracle(caller) {
        return Err("Unauthorized oracle".to_string());
    }
    
    // 2. Dapatkan data NFT
    let nft_data = get_nft_by_token_id(token_id)?;
    
    // 3. Update metadata valuasi
    let mut updated_metadata = nft_data.metadata.clone();
    for (key, value) in updated_metadata.iter_mut() {
        if key == "rwa:valuation_idr" {
            *value = MetadataValue::Nat(new_valuation);
        }
    }
    
    // 4. Tambahkan metadata tracking
    updated_metadata.push(("rwa:last_price_update".to_string(), MetadataValue::Nat(time())));
    updated_metadata.push(("rwa:price_source".to_string(), MetadataValue::Text(price_source)));
    
    // 5. Update NFT dan collateral record
    update_nft_metadata(token_id, updated_metadata);
    update_collateral_valuation(token_id, new_valuation);
    
    Ok(())
}
```

## 4. Integrasi dengan Sistem Notifikasi

### A. Event Listener
```rust
#[update]
pub fn handle_nft_events(event: NFTEvent) {
    match event {
        NFTEvent::Minted { token_id, owner } => {
            send_notification(
                owner,
                format!("NFT #{} has been minted successfully", token_id)
            );
        },
        NFTEvent::Locked { token_id, loan_id } => {
            let nft_data = get_nft_by_token_id(token_id).unwrap();
            send_notification(
                nft_data.owner,
                format!("NFT #{} has been locked as collateral for loan #{}", token_id, loan_id)
            );
        },
        NFTEvent::Unlocked { token_id } => {
            let nft_data = get_nft_by_token_id(token_id).unwrap();
            send_notification(
                nft_data.owner,
                format!("NFT #{} has been unlocked and returned to you", token_id)
            );
        },
        NFTEvent::Liquidated { token_id } => {
            let nft_data = get_nft_by_token_id(token_id).unwrap();
            send_notification(
                nft_data.owner,
                format!("NFT #{} has been liquidated due to loan default", token_id)
            );
        },
    }
}
```

## 5. Integrasi dengan Frontend

### A. React Component untuk Minting NFT
```javascript
import React, { useState } from 'react';
import { canisterId, createActor } from '../declarations/agrilends_backend_backend';

const MintNFTForm = ({ userPrincipal }) => {
    const [metadata, setMetadata] = useState({
        legalDocHash: '',
        valuationIdr: '',
        assetDescription: ''
    });
    
    const handleMint = async () => {
        try {
            const actor = createActor(canisterId);
            
            const nftMetadata = [
                ["rwa:legal_doc_hash", { Text: metadata.legalDocHash }],
                ["rwa:valuation_idr", { Nat: BigInt(metadata.valuationIdr) }],
                ["rwa:asset_description", { Text: metadata.assetDescription }],
                ["immutable", { Bool: true }]
            ];
            
            const result = await actor.mint_rwa_nft(userPrincipal, nftMetadata);
            
            if ('Ok' in result) {
                alert(`NFT minted successfully! Token ID: ${result.Ok}`);
            } else {
                alert(`Error: ${result.Err}`);
            }
        } catch (error) {
            console.error('Minting failed:', error);
        }
    };
    
    return (
        <div>
            <h2>Mint RWA NFT</h2>
            <input
                type="text"
                placeholder="Legal Document Hash"
                value={metadata.legalDocHash}
                onChange={(e) => setMetadata({...metadata, legalDocHash: e.target.value})}
            />
            <input
                type="number"
                placeholder="Valuation (IDR)"
                value={metadata.valuationIdr}
                onChange={(e) => setMetadata({...metadata, valuationIdr: e.target.value})}
            />
            <textarea
                placeholder="Asset Description"
                value={metadata.assetDescription}
                onChange={(e) => setMetadata({...metadata, assetDescription: e.target.value})}
            />
            <button onClick={handleMint}>Mint NFT</button>
        </div>
    );
};
```

### B. Dashboard Component
```javascript
const CollateralDashboard = ({ userPrincipal }) => {
    const [userNFTs, setUserNFTs] = useState([]);
    const [availableCollateral, setAvailableCollateral] = useState([]);
    
    useEffect(() => {
        const fetchData = async () => {
            const actor = createActor(canisterId);
            
            const nfts = await actor.get_user_nfts(userPrincipal);
            const collateral = await actor.get_available_collateral(userPrincipal);
            
            setUserNFTs(nfts);
            setAvailableCollateral(collateral);
        };
        
        fetchData();
    }, [userPrincipal]);
    
    return (
        <div>
            <h2>My NFTs</h2>
            {userNFTs.map(nft => (
                <div key={nft.token_id}>
                    <h3>NFT #{nft.token_id}</h3>
                    <p>Status: {nft.is_locked ? 'Locked' : 'Available'}</p>
                    <p>Created: {new Date(nft.created_at / 1000000).toLocaleString()}</p>
                </div>
            ))}
            
            <h2>Available Collateral</h2>
            {availableCollateral.map(collateral => (
                <div key={collateral.collateral_id}>
                    <h3>Collateral #{collateral.collateral_id}</h3>
                    <p>Asset: {collateral.asset_description}</p>
                    <p>Valuation: {collateral.valuation_idr.toLocaleString()} IDR</p>
                    <p>Status: {collateral.status}</p>
                </div>
            ))}
        </div>
    );
};
```

## 6. Best Practices untuk Integrasi

### A. Error Handling
```rust
// Selalu wrap inter-canister calls dalam try-catch
pub async fn safe_inter_canister_call<T>(
    canister_id: Principal,
    method: &str,
    args: candid::CandidType
) -> Result<T, String> {
    match call_canister(canister_id, method, args).await {
        Ok(result) => Ok(result),
        Err(e) => {
            log_error(&format!("Inter-canister call failed: {}", e));
            Err(format!("Service temporarily unavailable: {}", e))
        }
    }
}
```

### B. Data Consistency
```rust
// Gunakan atomic operations untuk consistency
pub fn transfer_with_collateral_update(
    from: Account,
    to: Account,
    token_id: u64
) -> Result<(), String> {
    // Start transaction
    let tx_id = start_transaction();
    
    match icrc7_transfer(from, to, token_id) {
        Ok(_) => {
            // Update collateral record
            match update_collateral_owner(token_id, to.owner) {
                Ok(_) => {
                    commit_transaction(tx_id);
                    Ok(())
                },
                Err(e) => {
                    rollback_transaction(tx_id);
                    Err(e)
                }
            }
        },
        Err(e) => {
            rollback_transaction(tx_id);
            Err(e)
        }
    }
}
```

### C. Monitoring dan Logging
```rust
// Implement comprehensive logging
pub fn log_nft_activity(event: NFTEvent) {
    let log_entry = ActivityLog {
        timestamp: time(),
        event: event.clone(),
        user: msg_caller(),
        metadata: extract_event_metadata(event),
    };
    
    ACTIVITY_LOGS.with(|logs| {
        logs.borrow_mut().push(log_entry);
    });
}
```

## 7. Keamanan dalam Integrasi

### A. Validasi Inter-Canister
```rust
// Selalu validasi canister caller
pub fn validate_canister_caller(expected_canister: Principal) -> Result<(), String> {
    let caller = msg_caller();
    if caller != expected_canister {
        return Err("Unauthorized canister call".to_string());
    }
    Ok(())
}
```

### B. Rate Limiting
```rust
// Implement rate limiting untuk mencegah spam
pub fn check_rate_limit(user: Principal) -> Result<(), String> {
    let current_time = time();
    let user_activity = get_user_activity(user);
    
    if user_activity.last_action + MIN_ACTION_INTERVAL > current_time {
        return Err("Rate limit exceeded".to_string());
    }
    
    Ok(())
}
```

Implementasi lengkap ini memungkinkan sistem RWA-NFT untuk berintegrasi dengan semua komponen lain dalam ekosistem Agrilends secara seamless dan aman.
