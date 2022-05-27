use anchor_lang::prelude::*;

use crate::state::{addin::VoterWeightRecord, claim_data::ClaimData, distribution::Distribution};
use crate::{error::GovernanceRewardsError, state::preferences::UserPreferences};

/**
 * Instruction to update a claim.
 *
 * The account signature of this instruction is the same as [RegisterForRewards].
 */
#[derive(Accounts)]
pub struct UpdateRegistration<'info> {
    /// CHECK: Manually deserialized
    #[account(
        owner=distribution.voter_weight_program,
    )]
    voter_weight_record: AccountInfo<'info>,

    #[account(mut)]
    distribution: Box<Account<'info, Distribution>>,

    /**
     * User claim preferences.
     *
     * This account does not have the be initialized when it is passed to the program.
     * If an empty account is provided, default preferences will be used.
     */
    /// CHECK: Manually deserialized
    #[account(
        seeds = [distribution.realm.as_ref(), b"preferences".as_ref(), registrant.key().as_ref()],
        bump
    )]
    preferences: AccountInfo<'info>,

    #[account(
        seeds = [distribution.key().as_ref(), b"claim data".as_ref(), registrant.key().as_ref()],
        bump
    )]
    claim_data: Account<'info, ClaimData>,

    /// CHECK: Not read
    registrant: AccountInfo<'info>,

    #[account(mut)]
    payer: Signer<'info>,
}

pub fn update_registration(ctx: Context<UpdateRegistration>) -> Result<()> {
    let voter_weight_record = VoterWeightRecord::try_from(&ctx.accounts.voter_weight_record)?;
    let voter_weight_record =
        voter_weight_record.validate(&ctx.accounts.distribution, &ctx.accounts.registrant.key())?;

    require!(
        ctx.accounts.distribution.can_register(),
        GovernanceRewardsError::RegistrationOver
    );

    let weight = voter_weight_record.voter_weight;
    require!(weight > 0, GovernanceRewardsError::NoVoteWeight);

    let old_weight = ctx.accounts.claim_data.weight;

    // Update total vote weight
    ctx.accounts.distribution.total_vote_weight = ctx
        .accounts
        .distribution
        .total_vote_weight
        .checked_sub(old_weight)
        .unwrap()
        .checked_add(weight)
        .unwrap();

    // Remove vote weight from old distribution option
    let old_option = &mut ctx.accounts.distribution.distribution_options
        [ctx.accounts.claim_data.claim_option as usize]
        .unwrap();
    old_option.total_vote_weight = old_option
        .total_vote_weight
        .checked_sub(old_weight)
        .unwrap();

    // Add new vote weight to new distribution option
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
