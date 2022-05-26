use anchor_lang::prelude::*;
use std::mem::size_of;

use crate::state::{
    addin::{VoterWeightAction, VoterWeightRecord},
    claim_data::ClaimData,
    distribution::Distribution,
};
use crate::{error::GovernanceRewardsError, state::preferences::UserPreferences};

#[derive(Accounts)]
pub struct RegisterForRewards<'info> {
    #[account(
        owner=distribution.voter_weight_program,
    )]
    voter_weight_record: AccountInfo<'info>,

    #[account(mut)]
    distribution: Box<Account<'info, Distribution>>,

    /// CHECK: Manually deserialized
    #[account(
        seeds = [distribution.realm.as_ref(), b"preferences".as_ref(), registrant.key().as_ref()],
        bump
    )]
    preferences: AccountInfo<'info>,

    #[account(
        init,
        space = 8 + size_of::<ClaimData>(),
        payer = payer,
        seeds = [distribution.key().as_ref(), b"claim data".as_ref(), registrant.key().as_ref()],
        bump
    )]
    claim_data: Account<'info, ClaimData>,

    /// CHECK: Not read
    registrant: AccountInfo<'info>,

    #[account(mut)]
    payer: Signer<'info>,

    system_program: Program<'info, System>,
}

pub fn register_for_rewards(ctx: Context<RegisterForRewards>) -> Result<()> {
    let voter_weight_record = VoterWeightRecord::try_from(&ctx.accounts.voter_weight_record)?;
    let voter_weight_record =
        voter_weight_record.validate(&ctx.accounts.distribution, &ctx.accounts.registrant.key())?;

    require!(
        ctx.accounts.distribution.can_register(),
        GovernanceRewardsError::RegistrationOver
    );

    let weight = voter_weight_record.voter_weight;
    require!(weight > 0, GovernanceRewardsError::NoVoteWeight);

    ctx.accounts.distribution.total_vote_weight = ctx
        .accounts
        .distribution
        .total_vote_weight
        .checked_add(weight)
        .unwrap();

    let preferences = UserPreferences::get_or_default(&ctx.accounts.preferences);

    let (index, preferred_distribution_option) = ctx
        .accounts
        .distribution
        .distribution_options
        .pick_by_mint(preferences.preferred_mint)?;

    preferred_distribution_option.total_vote_weight = preferred_distribution_option
        .total_vote_weight
        .checked_add(weight)
        .unwrap();

    ctx.accounts.claim_data.set_inner(ClaimData {
        weight,
        distribution: ctx.accounts.distribution.key(),
        claim_option: index,
        has_claimed: false,
    });

    Ok(())
}
