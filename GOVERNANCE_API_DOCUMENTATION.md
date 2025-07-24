# 🏛️ Governance & Administration API Documentation

## Overview

Comprehensive API documentation for the Agrilends Governance & Administration system. This module provides complete protocol management capabilities including parameter updates, admin role management, DAO-style governance, and emergency controls.

## 🔧 Core Protocol Parameter Management

### `set_protocol_parameter(key: String, value: u64) -> Result<String, String>`

**Type**: `update`  
**Security**: Admin only  
**Description**: Update a single protocol parameter with validation

**Parameters**:
- `key`: Parameter identifier
- `value`: New parameter value

**Supported Parameters**:
```rust
// Loan Parameters
"loan_to_value_ratio"     // LTV ratio in basis points (default: 6000 = 60%)
"base_apr"                // Base annual percentage rate (default: 1000 = 10%)
"max_loan_duration_days"  // Maximum loan duration in days

// Liquidation Parameters  
"liquidation_threshold"   // Liquidation threshold ratio (default: 7500 = 75%)
"grace_period_days"       // Grace period before liquidation (default: 30)
"liquidation_penalty"     // Liquidation penalty in basis points (default: 500 = 5%)

// Pool Parameters
"max_utilization_rate"    // Maximum pool utilization (default: 8000 = 80%)
"emergency_reserve_ratio" // Emergency reserve ratio (default: 1000 = 10%)
"protocol_fee_rate"       // Protocol fee rate (default: 200 = 2%)

// System Parameters
"emergency_stop"          // Emergency stop flag (0=disabled, 1=enabled)
"maintenance_mode"        // Maintenance mode flag (0=disabled, 1=enabled)
"min_collateral_value"    // Minimum collateral value in IDR
"max_collateral_value"    // Maximum collateral value in IDR
```

**Example**:
```rust
// Update LTV ratio to 65%
let result = set_protocol_parameter("loan_to_value_ratio".to_string(), 6500);
```

**Validation**:
- Caller must be admin
- Value must be within parameter bounds
- Parameter key must exist

---

### `get_protocol_parameter(key: String) -> Result<ProtocolParameter, String>`

**Type**: `query`  
**Security**: Public  
**Description**: Retrieve a specific protocol parameter

**Returns**: `ProtocolParameter` structure:
```rust
pub struct ProtocolParameter {
    pub key: String,
    pub current_value: u64,
    pub value_type: ParameterType,
    pub min_value: Option<u64>,
    pub max_value: Option<u64>,
    pub description: String,
    pub last_updated: u64,
    pub updated_by: Principal,
}
```

---

### `get_all_protocol_parameters() -> Vec<ProtocolParameter>`

**Type**: `query`  
**Security**: Public  
**Description**: Retrieve all protocol parameters

---

### `set_multiple_protocol_parameters(parameters: Vec<(String, u64)>) -> Vec<Result<String, String>>`

**Type**: `update`  
**Security**: Admin only  
**Description**: Update multiple parameters in a single call

**Example**:
```rust
let parameters = vec![
    ("loan_to_value_ratio".to_string(), 6500),
    ("base_apr".to_string(), 1200),
    ("liquidation_threshold".to_string(), 8000),
];
let results = set_multiple_protocol_parameters(parameters);
```

---

### `get_protocol_parameters_by_category(category: String) -> Vec<ProtocolParameter>`

**Type**: `query`  
**Security**: Public  
**Description**: Get parameters filtered by category

**Categories**:
- `"loan"`: Loan-related parameters
- `"liquidation"`: Liquidation-related parameters  
- `"system"`: System control parameters
- `"pool"`: Liquidity pool parameters

---

### `validate_parameter_value(key: String, value: u64) -> Result<String, String>`

**Type**: `query`  
**Security**: Public  
**Description**: Validate a parameter value before setting

---

## 👥 Admin Role Management

### `initialize_super_admin(admin_principal: Principal) -> Result<String, String>`

**Type**: `update`  
**Security**: One-time setup only  
**Description**: Initialize the first super admin (can only be called once)

---

### `grant_admin_role(principal: Principal, role_type: AdminRoleType, permissions: Vec<Permission>, expires_at: Option<u64>) -> Result<String, String>`

**Type**: `update`  
**Security**: Super admin only  
**Description**: Grant admin role to a principal

**Admin Role Types**:
```rust
pub enum AdminRoleType {
    SuperAdmin,      // Full system control
    ProtocolAdmin,   // Protocol parameter management
    TreasuryAdmin,   // Treasury operations
    RiskAdmin,       // Risk management
    LiquidationAdmin,// Liquidation operations
    OracleAdmin,     // Oracle management
    EmergencyAdmin,  // Emergency controls
}
```

**Permissions**:
```rust
pub enum Permission {
    ManageParameters,  // Update protocol parameters
    ManageAdmins,      // Grant/revoke admin roles
    EmergencyStop,     // Activate emergency stop
    ManageTreasury,    // Treasury operations
    ManageLiquidation, // Liquidation management
    ManageOracle,      // Oracle management
    ViewMetrics,       // View system metrics
    ExecuteProposals,  // Execute governance proposals
}
```

**Example**:
```rust
let result = grant_admin_role(
    principal,
    AdminRoleType::ProtocolAdmin,
    vec![Permission::ManageParameters, Permission::ViewMetrics],
    None // No expiration
);
```

---

### `revoke_admin_role(principal: Principal) -> Result<String, String>`

**Type**: `update`  
**Security**: Super admin only  
**Description**: Revoke admin role from a principal

---

### `transfer_admin_role(new_admin: Principal) -> Result<String, String>`

**Type**: `update`  
**Security**: Super admin only  
**Description**: Transfer super admin role to another principal

---

### `get_admin_role(principal: Principal) -> Option<AdminRole>`

**Type**: `query`  
**Security**: Public  
**Description**: Get admin role information for a principal

---

### `get_all_admin_roles() -> Vec<AdminRole>`

**Type**: `query`  
**Security**: Public  
**Description**: Get all admin roles in the system

---

### `get_active_admin_count() -> u64`

**Type**: `query`  
**Security**: Public  
**Description**: Get count of active admins

---

## 📊 DAO-Style Governance

### `create_proposal(proposal_type: ProposalType, title: String, description: String, execution_payload: Option<Vec<u8>>) -> GovernanceResult<u64>`

**Type**: `update`  
**Security**: Admin or authorized users  
**Description**: Create a new governance proposal

**Proposal Types**:
```rust
pub enum ProposalType {
    ProtocolParameterUpdate, // Update protocol parameters
    AdminRoleUpdate,         // Admin role changes
    CanisterUpgrade,         // Canister upgrades
    EmergencyAction,         // Emergency actions
    SystemConfiguration,    // System configuration
    TreasuryManagement,      // Treasury operations
}
```

**Example**:
```rust
let proposal_id = create_proposal(
    ProposalType::ProtocolParameterUpdate,
    "Increase LTV Ratio".to_string(),
    "Proposal to increase LTV ratio to 65% for better capital efficiency".to_string(),
    Some(b"loan_to_value_ratio:6500".to_vec())
)?;
```

---

### `vote_on_proposal(proposal_id: u64, choice: VoteChoice, reason: Option<String>) -> GovernanceResult<String>`

**Type**: `update`  
**Security**: Voting power required  
**Description**: Cast a vote on an active proposal

**Vote Choices**:
```rust
pub enum VoteChoice {
    Yes,     // Support the proposal
    No,      // Oppose the proposal  
    Abstain, // Abstain from voting
}
```

**Example**:
```rust
let result = vote_on_proposal(
    proposal_id,
    VoteChoice::Yes,
    Some("This change improves capital efficiency".to_string())
)?;
```

---

### `execute_proposal(proposal_id: u64) -> GovernanceResult<String>`

**Type**: `update`  
**Security**: Admin only  
**Description**: Execute an approved proposal

**Execution Conditions**:
- Proposal must be in Active status
- Voting deadline must have passed
- Quorum threshold must be met
- Approval threshold must be met
- Execution deadline must not have passed

---

### `get_proposal(proposal_id: u64) -> Option<Proposal>`

**Type**: `query`  
**Security**: Public  
**Description**: Get proposal details by ID

---

### `get_proposals(offset: u64, limit: u64) -> Vec<Proposal>`

**Type**: `query`  
**Security**: Public  
**Description**: Get proposals with pagination

---

### `get_proposals_by_status(status: ProposalStatus, offset: u64, limit: u64) -> Vec<Proposal>`

**Type**: `query`  
**Security**: Public  
**Description**: Get proposals filtered by status

**Proposal Status**:
```rust
pub enum ProposalStatus {
    Pending,   // Created but not yet active
    Active,    // Currently accepting votes
    Approved,  // Approved but not yet executed
    Rejected,  // Rejected by voters
    Executed,  // Successfully executed
    Expired,   // Expired without execution
}
```

---

### `get_proposal_votes(proposal_id: u64) -> Vec<Vote>`

**Type**: `query`  
**Security**: Public  
**Description**: Get all votes for a specific proposal

---

### `can_execute_proposal(proposal_id: u64) -> Result<bool, String>`

**Type**: `query`  
**Security**: Public  
**Description**: Check if a proposal can be executed

---

## 🚨 Emergency Controls

### `emergency_stop() -> Result<String, String>`

**Type**: `update`  
**Security**: Emergency admin permission required  
**Description**: Activate emergency stop to halt all system operations

**Effects**:
- Sets `emergency_stop` parameter to 1
- Stops all loan operations
- Prevents new deposits/withdrawals
- Blocks parameter updates (except by super admin)

---

### `resume_operations() -> Result<String, String>`

**Type**: `update`  
**Security**: Super admin only  
**Description**: Resume operations after emergency stop

**Effects**:
- Sets `emergency_stop` parameter to 0
- Sets `maintenance_mode` parameter to 0
- Resumes all system operations

---

### `set_maintenance_mode(enabled: bool) -> Result<String, String>`

**Type**: `update`  
**Security**: Admin only  
**Description**: Enable/disable maintenance mode

**Effects**:
- Temporarily pauses non-critical operations
- Allows emergency operations to continue
- Prevents new loan applications

---

## 📊 System Status & Monitoring

### `get_system_status() -> HashMap<String, bool>`

**Type**: `query`  
**Security**: Public  
**Description**: Get current system status flags

**Returns**:
```rust
{
    "emergency_stop": false,
    "maintenance_mode": false,
    // other system flags
}
```

---

### `get_governance_stats() -> GovernanceStats`

**Type**: `query`  
**Security**: Public  
**Description**: Get governance system statistics

**Returns**:
```rust
pub struct GovernanceStats {
    pub total_proposals: u64,
    pub active_proposals: u64,
    pub executed_proposals: u64,
    pub total_votes_cast: u64,
    pub total_voting_power: u64,
    pub average_participation_rate: u64,
    pub last_proposal_id: u64,
}
```

---

### `get_governance_dashboard() -> GovernanceDashboard`

**Type**: `query`  
**Security**: Public  
**Description**: Get comprehensive dashboard data

**Returns**:
```rust
pub struct GovernanceDashboard {
    pub stats: GovernanceStats,
    pub active_proposals: Vec<Proposal>,
    pub recent_proposals: Vec<Proposal>,
    pub admin_count: u64,
    pub system_status: HashMap<String, bool>,
    pub parameter_count: u64,
    pub last_updated: u64,
}
```

---

## ⚙️ Configuration Management

### `update_governance_config(config: GovernanceConfig) -> Result<String, String>`

**Type**: `update`  
**Security**: Super admin only  
**Description**: Update governance system configuration

**Configuration Parameters**:
```rust
pub struct GovernanceConfig {
    pub voting_period_seconds: u64,      // How long proposals are open for voting
    pub execution_delay_seconds: u64,    // Delay between approval and execution
    pub proposal_threshold: u64,         // Minimum voting power to create proposal
    pub quorum_threshold: u64,           // Minimum participation for valid vote
    pub approval_threshold: u64,         // Percentage needed for approval
    pub max_proposals_per_user: u64,     // Max active proposals per user
    pub emergency_action_threshold: u64, // Lower threshold for emergency actions
    pub treasury_action_threshold: u64,  // Higher threshold for treasury actions
}
```

---

### `get_governance_config_public() -> GovernanceConfig`

**Type**: `query`  
**Security**: Public  
**Description**: Get current governance configuration

---

## 🔍 Advanced Queries

### `get_parameter_history(key: String) -> Vec<(u64, u64, Principal)>`

**Type**: `query`  
**Security**: Public  
**Description**: Get parameter change history (timestamp, value, updater)

---

### `create_batch_proposals(proposals: Vec<(ProposalType, String, String, Option<Vec<u8>>)>) -> Vec<GovernanceResult<u64>>`

**Type**: `update`  
**Security**: Admin only  
**Description**: Create multiple proposals in a single call

---

## 🛡️ Security Features

### Access Control
- **Multi-level admin roles** with granular permissions
- **Principal-based authentication** for all operations
- **Role expiration** support for temporary access
- **Permission checking** for all sensitive operations

### Parameter Validation
- **Range validation** for all parameter values
- **Type checking** for parameter types
- **Dependency validation** for related parameters
- **Real-time validation** before application

### Audit Logging
- **Complete action tracking** for all governance operations
- **Principal identification** for all changes
- **Timestamp recording** for all events
- **Error logging** for failed operations

### Emergency Protections
- **Emergency stop** capability for system-wide halt
- **Maintenance mode** for controlled operations
- **Admin override** capabilities for emergency situations
- **Recovery procedures** for system restoration

## 📊 Usage Examples

### Basic Parameter Update
```rust
// Update LTV ratio
let result = set_protocol_parameter("loan_to_value_ratio".to_string(), 6500);
match result {
    Ok(message) => ic_cdk::println!("Success: {}", message),
    Err(error) => ic_cdk::println!("Error: {}", error),
}
```

### Batch Parameter Update
```rust
// Update multiple parameters
let parameters = vec![
    ("loan_to_value_ratio".to_string(), 6500),
    ("base_apr".to_string(), 1200),
    ("liquidation_threshold".to_string(), 8000),
];

let results = set_multiple_protocol_parameters(parameters);
for (i, result) in results.iter().enumerate() {
    match result {
        Ok(msg) => ic_cdk::println!("Parameter {}: {}", i, msg),
        Err(err) => ic_cdk::println!("Parameter {} failed: {}", i, err),
    }
}
```

### Admin Role Management
```rust
// Grant protocol admin role
let result = grant_admin_role(
    principal,
    AdminRoleType::ProtocolAdmin,
    vec![Permission::ManageParameters, Permission::ViewMetrics],
    Some(time() + (365 * 24 * 60 * 60 * 1_000_000_000)) // 1 year expiration
);
```

### Proposal Creation and Voting
```rust
// Create proposal
let proposal_id = create_proposal(
    ProposalType::ProtocolParameterUpdate,
    "Increase LTV Ratio".to_string(),
    "Proposal to increase LTV ratio to 65% for improved capital efficiency".to_string(),
    Some(b"loan_to_value_ratio:6500".to_vec())
)?;

// Vote on proposal
let vote_result = vote_on_proposal(
    proposal_id,
    VoteChoice::Yes,
    Some("This change will improve borrower access to capital".to_string())
)?;
```

### Emergency Operations
```rust
// Emergency stop
let result = emergency_stop();
if result.is_ok() {
    ic_cdk::println!("Emergency stop activated successfully");
}

// Resume operations
let result = resume_operations();
if result.is_ok() {
    ic_cdk::println!("Operations resumed successfully");
}
```

### System Monitoring
```rust
// Get system status
let status = get_system_status();
for (key, value) in status {
    ic_cdk::println!("System {}: {}", key, if value { "ENABLED" } else { "DISABLED" });
}

// Get governance dashboard
let dashboard = get_governance_dashboard();
ic_cdk::println!("Total proposals: {}", dashboard.stats.total_proposals);
ic_cdk::println!("Active admins: {}", dashboard.admin_count);
```

## 🚀 Production Deployment

### Pre-deployment Checklist
- [ ] Super admin principal configured
- [ ] Default parameters validated
- [ ] Emergency procedures tested
- [ ] Access controls verified
- [ ] Audit logging functional

### Post-deployment Setup
1. **Initialize Super Admin**: Call `initialize_super_admin()` with deployer principal
2. **Configure Governance**: Update governance config if needed
3. **Grant Admin Roles**: Assign appropriate roles to team members
4. **Test Emergency Controls**: Verify emergency stop/resume functionality
5. **Monitor Operations**: Set up monitoring for governance activities

### Best Practices
- **Regular Reviews**: Periodically review admin roles and permissions
- **Parameter Monitoring**: Monitor parameter changes and their effects
- **Emergency Drills**: Practice emergency procedures regularly
- **Documentation**: Keep governance procedures well documented
- **Backup Plans**: Maintain emergency recovery procedures

---

**Note**: This governance system provides enterprise-grade administration capabilities with complete transparency, security, and flexibility for managing the Agrilends protocol.
