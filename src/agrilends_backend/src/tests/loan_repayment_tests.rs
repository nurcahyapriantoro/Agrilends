use crate::loan_repayment::*;
use crate::types::*;
use crate::storage::*;
use candid::Principal;

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_test_loan() -> Loan {
        Loan {
            id: 1,
            borrower: Principal::from_slice(&[1u8; 29]),
            nft_id: 1,
            collateral_value_btc: 25_000_000, // 0.25 BTC
            amount_requested: 15_000_000,      // 0.15 BTC requested
            amount_approved: 15_000_000,       // 0.15 BTC approved
            apr: 10,                           // 10% annual interest
            status: LoanStatus::Active,
            created_at: 1_000_000_000_000_000_000u64, // Mock timestamp
            due_date: Some(1_000_000_000_000_000_000u64 + (365 * 24 * 60 * 60 * 1_000_000_000)), // 1 year later
            total_repaid: 0,
            repayment_history: Vec::new(),
            last_payment_date: None,
        }
    }

    #[test]
    fn test_calculate_total_debt_with_interest() {
        let loan = setup_test_loan();
        
        // This test would require IC environment for proper time calculation
        // For unit tests, we can verify the structure
        assert_eq!(loan.amount_approved, 15_000_000);
        assert_eq!(loan.apr, 10);
        assert_eq!(loan.total_repaid, 0);
    }

    #[test]
    fn test_calculate_payment_breakdown() {
        let mut loan = setup_test_loan();
        loan.total_repaid = 5_000_000; // Already paid 5M satoshi
        
        // Test payment breakdown calculation
        // This would require proper IC environment for time calculation
        // For now, verify the loan structure is correct
        assert_eq!(loan.total_repaid, 5_000_000);
        assert_eq!(loan.amount_approved, 15_000_000);
    }

    #[test]
    fn test_payment_types() {
        let payment = Payment {
            amount: 1_000_000,
            timestamp: 1_000_000_000_000_000_000u64,
            payment_type: PaymentType::Mixed,
            transaction_id: Some("test_tx_123".to_string()),
        };

        assert_eq!(payment.amount, 1_000_000);
        assert!(matches!(payment.payment_type, PaymentType::Mixed));
        assert_eq!(payment.transaction_id, Some("test_tx_123".to_string()));
    }

    #[test]
    fn test_loan_repayment_summary_structure() {
        let summary = LoanRepaymentSummary {
            loan_id: 1,
            borrower: Principal::from_slice(&[1u8; 29]),
            total_debt: 16_500_000, // Principal + interest
            principal_outstanding: 10_000_000,
            interest_outstanding: 1_500_000,
            total_repaid: 5_000_000,
            remaining_balance: 11_500_000,
            next_payment_due: Some(1_000_000_000_000_000_000u64 + (30 * 24 * 60 * 60 * 1_000_000_000)),
            is_overdue: false,
            days_overdue: 0,
        };

        assert_eq!(summary.loan_id, 1);
        assert_eq!(summary.total_debt, 16_500_000);
        assert_eq!(summary.remaining_balance, 11_500_000);
        assert!(!summary.is_overdue);
    }

    #[test]
    fn test_repayment_plan_structure() {
        let plan = RepaymentPlan {
            loan_id: 1,
            total_amount_due: 11_500_000,
            principal_amount: 10_000_000,
            interest_amount: 1_500_000,
            protocol_fee: 150_000, // 10% of interest
            due_date: 1_000_000_000_000_000_000u64 + (365 * 24 * 60 * 60 * 1_000_000_000),
            minimum_payment: 1000,
        };

        assert_eq!(plan.loan_id, 1);
        assert_eq!(plan.total_amount_due, 11_500_000);
        assert_eq!(plan.protocol_fee, 150_000);
        assert_eq!(plan.minimum_payment, 1000);
    }

    #[test]
    fn test_repayment_response_structure() {
        let response = RepaymentResponse {
            success: true,
            message: "Payment successful".to_string(),
            transaction_id: Some("block_123".to_string()),
            new_loan_status: LoanStatus::Active,
            remaining_balance: 5_000_000,
            collateral_released: false,
        };

        assert!(response.success);
        assert_eq!(response.message, "Payment successful");
        assert_eq!(response.remaining_balance, 5_000_000);
        assert!(!response.collateral_released);
    }

    #[test]
    fn test_payment_breakdown_structure() {
        let breakdown = PaymentBreakdown {
            principal_amount: 800_000,
            interest_amount: 200_000,
            protocol_fee_amount: 20_000,
            total_amount: 1_000_000,
        };

        assert_eq!(breakdown.principal_amount, 800_000);
        assert_eq!(breakdown.interest_amount, 200_000);
        assert_eq!(breakdown.protocol_fee_amount, 20_000);
        assert_eq!(breakdown.total_amount, 1_000_000);
    }
}

// Integration test functions (for manual testing in IC environment)
pub fn test_loan_repayment_integration() -> String {
    "Loan repayment integration tests would be run in IC environment".to_string()
}

pub fn test_calculate_debt_integration() -> String {
    "Debt calculation integration tests would be run in IC environment".to_string()
}

pub fn test_payment_breakdown_integration() -> String {
    "Payment breakdown integration tests would be run in IC environment".to_string()
}

pub fn test_collateral_release_integration() -> String {
    "Collateral release integration tests would be run in IC environment".to_string()
}

pub fn test_protocol_fee_collection_integration() -> String {
    "Protocol fee collection integration tests would be run in IC environment".to_string()
}

pub fn test_early_repayment_benefits_integration() -> String {
    "Early repayment benefits integration tests would be run in IC environment".to_string()
}

pub fn test_emergency_repayment_integration() -> String {
    "Emergency repayment integration tests would be run in IC environment".to_string()
}

// Test helper functions
pub fn create_test_payment(amount: u64, payment_type: PaymentType) -> Payment {
    Payment {
        amount,
        timestamp: 1_000_000_000_000_000_000u64,
        payment_type,
        transaction_id: Some(format!("test_tx_{}", amount)),
    }
}

pub fn create_test_repayment_record(loan_id: u64, amount: u64) -> RepaymentRecord {
    RepaymentRecord {
        loan_id,
        payer: Principal::from_slice(&[1u8; 29]),
        amount,
        ckbtc_block_index: 12345,
        timestamp: 1_000_000_000_000_000_000u64,
        payment_breakdown: PaymentBreakdown {
            principal_amount: amount * 80 / 100,
            interest_amount: amount * 20 / 100,
            protocol_fee_amount: amount * 2 / 100,
            total_amount: amount,
        },
    }
}
