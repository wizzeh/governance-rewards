use anchor_lang::prelude::*;
use std::mem::size_of;

use crate::{
    error::GovernanceRewardsError,
    state::{distribution::Distribution, distribution_option::DistributionOptions},
};

#[derive(Accounts)]
pub struct CreateDistribution<'info> {
    #[account(
        init,
        payer = payer,
        space = 8 + size_of::<Distribution>()
    )]
    pub distribution: Box<Account<'info, Distribution>>,

    /// CHECK: Not read
    #[account(seeds = [b"payout authority".as_ref(), distribution.key().as_ref()], bump)]
    pub payout_authority: AccountInfo<'info>,

    /// CHECK: Not read
    pub realm: AccountInfo<'info>,

    /// CHECK: Not read
    pub voter_weight_program: AccountInfo<'info>,

    #[account(mut)]
    pub payer: AccountInfo<'info>,

    pub admin: Signer<'info>,

    system_program: Program<'info, System>,
}

pub fn create_distribution(
    ctx: Context<CreateDistribution>,
    registration_cutoff: u64,
) -> Result<()> {
    require!(
        registration_cutoff > Clock::get().unwrap().unix_timestamp as u64,
        GovernanceRewardsError::RegistrationCutoffInPast
    );

    ctx.accounts.distribution.set_inner(Distribution {
        registration_period_end_ts: registration_cutoff,
        realm: ctx.accounts.realm.key(),
        total_vote_weight: 0,
        total_vote_weight_claimed: 0,
        distribution_options: DistributionOptions::from_accounts(
            ctx.remaining_accounts,
            ctx.accounts.payout_authority.key(),
        )?,
        voter_weight_program: ctx.accounts.voter_weight_program.key(),
        admin: ctx.accounts.admin.key(),
    });

    Ok(())
}
