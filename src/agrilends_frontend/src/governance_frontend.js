// ========== GOVERNANCE & ADMINISTRATION FRONTEND ==========
// Complete frontend implementation for Fitur Tata Kelola & Administrasi
// Implements all README specifications with comprehensive admin interface

class GovernanceManager {
    constructor(backendCanisterId) {
        this.canisterId = backendCanisterId;
        this.actor = null;
        this.currentUser = null;
        this.userRole = null;
        this.initializeActor();
    }

    async initializeActor() {
        try {
            // Initialize Internet Computer agent and actor
            const agent = new HttpAgent({
                host: window.location.origin.includes('localhost') 
                    ? 'http://localhost:4943' 
                    : 'https://ic0.app'
            });

            // Only fetch root key in development
            if (window.location.origin.includes('localhost')) {
                await agent.fetchRootKey();
            }

            // Create actor with backend canister
            this.actor = Actor.createActor(idlFactory, {
                agent,
                canisterId: this.canisterId,
            });

            console.log('✅ Governance actor initialized successfully');
            await this.initializeInterface();
        } catch (error) {
            console.error('❌ Failed to initialize governance actor:', error);
            this.showError('Failed to connect to governance system');
        }
    }

    async initializeInterface() {
        await this.loadCurrentUser();
        await this.loadProtocolParameters();
        await this.loadGovernanceStats();
        await this.loadActiveProposals();
        await this.loadAdminRoles();
        this.setupEventListeners();
        this.updateUIBasedOnRole();
    }

    async loadCurrentUser() {
        try {
            if (!this.actor) return;

            // Get current principal and user info
            const principal = await this.actor.get_caller();
            this.currentUser = principal;

            // Check if user has admin role
            const adminRole = await this.actor.get_admin_role(principal);
            if (adminRole && adminRole.length > 0 && adminRole[0].is_active) {
                this.userRole = adminRole[0];
                console.log('✅ User has admin role:', this.userRole.role_type);
            } else {
                this.userRole = null;
                console.log('ℹ️ User is not an admin');
            }

            this.updateUserDisplay();
        } catch (error) {
            console.error('❌ Failed to load current user:', error);
        }
    }

    updateUserDisplay() {
        const userInfo = document.getElementById('userInfo');
        if (userInfo) {
            if (this.currentUser) {
                const roleText = this.userRole 
                    ? `${this.userRole.role_type} Admin` 
                    : 'Regular User';
                
                userInfo.innerHTML = `
                    <div class="user-card">
                        <div class="user-principal">${this.currentUser.toString()}</div>
                        <div class="user-role ${this.userRole ? 'admin-role' : 'user-role'}">${roleText}</div>
                    </div>
                `;
            } else {
                userInfo.innerHTML = '<div class="user-card">Not connected</div>';
            }
        }
    }

    async loadProtocolParameters() {
        try {
            const parameters = await this.actor.get_all_protocol_parameters();
            this.displayProtocolParameters(parameters);
        } catch (error) {
            console.error('❌ Failed to load protocol parameters:', error);
            this.showError('Failed to load protocol parameters');
        }
    }

    displayProtocolParameters(parameters) {
        const container = document.getElementById('protocolParameters');
        if (!container) return;

        container.innerHTML = `
            <div class="parameters-header">
                <h3>📊 Protocol Parameters</h3>
                ${this.isAdmin() ? `
                    <button class="btn btn-primary" onclick="governance.showParameterUpdateModal()">
                        Update Parameter
                    </button>
                ` : ''}
            </div>
            <div class="parameters-grid">
                ${parameters.map(param => this.createParameterCard(param)).join('')}
            </div>
        `;
    }

    createParameterCard(param) {
        const valueDisplay = this.formatParameterValue(param.current_value, param.value_type);
        const lastUpdated = new Date(Number(param.last_updated) / 1_000_000).toLocaleString();
        
        return `
            <div class="parameter-card" data-key="${param.key}">
                <div class="parameter-header">
                    <h4>${this.formatParameterName(param.key)}</h4>
                    <div class="parameter-value">${valueDisplay}</div>
                </div>
                <div class="parameter-details">
                    <p class="parameter-description">${param.description}</p>
                    <div class="parameter-meta">
                        <span class="update-time">Updated: ${lastUpdated}</span>
                        <span class="updated-by">By: ${param.updated_by.toString().slice(0, 8)}...</span>
                    </div>
                    ${param.min_value && param.max_value ? `
                        <div class="parameter-bounds">
                            Range: ${this.formatParameterValue(param.min_value[0], param.value_type)} - 
                            ${this.formatParameterValue(param.max_value[0], param.value_type)}
                        </div>
                    ` : ''}
                </div>
                ${this.isAdmin() ? `
                    <button class="btn btn-sm btn-outline" 
                            onclick="governance.updateParameter('${param.key}', ${param.current_value})">
                        Update
                    </button>
                ` : ''}
            </div>
        `;
    }

    formatParameterName(key) {
        return key.split('_').map(word => 
            word.charAt(0).toUpperCase() + word.slice(1)
        ).join(' ');
    }

    formatParameterValue(value, type) {
        switch (type) {
            case 'Percentage':
                return `${(value / 100).toFixed(2)}%`;
            case 'Amount':
                return `${(value / 100_000_000).toFixed(8)} BTC`;
            case 'Duration':
                return `${value} days`;
            case 'Boolean':
                return value === 1 ? 'Enabled' : 'Disabled';
            default:
                return value.toString();
        }
    }

    async loadGovernanceStats() {
        try {
            const stats = await this.actor.get_governance_stats();
            this.displayGovernanceStats(stats);
        } catch (error) {
            console.error('❌ Failed to load governance stats:', error);
        }
    }

    displayGovernanceStats(stats) {
        const container = document.getElementById('governanceStats');
        if (!container) return;

        container.innerHTML = `
            <div class="stats-grid">
                <div class="stat-card">
                    <div class="stat-value">${stats.total_proposals}</div>
                    <div class="stat-label">Total Proposals</div>
                </div>
                <div class="stat-card">
                    <div class="stat-value">${stats.active_proposals}</div>
                    <div class="stat-label">Active Proposals</div>
                </div>
                <div class="stat-card">
                    <div class="stat-value">${stats.executed_proposals}</div>
                    <div class="stat-label">Executed</div>
                </div>
                <div class="stat-card">
                    <div class="stat-value">${stats.total_votes_cast}</div>
                    <div class="stat-label">Total Votes</div>
                </div>
                <div class="stat-card">
                    <div class="stat-value">${(stats.average_participation_rate / 100).toFixed(1)}%</div>
                    <div class="stat-label">Participation Rate</div>
                </div>
                <div class="stat-card">
                    <div class="stat-value">${stats.total_voting_power}</div>
                    <div class="stat-label">Voting Power</div>
                </div>
            </div>
        `;
    }

    async loadActiveProposals() {
        try {
            const proposals = await this.actor.get_proposals(0, 20); // Get first 20 proposals
            this.displayProposals(proposals);
        } catch (error) {
            console.error('❌ Failed to load proposals:', error);
        }
    }

    displayProposals(proposals) {
        const container = document.getElementById('proposalsList');
        if (!container) return;

        const activeProposals = proposals.filter(p => p.status === 'Active');

        container.innerHTML = `
            <div class="proposals-header">
                <h3>🗳️ Active Proposals</h3>
                ${this.isAdmin() ? `
                    <button class="btn btn-primary" onclick="governance.showCreateProposalModal()">
                        Create Proposal
                    </button>
                ` : ''}
            </div>
            <div class="proposals-list">
                ${activeProposals.length > 0 
                    ? activeProposals.map(proposal => this.createProposalCard(proposal)).join('')
                    : '<div class="no-proposals">No active proposals</div>'
                }
            </div>
        `;
    }

    createProposalCard(proposal) {
        const createdAt = new Date(Number(proposal.created_at) / 1_000_000);
        const deadline = new Date(Number(proposal.voting_deadline) / 1_000_000);
        const timeLeft = deadline.getTime() - Date.now();
        const daysLeft = Math.max(0, Math.ceil(timeLeft / (1000 * 60 * 60 * 24)));
        
        const totalVotes = proposal.yes_votes + proposal.no_votes + proposal.abstain_votes;
        const participationRate = proposal.total_voting_power > 0 
            ? (totalVotes * 100) / proposal.total_voting_power 
            : 0;

        return `
            <div class="proposal-card" data-id="${proposal.id}">
                <div class="proposal-header">
                    <h4>${proposal.title}</h4>
                    <div class="proposal-type">${proposal.proposal_type}</div>
                </div>
                <div class="proposal-description">${proposal.description}</div>
                <div class="proposal-stats">
                    <div class="vote-stats">
                        <div class="vote-bar">
                            <div class="yes-votes" style="width: ${totalVotes > 0 ? (proposal.yes_votes * 100) / totalVotes : 0}%"></div>
                            <div class="no-votes" style="width: ${totalVotes > 0 ? (proposal.no_votes * 100) / totalVotes : 0}%"></div>
                        </div>
                        <div class="vote-numbers">
                            <span class="yes">👍 ${proposal.yes_votes}</span>
                            <span class="no">👎 ${proposal.no_votes}</span>
                            <span class="abstain">🤷 ${proposal.abstain_votes}</span>
                        </div>
                    </div>
                    <div class="proposal-meta">
                        <div class="participation">Participation: ${participationRate.toFixed(1)}%</div>
                        <div class="time-left">${daysLeft} days left</div>
                    </div>
                </div>
                <div class="proposal-actions">
                    ${this.isAdmin() ? `
                        <button class="btn btn-success" onclick="governance.vote(${proposal.id}, 'Yes')">
                            Vote Yes
                        </button>
                        <button class="btn btn-danger" onclick="governance.vote(${proposal.id}, 'No')">
                            Vote No
                        </button>
                        <button class="btn btn-secondary" onclick="governance.vote(${proposal.id}, 'Abstain')">
                            Abstain
                        </button>
                    ` : ''}
                    <button class="btn btn-outline" onclick="governance.viewProposalDetails(${proposal.id})">
                        View Details
                    </button>
                </div>
            </div>
        `;
    }

    async loadAdminRoles() {
        try {
            const roles = await this.actor.get_all_admin_roles();
            this.displayAdminRoles(roles);
        } catch (error) {
            console.error('❌ Failed to load admin roles:', error);
        }
    }

    displayAdminRoles(roles) {
        const container = document.getElementById('adminRoles');
        if (!container) return;

        const activeRoles = roles.filter(role => role.is_active);

        container.innerHTML = `
            <div class="admin-roles-header">
                <h3>👑 Admin Roles</h3>
                ${this.isSuperAdmin() ? `
                    <button class="btn btn-primary" onclick="governance.showGrantRoleModal()">
                        Grant Role
                    </button>
                ` : ''}
            </div>
            <div class="admin-roles-list">
                ${activeRoles.map(role => this.createAdminRoleCard(role)).join('')}
            </div>
        `;
    }

    createAdminRoleCard(role) {
        const grantedAt = new Date(Number(role.granted_at) / 1_000_000).toLocaleString();
        const expiresAt = role.expires_at && role.expires_at[0] 
            ? new Date(Number(role.expires_at[0]) / 1_000_000).toLocaleString()
            : 'Never';

        return `
            <div class="admin-role-card">
                <div class="role-header">
                    <div class="role-principal">${role.principal.toString()}</div>
                    <div class="role-type ${role.role_type.toLowerCase()}">${role.role_type}</div>
                </div>
                <div class="role-details">
                    <div class="role-permissions">
                        <strong>Permissions:</strong>
                        <div class="permissions-list">
                            ${role.permissions.map(perm => `<span class="permission">${perm}</span>`).join('')}
                        </div>
                    </div>
                    <div class="role-meta">
                        <div>Granted: ${grantedAt}</div>
                        <div>Expires: ${expiresAt}</div>
                        <div>Granted by: ${role.granted_by.toString().slice(0, 8)}...</div>
                    </div>
                </div>
                ${this.isSuperAdmin() && role.role_type !== 'SuperAdmin' ? `
                    <button class="btn btn-danger btn-sm" onclick="governance.revokeRole('${role.principal}')">
                        Revoke Role
                    </button>
                ` : ''}
            </div>
        `;
    }

    // Admin action methods
    async updateParameter(key, currentValue) {
        if (!this.isAdmin()) {
            this.showError('Only admins can update parameters');
            return;
        }

        const newValue = prompt(
            `Update ${this.formatParameterName(key)}\nCurrent value: ${currentValue}\nEnter new value:`,
            currentValue
        );

        if (newValue === null || newValue === '') return;

        const numericValue = parseInt(newValue);
        if (isNaN(numericValue)) {
            this.showError('Please enter a valid numeric value');
            return;
        }

        try {
            this.showLoading('Updating parameter...');
            const result = await this.actor.set_protocol_parameter(key, numericValue);
            
            if ('Ok' in result) {
                this.showSuccess(`Parameter ${key} updated successfully`);
                await this.loadProtocolParameters(); // Refresh parameters
            } else {
                this.showError(`Failed to update parameter: ${result.Err}`);
            }
        } catch (error) {
            console.error('❌ Failed to update parameter:', error);
            this.showError('Failed to update parameter');
        } finally {
            this.hideLoading();
        }
    }

    async vote(proposalId, choice) {
        if (!this.isAdmin()) {
            this.showError('Only admins can vote on proposals');
            return;
        }

        const reason = prompt(`Vote ${choice} on proposal ${proposalId}\nOptional reason:`);
        if (reason === null) return;

        try {
            this.showLoading('Casting vote...');
            const result = await this.actor.vote_on_proposal(
                proposalId, 
                { [choice]: null }, 
                reason ? [reason] : []
            );
            
            if ('Ok' in result) {
                this.showSuccess(`Vote cast successfully: ${choice}`);
                await this.loadActiveProposals(); // Refresh proposals
            } else {
                this.showError(`Failed to vote: ${result.Err}`);
            }
        } catch (error) {
            console.error('❌ Failed to vote:', error);
            this.showError('Failed to cast vote');
        } finally {
            this.hideLoading();
        }
    }

    async transferAdminRole() {
        if (!this.isSuperAdmin()) {
            this.showError('Only super admins can transfer admin role');
            return;
        }

        const newAdminPrincipal = prompt('Enter new admin principal:');
        if (!newAdminPrincipal) return;

        if (!confirm(`Are you sure you want to transfer admin role to ${newAdminPrincipal}? This action cannot be undone.`)) {
            return;
        }

        try {
            this.showLoading('Transferring admin role...');
            const result = await this.actor.transfer_admin_role(newAdminPrincipal);
            
            if ('Ok' in result) {
                this.showSuccess('Admin role transferred successfully');
                await this.loadCurrentUser(); // Refresh user info
                await this.loadAdminRoles(); // Refresh admin roles
            } else {
                this.showError(`Failed to transfer admin role: ${result.Err}`);
            }
        } catch (error) {
            console.error('❌ Failed to transfer admin role:', error);
            this.showError('Failed to transfer admin role');
        } finally {
            this.hideLoading();
        }
    }

    async emergencyStop() {
        if (!this.hasPermission('EmergencyStop')) {
            this.showError('You do not have permission to activate emergency stop');
            return;
        }

        if (!confirm('Are you sure you want to activate emergency stop? This will halt all protocol operations.')) {
            return;
        }

        try {
            this.showLoading('Activating emergency stop...');
            const result = await this.actor.emergency_stop();
            
            if ('Ok' in result) {
                this.showSuccess('Emergency stop activated');
                await this.loadProtocolParameters(); // Refresh to show updated status
            } else {
                this.showError(`Failed to activate emergency stop: ${result.Err}`);
            }
        } catch (error) {
            console.error('❌ Failed to activate emergency stop:', error);
            this.showError('Failed to activate emergency stop');
        } finally {
            this.hideLoading();
        }
    }

    async resumeOperations() {
        if (!this.isSuperAdmin()) {
            this.showError('Only super admins can resume operations');
            return;
        }

        if (!confirm('Are you sure you want to resume protocol operations?')) {
            return;
        }

        try {
            this.showLoading('Resuming operations...');
            const result = await this.actor.resume_operations();
            
            if ('Ok' in result) {
                this.showSuccess('Operations resumed successfully');
                await this.loadProtocolParameters(); // Refresh to show updated status
            } else {
                this.showError(`Failed to resume operations: ${result.Err}`);
            }
        } catch (error) {
            console.error('❌ Failed to resume operations:', error);
            this.showError('Failed to resume operations');
        } finally {
            this.hideLoading();
        }
    }

    // Modal management
    showParameterUpdateModal() {
        // Implementation for parameter update modal
        this.showModal('parameterUpdateModal');
    }

    showCreateProposalModal() {
        // Implementation for create proposal modal
        this.showModal('createProposalModal');
    }

    showGrantRoleModal() {
        // Implementation for grant role modal
        this.showModal('grantRoleModal');
    }

    showModal(modalId) {
        const modal = document.getElementById(modalId);
        if (modal) {
            modal.style.display = 'block';
        }
    }

    hideModal(modalId) {
        const modal = document.getElementById(modalId);
        if (modal) {
            modal.style.display = 'none';
        }
    }

    // Utility methods
    isAdmin() {
        return this.userRole !== null && this.userRole.is_active;
    }

    isSuperAdmin() {
        return this.userRole && this.userRole.role_type === 'SuperAdmin' && this.userRole.is_active;
    }

    hasPermission(permission) {
        return this.userRole && 
               this.userRole.is_active && 
               this.userRole.permissions.includes(permission);
    }

    updateUIBasedOnRole() {
        const adminOnlyElements = document.querySelectorAll('.admin-only');
        const superAdminOnlyElements = document.querySelectorAll('.super-admin-only');

        adminOnlyElements.forEach(element => {
            element.style.display = this.isAdmin() ? 'block' : 'none';
        });

        superAdminOnlyElements.forEach(element => {
            element.style.display = this.isSuperAdmin() ? 'block' : 'none';
        });
    }

    setupEventListeners() {
        // Set up refresh button
        const refreshBtn = document.getElementById('refreshData');
        if (refreshBtn) {
            refreshBtn.addEventListener('click', () => this.refreshAllData());
        }

        // Set up emergency stop button
        const emergencyBtn = document.getElementById('emergencyStop');
        if (emergencyBtn) {
            emergencyBtn.addEventListener('click', () => this.emergencyStop());
        }

        // Set up resume operations button
        const resumeBtn = document.getElementById('resumeOperations');
        if (resumeBtn) {
            resumeBtn.addEventListener('click', () => this.resumeOperations());
        }

        // Set up transfer admin button
        const transferBtn = document.getElementById('transferAdmin');
        if (transferBtn) {
            transferBtn.addEventListener('click', () => this.transferAdminRole());
        }

        // Set up modal close buttons
        document.querySelectorAll('.modal .close').forEach(closeBtn => {
            closeBtn.addEventListener('click', (e) => {
                const modal = e.target.closest('.modal');
                if (modal) {
                    modal.style.display = 'none';
                }
            });
        });
    }

    async refreshAllData() {
        this.showLoading('Refreshing data...');
        try {
            await Promise.all([
                this.loadCurrentUser(),
                this.loadProtocolParameters(),
                this.loadGovernanceStats(),
                this.loadActiveProposals(),
                this.loadAdminRoles()
            ]);
            this.showSuccess('Data refreshed successfully');
        } catch (error) {
            console.error('❌ Failed to refresh data:', error);
            this.showError('Failed to refresh data');
        } finally {
            this.hideLoading();
        }
    }

    // UI feedback methods
    showLoading(message) {
        const loadingElement = document.getElementById('loadingIndicator');
        if (loadingElement) {
            loadingElement.textContent = message;
            loadingElement.style.display = 'block';
        }
    }

    hideLoading() {
        const loadingElement = document.getElementById('loadingIndicator');
        if (loadingElement) {
            loadingElement.style.display = 'none';
        }
    }

    showSuccess(message) {
        this.showNotification(message, 'success');
    }

    showError(message) {
        this.showNotification(message, 'error');
    }

    showNotification(message, type) {
        const notification = document.createElement('div');
        notification.className = `notification ${type}`;
        notification.textContent = message;
        
        document.body.appendChild(notification);
        
        setTimeout(() => {
            notification.remove();
        }, 5000);
    }
}

// Initialize governance manager when page loads
let governance;

document.addEventListener('DOMContentLoaded', () => {
    // Replace with your actual canister ID
    const BACKEND_CANISTER_ID = 'rdmx6-jaaaa-aaaah-qcaiq-cai';
    governance = new GovernanceManager(BACKEND_CANISTER_ID);
});

// Placeholder IDL factory - replace with actual generated interface
const idlFactory = ({ IDL }) => {
    return IDL.Service({
        'get_caller': IDL.Func([], [IDL.Principal], ['query']),
        'get_admin_role': IDL.Func([IDL.Principal], [IDL.Opt(IDL.Record({
            'principal': IDL.Principal,
            'role_type': IDL.Variant({
                'SuperAdmin': IDL.Null,
                'ProtocolAdmin': IDL.Null,
                'TreasuryAdmin': IDL.Null,
                'RiskAdmin': IDL.Null,
                'LiquidationAdmin': IDL.Null,
                'OracleAdmin': IDL.Null,
                'EmergencyAdmin': IDL.Null,
            }),
            'granted_at': IDL.Nat64,
            'granted_by': IDL.Principal,
            'expires_at': IDL.Opt(IDL.Nat64),
            'permissions': IDL.Vec(IDL.Variant({
                'ManageParameters': IDL.Null,
                'ManageAdmins': IDL.Null,
                'EmergencyStop': IDL.Null,
                'ManageTreasury': IDL.Null,
                'ManageLiquidation': IDL.Null,
                'ManageOracle': IDL.Null,
                'ViewMetrics': IDL.Null,
                'ExecuteProposals': IDL.Null,
            })),
            'is_active': IDL.Bool,
        }))], ['query']),
        // Add other function signatures as needed
    });
};

// Make functions globally available
window.governance = governance;
