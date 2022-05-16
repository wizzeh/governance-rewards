use anchor_lang::prelude::*;

#[error_code]
pub enum GovernanceRewardsError {
    #[msg("Vote weight record is no longer valid")]
    OutdatedVoteWeightRecord,
    #[msg("Vote weight record has wrong type of action")]
    WrongAction,
    #[msg("Vote weight record has wrong action target")]
    WrongActionTarget,
    #[msg("Vote weight record has wrong realm")]
    WrongRealm,
    #[msg("Registration period is over")]
    RegistrationOver,
    #[msg("No vote weight")]
    NoVoteWeight,
    #[msg("No valid distribution options")]
    NoDistributionOptions,
    #[msg("Vote weight record does not match registrant")]
    WrongRegistrant,
    #[msg("Already registered for this rewards distribution")]
    AlreadyRegistered,

    #[msg("Cannot create a distribution with a registration period ending in the past")]
    RegistrationCutoffInPast,

    #[msg("You can only claim during the claim period")]
    NotInClaimPeriod,
    #[msg("User has already claimed")]
    AlreadyClaimed,

    #[msg("Incorrect payout account provided")]
    WrongPayoutAccount,

    #[msg("Provided the wrong distribution for this claim")]
    WrongDistributionForClaim,
    #[msg("Cannot clean up user claims until all claims have been made")]
    CannotCleanUpYet,

    #[msg("Must be a distribution admin to take this action")]
    AdminOnly,
    #[msg("No distribution option matching the provided wallet")]
    NoMatchingOption,
    #[msg("Cannot reclaim excess funds until the registration period is over")]
    CannotReclaimFunds,
}
