use anchor_lang::prelude::*;
use anchor_spl::associated_token::get_associated_token_address;

#[account]
#[derive(Default, Copy, Debug)]
pub struct UserPreferences {
    pub preferred_mint: Option<Pubkey>,
    pub resolution_preference: ResolutionPreference,
}

#[derive(AnchorSerialize, AnchorDeserialize, Copy, Clone, Debug)]
pub enum ResolutionPreference {
    Wallet,
    Escrow,
}

impl Default for ResolutionPreference {
    fn default() -> Self {
        Self::Wallet
    }
}

impl ResolutionPreference {
    pub fn payout_address(&self, user: Pubkey, mint: Pubkey, realm: Pubkey) -> Pubkey {
        match self {
            ResolutionPreference::Wallet => get_associated_token_address(&user, &mint),
            ResolutionPreference::Escrow => {
                Pubkey::find_program_address(
                    &[
                        realm.as_ref(),
                        b"escrow".as_ref(),
                        user.as_ref(),
                        mint.as_ref(),
                    ],
                    &crate::ID,
                )
                .0
            }
        }
    }
}

impl UserPreferences {
    pub fn get_or_default(account: &AccountInfo) -> Self {
        Account::<UserPreferences>::try_from(account)
            .map(|acct| *acct)
            .unwrap_or_default()
    }

    pub fn get_address(user: Pubkey, realm: Pubkey) -> Pubkey {
        Pubkey::find_program_address(
            &[realm.as_ref(), b"preferences".as_ref(), user.as_ref()],
            &crate::id(),
        )
        .0
    }
}
