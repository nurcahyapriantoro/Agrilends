#!/bin/bash

# User Management Testing Script for Agrilends Backend
# This script tests all user management functionality

set -e

echo "==============================================="
echo "🧪 AGRILENDS USER MANAGEMENT TESTING SCRIPT"
echo "==============================================="
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Test configuration
CANISTER_NAME="agrilends_backend_backend"
IDENTITY_1="test_farmer"
IDENTITY_2="test_investor"
IDENTITY_3="test_admin"

# Function to print colored output
print_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

print_info() {
    echo -e "${YELLOW}ℹ️  $1${NC}"
}

print_test_header() {
    echo -e "\n${YELLOW}🔍 Testing: $1${NC}"
    echo "----------------------------------------"
}

# Function to create test identities
create_identities() {
    print_info "Creating test identities..."
    
    # Create farmer identity
    if dfx identity list | grep -q "$IDENTITY_1"; then
        print_info "Identity $IDENTITY_1 already exists"
    else
        dfx identity new "$IDENTITY_1" --storage-mode=plaintext
        print_success "Created identity: $IDENTITY_1"
    fi
    
    # Create investor identity
    if dfx identity list | grep -q "$IDENTITY_2"; then
        print_info "Identity $IDENTITY_2 already exists"
    else
        dfx identity new "$IDENTITY_2" --storage-mode=plaintext
        print_success "Created identity: $IDENTITY_2"
    fi
    
    # Create admin identity
    if dfx identity list | grep -q "$IDENTITY_3"; then
        print_info "Identity $IDENTITY_3 already exists"
    else
        dfx identity new "$IDENTITY_3" --storage-mode=plaintext
        print_success "Created identity: $IDENTITY_3"
    fi
}

# Function to clean up identities
cleanup_identities() {
    print_info "Cleaning up test identities..."
    
    # Switch back to default identity
    dfx identity use default
    
    # Remove test identities
    if dfx identity list | grep -q "$IDENTITY_1"; then
        dfx identity remove "$IDENTITY_1"
        print_success "Removed identity: $IDENTITY_1"
    fi
    
    if dfx identity list | grep -q "$IDENTITY_2"; then
        dfx identity remove "$IDENTITY_2"
        print_success "Removed identity: $IDENTITY_2"
    fi
    
    if dfx identity list | grep -q "$IDENTITY_3"; then
        dfx identity remove "$IDENTITY_3"
        print_success "Removed identity: $IDENTITY_3"
    fi
}

# Function to check if canister is running
check_canister_status() {
    print_info "Checking canister status..."
    
    if dfx canister status "$CANISTER_NAME" >/dev/null 2>&1; then
        print_success "Canister is running"
        return 0
    else
        print_error "Canister is not running. Please start it first with 'dfx start' and 'dfx deploy'"
        return 1
    fi
}

# Function to call canister method and handle response
call_canister() {
    local method=$1
    local args=${2:-"()"}
    local expected_success=${3:-true}
    
    print_info "Calling: $method with args: $args"
    
    local response
    if response=$(dfx canister call "$CANISTER_NAME" "$method" "$args" 2>&1); then
        if [ "$expected_success" = true ]; then
            print_success "✅ $method: $response"
        else
            print_error "❌ Expected failure but got success: $response"
        fi
        echo "$response"
    else
        if [ "$expected_success" = false ]; then
            print_success "✅ Expected failure: $response"
        else
            print_error "❌ $method failed: $response"
        fi
        echo "$response"
    fi
}

# Test 1: Health Check
test_health_check() {
    print_test_header "Health Check"
    
    dfx identity use default
    call_canister "health_check"
}

# Test 2: Register Farmer
test_register_farmer() {
    print_test_header "Register Farmer"
    
    dfx identity use "$IDENTITY_1"
    call_canister "register_as_farmer"
}

# Test 3: Register Investor
test_register_investor() {
    print_test_header "Register Investor"
    
    dfx identity use "$IDENTITY_2"
    call_canister "register_as_investor"
}

# Test 4: Double Registration (Should Fail)
test_double_registration() {
    print_test_header "Double Registration (Should Fail)"
    
    dfx identity use "$IDENTITY_1"
    call_canister "register_as_farmer" "()" false
}

# Test 5: Get User Data
test_get_user() {
    print_test_header "Get User Data"
    
    # Test farmer
    dfx identity use "$IDENTITY_1"
    call_canister "get_user"
    
    # Test investor
    dfx identity use "$IDENTITY_2"
    call_canister "get_user"
}

# Test 6: Get Non-existent User (Should Fail)
test_get_nonexistent_user() {
    print_test_header "Get Non-existent User (Should Fail)"
    
    dfx identity use "$IDENTITY_3"
    call_canister "get_user" "()" false
}

# Test 7: Update BTC Address
test_update_btc_address() {
    print_test_header "Update BTC Address"
    
    dfx identity use "$IDENTITY_1"
    call_canister "update_btc_address" '("1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa")'
    
    dfx identity use "$IDENTITY_2"
    call_canister "update_btc_address" '("3J98t1WpEZ73CNmQviecrnyiWrnqRhWNLy")'
}

# Test 8: Update BTC Address with Invalid Format (Should Fail)
test_invalid_btc_address() {
    print_test_header "Invalid BTC Address (Should Fail)"
    
    dfx identity use "$IDENTITY_1"
    call_canister "update_btc_address" '("invalid_address")' false
}

# Test 9: Update User Profile
test_update_user_profile() {
    print_test_header "Update User Profile"
    
    dfx identity use "$IDENTITY_1"
    call_canister "update_user_profile" '(record {
        btc_address = opt "bc1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh";
        email = opt "farmer@example.com";
        phone = opt "+1234567890";
    })'
    
    dfx identity use "$IDENTITY_2"
    call_canister "update_user_profile" '(record {
        btc_address = opt "3J98t1WpEZ73CNmQviecrnyiWrnqRhWNLy";
        email = opt "investor@example.com";
        phone = opt "+0987654321";
    })'
}

# Test 10: Invalid Email Format (Should Fail)
test_invalid_email() {
    print_test_header "Invalid Email Format (Should Fail)"
    
    dfx identity use "$IDENTITY_1"
    call_canister "update_user_profile" '(record {
        email = opt "invalid_email";
        btc_address = null;
        phone = null;
    })' false
}

# Test 11: Get User Statistics
test_get_user_stats() {
    print_test_header "Get User Statistics"
    
    dfx identity use default
    call_canister "get_user_stats"
}

# Test 12: Check User Role Functions
test_user_role_functions() {
    print_test_header "User Role Functions"
    
    dfx identity use default
    
    # Get farmer principal
    dfx identity use "$IDENTITY_1"
    local farmer_principal=$(dfx canister call "$CANISTER_NAME" "get_caller" | cut -d'"' -f2)
    
    # Get investor principal
    dfx identity use "$IDENTITY_2"
    local investor_principal=$(dfx canister call "$CANISTER_NAME" "get_caller" | cut -d'"' -f2)
    
    dfx identity use default
    
    # Test is_farmer
    print_info "Testing is_farmer for farmer principal"
    call_canister "is_farmer" "(principal \"$farmer_principal\")"
    
    print_info "Testing is_farmer for investor principal"
    call_canister "is_farmer" "(principal \"$investor_principal\")"
    
    # Test is_investor
    print_info "Testing is_investor for investor principal"
    call_canister "is_investor" "(principal \"$investor_principal\")"
    
    print_info "Testing is_investor for farmer principal"
    call_canister "is_investor" "(principal \"$farmer_principal\")"
}

# Test 13: Get Users by Role
test_get_users_by_role() {
    print_test_header "Get Users by Role"
    
    dfx identity use default
    
    print_info "Getting all farmers"
    call_canister "get_users_by_role" "(variant { Farmer })"
    
    print_info "Getting all investors"
    call_canister "get_users_by_role" "(variant { Investor })"
}

# Test 14: Get All Users
test_get_all_users() {
    print_test_header "Get All Users"
    
    dfx identity use default
    call_canister "get_all_users"
}

# Test 15: Get Active Users
test_get_active_users() {
    print_test_header "Get Active Users"
    
    dfx identity use default
    call_canister "get_active_users"
}

# Test 16: Deactivate User
test_deactivate_user() {
    print_test_header "Deactivate User"
    
    dfx identity use "$IDENTITY_1"
    call_canister "deactivate_user"
}

# Test 17: Reactivate User
test_reactivate_user() {
    print_test_header "Reactivate User"
    
    dfx identity use "$IDENTITY_1"
    call_canister "reactivate_user"
}

# Test 18: Check Profile Completion
test_profile_completion() {
    print_test_header "Profile Completion Check"
    
    dfx identity use "$IDENTITY_1"
    local farmer_principal=$(dfx canister call "$CANISTER_NAME" "get_caller" | cut -d'"' -f2)
    
    dfx identity use "$IDENTITY_2"
    local investor_principal=$(dfx canister call "$CANISTER_NAME" "get_caller" | cut -d'"' -f2)
    
    dfx identity use default
    
    print_info "Checking profile completion for farmer"
    call_canister "has_completed_profile" "(principal \"$farmer_principal\")"
    
    print_info "Checking profile completion for investor"
    call_canister "has_completed_profile" "(principal \"$investor_principal\")"
}

# Main test execution
main() {
    echo "Starting User Management Tests..."
    
    # Check if canister is running
    if ! check_canister_status; then
        exit 1
    fi
    
    # Create test identities
    create_identities
    
    # Run tests
    test_health_check
    test_register_farmer
    test_register_investor
    test_double_registration
    test_get_user
    test_get_nonexistent_user
    test_update_btc_address
    test_invalid_btc_address
    test_update_user_profile
    test_invalid_email
    test_get_user_stats
    test_user_role_functions
    test_get_users_by_role
    test_get_all_users
    test_get_active_users
    test_deactivate_user
    test_reactivate_user
    test_profile_completion
    
    echo ""
    echo "==============================================="
    print_success "🎉 ALL TESTS COMPLETED!"
    echo "==============================================="
    
    # Ask if user wants to clean up
    echo ""
    read -p "Do you want to clean up test identities? (y/n): " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        cleanup_identities
    fi
}

# Handle script interruption
trap 'echo -e "\n${RED}Script interrupted by user${NC}"; cleanup_identities; exit 1' INT

# Run main function
main "$@"
