# Governance Rewards Plugin

## Instructions
### Core
- `create_distribution`: A Distribution is the core data structure of this plugin. A Distribution has two phases: the registration phase, and the claim phase. When creating a distribution you must supply the timestamp of the end of the registration phase.
- `register` and `update_registration`: Called by a user with a voter weight record to register for rewards.
- `claim`: Called after the registration phase ends to disburse rewards.

### User Preferences
User preferences are realm-wide.
- `set_preferred_mint`: Called to set the preferred currency for Distributions with multiple options.
- `set_resolution_preference`: Called to set the preferred resolution preference. Options are direct payout and escrow.

### Reclaim
- `reclaim_funds`: Called to reclaim excess funding from the Distribution after the registration period ends.
- `reclaim_user_data`: Called to reclaim rent for user claim data after the claim has been paid out.
