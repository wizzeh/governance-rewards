use crate::state::preferences::ResolutionPreference;
use instructions::*;

#[cfg(feature = "client")]
pub mod client;
pub mod error;
pub mod instructions;
#[cfg(feature = "program-test")]
pub mod program_test;
pub mod state;

use anchor_lang::prelude::*;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod governance_rewards {
    use super::*;

    pub fn claim(ctx: Context<Claim>) -> Result<()> {
        instructions::claim(ctx)
    }

    pub fn create_distribution(
        ctx: Context<CreateDistribution>,
        registration_cutoff: u64,
        registrar: Option<Pubkey>,
    ) -> Result<()> {
        instructions::create_distribution(ctx, registration_cutoff, registrar)
    }

    pub fn reclaim_funds(ctx: Context<ReclaimFunds>) -> Result<()> {
        instructions::reclaim_funds(ctx)
    }

    pub fn reclaim_user_data(ctx: Context<ReclaimUserData>) -> Result<()> {
        instructions::reclaim_user_data(ctx)
    }

    pub fn register(ctx: Context<RegisterForRewards>) -> Result<()> {
        instructions::register_for_rewards(ctx)
    }

    pub fn set_preferred_mint(
        ctx: Context<SetPreferredMint>,
        new_preference: Option<Pubkey>,
    ) -> Result<()> {
        instructions::set_preferred_mint(ctx, new_preference)
    }

    pub fn set_resolution_preference(
        ctx: Context<SetResolutionPreference>,
        new_preference: ResolutionPreference,
    ) -> Result<()> {
        instructions::set_resolution_preference(ctx, new_preference)
    }

    pub fn create_escrow(ctx: Context<CreateEscrow>) -> Result<()> {
        instructions::create_escrow(ctx)
    }

    pub fn transfer_from_escrow(ctx: Context<TransferFromEscrow>, amount: u64) -> Result<()> {
        instructions::transfer_from_escrow(ctx, amount)
    }
}
