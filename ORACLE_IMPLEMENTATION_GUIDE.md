# AGRILENDS ORACLE SYSTEM
## Production Implementation Guide

This comprehensive guide covers the implementation, deployment, and maintenance of the Agrilends Oracle system for production environments.

---

## 🎯 SYSTEM OVERVIEW

### Purpose
The Agrilends Oracle system provides secure, reliable commodity price data for the DeFi lending platform, enabling accurate collateral valuation and automated liquidation triggers.

### Key Features
- **HTTPS Outcalls**: Secure external API integration with consensus mechanisms
- **Multi-source Redundancy**: Backup APIs for maximum reliability
- **Rate Limiting**: Intelligent throttling to prevent API abuse
- **Emergency Mode**: Fallback pricing when external sources fail
- **Real-time Monitoring**: Comprehensive health checks and alerting
- **Audit Logging**: Complete transaction history for compliance

---

## 🏗️ ARCHITECTURE

### Core Components

```
┌─────────────────────────────────────────────────────────────┐
│                    Oracle System                           │
├─────────────────────────────────────────────────────────────┤
│  HTTP Outcalls ─→ Transform ─→ Validation ─→ Storage       │
│       ↓              ↓            ↓           ↓            │
│   External APIs  Consensus     Quality     Thread Local     │
│                               Scoring        Storage        │
└─────────────────────────────────────────────────────────────┘
```

### Data Flow
1. **Price Request** → Oracle receives commodity price request
2. **API Call** → HTTPS outcall to external price API
3. **Transform** → Consensus function processes raw response
4. **Validation** → Quality scoring and data validation
5. **Storage** → Persistent storage with metadata
6. **Response** → Clean price data returned to caller

---

## 📁 FILE STRUCTURE

### Core Implementation (`src/agrilends_backend/src/`)
```
oracle.rs                    # Main Oracle implementation
├── Price fetching functions
├── Configuration management
├── Health monitoring
├── Statistics tracking
└── Admin functions

types.rs                     # Data structures
├── CommodityPriceData
├── OracleConfig
├── OracleStatistics
└── PriceAlert types

storage.rs                   # Data persistence
├── Price storage functions
├── Configuration storage
└── Statistics tracking

helpers.rs                   # Utility functions
├── Validation helpers
├── Conversion utilities
└── Error handling
```

### Frontend Components (`src/agrilends_frontend/`)
```
oracle_dashboard.html        # Web-based management interface
├── Real-time price display
├── Health monitoring
├── Configuration management
└── Statistics visualization
```

### Configuration Files
```
dfx.json                     # IC project configuration
package.json                 # Frontend dependencies
Cargo.toml                   # Rust dependencies
```

---

## 🔧 IMPLEMENTATION DETAILS

### 1. Oracle Core Functions

#### Price Fetching
```rust
#[ic_cdk::update]
async fn fetch_commodity_price(commodity_id: String) -> Result<CommodityPriceData, String> {
    // Rate limiting check
    // Admin permission verification  
    // External API call with HTTPS outcalls
    // Response validation and consensus
    // Storage and audit logging
    // Return processed data
}
```

#### Query Functions
```rust
#[ic_cdk::query]
fn get_commodity_price(commodity_id: String) -> Result<CommodityPriceData, String> {
    // Retrieve from thread-local storage
    // Validate data freshness
    // Return price data
}
```

#### Health Monitoring
```rust
#[ic_cdk::query]
fn oracle_health_check() -> Result<String, String> {
    // Check system statistics
    // Validate data freshness
    // Verify API connectivity
    // Return health status
}
```

### 2. HTTPS Outcalls Implementation

#### HTTP Request Structure
```rust
let request = CanisterHttpRequestArgument {
    url: api_endpoint,
    method: HttpMethod::GET,
    body: None,
    max_response_bytes: Some(10_000),
    transform: Some(TransformContext::new(transform_commodity_response, vec![])),
    headers: vec![]
};
```

#### Transform Function (Consensus)
```rust
fn transform_commodity_response(args: TransformArgs) -> HttpResponse {
    // Parse JSON response
    // Extract price data
    // Apply data validation
    // Return deterministic response for consensus
}
```

### 3. Configuration Management

#### Oracle Configuration Structure
```rust
pub struct OracleConfig {
    pub enabled_commodities: Vec<String>,
    pub api_endpoints: Vec<(String, String)>,
    pub fetch_interval_seconds: u64,
    pub stale_threshold_seconds: u64,
    pub max_fetch_retries: u32,
    pub confidence_threshold: u64,
    pub rate_limit_per_commodity: u32,
    pub emergency_mode: bool,
    pub backup_prices: Vec<(String, u64)>,
}
```

### 4. Rate Limiting System

#### Implementation Strategy
- **Per-commodity limits**: 24 requests per hour per commodity
- **Global rate tracking**: Sliding window algorithm
- **Exponential backoff**: Automatic retry with increasing delays
- **Emergency override**: Admin bypass for critical situations

#### Rate Limit Logic
```rust
fn check_rate_limit(commodity_id: &str) -> Result<(), String> {
    let now = time();
    let hour_ago = now - (60 * 60 * 1_000_000_000);
    
    // Count requests in last hour
    let recent_requests = RATE_LIMITS.with(|limits| {
        limits.borrow()
            .get(commodity_id)
            .map(|requests| {
                requests.iter()
                    .filter(|&&timestamp| timestamp > hour_ago)
                    .count()
            })
            .unwrap_or(0)
    });
    
    if recent_requests >= 24 {
        return Err("Rate limit exceeded".to_string());
    }
    
    Ok(())
}
```

---

## 🛡️ SECURITY FEATURES

### 1. Admin Access Control
```rust
fn ensure_admin() -> Result<(), String> {
    let caller = ic_cdk::caller();
    ADMIN_PRINCIPALS.with(|admins| {
        if admins.borrow().contains(&caller) {
            Ok(())
        } else {
            Err("Admin access required".to_string())
        }
    })
}
```

### 2. Data Validation
```rust
fn validate_price_data(data: &CommodityPriceData) -> Result<(), String> {
    // Price range validation
    if data.price_per_unit < 100 || data.price_per_unit > 10_000_000 {
        return Err("Price outside valid range".to_string());
    }
    
    // Confidence score validation
    if data.confidence_score < 50 {
        return Err("Confidence score too low".to_string());
    }
    
    // Timestamp validation
    let now = time();
    if data.timestamp > now + (5 * 60 * 1_000_000_000) { // 5 minutes future
        return Err("Timestamp too far in future".to_string());
    }
    
    Ok(())
}
```

### 3. Emergency Protocols
```rust
#[ic_cdk::update]
fn enable_emergency_mode() -> Result<(), String> {
    ensure_admin()?;
    
    ORACLE_CONFIG.with(|config| {
        config.borrow_mut().emergency_mode = true;
    });
    
    log_audit_action(
        ic_cdk::caller(),
        "enable_emergency_mode".to_string(),
        "Emergency mode activated - using backup prices".to_string(),
        true,
    );
    
    Ok(())
}
```

---

## 📊 MONITORING & STATISTICS

### 1. Statistics Collection
```rust
pub struct OracleStatistics {
    pub total_fetches: u64,
    pub successful_fetches: u64,
    pub failed_fetches: u64,
    pub average_response_time: u64,
    pub uptime_percentage: f64,
    pub commodities_tracked: u64,
    pub stale_prices_count: u64,
    pub last_update: u64,
}
```

### 2. Health Monitoring
```rust
fn calculate_health_metrics() -> (f64, u64, String) {
    let stats = get_oracle_statistics_internal();
    
    let uptime = if stats.total_fetches > 0 {
        (stats.successful_fetches as f64 / stats.total_fetches as f64) * 100.0
    } else {
        100.0
    };
    
    let stale_count = count_stale_prices();
    let status = if uptime >= 95.0 && stale_count <= 1 {
        "Healthy".to_string()
    } else {
        "Degraded".to_string()
    };
    
    (uptime, stale_count, status)
}
```

### 3. Real-time Alerts
```rust
pub struct PriceAlert {
    pub commodity_id: String,
    pub threshold_type: AlertType,
    pub threshold_value: u64,
    pub is_active: bool,
    pub created_by: Principal,
    pub created_at: u64,
    pub triggered_at: Option<u64>,
}

pub enum AlertType {
    Above(u64),    // Price above threshold
    Below(u64),    // Price below threshold  
    Change(u64),   // Price change percentage (basis points)
}
```

---

## 🔄 HEARTBEAT AUTOMATION

### 1. Automatic Price Updates
```rust
#[ic_cdk::heartbeat]
async fn heartbeat_price_update() {
    let config = get_oracle_config_internal();
    
    if config.emergency_mode {
        return; // Skip updates in emergency mode
    }
    
    let now = time();
    let update_interval = config.fetch_interval_seconds * 1_000_000_000;
    
    for commodity in &config.enabled_commodities {
        if should_update_price(commodity, now, update_interval) {
            if let Err(e) = fetch_commodity_price_internal(commodity.clone()).await {
                ic_cdk::println!("Heartbeat price update failed for {}: {}", commodity, e);
            }
        }
    }
}
```

### 2. Health Monitoring Automation
```rust
fn should_update_price(commodity: &str, now: u64, interval: u64) -> bool {
    COMMODITY_PRICES.with(|prices| {
        prices.borrow()
            .get(commodity)
            .map(|price_data| {
                now - price_data.timestamp > interval
            })
            .unwrap_or(true) // Update if no price exists
    })
}
```

---

## 🚀 DEPLOYMENT

### 1. Prerequisites
```bash
# Install DFX (Internet Computer SDK)
sh -ci "$(curl -fsSL https://sdk.dfinity.org/install.sh)"

# Install Node.js dependencies
npm install

# Install Rust dependencies  
cargo build
```

### 2. Configuration
```bash
# Configure dfx.json for production
{
  "canisters": {
    "agrilends_backend": {
      "type": "rust",
      "candid": "src/agrilends_backend/agrilends_backend.did",
      "package": "agrilends_backend"
    }
  },
  "networks": {
    "ic": {
      "providers": ["https://ic0.app"],
      "type": "persistent"
    }
  }
}
```

### 3. Production Deployment
```bash
# Deploy to Internet Computer mainnet
dfx deploy --network ic

# Verify deployment
dfx canister status agrilends_backend --network ic

# Initialize Oracle configuration
dfx canister call agrilends_backend update_oracle_config \
  '(record {
    enabled_commodities = vec {"rice"; "corn"; "wheat"; "soybean"; "sugar"};
    fetch_interval_seconds = 3600;
    stale_threshold_seconds = 86400;
    emergency_mode = false;
  })' --network ic
```

### 4. Post-Deployment Verification
```bash
# Test Oracle functionality
dfx canister call agrilends_backend oracle_health_check --network ic

# Test price fetching
dfx canister call agrilends_backend fetch_commodity_price '("rice")' --network ic

# Verify statistics
dfx canister call agrilends_backend get_oracle_statistics --network ic
```

---

## 📋 MAINTENANCE

### 1. Daily Operations
```bash
#!/bin/bash
# Daily Oracle maintenance script

echo "=== Daily Oracle Maintenance ==="

# Health check
dfx canister call agrilends_backend oracle_health_check --network ic

# Update stale prices
for commodity in rice corn wheat soybean sugar; do
    dfx canister call agrilends_backend is_price_stale "(\"$commodity\")" --network ic
    if [ $? -eq 0 ]; then
        dfx canister call agrilends_backend fetch_commodity_price "(\"$commodity\")" --network ic
        sleep 180  # Rate limiting
    fi
done

# Collect statistics
dfx canister call agrilends_backend get_oracle_statistics --network ic
```

### 2. Monitoring Setup
```bash
# Continuous monitoring script
while true; do
    health=$(dfx canister call agrilends_backend oracle_health_check --network ic)
    
    if [[ $health == *"Err"* ]]; then
        echo "ALERT: Oracle health check failed"
        # Send notification (email, Slack, etc.)
    fi
    
    sleep 900  # Check every 15 minutes
done
```

### 3. Emergency Procedures
```bash
# Emergency response script
if [[ $(dfx canister call agrilends_backend oracle_health_check --network ic) == *"Err"* ]]; then
    echo "Enabling emergency mode..."
    dfx canister call agrilends_backend enable_emergency_mode --network ic
    
    # Alert administrators
    # Investigate root cause
    # Plan recovery steps
fi
```

---

## 🔍 TROUBLESHOOTING

### Common Issues

#### 1. Rate Limiting Errors
**Problem**: "Rate limit exceeded" errors
**Solution**: 
- Check rate limit status with diagnostics
- Wait for rate limit window to reset (1 hour)
- Consider emergency mode for critical operations

#### 2. Stale Price Data  
**Problem**: Prices older than threshold
**Solution**:
- Manually trigger price updates
- Check external API availability
- Verify Internet Computer HTTPS outcalls status

#### 3. Low Confidence Scores
**Problem**: Price data quality below threshold
**Solution**:
- Check external API data quality
- Verify transform function logic
- Consider alternative data sources

#### 4. Emergency Mode Activation
**Problem**: System automatically enters emergency mode
**Solution**:
- Check Oracle health diagnostics
- Verify external API connectivity
- Review audit logs for error patterns

### Diagnostic Commands
```bash
# Get detailed diagnostics
dfx canister call agrilends_backend oracle_diagnostics --network ic

# Check specific commodity status
dfx canister call agrilends_backend get_commodity_price '("rice")' --network ic

# Verify rate limits
dfx canister call agrilends_backend get_oracle_statistics --network ic

# Test connectivity
dfx canister call agrilends_backend test_oracle_connectivity --network ic
```

---

## 📚 API REFERENCE

### Query Functions (Read-only)
- `get_commodity_price(commodity_id: String)` - Get cached price data
- `get_all_commodity_prices()` - Get all available prices
- `is_price_stale(commodity_id: String)` - Check if price is outdated
- `get_oracle_statistics()` - Get performance statistics
- `oracle_health_check()` - Check system health
- `get_oracle_config()` - Get current configuration

### Update Functions (State-changing)
- `fetch_commodity_price(commodity_id: String)` - Fetch fresh price
- `admin_set_commodity_price(commodity_id: String, price: u64)` - Manual override
- `update_oracle_config(config: OracleConfig)` - Update configuration
- `enable_emergency_mode()` - Activate emergency mode
- `disable_emergency_mode()` - Deactivate emergency mode
- `create_price_alert(commodity_id: String, alert_type: AlertType)` - Create alert

This implementation guide provides comprehensive coverage of the Agrilends Oracle system for production deployment and maintenance.
```

### OracleConfig
```rust
pub struct OracleConfig {
    pub enabled_commodities: Vec<String>,
    pub api_endpoints: Vec<(String, String)>,
    pub fetch_interval_seconds: u64,
    pub stale_threshold_seconds: u64,
    pub max_fetch_retries: u32,
    pub confidence_threshold: u64,
    pub rate_limit_per_commodity: u32,
    pub emergency_mode: bool,
    pub backup_prices: Vec<(String, u64)>,
}
```

## 🔧 Core Functions

### Public API Functions

#### `fetch_commodity_price(commodity_id: String) -> Result<CommodityPriceData, String>`
- **Type**: Update function
- **Access**: Admin only
- **Purpose**: Fetch latest price from external API
- **Features**: Rate limiting, retry logic, confidence scoring

#### `get_commodity_price(commodity_id: String) -> Result<CommodityPriceData, String>`
- **Type**: Query function
- **Access**: Public
- **Purpose**: Retrieve cached price data
- **Features**: Stale data detection, fast response

#### `admin_set_commodity_price(commodity_id: String, price_idr: u64, source: Option<String>) -> Result<(), String>`
- **Type**: Update function
- **Access**: Admin only
- **Purpose**: Manual price override for emergencies
- **Features**: Audit logging, high confidence scoring

#### `get_oracle_statistics() -> OracleStatistics`
- **Type**: Query function
- **Access**: Public
- **Purpose**: System performance metrics
- **Features**: Real-time stats, health indicators

#### `oracle_health_check() -> Result<String, String>`
- **Type**: Query function
- **Access**: Public
- **Purpose**: System health validation
- **Features**: Multi-point health assessment

### Configuration Functions

#### `configure_oracle(config: OracleConfig) -> Result<(), String>`
- **Type**: Update function
- **Access**: Admin only
- **Purpose**: Update Oracle configuration
- **Features**: Runtime reconfiguration, validation

#### `enable_emergency_mode() -> Result<(), String>`
- **Type**: Update function
- **Access**: Admin only
- **Purpose**: Activate emergency backup pricing
- **Features**: Immediate failover, audit logging

## 📈 API Integration

### Supported APIs
1. **HargaPangan.id** (Primary)
   - Endpoint: `https://api.hargapangan.id/tabel/pasar/provinsi/komoditas/{region}/{commodity}`
   - Coverage: Rice, Corn, Wheat, Soybean, Sugar
   - Update Frequency: Daily

2. **Custom Fallback APIs** (Configurable)
   - Support for additional data sources
   - Automatic failover mechanisms

### HTTP Outcalls Configuration
```rust
const MAX_RESPONSE_BYTES: u64 = 2_000_000;  // 2MB limit
const CYCLES_PER_REQUEST: u64 = 50_000_000; // 50M cycles per request
const MAX_RETRIES: u32 = 3;                 // Maximum retry attempts
```

### Transform Function
The `transform_commodity_response` function ensures deterministic consensus by:
- Removing non-deterministic headers
- Extracting only essential price data
- Standardizing response format
- Validating data integrity

## 🛡️ Security & Reliability

### Rate Limiting
- Maximum 24 requests per hour per commodity
- Exponential backoff on failures
- Cycle cost management

### Data Validation
- Price range validation
- Source verification
- Timestamp validation
- Confidence scoring (0-100)

### Error Handling
- Comprehensive error types
- Graceful degradation
- Audit logging for all failures
- Emergency mode activation

### Emergency Protocols
- Backup price activation
- Manual override capabilities
- System health monitoring
- Administrative alerts

## 📊 Monitoring & Analytics

### Key Metrics
- **Uptime Percentage**: System availability
- **Average Response Time**: Performance indicator
- **Success Rate**: Request reliability
- **Stale Price Count**: Data freshness
- **Confidence Scores**: Data quality

### Health Indicators
- Response time thresholds
- Error rate monitoring
- Stale data detection
- Emergency mode status

### Audit Logging
All Oracle activities are logged with:
- Timestamp and caller identity
- Action performed
- Success/failure status
- Detailed error messages
- Performance metrics

## 🚀 Deployment Guide

### Prerequisites
1. Internet Computer canister with HTTP outcalls enabled
2. Sufficient cycles for API requests (50M+ cycles per request)
3. Admin principal configuration
4. API endpoint access (IPv6 compatible)

### Installation Steps

1. **Deploy Canister**
   ```bash
   dfx deploy agrilends_backend
   ```

2. **Configure Admin Access**
   ```bash
   dfx canister call agrilends_backend add_admin '(principal "your-admin-principal")'
   ```

3. **Initialize Oracle Configuration**
   ```bash
   dfx canister call agrilends_backend configure_oracle '(record {
     enabled_commodities = vec {"rice"; "corn"; "wheat"};
     fetch_interval_seconds = 3600;
     stale_threshold_seconds = 86400;
     emergency_mode = false;
   })'
   ```

4. **Test Price Fetching**
   ```bash
   dfx canister call agrilends_backend fetch_commodity_price '("rice")'
   ```

### Configuration Examples

#### Production Configuration
```bash
dfx canister call agrilends_backend configure_oracle '(record {
  enabled_commodities = vec {"rice"; "corn"; "wheat"; "soybean"; "sugar"};
  fetch_interval_seconds = 3600;
  stale_threshold_seconds = 86400;
  max_fetch_retries = 3;
  confidence_threshold = 70;
  rate_limit_per_commodity = 24;
  emergency_mode = false;
})'
```

#### Emergency Mode Setup
```bash
# Enable emergency mode
dfx canister call agrilends_backend enable_emergency_mode

# Set backup prices
dfx canister call agrilends_backend admin_set_commodity_price '("rice", 15000, opt "emergency_backup")'
```

## 📱 Dashboard Usage

### Access the Dashboard
1. Open `oracle_dashboard.html` in your browser
2. Update the canister ID in the JavaScript configuration
3. Connect to your Internet Computer network

### Dashboard Features
- **Real-time Statistics**: Live system metrics
- **Price Management**: Fetch and override prices
- **Health Monitoring**: System status indicators
- **Emergency Controls**: Crisis management tools
- **Activity Logging**: Real-time operation logs

### Key Operations
1. **Fetch Prices**: Select commodity and click "Fetch Price"
2. **Manual Override**: Set emergency prices manually
3. **Health Check**: Monitor system status
4. **Emergency Mode**: Activate backup pricing
5. **Statistics**: View performance metrics

## 🔄 Integration with Loan System

### Price Valuation
The Oracle integrates with the loan lifecycle system to provide real-time collateral valuation:

```rust
// Example integration in loan_lifecycle.rs
pub async fn calculate_collateral_value(
    commodity_type: &str,
    quantity: u64
) -> Result<u64, String> {
    let price_data = get_commodity_price(commodity_type.to_string())?;
    
    if price_data.is_stale {
        return Err("Price data is stale - cannot value collateral".to_string());
    }
    
    if price_data.confidence_score < CONFIDENCE_THRESHOLD {
        return Err("Price confidence too low for collateral valuation".to_string());
    }
    
    Ok(price_data.price_per_unit * quantity)
}
```

### Loan Application Integration
```rust
// In submit_loan_application function
let collateral_value = calculate_collateral_value(
    &application.commodity_type,
    application.quantity
).await?;

let max_loan_amount = (collateral_value * loan_to_value_ratio) / BASIS_POINTS_SCALE;
```

## 🧪 Testing

### Unit Tests
```bash
cargo test oracle --lib
```

### Integration Tests
```bash
# Test price fetching
dfx canister call agrilends_backend fetch_commodity_price '("rice")'

# Test health check
dfx canister call agrilends_backend oracle_health_check

# Test statistics
dfx canister call agrilends_backend get_oracle_statistics
```

### Load Testing
- Monitor cycle consumption during high-frequency requests
- Test rate limiting mechanisms
- Validate error handling under stress

## 🚨 Emergency Procedures

### System Outage Response
1. **Activate Emergency Mode**
   ```bash
   dfx canister call agrilends_backend enable_emergency_mode
   ```

2. **Set Backup Prices**
   ```bash
   dfx canister call agrilends_backend admin_set_commodity_price '("rice", 15000, opt "emergency")'
   ```

3. **Monitor System Recovery**
   ```bash
   dfx canister call agrilends_backend oracle_health_check
   ```

### Data Quality Issues
1. **Check Confidence Scores**
2. **Verify Source APIs**
3. **Manual Price Override if Needed**
4. **Investigate Root Cause**

### API Endpoint Failures
1. **Monitor Error Logs**
2. **Switch to Backup APIs**
3. **Update Configuration**
4. **Test New Endpoints**

## 📚 Best Practices

### Performance Optimization
- Use query functions for read operations
- Implement efficient caching strategies
- Monitor cycle consumption
- Optimize HTTP request frequency

### Security Guidelines
- Limit admin access to trusted principals
- Validate all input parameters
- Use rate limiting to prevent abuse
- Monitor for unusual activity patterns

### Reliability Standards
- Maintain multiple data sources
- Implement comprehensive error handling
- Use confidence scoring for data quality
- Regular health checks and monitoring

### Maintenance Procedures
- Regular configuration reviews
- API endpoint health monitoring
- Performance metric analysis
- Security audit compliance

## 🔗 Related Documentation

- [Loan Lifecycle Integration](./README_LOAN_REPAYMENT.md)
- [Treasury Management](./TREASURY_MANAGEMENT_IMPLEMENTATION.md)
- [Governance System](./README_GOVERNANCE_IMPLEMENTATION.md)
- [Audit Logging](./AUDIT_LOGGING_INTEGRATION_GUIDE.md)

## 📞 Support & Troubleshooting

### Common Issues
1. **Rate Limit Exceeded**: Wait for rate limit reset or adjust configuration
2. **Stale Data**: Force refresh or check API endpoints
3. **Low Confidence**: Investigate data quality or switch sources
4. **Network Errors**: Check Internet Computer network status

### Debug Commands
```bash
# Check Oracle statistics
dfx canister call agrilends_backend get_oracle_statistics

# Health check
dfx canister call agrilends_backend oracle_health_check

# Get specific price data
dfx canister call agrilends_backend get_commodity_price '("rice")'

# Check configuration
dfx canister call agrilends_backend get_oracle_config
```

### Contact Information
- Technical Support: [Your Support Contact]
- Emergency Response: [Emergency Contact]
- Documentation: [Documentation Link]

---

**Version**: 1.0.0  
**Last Updated**: [Current Date]  
**Status**: Production Ready  
**Compatibility**: Internet Computer Protocol
