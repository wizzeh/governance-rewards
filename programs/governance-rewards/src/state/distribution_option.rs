use std::ops::{Deref, DerefMut};

use anchor_lang::prelude::*;
use anchor_spl::token::TokenAccount;

use crate::error::GovernanceRewardsError;

#[derive(AnchorDeserialize, AnchorSerialize, Clone, Default, Copy, Debug)]
pub struct DistributionOption {
    pub total_vote_weight: u64,
    pub total_amount: u64,
    pub mint: Pubkey,
    pub wallet: Pubkey,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default, Copy)]
pub struct DistributionOptions([Option<DistributionOption>; 8]);

impl DistributionOptions {
    pub fn pick_one(&mut self, key: Option<Pubkey>) -> Result<(u8, &mut DistributionOption)> {
        for (i, option) in self.iter_mut().flatten().enumerate() {
            match key {
                Some(k) => {
                    if option.mint == k {
                        return Ok((i as u8, option));
                    }
                }
                None => return Ok((i as u8, option)),
            }
        }

        Err(GovernanceRewardsError::NoDistributionOptions.into())
    }

    pub fn with_wallet(&mut self, wallet: Pubkey) -> Option<&DistributionOption> {
        self.iter().flatten().find(|option| option.wallet == wallet)
    }

    pub fn from_accounts(infos: &[AccountInfo], authority: Pubkey) -> Self {
        let mut options = infos
            .iter()
            .map(Account::<TokenAccount>::try_from)
            .map(|acct| acct.ok())
            .map(|acct| acct.filter(|acct| acct.owner == authority))
            .map(|account| {
                account.map(|some_acct| DistributionOption {
                    mint: some_acct.mint,
                    wallet: some_acct.key(),
                    total_vote_weight: 0,
                    total_amount: some_acct.amount,
                })
            })
            .collect::<Vec<Option<DistributionOption>>>();
        options.resize(8, None);
        DistributionOptions(options.try_into().unwrap())
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
