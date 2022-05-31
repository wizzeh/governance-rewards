use anchor_lang::prelude::{Clock, ErrorCode};
use governance_rewards::{
    error::GovernanceRewardsError,
    state::{
        addin::{VoterWeightAction, VoterWeightRecord},
        claim_data::ClaimData,
        preferences::UserPreferences,
    },
};
use program_test::{
    governance_rewards_test::{GovernanceRewardsTest, VoterWeightRecordCookie},
    tools::{assert_governance_rewards_err, assert_ix_err},
};
use solana_program::instruction::InstructionError;
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
            &VoterWeightRecord::create_test(
                realm_cookie.address,
                token_mint.address,
                token_account.address,
                distribution_cookie.address,
                vote_weight,
                Some(u64::MAX),
            ),
            distribution_cookie.account.voter_weight_program,
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

    let claim_record = governance_rewards_test
        .bench
        .get_anchor_account::<ClaimData>(ClaimData::get_address(
            vwr.user,
            distribution_cookie.address,
        ))
        .await;
    assert_eq!(claim_record.claim_option, 0);
    assert_eq!(claim_record.distribution, distribution_cookie.address);
    assert!(!claim_record.has_claimed);
    assert_eq!(claim_record.weight, vote_weight);

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
            &VoterWeightRecord::create_test(
                realm_cookie.address,
                token_mint.address,
                token_account.address,
                distribution_cookie.address,
                vote_weight,
                Some(0),
            ),
            distribution_cookie.account.voter_weight_program,
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
            &VoterWeightRecord::create_test(
                realm_cookie.address,
                token_mint.address,
                token_account.address,
                distribution_cookie.address,
                vote_weight,
                Some(u64::MAX),
            ),
            Keypair::new().pubkey(),
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
            &VoterWeightRecord::create_test(
                realm_cookie.address,
                token_mint.address,
                token_account.address,
                distribution_cookie.address,
                vote_weight,
                Some(u64::MAX),
            ),
            distribution_cookie.account.voter_weight_program,
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

#[tokio::test]
async fn test_register_with_wrong_realm_vwr_err() -> TestOutcome {
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
            &VoterWeightRecord::create_test(
                Keypair::new().pubkey(),
                token_mint.address,
                token_account.address,
                distribution_cookie.address,
                vote_weight,
                Some(u64::MAX),
            ),
            distribution_cookie.account.voter_weight_program,
        )
        .await?;

    // Act
    let err = governance_rewards_test
        .with_registrant(&distribution_cookie, &vwr)
        .await
        .err()
        .unwrap();

    // Assert
    assert_governance_rewards_err(err, GovernanceRewardsError::WrongRealm);

    Ok(())
}

#[tokio::test]
async fn test_register_with_wrong_token_owner_vwr_err() -> TestOutcome {
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
            &VoterWeightRecord::create_test(
                realm_cookie.address,
                token_mint.address,
                token_account.address,
                distribution_cookie.address,
                vote_weight,
                Some(u64::MAX),
            ),
            distribution_cookie.account.voter_weight_program,
        )
        .await?;

    // Act
    let fake_vwr = VoterWeightRecordCookie {
        user: Keypair::new().pubkey(),
        ..vwr
    };
    let err = governance_rewards_test
        .with_registrant(&distribution_cookie, &fake_vwr)
        .await
        .err()
        .unwrap();

    // Assert
    assert_governance_rewards_err(err, GovernanceRewardsError::WrongRegistrant);

    Ok(())
}

#[tokio::test]
async fn test_register_with_wrong_action_vwr_err() -> TestOutcome {
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
    let mut vwr_data = VoterWeightRecord::create_test(
        realm_cookie.address,
        token_mint.address,
        token_account.address,
        distribution_cookie.address,
        vote_weight,
        Some(u64::MAX),
    );
    vwr_data.weight_action = Some(VoterWeightAction::CastVote);
    let vwr = governance_rewards_test
        .with_dummy_voter_weight_record(&vwr_data, distribution_cookie.account.voter_weight_program)
        .await?;

    // Act
    let err = governance_rewards_test
        .with_registrant(&distribution_cookie, &vwr)
        .await
        .err()
        .unwrap();

    // Assert
    assert_governance_rewards_err(err, GovernanceRewardsError::WrongAction);

    Ok(())
}

#[tokio::test]
async fn test_register_with_wrong_action_target_vwr_err() -> TestOutcome {
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
    let mut vwr_data = VoterWeightRecord::create_test(
        realm_cookie.address,
        token_mint.address,
        token_account.address,
        distribution_cookie.address,
        vote_weight,
        Some(u64::MAX),
    );
    vwr_data.weight_action_target = Some(Keypair::new().pubkey());
    let vwr = governance_rewards_test
        .with_dummy_voter_weight_record(&vwr_data, distribution_cookie.account.voter_weight_program)
        .await?;

    // Act
    let err = governance_rewards_test
        .with_registrant(&distribution_cookie, &vwr)
        .await
        .err()
        .unwrap();

    // Assert
    assert_governance_rewards_err(err, GovernanceRewardsError::WrongActionTarget);

    Ok(())
}

#[tokio::test]
async fn test_register_with_no_action_target_vwr_err() -> TestOutcome {
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
    let mut vwr_data = VoterWeightRecord::create_test(
        realm_cookie.address,
        token_mint.address,
        token_account.address,
        distribution_cookie.address,
        vote_weight,
        Some(u64::MAX),
    );
    vwr_data.weight_action_target = None;
    let vwr = governance_rewards_test
        .with_dummy_voter_weight_record(&vwr_data, distribution_cookie.account.voter_weight_program)
        .await?;

    // Act
    let err = governance_rewards_test
        .with_registrant(&distribution_cookie, &vwr)
        .await
        .err()
        .unwrap();

    // Assert
    assert_governance_rewards_err(err, GovernanceRewardsError::WrongActionTarget);

    Ok(())
}

#[tokio::test]
async fn test_register_twice_err() -> TestOutcome {
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
            &VoterWeightRecord::create_test(
                realm_cookie.address,
                token_mint.address,
                token_account.address,
                distribution_cookie.address,
                vote_weight,
                Some(u64::MAX),
            ),
            distribution_cookie.account.voter_weight_program,
        )
        .await?;

    governance_rewards_test
        .with_registrant(&distribution_cookie, &vwr)
        .await?;

    governance_rewards_test.bench.advance_clock().await;

    // Act
    let err = governance_rewards_test
        .with_registrant(&distribution_cookie, &vwr)
        .await
        .err()
        .unwrap();

    // Assert
    assert_ix_err(err, InstructionError::Custom(0));

    Ok(())
}

#[tokio::test]
async fn test_register_with_preferred_mint() -> TestOutcome {
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

    let distribution_cookie = governance_rewards_test
        .with_distribution(
            &realm_cookie,
            &key_cookie,
            u64::max_value(),
            &[&funding_account_1, &funding_account_2],
        )
        .await?;
    let token_mint = governance_rewards_test.bench.with_mint().await?;
    let token_account = governance_rewards_test
        .with_owned_tokens(&token_mint, &key_cookie, 1)
        .await?;

    let vote_weight = 10;
    let vwr = governance_rewards_test
        .with_dummy_voter_weight_record(
            &VoterWeightRecord::create_test(
                realm_cookie.address,
                token_mint.address,
                token_account.address,
                distribution_cookie.address,
                vote_weight,
                Some(u64::MAX),
            ),
            distribution_cookie.account.voter_weight_program,
        )
        .await?;

    governance_rewards_test
        .with_preferences(
            &UserPreferences {
                preferred_mint: Some(funding_mint_2.address),
                resolution_preference: Default::default(),
            },
            &realm_cookie,
            vwr.user,
        )
        .await?;
    governance_rewards_test.bench.advance_clock().await;

    dbg!(
        governance_rewards_test
            .bench
            .get_anchor_account::<UserPreferences>(UserPreferences::get_address(
                vwr.user,
                realm_cookie.address,
            ))
            .await
    );

    // Act
    governance_rewards_test
        .with_registrant(&distribution_cookie, &vwr)
        .await?;

    // Assert
    let distribution_record = governance_rewards_test
        .get_distribution_account(distribution_cookie.address)
        .await;

    dbg!(&distribution_record);

    assert_eq!(
        distribution_record.distribution_options[1]
            .unwrap()
            .total_vote_weight,
        vote_weight
    );

    let claim_record = governance_rewards_test
        .bench
        .get_anchor_account::<ClaimData>(ClaimData::get_address(
            vwr.user,
            distribution_cookie.address,
        ))
        .await;
    assert_eq!(claim_record.claim_option, 1);

    Ok(())
}

#[tokio::test]
async fn test_register_with_unavailable_preferred_mint() -> TestOutcome {
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

    let distribution_cookie = governance_rewards_test
        .with_distribution(
            &realm_cookie,
            &key_cookie,
            u64::max_value(),
            &[&funding_account_1, &funding_account_2],
        )
        .await?;
    let token_mint = governance_rewards_test.bench.with_mint().await?;
    let token_account = governance_rewards_test
        .with_owned_tokens(&token_mint, &key_cookie, 1)
        .await?;

    let vote_weight = 10;
    let vwr = governance_rewards_test
        .with_dummy_voter_weight_record(
            &VoterWeightRecord::create_test(
                realm_cookie.address,
                token_mint.address,
                token_account.address,
                distribution_cookie.address,
                vote_weight,
                Some(u64::MAX),
            ),
            distribution_cookie.account.voter_weight_program,
        )
        .await?;

    governance_rewards_test
        .with_preferences(
            &UserPreferences {
                preferred_mint: Some(Keypair::new().pubkey()),
                resolution_preference: Default::default(),
            },
            &realm_cookie,
            vwr.user,
        )
        .await?;
    governance_rewards_test.bench.advance_clock().await;

    dbg!(
        governance_rewards_test
            .bench
            .get_anchor_account::<UserPreferences>(UserPreferences::get_address(
                vwr.user,
                realm_cookie.address,
            ))
            .await
    );

    // Act
    governance_rewards_test
        .with_registrant(&distribution_cookie, &vwr)
        .await?;

    // Assert
    let claim_record = governance_rewards_test
        .bench
        .get_anchor_account::<ClaimData>(ClaimData::get_address(
            vwr.user,
            distribution_cookie.address,
        ))
        .await;
    assert_eq!(claim_record.claim_option, 0);

    Ok(())
}
