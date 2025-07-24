// EXAMPLE: Integration dengan Governance System Module
// File: src/governance.rs (contoh integrasi)

use ic_cdk::{caller, api::time, id};
use ic_cdk_macros::{query, update};
use candid::Principal;
use std::collections::HashMap;
use crate::types::*;
use crate::storage::*;
use crate::helpers::{is_admin, log_audit_action};

// Import notification system
use crate::notification_system::{
    create_batch_notifications,
    NotificationEvent,
    NotificationPriority,
    NotificationCategory,
};

/// Create a new governance proposal with notifications
#[update]
pub async fn create_proposal_with_notifications(
    title: String,
    description: String,
    proposal_type: ProposalType,
    execution_payload: Option<Vec<u8>>,
) -> Result<u64, String> {
    let caller = caller();
        
    // Validate caller permissions
    let user_data = get_user(&caller).ok_or_else(|| "User not found".to_string())?;
    if user_data.reputation_score < 100 {
        return Err("Insufficient reputation to create proposals".to_string());
    }
    
    // Create the proposal
    let proposal_id = create_proposal(caller, title.clone(), description.clone(), proposal_type.clone(), execution_payload)?;
    
    // Send notification to proposal creator
    let creator_event = NotificationEvent::GovernanceProposalCreated {
        proposal_id,
        title: title.clone(),
        proposal_type: proposal_type.clone(),
        creator: caller,
    };
    
    match crate::notification_system::create_notification(
        caller,
        creator_event.clone(),
        Some(format!("Your governance proposal '{}' has been created and is now open for voting", title)),
        Some(NotificationPriority::High),
        Some(NotificationCategory::Governance),
        Some(true), // Mark as actionable - user can check voting status
    ) {
        Ok(notification_id) => {
            log_audit_action(
                caller,
                "PROPOSAL_CREATED_NOTIFICATION_SENT".to_string(),
                format!("Sent proposal creation notification {} for proposal #{}", notification_id, proposal_id),
                true,
            );
        }
        Err(e) => {
            log_audit_action(
                caller,
                "PROPOSAL_CREATED_NOTIFICATION_FAILED".to_string(),
                format!("Failed to send proposal creation notification: {}", e),
                false,
            );
        }
    }
    
    // Send notifications to all governance participants
    let governance_participants = get_governance_participants();
    
    let participant_event = NotificationEvent::GovernanceProposalCreated {
        proposal_id,
        title: title.clone(),
        proposal_type: proposal_type.clone(),
        creator: caller,
    };
    
    match create_batch_notifications(
        governance_participants,
        participant_event,
        Some(format!("New governance proposal: '{}' by {}. Please review and vote.", title, caller.to_text())),
        Some(NotificationPriority::Medium)
    ) {
        Ok(notification_ids) => {
            log_audit_action(
                caller,
                "PROPOSAL_BATCH_NOTIFICATIONS_SENT".to_string(),
                format!("Sent {} notifications to governance participants for proposal #{}", notification_ids.len(), proposal_id),
                true,
            );
        }
        Err(e) => {
            log_audit_action(
                caller,
                "PROPOSAL_BATCH_NOTIFICATIONS_FAILED".to_string(),
                format!("Failed to send batch notifications for proposal #{}: {}", proposal_id, e),
                false,
            );
        }
    }
    
    // Send notification to admins for high-risk proposals
    if matches!(proposal_type, ProposalType::SystemUpgrade | ProposalType::ParameterChange | ProposalType::EmergencyAction) {
        let admin_principals = get_admin_principals();
        
        let admin_event = NotificationEvent::Custom {
            event_type: "high_risk_proposal_created".to_string(),
            data: {
                let mut data = HashMap::new();
                data.insert("proposal_id".to_string(), proposal_id.to_string());
                data.insert("creator".to_string(), caller.to_text());
                data.insert("title".to_string(), title.clone());
                data.insert("type".to_string(), format!("{:?}", proposal_type));
                data.insert("timestamp".to_string(), time().to_string());
                data
            },
        };
        
        match create_batch_notifications(
            admin_principals,
            admin_event,
            Some(format!("HIGH RISK PROPOSAL: '{}' created by {}. Requires admin attention.", title, caller.to_text())),
            Some(NotificationPriority::Critical)
        ) {
            Ok(notification_ids) => {
                log_audit_action(
                    caller,
                    "HIGH_RISK_PROPOSAL_ADMIN_NOTIFICATIONS_SENT".to_string(),
                    format!("Sent {} high-risk proposal notifications to admins", notification_ids.len()),
                    true,
                );
            }
            Err(e) => {
                log_audit_action(
                    caller,
                    "HIGH_RISK_PROPOSAL_ADMIN_NOTIFICATIONS_FAILED".to_string(),
                    format!("Failed to send high-risk proposal notifications to admins: {}", e),
                    false,
                );
            }
        }
    }
    
    log_audit_action(
        caller,
        "PROPOSAL_CREATED_WITH_NOTIFICATIONS".to_string(),
        format!("Created proposal #{} with notification system integration", proposal_id),
        true,
    );
    
    Ok(proposal_id)
}

/// Vote on a governance proposal with notifications
#[update]
pub async fn vote_on_proposal_with_notifications(
    proposal_id: u64,
    vote: Vote,
    rationale: Option<String>,
) -> Result<(), String> {
    let caller = caller();
    
    // Validate voting eligibility
    let user_data = get_user(&caller).ok_or_else(|| "User not found".to_string())?;
    if user_data.reputation_score < 50 {
        return Err("Insufficient reputation to vote".to_string());
    }
    
    // Cast the vote
    cast_vote(caller, proposal_id, vote.clone(), rationale.clone())?;
    
    let proposal = get_proposal(proposal_id).ok_or_else(|| "Proposal not found".to_string())?;
    
    // Send notification to voter
    let voter_event = NotificationEvent::GovernanceVoteCast {
        proposal_id,
        voter: caller,
        vote: vote.clone(),
        voting_power: user_data.governance_power,
    };
    
    match crate::notification_system::create_notification(
        caller,
        voter_event,
        Some(format!("Your vote has been recorded for proposal: '{}'", proposal.title)),
        Some(NotificationPriority::Low),
        Some(NotificationCategory::Governance),
        Some(true), // Actionable - user can change vote if allowed
    ) {
        Ok(notification_id) => {
            log_audit_action(
                caller,
                "VOTE_CAST_NOTIFICATION_SENT".to_string(),
                format!("Sent vote cast notification {} for proposal #{}", notification_id, proposal_id),
                true,
            );
        }
        Err(e) => {
            log_audit_action(
                caller,
                "VOTE_CAST_NOTIFICATION_FAILED".to_string(),
                format!("Failed to send vote cast notification: {}", e),
                false,
            );
        }
    }
    
    // Send notification to proposal creator
    let creator_event = NotificationEvent::Custom {
        event_type: "proposal_vote_received".to_string(),
        data: {
            let mut data = HashMap::new();
            data.insert("proposal_id".to_string(), proposal_id.to_string());
            data.insert("voter".to_string(), caller.to_text());
            data.insert("vote".to_string(), format!("{:?}", vote));
            data.insert("voting_power".to_string(), user_data.governance_power.to_string());
            if let Some(rationale) = &rationale {
                data.insert("rationale".to_string(), rationale.clone());
            }
            data.insert("timestamp".to_string(), time().to_string());
            data
        },
    };
    
    let creator_message = match vote {
        Vote::Yes => format!("Your proposal '{}' received a YES vote from {}", proposal.title, caller.to_text()),
        Vote::No => format!("Your proposal '{}' received a NO vote from {}", proposal.title, caller.to_text()),
        Vote::Abstain => format!("Your proposal '{}' received an ABSTAIN vote from {}", proposal.title, caller.to_text()),
    };
    
    match crate::notification_system::create_notification(
        proposal.creator,
        creator_event,
        Some(creator_message),
        Some(NotificationPriority::Low),
        Some(NotificationCategory::Governance),
        Some(false), // Not actionable for creator
    ) {
        Ok(notification_id) => {
            log_audit_action(
                caller,
                "PROPOSAL_CREATOR_VOTE_NOTIFICATION_SENT".to_string(),
                format!("Sent vote notification {} to proposal creator", notification_id),
                true,
            );
        }
        Err(e) => {
            log_audit_action(
                caller,
                "PROPOSAL_CREATOR_VOTE_NOTIFICATION_FAILED".to_string(),
                format!("Failed to send vote notification to proposal creator: {}", e),
                false,
            );
        }
    }
    
    // Check if proposal has reached quorum or decisive outcome
    let vote_stats = get_proposal_vote_stats(proposal_id)?;
    
    // Check for quorum reached
    if vote_stats.total_votes >= proposal.quorum_threshold && !proposal.quorum_reached {
        // Mark quorum as reached and send notifications
        mark_proposal_quorum_reached(proposal_id)?;
        
        let quorum_event = NotificationEvent::Custom {
            event_type: "proposal_quorum_reached".to_string(),
            data: {
                let mut data = HashMap::new();
                data.insert("proposal_id".to_string(), proposal_id.to_string());
                data.insert("total_votes".to_string(), vote_stats.total_votes.to_string());
                data.insert("quorum_threshold".to_string(), proposal.quorum_threshold.to_string());
                data.insert("timestamp".to_string(), time().to_string());
                data
            },
        };
        
        // Notify all governance participants
        let governance_participants = get_governance_participants();
        
        match create_batch_notifications(
            governance_participants,
            quorum_event.clone(),
            Some(format!("Proposal '{}' has reached quorum! Final voting period is now active.", proposal.title)),
            Some(NotificationPriority::High)
        ) {
            Ok(notification_ids) => {
                log_audit_action(
                    caller,
                    "QUORUM_REACHED_NOTIFICATIONS_SENT".to_string(),
                    format!("Sent {} quorum reached notifications for proposal #{}", notification_ids.len(), proposal_id),
                    true,
                );
            }
            Err(e) => {
                log_audit_action(
                    caller,
                    "QUORUM_REACHED_NOTIFICATIONS_FAILED".to_string(),
                    format!("Failed to send quorum reached notifications: {}", e),
                    false,
                );
            }
        }
        
        // Send special notification to proposal creator
        match crate::notification_system::create_notification(
            proposal.creator,
            quorum_event,
            Some(format!("Great news! Your proposal '{}' has reached quorum and is now in the final voting period.", proposal.title)),
            Some(NotificationPriority::High),
            Some(NotificationCategory::Governance),
            Some(true), // Actionable - creator can monitor final votes
        ) {
            Ok(notification_id) => {
                log_audit_action(
                    proposal.creator,
                    "CREATOR_QUORUM_NOTIFICATION_SENT".to_string(),
                    format!("Sent quorum reached notification {} to proposal creator", notification_id),
                    true,
                );
            }
            Err(e) => {
                log_audit_action(
                    proposal.creator,
                    "CREATOR_QUORUM_NOTIFICATION_FAILED".to_string(),
                    format!("Failed to send quorum notification to creator: {}", e),
                    false,
                );
            }
        }
    }
    
    // Check for early decisive outcome (e.g., overwhelming majority)
    let total_possible_votes = get_total_governance_power();
    let decisive_threshold = (total_possible_votes * 80) / 100; // 80% of total power
    
    if vote_stats.yes_votes >= decisive_threshold && proposal.status == ProposalStatus::Active {
        // Proposal can be executed early due to overwhelming support
        let early_success_event = NotificationEvent::Custom {
            event_type: "proposal_early_success".to_string(),
            data: {
                let mut data = HashMap::new();
                data.insert("proposal_id".to_string(), proposal_id.to_string());
                data.insert("yes_votes".to_string(), vote_stats.yes_votes.to_string());
                data.insert("total_possible".to_string(), total_possible_votes.to_string());
                data.insert("percentage".to_string(), ((vote_stats.yes_votes * 100) / total_possible_votes).to_string());
                data.insert("timestamp".to_string(), time().to_string());
                data
            },
        };
        
        // Notify all participants about early success
        let governance_participants = get_governance_participants();
        
        match create_batch_notifications(
            governance_participants,
            early_success_event.clone(),
            Some(format!("Proposal '{}' has achieved overwhelming support ({}%) and can be executed early!", 
                        proposal.title, (vote_stats.yes_votes * 100) / total_possible_votes)),
            Some(NotificationPriority::Critical)
        ) {
            Ok(notification_ids) => {
                log_audit_action(
                    caller,
                    "EARLY_SUCCESS_NOTIFICATIONS_SENT".to_string(),
                    format!("Sent {} early success notifications for proposal #{}", notification_ids.len(), proposal_id),
                    true,
                );
            }
            Err(e) => {
                log_audit_action(
                    caller,
                    "EARLY_SUCCESS_NOTIFICATIONS_FAILED".to_string(),
                    format!("Failed to send early success notifications: {}", e),
                    false,
                );
            }
        }
    }
    
    log_audit_action(
        caller,
        "VOTE_CAST_WITH_NOTIFICATIONS".to_string(),
        format!("Cast vote on proposal #{} with notification system integration", proposal_id),
        true,
    );
    
    Ok(())
}

/// Execute a governance proposal with notifications
#[update]
pub async fn execute_proposal_with_notifications(proposal_id: u64) -> Result<(), String> {
    let caller = caller();
    
    // Only admins or the system can execute proposals
    if !is_admin(&caller) && caller != id() {
        return Err("Unauthorized: Only admins can execute proposals".to_string());
    }
    
    let mut proposal = get_proposal(proposal_id).ok_or_else(|| "Proposal not found".to_string())?;
    
    // Validate proposal can be executed
    if proposal.status != ProposalStatus::Passed {
        return Err("Proposal cannot be executed in current status".to_string());
    }
    
    // Execute the proposal
    let execution_result = execute_proposal_logic(proposal_id).await;
    
    match execution_result {
        Ok(execution_data) => {
            // Mark proposal as executed
            proposal.status = ProposalStatus::Executed;
            proposal.executed_at = Some(time());
            store_proposal(proposal.clone())?;
            
            // Send success notification to proposal creator
            let creator_success_event = NotificationEvent::GovernanceProposalExecuted {
                proposal_id,
                title: proposal.title.clone(),
                execution_result: "Success".to_string(),
                executed_by: caller,
            };
            
            match crate::notification_system::create_notification(
                proposal.creator,
                creator_success_event.clone(),
                Some(format!("Your proposal '{}' has been successfully executed!", proposal.title)),
                Some(NotificationPriority::High),
                Some(NotificationCategory::Governance),
                Some(true), // Actionable - creator can view execution details
            ) {
                Ok(notification_id) => {
                    log_audit_action(
                        caller,
                        "PROPOSAL_EXECUTION_SUCCESS_NOTIFICATION_SENT".to_string(),
                        format!("Sent execution success notification {} to proposal creator", notification_id),
                        true,
                    );
                }
                Err(e) => {
                    log_audit_action(
                        caller,
                        "PROPOSAL_EXECUTION_SUCCESS_NOTIFICATION_FAILED".to_string(),
                        format!("Failed to send execution success notification: {}", e),
                        false,
                    );
                }
            }
            
            // Send notifications to all governance participants
            let governance_participants = get_governance_participants();
            
            match create_batch_notifications(
                governance_participants,
                creator_success_event,
                Some(format!("Proposal '{}' has been executed successfully. Changes are now in effect.", proposal.title)),
                Some(NotificationPriority::Medium)
            ) {
                Ok(notification_ids) => {
                    log_audit_action(
                        caller,
                        "PROPOSAL_EXECUTION_BATCH_NOTIFICATIONS_SENT".to_string(),
                        format!("Sent {} execution success notifications for proposal #{}", notification_ids.len(), proposal_id),
                        true,
                    );
                }
                Err(e) => {
                    log_audit_action(
                        caller,
                        "PROPOSAL_EXECUTION_BATCH_NOTIFICATIONS_FAILED".to_string(),
                        format!("Failed to send execution success batch notifications: {}", e),
                        false,
                    );
                }
            }
            
            // Send detailed notification to admins
            let admin_principals = get_admin_principals();
            
            let admin_execution_event = NotificationEvent::Custom {
                event_type: "proposal_executed_admin_details".to_string(),
                data: {
                    let mut data = HashMap::new();
                    data.insert("proposal_id".to_string(), proposal_id.to_string());
                    data.insert("title".to_string(), proposal.title.clone());
                    data.insert("executor".to_string(), caller.to_text());
                    data.insert("execution_data".to_string(), execution_data);
                    data.insert("timestamp".to_string(), time().to_string());
                    data
                },
            };
            
            match create_batch_notifications(
                admin_principals,
                admin_execution_event,
                Some(format!("ADMIN: Proposal '{}' executed successfully by {}. Review execution details.", proposal.title, caller.to_text())),
                Some(NotificationPriority::High)
            ) {
                Ok(notification_ids) => {
                    log_audit_action(
                        caller,
                        "ADMIN_EXECUTION_NOTIFICATIONS_SENT".to_string(),
                        format!("Sent {} admin execution notifications for proposal #{}", notification_ids.len(), proposal_id),
                        true,
                    );
                }
                Err(e) => {
                    log_audit_action(
                        caller,
                        "ADMIN_EXECUTION_NOTIFICATIONS_FAILED".to_string(),
                        format!("Failed to send admin execution notifications: {}", e),
                        false,
                    );
                }
            }
            
            log_audit_action(
                caller,
                "PROPOSAL_EXECUTED_SUCCESSFULLY".to_string(),
                format!("Successfully executed proposal #{} with notifications", proposal_id),
                true,
            );
            
            Ok(())
        }
        Err(execution_error) => {
            // Mark proposal as failed
            proposal.status = ProposalStatus::Failed;
            store_proposal(proposal.clone())?;
            
            // Send failure notification to proposal creator
            let creator_failure_event = NotificationEvent::GovernanceProposalExecuted {
                proposal_id,
                title: proposal.title.clone(),
                execution_result: format!("Failed: {}", execution_error),
                executed_by: caller,
            };
            
            match crate::notification_system::create_notification(
                proposal.creator,
                creator_failure_event.clone(),
                Some(format!("Your proposal '{}' failed to execute: {}", proposal.title, execution_error)),
                Some(NotificationPriority::High),
                Some(NotificationCategory::Governance),
                Some(true), // Actionable - creator can view failure details
            ) {
                Ok(notification_id) => {
                    log_audit_action(
                        caller,
                        "PROPOSAL_EXECUTION_FAILURE_NOTIFICATION_SENT".to_string(),
                        format!("Sent execution failure notification {} to proposal creator", notification_id),
                        true,
                    );
                }
                Err(e) => {
                    log_audit_action(
                        caller,
                        "PROPOSAL_EXECUTION_FAILURE_NOTIFICATION_FAILED".to_string(),
                        format!("Failed to send execution failure notification: {}", e),
                        false,
                    );
                }
            }
            
            // Send failure notifications to admins (critical priority)
            let admin_principals = get_admin_principals();
            
            let admin_failure_event = NotificationEvent::Custom {
                event_type: "proposal_execution_failed_admin".to_string(),
                data: {
                    let mut data = HashMap::new();
                    data.insert("proposal_id".to_string(), proposal_id.to_string());
                    data.insert("title".to_string(), proposal.title.clone());
                    data.insert("executor".to_string(), caller.to_text());
                    data.insert("error".to_string(), execution_error.clone());
                    data.insert("timestamp".to_string(), time().to_string());
                    data
                },
            };
            
            match create_batch_notifications(
                admin_principals,
                admin_failure_event,
                Some(format!("CRITICAL: Proposal '{}' execution failed: {}. Admin intervention may be required.", proposal.title, execution_error)),
                Some(NotificationPriority::Critical)
            ) {
                Ok(notification_ids) => {
                    log_audit_action(
                        caller,
                        "ADMIN_EXECUTION_FAILURE_NOTIFICATIONS_SENT".to_string(),
                        format!("Sent {} admin failure notifications for proposal #{}", notification_ids.len(), proposal_id),
                        true,
                    );
                }
                Err(e) => {
                    log_audit_action(
                        caller,
                        "ADMIN_EXECUTION_FAILURE_NOTIFICATIONS_FAILED".to_string(),
                        format!("Failed to send admin failure notifications: {}", e),
                        false,
                    );
                }
            }
            
            log_audit_action(
                caller,
                "PROPOSAL_EXECUTION_FAILED".to_string(),
                format!("Failed to execute proposal #{}: {}", proposal_id, execution_error),
                false,
            );
            
            Err(execution_error)
        }
    }
}

/// Close expired proposals with notifications
#[update]
pub async fn close_expired_proposals_with_notifications() -> Result<Vec<u64>, String> {
    let caller = caller();
    
    if !is_admin(&caller) {
        return Err("Unauthorized: Only admins can close expired proposals".to_string());
    }
    
    let current_time = time();
    let mut closed_proposals = Vec::new();
    
    // Get all active proposals
    let active_proposals = get_active_proposals();
    
    for mut proposal in active_proposals {
        if current_time > proposal.voting_end_time {
            // Calculate voting results
            let vote_stats = get_proposal_vote_stats(proposal.id)?;
            
            // Determine final status
            let new_status = if vote_stats.total_votes >= proposal.quorum_threshold {
                if vote_stats.yes_votes > vote_stats.no_votes {
                    ProposalStatus::Passed
                } else {
                    ProposalStatus::Rejected
                }
            } else {
                ProposalStatus::Rejected // Failed to reach quorum
            };
            
            proposal.status = new_status.clone();
            proposal.closed_at = Some(current_time);
            store_proposal(proposal.clone())?;
            
            closed_proposals.push(proposal.id);
            
            // Send notification to proposal creator
            let creator_event = match new_status {
                ProposalStatus::Passed => NotificationEvent::Custom {
                    event_type: "proposal_passed".to_string(),
                    data: {
                        let mut data = HashMap::new();
                        data.insert("proposal_id".to_string(), proposal.id.to_string());
                        data.insert("title".to_string(), proposal.title.clone());
                        data.insert("yes_votes".to_string(), vote_stats.yes_votes.to_string());
                        data.insert("no_votes".to_string(), vote_stats.no_votes.to_string());
                        data.insert("total_votes".to_string(), vote_stats.total_votes.to_string());
                        data
                    },
                },
                _ => NotificationEvent::Custom {
                    event_type: "proposal_rejected".to_string(),
                    data: {
                        let mut data = HashMap::new();
                        data.insert("proposal_id".to_string(), proposal.id.to_string());
                        data.insert("title".to_string(), proposal.title.clone());
                        data.insert("yes_votes".to_string(), vote_stats.yes_votes.to_string());
                        data.insert("no_votes".to_string(), vote_stats.no_votes.to_string());
                        data.insert("total_votes".to_string(), vote_stats.total_votes.to_string());
                        data.insert("quorum_reached".to_string(), (vote_stats.total_votes >= proposal.quorum_threshold).to_string());
                        data
                    },
                },
            };
            
            let creator_message = match new_status {
                ProposalStatus::Passed => {
                    format!("Great news! Your proposal '{}' has passed with {} YES votes vs {} NO votes and will be scheduled for execution.", 
                           proposal.title, vote_stats.yes_votes, vote_stats.no_votes)
                }
                _ => {
                    if vote_stats.total_votes < proposal.quorum_threshold {
                        format!("Your proposal '{}' was rejected due to insufficient participation ({} votes, {} required for quorum).", 
                               proposal.title, vote_stats.total_votes, proposal.quorum_threshold)
                    } else {
                        format!("Your proposal '{}' was rejected with {} YES votes vs {} NO votes.", 
                               proposal.title, vote_stats.yes_votes, vote_stats.no_votes)
                    }
                }
            };
            
            let creator_priority = match new_status {
                ProposalStatus::Passed => NotificationPriority::High,
                _ => NotificationPriority::Medium,
            };
            
            match crate::notification_system::create_notification(
                proposal.creator,
                creator_event.clone(),
                Some(creator_message),
                Some(creator_priority),
                Some(NotificationCategory::Governance),
                Some(true), // Actionable - creator can view detailed results
            ) {
                Ok(notification_id) => {
                    log_audit_action(
                        caller,
                        "PROPOSAL_CLOSURE_NOTIFICATION_SENT".to_string(),
                        format!("Sent closure notification {} to proposal creator for #{}", notification_id, proposal.id),
                        true,
                    );
                }
                Err(e) => {
                    log_audit_action(
                        caller,
                        "PROPOSAL_CLOSURE_NOTIFICATION_FAILED".to_string(),
                        format!("Failed to send closure notification to creator: {}", e),
                        false,
                    );
                }
            }
            
            // Send notifications to all governance participants
            let governance_participants = get_governance_participants();
            
            let participant_message = match new_status {
                ProposalStatus::Passed => {
                    format!("Proposal '{}' has PASSED and will be executed. Final votes: {} YES, {} NO", 
                           proposal.title, vote_stats.yes_votes, vote_stats.no_votes)
                }
                _ => {
                    format!("Proposal '{}' has been REJECTED. Final votes: {} YES, {} NO", 
                           proposal.title, vote_stats.yes_votes, vote_stats.no_votes)
                }
            };
            
            match create_batch_notifications(
                governance_participants,
                creator_event,
                Some(participant_message),
                Some(NotificationPriority::Medium)
            ) {
                Ok(notification_ids) => {
                    log_audit_action(
                        caller,
                        "PROPOSAL_CLOSURE_BATCH_NOTIFICATIONS_SENT".to_string(),
                        format!("Sent {} closure notifications for proposal #{}", notification_ids.len(), proposal.id),
                        true,
                    );
                }
                Err(e) => {
                    log_audit_action(
                        caller,
                        "PROPOSAL_CLOSURE_BATCH_NOTIFICATIONS_FAILED".to_string(),
                        format!("Failed to send closure batch notifications: {}", e),
                        false,
                    );
                }
            }
            
            // Schedule execution for passed proposals
            if matches!(new_status, ProposalStatus::Passed) {
                schedule_proposal_execution(proposal.id, current_time + (24 * 60 * 60 * 1_000_000_000))?; // Execute in 24 hours
            }
        }
    }
    
    // Send summary to admins if proposals were closed
    if !closed_proposals.is_empty() {
        let admin_principals = get_admin_principals();
        
        let admin_summary_event = NotificationEvent::Custom {
            event_type: "proposals_closed_summary".to_string(),
            data: {
                let mut data = HashMap::new();
                data.insert("closed_count".to_string(), closed_proposals.len().to_string());
                data.insert("proposal_ids".to_string(), format!("{:?}", closed_proposals));
                data.insert("timestamp".to_string(), current_time.to_string());
                data
            },
        };
        
        match create_batch_notifications(
            admin_principals,
            admin_summary_event,
            Some(format!("Admin Summary: {} governance proposals have been closed and processed.", closed_proposals.len())),
            Some(NotificationPriority::Low)
        ) {
            Ok(notification_ids) => {
                log_audit_action(
                    caller,
                    "ADMIN_CLOSURE_SUMMARY_SENT".to_string(),
                    format!("Sent closure summary to {} admins", notification_ids.len()),
                    true,
                );
            }
            Err(e) => {
                log_audit_action(
                    caller,
                    "ADMIN_CLOSURE_SUMMARY_FAILED".to_string(),
                    format!("Failed to send closure summary to admins: {}", e),
                    false,
                );
            }
        }
    }
    
    log_audit_action(
        caller,
        "EXPIRED_PROPOSALS_CLOSED".to_string(),
        format!("Closed {} expired proposals with notifications", closed_proposals.len()),
        true,
    );
    
    Ok(closed_proposals)
}

/// Get governance notifications for a user
#[query]
pub fn get_governance_notifications(user: Principal, limit: Option<u64>) -> Vec<crate::notification_system::NotificationRecord> {
    let limit = limit.unwrap_or(50);
    
    // Get user's notifications and filter for governance-related ones
    let user_notifications = crate::notification_system::get_user_notifications(user, Some(limit * 2)); // Get more to filter
    
    user_notifications
        .into_iter()
        .filter(|notification| {
            matches!(notification.category, Some(NotificationCategory::Governance)) ||
            matches!(notification.event, 
                NotificationEvent::GovernanceProposalCreated { .. } |
                NotificationEvent::GovernanceVoteCast { .. } |
                NotificationEvent::GovernanceProposalExecuted { .. } |
                NotificationEvent::Custom { ref event_type, .. } if event_type.starts_with("proposal_") || event_type.contains("governance")
            )
        })
        .take(limit as usize)
        .collect()
}

// Helper functions and structures

#[derive(candid::CandidType, candid::Deserialize, Clone, Debug)]
pub enum ProposalType {
    ParameterChange,
    SystemUpgrade,
    TreasuryAllocation,
    EmergencyAction,
    GovernanceChange,
    Other(String),
}

#[derive(candid::CandidType, candid::Deserialize, Clone, Debug, PartialEq)]
pub enum ProposalStatus {
    Active,
    Passed,
    Rejected,
    Executed,
    Failed,
}

#[derive(candid::CandidType, candid::Deserialize, Clone, Debug)]
pub enum Vote {
    Yes,
    No,
    Abstain,
}

#[derive(candid::CandidType, candid::Deserialize, Clone, Debug)]
pub struct VoteStats {
    pub yes_votes: u64,
    pub no_votes: u64,
    pub abstain_votes: u64,
    pub total_votes: u64,
}

/// Get all governance participants (users with governance power > 0)
fn get_governance_participants() -> Vec<Principal> {
    // Implementation would return users eligible for governance notifications
    get_admin_principals() // Placeholder - return admins for now
}

/// Get admin principals
fn get_admin_principals() -> Vec<Principal> {
    let config = get_canister_config();
    config.admins
}

/// Execute proposal logic based on proposal type
async fn execute_proposal_logic(proposal_id: u64) -> Result<String, String> {
    let proposal = get_proposal(proposal_id).ok_or_else(|| "Proposal not found".to_string())?;
    
    match proposal.proposal_type {
        ProposalType::ParameterChange => {
            // Execute parameter changes
            execute_parameter_changes(&proposal.execution_payload)
        }
        ProposalType::SystemUpgrade => {
            // Execute system upgrade
            execute_system_upgrade(&proposal.execution_payload).await
        }
        ProposalType::TreasuryAllocation => {
            // Execute treasury allocation
            execute_treasury_allocation(&proposal.execution_payload)
        }
        ProposalType::EmergencyAction => {
            // Execute emergency action
            execute_emergency_action(&proposal.execution_payload).await
        }
        ProposalType::GovernanceChange => {
            // Execute governance changes
            execute_governance_changes(&proposal.execution_payload)
        }
        ProposalType::Other(_) => {
            Err("Custom proposal execution not implemented".to_string())
        }
    }
}

/// Execute parameter changes
fn execute_parameter_changes(payload: &Option<Vec<u8>>) -> Result<String, String> {
    // Implementation would decode payload and apply parameter changes
    Ok("Parameter changes applied successfully".to_string())
}

/// Execute system upgrade
async fn execute_system_upgrade(payload: &Option<Vec<u8>>) -> Result<String, String> {
    // Implementation would perform system upgrade
    Ok("System upgrade completed successfully".to_string())
}

/// Execute treasury allocation
fn execute_treasury_allocation(payload: &Option<Vec<u8>>) -> Result<String, String> {
    // Implementation would allocate treasury funds
    Ok("Treasury allocation completed successfully".to_string())
}

/// Execute emergency action
async fn execute_emergency_action(payload: &Option<Vec<u8>>) -> Result<String, String> {
    // Implementation would perform emergency actions
    Ok("Emergency action executed successfully".to_string())
}

/// Execute governance changes
fn execute_governance_changes(payload: &Option<Vec<u8>>) -> Result<String, String> {
    // Implementation would apply governance parameter changes
    Ok("Governance changes applied successfully".to_string())
}

/// Schedule proposal execution
fn schedule_proposal_execution(proposal_id: u64, execution_time: u64) -> Result<(), String> {
    // Implementation would schedule proposal for execution
    Ok(())
}
