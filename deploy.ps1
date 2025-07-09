# Agrilends User Management Deployment Script (PowerShell)

param(
    [switch]$Build,
    [switch]$Deploy,
    [switch]$Test,
    [switch]$All,
    [switch]$Clean,
    [switch]$Stop,
    [string]$Network = "local"
)

# Colors for output
$Red = "Red"
$Green = "Green"
$Yellow = "Yellow"
$Blue = "Blue"
$White = "White"

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

function Print-Header {
    param($Message)
    Write-Host ""
    Write-Host "🚀 $Message" -ForegroundColor $Blue
    Write-Host "================================================="
}

function Check-Prerequisites {
    Print-Header "Checking Prerequisites"
    
    # Check if dfx is installed
    try {
        $dfxVersion = dfx --version
        Print-Success "dfx is installed: $dfxVersion"
    } catch {
        Print-Error "dfx is not installed. Please install the Internet Computer SDK."
        exit 1
    }
    
    # Check if Rust is installed
    try {
        $rustVersion = cargo --version
        Print-Success "Rust is installed: $rustVersion"
    } catch {
        Print-Error "Rust is not installed. Please install Rust."
        exit 1
    }
    
    # Check if we're in the correct directory
    if (-not (Test-Path "agrilends_backend")) {
        Print-Error "Please run this script from the Agrilends root directory."
        exit 1
    }
    
    Print-Success "All prerequisites met!"
}

function Start-Replica {
    Print-Header "Starting Internet Computer Replica"
    
    # Check if replica is already running
    try {
        dfx ping | Out-Null
        Print-Info "IC replica is already running"
    } catch {
        Print-Info "Starting IC replica..."
        if ($Network -eq "local") {
            dfx start --background
            
            # Wait for replica to be ready
            $maxAttempts = 30
            $attempts = 0
            while ($attempts -lt $maxAttempts) {
                try {
                    dfx ping | Out-Null
                    Print-Success "IC replica is ready!"
                    break
                } catch {
                    $attempts++
                    Write-Host "." -NoNewline
                    Start-Sleep -Seconds 1
                }
            }
            
            if ($attempts -eq $maxAttempts) {
                Print-Error "Failed to start IC replica after $maxAttempts attempts"
                exit 1
            }
        } else {
            Print-Info "Using network: $Network"
        }
    }
}

function Stop-Replica {
    Print-Header "Stopping Internet Computer Replica"
    
    try {
        dfx stop
        Print-Success "IC replica stopped"
    } catch {
        Print-Info "IC replica was not running"
    }
}

function Build-Canister {
    Print-Header "Building Canister"
    
    Set-Location "agrilends_backend"
    
    try {
        Print-Info "Building Rust canister..."
        dfx build --network $Network
        Print-Success "Canister built successfully!"
    } catch {
        Print-Error "Failed to build canister: $($_.Exception.Message)"
        exit 1
    } finally {
        Set-Location ".."
    }
}

function Deploy-Canister {
    Print-Header "Deploying Canister"
    
    Set-Location "agrilends_backend"
    
    try {
        Print-Info "Deploying canister to $Network network..."
        dfx deploy --network $Network
        Print-Success "Canister deployed successfully!"
        
        # Get canister ID
        $canisterId = dfx canister id agrilends_backend_backend --network $Network
        Print-Info "Canister ID: $canisterId"
        
        # Test deployment
        Print-Info "Testing deployment..."
        $healthResponse = dfx canister call agrilends_backend_backend health_check --network $Network
        Print-Success "Health check response: $healthResponse"
        
    } catch {
        Print-Error "Failed to deploy canister: $($_.Exception.Message)"
        exit 1
    } finally {
        Set-Location ".."
    }
}

function Run-Tests {
    Print-Header "Running Tests"
    
    try {
        # Check if test script exists
        if (Test-Path "testingmanagementuser.ps1") {
            Print-Info "Running user management tests..."
            .\testingmanagementuser.ps1
        } else {
            Print-Error "Test script not found: testingmanagementuser.ps1"
            exit 1
        }
    } catch {
        Print-Error "Tests failed: $($_.Exception.Message)"
        exit 1
    }
}

function Clean-Environment {
    Print-Header "Cleaning Environment"
    
    Set-Location "agrilends_backend"
    
    try {
        # Clean build artifacts
        Print-Info "Cleaning build artifacts..."
        if (Test-Path "target") {
            Remove-Item -Recurse -Force "target"
            Print-Success "Removed target directory"
        }
        
        if (Test-Path ".dfx") {
            Remove-Item -Recurse -Force ".dfx"
            Print-Success "Removed .dfx directory"
        }
        
        # Remove node_modules if exists
        if (Test-Path "node_modules") {
            Remove-Item -Recurse -Force "node_modules"
            Print-Success "Removed node_modules directory"
        }
        
        Print-Success "Environment cleaned successfully!"
        
    } catch {
        Print-Error "Failed to clean environment: $($_.Exception.Message)"
        exit 1
    } finally {
        Set-Location ".."
    }
}

function Show-Status {
    Print-Header "Deployment Status"
    
    Set-Location "agrilends_backend"
    
    try {
        # Check replica status
        try {
            dfx ping | Out-Null
            Print-Success "IC replica is running"
        } catch {
            Print-Error "IC replica is not running"
        }
        
        # Check canister status
        try {
            $status = dfx canister status agrilends_backend_backend --network $Network
            Print-Success "Canister status: $status"
        } catch {
            Print-Error "Canister is not deployed"
        }
        
        # Show canister info
        try {
            $canisterId = dfx canister id agrilends_backend_backend --network $Network
            Print-Info "Canister ID: $canisterId"
            
            if ($Network -eq "local") {
                $url = "http://localhost:4943/?canisterId=$canisterId"
                Print-Info "Candid UI: $url"
            }
        } catch {
            Print-Info "Canister ID not available"
        }
        
    } catch {
        Print-Error "Failed to get status: $($_.Exception.Message)"
    } finally {
        Set-Location ".."
    }
}

function Show-Help {
    Write-Host "Agrilends User Management Deployment Script" -ForegroundColor $Blue
    Write-Host "Usage: .\deploy.ps1 [OPTIONS]" -ForegroundColor $White
    Write-Host ""
    Write-Host "Options:" -ForegroundColor $Yellow
    Write-Host "  -Build          Build the canister only" -ForegroundColor $White
    Write-Host "  -Deploy         Deploy the canister only" -ForegroundColor $White
    Write-Host "  -Test           Run tests only" -ForegroundColor $White
    Write-Host "  -All            Build, deploy, and test" -ForegroundColor $White
    Write-Host "  -Clean          Clean build artifacts" -ForegroundColor $White
    Write-Host "  -Stop           Stop the IC replica" -ForegroundColor $White
    Write-Host "  -Network <name> Specify network (default: local)" -ForegroundColor $White
    Write-Host ""
    Write-Host "Examples:" -ForegroundColor $Yellow
    Write-Host "  .\deploy.ps1 -All                  # Full deployment" -ForegroundColor $White
    Write-Host "  .\deploy.ps1 -Build                # Build only" -ForegroundColor $White
    Write-Host "  .\deploy.ps1 -Deploy -Network ic   # Deploy to mainnet" -ForegroundColor $White
    Write-Host "  .\deploy.ps1 -Clean                # Clean environment" -ForegroundColor $White
}

# Main execution
function Main {
    Write-Host "=============================================" -ForegroundColor $Blue
    Write-Host "🚀 AGRILENDS USER MANAGEMENT DEPLOYMENT" -ForegroundColor $Blue
    Write-Host "=============================================" -ForegroundColor $Blue
    
    # Show help if no parameters
    if (-not ($Build -or $Deploy -or $Test -or $All -or $Clean -or $Stop)) {
        Show-Help
        return
    }
    
    # Handle clean operation
    if ($Clean) {
        Clean-Environment
        return
    }
    
    # Handle stop operation
    if ($Stop) {
        Stop-Replica
        return
    }
    
    # Check prerequisites
    Check-Prerequisites
    
    # Start replica
    Start-Replica
    
    # Handle different operations
    if ($All) {
        Build-Canister
        Deploy-Canister
        Run-Tests
        Show-Status
    } elseif ($Build) {
        Build-Canister
    } elseif ($Deploy) {
        Deploy-Canister
        Show-Status
    } elseif ($Test) {
        Run-Tests
    }
    
    Write-Host ""
    Write-Host "=============================================" -ForegroundColor $Blue
    Print-Success "🎉 DEPLOYMENT COMPLETED!"
    Write-Host "=============================================" -ForegroundColor $Blue
}

# Error handling
trap {
    Print-Error "An error occurred: $($_.Exception.Message)"
    exit 1
}

# Run main function
Main
