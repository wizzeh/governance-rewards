mod temp_override;

use std::ops::{Deref, DerefMut};

use anchor_lang::prelude::*;

pub use temp_override::VoterWeightAction;

#[derive(Clone)]
pub struct VoterWeightRecord(temp_override::VoterWeightRecord);

impl AccountDeserialize for VoterWeightRecord {
    fn try_deserialize_unchecked(buf: &mut &[u8]) -> Result<Self> {
        let record =
            AnchorDeserialize::deserialize(buf).map_err(|_| ErrorCode::AccountDidNotDeserialize)?;
        Ok(VoterWeightRecord(record))
    }
}

impl AccountSerialize for VoterWeightRecord {}

impl Deref for VoterWeightRecord {
    type Target = temp_override::VoterWeightRecord;

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

    pub fn create_test(
        realm: Pubkey,
        governing_token_mint: Pubkey,
        governing_token_owner: Pubkey,
        distribution: Pubkey,
        weight: u64,
        expiry: Option<u64>,
    ) -> Self {
        VoterWeightRecord(temp_override::VoterWeightRecord {
            account_discriminator: temp_override::VoterWeightRecord::ACCOUNT_DISCRIMINATOR,
            realm,
            governing_token_mint,
            governing_token_owner,
            voter_weight: weight,
            voter_weight_expiry: expiry,
            weight_action: Some(temp_override::VoterWeightAction::RegisterForRewards),
            weight_action_target: Some(distribution),
            reserved: [0_u8; 8],
        })
    }
}
