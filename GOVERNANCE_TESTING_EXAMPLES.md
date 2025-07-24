# 🧪 Governance System Testing Examples

## Contoh Penggunaan dan Testing untuk Fitur Tata Kelola & Administrasi

### 1. Setup Awal Sistem

```bash
# Deploy canister
dfx deploy agrilends_backend

# Get canister ID
dfx canister id agrilends_backend
```

### 2. Inisialisasi Super Admin

```bash
# Set principal sebagai super admin (hanya sekali)
dfx canister call agrilends_backend initialize_super_admin '(principal "rrkah-fqaaa-aaaah-qcuaq-cai")'
```

### 3. Testing Parameter Management

#### Update Single Parameter
```bash
# Update LTV ratio ke 65%
dfx canister call agrilends_backend set_protocol_parameter '("loan_to_value_ratio", 6500)'

# Expected Response: (variant { Ok = "Parameter loan_to_value_ratio updated successfully" })
```

#### Get Parameter
```bash
# Get current LTV ratio
dfx canister call agrilends_backend get_protocol_parameter '("loan_to_value_ratio")'

# Expected Response: 
# (variant { 
#   Ok = record {
#     key = "loan_to_value_ratio";
#     current_value = 6500;
#     value_type = variant { Percentage };
#     min_value = opt (5000);
#     max_value = opt (8000);
#     description = "Maximum loan-to-value ratio for collateral (default 60%)";
#     last_updated = 1_640_995_200_000_000_000;
#     updated_by = principal "rrkah-fqaaa-aaaah-qcuaq-cai";
#   }
# })
```

#### Get All Parameters
```bash
# List semua protocol parameters
dfx canister call agrilends_backend get_all_protocol_parameters '()'
```

#### Batch Parameter Update
```bash
# Update multiple parameters sekaligus
dfx canister call agrilends_backend set_multiple_protocol_parameters '(vec {
  record { "loan_to_value_ratio"; 6500 };
  record { "base_apr"; 1200 };
  record { "liquidation_threshold"; 8000 };
})'
```

#### Parameter Validation
```bash
# Test validation - value terlalu tinggi (should fail)
dfx canister call agrilends_backend set_protocol_parameter '("loan_to_value_ratio", 9500)'

# Expected Response: (variant { Err = "Value 9500 is above maximum 8000" })

# Test validation - value valid
dfx canister call agrilends_backend validate_parameter_value '("loan_to_value_ratio", 7000)'

# Expected Response: (variant { Ok = "Parameter value is valid" })
```

### 4. Testing Admin Role Management

#### Grant Admin Role
```bash
# Grant Protocol Admin role
dfx canister call agrilends_backend grant_admin_role '(
  principal "rdmx6-jaaaa-aaaah-qdrha-cai",
  variant { ProtocolAdmin },
  vec { variant { ManageParameters }; variant { ViewMetrics } },
  null
)'

# Expected Response: (variant { Ok = "Admin role granted successfully" })
```

#### Get Admin Role
```bash
# Check admin role
dfx canister call agrilends_backend get_admin_role '(principal "rdmx6-jaaaa-aaaah-qdrha-cai")'

# Expected Response:
# (opt record {
#   principal = principal "rdmx6-jaaaa-aaaah-qdrha-cai";
#   role_type = variant { ProtocolAdmin };
#   granted_at = 1_640_995_200_000_000_000;
#   granted_by = principal "rrkah-fqaaa-aaaah-qcuaq-cai";
#   expires_at = null;
#   permissions = vec { variant { ManageParameters }; variant { ViewMetrics } };
#   is_active = true;
# })
```

#### Revoke Admin Role
```bash
# Revoke admin role
dfx canister call agrilends_backend revoke_admin_role '(principal "rdmx6-jaaaa-aaaah-qdrha-cai")'

# Expected Response: (variant { Ok = "Admin role revoked successfully" })
```

#### Transfer Super Admin
```bash
# Transfer super admin role ke principal lain
dfx canister call agrilends_backend transfer_admin_role '(principal "new-admin-principal")'

# Expected Response: (variant { Ok = "Admin role transferred successfully" })
```

### 5. Testing Proposal System

#### Create Proposal
```bash
# Create parameter update proposal
dfx canister call agrilends_backend create_proposal '(
  variant { ProtocolParameterUpdate },
  "Increase LTV Ratio to 70%",
  "This proposal aims to increase the loan-to-value ratio to 70% to improve capital efficiency for borrowers while maintaining acceptable risk levels.",
  opt blob "loan_to_value_ratio:7000"
)'

# Expected Response: (variant { Ok = 1 })  // Proposal ID
```

#### Vote on Proposal
```bash
# Vote Yes on proposal
dfx canister call agrilends_backend vote_on_proposal '(
  1,
  variant { Yes },
  opt "This change will improve borrower access to capital while maintaining risk controls"
)'

# Expected Response: (variant { Ok = "Vote cast successfully" })
```

#### Get Proposal
```bash
# Get proposal details
dfx canister call agrilends_backend get_proposal '(1)'

# Expected Response: 
# (opt record {
#   id = 1;
#   proposer = principal "rrkah-fqaaa-aaaah-qcuaq-cai";
#   proposal_type = variant { ProtocolParameterUpdate };
#   title = "Increase LTV Ratio to 70%";
#   description = "This proposal aims to...";
#   created_at = 1_640_995_200_000_000_000;
#   voting_deadline = 1_641_600_000_000_000_000;
#   execution_deadline = 1_641_772_800_000_000_000;
#   status = variant { Active };
#   yes_votes = 1000;
#   no_votes = 0;
#   abstain_votes = 0;
#   // ... other fields
# })
```

#### Check if Proposal Can Be Executed
```bash
# Check execution eligibility
dfx canister call agrilends_backend can_execute_proposal '(1)'

# Expected Response: (variant { Ok = true })  // If conditions met
```

#### Execute Proposal
```bash
# Execute approved proposal
dfx canister call agrilends_backend execute_proposal '(1)'

# Expected Response: (variant { Ok = "Parameter loan_to_value_ratio updated to 7000" })
```

#### Get Proposals by Status
```bash
# Get active proposals
dfx canister call agrilends_backend get_proposals_by_status '(variant { Active }, 0, 10)'

# Get executed proposals
dfx canister call agrilends_backend get_proposals_by_status '(variant { Executed }, 0, 10)'
```

### 6. Testing Emergency Controls

#### Emergency Stop
```bash
# Activate emergency stop
dfx canister call agrilends_backend emergency_stop '()'

# Expected Response: (variant { Ok = "Emergency stop activated" })

# Verify emergency stop parameter
dfx canister call agrilends_backend get_protocol_parameter '("emergency_stop")'

# Expected Response: current_value = 1
```

#### Resume Operations
```bash
# Resume operations after emergency
dfx canister call agrilends_backend resume_operations '()'

# Expected Response: (variant { Ok = "Operations resumed successfully" })
```

#### Maintenance Mode
```bash
# Enable maintenance mode
dfx canister call agrilends_backend set_maintenance_mode '(true)'

# Expected Response: (variant { Ok = "Maintenance mode enabled" })

# Disable maintenance mode
dfx canister call agrilends_backend set_maintenance_mode '(false)'

# Expected Response: (variant { Ok = "Maintenance mode disabled" })
```

### 7. Testing System Status & Monitoring

#### Get System Status
```bash
# Get current system status
dfx canister call agrilends_backend get_system_status '()'

# Expected Response:
# (vec {
#   record { "emergency_stop"; false };
#   record { "maintenance_mode"; false };
# })
```

#### Get Governance Statistics
```bash
# Get governance stats
dfx canister call agrilends_backend get_governance_stats '()'

# Expected Response:
# (record {
#   total_proposals = 5;
#   active_proposals = 2;
#   executed_proposals = 2;
#   total_votes_cast = 15;
#   total_voting_power = 5000;
#   average_participation_rate = 7500;
#   last_proposal_id = 5;
# })
```

#### Get Governance Dashboard
```bash
# Get complete dashboard data
dfx canister call agrilends_backend get_governance_dashboard '()'

# Returns comprehensive dashboard data including stats, proposals, admins, etc.
```

### 8. Testing Error Cases

#### Unauthorized Access
```bash
# Try to update parameter as non-admin (should fail)
# First switch to different identity
dfx identity use default

dfx canister call agrilends_backend set_protocol_parameter '("loan_to_value_ratio", 6500)'

# Expected Response: (variant { Err = "Unauthorized: Only admins can set parameters directly" })
```

#### Invalid Parameter Values
```bash
# Try invalid parameter value (should fail)
dfx canister call agrilends_backend set_protocol_parameter '("loan_to_value_ratio", 15000)'

# Expected Response: (variant { Err = "Value 15000 is above maximum 8000" })
```

#### Double Voting
```bash
# Try to vote twice on same proposal (should fail)
dfx canister call agrilends_backend vote_on_proposal '(1, variant { Yes }, null)'
dfx canister call agrilends_backend vote_on_proposal '(1, variant { No }, null)'

# Second call expected response: (variant { Err = "Already voted" })
```

### 9. Comprehensive Testing Script

```bash
#!/bin/bash

# governance_test.sh - Comprehensive governance testing script

echo "🏛️ Starting Agrilends Governance System Tests"

# Setup
echo "1. Setting up test environment..."
dfx identity use alice
ALICE_PRINCIPAL=$(dfx identity get-principal)

dfx identity use bob  
BOB_PRINCIPAL=$(dfx identity get-principal)

dfx identity use alice

# Initialize super admin
echo "2. Initializing super admin..."
dfx canister call agrilends_backend initialize_super_admin "(principal \"$ALICE_PRINCIPAL\")"

# Test parameter management
echo "3. Testing parameter management..."
dfx canister call agrilends_backend set_protocol_parameter '("loan_to_value_ratio", 6500)'
dfx canister call agrilends_backend get_protocol_parameter '("loan_to_value_ratio")'

# Test admin role management
echo "4. Testing admin role management..."
dfx canister call agrilends_backend grant_admin_role "(principal \"$BOB_PRINCIPAL\", variant { ProtocolAdmin }, vec { variant { ManageParameters } }, null)"
dfx canister call agrilends_backend get_admin_role "(principal \"$BOB_PRINCIPAL\")"

# Test proposal system
echo "5. Testing proposal system..."
PROPOSAL_ID=$(dfx canister call agrilends_backend create_proposal '(variant { ProtocolParameterUpdate }, "Test Proposal", "Test proposal description", opt blob "test_key:1000")' | grep -o '[0-9]*')
dfx canister call agrilends_backend vote_on_proposal "($PROPOSAL_ID, variant { Yes }, opt \"Test vote\")"

# Test emergency controls
echo "6. Testing emergency controls..."
dfx canister call agrilends_backend emergency_stop '()'
dfx canister call agrilends_backend get_system_status '()'
dfx canister call agrilends_backend resume_operations '()'

# Test unauthorized access
echo "7. Testing security..."
dfx identity use bob
dfx canister call agrilends_backend emergency_stop '()' 2>&1 | grep -q "Unauthorized" && echo "✅ Security test passed" || echo "❌ Security test failed"

dfx identity use alice

echo "🎉 Governance tests completed!"
```

### 10. Performance Testing

```bash
# Test batch operations performance
dfx canister call agrilends_backend set_multiple_protocol_parameters '(vec {
  record { "loan_to_value_ratio"; 6000 };
  record { "base_apr"; 1000 };
  record { "liquidation_threshold"; 7500 };
  record { "grace_period_days"; 30 };
  record { "max_utilization_rate"; 8000 };
  record { "protocol_fee_rate"; 200 };
})'

# Test large proposal list retrieval
dfx canister call agrilends_backend get_proposals '(0, 100)'

# Test parameter category filtering
dfx canister call agrilends_backend get_protocol_parameters_by_category '("loan")'
```

### 11. Integration Testing

```bash
# Test governance integration with other modules

# 1. Test parameter change affects loan lifecycle
dfx canister call agrilends_backend set_protocol_parameter '("loan_to_value_ratio", 7000)'

# Then test loan application with new LTV ratio
dfx canister call agrilends_backend submit_loan_application '(...)'

# 2. Test emergency stop affects all operations
dfx canister call agrilends_backend emergency_stop '()'

# Try to perform operations (should be blocked)
dfx canister call agrilends_backend submit_loan_application '(...)'
dfx canister call agrilends_backend deposit_liquidity '(...)'
```

### 12. Web Interface Testing

```bash
# Open governance dashboard in browser
open governance_dashboard.html

# Test dashboard functionality:
# - Parameter updates
# - Proposal creation
# - Admin management
# - System status monitoring
# - Emergency controls
```

## Expected Test Results

### ✅ Success Cases:
- Parameter updates by admin succeed
- Valid parameter values are accepted
- Admin roles can be granted/revoked
- Proposals can be created and voted on
- Emergency controls work as expected
- System status is properly reported

### ❌ Failure Cases:
- Non-admin parameter updates are rejected
- Invalid parameter values are rejected
- Unauthorized admin operations are blocked
- Double voting is prevented
- Emergency stop blocks operations
- Invalid proposal execution is prevented

## Testing Checklist

- [ ] Super admin initialization works
- [ ] Parameter management functions correctly
- [ ] Admin role system is secure
- [ ] Proposal lifecycle works end-to-end
- [ ] Voting system functions properly
- [ ] Emergency controls are effective
- [ ] System status monitoring is accurate
- [ ] Unauthorized access is blocked
- [ ] Parameter validation works
- [ ] Integration with other modules is seamless
- [ ] Web interface is functional
- [ ] Performance is acceptable
- [ ] Error handling is robust

---

**Note**: Pastikan untuk menjalankan tests ini di environment development sebelum deployment ke production. Semua test cases ini dirancang untuk memverifikasi bahwa sistem governance berfungsi dengan benar dan aman.
