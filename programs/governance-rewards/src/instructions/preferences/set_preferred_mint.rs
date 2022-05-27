use anchor_lang::prelude::*;

use crate::state::preferences::UserPreferences;

#[derive(Accounts)]
pub struct SetPreferredMint<'info> {
    #[account(
        init_if_needed,
        payer = user,
        space = 8 + std::mem::size_of::<UserPreferences>(),
        seeds = [realm.key().as_ref(), b"preferences".as_ref(), user.key().as_ref()],
        bump
    )]
    preferences: Account<'info, UserPreferences>,

    /// CHECK: Not read
    realm: AccountInfo<'info>,

    #[account(mut)]
    user: Signer<'info>,

    system_program: Program<'info, System>,
}

pub fn set_preferred_mint(
    ctx: Context<SetPreferredMint>,
    new_preference: Option<Pubkey>,
) -> Result<()> {
    ctx.accounts.preferences.preferred_mint = new_preference;
    Ok(())
}
