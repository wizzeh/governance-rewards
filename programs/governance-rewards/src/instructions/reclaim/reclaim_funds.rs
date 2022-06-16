use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount};

use crate::{
    distribution_payout_seeds, error::GovernanceRewardsError, state::distribution::Distribution,
};

#[derive(Accounts)]
pub struct ReclaimFunds<'info> {
    admin: Signer<'info>,

    #[account(mut)]
    from: Account<'info, TokenAccount>,

    #[account(mut)]
    to: Account<'info, TokenAccount>,

    #[account(mut, has_one = admin @ GovernanceRewardsError::AdminOnly)]
    distribution: Box<Account<'info, Distribution>>,

    /// CHECK: Not read
    #[account(seeds = [b"payout authority".as_ref(), distribution.key().as_ref()], bump)]
    pub payout_authority: AccountInfo<'info>,

    token_program: Program<'info, Token>,
}

impl<'info> ReclaimFunds<'info> {
    pub fn transfer_context(&self) -> CpiContext<'_, '_, '_, 'info, token::Transfer<'info>> {
        let token_program = self.token_program.to_account_info();

        let to = self.to.to_account_info();

        let accounts = token::Transfer {
            from: self.from.to_account_info(),
            to,
            authority: self.payout_authority.to_account_info(),
        };
        CpiContext::new(token_program, accounts)
    }
}

pub fn reclaim_funds(ctx: Context<ReclaimFunds>) -> Result<()> {
    require!(
        !ctx.accounts.distribution.can_register(),
        GovernanceRewardsError::CannotReclaimFundsYet
    );

    let option = {
        let mut_option = ctx
            .accounts
            .distribution
            .distribution_options
            .by_wallet(ctx.accounts.from.key())
            .ok_or(GovernanceRewardsError::NoMatchingOption)?;

        require!(
            !mut_option.extra_reclaimed,
            GovernanceRewardsError::AlreadyReclaimed
        );

        mut_option.extra_reclaimed = true;
        *mut_option
    };

    let reclaimable_funds = ctx.accounts.distribution.calculate_unused_rewards(option);

    token::transfer(
        ctx.accounts
            .transfer_context()
            .with_signer(distribution_payout_seeds!(
                ctx.accounts.distribution,
                ctx.bumps
            )),
        reclaimable_funds,
    )
}
