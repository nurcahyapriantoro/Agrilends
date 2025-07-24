/**
 * AGRILENDS - LIQUIDATION MANAGEMENT FRONTEND
 * Complete implementation for liquidation system UI
 */

class LiquidationManager {
    constructor(actor) {
        this.actor = actor;
        this.currentUser = null;
        this.isAdmin = false;
    }

    // Initialize liquidation management system
    async init() {
        try {
            this.currentUser = await this.actor.get_caller();
            this.isAdmin = await this.checkAdminStatus();
            this.setupEventListeners();
            await this.loadDashboard();
            console.log('✅ Liquidation Manager initialized successfully');
        } catch (error) {
            console.error('❌ Failed to initialize Liquidation Manager:', error);
            this.showError('Failed to initialize liquidation system');
        }
    }

    // Check if current user is admin
    async checkAdminStatus() {
        try {
            const canisterConfig = await this.actor.get_canister_config();
            return canisterConfig.admins.includes(this.currentUser.toString());
        } catch (error) {
            console.error('Error checking admin status:', error);
            return false;
        }
    }

    // Setup event listeners
    setupEventListeners() {
        // Liquidation trigger button
        document.getElementById('triggerLiquidationBtn')?.addEventListener('click', () => {
            this.showLiquidationModal();
        });

        // Bulk liquidation button
        document.getElementById('bulkLiquidationBtn')?.addEventListener('click', () => {
            this.showBulkLiquidationModal();
        });

        // Emergency liquidation button
        document.getElementById('emergencyLiquidationBtn')?.addEventListener('click', () => {
            this.showEmergencyLiquidationModal();
        });

        // Refresh data button
        document.getElementById('refreshLiquidationData')?.addEventListener('click', () => {
            this.loadDashboard();
        });

        // Modal close buttons
        document.querySelectorAll('.modal .close').forEach(closeBtn => {
            closeBtn.addEventListener('click', (e) => {
                e.target.closest('.modal').style.display = 'none';
            });
        });
    }

    // Load liquidation dashboard
    async loadDashboard() {
        this.showLoading(true);
        
        try {
            // Load all data in parallel
            const [eligibleLoans, statistics, metrics] = await Promise.all([
                this.actor.get_loans_eligible_for_liquidation(),
                this.actor.get_liquidation_statistics(),
                this.isAdmin ? this.actor.get_liquidation_metrics() : null
            ]);

            this.displayEligibleLoans(eligibleLoans);
            this.displayStatistics(statistics);
            
            if (this.isAdmin && metrics?.Ok) {
                this.displayMetrics(metrics.Ok);
            }

            this.updateLastRefresh();
        } catch (error) {
            console.error('Error loading liquidation dashboard:', error);
            this.showError('Failed to load liquidation data');
        } finally {
            this.showLoading(false);
        }
    }

    // Display eligible loans for liquidation
    displayEligibleLoans(eligibleLoans) {
        const container = document.getElementById('eligibleLoansContainer');
        if (!container) return;

        if (eligibleLoans.length === 0) {
            container.innerHTML = `
                <div class="no-data">
                    <i class="fas fa-check-circle"></i>
                    <h3>No Loans Eligible for Liquidation</h3>
                    <p>All loans are current and within grace period.</p>
                </div>
            `;
            return;
        }

        container.innerHTML = `
            <div class="loans-grid">
                ${eligibleLoans.map(loan => this.createEligibleLoanCard(loan)).join('')}
            </div>
        `;
    }

    // Create loan card for eligible loan
    createEligibleLoanCard(loan) {
        const riskLevel = this.calculateRiskLevel(loan.days_overdue, loan.health_ratio);
        const riskClass = riskLevel.toLowerCase().replace(' ', '-');

        return `
            <div class="loan-card ${riskClass}" data-loan-id="${loan.loan_id}">
                <div class="loan-header">
                    <h4>Loan #${loan.loan_id}</h4>
                    <span class="risk-badge ${riskClass}">${riskLevel}</span>
                </div>
                
                <div class="loan-details">
                    <div class="detail-row">
                        <span class="label">Days Overdue:</span>
                        <span class="value danger">${loan.days_overdue} days</span>
                    </div>
                    
                    <div class="detail-row">
                        <span class="label">Health Ratio:</span>
                        <span class="value ${loan.health_ratio < 1.2 ? 'danger' : 'warning'}">
                            ${loan.health_ratio.toFixed(2)}
                        </span>
                    </div>
                    
                    <div class="detail-row">
                        <span class="label">Grace Period:</span>
                        <span class="value ${loan.grace_period_expired ? 'danger' : 'success'}">
                            ${loan.grace_period_expired ? 'Expired' : 'Active'}
                        </span>
                    </div>
                    
                    <div class="detail-row">
                        <span class="label">Status:</span>
                        <span class="value ${loan.is_eligible ? 'danger' : 'warning'}">
                            ${loan.is_eligible ? 'Ready for Liquidation' : 'Under Review'}
                        </span>
                    </div>
                </div>

                ${loan.is_eligible ? `
                    <div class="loan-actions">
                        <button class="btn-danger" onclick="liquidationManager.triggerSingleLiquidation(${loan.loan_id})">
                            <i class="fas fa-exclamation-triangle"></i>
                            Liquidate Now
                        </button>
                        
                        <button class="btn-secondary" onclick="liquidationManager.viewLoanDetails(${loan.loan_id})">
                            <i class="fas fa-eye"></i>
                            View Details
                        </button>
                    </div>
                ` : ''}
            </div>
        `;
    }

    // Calculate risk level based on days overdue and health ratio
    calculateRiskLevel(daysOverdue, healthRatio) {
        if (daysOverdue > 60 || healthRatio < 1.1) return 'Critical Risk';
        if (daysOverdue > 30 || healthRatio < 1.3) return 'High Risk';
        if (daysOverdue > 10 || healthRatio < 1.5) return 'Medium Risk';
        return 'Low Risk';
    }

    // Display liquidation statistics
    displayStatistics(statistics) {
        const container = document.getElementById('liquidationStatistics');
        if (!container) return;

        container.innerHTML = `
            <div class="stats-grid">
                <div class="stat-card">
                    <div class="stat-icon">
                        <i class="fas fa-gavel"></i>
                    </div>
                    <div class="stat-content">
                        <h3>${statistics.total_liquidations}</h3>
                        <p>Total Liquidations</p>
                    </div>
                </div>

                <div class="stat-card">
                    <div class="stat-icon">
                        <i class="fas fa-bitcoin"></i>
                    </div>
                    <div class="stat-content">
                        <h3>${this.formatBTC(statistics.total_liquidated_debt)}</h3>
                        <p>Total Liquidated Debt</p>
                    </div>
                </div>

                <div class="stat-card">
                    <div class="stat-icon">
                        <i class="fas fa-shield-alt"></i>
                    </div>
                    <div class="stat-content">
                        <h3>${this.formatBTC(statistics.total_liquidated_collateral_value)}</h3>
                        <p>Collateral Recovered</p>
                    </div>
                </div>

                <div class="stat-card">
                    <div class="stat-icon">
                        <i class="fas fa-percentage"></i>
                    </div>
                    <div class="stat-content">
                        <h3>${statistics.recovery_rate.toFixed(1)}%</h3>
                        <p>Recovery Rate</p>
                    </div>
                </div>

                <div class="stat-card">
                    <div class="stat-icon">
                        <i class="fas fa-calendar-alt"></i>
                    </div>
                    <div class="stat-content">
                        <h3>${statistics.liquidations_this_month}</h3>
                        <p>This Month</p>
                    </div>
                </div>
            </div>
        `;
    }

    // Display admin metrics
    displayMetrics(metrics) {
        const container = document.getElementById('liquidationMetrics');
        if (!container) return;

        container.innerHTML = `
            <div class="metrics-panel">
                <h3>Admin Metrics</h3>
                
                <div class="metrics-grid">
                    <div class="metric-item">
                        <span class="metric-label">Eligible for Liquidation:</span>
                        <span class="metric-value danger">${metrics.loans_eligible_for_liquidation}</span>
                    </div>
                    
                    <div class="metric-item">
                        <span class="metric-label">Average Recovery:</span>
                        <span class="metric-value">${metrics.recovery_rate.toFixed(1)}%</span>
                    </div>
                    
                    <div class="metric-item">
                        <span class="metric-label">Last Updated:</span>
                        <span class="metric-value">${new Date(Number(metrics.timestamp) / 1000000).toLocaleString()}</span>
                    </div>
                </div>

                ${this.isAdmin ? `
                    <div class="admin-actions">
                        <button class="btn-warning" onclick="liquidationManager.exportLiquidationReport()">
                            <i class="fas fa-download"></i>
                            Export Report
                        </button>
                        
                        <button class="btn-info" onclick="liquidationManager.viewAllRecords()">
                            <i class="fas fa-history"></i>
                            View All Records
                        </button>
                    </div>
                ` : ''}
            </div>
        `;
    }

    // Trigger single liquidation
    async triggerSingleLiquidation(loanId) {
        if (!confirm(`Are you sure you want to liquidate loan #${loanId}? This action cannot be undone.`)) {
            return;
        }

        this.showLoading(true, 'Processing liquidation...');

        try {
            const result = await this.actor.trigger_liquidation(BigInt(loanId));
            
            if (result.Ok) {
                this.showSuccess(`Loan #${loanId} liquidated successfully`);
                await this.loadDashboard(); // Refresh data
            } else {
                this.showError(`Liquidation failed: ${result.Err}`);
            }
        } catch (error) {
            console.error('Liquidation error:', error);
            this.showError('Failed to process liquidation');
        } finally {
            this.showLoading(false);
        }
    }

    // Show liquidation modal
    showLiquidationModal() {
        const modal = document.getElementById('liquidationModal');
        if (!modal) {
            this.createLiquidationModal();
            return;
        }
        modal.style.display = 'block';
    }

    // Create liquidation modal
    createLiquidationModal() {
        const modalHTML = `
            <div id="liquidationModal" class="modal">
                <div class="modal-content">
                    <div class="modal-header">
                        <h2>Liquidate Loan</h2>
                        <span class="close">&times;</span>
                    </div>
                    
                    <div class="modal-body">
                        <form id="liquidationForm">
                            <div class="form-group">
                                <label for="loanIdInput">Loan ID:</label>
                                <input type="number" id="loanIdInput" required min="1">
                                <small>Enter the ID of the loan to liquidate</small>
                            </div>
                            
                            <div class="form-group">
                                <button type="button" id="checkEligibilityBtn" class="btn-secondary">
                                    Check Eligibility
                                </button>
                            </div>
                            
                            <div id="eligibilityResult" class="eligibility-result" style="display: none;"></div>
                            
                            <div class="form-actions">
                                <button type="submit" class="btn-danger" id="confirmLiquidationBtn" disabled>
                                    <i class="fas fa-exclamation-triangle"></i>
                                    Confirm Liquidation
                                </button>
                                <button type="button" class="btn-secondary" onclick="document.getElementById('liquidationModal').style.display='none'">
                                    Cancel
                                </button>
                            </div>
                        </form>
                    </div>
                </div>
            </div>
        `;

        document.body.insertAdjacentHTML('beforeend', modalHTML);
        this.setupLiquidationModalEvents();
        document.getElementById('liquidationModal').style.display = 'block';
    }

    // Setup liquidation modal events
    setupLiquidationModalEvents() {
        document.getElementById('checkEligibilityBtn').addEventListener('click', async () => {
            const loanId = document.getElementById('loanIdInput').value;
            if (!loanId) {
                this.showError('Please enter a loan ID');
                return;
            }

            await this.checkLoanEligibility(parseInt(loanId));
        });

        document.getElementById('liquidationForm').addEventListener('submit', async (e) => {
            e.preventDefault();
            const loanId = document.getElementById('loanIdInput').value;
            document.getElementById('liquidationModal').style.display = 'none';
            await this.triggerSingleLiquidation(parseInt(loanId));
        });
    }

    // Check loan eligibility
    async checkLoanEligibility(loanId) {
        try {
            const result = await this.actor.check_liquidation_eligibility(BigInt(loanId));
            
            if (result.Ok) {
                this.displayEligibilityResult(result.Ok);
            } else {
                this.showError(`Eligibility check failed: ${result.Err}`);
            }
        } catch (error) {
            console.error('Eligibility check error:', error);
            this.showError('Failed to check loan eligibility');
        }
    }

    // Display eligibility result
    displayEligibilityResult(eligibility) {
        const container = document.getElementById('eligibilityResult');
        const confirmBtn = document.getElementById('confirmLiquidationBtn');
        
        container.style.display = 'block';
        container.innerHTML = `
            <div class="eligibility-card ${eligibility.is_eligible ? 'eligible' : 'not-eligible'}">
                <h4>Loan #${eligibility.loan_id} Eligibility</h4>
                
                <div class="eligibility-details">
                    <div class="detail-row">
                        <span class="label">Status:</span>
                        <span class="value ${eligibility.is_eligible ? 'success' : 'danger'}">
                            ${eligibility.is_eligible ? 'Eligible for Liquidation' : 'Not Eligible'}
                        </span>
                    </div>
                    
                    <div class="detail-row">
                        <span class="label">Days Overdue:</span>
                        <span class="value">${eligibility.days_overdue}</span>
                    </div>
                    
                    <div class="detail-row">
                        <span class="label">Health Ratio:</span>
                        <span class="value">${eligibility.health_ratio.toFixed(2)}</span>
                    </div>
                    
                    <div class="detail-row">
                        <span class="label">Grace Period:</span>
                        <span class="value ${eligibility.grace_period_expired ? 'danger' : 'success'}">
                            ${eligibility.grace_period_expired ? 'Expired' : 'Active'}
                        </span>
                    </div>
                </div>
                
                <div class="eligibility-reason">
                    <strong>Reason:</strong> ${eligibility.reason}
                </div>
            </div>
        `;

        confirmBtn.disabled = !eligibility.is_eligible;
    }

    // Show bulk liquidation modal
    showBulkLiquidationModal() {
        if (!this.isAdmin) {
            this.showError('Only administrators can perform bulk liquidations');
            return;
        }

        const modalHTML = `
            <div id="bulkLiquidationModal" class="modal">
                <div class="modal-content large">
                    <div class="modal-header">
                        <h2>Bulk Liquidation</h2>
                        <span class="close">&times;</span>
                    </div>
                    
                    <div class="modal-body">
                        <div id="bulkLiquidationContent">
                            <p>Loading eligible loans...</p>
                        </div>
                    </div>
                </div>
            </div>
        `;

        document.body.insertAdjacentHTML('beforeend', modalHTML);
        document.getElementById('bulkLiquidationModal').style.display = 'block';
        this.loadBulkLiquidationContent();
    }

    // Load bulk liquidation content
    async loadBulkLiquidationContent() {
        try {
            const eligibleLoans = await this.actor.get_loans_eligible_for_liquidation();
            
            const container = document.getElementById('bulkLiquidationContent');
            
            if (eligibleLoans.length === 0) {
                container.innerHTML = `
                    <div class="no-data">
                        <h3>No Loans Eligible for Bulk Liquidation</h3>
                        <p>All loans are current or already processed.</p>
                    </div>
                `;
                return;
            }

            container.innerHTML = `
                <div class="bulk-selection">
                    <div class="selection-header">
                        <label>
                            <input type="checkbox" id="selectAllLoans"> Select All (${eligibleLoans.length} loans)
                        </label>
                        
                        <button class="btn-danger" id="bulkLiquidateBtn" disabled>
                            Liquidate Selected
                        </button>
                    </div>
                    
                    <div class="loans-list">
                        ${eligibleLoans.map(loan => `
                            <label class="loan-checkbox">
                                <input type="checkbox" class="loan-select" value="${loan.loan_id}">
                                <div class="loan-summary">
                                    <strong>Loan #${loan.loan_id}</strong>
                                    <span class="overdue">${loan.days_overdue} days overdue</span>
                                    <span class="health-ratio">Health: ${loan.health_ratio.toFixed(2)}</span>
                                </div>
                            </label>
                        `).join('')}
                    </div>
                </div>
            `;

            this.setupBulkLiquidationEvents();
        } catch (error) {
            console.error('Error loading bulk liquidation content:', error);
            document.getElementById('bulkLiquidationContent').innerHTML = `
                <div class="error">Failed to load eligible loans</div>
            `;
        }
    }

    // Setup bulk liquidation events
    setupBulkLiquidationEvents() {
        const selectAllCheckbox = document.getElementById('selectAllLoans');
        const loanCheckboxes = document.querySelectorAll('.loan-select');
        const bulkBtn = document.getElementById('bulkLiquidateBtn');

        selectAllCheckbox.addEventListener('change', () => {
            loanCheckboxes.forEach(checkbox => {
                checkbox.checked = selectAllCheckbox.checked;
            });
            this.updateBulkLiquidateButton();
        });

        loanCheckboxes.forEach(checkbox => {
            checkbox.addEventListener('change', () => {
                this.updateBulkLiquidateButton();
            });
        });

        bulkBtn.addEventListener('click', () => {
            this.processBulkLiquidation();
        });
    }

    // Update bulk liquidate button state
    updateBulkLiquidateButton() {
        const selectedCount = document.querySelectorAll('.loan-select:checked').length;
        const bulkBtn = document.getElementById('bulkLiquidateBtn');
        
        bulkBtn.disabled = selectedCount === 0;
        bulkBtn.textContent = selectedCount > 0 ? 
            `Liquidate Selected (${selectedCount})` : 'Liquidate Selected';
    }

    // Process bulk liquidation
    async processBulkLiquidation() {
        const selectedLoanIds = Array.from(document.querySelectorAll('.loan-select:checked'))
            .map(checkbox => parseInt(checkbox.value));

        if (selectedLoanIds.length === 0) {
            this.showError('No loans selected for liquidation');
            return;
        }

        if (!confirm(`Are you sure you want to liquidate ${selectedLoanIds.length} loans? This action cannot be undone.`)) {
            return;
        }

        document.getElementById('bulkLiquidationModal').style.display = 'none';
        this.showLoading(true, `Processing ${selectedLoanIds.length} liquidations...`);

        try {
            const loanIdsBigInt = selectedLoanIds.map(id => BigInt(id));
            const results = await this.actor.trigger_bulk_liquidation(loanIdsBigInt);
            
            this.displayBulkLiquidationResults(results);
            await this.loadDashboard(); // Refresh data
        } catch (error) {
            console.error('Bulk liquidation error:', error);
            this.showError('Failed to process bulk liquidation');
        } finally {
            this.showLoading(false);
        }
    }

    // Display bulk liquidation results
    displayBulkLiquidationResults(results) {
        const successful = results.filter(([, result]) => result.Ok).length;
        const failed = results.length - successful;

        let message = `Bulk liquidation completed: ${successful} successful`;
        if (failed > 0) {
            message += `, ${failed} failed`;
        }

        if (failed === 0) {
            this.showSuccess(message);
        } else {
            this.showWarning(message);
        }

        // Show detailed results
        console.log('Bulk liquidation results:', results);
    }

    // Show emergency liquidation modal
    showEmergencyLiquidationModal() {
        if (!this.isAdmin) {
            this.showError('Only administrators can perform emergency liquidations');
            return;
        }

        const modalHTML = `
            <div id="emergencyLiquidationModal" class="modal">
                <div class="modal-content">
                    <div class="modal-header">
                        <h2>Emergency Liquidation</h2>
                        <span class="close">&times;</span>
                    </div>
                    
                    <div class="modal-body">
                        <div class="warning-banner">
                            <i class="fas fa-exclamation-triangle"></i>
                            <strong>Warning:</strong> Emergency liquidation bypasses normal eligibility checks.
                        </div>
                        
                        <form id="emergencyLiquidationForm">
                            <div class="form-group">
                                <label for="emergencyLoanId">Loan ID:</label>
                                <input type="number" id="emergencyLoanId" required min="1">
                            </div>
                            
                            <div class="form-group">
                                <label for="emergencyReason">Emergency Reason:</label>
                                <textarea id="emergencyReason" required rows="4" 
                                    placeholder="Describe the emergency reason for this liquidation..."></textarea>
                            </div>
                            
                            <div class="form-actions">
                                <button type="submit" class="btn-danger">
                                    <i class="fas fa-exclamation-triangle"></i>
                                    Emergency Liquidate
                                </button>
                                <button type="button" class="btn-secondary" 
                                    onclick="document.getElementById('emergencyLiquidationModal').style.display='none'">
                                    Cancel
                                </button>
                            </div>
                        </form>
                    </div>
                </div>
            </div>
        `;

        document.body.insertAdjacentHTML('beforeend', modalHTML);
        
        document.getElementById('emergencyLiquidationForm').addEventListener('submit', async (e) => {
            e.preventDefault();
            await this.processEmergencyLiquidation();
        });

        document.getElementById('emergencyLiquidationModal').style.display = 'block';
    }

    // Process emergency liquidation
    async processEmergencyLiquidation() {
        const loanId = document.getElementById('emergencyLoanId').value;
        const reason = document.getElementById('emergencyReason').value;

        if (!loanId || !reason.trim()) {
            this.showError('Please fill in all fields');
            return;
        }

        if (!confirm(`Confirm emergency liquidation of loan #${loanId}?`)) {
            return;
        }

        document.getElementById('emergencyLiquidationModal').style.display = 'none';
        this.showLoading(true, 'Processing emergency liquidation...');

        try {
            const result = await this.actor.emergency_liquidation(BigInt(loanId), reason);
            
            if (result.Ok) {
                this.showSuccess(`Emergency liquidation of loan #${loanId} completed`);
                await this.loadDashboard();
            } else {
                this.showError(`Emergency liquidation failed: ${result.Err}`);
            }
        } catch (error) {
            console.error('Emergency liquidation error:', error);
            this.showError('Failed to process emergency liquidation');
        } finally {
            this.showLoading(false);
        }
    }

    // View loan details
    async viewLoanDetails(loanId) {
        try {
            const loan = await this.actor.get_loan(BigInt(loanId));
            if (loan) {
                this.showLoanDetailsModal(loan);
            } else {
                this.showError('Loan not found');
            }
        } catch (error) {
            console.error('Error loading loan details:', error);
            this.showError('Failed to load loan details');
        }
    }

    // Show loan details modal
    showLoanDetailsModal(loan) {
        const modalHTML = `
            <div id="loanDetailsModal" class="modal">
                <div class="modal-content large">
                    <div class="modal-header">
                        <h2>Loan #${loan.id} Details</h2>
                        <span class="close">&times;</span>
                    </div>
                    
                    <div class="modal-body">
                        <div class="loan-detail-grid">
                            <div class="detail-section">
                                <h3>Basic Information</h3>
                                <div class="detail-row">
                                    <span class="label">Loan ID:</span>
                                    <span class="value">#${loan.id}</span>
                                </div>
                                <div class="detail-row">
                                    <span class="label">Borrower:</span>
                                    <span class="value">${loan.borrower.toString()}</span>
                                </div>
                                <div class="detail-row">
                                    <span class="label">Status:</span>
                                    <span class="value status-${loan.status.toLowerCase()}">${loan.status}</span>
                                </div>
                                <div class="detail-row">
                                    <span class="label">Created:</span>
                                    <span class="value">${new Date(Number(loan.created_at) / 1000000).toLocaleDateString()}</span>
                                </div>
                            </div>

                            <div class="detail-section">
                                <h3>Financial Details</h3>
                                <div class="detail-row">
                                    <span class="label">Amount Approved:</span>
                                    <span class="value">${this.formatBTC(loan.amount_approved)}</span>
                                </div>
                                <div class="detail-row">
                                    <span class="label">APR:</span>
                                    <span class="value">${loan.apr}%</span>
                                </div>
                                <div class="detail-row">
                                    <span class="label">Total Repaid:</span>
                                    <span class="value">${this.formatBTC(loan.total_repaid)}</span>
                                </div>
                                <div class="detail-row">
                                    <span class="label">Due Date:</span>
                                    <span class="value">${loan.due_date ? new Date(Number(loan.due_date) / 1000000).toLocaleDateString() : 'Not set'}</span>
                                </div>
                            </div>

                            <div class="detail-section">
                                <h3>Collateral Information</h3>
                                <div class="detail-row">
                                    <span class="label">NFT ID:</span>
                                    <span class="value">#${loan.nft_id}</span>
                                </div>
                                <div class="detail-row">
                                    <span class="label">Collateral Value:</span>
                                    <span class="value">${this.formatBTC(loan.collateral_value_btc)}</span>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        `;

        document.body.insertAdjacentHTML('beforeend', modalHTML);
        document.getElementById('loanDetailsModal').style.display = 'block';
    }

    // View all liquidation records
    async viewAllRecords() {
        if (!this.isAdmin) {
            this.showError('Only administrators can view all liquidation records');
            return;
        }

        this.showLoading(true);

        try {
            const result = await this.actor.get_all_liquidation_records();
            
            if (result.Ok) {
                this.showLiquidationRecordsModal(result.Ok);
            } else {
                this.showError(`Failed to load records: ${result.Err}`);
            }
        } catch (error) {
            console.error('Error loading liquidation records:', error);
            this.showError('Failed to load liquidation records');
        } finally {
            this.showLoading(false);
        }
    }

    // Show liquidation records modal
    showLiquidationRecordsModal(records) {
        const modalHTML = `
            <div id="liquidationRecordsModal" class="modal">
                <div class="modal-content extra-large">
                    <div class="modal-header">
                        <h2>All Liquidation Records (${records.length})</h2>
                        <span class="close">&times;</span>
                    </div>
                    
                    <div class="modal-body">
                        ${records.length === 0 ? `
                            <div class="no-data">No liquidation records found</div>
                        ` : `
                            <div class="records-table-container">
                                <table class="records-table">
                                    <thead>
                                        <tr>
                                            <th>Loan ID</th>
                                            <th>Date</th>
                                            <th>Liquidated By</th>
                                            <th>Outstanding Debt</th>
                                            <th>Collateral Value</th>
                                            <th>Reason</th>
                                            <th>Recovery %</th>
                                        </tr>
                                    </thead>
                                    <tbody>
                                        ${records.map(record => `
                                            <tr>
                                                <td>#${record.loan_id}</td>
                                                <td>${new Date(Number(record.liquidated_at) / 1000000).toLocaleDateString()}</td>
                                                <td>${record.liquidated_by.toString().substring(0, 10)}...</td>
                                                <td>${this.formatBTC(record.outstanding_debt)}</td>
                                                <td>${this.formatBTC(record.collateral_value)}</td>
                                                <td>${record.liquidation_reason}</td>
                                                <td>${((record.collateral_value / record.outstanding_debt) * 100).toFixed(1)}%</td>
                                            </tr>
                                        `).join('')}
                                    </tbody>
                                </table>
                            </div>
                        `}
                    </div>
                </div>
            </div>
        `;

        document.body.insertAdjacentHTML('beforeend', modalHTML);
        document.getElementById('liquidationRecordsModal').style.display = 'block';
    }

    // Export liquidation report
    async exportLiquidationReport() {
        if (!this.isAdmin) {
            this.showError('Only administrators can export reports');
            return;
        }

        try {
            const [records, statistics, metrics] = await Promise.all([
                this.actor.get_all_liquidation_records(),
                this.actor.get_liquidation_statistics(),
                this.actor.get_liquidation_metrics()
            ]);

            const reportData = {
                export_date: new Date().toISOString(),
                statistics: statistics,
                metrics: metrics.Ok || null,
                records: records.Ok || []
            };

            this.downloadJSON(reportData, `liquidation-report-${new Date().toISOString().split('T')[0]}.json`);
            this.showSuccess('Liquidation report exported successfully');
        } catch (error) {
            console.error('Error exporting report:', error);
            this.showError('Failed to export liquidation report');
        }
    }

    // Utility functions
    formatBTC(satoshi) {
        return `${(satoshi / 100_000_000).toFixed(8)} BTC`;
    }

    downloadJSON(data, filename) {
        const blob = new Blob([JSON.stringify(data, null, 2)], { type: 'application/json' });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = filename;
        document.body.appendChild(a);
        a.click();
        document.body.removeChild(a);
        URL.revokeObjectURL(url);
    }

    updateLastRefresh() {
        const element = document.getElementById('lastRefresh');
        if (element) {
            element.textContent = `Last updated: ${new Date().toLocaleTimeString()}`;
        }
    }

    showLoading(show, message = 'Loading...') {
        const loader = document.getElementById('loadingIndicator');
        if (loader) {
            loader.style.display = show ? 'block' : 'none';
            if (show && message) {
                loader.querySelector('.loading-text').textContent = message;
            }
        }
    }

    showError(message) {
        this.showNotification(message, 'error');
    }

    showSuccess(message) {
        this.showNotification(message, 'success');
    }

    showWarning(message) {
        this.showNotification(message, 'warning');
    }

    showNotification(message, type) {
        const notification = document.createElement('div');
        notification.className = `notification ${type}`;
        notification.innerHTML = `
            <i class="fas fa-${type === 'error' ? 'exclamation-circle' : type === 'success' ? 'check-circle' : 'exclamation-triangle'}"></i>
            <span>${message}</span>
            <button class="close-notification">&times;</button>
        `;

        document.body.appendChild(notification);

        notification.querySelector('.close-notification').addEventListener('click', () => {
            notification.remove();
        });

        setTimeout(() => {
            if (notification.parentNode) {
                notification.remove();
            }
        }, 5000);
    }
}

// Initialize liquidation manager when page loads
document.addEventListener('DOMContentLoaded', async () => {
    if (typeof window.actor !== 'undefined') {
        window.liquidationManager = new LiquidationManager(window.actor);
        await window.liquidationManager.init();
    } else {
        console.error('Actor not found. Please ensure the canister connection is established.');
    }
});

// Export for global access
window.LiquidationManager = LiquidationManager;
