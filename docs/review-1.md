Overall Score
55/100
Brief: Final Test: Solana Crowdfunding
The code has excellent documentation and correctly implements the PDA vault pattern, but suffers from critical flaws in account derivation and state tracking that completely break withdrawals and refunds.
Category Scores

Review Criteria
Functionality
10/30
Code Quality
15/20
Program Design
10/20
Documentation
15/15
Security
2/10
Innovation
3/5
Critical Issues (2)
Inconsistent Account Derivation: `campaign` is initialized as a standard keypair account in `CreateCampaign` (no seeds), but `Withdraw` and `Refund` enforce PDA seeds `[b"campaign", creator]`. This mismatch guarantees that `withdraw` and `refund` will always fail with a `ConstraintSeeds` error.
Click here to learn more
Missing Donor State Tracking: The `refund` instruction accepts an arbitrary `amount` and does not verify how much the `donor` actually contributed. Any user can call `refund` repeatedly to drain the entire vault when a campaign fails.
Click here to learn more
Warnings (3)
Missing Checked Math: `campaign.raised += amount` can theoretically overflow. Use `campaign.raised.checked_add(amount).unwrap()` to prevent arithmetic overflow vulnerabilities.
Click here to learn more
Unused Account in Context: The `vault` account is validated in `CreateCampaign` but never used in the instruction logic. It should be removed from the `CreateCampaign` struct to optimize transaction size and compute units.
Click here to learn more
Implicit Account Closure: Withdrawing `vault.lamports()` drains the rent exemption, causing the system to garbage collect the PDA. While functional, it's safer to explicitly calculate `lamports - rent` or use Anchor's `close` constraint.
Click here to learn more
Strengths
Excellent inline documentation and comments explaining the purpose of each account and instruction.
Correct implementation of the PDA vault pattern for holding funds securely instead of sending them to the creator.
Good use of Anchor's `InitSpace` macro for accurate account sizing.
Comprehensive custom error definitions providing clear feedback for failure cases.
