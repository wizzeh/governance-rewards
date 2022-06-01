use crate::program_test::governance_rewards_test::GovernanceRewardsTest;
use crate::program_test::tools::assert_governance_rewards_err;
use governance_rewards::{
    error::GovernanceRewardsError,
    state::{
        addin::VoterWeightRecord,
        claim_data::ClaimData,
        preferences::{ResolutionPreference, UserPreferences},
    },
};
use solana_program_test::tokio;
use solana_sdk::{signature::Keypair, signer::Signer, transport::TransportError};

mod program_test;

type TestOutcome = Result<(), TransportError>;

#[tokio::test]
async fn test_claim() -> TestOutcome {
    // Arrange
    let mut governance_rewards_test = GovernanceRewardsTest::start_new().await;
    let user = Keypair::new();
    let realm_cookie = governance_rewards_test.governance.with_realm().await?;
    let key_cookie = governance_rewards_test.with_distribution_keypair();

    governance_rewards_test.bench.set_unix_time(5).await;
    let distribution_cookie = governance_rewards_test
        .with_funded_distribution(&realm_cookie, &key_cookie, 10)
        .await?;
    let token_mint = governance_rewards_test.bench.with_mint().await?;
    let token_account = governance_rewards_test
        .bench
        .with_tokens(&token_mint, &user.pubkey(), 1)
        .await?;

    let vote_weight = 10;
    let vwr = governance_rewards_test
        .with_dummy_voter_weight_record(
            &VoterWeightRecord::create_test(
                realm_cookie.address,
                token_mint.address,
                user.pubkey(),
                distribution_cookie.address,
                vote_weight,
                Some(u64::MAX),
            ),
            distribution_cookie.account.voter_weight_program,
        )
        .await?;

    let preferences = governance_rewards_test
        .with_preferences(
            &UserPreferences {
                preferred_mint: None,
                resolution_preference: ResolutionPreference::Wallet,
            },
            &realm_cookie,
            user.pubkey(),
        )
        .await?;

    let registrar = governance_rewards_test
        .with_registrant(&distribution_cookie, &vwr)
        .await?;

    governance_rewards_test.bench.set_unix_time(11).await;

    // Act
    let target_payout = distribution_cookie.funding[0];
    let user_token_account_cookie = governance_rewards_test
        .bench
        .create_associated_token_account(user.pubkey(), target_payout.mint)
        .await?;
    governance_rewards_test
        .claim(
            &user,
            target_payout.address,
            target_payout.mint,
            &distribution_cookie,
            &preferences,
        )
        .await?;

    // Assert
    let claim_record = governance_rewards_test
        .bench
        .get_anchor_account::<ClaimData>(ClaimData::get_address(
            vwr.user,
            distribution_cookie.address,
        ))
        .await;
    assert!(claim_record.has_claimed);

    let user_token_account = governance_rewards_test
        .bench
        .get_token_account(&user_token_account_cookie.address)
        .await
        .unwrap();
    assert_eq!(user_token_account.amount, 100);

    let distribution = governance_rewards_test
        .get_distribution_account(distribution_cookie.address)
        .await;

    assert_eq!(distribution.total_vote_weight_claimed, vwr.weight);

    Ok(())
}

#[tokio::test]
async fn test_claim_early_err() -> TestOutcome {
    // Arrange
    let mut governance_rewards_test = GovernanceRewardsTest::start_new().await;
    let user = Keypair::new();
    let realm_cookie = governance_rewards_test.governance.with_realm().await?;
    let key_cookie = governance_rewards_test.with_distribution_keypair();

    let distribution_cookie = governance_rewards_test
        .with_funded_distribution(&realm_cookie, &key_cookie, u64::MAX)
        .await?;
    let token_mint = governance_rewards_test.bench.with_mint().await?;
    let token_account = governance_rewards_test
        .bench
        .with_tokens(&token_mint, &user.pubkey(), 1)
        .await?;

    let vote_weight = 10;
    let vwr = governance_rewards_test
        .with_dummy_voter_weight_record(
            &VoterWeightRecord::create_test(
                realm_cookie.address,
                token_mint.address,
                user.pubkey(),
                distribution_cookie.address,
                vote_weight,
                Some(u64::MAX),
            ),
            distribution_cookie.account.voter_weight_program,
        )
        .await?;

    let preferences = governance_rewards_test
        .with_preferences(
            &UserPreferences {
                preferred_mint: None,
                resolution_preference: ResolutionPreference::Wallet,
            },
            &realm_cookie,
            user.pubkey(),
        )
        .await?;

    let registrar = governance_rewards_test
        .with_registrant(&distribution_cookie, &vwr)
        .await?;

    // Act
    let target_payout = distribution_cookie.funding[0];
    let user_token_account_cookie = governance_rewards_test
        .bench
        .create_associated_token_account(user.pubkey(), target_payout.mint)
        .await?;
    let err = governance_rewards_test
        .claim(
            &user,
            target_payout.address,
            target_payout.mint,
            &distribution_cookie,
            &preferences,
        )
        .await
        .err()
        .unwrap();

    // Assert
    assert_governance_rewards_err(err, GovernanceRewardsError::NotInClaimPeriod);

    Ok(())
}

#[tokio::test]
async fn test_claim_twice_err() -> TestOutcome {
    // Arrange
    let mut governance_rewards_test = GovernanceRewardsTest::start_new().await;
    let user = Keypair::new();
    let realm_cookie = governance_rewards_test.governance.with_realm().await?;
    let key_cookie = governance_rewards_test.with_distribution_keypair();

    governance_rewards_test.bench.set_unix_time(5).await;
    let distribution_cookie = governance_rewards_test
        .with_funded_distribution(&realm_cookie, &key_cookie, 10)
        .await?;
    let token_mint = governance_rewards_test.bench.with_mint().await?;
    let token_account = governance_rewards_test
        .bench
        .with_tokens(&token_mint, &user.pubkey(), 1)
        .await?;

    let vote_weight = 10;
    let vwr = governance_rewards_test
        .with_dummy_voter_weight_record(
            &VoterWeightRecord::create_test(
                realm_cookie.address,
                token_mint.address,
                user.pubkey(),
                distribution_cookie.address,
                vote_weight,
                Some(u64::MAX),
            ),
            distribution_cookie.account.voter_weight_program,
        )
        .await?;

    let preferences = governance_rewards_test
        .with_preferences(
            &UserPreferences {
                preferred_mint: None,
                resolution_preference: ResolutionPreference::Wallet,
            },
            &realm_cookie,
            user.pubkey(),
        )
        .await?;

    let registrar = governance_rewards_test
        .with_registrant(&distribution_cookie, &vwr)
        .await?;

    governance_rewards_test.bench.set_unix_time(11).await;

    // Act
    let target_payout = distribution_cookie.funding[0];
    let user_token_account_cookie = governance_rewards_test
        .bench
        .create_associated_token_account(user.pubkey(), target_payout.mint)
        .await?;
    governance_rewards_test
        .claim(
            &user,
            target_payout.address,
            target_payout.mint,
            &distribution_cookie,
            &preferences,
        )
        .await?;

    governance_rewards_test.bench.advance_clock().await;

    let err = governance_rewards_test
        .claim(
            &user,
            target_payout.address,
            target_payout.mint,
            &distribution_cookie,
            &preferences,
        )
        .await
        .err()
        .unwrap();

    // Assert
    assert_governance_rewards_err(err, GovernanceRewardsError::AlreadyClaimed);

    Ok(())
}
