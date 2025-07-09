use candid::Principal;
use ic_cdk::{api::caller, id};
use ic_cdk_macros::{query, pre_upgrade, post_upgrade};

mod user_management;
mod rwa_nft;

pub use user_management::*;
pub use rwa_nft::*;

// System functions
#[query]
pub fn get_canister_id() -> Principal {
    id()
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
}

// Generate Candid interface
#[query(name = "__get_candid_interface_tmp_hack")]
fn export_candid() -> String {
    "service : { ... }".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use candid::Principal;
    
    #[test]
    fn test_user_creation() {
        let principal = Principal::from_text("2vxsx-fae").unwrap();
        let user = user_management::create_user(principal, user_management::Role::Farmer);
        assert_eq!(user.id, principal);
        assert_eq!(user.role, user_management::Role::Farmer);
        assert!(user.is_active);
        assert!(!user.profile_completed);
    }
}
