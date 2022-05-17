use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::get_associated_token_address,
    token::{self, Token, TokenAccount},
};
use std::mem::size_of;

use crate::{
    distribution_payout_seeds,
    error::GovernanceRewardsError,
    state::{
        claim_data::ClaimData,
        distribution::Distribution,
        preferences::{ResolutionPreference, UserPreferences},
    },
};

#[derive(Accounts)]
pub struct Claim<'info> {
    #[account(mut)]
    distribution: Account<'info, Distribution>,

    #[account(
        mut,
        seeds = [distribution.key().as_ref(), b"user data".as_ref(), claimant.key().as_ref()],
        bump
    )]
    user_data: Account<'info, ClaimData>,

    #[account(
        mut,
        address = user_data.chosen_option(&distribution).wallet
    )]
    rewards_account: Account<'info, TokenAccount>,

    #[account(seeds = [b"payout authority".as_ref(), distribution.key().as_ref()], bump)]
    payout_authority: AccountInfo<'info>,

    #[account(init_if_needed, payer = caller, space = size_of::<TokenAccount>())]
    to_account: Account<'info, TokenAccount>,

    #[account(
        seeds = [distribution.realm.as_ref(), b"preferences".as_ref(), claimant.key().as_ref()],
        bump
    )]
    preferences: AccountInfo<'info>,

    claimant: AccountInfo<'info>,

    #[account(mut)]
    caller: AccountInfo<'info>,

    token_program: Program<'info, Token>,
    system_program: Program<'info, System>,
}

impl<'info> Claim<'info> {
    pub fn transfer_context(&self) -> CpiContext<'_, '_, '_, 'info, token::Transfer<'info>> {
        let token_program = self.token_program.to_account_info();

        let to = self.to_account.to_account_info();

        let accounts = token::Transfer {
            from: self.rewards_account.to_account_info(),
            to,
            authority: self.payout_authority.to_account_info(),
        };
        CpiContext::new(token_program, accounts)
    }

    pub fn payout_mint(&self) -> Pubkey {
        self.user_data.chosen_option(&self.distribution).mint
    }

    pub fn assert_payout_is_ata(&self) -> Result<()> {
        let expected_address =
            get_associated_token_address(&self.claimant.key(), &self.payout_mint());

        require!(
            expected_address == self.to_account.key(),
            GovernanceRewardsError::WrongPayoutAccount
        );

        require!(
            self.to_account.owner == self.claimant.key(),
            GovernanceRewardsError::WrongPayoutAccount
        );

        require!(
            self.to_account.mint == self.payout_mint(),
            GovernanceRewardsError::WrongPayoutAccount
        );

        Ok(())
    }

    pub fn assert_payout_is_escrow(&self) -> Result<()> {
        // TODO create this
        let expected_address = Pubkey::find_program_address(
            &[
                self.distribution.realm.as_ref(),
                b"escrow".as_ref(),
                self.claimant.key().as_ref(),
                self.payout_mint().as_ref(),
            ],
            &crate::ID,
        )
        .0;

        require!(
            expected_address == self.to_account.key(),
            GovernanceRewardsError::WrongPayoutAccount
        );

        Ok(())
    }
}

pub fn claim(ctx: Context<Claim>) -> Result<()> {
    require!(
        ctx.accounts.distribution.can_claim(),
        GovernanceRewardsError::NotInClaimPeriod
    );
    require!(
        !ctx.accounts.user_data.has_claimed,
        GovernanceRewardsError::AlreadyClaimed
    );

    let rewards = ctx.accounts.distribution.calculate_rewards(
        ctx.accounts
            .user_data
            .chosen_option(&ctx.accounts.distribution),
        ctx.accounts.user_data.weight,
    );

    ctx.accounts.user_data.has_claimed = true;
    ctx.accounts.distribution.total_vote_weight_claimed = ctx
        .accounts
        .distribution
        .total_vote_weight_claimed
        .checked_add(ctx.accounts.user_data.weight)
        .unwrap();

    let preferences = UserPreferences::get_or_default(&ctx.accounts.preferences);
    match preferences.resolution_preference {
        ResolutionPreference::Wallet => ctx.accounts.assert_payout_is_ata()?,
        ResolutionPreference::Escrow => ctx.accounts.assert_payout_is_escrow()?,
    }

    token::transfer(
        ctx.accounts
            .transfer_context()
            .with_signer(distribution_payout_seeds!(
                ctx.accounts.distribution,
                ctx.bumps
            )),
        rewards,
    )
}
