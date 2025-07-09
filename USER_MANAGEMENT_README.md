# Agrilends User Management System

## Overview

The Agrilends User Management System is a comprehensive solution for managing user accounts on the Agrilends platform. It provides secure user registration, authentication, profile management, and role-based access control for both farmers and investors.

## Features

### Core Features
- ✅ User registration as Farmer or Investor
- ✅ User authentication via Internet Identity
- ✅ Secure profile management
- ✅ BTC address management for payments
- ✅ User role verification
- ✅ Account activation/deactivation
- ✅ User statistics and analytics
- ✅ Data persistence across canister upgrades

### Enhanced Features
- ✅ Email and phone number management
- ✅ Profile completion tracking
- ✅ Input validation for all fields
- ✅ Role-based user queries
- ✅ Active user filtering
- ✅ Comprehensive error handling

## Data Structures

### User Structure
```rust
pub struct User {
    pub id: Principal,           // Unique user identifier
    pub role: Role,              // User role (Farmer or Investor)
    pub created_at: u64,         // Creation timestamp
    pub btc_address: Option<String>, // Bitcoin address for payments
    pub is_active: bool,         // Account activation status
    pub updated_at: u64,         // Last update timestamp
    pub email: Option<String>,   // Email address
    pub phone: Option<String>,   // Phone number
    pub profile_completed: bool, // Profile completion status
}
```

### Role Enumeration
```rust
pub enum Role {
    Farmer,    // Agricultural producers
    Investor,  // Capital providers
}
```

### User Statistics
```rust
pub struct UserStats {
    pub total_users: u64,
    pub total_farmers: u64,
    pub total_investors: u64,
    pub active_users: u64,
    pub inactive_users: u64,
    pub users_with_btc_address: u64,
    pub completed_profiles: u64,
}
```

## API Reference

### Registration Functions

#### `register_as_farmer() -> UserResult`
- **Type**: Update call
- **Description**: Registers the caller as a farmer
- **Returns**: `UserResult` containing user data or error message
- **Authentication**: Required (Internet Identity)

#### `register_as_investor() -> UserResult`
- **Type**: Update call
- **Description**: Registers the caller as an investor
- **Returns**: `UserResult` containing user data or error message
- **Authentication**: Required (Internet Identity)

### Profile Management Functions

#### `get_user() -> UserResult`
- **Type**: Query call
- **Description**: Retrieves user data for the authenticated caller
- **Returns**: `UserResult` containing user data or error message
- **Authentication**: Required (Internet Identity)

#### `update_btc_address(btc_address: String) -> UserResult`
- **Type**: Update call
- **Description**: Updates the user's Bitcoin address
- **Parameters**: 
  - `btc_address`: Valid Bitcoin address (26-62 characters)
- **Returns**: `UserResult` containing updated user data or error message
- **Authentication**: Required (Internet Identity)

#### `update_user_profile(update_request: UserUpdateRequest) -> UserResult`
- **Type**: Update call
- **Description**: Updates user profile information
- **Parameters**: 
  - `update_request`: UserUpdateRequest containing optional fields
- **Returns**: `UserResult` containing updated user data or error message
- **Authentication**: Required (Internet Identity)

### Account Management Functions

#### `deactivate_user() -> UserResult`
- **Type**: Update call
- **Description**: Deactivates the caller's account
- **Returns**: `UserResult` containing updated user data or error message
- **Authentication**: Required (Internet Identity)

#### `reactivate_user() -> UserResult`
- **Type**: Update call
- **Description**: Reactivates the caller's account
- **Returns**: `UserResult` containing updated user data or error message
- **Authentication**: Required (Internet Identity)

### Query Functions

#### `get_user_by_id(user_id: Principal) -> UserResult`
- **Type**: Query call
- **Description**: Retrieves user data by Principal ID
- **Parameters**: 
  - `user_id`: Principal ID of the user
- **Returns**: `UserResult` containing user data or error message
- **Authentication**: Required (Internet Identity)

#### `get_user_stats() -> UserStats`
- **Type**: Query call
- **Description**: Retrieves platform user statistics
- **Returns**: `UserStats` containing comprehensive statistics
- **Authentication**: Required (Internet Identity)

#### `is_farmer(user_id: Principal) -> bool`
- **Type**: Query call
- **Description**: Checks if a user is a farmer
- **Parameters**: 
  - `user_id`: Principal ID of the user
- **Returns**: `bool` indicating if user is an active farmer
- **Authentication**: Required (Internet Identity)

#### `is_investor(user_id: Principal) -> bool`
- **Type**: Query call
- **Description**: Checks if a user is an investor
- **Parameters**: 
  - `user_id`: Principal ID of the user
- **Returns**: `bool` indicating if user is an active investor
- **Authentication**: Required (Internet Identity)

#### `is_user_active(user_id: Principal) -> bool`
- **Type**: Query call
- **Description**: Checks if a user account is active
- **Parameters**: 
  - `user_id`: Principal ID of the user
- **Returns**: `bool` indicating if user is active
- **Authentication**: Required (Internet Identity)

#### `has_completed_profile(user_id: Principal) -> bool`
- **Type**: Query call
- **Description**: Checks if a user has completed their profile
- **Parameters**: 
  - `user_id`: Principal ID of the user
- **Returns**: `bool` indicating if profile is completed
- **Authentication**: Required (Internet Identity)

#### `get_users_by_role(role: Role) -> Vec<User>`
- **Type**: Query call
- **Description**: Retrieves all users with a specific role
- **Parameters**: 
  - `role`: Role enum (Farmer or Investor)
- **Returns**: `Vec<User>` containing all users with the specified role
- **Authentication**: Required (Internet Identity)

#### `get_active_users() -> Vec<User>`
- **Type**: Query call
- **Description**: Retrieves all active users
- **Returns**: `Vec<User>` containing all active users
- **Authentication**: Required (Internet Identity)

#### `get_all_users() -> Vec<User>`
- **Type**: Query call
- **Description**: Retrieves all users (admin function)
- **Returns**: `Vec<User>` containing all users
- **Authentication**: Required (Internet Identity)
- **Note**: Should be restricted to admin users in production

## Validation Rules

### Bitcoin Address Validation
- Must be 26-62 characters long
- Must start with '1', '3', or 'bc1'
- Basic format validation applied

### Email Validation
- Must contain '@' symbol
- Must be 5-254 characters long
- Basic format validation applied

### Phone Number Validation
- Must be 10-20 characters long
- Must contain only numbers, '+', '-', or spaces
- International format supported

## Error Handling

The system provides comprehensive error handling for all operations:

- **Registration Errors**: Duplicate registration attempts
- **Validation Errors**: Invalid input formats
- **Authentication Errors**: Unauthenticated access attempts
- **Data Errors**: Missing or corrupted user data

## Security Features

### Authentication
- Uses Internet Identity for secure authentication
- Principal-based user identification
- No password storage required

### Data Protection
- Stable storage ensures data persistence
- Automatic data backup during canister upgrades
- Secure memory management

### Access Control
- Role-based access control
- User-specific data access
- Admin function restrictions

## Testing

### Automated Testing
Use the provided testing scripts to verify functionality:

#### PowerShell (Windows)
```powershell
.\testingmanagementuser.ps1
```

#### Bash (Linux/Mac)
```bash
./testingmanagementuser.sh
```

### Test Coverage
The testing suite covers:
- User registration scenarios
- Profile management operations
- Role verification functions
- Error handling cases
- Data validation
- Account management operations

### Manual Testing
You can also test manually using dfx:

```bash
# Register as farmer
dfx identity use your_identity
dfx canister call agrilends_backend_backend register_as_farmer

# Get user data
dfx canister call agrilends_backend_backend get_user

# Update profile
dfx canister call agrilends_backend_backend update_user_profile '(record {
    btc_address = opt "1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa";
    email = opt "user@example.com";
    phone = opt "+1234567890";
})'
```

## Deployment

### Prerequisites
- dfx CLI installed
- Internet Computer replica running
- Rust and Cargo installed

### Build and Deploy
```bash
# Start local replica
dfx start --background

# Deploy canister
dfx deploy

# Test deployment
dfx canister call agrilends_backend_backend health_check
```

## Architecture

### Memory Management
- Uses `StableBTreeMap` for persistent storage
- Memory-efficient data structures
- Automatic garbage collection

### Canister Structure
```
agrilends_backend_backend/
├── src/
│   ├── lib.rs              # Main canister interface
│   ├── user_management.rs  # User management logic
│   └── rwa_nft.rs         # RWA NFT integration
├── Cargo.toml             # Rust dependencies
└── agrilends_backend_backend.did  # Candid interface
```

## Integration

### Frontend Integration
The system provides a clean API for frontend integration:

```javascript
// Example frontend integration
const actor = Actor.createActor(idlFactory, {
  agent,
  canisterId: process.env.CANISTER_ID_AGRILENDS_BACKEND_BACKEND,
});

// Register user
const result = await actor.register_as_farmer();

// Get user data
const user = await actor.get_user();
```

### Other Services
The user management system integrates with:
- RWA NFT system for asset management
- Loan management system for credit operations
- Treasury system for financial operations

## Monitoring and Analytics

### User Statistics
Monitor platform growth and engagement:
- Total user registrations
- Active user metrics
- Role distribution
- Profile completion rates

### Health Monitoring
- Canister health checks
- Memory usage tracking
- Performance metrics

## Future Enhancements

### Planned Features
- Advanced user verification
- Multi-factor authentication
- Enhanced profile fields
- User activity tracking
- Advanced analytics dashboard

### Scalability Improvements
- Data sharding for large user bases
- Caching mechanisms
- Performance optimizations
- Load balancing

## Support

### Common Issues
1. **Registration Fails**: Check Internet Identity connection
2. **Profile Update Errors**: Verify input validation rules
3. **Authentication Issues**: Ensure correct identity is selected

### Troubleshooting
- Check canister logs for detailed error messages
- Verify network connectivity
- Ensure sufficient cycles for operations

## Contributing

### Development Guidelines
- Follow Rust best practices
- Include comprehensive tests
- Document all public functions
- Maintain backward compatibility

### Code Style
- Use descriptive function names
- Include detailed comments
- Follow project structure conventions
- Implement proper error handling

## License

This project is part of the Agrilends platform and follows the project's licensing terms.
