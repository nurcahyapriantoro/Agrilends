# User Management Testing Script for Agrilends Backend (PowerShell)
# This script tests all user management functionality

param(
    [switch]$CleanupOnly
)

# Colors for output
$Red = "Red"
$Green = "Green"
$Yellow = "Yellow"
$White = "White"

# Test configuration
$CANISTER_NAME = "agrilends_backend_backend"
$IDENTITY_1 = "test_farmer"
$IDENTITY_2 = "test_investor"
$IDENTITY_3 = "test_admin"

# Function to print colored output
function Print-Success {
    param($Message)
    Write-Host "✅ $Message" -ForegroundColor $Green
}

function Print-Error {
    param($Message)
    Write-Host "❌ $Message" -ForegroundColor $Red
}

function Print-Info {
    param($Message)
    Write-Host "ℹ️  $Message" -ForegroundColor $Yellow
}

function Print-TestHeader {
    param($Message)
    Write-Host ""
    Write-Host "🔍 Testing: $Message" -ForegroundColor $Yellow
    Write-Host "----------------------------------------"
}

# Function to create test identities
function Create-Identities {
    Print-Info "Creating test identities..."
    
    # Create farmer identity
    $existingIdentities = dfx identity list
    if ($existingIdentities -contains $IDENTITY_1) {
        Print-Info "Identity $IDENTITY_1 already exists"
    } else {
        dfx identity new $IDENTITY_1 --storage-mode=plaintext
        Print-Success "Created identity: $IDENTITY_1"
    }
    
    # Create investor identity
    if ($existingIdentities -contains $IDENTITY_2) {
        Print-Info "Identity $IDENTITY_2 already exists"
    } else {
        dfx identity new $IDENTITY_2 --storage-mode=plaintext
        Print-Success "Created identity: $IDENTITY_2"
    }
    
    # Create admin identity
    if ($existingIdentities -contains $IDENTITY_3) {
        Print-Info "Identity $IDENTITY_3 already exists"
    } else {
        dfx identity new $IDENTITY_3 --storage-mode=plaintext
        Print-Success "Created identity: $IDENTITY_3"
    }
}

# Function to clean up identities
function Cleanup-Identities {
    Print-Info "Cleaning up test identities..."
    
    # Switch back to default identity
    dfx identity use default
    
    # Remove test identities
    $existingIdentities = dfx identity list
    if ($existingIdentities -contains $IDENTITY_1) {
        dfx identity remove $IDENTITY_1
        Print-Success "Removed identity: $IDENTITY_1"
    }
    
    if ($existingIdentities -contains $IDENTITY_2) {
        dfx identity remove $IDENTITY_2
        Print-Success "Removed identity: $IDENTITY_2"
    }
    
    if ($existingIdentities -contains $IDENTITY_3) {
        dfx identity remove $IDENTITY_3
        Print-Success "Removed identity: $IDENTITY_3"
    }
}

# Function to check if canister is running
function Check-CanisterStatus {
    Print-Info "Checking canister status..."
    
    try {
        dfx canister status $CANISTER_NAME | Out-Null
        Print-Success "Canister is running"
        return $true
    } catch {
        Print-Error "Canister is not running. Please start it first with 'dfx start' and 'dfx deploy'"
        return $false
    }
}

# Function to call canister method and handle response
function Call-Canister {
    param(
        [string]$Method,
        [string]$Args = "()",
        [bool]$ExpectedSuccess = $true
    )
    
    Print-Info "Calling: $Method with args: $Args"
    
    try {
        $response = dfx canister call $CANISTER_NAME $Method $Args 2>&1
        if ($ExpectedSuccess) {
            Print-Success "✅ $Method`: $response"
        } else {
            Print-Error "❌ Expected failure but got success: $response"
        }
        return $response
    } catch {
        if (-not $ExpectedSuccess) {
            Print-Success "✅ Expected failure: $($_.Exception.Message)"
        } else {
            Print-Error "❌ $Method failed: $($_.Exception.Message)"
        }
        return $null
    }
}

# Test 1: Health Check
function Test-HealthCheck {
    Print-TestHeader "Health Check"
    
    dfx identity use default
    Call-Canister "health_check"
}

# Test 2: Register Farmer
function Test-RegisterFarmer {
    Print-TestHeader "Register Farmer"
    
    dfx identity use $IDENTITY_1
    Call-Canister "register_as_farmer"
}

# Test 3: Register Investor
function Test-RegisterInvestor {
    Print-TestHeader "Register Investor"
    
    dfx identity use $IDENTITY_2
    Call-Canister "register_as_investor"
}

# Test 4: Double Registration (Should Fail)
function Test-DoubleRegistration {
    Print-TestHeader "Double Registration (Should Fail)"
    
    dfx identity use $IDENTITY_1
    Call-Canister "register_as_farmer" "()" $false
}

# Test 5: Get User Data
function Test-GetUser {
    Print-TestHeader "Get User Data"
    
    # Test farmer
    dfx identity use $IDENTITY_1
    Call-Canister "get_user"
    
    # Test investor
    dfx identity use $IDENTITY_2
    Call-Canister "get_user"
}

# Test 6: Get Non-existent User (Should Fail)
function Test-GetNonexistentUser {
    Print-TestHeader "Get Non-existent User (Should Fail)"
    
    dfx identity use $IDENTITY_3
    Call-Canister "get_user" "()" $false
}

# Test 7: Update BTC Address
function Test-UpdateBtcAddress {
    Print-TestHeader "Update BTC Address"
    
    dfx identity use $IDENTITY_1
    Call-Canister "update_btc_address" '("1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa")'
    
    dfx identity use $IDENTITY_2
    Call-Canister "update_btc_address" '("3J98t1WpEZ73CNmQviecrnyiWrnqRhWNLy")'
}

# Test 8: Update BTC Address with Invalid Format (Should Fail)
function Test-InvalidBtcAddress {
    Print-TestHeader "Invalid BTC Address (Should Fail)"
    
    dfx identity use $IDENTITY_1
    Call-Canister "update_btc_address" '("invalid_address")' $false
}

# Test 9: Update User Profile
function Test-UpdateUserProfile {
    Print-TestHeader "Update User Profile"
    
    dfx identity use $IDENTITY_1
    Call-Canister "update_user_profile" '(record {
        btc_address = opt "bc1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh";
        email = opt "farmer@example.com";
        phone = opt "+1234567890";
    })'
    
    dfx identity use $IDENTITY_2
    Call-Canister "update_user_profile" '(record {
        btc_address = opt "3J98t1WpEZ73CNmQviecrnyiWrnqRhWNLy";
        email = opt "investor@example.com";
        phone = opt "+0987654321";
    })'
}

# Test 10: Invalid Email Format (Should Fail)
function Test-InvalidEmail {
    Print-TestHeader "Invalid Email Format (Should Fail)"
    
    dfx identity use $IDENTITY_1
    Call-Canister "update_user_profile" '(record {
        email = opt "invalid_email";
        btc_address = null;
        phone = null;
    })' $false
}

# Test 11: Get User Statistics
function Test-GetUserStats {
    Print-TestHeader "Get User Statistics"
    
    dfx identity use default
    Call-Canister "get_user_stats"
}

# Test 12: Check User Role Functions
function Test-UserRoleFunctions {
    Print-TestHeader "User Role Functions"
    
    dfx identity use default
    
    # Get farmer principal
    dfx identity use $IDENTITY_1
    $farmerPrincipal = (dfx canister call $CANISTER_NAME "get_caller").Split('"')[1]
    
    # Get investor principal
    dfx identity use $IDENTITY_2
    $investorPrincipal = (dfx canister call $CANISTER_NAME "get_caller").Split('"')[1]
    
    dfx identity use default
    
    # Test is_farmer
    Print-Info "Testing is_farmer for farmer principal"
    Call-Canister "is_farmer" "(principal `"$farmerPrincipal`")"
    
    Print-Info "Testing is_farmer for investor principal"
    Call-Canister "is_farmer" "(principal `"$investorPrincipal`")"
    
    # Test is_investor
    Print-Info "Testing is_investor for investor principal"
    Call-Canister "is_investor" "(principal `"$investorPrincipal`")"
    
    Print-Info "Testing is_investor for farmer principal"
    Call-Canister "is_investor" "(principal `"$farmerPrincipal`")"
}

# Test 13: Get Users by Role
function Test-GetUsersByRole {
    Print-TestHeader "Get Users by Role"
    
    dfx identity use default
    
    Print-Info "Getting all farmers"
    Call-Canister "get_users_by_role" "(variant { Farmer })"
    
    Print-Info "Getting all investors"
    Call-Canister "get_users_by_role" "(variant { Investor })"
}

# Test 14: Get All Users
function Test-GetAllUsers {
    Print-TestHeader "Get All Users"
    
    dfx identity use default
    Call-Canister "get_all_users"
}

# Test 15: Get Active Users
function Test-GetActiveUsers {
    Print-TestHeader "Get Active Users"
    
    dfx identity use default
    Call-Canister "get_active_users"
}

# Test 16: Deactivate User
function Test-DeactivateUser {
    Print-TestHeader "Deactivate User"
    
    dfx identity use $IDENTITY_1
    Call-Canister "deactivate_user"
}

# Test 17: Reactivate User
function Test-ReactivateUser {
    Print-TestHeader "Reactivate User"
    
    dfx identity use $IDENTITY_1
    Call-Canister "reactivate_user"
}

# Test 18: Check Profile Completion
function Test-ProfileCompletion {
    Print-TestHeader "Profile Completion Check"
    
    dfx identity use $IDENTITY_1
    $farmerPrincipal = (dfx canister call $CANISTER_NAME "get_caller").Split('"')[1]
    
    dfx identity use $IDENTITY_2
    $investorPrincipal = (dfx canister call $CANISTER_NAME "get_caller").Split('"')[1]
    
    dfx identity use default
    
    Print-Info "Checking profile completion for farmer"
    Call-Canister "has_completed_profile" "(principal `"$farmerPrincipal`")"
    
    Print-Info "Checking profile completion for investor"
    Call-Canister "has_completed_profile" "(principal `"$investorPrincipal`")"
}

# Main execution
function Main {
    Write-Host "===============================================" -ForegroundColor $Yellow
    Write-Host "🧪 AGRILENDS USER MANAGEMENT TESTING SCRIPT" -ForegroundColor $Yellow
    Write-Host "===============================================" -ForegroundColor $Yellow
    Write-Host ""
    
    if ($CleanupOnly) {
        Cleanup-Identities
        return
    }
    
    Write-Host "Starting User Management Tests..." -ForegroundColor $White
    
    # Check if canister is running
    if (-not (Check-CanisterStatus)) {
        exit 1
    }
    
    # Create test identities
    Create-Identities
    
    # Run tests
    Test-HealthCheck
    Test-RegisterFarmer
    Test-RegisterInvestor
    Test-DoubleRegistration
    Test-GetUser
    Test-GetNonexistentUser
    Test-UpdateBtcAddress
    Test-InvalidBtcAddress
    Test-UpdateUserProfile
    Test-InvalidEmail
    Test-GetUserStats
    Test-UserRoleFunctions
    Test-GetUsersByRole
    Test-GetAllUsers
    Test-GetActiveUsers
    Test-DeactivateUser
    Test-ReactivateUser
    Test-ProfileCompletion
    
    Write-Host ""
    Write-Host "===============================================" -ForegroundColor $Yellow
    Print-Success "🎉 ALL TESTS COMPLETED!"
    Write-Host "===============================================" -ForegroundColor $Yellow
    
    # Ask if user wants to clean up
    Write-Host ""
    $cleanup = Read-Host "Do you want to clean up test identities? (y/n)"
    if ($cleanup -eq "y" -or $cleanup -eq "Y") {
        Cleanup-Identities
    }
}

# Handle script interruption
trap {
    Write-Host ""
    Print-Error "Script interrupted by user"
    Cleanup-Identities
    exit 1
}

# Run main function
Main
