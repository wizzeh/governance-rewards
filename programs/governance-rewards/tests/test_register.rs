use program_test::governance_rewards_test::GovernanceRewardsTest;
use solana_program_test::*;
use solana_sdk::transport::TransportError;

mod program_test;

type TestOutcome = Result<(), TransportError>;

#[tokio::test]
async fn test_register() -> TestOutcome {
    // Arrange
    let mut governance_rewards_test = GovernanceRewardsTest::start_new().await;
    let realm_cookie = governance_rewards_test.governance.with_realm().await?;
    let key_cookie = governance_rewards_test.with_distribution_keypair();
    let distribution_cookie = governance_rewards_test
        .with_distribution(&realm_cookie, &key_cookie, u64::max_value(), &[])
        .await?;

    Ok(())
}
