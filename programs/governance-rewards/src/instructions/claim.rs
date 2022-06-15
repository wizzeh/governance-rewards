use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount};

use crate::{
    distribution_payout_seeds,
    error::GovernanceRewardsError,
    state::{
        claim_data::ClaimData,
        distribution::Distribution,
        preferences::{ResolutionPreference, UserPreferences},
    },
};

/**
 * Instruction to redeem a claim.
 *
 * May only be called after the registration period ends.
 */
#[derive(Accounts)]
pub struct Claim<'info> {
    #[account(mut)]
    distribution: Box<Account<'info, Distribution>>,

    #[account(
        mut,
        seeds = [distribution.key().as_ref(), b"claim data".as_ref(), claimant.key().as_ref()],
        bump
    )]
    claim_data: Account<'info, ClaimData>,

    /**
     * Account from which to pay out rewards.
     *
     * This account should be the Token Account associated with the user's chosen
     * distribution option.
     */
    #[account(
        mut,
        address = claim_data.chosen_option(&distribution).wallet
    )]
    rewards_account: Account<'info, TokenAccount>,

    /// CHECK: Not read
    #[account(seeds = [b"payout authority".as_ref(), distribution.key().as_ref()], bump)]
    payout_authority: AccountInfo<'info>,

    /**
     * Account to receive rewards payout.
     *
     * If `UserPreferences.resolution_preference == Wallet`, this should be the associated
     * token program wallet associated with the claimant.
     *
     * If `UserPreferences.resolution_preference == Escrow`, this should be the user's
     * escrow wallet for the mint. See `assert_payout_is_escrow` for the PDA seeds.
     */
    #[account(mut)]
    to_account: Account<'info, TokenAccount>,

    /**
     * User claim preferences.
     *
     * This account does not have the be initialized when it is passed to the program.
     * If an empty account is provided, default preferences will be used.
     */
    /// CHECK: Manually deserialized
    #[account(
        seeds = [distribution.realm.as_ref(), b"preferences".as_ref(), claimant.key().as_ref()],
        bump
    )]
    preferences: AccountInfo<'info>,

    /**
     * User to receive rewards payout.
     */
    /// CHECK: Not read
    claimant: AccountInfo<'info>,

    /// CHECK: Not read
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
        self.claim_data.chosen_option(&self.distribution).mint
    }

    pub fn assert_payout_is_ata(&self) -> Result<()> {
        let expected_address = ResolutionPreference::Wallet.payout_address(
            self.claimant.key(),
            self.payout_mint(),
            self.distribution.realm,
        );

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

    pub fn assert_payout_is_escrow(&self, admin: Pubkey) -> Result<()> {
        let expected_address = ResolutionPreference::Escrow { admin }.payout_address(
            self.claimant.key(),
            self.payout_mint(),
            self.distribution.realm,
        );

        require!(
            expected_address == self.to_account.key(),
            GovernanceRewardsError::WrongPayoutAccount
        );

        Ok(())
    }
}

pub fn claim(ctx: Context<Claim>) -> Result<()> {
    require!(
        !ctx.accounts.claim_data.has_claimed,
        GovernanceRewardsError::AlreadyClaimed
    );
    require!(
        ctx.accounts.distribution.can_claim(),
        GovernanceRewardsError::NotInClaimPeriod
    );

    let rewards = ctx.accounts.distribution.calculate_rewards(
        ctx.accounts
            .claim_data
            .chosen_option(&ctx.accounts.distribution),
        ctx.accounts.claim_data.weight,
    );

    ctx.accounts.claim_data.has_claimed = true;
    ctx.accounts.distribution.total_vote_weight_claimed = ctx
        .accounts
        .distribution
        .total_vote_weight_claimed
        .checked_add(ctx.accounts.claim_data.weight)
        .unwrap();

    let preferences = UserPreferences::get_or_default(&ctx.accounts.preferences);
    match preferences.resolution_preference {
        ResolutionPreference::Wallet => ctx.accounts.assert_payout_is_ata()?,
        ResolutionPreference::Escrow { admin } => ctx.accounts.assert_payout_is_escrow(admin)?,
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
