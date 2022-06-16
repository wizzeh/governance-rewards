use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount};
use spl_governance::tools::spl_token;

#[derive(Accounts)]
pub struct TransferFromEscrow<'info> {
    #[account(
        mut,
        seeds = [
            realm.key().as_ref(),
            escrow_release_admin.key().as_ref(),
            b"escrow".as_ref(),
            user.key().as_ref(),
            mint.key().as_ref(),
        ],
        bump
    )]
    escrow: Account<'info, TokenAccount>,

    #[account(mut, associated_token::mint = escrow.mint, associated_token::authority = user)]
    to_account: Account<'info, TokenAccount>,

    /// CHECK: Not read
    #[account(seeds = [b"escrow owner".as_ref(), realm.key().as_ref()], bump)]
    pub escrow_owner: AccountInfo<'info>,

    /// CHECK: Not read
    realm: AccountInfo<'info>,

    /// CHECK: Not read
    mint: AccountInfo<'info>,

    /// CHECK: Not read
    user: AccountInfo<'info>,

    escrow_release_admin: Signer<'info>,

    token_program: Program<'info, Token>,
}

pub fn transfer_from_escrow(ctx: Context<TransferFromEscrow>, amount: u64) -> Result<()> {
    spl_token::transfer_spl_tokens_signed(
        &ctx.accounts.escrow.to_account_info(),
        &ctx.accounts.to_account.to_account_info(),
        &ctx.accounts.escrow_owner.to_account_info(),
        &[b"escrow owner".as_ref(), ctx.accounts.realm.key().as_ref()],
        &crate::ID,
        amount,
        &ctx.accounts.token_program.to_account_info(),
    )
    .map_err(Error::from)
}
