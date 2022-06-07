use anchor_lang::{prelude::Pubkey, solana_program::instruction::Instruction};
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
