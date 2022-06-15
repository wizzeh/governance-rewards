use crate::program_test::governance_rewards_test::GovernanceRewardsTest;
use solana_program::program_pack::Pack;
use solana_program_test::tokio;
use solana_sdk::{signature::Keypair, signer::Signer, transport::TransportError};

mod program_test;

type TestOutcome = Result<(), TransportError>;

#[tokio::test]
async fn test_claim_from_escrow() -> TestOutcome {
    // Arrange
    let mut governance_rewards_test = GovernanceRewardsTest::start_new().await;
    let user = Keypair::new();
    let realm_cookie = governance_rewards_test.governance.with_realm().await?;
    let mint = Keypair::new();
    let mint_authority = Keypair::new();
    governance_rewards_test
        .bench
        .create_mint(&mint, &mint_authority.pubkey(), None)
        .await?;
    let admin = Keypair::new();
    let escrow_address = governance_rewards_test
        .with_escrow(
            &user.pubkey(),
            &mint.pubkey(),
            &realm_cookie,
            &admin.pubkey(),
        )
        .await?;

    let account_info = governance_rewards_test
        .bench
        .get_account(&escrow_address)
        .await
        .unwrap();
    let mut data = governance_rewards_test
        .bench
        .get_token_account(&escrow_address)
        .await
        .unwrap();
    data.amount = 100;
    let mut data_pack = Vec::new();
    data_pack.resize(anchor_spl::token::spl_token::state::Account::LEN, 0);
    data.pack_into_slice(&mut data_pack);
    governance_rewards_test
        .bench
        .set_account(data_pack, escrow_address, account_info.owner)
        .await?;

    let recipient = governance_rewards_test
        .bench
        .with_token_account(&mint.pubkey())
        .await?;

    // Act
    governance_rewards_test
        .transfer_from_escrow(
            &escrow_address,
            &user,
            &realm_cookie,
            &recipient,
            &admin,
            100,
        )
        .await?;

    // Assert
    let token_account = governance_rewards_test
        .bench
        .get_token_account(&recipient.address)
        .await
        .unwrap();
    assert_eq!(token_account.amount, 100);

    Ok(())
}
