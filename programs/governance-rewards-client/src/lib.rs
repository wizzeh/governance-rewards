use anchor_lang::{prelude::Pubkey, solana_program::instruction::Instruction};
use governance_rewards::state::distribution::Distribution;
use governance_rewards::state::{claim_data::ClaimData, preferences::UserPreferences};

pub fn register(
    user: Pubkey,
    distribution: Pubkey,
    realm: Pubkey,
    voter_weight_record: Pubkey,
    payer: Pubkey,
) -> Instruction {
    let data = anchor_lang::InstructionData::data(&governance_rewards::instruction::Register {});
    let accounts = anchor_lang::ToAccountMetas::to_account_metas(
        &governance_rewards::accounts::RegisterForRewards {
            voter_weight_record,
            distribution,
            preferences: UserPreferences::get_address(user, realm),
            claim_data: ClaimData::get_address(user, distribution),
            registrant: user,
            payer,
            system_program: solana_sdk::system_program::id(),
        },
        None,
    );

    Instruction {
        program_id: governance_rewards::id(),
        accounts,
        data,
    }
}

pub fn claim(
    user: Pubkey,
    distribution: Pubkey,
    realm: Pubkey,
    rewards_account: Pubkey,
    to_account: Pubkey,
    payer: Pubkey,
) -> Instruction {
    let data = anchor_lang::InstructionData::data(&governance_rewards::instruction::Claim {});
    let accounts = anchor_lang::ToAccountMetas::to_account_metas(
        &governance_rewards::accounts::Claim {
            caller: payer,
            claimant: user,
            distribution,
            rewards_account,
            to_account,
            payout_authority: Distribution::get_payout_authority(distribution),
            claim_data: ClaimData::get_address(user, distribution),
            preferences: UserPreferences::get_address(user, realm),
            token_program: anchor_spl::token::ID,
            system_program: solana_sdk::system_program::id(),
        },
        None,
    );

    Instruction {
        program_id: governance_rewards::id(),
        accounts,
        data,
    }
}

pub fn reclaim_funds(distribution: Pubkey, admin: Pubkey, from: Pubkey, to: Pubkey) -> Instruction {
    let data =
        anchor_lang::InstructionData::data(&governance_rewards::instruction::ReclaimFunds {});
    let accounts = anchor_lang::ToAccountMetas::to_account_metas(
        &governance_rewards::accounts::ReclaimFunds {
            admin,
            from,
            to,
            distribution,
            payout_authority: Distribution::get_payout_authority(distribution),
            token_program: anchor_spl::token::ID,
        },
        None,
    );

    Instruction {
        program_id: governance_rewards::id(),
        accounts,
        data,
    }
}
