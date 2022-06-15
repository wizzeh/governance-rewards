use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount};
use spl_governance::tools::spl_token;

#[derive(Accounts)]
pub struct TransferFromEscrow<'info> {
    #[account(
        mut,
        seeds = [
            realm.key().as_ref(),
            admin.key().as_ref(),
            b"escrow".as_ref(),
            user.key().as_ref(),
            mint.key().as_ref(),
        ],
        bump
    )]
    escrow: Account<'info, TokenAccount>,

    #[account(mut, token::mint = escrow.mint)]
    to_account: Account<'info, TokenAccount>,

    /// CHECK: Not read
    #[account(seeds = [b"escrow authority".as_ref(), realm.key().as_ref()], bump)]
    pub escrow_authority: AccountInfo<'info>,

    /// CHECK: Not read
    realm: AccountInfo<'info>,

    /// CHECK: Not read
    mint: AccountInfo<'info>,

    /// CHECK: Not read
    user: AccountInfo<'info>,

    admin: Signer<'info>,

    token_program: Program<'info, Token>,
}

pub fn transfer_from_escrow(ctx: Context<TransferFromEscrow>, amount: u64) -> Result<()> {
    spl_token::transfer_spl_tokens_signed(
        &ctx.accounts.escrow.to_account_info(),
        &ctx.accounts.to_account.to_account_info(),
        &ctx.accounts.escrow_authority.to_account_info(),
        &[
            b"escrow authority".as_ref(),
            ctx.accounts.realm.key().as_ref(),
        ],
        &crate::ID,
        amount,
        &ctx.accounts.token_program.to_account_info(),
    )
    .map_err(Error::from)
}
