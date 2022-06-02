mod temp_override;

use std::ops::{Deref, DerefMut};

use anchor_lang::prelude::*;

use solana_program::program_pack::IsInitialized;
pub use temp_override::VoterWeightAction;

use crate::error::GovernanceRewardsError;

use super::distribution::Distribution;

#[derive(Clone)]
pub struct VoterWeightRecord(temp_override::VoterWeightRecord);

impl AccountDeserialize for VoterWeightRecord {
    fn try_deserialize_unchecked(buf: &mut &[u8]) -> Result<Self> {
        let record =
            AnchorDeserialize::deserialize(buf).map_err(|_| ErrorCode::AccountDidNotDeserialize)?;
        Ok(VoterWeightRecord(record))
    }

    fn try_deserialize(buf: &mut &[u8]) -> Result<Self> {
        let record = Self::try_deserialize_unchecked(buf)?;
        if !record.is_initialized() {
            return Err(ErrorCode::AccountDidNotDeserialize.into());
        }
        Ok(record)
    }
}

impl AccountSerialize for VoterWeightRecord {}

impl AnchorSerialize for VoterWeightRecord {
    fn serialize<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        self.0.serialize(writer)
    }
}

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

pub struct Unvalidated<T> {
    inner: T,
}

impl Unvalidated<VoterWeightRecord> {
    pub fn validate(
        self,
        distribution: &Account<Distribution>,
        registrant: &Pubkey,
    ) -> Result<VoterWeightRecord> {
        require!(
            self.inner.is_not_expired(Clock::get()?),
            GovernanceRewardsError::OutdatedVoteWeightRecord
        );
        require!(
            self.inner.weight_action == Some(VoterWeightAction::RegisterForRewards),
            GovernanceRewardsError::WrongAction
        );
        require!(
            self.inner.weight_action_target.is_none()
                || self.inner.weight_action_target == Some(distribution.key()),
            GovernanceRewardsError::WrongActionTarget
        );
        require!(
            self.inner.realm == distribution.realm,
            GovernanceRewardsError::WrongRealm
        );
        require!(
            self.inner.governing_token_owner == registrant.key(),
            GovernanceRewardsError::WrongRegistrant
        );

        Ok(self.inner)
    }
}

impl VoterWeightRecord {
    pub fn is_not_expired(&self, clock: Clock) -> bool {
        match self.voter_weight_expiry {
            Some(slot) => slot >= clock.slot,
            None => true,
        }
    }

    pub fn try_from(account: &AccountInfo<'_>) -> Result<Unvalidated<Self>> {
        let mut data: &[u8] = &account.try_borrow_data()?;
        VoterWeightRecord::try_deserialize(&mut data)
            .map(|record| Unvalidated { inner: record })
            .map_err(|_| ErrorCode::AccountDidNotDeserialize.into())
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
