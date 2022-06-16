use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount};

#[derive(Accounts)]
pub struct CreateEscrow<'info> {
    #[account(
        init,
        payer = payer,
        seeds = [
            realm.key().as_ref(),
            escrow_release_admin.key().as_ref(),
            b"escrow".as_ref(),
            user.key().as_ref(),
            mint.key().as_ref(),
        ],
        bump,
        token::mint = mint,
        token::authority = escrow_owner
    )]
    escrow: Account<'info, TokenAccount>,

    /// CHECK: Not read
    #[account(seeds = [b"escrow owner".as_ref(), realm.key().as_ref()], bump)]
    pub escrow_owner: AccountInfo<'info>,

    /// CHECK: Not read
    realm: AccountInfo<'info>,

    /// CHECK: Not read
    mint: AccountInfo<'info>,

    /// CHECK: Not read
    user: AccountInfo<'info>,

    /// CHECK: Not read
    escrow_release_admin: AccountInfo<'info>,

    /// CHECK: Not read
    #[account(mut)]
    payer: AccountInfo<'info>,

    token_program: Program<'info, Token>,

    system_program: Program<'info, System>,

    rent: Sysvar<'info, Rent>,
}

pub fn create_escrow(_ctx: Context<CreateEscrow>) -> Result<()> {
    Ok(())
}
