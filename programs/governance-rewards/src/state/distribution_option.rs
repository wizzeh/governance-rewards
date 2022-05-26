use std::ops::{Deref, DerefMut};

use anchor_lang::prelude::*;
use anchor_spl::token::TokenAccount;

use crate::error::GovernanceRewardsError;

#[derive(AnchorDeserialize, AnchorSerialize, Clone, Default, Copy, Debug, PartialEq, Eq)]
pub struct DistributionOption {
    pub total_vote_weight: u64,
    pub total_amount: u64,
    pub extra_reclaimed: bool,
    pub mint: Pubkey,
    pub wallet: Pubkey,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default, Copy, Debug, PartialEq, Eq)]
pub struct DistributionOptions([Option<DistributionOption>; 8]);

impl DistributionOptions {
    pub fn pick_by_mint(&mut self, key: Option<Pubkey>) -> Result<(u8, &mut DistributionOption)> {
        for (i, option) in self.iter_mut().flatten().enumerate() {
            if key.is_none() || key == Some(option.mint) {
                return Ok((i as u8, option));
            }
        }

        Err(GovernanceRewardsError::NoDistributionOptions.into())
    }

    pub fn with_wallet(&mut self, wallet: Pubkey) -> Option<&mut DistributionOption> {
        self.iter_mut()
            .flatten()
            .find(|option| option.wallet == wallet)
    }

    pub fn from_accounts(infos: &[AccountInfo], authority: Pubkey) -> Result<Self> {
        let mut options = infos
            .iter()
            .map(Account::<TokenAccount>::try_from)
            .map(|acct| {
                acct.and_then(|acct| {
                    if acct.owner == authority {
                        Ok(acct)
                    } else {
                        Err(GovernanceRewardsError::TokenAccountNotOwned.into())
                    }
                })
            })
            .map(|account| {
                account.map(|some_acct| {
                    Some(DistributionOption {
                        mint: some_acct.mint,
                        wallet: some_acct.key(),
                        total_vote_weight: 0,
                        total_amount: some_acct.amount,
                        extra_reclaimed: false,
                    })
                })
            })
            .collect::<Result<Vec<_>>>()?;
        options.resize(8, None);
        Ok(DistributionOptions(options.try_into().unwrap()))
    }

    pub fn empty() -> Self {
        Self([None; 8])
    }
}

impl Deref for DistributionOptions {
    type Target = [Option<DistributionOption>; 8];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for DistributionOptions {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
