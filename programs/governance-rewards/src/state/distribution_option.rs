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

impl DistributionOption {
    fn try_from_account(account_info: &AccountInfo, authority: Pubkey) -> Result<Self> {
        let token_account = Account::<TokenAccount>::try_from(account_info)?;
        if token_account.owner != authority {
            return Err(GovernanceRewardsError::TokenAccountNotOwned.into());
        }

        Ok(DistributionOption {
            mint: token_account.mint,
            wallet: token_account.key(),
            total_vote_weight: 0,
            total_amount: token_account.amount,
            extra_reclaimed: false,
        })
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default, Copy, Debug, PartialEq, Eq)]
pub struct DistributionOptions([Option<DistributionOption>; 8]);

impl DistributionOptions {
    pub fn pick_by_mint(&mut self, key: Option<Pubkey>) -> Result<(u8, &mut DistributionOption)> {
        let mut first_valid_mint: Option<(u8, &mut DistributionOption)> = None;
        for (i, option) in self.iter_mut().enumerate() {
            if let Some(option) = option {
                if key.is_none() || key == Some(option.mint) {
                    return Ok((i as u8, option));
                }
                first_valid_mint.get_or_insert((i as u8, option));
            }
        }

        if let Some(default) = first_valid_mint {
            return Ok(default);
        }
        Err(GovernanceRewardsError::NoDistributionOptions.into())
    }

    pub fn by_wallet(&mut self, wallet: Pubkey) -> Option<&mut DistributionOption> {
        self.iter_mut()
            .flatten()
            .find(|option| option.wallet == wallet)
    }

    pub fn from_accounts(infos: &[AccountInfo], authority: Pubkey) -> Result<Self> {
        let mut options = infos
            .iter()
            .map(|acct| DistributionOption::try_from_account(acct, authority).map(Some))
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
