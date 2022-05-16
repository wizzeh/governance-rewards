use anchor_lang::prelude::*;

use super::{distribution::Distribution, distribution_option::DistributionOption};

#[account]
pub struct ClaimData {
    pub weight: u64,
    pub distribution: Pubkey,
    pub claim_option: u8,
    pub has_claimed: bool,
}

impl ClaimData {
    pub fn chosen_option(&self, distribution: &Distribution) -> DistributionOption {
        distribution.distribution_options[self.claim_option as usize].unwrap()
    }
}
