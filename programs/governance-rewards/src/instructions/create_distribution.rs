use anchor_lang::prelude::*;
use std::mem::size_of;

use crate::{
    error::GovernanceRewardsError,
    state::{distribution::Distribution, distribution_option::DistributionOptions},
};

/**
 * Instruction to create a Distribution.
 *
 * The caller must provide a registration_cutoff, which specifies the timestamp at
 * which registration will end.
 *
 * This instruction accepts up to 8 remaining accounts to be used to fund the
 * distribution. These accounts should be SPL Token Accounts owned by the payout
 * authority.
 */
#[derive(Accounts)]
pub struct CreateDistribution<'info> {
    /**
     * Address of the distribution to be created.
     */
    #[account(
        init,
        payer = payer,
        space = 8 + size_of::<Distribution>()
    )]
    pub distribution: Box<Account<'info, Distribution>>,

    /**
     * Account to own any provided token accounts.
     */
    /// CHECK: Not read
    #[account(seeds = [b"payout authority".as_ref(), distribution.key().as_ref()], bump)]
    pub payout_authority: AccountInfo<'info>,

    /**
     * Realm to which the distribution belongs.
     */
    /// CHECK: Not read
    pub realm: AccountInfo<'info>,

    /**
     * Choice of program to create voter weight records.
     *
     * The program will check whether presented voter weight records are owned by this
     * account. For example, you might provide the address of the voter stake registry
     * program.
     */
    /// CHECK: Not read
    pub voter_weight_program: AccountInfo<'info>,

    /// CHECK: Not read
    #[account(mut)]
    pub payer: Signer<'info>,

    /**
     * Admin for the distribution.
     *
     * Currently this is used to determine who can reclaim unused funds.
     */
    pub admin: Signer<'info>,

    system_program: Program<'info, System>,
}

pub fn create_distribution(
    ctx: Context<CreateDistribution>,
    registration_cutoff: u64,
    registrar: Option<Pubkey>,
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
        registrar,
    });

    Ok(())
}
