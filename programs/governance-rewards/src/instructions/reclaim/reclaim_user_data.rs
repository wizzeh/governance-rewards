use anchor_lang::prelude::*;

use crate::{
    error::GovernanceRewardsError,
    state::{claim_data::ClaimData, distribution::Distribution},
};

#[derive(Accounts)]
pub struct ReclaimUserData<'info> {
    distribution: Account<'info, Distribution>,

    #[account(
        mut,
        close = caller,
    )]
    claim_data: Account<'info, ClaimData>,

    caller: AccountInfo<'info>,
}

pub fn reclaim_user_data(ctx: Context<ReclaimUserData>) -> Result<()> {
    require!(
        ctx.accounts.claim_data.distribution == ctx.accounts.distribution.key(),
        GovernanceRewardsError::WrongDistributionForClaim
    );

    require!(
        ctx.accounts.distribution.fully_claimed(),
        GovernanceRewardsError::CannotCleanUpYet
    );

    Ok(())
}
