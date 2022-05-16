use anchor_lang::prelude::*;

use crate::state::preferences::{ResolutionPreference, UserPreferences};

#[derive(Accounts)]
pub struct SetResolutionPreference<'info> {
    #[account(
        mut,
        seeds = [realm.key().as_ref(), b"preferences".as_ref(), user.key().as_ref()],
        bump
    )]
    preferences: Account<'info, UserPreferences>,

    realm: AccountInfo<'info>,

    user: Signer<'info>,
}

pub fn set_resolution_preference(
    ctx: Context<SetResolutionPreference>,
    new_preference: ResolutionPreference,
) -> Result<()> {
    ctx.accounts.preferences.resolution_preference = new_preference;
    Ok(())
}
