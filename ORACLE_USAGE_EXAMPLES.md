# AGRILENDS ORACLE USAGE EXAMPLES
## Comprehensive Usage Guide for Oracle System

This document provides practical examples and usage patterns for the Agrilends Oracle system, covering everything from basic price queries to advanced administration tasks.

---

## 🚀 QUICK START

### Basic Price Retrieval
```bash
# Get current rice price
dfx canister call agrilends_backend get_commodity_price '("rice")'

# Result:
# (variant { 
#   Ok = record {
#     price_per_unit = 15_000 : nat64;
#     currency = "IDR";
#     timestamp = 1_672_531_200_000_000_000 : nat64;
#   }
# })
```

### Check if Price is Fresh
```bash
# Check if rice price is stale (older than 24 hours)
dfx canister call agrilends_backend is_price_stale '("rice")'

# Result: (false)
```

---

## 📈 PRICE FETCHING

### Manual Price Updates (Admin Only)
```bash
# Fetch fresh rice price from external API
dfx canister call agrilends_backend fetch_commodity_price '("rice")'

# Expected success response:
# (variant { 
#   Ok = record {
#     price_per_unit = 15_500 : nat64;
#     currency = "IDR";
#     timestamp = 1_672_534_800_000_000_000 : nat64;
#   }
# })

# If rate limited:
# (variant { Err = "Rate limit exceeded. Please wait before fetching again" })
```

### Fetch All Supported Commodities
```bash
# Fetch prices for all enabled commodities
for commodity in rice corn wheat soybean sugar; do
    echo "Fetching $commodity price..."
    dfx canister call agrilends_backend fetch_commodity_price "(\"$commodity\")"
    sleep 65  # Wait to avoid rate limiting
done
```

### Get All Current Prices
```bash
# Retrieve all stored commodity prices
dfx canister call agrilends_backend get_all_commodity_prices

# Result:
# (vec {
#   record {
#     commodity_type = "rice";
#     price_per_unit = 15_000 : nat64;
#     currency = "IDR";
#     timestamp = 1_672_531_200_000_000_000 : nat64;
#     source = "hargapangan.id";
#     confidence_score = 85 : nat64;
#     is_stale = false;
#   };
#   record {
#     commodity_type = "corn";
#     price_per_unit = 8_000 : nat64;
#     ...
#   }
# })
```

---

## ⚙️ CONFIGURATION MANAGEMENT

### View Current Oracle Configuration
```bash
dfx canister call agrilends_backend get_oracle_config

# Result shows enabled commodities, API endpoints, thresholds, etc.
```

### Update Oracle Configuration (Admin Only)
```bash
# Create new configuration
dfx canister call agrilends_backend update_oracle_config '(record {
    enabled_commodities = vec {"rice"; "corn"; "wheat"; "soybean"; "sugar"};
    api_endpoints = vec {
        ("rice", "https://api.hargapangan.id/tabel/pasar/provinsi/komoditas/33/1");
        ("corn", "https://api.hargapangan.id/tabel/pasar/provinsi/komoditas/33/2");
        ("wheat", "https://api.hargapangan.id/tabel/pasar/provinsi/komoditas/33/3");
    };
    fetch_interval_seconds = 3600 : nat64;
    stale_threshold_seconds = 86400 : nat64;
    max_fetch_retries = 3 : nat32;
    confidence_threshold = 70 : nat64;
    rate_limit_per_commodity = 24 : nat32;
    emergency_mode = false;
    backup_prices = vec {
        ("rice", 15000 : nat64);
        ("corn", 8000 : nat64);
        ("wheat", 12000 : nat64);
    };
})'
```

---

## 🎯 ADMIN FUNCTIONS

### Manual Price Override
```bash
# Set rice price manually (emergency/testing)
dfx canister call agrilends_backend admin_set_commodity_price '("rice", 16000 : nat64)'

# Success response:
# (variant { Ok })
```

### Emergency Mode Management
```bash
# Enable emergency mode (uses backup prices)
dfx canister call agrilends_backend enable_emergency_mode

# Check if emergency mode is active
dfx canister call agrilends_backend get_oracle_config
# Look for: emergency_mode = true

# Disable emergency mode
dfx canister call agrilends_backend disable_emergency_mode
```

---

## 📊 MONITORING & STATISTICS

### Oracle Health Check
```bash
# Get comprehensive health status
dfx canister call agrilends_backend oracle_health_check

# Healthy response:
# (variant { Ok = "Oracle system is healthy - Tracking 5 commodities with 98.50% uptime" })

# Unhealthy response:
# (variant { Err = "Health issues detected: Low uptime: 85.20%, 2 stale prices detected" })
```

### Get Oracle Statistics
```bash
dfx canister call agrilends_backend get_oracle_statistics

# Result:
# (record {
#   total_fetches = 150 : nat64;
#   successful_fetches = 142 : nat64;
#   failed_fetches = 8 : nat64;
#   average_response_time = 2_500_000_000 : nat64;  # 2.5 seconds in nanoseconds
#   uptime_percentage = 94.67 : float64;
#   commodities_tracked = 5 : nat64;
#   stale_prices_count = 0 : nat64;
#   last_update = 1_672_534_800_000_000_000 : nat64;
# })
```

### Detailed Diagnostics
```bash
dfx canister call agrilends_backend oracle_diagnostics

# Result:
# "Oracle Diagnostics Report:
#  - Status: Healthy
#  - Total Fetches: 150
#  - Success Rate: 94.67%
#  - Commodities Tracked: 5
#  - Stale Prices: 0
#  - Average Response Time: 2500ms
#  - Enabled Commodities: [\"rice\", \"corn\", \"wheat\", \"soybean\", \"sugar\"]
#  - Emergency Mode: false
#  - Last Update: 3600 seconds ago"
```

---

## 🚨 PRICE ALERTS

### Create Price Alerts
```bash
# Alert when rice price goes above 20,000 IDR
dfx canister call agrilends_backend create_price_alert '(
    "rice",
    variant { Above = 20000 : nat64 }
)'

# Alert when corn price goes below 5,000 IDR
dfx canister call agrilends_backend create_price_alert '(
    "corn", 
    variant { Below = 5000 : nat64 }
)'

# Alert when wheat price changes by more than 10% (1000 basis points)
dfx canister call agrilends_backend create_price_alert '(
    "wheat",
    variant { Change = 1000 : nat64 }
)'
```

### View Active Alerts
```bash
dfx canister call agrilends_backend get_price_alerts

# Result:
# (vec {
#   record {
#     commodity_id = "rice";
#     threshold_type = variant { Above = 20000 : nat64 };
#     threshold_value = 20000 : nat64;
#     is_active = true;
#     created_by = principal "rdmx6-jaaaa-aaaah-qcaiq-cai";
#     created_at = 1_672_531_200_000_000_000 : nat64;
#     triggered_at = null;
#   }
# })
```

---

## 🧪 TESTING & DEBUGGING

### Test Oracle Connectivity
```bash
# Test basic connectivity
dfx canister call agrilends_backend test_oracle_connectivity

# Test with custom URL
dfx canister call agrilends_backend test_oracle_connectivity '(opt "https://httpbin.org/json")'

# Expected response:
# (variant { 
#   Ok = "Connectivity test successful:
#         - URL: https://httpbin.org/json
#         - Status: 200
#         - Response time: 1250ms
#         - Body size: 429 bytes"
# })
```

### Validate Price Data Format
```bash
# Test if JSON format is valid for Oracle parsing
dfx canister call agrilends_backend validate_price_format '(
    "{\"price\": 15000, \"currency\": \"IDR\", \"timestamp\": 1672531200}"
)'

# Valid format response:
# (variant { Ok = "Valid price format - Extracted price: 15000 IDR" })

# Invalid format response:
# (variant { Err = "Invalid price format - Could not extract price data" })
```

### Reset Statistics (Testing)
```bash
# Reset all Oracle statistics (admin only)
dfx canister call agrilends_backend reset_oracle_statistics

# Success response:
# (variant { Ok })
```

## JavaScript/TypeScript Integration

### Frontend Integration
```typescript
import { Actor, HttpAgent } from "@dfinity/agent";
import { agrilends_backend } from "./declarations/agrilends_backend";

// Initialize agent and actor
const agent = new HttpAgent({
  host: process.env.DFX_NETWORK === "local" ? "http://localhost:8080" : "https://ic0.app"
});

if (process.env.DFX_NETWORK === "local") {
  await agent.fetchRootKey();
}

const actor = Actor.createActor(agrilends_backend, {
  agent,
  canisterId: process.env.AGRILENDS_BACKEND_CANISTER_ID,
});

// Fetch commodity price
async function fetchRicePrice() {
  try {
    const result = await actor.fetch_commodity_price("rice");
    console.log("Rice price updated:", result);
    return result;
  } catch (error) {
    console.error("Failed to fetch rice price:", error);
    throw error;
  }
}

// Get current price (cached)
async function getCurrentPrice(commodity: string) {
  try {
    const priceData = await actor.get_commodity_price(commodity);
    return {
      commodity: priceData.commodity_type,
      price: priceData.price_per_unit,
      currency: priceData.currency,
      isStale: priceData.is_stale,
      confidence: priceData.confidence_score,
      lastUpdated: new Date(Number(priceData.timestamp) / 1000000),
      source: priceData.source
    };
  } catch (error) {
    console.error(`Failed to get ${commodity} price:`, error);
    return null;
  }
}

// Monitor Oracle health
async function monitorOracleHealth() {
  try {
    const stats = await actor.get_oracle_statistics();
    const health = await actor.oracle_health_check();
    
    return {
      statistics: {
        totalFetches: stats.total_fetches,
        successRate: stats.uptime_percentage,
        averageResponseTime: Number(stats.average_response_time) / 1000000, // Convert to ms
        commoditiesTracked: stats.commodities_tracked,
        stalePrices: stats.stale_prices_count
      },
      healthStatus: health,
      isHealthy: health.includes("healthy")
    };
  } catch (error) {
    console.error("Health check failed:", error);
    return { isHealthy: false, error: error.message };
  }
}

// Real-time price dashboard
class OracleDashboard {
  private actor: any;
  private updateInterval: number = 30000; // 30 seconds
  private intervalId?: NodeJS.Timeout;

  constructor(actor: any) {
    this.actor = actor;
  }

  async startMonitoring() {
    await this.updateDashboard();
    this.intervalId = setInterval(() => {
      this.updateDashboard();
    }, this.updateInterval);
  }

  stopMonitoring() {
    if (this.intervalId) {
      clearInterval(this.intervalId);
    }
  }

  private async updateDashboard() {
    try {
      const [prices, statistics, health] = await Promise.all([
        this.actor.get_all_commodity_prices(),
        this.actor.get_oracle_statistics(),
        this.actor.oracle_health_check()
      ]);

      this.displayPrices(prices);
      this.displayStatistics(statistics);
      this.displayHealthStatus(health);
    } catch (error) {
      console.error("Dashboard update failed:", error);
    }
  }

  private displayPrices(prices: any[]) {
    prices.forEach(price => {
      const element = document.getElementById(`price-${price.commodity_type}`);
      if (element) {
        element.innerHTML = `
          <div class="price-card ${price.is_stale ? 'stale' : 'fresh'}">
            <h3>${price.commodity_type.toUpperCase()}</h3>
            <div class="price">IDR ${price.price_per_unit.toLocaleString()}</div>
            <div class="confidence">Confidence: ${price.confidence_score}%</div>
            <div class="timestamp">Updated: ${new Date(Number(price.timestamp) / 1000000).toLocaleString()}</div>
          </div>
        `;
      }
    });
  }

  private displayStatistics(stats: any) {
    document.getElementById('total-fetches')!.textContent = stats.total_fetches.toString();
    document.getElementById('success-rate')!.textContent = `${Math.round(stats.uptime_percentage)}%`;
    document.getElementById('avg-response-time')!.textContent = 
      `${Math.round(Number(stats.average_response_time) / 1000000)}ms`;
    document.getElementById('stale-count')!.textContent = stats.stale_prices_count.toString();
  }

  private displayHealthStatus(health: any) {
    const statusElement = document.getElementById('health-status')!;
    const isHealthy = typeof health === 'string' && health.includes('healthy');
    statusElement.className = `health-indicator ${isHealthy ? 'healthy' : 'unhealthy'}`;
    statusElement.textContent = isHealthy ? '🟢 Healthy' : '🔴 Issues Detected';
  }
}

// Usage example
const dashboard = new OracleDashboard(actor);
dashboard.startMonitoring();
```

### React Component Example
```tsx
import React, { useState, useEffect } from 'react';
import { agrilends_backend } from '../declarations/agrilends_backend';

interface PriceData {
  commodity_type: string;
  price_per_unit: bigint;
  currency: string;
  timestamp: bigint;
  confidence_score: bigint;
  is_stale: boolean;
  source: string;
}

const OracleComponent: React.FC = () => {
  const [prices, setPrices] = useState<PriceData[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const fetchPrices = async () => {
    setLoading(true);
    setError(null);
    try {
      const result = await agrilends_backend.get_all_commodity_prices();
      setPrices(result);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to fetch prices');
    } finally {
      setLoading(false);
    }
  };

  const refreshPrice = async (commodity: string) => {
    try {
      await agrilends_backend.fetch_commodity_price(commodity);
      await fetchPrices(); // Refresh all prices
    } catch (err) {
      setError(`Failed to refresh ${commodity} price`);
    }
  };

  useEffect(() => {
    fetchPrices();
    const interval = setInterval(fetchPrices, 60000); // Update every minute
    return () => clearInterval(interval);
  }, []);

  if (loading && prices.length === 0) {
    return <div className="loading">Loading prices...</div>;
  }

  return (
    <div className="oracle-component">
      <h2>Commodity Prices</h2>
      {error && <div className="error">{error}</div>}
      
      <div className="prices-grid">
        {prices.map((price) => (
          <div key={price.commodity_type} className={`price-card ${price.is_stale ? 'stale' : 'fresh'}`}>
            <div className="commodity-name">{price.commodity_type.toUpperCase()}</div>
            <div className="price-value">
              IDR {Number(price.price_per_unit).toLocaleString()}
            </div>
            <div className="price-meta">
              <span className="confidence">
                Confidence: {Number(price.confidence_score)}%
              </span>
              <span className="source">Source: {price.source}</span>
              <span className="timestamp">
                {new Date(Number(price.timestamp) / 1000000).toLocaleString()}
              </span>
            </div>
            <button 
              onClick={() => refreshPrice(price.commodity_type)}
              className="refresh-btn"
            >
              🔄 Refresh
            </button>
          </div>
        ))}
      </div>
      
      <button onClick={fetchPrices} className="refresh-all-btn">
        Refresh All Prices
      </button>
    </div>
  );
};

export default OracleComponent;
```

## Python Integration (for Backend Services)

### Using IC-Py Agent
```python
from ic.client import Client
from ic.identity import Identity
from ic.agent import Agent
from ic.candid import encode, decode
import asyncio

class AgrilendsOracle:
    def __init__(self, canister_id: str, identity: Identity, ic_url: str = "https://ic0.app"):
        self.client = Client(url=ic_url)
        self.agent = Agent(identity, self.client)
        self.canister_id = canister_id

    async def get_commodity_price(self, commodity: str):
        """Get cached commodity price"""
        try:
            response = await self.agent.query_raw(
                self.canister_id,
                "get_commodity_price",
                encode([{"type": "text", "value": commodity}])
            )
            return decode(response)
        except Exception as e:
            print(f"Error fetching {commodity} price: {e}")
            return None

    async def fetch_commodity_price(self, commodity: str):
        """Trigger fresh price fetch"""
        try:
            response = await self.agent.update_raw(
                self.canister_id,
                "fetch_commodity_price",
                encode([{"type": "text", "value": commodity}])
            )
            return decode(response)
        except Exception as e:
            print(f"Error fetching {commodity} price: {e}")
            return None

    async def get_oracle_statistics(self):
        """Get Oracle performance statistics"""
        try:
            response = await self.agent.query_raw(
                self.canister_id,
                "get_oracle_statistics",
                encode([])
            )
            return decode(response)
        except Exception as e:
            print(f"Error getting Oracle statistics: {e}")
            return None

    async def monitor_prices(self, commodities: list, callback):
        """Monitor commodity prices and call callback on changes"""
        last_prices = {}
        
        while True:
            for commodity in commodities:
                current_price = await self.get_commodity_price(commodity)
                if current_price and commodity not in last_prices:
                    last_prices[commodity] = current_price
                    callback(commodity, current_price, "new")
                elif current_price and current_price != last_prices.get(commodity):
                    callback(commodity, current_price, "updated")
                    last_prices[commodity] = current_price
            
            await asyncio.sleep(30)  # Check every 30 seconds

# Usage example
async def price_change_callback(commodity: str, price_data: dict, change_type: str):
    print(f"{change_type.upper()}: {commodity} price is now IDR {price_data['price_per_unit']:,}")
    
    # Send notification or update database
    if change_type == "updated":
        # Calculate percentage change
        # Send alert if significant change
        pass

async def main():
    # Initialize Oracle client
    identity = Identity()  # Load your identity
    oracle = AgrilendsOracle("your-canister-id", identity)
    
    # Get current prices
    rice_price = await oracle.get_commodity_price("rice")
    print(f"Rice price: IDR {rice_price['price_per_unit']:,}")
    
    # Get statistics
    stats = await oracle.get_oracle_statistics()
    print(f"Oracle uptime: {stats['uptime_percentage']:.2f}%")
    
    # Start monitoring (runs indefinitely)
    await oracle.monitor_prices(
        ["rice", "corn", "wheat"],
        price_change_callback
    )

if __name__ == "__main__":
    asyncio.run(main())
```

## Integration Examples

### Loan Application Integration
```rust
// In loan_lifecycle.rs
use crate::oracle::{get_commodity_price};

pub async fn submit_loan_application(
    application: LoanApplication
) -> Result<Loan, String> {
    // Get current commodity price for collateral valuation
    let price_data = get_commodity_price(application.commodity_type.clone())?;
    
    // Validate price data quality
    if price_data.is_stale {
        return Err("Cannot process loan - commodity price data is stale".to_string());
    }
    
    if price_data.confidence_score < 70 {
        return Err("Cannot process loan - commodity price confidence too low".to_string());
    }
    
    // Calculate collateral value
    let collateral_value_idr = price_data.price_per_unit * application.quantity;
    
    // Convert to BTC equivalent (assuming 1 BTC = 700,000,000 IDR)
    let btc_rate = 700_000_000u64; // This would come from another Oracle
    let collateral_value_btc = (collateral_value_idr * 100_000_000) / btc_rate; // Convert to satoshi
    
    // Calculate maximum loan amount (60% LTV)
    let max_loan_amount = (collateral_value_btc * 6000) / BASIS_POINTS_SCALE;
    
    if application.amount_requested > max_loan_amount {
        return Err(format!(
            "Requested amount {} exceeds maximum {} based on collateral value", 
            application.amount_requested, 
            max_loan_amount
        ));
    }
    
    // Create loan with real-time valuation
    let loan = Loan {
        id: next_loan_id(),
        borrower: caller(),
        nft_id: application.nft_id,
        collateral_value_btc,
        amount_requested: application.amount_requested,
        amount_approved: application.amount_requested, // Could be less based on risk assessment
        apr: 10, // 10% annual rate
        status: LoanStatus::PendingApproval,
        created_at: time(),
        due_date: Some(time() + (365 * 24 * 60 * 60 * 1_000_000_000)), // 1 year
        total_repaid: 0,
        repayment_history: vec![],
        last_payment_date: None,
    };
    
    store_loan(loan.clone())?;
    
    log_audit_action(
        caller(),
        "submit_loan_application".to_string(),
        format!(
            "Loan application submitted for {} with collateral value {} satoshi (price: {} IDR/unit, confidence: {}%)",
            application.commodity_type,
            collateral_value_btc,
            price_data.price_per_unit,
            price_data.confidence_score
        ),
        true,
    );
    
    Ok(loan)
}
```

### Liquidation Trigger Integration
```rust
// In liquidation.rs
use crate::oracle::{get_commodity_price};

pub async fn check_liquidation_eligibility(loan_id: u64) -> Result<bool, String> {
    let loan = get_loan_by_id(loan_id)
        .ok_or("Loan not found")?;
    
    // Get current NFT metadata to determine commodity type
    let nft = get_nft_by_token_id(loan.nft_id)
        .ok_or("NFT not found")?;
    
    let commodity_type = extract_commodity_type_from_nft(&nft)?;
    
    // Get current market price
    let current_price_data = get_commodity_price(commodity_type)?;
    
    if current_price_data.is_stale {
        return Err("Cannot check liquidation - price data is stale".to_string());
    }
    
    // Recalculate collateral value with current prices
    let commodity_quantity = extract_quantity_from_nft(&nft)?;
    let current_collateral_value_idr = current_price_data.price_per_unit * commodity_quantity;
    
    // Convert to BTC
    let btc_rate = 700_000_000u64; // This should also come from Oracle
    let current_collateral_value_btc = (current_collateral_value_idr * 100_000_000) / btc_rate;
    
    // Calculate current LTV ratio
    let outstanding_balance = calculate_outstanding_balance(&loan)?;
    let current_ltv = (outstanding_balance * BASIS_POINTS_SCALE) / current_collateral_value_btc;
    
    // Liquidation threshold is 80%
    let liquidation_threshold = 8000; // 80% in basis points
    
    if current_ltv > liquidation_threshold {
        log_audit_action(
            ic_cdk::api::id(),
            "liquidation_eligible".to_string(),
            format!(
                "Loan {} eligible for liquidation - LTV: {}%, threshold: {}%",
                loan_id,
                current_ltv / 100,
                liquidation_threshold / 100
            ),
            true,
        );
        
        return Ok(true);
    }
    
    Ok(false)
}
```

### Price Alert System Integration
```typescript
// Real-time price monitoring service
class PriceAlertService {
  private actor: any;
  private alerts: Map<string, PriceAlert[]> = new Map();
  private monitoringInterval?: NodeJS.Timeout;

  constructor(actor: any) {
    this.actor = actor;
  }

  addAlert(commodity: string, alertConfig: PriceAlert) {
    if (!this.alerts.has(commodity)) {
      this.alerts.set(commodity, []);
    }
    this.alerts.get(commodity)!.push(alertConfig);
  }

  startMonitoring() {
    this.monitoringInterval = setInterval(async () => {
      await this.checkAlerts();
    }, 60000); // Check every minute
  }

  stopMonitoring() {
    if (this.monitoringInterval) {
      clearInterval(this.monitoringInterval);
    }
  }

  private async checkAlerts() {
    for (const [commodity, alerts] of this.alerts.entries()) {
      try {
        const priceData = await this.actor.get_commodity_price(commodity);
        const currentPrice = Number(priceData.price_per_unit);

        for (const alert of alerts) {
          if (this.shouldTriggerAlert(alert, currentPrice, priceData)) {
            await this.triggerAlert(commodity, currentPrice, alert);
          }
        }
      } catch (error) {
        console.error(`Failed to check alerts for ${commodity}:`, error);
      }
    }
  }

  private shouldTriggerAlert(alert: PriceAlert, currentPrice: number, priceData: any): boolean {
    switch (alert.type) {
      case 'price_above':
        return currentPrice > alert.threshold;
      case 'price_below':
        return currentPrice < alert.threshold;
      case 'confidence_below':
        return Number(priceData.confidence_score) < alert.threshold;
      case 'data_stale':
        return priceData.is_stale;
      default:
        return false;
    }
  }

  private async triggerAlert(commodity: string, price: number, alert: PriceAlert) {
    const alertMessage = {
      commodity,
      currentPrice: price,
      alertType: alert.type,
      threshold: alert.threshold,
      timestamp: new Date(),
      severity: alert.severity || 'medium'
    };

    // Send notification (email, webhook, etc.)
    await this.sendNotification(alertMessage);
    
    // Log alert
    console.log(`ALERT: ${commodity} price ${price} triggered ${alert.type} alert`);
  }

  private async sendNotification(alert: any) {
    // Implementation depends on your notification system
    // Could be email, Slack, webhook, etc.
    
    if (alert.severity === 'high') {
      // Send immediate notification
      await this.sendEmailAlert(alert);
    }
    
    // Always log to monitoring system
    await this.logToMonitoring(alert);
  }

  private async sendEmailAlert(alert: any) {
    // Email implementation
  }

  private async logToMonitoring(alert: any) {
    // Monitoring system integration
  }
}

interface PriceAlert {
  type: 'price_above' | 'price_below' | 'confidence_below' | 'data_stale';
  threshold: number;
  severity?: 'low' | 'medium' | 'high';
  callback?: (alert: any) => void;
}

// Usage
const alertService = new PriceAlertService(actor);

// Add various alerts
alertService.addAlert('rice', {
  type: 'price_above',
  threshold: 20000,
  severity: 'high'
});

alertService.addAlert('rice', {
  type: 'confidence_below',
  threshold: 60,
  severity: 'medium'
});

alertService.startMonitoring();
```

## Error Handling Examples

### Robust Error Handling Pattern
```typescript
class OracleErrorHandler {
  static async withRetry<T>(
    operation: () => Promise<T>,
    maxRetries: number = 3,
    delay: number = 1000
  ): Promise<T> {
    let lastError: Error;
    
    for (let attempt = 1; attempt <= maxRetries; attempt++) {
      try {
        return await operation();
      } catch (error) {
        lastError = error as Error;
        
        if (attempt === maxRetries) {
          throw lastError;
        }
        
        // Exponential backoff
        await new Promise(resolve => setTimeout(resolve, delay * Math.pow(2, attempt - 1)));
      }
    }
    
    throw lastError!;
  }

  static handleOracleError(error: any): { message: string; shouldRetry: boolean; severity: 'low' | 'medium' | 'high' } {
    if (typeof error === 'string') {
      if (error.includes('Rate limit exceeded')) {
        return {
          message: 'API rate limit reached. Please wait before retrying.',
          shouldRetry: true,
          severity: 'medium'
        };
      }
      
      if (error.includes('Price not available')) {
        return {
          message: 'Price data unavailable for this commodity.',
          shouldRetry: false,
          severity: 'high'
        };
      }
      
      if (error.includes('confidence too low')) {
        return {
          message: 'Price data quality insufficient for operation.',
          shouldRetry: true,
          severity: 'medium'
        };
      }
      
      if (error.includes('stale')) {
        return {
          message: 'Price data is outdated. Triggering refresh.',
          shouldRetry: true,
          severity: 'medium'
        };
      }
    }
    
    return {
      message: 'Unknown Oracle error occurred.',
      shouldRetry: false,
      severity: 'high'
    };
  }
}

// Usage example
async function safeGetPrice(commodity: string) {
  try {
    return await OracleErrorHandler.withRetry(async () => {
      const price = await actor.get_commodity_price(commodity);
      
      if (price.is_stale) {
        // Try to fetch fresh data
        await actor.fetch_commodity_price(commodity);
        return await actor.get_commodity_price(commodity);
      }
      
      return price;
    });
  } catch (error) {
    const errorInfo = OracleErrorHandler.handleOracleError(error);
    console.error(`Oracle error (${errorInfo.severity}): ${errorInfo.message}`);
    
    if (errorInfo.severity === 'high') {
      // Send alert to administrators
      await sendAdminAlert(errorInfo);
    }
    
    throw error;
  }
}
```

These examples demonstrate comprehensive integration patterns for the Agrilends Oracle system across different platforms and use cases. The implementation provides robust error handling, real-time monitoring, and seamless integration with the loan lifecycle system.
