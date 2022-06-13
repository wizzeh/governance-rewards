use governance_rewards::program_test::governance_rewards_test::GovernanceRewardsTest;
use governance_rewards::program_test::tools::assert_governance_rewards_err;
use governance_rewards::{
    error::GovernanceRewardsError, state::distribution_option::DistributionOption,
};
use solana_program_test::tokio;
use solana_sdk::transport::TransportError;

type TestOutcome = Result<(), TransportError>;

#[tokio::test]
async fn test_reclaim() -> TestOutcome {
    // Arrange
    let mut governance_rewards_test = GovernanceRewardsTest::start_new().await;
    let realm_cookie = governance_rewards_test.governance.with_realm().await?;
    let key_cookie = governance_rewards_test.with_distribution_keypair();

    let funding_amount = 100;
    let funding_mint_1 = governance_rewards_test.bench.with_mint().await?;
    let funding_account_1 = governance_rewards_test
        .with_owned_tokens(&funding_mint_1, &key_cookie, funding_amount)
        .await?;

    let funding_amount = 150;
    let funding_mint_2 = governance_rewards_test.bench.with_mint().await?;
    let funding_account_2 = governance_rewards_test
        .with_owned_tokens(&funding_mint_2, &key_cookie, funding_amount)
        .await?;

    governance_rewards_test.bench.set_unix_time(9).await;

    let distribution_cookie = governance_rewards_test
        .with_distribution(
            &realm_cookie,
            &key_cookie,
            10,
            &[&funding_account_1, &funding_account_2],
        )
        .await?;

    let mut distribution_data = governance_rewards_test
        .get_distribution_account(distribution_cookie.address)
        .await;

    distribution_data.distribution_options[0] = Some(DistributionOption {
        total_vote_weight: 30,
        ..distribution_data.distribution_options[0].unwrap()
    });
    distribution_data.distribution_options[1] = Some(DistributionOption {
        total_vote_weight: 70,
        ..distribution_data.distribution_options[1].unwrap()
    });
    distribution_data.total_vote_weight = 100;

    governance_rewards_test
        .bench
        .set_anchor_account(
            &distribution_data,
            distribution_cookie.address,
            governance_rewards::id(),
        )
        .await?;
    governance_rewards_test.bench.set_unix_time(11).await;

    // Act
    let to_receive = governance_rewards_test
        .bench
        .with_token_account(&funding_mint_1.address)
        .await?;
    governance_rewards_test
        .reclaim_funds(&distribution_cookie, 0, &to_receive)
        .await?;

    // Assert
    let token_account = governance_rewards_test
        .bench
        .get_token_account(&to_receive.address)
        .await
        .unwrap();
    assert_eq!(token_account.amount, 70);

    let distribution = governance_rewards_test
        .get_distribution_account(distribution_cookie.address)
        .await;
    assert!(
        distribution.distribution_options[0]
            .unwrap()
            .extra_reclaimed
    );

    Ok(())
}

#[tokio::test]
async fn test_reclaim_twice_err() -> TestOutcome {
    // Arrange
    let mut governance_rewards_test = GovernanceRewardsTest::start_new().await;
    let realm_cookie = governance_rewards_test.governance.with_realm().await?;
    let key_cookie = governance_rewards_test.with_distribution_keypair();

    let funding_amount = 100;
    let funding_mint_1 = governance_rewards_test.bench.with_mint().await?;
    let funding_account_1 = governance_rewards_test
        .with_owned_tokens(&funding_mint_1, &key_cookie, funding_amount)
        .await?;

    governance_rewards_test.bench.set_unix_time(9).await;

    let distribution_cookie = governance_rewards_test
        .with_distribution(&realm_cookie, &key_cookie, 10, &[&funding_account_1])
        .await?;

    let mut distribution_data = governance_rewards_test
        .get_distribution_account(distribution_cookie.address)
        .await;

    distribution_data.distribution_options[0] = Some(DistributionOption {
        total_vote_weight: 30,
        ..distribution_data.distribution_options[0].unwrap()
    });
    distribution_data.total_vote_weight = 100;

    governance_rewards_test
        .bench
        .set_anchor_account(
            &distribution_data,
            distribution_cookie.address,
            governance_rewards::id(),
        )
        .await?;
    governance_rewards_test.bench.set_unix_time(11).await;

    // Act
    let to_receive = governance_rewards_test
        .bench
        .with_token_account(&funding_mint_1.address)
        .await?;
    governance_rewards_test
        .reclaim_funds(&distribution_cookie, 0, &to_receive)
        .await?;

    governance_rewards_test.bench.advance_clock().await;

    let err = governance_rewards_test
        .reclaim_funds(&distribution_cookie, 0, &to_receive)
        .await
        .err()
        .unwrap();

    // Assert
    assert_governance_rewards_err(err, GovernanceRewardsError::AlreadyReclaimed);

    Ok(())
}

#[tokio::test]
async fn test_reclaim_early_err() -> TestOutcome {
    // Arrange
    let mut governance_rewards_test = GovernanceRewardsTest::start_new().await;
    let realm_cookie = governance_rewards_test.governance.with_realm().await?;
    let key_cookie = governance_rewards_test.with_distribution_keypair();

    let funding_amount = 100;
    let funding_mint_1 = governance_rewards_test.bench.with_mint().await?;
    let funding_account_1 = governance_rewards_test
        .with_owned_tokens(&funding_mint_1, &key_cookie, funding_amount)
        .await?;

    governance_rewards_test.bench.set_unix_time(9).await;

    let distribution_cookie = governance_rewards_test
        .with_distribution(&realm_cookie, &key_cookie, 10, &[&funding_account_1])
        .await?;

    let mut distribution_data = governance_rewards_test
        .get_distribution_account(distribution_cookie.address)
        .await;

    distribution_data.distribution_options[0] = Some(DistributionOption {
        total_vote_weight: 30,
        ..distribution_data.distribution_options[0].unwrap()
    });
    distribution_data.total_vote_weight = 100;

    governance_rewards_test
        .bench
        .set_anchor_account(
            &distribution_data,
            distribution_cookie.address,
            governance_rewards::id(),
        )
        .await?;

    // Act
    let to_receive = governance_rewards_test
        .bench
        .with_token_account(&funding_mint_1.address)
        .await?;

    let err = governance_rewards_test
        .reclaim_funds(&distribution_cookie, 0, &to_receive)
        .await
        .err()
        .unwrap();

    // Assert
    assert_governance_rewards_err(err, GovernanceRewardsError::CannotReclaimFundsYet);

    Ok(())
}
