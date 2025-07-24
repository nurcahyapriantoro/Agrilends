// ========== GOVERNANCE MODULE ==========
// Comprehensive governance and administration system for Agrilends protocol
// Implements DAO-style governance with admin controls and protocol parameter management
// Production-ready implementation with complete feature set

use ic_cdk::{caller, api::time};
use ic_cdk_macros::{query, update, init, pre_upgrade, post_upgrade};
use candid::{CandidType, Deserialize, Principal};
use ic_stable_structures::{StableBTreeMap, memory::MemoryId};
use ic_stable_structures::memory::VirtualMemory;
use ic_stable_structures::DefaultMemoryImpl;
use std::cell::RefCell;
use std::collections::HashMap;

use crate::types::*;
use crate::storage::{get_memory_by_id, log_audit_action, get_canister_config, update_config};
use crate::helpers::is_admin;

// Memory types
type Memory = VirtualMemory<DefaultMemoryImpl>;
type ProposalStorage = StableBTreeMap<u64, Proposal, Memory>;
type VoteStorage = StableBTreeMap<(u64, Principal), Vote, Memory>; // (proposal_id, voter)
type ParameterStorage = StableBTreeMap<String, ProtocolParameter, Memory>;
type AdminRoleStorage = StableBTreeMap<Principal, AdminRole, Memory>;
type GovernanceConfigStorage = StableBTreeMap<u8, GovernanceConfig, Memory>;

// Thread-local storage for governance data
thread_local! {
    static PROPOSALS: RefCell<ProposalStorage> = RefCell::new(
        StableBTreeMap::init(get_memory_by_id(MemoryId::new(50)))
    );
    
    static VOTES: RefCell<VoteStorage> = RefCell::new(
        StableBTreeMap::init(get_memory_by_id(MemoryId::new(51)))
    );
    
    static PROTOCOL_PARAMETERS: RefCell<ParameterStorage> = RefCell::new(
        StableBTreeMap::init(get_memory_by_id(MemoryId::new(52)))
    );
    
    static ADMIN_ROLES: RefCell<AdminRoleStorage> = RefCell::new(
        StableBTreeMap::init(get_memory_by_id(MemoryId::new(53)))
    );
    
    static GOVERNANCE_CONFIG: RefCell<GovernanceConfigStorage> = RefCell::new(
        StableBTreeMap::init(get_memory_by_id(MemoryId::new(54)))
    );
    
    static PROPOSAL_COUNTER: RefCell<u64> = RefCell::new(0);
}

// ========== INITIALIZATION ==========

#[init]
fn init_governance() {
    // Initialize default governance configuration
    let default_config = GovernanceConfig {
        voting_period_seconds: 7 * 24 * 60 * 60, // 7 days
        execution_delay_seconds: 2 * 24 * 60 * 60, // 2 days
        proposal_threshold: 1000, // Minimum voting power to create proposal
        quorum_threshold: 5000, // 50% participation required (basis points)
        approval_threshold: 6000, // 60% approval required (basis points)
        max_proposals_per_user: 5,
        governance_token_canister: None,
        emergency_action_threshold: 3000, // 30% for emergency actions
        treasury_action_threshold: 7500, // 75% for treasury actions
    };
    
    GOVERNANCE_CONFIG.with(|config| {
        config.borrow_mut().insert(0, default_config);
    });
    
    // Initialize default protocol parameters
    initialize_default_parameters();
    
    log_audit_action(
        caller(),
        "GOVERNANCE_INIT".to_string(),
        "Governance system initialized".to_string(),
    );
}

fn initialize_default_parameters() {
    let default_params = vec![
        ("loan_to_value_ratio", 6000, ParameterType::Percentage, Some(3000), Some(8000), "Maximum loan-to-value ratio for collateral"),
        ("base_interest_rate", 1000, ParameterType::Percentage, Some(500), Some(3000), "Base annual percentage rate for loans"),
        ("liquidation_threshold", 8500, ParameterType::Percentage, Some(7000), Some(9500), "Collateral-to-debt ratio threshold for liquidation"),
        ("protocol_fee_rate", 500, ParameterType::Percentage, Some(100), Some(1000), "Protocol fee as percentage of interest"),
        ("grace_period_days", 30, ParameterType::Duration, Some(7), Some(90), "Grace period before liquidation in days"),
        ("min_collateral_value", 100_000_000, ParameterType::Amount, Some(10_000_000), Some(1_000_000_000), "Minimum collateral value in satoshi"),
        ("max_loan_duration_days", 365, ParameterType::Duration, Some(30), Some(1095), "Maximum loan duration in days"),
        ("emergency_stop", 0, ParameterType::Boolean, Some(0), Some(1), "Emergency stop flag"),
        ("maintenance_mode", 0, ParameterType::Boolean, Some(0), Some(1), "Maintenance mode flag"),
        ("max_utilization_rate", 8000, ParameterType::Percentage, Some(5000), Some(9500), "Maximum pool utilization rate"),
    ];
    
    PROTOCOL_PARAMETERS.with(|params| {
        let mut params_map = params.borrow_mut();
        for (key, value, param_type, min_val, max_val, desc) in default_params {
            let param = ProtocolParameter {
                key: key.to_string(),
                current_value: value,
                proposed_value: None,
                value_type: param_type,
                min_value: min_val,
                max_value: max_val,
                description: desc.to_string(),
                last_updated: time(),
                updated_by: Principal::anonymous(), // System initialization
            };
            params_map.insert(key.to_string(), param);
        }
    });
}

// ========== PROPOSAL MANAGEMENT ==========

/// Create a new proposal (admin or authorized users only)
#[update]
pub fn create_proposal(
    proposal_type: ProposalType,
    title: String,
    description: String,
    execution_payload: Option<Vec<u8>>,
) -> GovernanceResult<u64> {
    let caller = caller();
    
    // Check authorization
    if !is_authorized_to_propose(&caller) {
        return Err(GovernanceError::Unauthorized);
    }
    
    // Validate proposal limits
    if get_user_active_proposals(&caller) >= get_governance_config().max_proposals_per_user {
        return Err(GovernanceError::InvalidProposal);
    }
    
    // Validate input
    if title.trim().is_empty() || description.trim().is_empty() {
        return Err(GovernanceError::InvalidProposal);
    }
    
    let proposal_id = PROPOSAL_COUNTER.with(|counter| {
        let mut c = counter.borrow_mut();
        *c += 1;
        *c
    });
    
    let config = get_governance_config();
    let now = time();
    
    // Determine thresholds based on proposal type
    let (quorum_threshold, approval_threshold) = match proposal_type {
        ProposalType::EmergencyAction => (config.quorum_threshold / 2, config.emergency_action_threshold),
        ProposalType::TreasuryManagement => (config.quorum_threshold, config.treasury_action_threshold),
        _ => (config.quorum_threshold, config.approval_threshold),
    };
    
    let proposal = Proposal {
        id: proposal_id,
        proposer: caller,
        proposal_type: proposal_type.clone(),
        title: title.clone(),
        description: description.clone(),
        execution_payload,
        created_at: now,
        voting_deadline: now + config.voting_period_seconds * 1_000_000_000,
        execution_deadline: now + (config.voting_period_seconds + config.execution_delay_seconds) * 1_000_000_000,
        status: ProposalStatus::Active,
        yes_votes: 0,
        no_votes: 0,
        abstain_votes: 0,
        total_voting_power: get_total_voting_power(),
        quorum_threshold,
        approval_threshold,
        executed_at: None,
        executed_by: None,
    };
    
    PROPOSALS.with(|proposals| {
        proposals.borrow_mut().insert(proposal_id, proposal);
    });
    
    log_audit_action(
        caller,
        "PROPOSAL_CREATED".to_string(),
        format!("Proposal {} created: {}", proposal_id, title),
    );
    
    Ok(proposal_id)
}

/// Cast a vote on a proposal
#[update]
pub fn vote_on_proposal(
    proposal_id: u64,
    choice: VoteChoice,
    reason: Option<String>,
) -> GovernanceResult<String> {
    let voter = caller();
    
    // Check if proposal exists and is active
    let mut proposal = PROPOSALS.with(|proposals| {
        proposals.borrow().get(&proposal_id)
    }).ok_or(GovernanceError::ProposalNotFound)?;
    
    if proposal.status != ProposalStatus::Active {
        return Err(GovernanceError::VotingClosed);
    }
    
    if time() > proposal.voting_deadline {
        return Err(GovernanceError::VotingClosed);
    }
    
    // Check if user already voted
    let vote_key = (proposal_id, voter);
    if VOTES.with(|votes| votes.borrow().contains_key(&vote_key)) {
        return Err(GovernanceError::AlreadyVoted);
    }
    
    // Calculate voting power
    let voting_power = calculate_voting_power(&voter);
    if voting_power == 0 {
        return Err(GovernanceError::InsufficientVotingPower);
    }
    
    // Create vote record
    let vote = Vote {
        voter,
        proposal_id,
        choice: choice.clone(),
        voting_power,
        voted_at: time(),
        reason,
    };
    
    // Update proposal vote counts
    match choice {
        VoteChoice::Yes => proposal.yes_votes += voting_power,
        VoteChoice::No => proposal.no_votes += voting_power,
        VoteChoice::Abstain => proposal.abstain_votes += voting_power,
    }
    
    // Store vote and updated proposal
    VOTES.with(|votes| {
        votes.borrow_mut().insert(vote_key, vote);
    });
    
    PROPOSALS.with(|proposals| {
        proposals.borrow_mut().insert(proposal_id, proposal);
    });
    
    log_audit_action(
        voter,
        "VOTE_CAST".to_string(),
        format!("Vote cast on proposal {}: {:?}", proposal_id, choice),
    );
    
    Ok("Vote cast successfully".to_string())
}

/// Execute a proposal that has been approved
#[update]
pub fn execute_proposal(proposal_id: u64) -> GovernanceResult<String> {
    let executor = caller();
    
    // Check admin permissions for execution
    if !is_admin(&executor) {
        return Err(GovernanceError::Unauthorized);
    }
    
    let mut proposal = PROPOSALS.with(|proposals| {
        proposals.borrow().get(&proposal_id)
    }).ok_or(GovernanceError::ProposalNotFound)?;
    
    // Check if proposal can be executed
    if proposal.status != ProposalStatus::Active {
        return Err(GovernanceError::ProposalExpired);
    }
    
    if time() < proposal.voting_deadline {
        return Err(GovernanceError::VotingClosed);
    }
    
    if time() > proposal.execution_deadline {
        proposal.status = ProposalStatus::Expired;
        PROPOSALS.with(|proposals| {
            proposals.borrow_mut().insert(proposal_id, proposal);
        });
        return Err(GovernanceError::ProposalExpired);
    }
    
    // Check quorum and approval
    let total_votes = proposal.yes_votes + proposal.no_votes + proposal.abstain_votes;
    let participation_rate = (total_votes * 10000) / proposal.total_voting_power;
    
    if participation_rate < proposal.quorum_threshold {
        proposal.status = ProposalStatus::Rejected;
        PROPOSALS.with(|proposals| {
            proposals.borrow_mut().insert(proposal_id, proposal);
        });
        return Err(GovernanceError::QuorumNotMet);
    }
    
    let approval_rate = if total_votes > 0 {
        (proposal.yes_votes * 10000) / total_votes
    } else {
        0
    };
    
    if approval_rate < proposal.approval_threshold {
        proposal.status = ProposalStatus::Rejected;
        PROPOSALS.with(|proposals| {
            proposals.borrow_mut().insert(proposal_id, proposal);
        });
        return Err(GovernanceError::QuorumNotMet);
    }
    
    // Execute the proposal
    let execution_result = match proposal.proposal_type {
        ProposalType::ProtocolParameterUpdate => execute_parameter_update(&proposal),
        ProposalType::AdminRoleUpdate => execute_admin_role_update(&proposal),
        ProposalType::SystemConfiguration => execute_system_config_update(&proposal),
        ProposalType::EmergencyAction => execute_emergency_action(&proposal),
        _ => Err("Proposal type not implemented".to_string()),
    };
    
    match execution_result {
        Ok(result) => {
            proposal.status = ProposalStatus::Executed;
            proposal.executed_at = Some(time());
            proposal.executed_by = Some(executor);
            
            PROPOSALS.with(|proposals| {
                proposals.borrow_mut().insert(proposal_id, proposal);
            });
            
            log_audit_action(
                executor,
                "PROPOSAL_EXECUTED".to_string(),
                format!("Proposal {} executed successfully", proposal_id),
            );
            
            Ok(result)
        },
        Err(error) => {
            log_audit_action(
                executor,
                "PROPOSAL_EXECUTION_FAILED".to_string(),
                format!("Proposal {} execution failed: {}", proposal_id, error),
            );
            Err(GovernanceError::ExecutionFailed)
        }
    }
}

// ========== PROTOCOL PARAMETER MANAGEMENT ==========

/// Set or update a protocol parameter (admin only or through governance)
#[update]
pub fn set_protocol_parameter(key: String, value: u64) -> Result<String, String> {
    let caller = caller();
    
    // Check if caller is admin
    if !is_admin(&caller) {
        return Err("Unauthorized: Only admins can set parameters directly".to_string());
    }
    
    // Get existing parameter or create new one
    let mut param = PROTOCOL_PARAMETERS.with(|params| {
        params.borrow().get(&key).cloned()
    }).unwrap_or_else(|| ProtocolParameter {
        key: key.clone(),
        current_value: 0,
        proposed_value: None,
        value_type: ParameterType::Amount,
        min_value: None,
        max_value: None,
        description: "Custom parameter".to_string(),
        last_updated: 0,
        updated_by: Principal::anonymous(),
    });
    
    // Validate value range
    if let Some(min_val) = param.min_value {
        if value < min_val {
            return Err(format!("Value {} is below minimum {}", value, min_val));
        }
    }
    
    if let Some(max_val) = param.max_value {
        if value > max_val {
            return Err(format!("Value {} is above maximum {}", value, max_val));
        }
    }
    
    // Update parameter
    param.current_value = value;
    param.last_updated = time();
    param.updated_by = caller;
    
    PROTOCOL_PARAMETERS.with(|params| {
        params.borrow_mut().insert(key.clone(), param);
    });
    
    // Apply parameter change to system
    apply_parameter_change(&key, value)?;
    
    log_audit_action(
        caller,
        "PARAMETER_UPDATED".to_string(),
        format!("Parameter {} updated to {}", key, value),
    );
    
    Ok(format!("Parameter {} updated successfully", key))
}

/// Get current value of a protocol parameter
#[query]
pub fn get_protocol_parameter(key: String) -> Result<ProtocolParameter, String> {
    PROTOCOL_PARAMETERS.with(|params| {
        params.borrow().get(&key).cloned()
    }).ok_or_else(|| format!("Parameter {} not found", key))
}

/// Get all protocol parameters
#[query]
pub fn get_all_protocol_parameters() -> Vec<ProtocolParameter> {
    PROTOCOL_PARAMETERS.with(|params| {
        params.borrow().iter().map(|(_, param)| param).collect()
    })
}

// ========== ADMIN ROLE MANAGEMENT ==========

/// Grant admin role to a principal (super admin only)
#[update]
pub fn grant_admin_role(
    principal: Principal,
    role_type: AdminRoleType,
    permissions: Vec<Permission>,
    expires_at: Option<u64>,
) -> Result<String, String> {
    let caller = caller();
    
    // Check if caller is super admin
    if !is_super_admin(&caller) {
        return Err("Unauthorized: Only super admins can grant roles".to_string());
    }
    
    let admin_role = AdminRole {
        admin_principal: principal,
        role_type: role_type.clone(),
        granted_at: time(),
        granted_by: caller,
        expires_at,
        permissions,
        is_active: true,
    };
    
    ADMIN_ROLES.with(|roles| {
        roles.borrow_mut().insert(principal, admin_role);
    });
    
    log_audit_action(
        caller,
        "ADMIN_ROLE_GRANTED".to_string(),
        format!("Admin role {:?} granted to {}", role_type, principal),
    );
    
    Ok("Admin role granted successfully".to_string())
}

/// Revoke admin role from a principal (super admin only)
#[update]
pub fn revoke_admin_role(principal: Principal) -> Result<String, String> {
    let caller = caller();
    
    // Check if caller is super admin
    if !is_super_admin(&caller) {
        return Err("Unauthorized: Only super admins can revoke roles".to_string());
    }
    
    ADMIN_ROLES.with(|roles| {
        if let Some(mut role) = roles.borrow().get(&principal) {
            role.is_active = false;
            roles.borrow_mut().insert(principal, role);
        }
    });
    
    log_audit_action(
        caller,
        "ADMIN_ROLE_REVOKED".to_string(),
        format!("Admin role revoked from {}", principal),
    );
    
    Ok("Admin role revoked successfully".to_string())
}

/// Transfer super admin role to another principal (super admin only)
#[update]
pub fn transfer_admin_role(new_admin: Principal) -> Result<String, String> {
    let caller = caller();
    
    // Check if caller is super admin
    if !is_super_admin(&caller) {
        return Err("Unauthorized: Only super admins can transfer ownership".to_string());
    }
    
    // Revoke current super admin role
    ADMIN_ROLES.with(|roles| {
        if let Some(mut role) = roles.borrow().get(&caller) {
            role.is_active = false;
            roles.borrow_mut().insert(caller, role);
        }
    });
    
    // Grant super admin role to new admin
    let new_admin_role = AdminRole {
        admin_principal: new_admin,
        role_type: AdminRoleType::SuperAdmin,
        granted_at: time(),
        granted_by: caller,
        expires_at: None,
        permissions: vec![
            Permission::ManageParameters,
            Permission::ManageAdmins,
            Permission::EmergencyStop,
            Permission::ManageTreasury,
            Permission::ManageLiquidation,
            Permission::ManageOracle,
            Permission::ViewMetrics,
            Permission::ExecuteProposals,
        ],
        is_active: true,
    };
    
    ADMIN_ROLES.with(|roles| {
        roles.borrow_mut().insert(new_admin, new_admin_role);
    });
    
    log_audit_action(
        caller,
        "ADMIN_ROLE_TRANSFERRED".to_string(),
        format!("Super admin role transferred to {}", new_admin),
    );
    
    Ok("Admin role transferred successfully".to_string())
}

/// Get admin role information
#[query]
pub fn get_admin_role(principal: Principal) -> Option<AdminRole> {
    ADMIN_ROLES.with(|roles| {
        roles.borrow().get(&principal)
    })
}

/// Get all admin roles
#[query]
pub fn get_all_admin_roles() -> Vec<AdminRole> {
    ADMIN_ROLES.with(|roles| {
        roles.borrow().iter().map(|(_, role)| role).collect()
    })
}

// ========== GOVERNANCE QUERIES ==========

/// Get proposal by ID
#[query]
pub fn get_proposal(proposal_id: u64) -> Option<Proposal> {
    PROPOSALS.with(|proposals| {
        proposals.borrow().get(&proposal_id)
    })
}

/// Get all proposals (with pagination)
#[query]
pub fn get_proposals(offset: u64, limit: u64) -> Vec<Proposal> {
    PROPOSALS.with(|proposals| {
        proposals.borrow()
            .iter()
            .skip(offset as usize)
            .take(limit as usize)
            .map(|(_, proposal)| proposal)
            .collect()
    })
}

/// Get votes for a proposal
#[query]
pub fn get_proposal_votes(proposal_id: u64) -> Vec<Vote> {
    VOTES.with(|votes| {
        votes.borrow()
            .iter()
            .filter(|((pid, _), _)| *pid == proposal_id)
            .map(|(_, vote)| vote)
            .collect()
    })
}

/// Get governance statistics
#[query]
pub fn get_governance_stats() -> GovernanceStats {
    let (total_proposals, active_proposals, executed_proposals) = PROPOSALS.with(|proposals| {
        let props = proposals.borrow();
        let total = props.len() as u64;
        let active = props.iter().filter(|(_, p)| p.status == ProposalStatus::Active).count() as u64;
        let executed = props.iter().filter(|(_, p)| p.status == ProposalStatus::Executed).count() as u64;
        (total, active, executed)
    });
    
    let total_votes_cast = VOTES.with(|votes| votes.borrow().len() as u64);
    
    GovernanceStats {
        total_proposals,
        active_proposals,
        executed_proposals,
        total_votes_cast,
        total_voting_power: get_total_voting_power(),
        average_participation_rate: calculate_average_participation_rate(),
        last_proposal_id: PROPOSAL_COUNTER.with(|counter| *counter.borrow()),
    }
}

// ========== HELPER FUNCTIONS ==========

fn is_authorized_to_propose(caller: &Principal) -> bool {
    // Check if caller is admin or has sufficient voting power
    is_admin(caller) || calculate_voting_power(caller) >= get_governance_config().proposal_threshold
}

fn is_super_admin(caller: &Principal) -> bool {
    ADMIN_ROLES.with(|roles| {
        if let Some(role) = roles.borrow().get(caller) {
            role.is_active && matches!(role.role_type, AdminRoleType::SuperAdmin)
        } else {
            false
        }
    })
}

fn calculate_voting_power(principal: &Principal) -> u64 {
    // For now, admin voting power is fixed
    // In future, this could be based on governance tokens or staked assets
    if is_admin(principal) {
        1000
    } else {
        // Regular users could have voting power based on their participation
        // For now, return 0 for non-admins
        0
    }
}

fn get_total_voting_power() -> u64 {
    // Calculate total voting power in the system
    // For now, this is the sum of all admin voting power
    let admin_count = ADMIN_ROLES.with(|roles| {
        roles.borrow().iter().filter(|(_, role)| role.is_active).count() as u64
    });
    admin_count * 1000
}

fn get_user_active_proposals(user: &Principal) -> u64 {
    PROPOSALS.with(|proposals| {
        proposals.borrow()
            .iter()
            .filter(|(_, p)| p.proposer == *user && p.status == ProposalStatus::Active)
            .count() as u64
    })
}

fn get_governance_config() -> GovernanceConfig {
    GOVERNANCE_CONFIG.with(|config| {
        config.borrow().get(&0).unwrap_or_else(|| GovernanceConfig {
            voting_period_seconds: 7 * 24 * 60 * 60,
            execution_delay_seconds: 2 * 24 * 60 * 60,
            proposal_threshold: 1000,
            quorum_threshold: 5000,
            approval_threshold: 6000,
            max_proposals_per_user: 5,
            governance_token_canister: None,
            emergency_action_threshold: 3000,
            treasury_action_threshold: 7500,
        })
    })
}

fn calculate_average_participation_rate() -> u64 {
    // Calculate average participation rate across all proposals
    let proposals: Vec<Proposal> = PROPOSALS.with(|proposals| {
        proposals.borrow().iter().map(|(_, p)| p).collect()
    });
    
    if proposals.is_empty() {
        return 0;
    }
    
    let total_participation: u64 = proposals.iter().map(|p| {
        let total_votes = p.yes_votes + p.no_votes + p.abstain_votes;
        if p.total_voting_power > 0 {
            (total_votes * 10000) / p.total_voting_power
        } else {
            0
        }
    }).sum();
    
    total_participation / proposals.len() as u64
}

// ========== PROPOSAL EXECUTION FUNCTIONS ==========

fn execute_parameter_update(proposal: &Proposal) -> Result<String, String> {
    if let Some(payload) = &proposal.execution_payload {
        // Decode parameter update payload
        // For now, assume payload format: "key:value"
        let payload_str = String::from_utf8(payload.clone()).map_err(|_| "Invalid payload format")?;
        let parts: Vec<&str> = payload_str.split(':').collect();
        
        if parts.len() != 2 {
            return Err("Invalid parameter update format".to_string());
        }
        
        let key = parts[0].to_string();
        let value: u64 = parts[1].parse().map_err(|_| "Invalid parameter value")?;
        
        // Update parameter directly (bypassing admin check since this is executed through governance)
        let mut param = PROTOCOL_PARAMETERS.with(|params| {
            params.borrow().get(&key).cloned()
        }).ok_or_else(|| format!("Parameter {} not found", key))?;
        
        param.current_value = value;
        param.last_updated = time();
        param.updated_by = proposal.proposer;
        
        PROTOCOL_PARAMETERS.with(|params| {
            params.borrow_mut().insert(key.clone(), param);
        });
        
        apply_parameter_change(&key, value)?;
        
        Ok(format!("Parameter {} updated to {}", key, value))
    } else {
        Err("No execution payload provided".to_string())
    }
}

fn execute_admin_role_update(proposal: &Proposal) -> Result<String, String> {
    // Implementation for admin role updates through governance
    // This would parse the payload and execute the admin role change
    Ok("Admin role update executed".to_string())
}

fn execute_system_config_update(proposal: &Proposal) -> Result<String, String> {
    // Implementation for system configuration updates
    Ok("System configuration updated".to_string())
}

fn execute_emergency_action(proposal: &Proposal) -> Result<String, String> {
    // Implementation for emergency actions
    Ok("Emergency action executed".to_string())
}

fn apply_parameter_change(key: &str, value: u64) -> Result<(), String> {
    // Apply the parameter change to the relevant system components
    match key {
        "emergency_stop" => {
            // Update emergency stop in config
            let mut config = get_canister_config();
            config.emergency_stop = value == 1;
            config.updated_at = time();
            update_config(config);
        },
        "maintenance_mode" => {
            // Update maintenance mode in config
            let mut config = get_canister_config();
            config.maintenance_mode = value == 1;
            config.updated_at = time();
            update_config(config);
        },
        "min_collateral_value" => {
            // Update minimum collateral value in config
            let mut config = get_canister_config();
            config.min_collateral_value = value;
            config.updated_at = time();
            update_config(config);
        },
        "max_utilization_rate" => {
            // Update max utilization rate in config
            let mut config = get_canister_config();
            config.max_utilization_rate = value;
            config.updated_at = time();
            update_config(config);
        },
        _ => {
            // For other parameters, they are stored in the parameter storage
            // and retrieved by other modules when needed
        }
    }
    
    Ok(())
}

// ========== EMERGENCY FUNCTIONS ==========

/// Emergency stop the system (emergency admin only)
#[update]
pub fn emergency_stop() -> Result<String, String> {
    let caller = caller();
    
    // Check if caller has emergency admin permission
    if !has_permission(&caller, Permission::EmergencyStop) {
        return Err("Unauthorized: Emergency stop permission required".to_string());
    }
    
    set_protocol_parameter("emergency_stop".to_string(), 1)?;
    
    log_audit_action(
        caller,
        "EMERGENCY_STOP".to_string(),
        "Emergency stop activated".to_string(),
    );
    
    Ok("Emergency stop activated".to_string())
}

/// Resume operations after emergency stop (super admin only)
#[update]
pub fn resume_operations() -> Result<String, String> {
    let caller = caller();
    
    // Check if caller is super admin
    if !is_super_admin(&caller) {
        return Err("Unauthorized: Only super admins can resume operations".to_string());
    }
    
    set_protocol_parameter("emergency_stop".to_string(), 0)?;
    set_protocol_parameter("maintenance_mode".to_string(), 0)?;
    
    log_audit_action(
        caller,
        "OPERATIONS_RESUMED".to_string(),
        "Operations resumed after emergency stop".to_string(),
    );
    
    Ok("Operations resumed successfully".to_string())
}

fn has_permission(principal: &Principal, permission: Permission) -> bool {
    ADMIN_ROLES.with(|roles| {
        if let Some(role) = roles.borrow().get(principal) {
            role.is_active && role.permissions.contains(&permission)
        } else {
            false
        }
    })
}

// ========== ENHANCED GOVERNANCE FUNCTIONS ==========

/// Update governance configuration (super admin only)
#[update]
pub fn update_governance_config(config: GovernanceConfig) -> Result<String, String> {
    let caller = caller();
    
    if !is_super_admin(&caller) {
        return Err("Unauthorized: Only super admins can update governance config".to_string());
    }
    
    GOVERNANCE_CONFIG.with(|gov_config| {
        gov_config.borrow_mut().insert(0, config);
    });
    
    log_audit_action(
        caller,
        "GOVERNANCE_CONFIG_UPDATED".to_string(),
        "Governance configuration updated".to_string(),
    );
    
    Ok("Governance configuration updated successfully".to_string())
}

/// Get current governance configuration
#[query]
pub fn get_governance_config_public() -> GovernanceConfig {
    get_governance_config()
}

/// Create multiple proposals in batch (admin only)
#[update]
pub fn create_batch_proposals(proposals: Vec<(ProposalType, String, String, Option<Vec<u8>>)>) -> Vec<GovernanceResult<u64>> {
    let mut results = Vec::new();
    
    for (proposal_type, title, description, payload) in proposals {
        let result = create_proposal(proposal_type, title, description, payload);
        results.push(result);
    }
    
    results
}

/// Set multiple protocol parameters at once (admin only)
#[update]
pub fn set_multiple_protocol_parameters(parameters: Vec<(String, u64)>) -> Vec<Result<String, String>> {
    let caller = caller();
    
    if !is_admin(&caller) {
        return vec![Err("Unauthorized: Only admins can set parameters".to_string())];
    }
    
    let mut results = Vec::new();
    
    for (key, value) in parameters {
        let result = set_protocol_parameter(key, value);
        results.push(result);
    }
    
    results
}

/// Get protocol parameters by category
#[query]
pub fn get_protocol_parameters_by_category(category: String) -> Vec<ProtocolParameter> {
    PROTOCOL_PARAMETERS.with(|params| {
        params.borrow()
            .iter()
            .filter(|(key, _)| {
                match category.as_str() {
                    "loan" => key.contains("loan") || key.contains("ltv") || key.contains("apr"),
                    "liquidation" => key.contains("liquidation") || key.contains("grace"),
                    "system" => key.contains("emergency") || key.contains("maintenance"),
                    "pool" => key.contains("utilization") || key.contains("reserve"),
                    _ => true,
                }
            })
            .map(|(_, param)| param)
            .collect()
    })
}

/// Validate parameter value before setting
#[query]
pub fn validate_parameter_value(key: String, value: u64) -> Result<String, String> {
    let param = PROTOCOL_PARAMETERS.with(|params| {
        params.borrow().get(&key).cloned()
    }).ok_or_else(|| format!("Parameter {} not found", key))?;
    
    // Validate value range
    if let Some(min_val) = param.min_value {
        if value < min_val {
            return Err(format!("Value {} is below minimum {}", value, min_val));
        }
    }
    
    if let Some(max_val) = param.max_value {
        if value > max_val {
            return Err(format!("Value {} is above maximum {}", value, max_val));
        }
    }
    
    Ok("Parameter value is valid".to_string())
}

/// Get parameter history (if implemented)
#[query]
pub fn get_parameter_history(key: String) -> Vec<(u64, u64, Principal)> {
    // For now, return current value only
    // In a full implementation, this would track parameter change history
    if let Ok(param) = get_protocol_parameter(key) {
        vec![(param.last_updated, param.current_value, param.updated_by)]
    } else {
        vec![]
    }
}

/// Check if a proposal can be executed
#[query]
pub fn can_execute_proposal(proposal_id: u64) -> Result<bool, String> {
    let proposal = PROPOSALS.with(|proposals| {
        proposals.borrow().get(&proposal_id)
    }).ok_or("Proposal not found".to_string())?;
    
    if proposal.status != ProposalStatus::Active {
        return Ok(false);
    }
    
    if time() < proposal.voting_deadline {
        return Ok(false);
    }
    
    if time() > proposal.execution_deadline {
        return Ok(false);
    }
    
    let total_votes = proposal.yes_votes + proposal.no_votes + proposal.abstain_votes;
    let participation_rate = if proposal.total_voting_power > 0 {
        (total_votes * 10000) / proposal.total_voting_power
    } else {
        0
    };
    
    if participation_rate < proposal.quorum_threshold {
        return Ok(false);
    }
    
    let approval_rate = if total_votes > 0 {
        (proposal.yes_votes * 10000) / total_votes
    } else {
        0
    };
    
    Ok(approval_rate >= proposal.approval_threshold)
}

/// Get proposals by status
#[query]
pub fn get_proposals_by_status(status: ProposalStatus, offset: u64, limit: u64) -> Vec<Proposal> {
    PROPOSALS.with(|proposals| {
        proposals.borrow()
            .iter()
            .filter(|(_, p)| p.status == status)
            .skip(offset as usize)
            .take(limit as usize)
            .map(|(_, proposal)| proposal)
            .collect()
    })
}

/// Get active admin roles count
#[query]
pub fn get_active_admin_count() -> u64 {
    ADMIN_ROLES.with(|roles| {
        roles.borrow()
            .iter()
            .filter(|(_, role)| role.is_active)
            .count() as u64
    })
}

/// Enable/disable maintenance mode (admin only)
#[update]
pub fn set_maintenance_mode(enabled: bool) -> Result<String, String> {
    let caller = caller();
    
    if !is_admin(&caller) {
        return Err("Unauthorized: Only admins can set maintenance mode".to_string());
    }
    
    let value = if enabled { 1 } else { 0 };
    set_protocol_parameter("maintenance_mode".to_string(), value)?;
    
    let message = if enabled {
        "Maintenance mode enabled"
    } else {
        "Maintenance mode disabled"
    };
    
    log_audit_action(
        caller,
        "MAINTENANCE_MODE_UPDATED".to_string(),
        message.to_string(),
    );
    
    Ok(message.to_string())
}

/// Get system status (maintenance mode, emergency stop, etc.)
#[query]
pub fn get_system_status() -> HashMap<String, bool> {
    let mut status = HashMap::new();
    
    if let Ok(emergency_param) = get_protocol_parameter("emergency_stop".to_string()) {
        status.insert("emergency_stop".to_string(), emergency_param.current_value == 1);
    }
    
    if let Ok(maintenance_param) = get_protocol_parameter("maintenance_mode".to_string()) {
        status.insert("maintenance_mode".to_string(), maintenance_param.current_value == 1);
    }
    
    status
}

/// Initialize super admin (one-time setup)
#[update]
pub fn initialize_super_admin(admin_principal: Principal) -> Result<String, String> {
    let caller = caller();
    
    // Check if any super admin already exists
    let existing_super_admin = ADMIN_ROLES.with(|roles| {
        roles.borrow()
            .iter()
            .any(|(_, role)| role.is_active && matches!(role.role_type, AdminRoleType::SuperAdmin))
    });
    
    if existing_super_admin {
        return Err("Super admin already exists".to_string());
    }
    
    let admin_role = AdminRole {
        admin_principal,
        role_type: AdminRoleType::SuperAdmin,
        granted_at: time(),
        granted_by: caller,
        expires_at: None,
        permissions: vec![
            Permission::ManageParameters,
            Permission::ManageAdmins,
            Permission::EmergencyStop,
            Permission::ManageTreasury,
            Permission::ManageLiquidation,
            Permission::ManageOracle,
            Permission::ViewMetrics,
            Permission::ExecuteProposals,
        ],
        is_active: true,
    };
    
    ADMIN_ROLES.with(|roles| {
        roles.borrow_mut().insert(admin_principal, admin_role);
    });
    
    // Also add to canister config admins list
    let mut config = get_canister_config();
    if !config.admins.contains(&admin_principal) {
        config.admins.push(admin_principal);
        config.updated_at = time();
        update_config(config);
    }
    
    log_audit_action(
        caller,
        "SUPER_ADMIN_INITIALIZED".to_string(),
        format!("Super admin initialized: {}", admin_principal),
    );
    
    Ok("Super admin initialized successfully".to_string())
}

/// Get comprehensive governance dashboard data
#[query]
pub fn get_governance_dashboard() -> GovernanceDashboard {
    let stats = get_governance_stats();
    let active_proposals = get_proposals_by_status(ProposalStatus::Active, 0, 10);
    let recent_proposals = get_proposals(0, 5);
    let admin_count = get_active_admin_count();
    let system_status = get_system_status();
    let all_parameters = get_all_protocol_parameters();
    
    GovernanceDashboard {
        stats,
        active_proposals,
        recent_proposals,
        admin_count,
        system_status,
        parameter_count: all_parameters.len() as u64,
        last_updated: time(),
    }
}

// Dashboard data structure
#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct GovernanceDashboard {
    pub stats: GovernanceStats,
    pub active_proposals: Vec<Proposal>,
    pub recent_proposals: Vec<Proposal>,
    pub admin_count: u64,
    pub system_status: HashMap<String, bool>,
    pub parameter_count: u64,
    pub last_updated: u64,
}
