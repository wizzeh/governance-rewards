use std::sync::Arc;

use anchor_lang::prelude::{AccountMeta, Pubkey};
use governance_rewards::state::{
    distribution::Distribution, distribution_option::DistributionOptions,
};
use solana_program::instruction::Instruction;
use solana_program_test::{processor, ProgramTest};
use solana_sdk::{signature::Keypair, signer::Signer, transport::TransportError};

use super::{
    governance_test::{GovernanceTest, RealmCookie},
    program_test_bench::{MintCookie, ProgramTestBench, TokenAccountCookie},
    tools::NopOverride,
};

#[derive(Debug)]
pub struct DistributionCookie {
    pub address: Pubkey,
    pub account: Distribution,

    pub registration_cutoff: u64,
}

#[derive(Debug)]
pub struct DistributionKeyCookie {
    pub keypair: Keypair,
}

pub struct GovernanceRewardsTest {
    pub program_id: Pubkey,
    pub bench: Arc<ProgramTestBench>,
    pub governance: GovernanceTest,
}

fn voter_weight_program() -> Pubkey {
    Pubkey::try_from("4Q6WW2ouZ6V3iaNm56MTd5n2tnTm4C5fiH8miFHnAFHo").unwrap()
}

impl GovernanceRewardsTest {
    pub fn add_program(program_test: &mut ProgramTest) {
        program_test.add_program(
            "governance_rewards",
            governance_rewards::id(),
            processor!(governance_rewards::entry),
        );
    }

    pub async fn start_new() -> Self {
        let mut program_test = ProgramTest::default();

        GovernanceRewardsTest::add_program(&mut program_test);
        GovernanceTest::add_program(&mut program_test);

        let program_id = governance_rewards::id();

        let bench = ProgramTestBench::start_new(program_test).await;
        let bench_rc = Arc::new(bench);

        let governance_bench =
            GovernanceTest::new(bench_rc.clone(), Some(program_id), Some(program_id));

        Self {
            program_id,
            bench: bench_rc,
            governance: governance_bench,
        }
    }

    pub fn with_distribution_keypair(&self) -> DistributionKeyCookie {
        DistributionKeyCookie {
            keypair: Keypair::new(),
        }
    }

    pub async fn with_distribution(
        &mut self,
        realm_cookie: &RealmCookie,
        key: &DistributionKeyCookie,
        registration_cutoff: u64,
        funding: &[&TokenAccountCookie],
    ) -> Result<DistributionCookie, TransportError> {
        self.with_distribution_using_ix(
            realm_cookie,
            key,
            registration_cutoff,
            funding,
            NopOverride,
            None,
        )
        .await
    }

    pub async fn with_distribution_using_ix<F: Fn(&mut Instruction)>(
        &mut self,
        realm_cookie: &RealmCookie,
        key: &DistributionKeyCookie,
        registration_cutoff: u64,
        funding: &[&TokenAccountCookie],
        instruction_override: F,
        signers_override: Option<&[&Keypair]>,
    ) -> Result<DistributionCookie, TransportError> {
        let data = anchor_lang::InstructionData::data(
            &governance_rewards::instruction::CreateDistribution {
                registration_cutoff,
            },
        );

        let mut accounts = anchor_lang::ToAccountMetas::to_account_metas(
            &governance_rewards::accounts::CreateDistribution {
                distribution: key.keypair.pubkey(),
                payout_authority: Distribution::get_payout_authority(key.keypair.pubkey()),
                realm: realm_cookie.address,
                voter_weight_program: voter_weight_program(),
                payer: self.bench.payer.pubkey(),
                system_program: solana_sdk::system_program::id(),
            },
            None,
        );

        for funding_source in funding {
            accounts.push(AccountMeta::new_readonly(funding_source.address, false));
        }

        let mut create_distribution_ix = Instruction {
            program_id: governance_rewards::id(),
            accounts,
            data,
        };

        instruction_override(&mut create_distribution_ix);

        let default_signers = &[&key.keypair, &self.bench.payer];
        let signers = signers_override.unwrap_or(default_signers);

        self.bench
            .process_transaction(&[create_distribution_ix], Some(signers))
            .await?;

        let account = Distribution {
            registration_period_end_ts: registration_cutoff,
            voter_weight_program: voter_weight_program(),
            realm: realm_cookie.address,
            total_vote_weight: 0,
            total_vote_weight_claimed: 0,
            distribution_options: DistributionOptions::empty(),
            admin: self.bench.payer.pubkey(),
        };

        Ok(DistributionCookie {
            address: key.keypair.pubkey(),
            account,
            registration_cutoff,
        })
    }

    pub async fn get_distribution_account(&mut self, distribution: Pubkey) -> Distribution {
        self.bench
            .get_anchor_account::<Distribution>(distribution)
            .await
    }

    pub async fn with_owned_tokens(
        &self,
        mint: &MintCookie,
        distribution: &DistributionKeyCookie,
        amount: u64,
    ) -> Result<TokenAccountCookie, TransportError> {
        let owner = Distribution::get_payout_authority(distribution.keypair.pubkey());
        self.bench.with_tokens(mint, &owner, amount).await
    }
}
