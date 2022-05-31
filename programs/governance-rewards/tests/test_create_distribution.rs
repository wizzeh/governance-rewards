use governance_rewards::{
    error::GovernanceRewardsError, state::distribution_option::DistributionOption,
};
use program_test::{
    governance_rewards_test::GovernanceRewardsTest,
    tools::{assert_anchor_err, assert_governance_rewards_err},
};
use solana_sdk::{signature::Keypair, signer::Signer, transport::TransportError};

use crate::program_test::{program_test_bench::TokenAccountCookie, *};
use solana_program_test::*;

mod program_test;

type TestOutcome = Result<(), TransportError>;

#[tokio::test]
async fn test_create_distribution() -> TestOutcome {
    // Arrange
    let mut governance_rewards_test = GovernanceRewardsTest::start_new().await;
    let realm_cookie = governance_rewards_test.governance.with_realm().await?;
    let key_cookie = governance_rewards_test.with_distribution_keypair();

    // Act
    let distribution_cookie = governance_rewards_test
        .with_distribution(&realm_cookie, &key_cookie, u64::max_value(), &[])
        .await?;

    // Assert
    let distribution_record = governance_rewards_test
        .get_distribution_account(distribution_cookie.address)
        .await;

    assert_eq!(distribution_record, distribution_cookie.account);

    Ok(())
}

#[tokio::test]
async fn test_create_distribution_with_multiple_funding_sources() -> TestOutcome {
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

    // Act
    let distribution_cookie = governance_rewards_test
        .with_distribution(
            &realm_cookie,
            &key_cookie,
            u64::max_value(),
            &[&funding_account_1, &funding_account_2],
        )
        .await?;

    // Assert
    let distribution_record = governance_rewards_test
        .get_distribution_account(distribution_cookie.address)
        .await;

    assert_eq!(
        distribution_record.distribution_options[0].unwrap().mint,
        funding_mint_1.address
    );
    assert_eq!(
        distribution_record.distribution_options[1].unwrap().mint,
        funding_mint_2.address
    );
    assert_eq!(distribution_record.distribution_options[2..8], [None; 6]);

    Ok(())
}

#[tokio::test]
async fn test_create_distribution_in_past_error() -> TestOutcome {
    // Arrange
    let mut governance_rewards_test = GovernanceRewardsTest::start_new().await;
    let realm_cookie = governance_rewards_test.governance.with_realm().await?;
    let key_cookie = governance_rewards_test.with_distribution_keypair();

    // Act
    let err = governance_rewards_test
        .with_distribution(&realm_cookie, &key_cookie, 0, &[])
        .await
        .err()
        .unwrap();

    // Assert
    assert_governance_rewards_err(err, GovernanceRewardsError::RegistrationCutoffInPast);

    Ok(())
}

#[tokio::test]
async fn test_create_distribution_with_funding() -> TestOutcome {
    // Arrange
    let mut governance_rewards_test = GovernanceRewardsTest::start_new().await;
    let realm_cookie = governance_rewards_test.governance.with_realm().await?;
    let key_cookie = governance_rewards_test.with_distribution_keypair();

    let funding_amount = 100;
    let funding_mint = governance_rewards_test.bench.with_mint().await?;
    let funding_account = governance_rewards_test
        .with_owned_tokens(&funding_mint, &key_cookie, funding_amount)
        .await?;

    // Act
    let distribution_cookie = governance_rewards_test
        .with_distribution(
            &realm_cookie,
            &key_cookie,
            u64::max_value(),
            &[&funding_account],
        )
        .await?;

    // Assert
    let distribution_record = governance_rewards_test
        .get_distribution_account(distribution_cookie.address)
        .await;

    let first_option = distribution_record.distribution_options[0].unwrap();
    let expected = DistributionOption {
        mint: funding_mint.address,
        wallet: funding_account.address,
        total_amount: funding_amount,
        ..Default::default()
    };

    assert_eq!(first_option, expected);

    assert_eq!(distribution_record.distribution_options[1..8], [None; 7]);

    Ok(())
}

#[tokio::test]
async fn test_create_distribution_with_funding_not_owned_err() -> TestOutcome {
    // Arrange
    let mut governance_rewards_test = GovernanceRewardsTest::start_new().await;
    let realm_cookie = governance_rewards_test.governance.with_realm().await?;
    let key_cookie = governance_rewards_test.with_distribution_keypair();

    let funding_amount = 100;
    let funding_mint = governance_rewards_test.bench.with_mint().await?;
    let some_other_owner = Keypair::new().pubkey();
    let funding_account = governance_rewards_test
        .bench
        .with_tokens(&funding_mint, &some_other_owner, funding_amount)
        .await?;

    // Act
    let err = governance_rewards_test
        .with_distribution(
            &realm_cookie,
            &key_cookie,
            u64::max_value(),
            &[&funding_account],
        )
        .await
        .err()
        .unwrap();

    // Assert
    assert_governance_rewards_err(err, GovernanceRewardsError::TokenAccountNotOwned);

    Ok(())
}

#[tokio::test]
async fn test_create_distribution_with_non_token_funding_account_err() -> TestOutcome {
    // Arrange
    let mut governance_rewards_test = GovernanceRewardsTest::start_new().await;
    let realm_cookie = governance_rewards_test.governance.with_realm().await?;
    let key_cookie = governance_rewards_test.with_distribution_keypair();
    let fake_token_cookie = TokenAccountCookie {
        address: Keypair::new().pubkey(),
        mint: Keypair::new().pubkey(),
    };

    // Act
    let err = governance_rewards_test
        .with_distribution(
            &realm_cookie,
            &key_cookie,
            u64::max_value(),
            &[&fake_token_cookie],
        )
        .await
        .err()
        .unwrap();

    // Assert
    assert_anchor_err(err, anchor_lang::error::ErrorCode::AccountNotInitialized);

    Ok(())
}
