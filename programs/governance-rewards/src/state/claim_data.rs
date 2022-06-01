use anchor_lang::prelude::*;

use super::{distribution::Distribution, distribution_option::DistributionOption};

#[account]
#[derive(Debug)]
pub struct ClaimData {
    pub weight: u64,
    pub distribution: Pubkey,
    pub claim_option: u8,
    pub has_claimed: bool,
    pub has_registered: bool,
}

impl ClaimData {
    pub fn chosen_option(&self, distribution: &Distribution) -> DistributionOption {
        distribution.distribution_options[self.claim_option as usize].unwrap()
    }

    pub fn get_address(user: Pubkey, distribution: Pubkey) -> Pubkey {
        Pubkey::find_program_address(
            &[distribution.as_ref(), b"claim data".as_ref(), user.as_ref()],
            &crate::id(),
        )
        .0
    }
}
