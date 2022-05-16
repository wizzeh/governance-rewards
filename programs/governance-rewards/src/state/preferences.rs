use anchor_lang::prelude::*;

#[account]
#[derive(Default, Copy)]
pub struct UserPreferences {
    pub preferred_mint: Option<Pubkey>,
    pub resolution_preference: ResolutionPreference,
}

#[derive(AnchorSerialize, AnchorDeserialize, Copy, Clone)]
pub enum ResolutionPreference {
    Wallet,
    Escrow,
}

impl Default for ResolutionPreference {
    fn default() -> Self {
        Self::Wallet
    }
}
