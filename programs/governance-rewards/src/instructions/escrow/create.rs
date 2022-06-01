use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount};

#[derive(Accounts)]
pub struct CreateEscrow<'info> {
    #[account(
        init,
        payer = payer,
        seeds = [
            realm.key().as_ref(),
            b"escrow".as_ref(),
            user.key().as_ref(),
            mint.key().as_ref(),
        ],
        bump,
        token::mint = mint,
        token::authority = escrow_authority
    )]
    escrow: Account<'info, TokenAccount>,

    /// CHECK: Not read
    #[account(seeds = [b"escrow authority".as_ref(), realm.key().as_ref()], bump)]
    pub escrow_authority: AccountInfo<'info>,

    /// CHECK: Not read
    realm: AccountInfo<'info>,

    /// CHECK: Not read
    mint: AccountInfo<'info>,

    /// CHECK: Not read
    user: AccountInfo<'info>,

    #[account(mut)]
    payer: AccountInfo<'info>,

    token_program: Program<'info, Token>,

    system_program: Program<'info, System>,

    rent: Sysvar<'info, Rent>,
}

pub fn create_escrow(ctx: Context<CreateEscrow>) -> Result<()> {
    Ok(())
}
