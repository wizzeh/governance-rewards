use anchor_lang::prelude::{Clock, ErrorCode};
use governance_rewards::error::GovernanceRewardsError;
use program_test::{
    governance_rewards_test::GovernanceRewardsTest, tools::assert_governance_rewards_err,
};
use solana_program_test::*;
use solana_sdk::{signature::Keypair, signer::Signer, transport::TransportError};

use crate::program_test::tools::assert_anchor_err;

mod program_test;

type TestOutcome = Result<(), TransportError>;

#[tokio::test]
async fn test_register() -> TestOutcome {
    // Arrange
    let mut governance_rewards_test = GovernanceRewardsTest::start_new().await;
    let realm_cookie = governance_rewards_test.governance.with_realm().await?;
    let key_cookie = governance_rewards_test.with_distribution_keypair();
    let distribution_cookie = governance_rewards_test
        .with_funded_distribution(&realm_cookie, &key_cookie, u64::max_value())
        .await?;
    let token_mint = governance_rewards_test.bench.with_mint().await?;
    let token_account = governance_rewards_test
        .with_owned_tokens(&token_mint, &key_cookie, 1)
        .await?;

    let vote_weight = 10;
    let vwr = governance_rewards_test
        .with_dummy_voter_weight_record(
            &realm_cookie,
            &token_mint,
            &distribution_cookie,
            token_account.address,
            vote_weight,
            u64::MAX,
            None,
        )
        .await?;

    // Act
    let registrar = governance_rewards_test
        .with_registrant(&distribution_cookie, &vwr)
        .await?;

    // Assert
    let distribution_record = governance_rewards_test
        .get_distribution_account(distribution_cookie.address)
        .await;

    assert_eq!(distribution_record.total_vote_weight, vote_weight);
    assert_eq!(distribution_record.total_vote_weight_claimed, 0);

    assert_eq!(
        distribution_record.distribution_options[0]
            .unwrap()
            .total_vote_weight,
        vote_weight
    );

    Ok(())
}

#[tokio::test]
async fn test_register_with_expired_vwr_err() -> TestOutcome {
    // Arrange
    let mut governance_rewards_test = GovernanceRewardsTest::start_new().await;
    let realm_cookie = governance_rewards_test.governance.with_realm().await?;
    let key_cookie = governance_rewards_test.with_distribution_keypair();
    let distribution_cookie = governance_rewards_test
        .with_funded_distribution(&realm_cookie, &key_cookie, u64::max_value())
        .await?;
    let token_mint = governance_rewards_test.bench.with_mint().await?;
    let token_account = governance_rewards_test
        .with_owned_tokens(&token_mint, &key_cookie, 1)
        .await?;

    let vote_weight = 10;
    let vwr = governance_rewards_test
        .with_dummy_voter_weight_record(
            &realm_cookie,
            &token_mint,
            &distribution_cookie,
            token_account.address,
            vote_weight,
            0,
            None,
        )
        .await?;

    // Act
    let err = governance_rewards_test
        .with_registrant(&distribution_cookie, &vwr)
        .await
        .err()
        .unwrap();

    // Assert
    assert_governance_rewards_err(err, GovernanceRewardsError::OutdatedVoteWeightRecord);

    Ok(())
}

#[tokio::test]
async fn test_register_with_wrong_source_vwr_err() -> TestOutcome {
    // Arrange
    let mut governance_rewards_test = GovernanceRewardsTest::start_new().await;
    let realm_cookie = governance_rewards_test.governance.with_realm().await?;
    let key_cookie = governance_rewards_test.with_distribution_keypair();
    let distribution_cookie = governance_rewards_test
        .with_funded_distribution(&realm_cookie, &key_cookie, u64::max_value())
        .await?;
    let token_mint = governance_rewards_test.bench.with_mint().await?;
    let token_account = governance_rewards_test
        .with_owned_tokens(&token_mint, &key_cookie, 1)
        .await?;

    let vote_weight = 10;
    let vwr = governance_rewards_test
        .with_dummy_voter_weight_record(
            &realm_cookie,
            &token_mint,
            &distribution_cookie,
            token_account.address,
            vote_weight,
            0,
            Some(Keypair::new().pubkey()),
        )
        .await?;

    // Act
    let err = governance_rewards_test
        .with_registrant(&distribution_cookie, &vwr)
        .await
        .err()
        .unwrap();

    // Assert
    assert_anchor_err(err, ErrorCode::ConstraintOwner);

    Ok(())
}

#[tokio::test]
async fn test_register_with_closed_registration_err() -> TestOutcome {
    // Arrange
    let mut governance_rewards_test = GovernanceRewardsTest::start_new().await;
    let realm_cookie = governance_rewards_test.governance.with_realm().await?;
    let key_cookie = governance_rewards_test.with_distribution_keypair();
    let near_future = governance_rewards_test
        .bench
        .get_clock()
        .await
        .unix_timestamp
        + 10000;
    let distribution_cookie = governance_rewards_test
        .with_funded_distribution(&realm_cookie, &key_cookie, near_future as u64)
        .await?;
    let token_mint = governance_rewards_test.bench.with_mint().await?;
    let token_account = governance_rewards_test
        .with_owned_tokens(&token_mint, &key_cookie, 1)
        .await?;

    let vote_weight = 10;
    let vwr = governance_rewards_test
        .with_dummy_voter_weight_record(
            &realm_cookie,
            &token_mint,
            &distribution_cookie,
            token_account.address,
            vote_weight,
            u64::MAX,
            None,
        )
        .await?;

    // Act
    let clock = Clock {
        unix_timestamp: near_future + 1,
        ..governance_rewards_test.bench.get_clock().await
    };
    governance_rewards_test
        .bench
        .context
        .borrow_mut()
        .set_sysvar(&clock);
    let err = governance_rewards_test
        .with_registrant(&distribution_cookie, &vwr)
        .await
        .err()
        .unwrap();

    // Assert
    assert_governance_rewards_err(err, GovernanceRewardsError::RegistrationOver);

    Ok(())
}
