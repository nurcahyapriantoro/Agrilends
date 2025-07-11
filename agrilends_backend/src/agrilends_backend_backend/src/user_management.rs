use candid::{CandidType, Deserialize, Principal};
use ic_cdk::api::time;
use ic_cdk::api::caller as msg_caller; // Perbaikan: gunakan caller sebagai msg_caller
use ic_cdk_macros::{query, update};
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{DefaultMemoryImpl, StableBTreeMap, Storable};
use std::cell::RefCell;
use std::borrow::Cow;

// Types and Memory Management
type Memory = VirtualMemory<DefaultMemoryImpl>;
type UserStorage = StableBTreeMap<Principal, User, Memory>;

// Define user roles
#[derive(CandidType, Deserialize, Clone, Debug, PartialEq)]
pub enum Role {
    Farmer,
    Investor,
}

// Enhanced user data structure
#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct User {
    pub id: Principal,
    pub role: Role,
    pub created_at: u64,
    pub btc_address: Option<String>,
    pub is_active: bool,
    pub updated_at: u64,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub profile_completed: bool,
}

// Enhanced result type for API responses
#[derive(CandidType, Deserialize)]
pub enum UserResult {
    Ok(User),
    Err(String),
}

// Boolean result type
#[derive(CandidType, Deserialize)]
pub enum BoolResult {
    Ok(bool),
    Err(String),
}

// Statistics for admin purposes
#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct UserStats {
    pub total_users: u64,
    pub total_farmers: u64,
    pub total_investors: u64,
    pub active_users: u64,
    pub inactive_users: u64,
    pub users_with_btc_address: u64,
    pub completed_profiles: u64,
}

// User update request
#[derive(CandidType, Deserialize, Clone)]
pub struct UserUpdateRequest {
    pub btc_address: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
}

// Implement Storable trait for User
impl Storable for User {
    const BOUND: ic_stable_structures::storable::Bound = ic_stable_structures::storable::Bound::Unbounded;
    
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(candid::encode_one(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        candid::decode_one(&bytes).unwrap()
    }
}

// Memory Management
thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(
        MemoryManager::init(DefaultMemoryImpl::default())
    );
}

// Storage for users
thread_local! {
    pub static USERS: RefCell<UserStorage> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0)))
        )
    );
}

// Helper function to create a new user
pub fn create_user(principal: Principal, role: Role) -> User {
    let current_time = time();
    User {
        id: principal,
        role,
        created_at: current_time,
        btc_address: None,
        is_active: true,
        updated_at: current_time,
        email: None,
        phone: None,
        profile_completed: false,
    }
}

// Helper function to check if user exists
pub fn user_exists(principal: &Principal) -> bool {
    USERS.with(|users| users.borrow().contains_key(principal))
}

// Helper function to get user by principal
pub fn get_user_by_principal(principal: &Principal) -> Option<User> {
    USERS.with(|users| users.borrow().get(principal))
}

// USER MANAGEMENT FUNCTIONS

/// Register caller as a farmer
#[update]
pub fn register_as_farmer() -> UserResult {
    let principal = msg_caller();
    
    // Check if user is already registered
    if user_exists(&principal) {
        return UserResult::Err("User already registered".to_string());
    }
    
    // Create new farmer user
    let new_user = create_user(principal, Role::Farmer);
    
    // Store user in stable storage
    USERS.with(|users| {
        users.borrow_mut().insert(principal, new_user.clone());
    });
    
    UserResult::Ok(new_user)
}

/// Register caller as an investor
#[update]
pub fn register_as_investor() -> UserResult {
    let principal = msg_caller();
    
    // Check if user is already registered
    if user_exists(&principal) {
        return UserResult::Err("User already registered".to_string());
    }
    
    // Create new investor user
    let new_user = create_user(principal, Role::Investor);
    
    // Store user in stable storage
    USERS.with(|users| {
        users.borrow_mut().insert(principal, new_user.clone());
    });
    
    UserResult::Ok(new_user)
}

/// Get user data for the caller
#[query]
pub fn get_user() -> UserResult {
    let principal = msg_caller();
    
    match get_user_by_principal(&principal) {
        Some(user) => UserResult::Ok(user),
        None => UserResult::Err("User not found. Please register first.".to_string()),
    }
}

/// Update user's BTC address
#[update]
pub fn update_btc_address(btc_address: String) -> UserResult {
    let principal = msg_caller();
    
    match get_user_by_principal(&principal) {
        Some(mut user) => {
            // Basic validation for BTC address (simple check)
            if btc_address.len() < 26 || btc_address.len() > 62 {
                return UserResult::Err("Invalid BTC address format".to_string());
            }
            
            user.btc_address = Some(btc_address);
            user.updated_at = time();
            
            // Check if profile is completed
            user.profile_completed = user.btc_address.is_some() 
                || user.email.is_some() 
                || user.phone.is_some();
            
            // Update user in storage
            USERS.with(|users| {
                users.borrow_mut().insert(principal, user.clone());
            });
            
            UserResult::Ok(user)
        }
        None => UserResult::Err("User not found. Please register first.".to_string()),
    }
}

/// Get user by principal (for admin/internal use)
#[query]
pub fn get_user_by_id(user_id: Principal) -> UserResult {
    let _caller_principal = msg_caller();
    
    // For now, allow any authenticated user to query others
    // In production, you might want to restrict this to admins only
    match get_user_by_principal(&user_id) {
        Some(user) => UserResult::Ok(user),
        None => UserResult::Err("User not found".to_string()),
    }
}

/// Get user statistics
#[query]
pub fn get_user_stats() -> UserStats {
    USERS.with(|users| {
        let users_ref = users.borrow();
        let mut total_farmers = 0u64;
        let mut total_investors = 0u64;
        let mut active_users = 0u64;
        let mut inactive_users = 0u64;
        let mut users_with_btc_address = 0u64;
        let mut completed_profiles = 0u64;
        
        for (_, user) in users_ref.iter() {
            if user.is_active {
                active_users += 1;
            } else {
                inactive_users += 1;
            }
            
            if user.btc_address.is_some() {
                users_with_btc_address += 1;
            }
            
            if user.profile_completed {
                completed_profiles += 1;
            }
            
            match user.role {
                Role::Farmer => total_farmers += 1,
                Role::Investor => total_investors += 1,
            }
        }
        
        UserStats {
            total_users: users_ref.len() as u64,
            total_farmers,
            total_investors,
            active_users,
            inactive_users,
            users_with_btc_address,
            completed_profiles,
        }
    })
}

/// Check if user is a farmer
#[query]
pub fn is_farmer(user_id: Principal) -> bool {
    match get_user_by_principal(&user_id) {
        Some(user) => user.role == Role::Farmer && user.is_active,
        None => false,
    }
}

/// Check if user is an investor
#[query]
pub fn is_investor(user_id: Principal) -> bool {
    match get_user_by_principal(&user_id) {
        Some(user) => user.role == Role::Investor && user.is_active,
        None => false,
    }
}

/// Check if user is active
#[query]
pub fn is_user_active(user_id: Principal) -> bool {
    match get_user_by_principal(&user_id) {
        Some(user) => user.is_active,
        None => false,
    }
}

/// Get all users (admin function - in production should be restricted)
#[query]
pub fn get_all_users() -> Vec<User> {
    USERS.with(|users| {
        users.borrow().iter().map(|(_, user)| user).collect()
    })
}

/// Update user profile
#[update]
pub fn update_user_profile(update_request: UserUpdateRequest) -> UserResult {
    let principal = msg_caller();
    
    match get_user_by_principal(&principal) {
        Some(mut user) => {
            let mut updated = false;
            
            // Update BTC address if provided
            if let Some(btc_address) = update_request.btc_address {
                if !btc_address.is_empty() {
                    if !validate_btc_address(&btc_address) {
                        return UserResult::Err("Invalid BTC address format".to_string());
                    }
                    user.btc_address = Some(btc_address);
                    updated = true;
                }
            }
            
            // Update email if provided
            if let Some(email) = update_request.email {
                if !email.is_empty() {
                    if !validate_email(&email) {
                        return UserResult::Err("Invalid email format".to_string());
                    }
                    user.email = Some(email);
                    updated = true;
                }
            }
            
            // Update phone if provided
            if let Some(phone) = update_request.phone {
                if !phone.is_empty() {
                    if !validate_phone(&phone) {
                        return UserResult::Err("Invalid phone number format".to_string());
                    }
                    user.phone = Some(phone);
                    updated = true;
                }
            }
            
            if updated {
                user.updated_at = time();
                
                // Check if profile is completed
                user.profile_completed = user.btc_address.is_some() 
                    || user.email.is_some() 
                    || user.phone.is_some();
                
                // Update user in storage
                USERS.with(|users| {
                    users.borrow_mut().insert(principal, user.clone());
                });
                
                UserResult::Ok(user)
            } else {
                UserResult::Err("No valid update data provided".to_string())
            }
        }
        None => UserResult::Err("User not found. Please register first.".to_string()),
    }
}

/// Deactivate user account
#[update]
pub fn deactivate_user() -> UserResult {
    let principal = msg_caller();
    
    match get_user_by_principal(&principal) {
        Some(mut user) => {
            user.is_active = false;
            user.updated_at = time();
            
            // Update user in storage
            USERS.with(|users| {
                users.borrow_mut().insert(principal, user.clone());
            });
            
            UserResult::Ok(user)
        }
        None => UserResult::Err("User not found. Please register first.".to_string()),
    }
}

/// Reactivate user account
#[update]
pub fn reactivate_user() -> UserResult {
    let principal = msg_caller();
    
    match get_user_by_principal(&principal) {
        Some(mut user) => {
            user.is_active = true;
            user.updated_at = time();
            
            // Update user in storage
            USERS.with(|users| {
                users.borrow_mut().insert(principal, user.clone());
            });
            
            UserResult::Ok(user)
        }
        None => UserResult::Err("User not found. Please register first.".to_string()),
    }
}

/// Check if user has completed profile
#[query]
pub fn has_completed_profile(user_id: Principal) -> bool {
    match get_user_by_principal(&user_id) {
        Some(user) => user.profile_completed,
        None => false,
    }
}

/// Get users by role
#[query]
pub fn get_users_by_role(role: Role) -> Vec<User> {
    USERS.with(|users| {
        users.borrow()
            .iter()
            .filter(|(_, user)| user.role == role)
            .map(|(_, user)| user)
            .collect()
    })
}

/// Get active users only
#[query]
pub fn get_active_users() -> Vec<User> {
    USERS.with(|users| {
        users.borrow()
            .iter()
            .filter(|(_, user)| user.is_active)
            .map(|(_, user)| user)
            .collect()
    })
}

/// Validate email format
pub fn validate_email(email: &str) -> bool {
    if email.is_empty() {
        return false;
    }
    
    // Basic email validation
    let parts: Vec<&str> = email.split('@').collect();
    if parts.len() != 2 {
        return false;
    }
    
    let local_part = parts[0];
    let domain_part = parts[1];
    
    // Check if local part is not empty
    if local_part.is_empty() {
        return false;
    }
    
    // Check if domain part is not empty and contains at least one dot
    if domain_part.is_empty() || !domain_part.contains('.') {
        return false;
    }
    
    // Check if domain has at least one character before and after the last dot
    let domain_parts: Vec<&str> = domain_part.split('.').collect();
    if domain_parts.len() < 2 {
        return false;
    }
    
    // Check if all domain parts are not empty
    for part in domain_parts {
        if part.is_empty() {
            return false;
        }
    }
    
    true
}

/// Validate phone number format
pub fn validate_phone(phone: &str) -> bool {
    if phone.is_empty() {
        return false;
    }
    
    // Remove common phone number separators
    let cleaned = phone.replace(&['-', ' ', '(', ')'][..], "");
    
    // Check if it starts with + (optional)
    let number_part = if cleaned.starts_with('+') {
        &cleaned[1..]
    } else {
        &cleaned
    };
    
    // Check if all remaining characters are digits
    if !number_part.chars().all(|c| c.is_ascii_digit()) {
        return false;
    }
    
    // Check minimum length (should be at least 10 digits)
    number_part.len() >= 10
}

/// Validate BTC address format
pub fn validate_btc_address(address: &str) -> bool {
    if address.is_empty() {
        return false;
    }
    
    // Basic BTC address validation
    // Legacy addresses (P2PKH): start with 1, length 26-35
    if address.starts_with('1') && address.len() >= 26 && address.len() <= 35 {
        return address.chars().all(|c| c.is_ascii_alphanumeric() && c != '0' && c != 'O' && c != 'I' && c != 'l');
    }
    
    // P2SH addresses: start with 3, length 26-35
    if address.starts_with('3') && address.len() >= 26 && address.len() <= 35 {
        return address.chars().all(|c| c.is_ascii_alphanumeric() && c != '0' && c != 'O' && c != 'I' && c != 'l');
    }
    
    // Bech32 addresses: start with bc1, length 42-62
    if address.starts_with("bc1") && address.len() >= 42 && address.len() <= 62 {
        return address.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit());
    }
    
    false
}
