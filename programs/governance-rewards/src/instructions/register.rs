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
        constraint = voter_weight_record.is_still_valid(Clock::get()?) @ GovernanceRewardsError::OutdatedVoteWeightRecord,
        constraint = voter_weight_record.weight_action == Some(VoterWeightAction::RegisterForRewards.into()) @ GovernanceRewardsError::WrongAction,
        constraint = voter_weight_record.weight_action_target == Some(distribution.key()) @ GovernanceRewardsError::WrongActionTarget,
        constraint = voter_weight_record.realm == distribution.realm @ GovernanceRewardsError::WrongRealm,
        constraint = voter_weight_record.governing_token_owner == registrant.key() @ GovernanceRewardsError::WrongRegistrant
    )]
    voter_weight_record: Account<'info, VoterWeightRecord>,

    #[account(mut)]
    distribution: Account<'info, Distribution>,

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

    registrant: AccountInfo<'info>,

    #[account(mut)]
    payer: Signer<'info>,

    system_program: Program<'info, System>,
}

pub fn register_for_rewards(ctx: Context<RegisterForRewards>) -> Result<()> {
    require!(
        ctx.accounts.distribution.can_register(),
        GovernanceRewardsError::RegistrationOver
    );

    let weight = ctx.accounts.voter_weight_record.voter_weight;
    require!(weight > 0, GovernanceRewardsError::NoVoteWeight);

    require!(
        ctx.accounts.claim_data.weight == 0,
        GovernanceRewardsError::AlreadyRegistered
    );

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
        .pick_one(preferences.preferred_mint)?;

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
