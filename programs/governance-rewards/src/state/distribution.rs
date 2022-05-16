use anchor_lang::prelude::*;

use super::distribution_option::{DistributionOption, DistributionOptions};

#[account]
pub struct Distribution {
    pub registration_period_end_ts: u64,
    pub voter_weight_program: Pubkey,
    pub realm: Pubkey,
    pub total_vote_weight: u64,
    pub total_vote_weight_claimed: u64,
    pub distribution_options: DistributionOptions,
    pub admin: Pubkey,
}

impl Distribution {
    pub fn can_register(&self) -> bool {
        let time = Clock::get().unwrap().unix_timestamp as u64;

        time < self.registration_period_end_ts
    }

    pub fn can_claim(&self) -> bool {
        !self.can_register() && !self.fully_claimed()
    }

    pub fn fully_claimed(&self) -> bool {
        self.total_vote_weight_claimed >= self.total_vote_weight && !self.can_register()
    }

    pub fn calculate_rewards(&self, option: DistributionOption, weight: u64) -> u64 {
        u64::try_from(
            self.calculate_total_rewards(option)
                .checked_mul(weight as u128)
                .unwrap()
                .checked_div(option.total_vote_weight as u128)
                .unwrap(),
        )
        .unwrap()
    }

    pub fn calculate_unused_rewards(&self, option: DistributionOption) -> u64 {
        option.total_amount - u64::try_from(self.calculate_total_rewards(option)).unwrap()
    }

    fn calculate_total_rewards(&self, option: DistributionOption) -> u128 {
        (option.total_amount as u128)
            .checked_mul(option.total_vote_weight as u128)
            .unwrap()
            .checked_div(self.total_vote_weight as u128)
            .unwrap()
    }
}
