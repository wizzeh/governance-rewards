use std::ops::{Deref, DerefMut};

use anchor_lang::prelude::*;
use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};

#[derive(Clone)]
pub struct VoterWeightRecord(spl_governance_addin_api::voter_weight::VoterWeightRecord);

impl AccountDeserialize for VoterWeightRecord {
    fn try_deserialize_unchecked(buf: &mut &[u8]) -> Result<Self> {
        let record =
            AnchorDeserialize::deserialize(buf).map_err(|_| ErrorCode::AccountDidNotDeserialize)?;
        Ok(VoterWeightRecord(record))
    }
}

impl AccountSerialize for VoterWeightRecord {}

impl Deref for VoterWeightRecord {
    type Target = spl_governance_addin_api::voter_weight::VoterWeightRecord;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for VoterWeightRecord {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Owner for VoterWeightRecord {
    fn owner() -> Pubkey {
        // This program does not issue Voter Weight Records, so this should not normally be used.
        crate::ID
    }
}

impl VoterWeightRecord {
    pub fn is_still_valid(&self, clock: Clock) -> bool {
        match self.voter_weight_expiry {
            Some(slot) => slot >= clock.slot,
            None => true,
        }
    }
}

#[derive(Clone, Debug, PartialEq, BorshDeserialize, BorshSerialize, BorshSchema)]
pub enum VoterWeightAction {
    /// Cast vote for a proposal. Target: Proposal
    CastVote,

    /// Comment a proposal. Target: Proposal
    CommentProposal,

    /// Create Governance within a realm. Target: Realm
    CreateGovernance,

    /// Create a proposal for a governance. Target: Governance
    CreateProposal,

    /// Signs off a proposal for a governance. Target: Proposal
    /// Note: SignOffProposal is not supported in the current version
    SignOffProposal,

    // Register for rewards distribution
    RegisterForRewards,
}

impl Into<spl_governance_addin_api::voter_weight::VoterWeightAction> for VoterWeightAction {
    fn into(self) -> spl_governance_addin_api::voter_weight::VoterWeightAction {
        match self {
            Self::CastVote => spl_governance_addin_api::voter_weight::VoterWeightAction::CastVote,
            Self::CommentProposal => {
                spl_governance_addin_api::voter_weight::VoterWeightAction::CommentProposal
            }
            Self::CreateGovernance => {
                spl_governance_addin_api::voter_weight::VoterWeightAction::CreateGovernance
            }
            Self::CreateProposal => {
                spl_governance_addin_api::voter_weight::VoterWeightAction::CreateProposal
            }
            Self::SignOffProposal => {
                spl_governance_addin_api::voter_weight::VoterWeightAction::SignOffProposal
            }
            Self::RegisterForRewards => todo!(),
        }
    }
}
