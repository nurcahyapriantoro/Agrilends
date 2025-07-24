// Frontend integration example for Loan Repayment Feature
// File: loan_repayment_frontend.js

class LoanRepaymentManager {
    constructor(canister, principal) {
        this.canister = canister;
        this.principal = principal;
    }

    /**
     * Get complete loan repayment information
     */
    async getLoanRepaymentInfo(loanId) {
        try {
            // Get repayment summary
            const summary = await this.canister.get_loan_repayment_summary(loanId);
            
            // Get repayment plan
            const plan = await this.canister.get_repayment_plan(loanId);
            
            // Get payment history
            const history = await this.canister.get_loan_payment_history(loanId);

            return {
                summary: summary.Ok,
                plan: plan.Ok,
                history: history.Ok,
                error: null
            };
        } catch (error) {
            console.error('Failed to get loan repayment info:', error);
            return {
                summary: null,
                plan: null,
                history: null,
                error: error.message
            };
        }
    }

    /**
     * Process loan repayment
     */
    async makeRepayment(loanId, amount) {
        try {
            // First, check repayment eligibility
            const eligibility = await this.canister.check_repayment_eligibility(loanId);
            if (eligibility.Err) {
                throw new Error(eligibility.Err);
            }

            // Get current repayment summary
            const summaryResult = await this.canister.get_loan_repayment_summary(loanId);
            if (summaryResult.Err) {
                throw new Error(summaryResult.Err);
            }

            const summary = summaryResult.Ok;
            
            // Validate payment amount
            if (amount > summary.remaining_balance) {
                throw new Error(`Payment amount ${amount} exceeds remaining balance ${summary.remaining_balance}`);
            }

            if (amount < 1000) { // Minimum payment
                throw new Error('Payment amount must be at least 1000 satoshi');
            }

            // Process the repayment
            const result = await this.canister.repay_loan(loanId, amount);
            
            if (result.Ok) {
                return {
                    success: true,
                    response: result.Ok,
                    error: null
                };
            } else {
                return {
                    success: false,
                    response: null,
                    error: result.Err
                };
            }
        } catch (error) {
            console.error('Repayment failed:', error);
            return {
                success: false,
                response: null,
                error: error.message
            };
        }
    }

    /**
     * Calculate payment breakdown before actual payment
     */
    async calculatePaymentBreakdown(loanId, amount) {
        try {
            // Get loan information for calculation
            const summaryResult = await this.canister.get_loan_repayment_summary(loanId);
            if (summaryResult.Err) {
                throw new Error(summaryResult.Err);
            }

            const summary = summaryResult.Ok;
            
            // Calculate breakdown (simplified client-side calculation)
            // In production, this should be calculated on-chain for accuracy
            const interestPayment = Math.min(amount, summary.interest_outstanding);
            const principalPayment = amount - interestPayment;
            const protocolFee = Math.floor(interestPayment * 0.1); // 10% of interest

            return {
                totalAmount: amount,
                principalAmount: principalPayment,
                interestAmount: interestPayment,
                protocolFeeAmount: protocolFee,
                remainingAfterPayment: summary.remaining_balance - amount
            };
        } catch (error) {
            console.error('Failed to calculate payment breakdown:', error);
            return null;
        }
    }

    /**
     * Check for early repayment benefits
     */
    async checkEarlyRepaymentBenefits(loanId) {
        try {
            const result = await this.canister.calculate_early_repayment_benefits(loanId);
            return result.Ok || 0;
        } catch (error) {
            console.error('Failed to check early repayment benefits:', error);
            return 0;
        }
    }

    /**
     * Format currency values for display
     */
    formatSatoshi(satoshi) {
        return (satoshi / 100_000_000).toFixed(8) + ' ckBTC';
    }

    /**
     * Format timestamp for display
     */
    formatTimestamp(timestamp) {
        return new Date(Number(timestamp) / 1_000_000).toLocaleString();
    }

    /**
     * Create repayment UI component
     */
    createRepaymentUI(containerId, loanId) {
        const container = document.getElementById(containerId);
        if (!container) {
            console.error('Container not found:', containerId);
            return;
        }

        container.innerHTML = `
            <div class="loan-repayment-container">
                <h2>Loan Repayment - Loan #${loanId}</h2>
                
                <div id="loan-summary" class="summary-section">
                    <h3>Loan Summary</h3>
                    <div class="loading">Loading loan information...</div>
                </div>

                <div id="payment-form" class="payment-section" style="display: none;">
                    <h3>Make Payment</h3>
                    <form id="repayment-form">
                        <div class="form-group">
                            <label for="payment-amount">Payment Amount (satoshi):</label>
                            <input type="number" id="payment-amount" min="1000" required>
                            <small>Minimum: 1,000 satoshi</small>
                        </div>
                        
                        <div id="payment-breakdown" class="breakdown-section">
                            <!-- Payment breakdown will be displayed here -->
                        </div>
                        
                        <div class="form-actions">
                            <button type="button" id="calculate-btn">Calculate Breakdown</button>
                            <button type="submit" id="pay-btn" disabled>Make Payment</button>
                        </div>
                    </form>
                </div>

                <div id="payment-history" class="history-section">
                    <h3>Payment History</h3>
                    <div class="loading">Loading payment history...</div>
                </div>

                <div id="message-area" class="message-area">
                    <!-- Messages will be displayed here -->
                </div>
            </div>
        `;

        this.initializeRepaymentUI(loanId);
    }

    /**
     * Initialize repayment UI with data and event handlers
     */
    async initializeRepaymentUI(loanId) {
        try {
            // Load loan repayment information
            const info = await this.getLoanRepaymentInfo(loanId);
            
            if (info.error) {
                this.showMessage('Error loading loan information: ' + info.error, 'error');
                return;
            }

            // Update loan summary
            this.updateLoanSummary(info.summary);
            
            // Update payment history
            this.updatePaymentHistory(info.history);

            // Show payment form if loan is eligible for repayment
            const eligibility = await this.canister.check_repayment_eligibility(loanId);
            if (eligibility.Ok) {
                document.getElementById('payment-form').style.display = 'block';
                this.setupPaymentFormHandlers(loanId);
            } else {
                this.showMessage('This loan is not eligible for repayment: ' + eligibility.Err, 'warning');
            }

        } catch (error) {
            this.showMessage('Failed to initialize repayment UI: ' + error.message, 'error');
        }
    }

    /**
     * Update loan summary display
     */
    updateLoanSummary(summary) {
        const summaryContainer = document.getElementById('loan-summary');
        
        const overdueWarning = summary.is_overdue ? 
            `<div class="overdue-warning">⚠️ This loan is ${summary.days_overdue} days overdue!</div>` : '';

        summaryContainer.innerHTML = `
            <h3>Loan Summary</h3>
            ${overdueWarning}
            <div class="summary-grid">
                <div class="summary-item">
                    <label>Total Debt:</label>
                    <span>${this.formatSatoshi(summary.total_debt)}</span>
                </div>
                <div class="summary-item">
                    <label>Principal Outstanding:</label>
                    <span>${this.formatSatoshi(summary.principal_outstanding)}</span>
                </div>
                <div class="summary-item">
                    <label>Interest Outstanding:</label>
                    <span>${this.formatSatoshi(summary.interest_outstanding)}</span>
                </div>
                <div class="summary-item">
                    <label>Total Repaid:</label>
                    <span>${this.formatSatoshi(summary.total_repaid)}</span>
                </div>
                <div class="summary-item total">
                    <label>Remaining Balance:</label>
                    <span>${this.formatSatoshi(summary.remaining_balance)}</span>
                </div>
                <div class="summary-item">
                    <label>Next Payment Due:</label>
                    <span>${summary.next_payment_due ? this.formatTimestamp(summary.next_payment_due) : 'N/A'}</span>
                </div>
            </div>
        `;
    }

    /**
     * Update payment history display
     */
    updatePaymentHistory(history) {
        const historyContainer = document.getElementById('payment-history');
        
        if (!history || history.length === 0) {
            historyContainer.innerHTML = `
                <h3>Payment History</h3>
                <p>No payments made yet.</p>
            `;
            return;
        }

        const historyHTML = history.map(payment => `
            <div class="payment-record">
                <div class="payment-amount">${this.formatSatoshi(payment.amount)}</div>
                <div class="payment-date">${this.formatTimestamp(payment.timestamp)}</div>
                <div class="payment-type">${payment.payment_type}</div>
                <div class="payment-tx">${payment.transaction_id || 'N/A'}</div>
            </div>
        `).join('');

        historyContainer.innerHTML = `
            <h3>Payment History</h3>
            <div class="payment-history-header">
                <span>Amount</span>
                <span>Date</span>
                <span>Type</span>
                <span>Transaction ID</span>
            </div>
            <div class="payment-history-list">
                ${historyHTML}
            </div>
        `;
    }

    /**
     * Setup payment form event handlers
     */
    setupPaymentFormHandlers(loanId) {
        const paymentAmountInput = document.getElementById('payment-amount');
        const calculateBtn = document.getElementById('calculate-btn');
        const payBtn = document.getElementById('pay-btn');
        const form = document.getElementById('repayment-form');

        // Calculate breakdown when amount changes or calculate button is clicked
        calculateBtn.addEventListener('click', async () => {
            const amount = parseInt(paymentAmountInput.value);
            if (amount > 0) {
                await this.updatePaymentBreakdown(loanId, amount);
                payBtn.disabled = false;
            }
        });

        // Auto-calculate on amount input
        paymentAmountInput.addEventListener('input', async () => {
            const amount = parseInt(paymentAmountInput.value);
            if (amount >= 1000) {
                await this.updatePaymentBreakdown(loanId, amount);
                payBtn.disabled = false;
            } else {
                payBtn.disabled = true;
            }
        });

        // Handle form submission
        form.addEventListener('submit', async (e) => {
            e.preventDefault();
            await this.handlePaymentSubmission(loanId);
        });
    }

    /**
     * Update payment breakdown display
     */
    async updatePaymentBreakdown(loanId, amount) {
        const breakdown = await this.calculatePaymentBreakdown(loanId, amount);
        const breakdownContainer = document.getElementById('payment-breakdown');
        
        if (breakdown) {
            const earlyBenefit = await this.checkEarlyRepaymentBenefits(loanId);
            const earlyBenefitDisplay = earlyBenefit > 0 ? 
                `<div class="early-benefit">🎉 Early payment benefit: ${this.formatSatoshi(earlyBenefit)}</div>` : '';

            breakdownContainer.innerHTML = `
                <h4>Payment Breakdown</h4>
                ${earlyBenefitDisplay}
                <div class="breakdown-grid">
                    <div class="breakdown-item">
                        <label>Principal Payment:</label>
                        <span>${this.formatSatoshi(breakdown.principalAmount)}</span>
                    </div>
                    <div class="breakdown-item">
                        <label>Interest Payment:</label>
                        <span>${this.formatSatoshi(breakdown.interestAmount)}</span>
                    </div>
                    <div class="breakdown-item">
                        <label>Protocol Fee:</label>
                        <span>${this.formatSatoshi(breakdown.protocolFeeAmount)}</span>
                    </div>
                    <div class="breakdown-item total">
                        <label>Total Payment:</label>
                        <span>${this.formatSatoshi(breakdown.totalAmount)}</span>
                    </div>
                    <div class="breakdown-item">
                        <label>Remaining After Payment:</label>
                        <span>${this.formatSatoshi(breakdown.remainingAfterPayment)}</span>
                    </div>
                </div>
            `;
        } else {
            breakdownContainer.innerHTML = '<div class="error">Failed to calculate payment breakdown</div>';
        }
    }

    /**
     * Handle payment form submission
     */
    async handlePaymentSubmission(loanId) {
        const amount = parseInt(document.getElementById('payment-amount').value);
        const payBtn = document.getElementById('pay-btn');
        
        // Disable button and show loading
        payBtn.disabled = true;
        payBtn.textContent = 'Processing Payment...';
        
        try {
            const result = await this.makeRepayment(loanId, amount);
            
            if (result.success) {
                this.showMessage(result.response.message, 'success');
                
                if (result.response.collateral_released) {
                    this.showMessage('🎉 Congratulations! Your collateral NFT has been released back to you!', 'success');
                }
                
                // Refresh the UI
                await this.initializeRepaymentUI(loanId);
            } else {
                this.showMessage('Payment failed: ' + result.error, 'error');
            }
        } catch (error) {
            this.showMessage('Payment error: ' + error.message, 'error');
        } finally {
            payBtn.disabled = false;
            payBtn.textContent = 'Make Payment';
        }
    }

    /**
     * Show message to user
     */
    showMessage(message, type) {
        const messageArea = document.getElementById('message-area');
        const messageDiv = document.createElement('div');
        messageDiv.className = `message ${type}`;
        messageDiv.textContent = message;
        
        messageArea.appendChild(messageDiv);
        
        // Auto-remove after 5 seconds
        setTimeout(() => {
            messageDiv.remove();
        }, 5000);
    }
}

// CSS Styles for the repayment UI
const repaymentCSS = `
.loan-repayment-container {
    max-width: 800px;
    margin: 0 auto;
    padding: 20px;
    font-family: Arial, sans-serif;
}

.summary-section, .payment-section, .history-section {
    background: #f5f5f5;
    padding: 20px;
    margin: 20px 0;
    border-radius: 8px;
}

.summary-grid, .breakdown-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 10px;
    margin-top: 15px;
}

.summary-item, .breakdown-item {
    display: flex;
    justify-content: space-between;
    padding: 8px;
    background: white;
    border-radius: 4px;
}

.summary-item.total, .breakdown-item.total {
    background: #e8f5e8;
    font-weight: bold;
}

.form-group {
    margin: 15px 0;
}

.form-group label {
    display: block;
    margin-bottom: 5px;
    font-weight: bold;
}

.form-group input {
    width: 100%;
    padding: 8px;
    border: 1px solid #ddd;
    border-radius: 4px;
}

.form-actions {
    display: flex;
    gap: 10px;
    margin-top: 20px;
}

.form-actions button {
    padding: 10px 20px;
    border: none;
    border-radius: 4px;
    cursor: pointer;
}

.form-actions button[type="submit"] {
    background: #007bff;
    color: white;
}

.form-actions button[type="button"] {
    background: #6c757d;
    color: white;
}

.form-actions button:disabled {
    background: #ccc;
    cursor: not-allowed;
}

.payment-history-header {
    display: grid;
    grid-template-columns: 1fr 1fr 1fr 1fr;
    gap: 10px;
    font-weight: bold;
    padding: 10px;
    background: #333;
    color: white;
    border-radius: 4px;
}

.payment-record {
    display: grid;
    grid-template-columns: 1fr 1fr 1fr 1fr;
    gap: 10px;
    padding: 10px;
    background: white;
    margin: 5px 0;
    border-radius: 4px;
}

.message {
    padding: 10px;
    margin: 10px 0;
    border-radius: 4px;
}

.message.success {
    background: #d4edda;
    color: #155724;
    border: 1px solid #c3e6cb;
}

.message.error {
    background: #f8d7da;
    color: #721c24;
    border: 1px solid #f5c6cb;
}

.message.warning {
    background: #fff3cd;
    color: #856404;
    border: 1px solid #ffeaa7;
}

.overdue-warning {
    background: #f8d7da;
    color: #721c24;
    padding: 10px;
    border-radius: 4px;
    margin-bottom: 15px;
    text-align: center;
    font-weight: bold;
}

.early-benefit {
    background: #d4edda;
    color: #155724;
    padding: 10px;
    border-radius: 4px;
    margin-bottom: 15px;
    text-align: center;
    font-weight: bold;
}

.loading {
    text-align: center;
    color: #666;
    font-style: italic;
}
`;

// Add CSS to the document
if (typeof document !== 'undefined') {
    const style = document.createElement('style');
    style.textContent = repaymentCSS;
    document.head.appendChild(style);
}

// Usage example
/*
// Initialize the loan repayment manager
const repaymentManager = new LoanRepaymentManager(canister, userPrincipal);

// Create the UI for a specific loan
repaymentManager.createRepaymentUI('repayment-container', loanId);

// Or use individual functions
async function showLoanInfo(loanId) {
    const info = await repaymentManager.getLoanRepaymentInfo(loanId);
    console.log('Loan info:', info);
}

async function makePayment(loanId, amount) {
    const result = await repaymentManager.makeRepayment(loanId, amount);
    if (result.success) {
        console.log('Payment successful!');
    } else {
        console.error('Payment failed:', result.error);
    }
}
*/

export { LoanRepaymentManager };
