# Project Plan: Solana Crowdfunding Platform

## Overview

This document outlines the complete development plan for building a crowdfunding smart contract on Solana as part of the Mancer submission requirements.

---

## Milestones

### Phase 1: Environment Setup & Learning

| Step | Task                             | Status  |
| ---- | -------------------------------- | ------- |
| 1.1  | Install Solana CLI & Anchor      | ✅ Done |
| 1.2  | Setup Phantom wallet with devnet | ✅ Done |
| 1.3  | Get SOL from devnet faucet       | ✅ Done |
| 1.4  | Verify project builds            | ✅ Done |

### Phase 2: Smart Contract Implementation

| Step | Task                                  | Status  |
| ---- | ------------------------------------- | ------- |
| 2.1  | Define Campaign struct                | ✅ Done |
| 2.2  | Implement create_campaign instruction | ✅ Done |
| 2.3  | Implement contribute instruction      | ✅ Done |
| 2.4  | Implement withdraw instruction        | ✅ Done |
| 2.5  | Implement refund instruction          | ✅ Done |
| 2.6  | Build and verify                      | ✅ Done |

### Phase 3: Testing

| Step | Task                               | Status     |
| ---- | ---------------------------------- | ---------- |
| 3.1  | Write unit tests                   | 🔄 Pending |
| 3.2  | Test create campaign               | 🔄 Pending |
| 3.3  | Test contribute                    | 🔄 Pending |
| 3.4  | Test withdraw (success/fail cases) | 🔄 Pending |
| 3.5  | Test refund (success/fail cases)   | 🔄 Pending |
| 3.6  | Run all tests locally              | 🔄 Pending |

### Phase 4: Deployment

| Step | Task                             | Status     |
| ---- | -------------------------------- | ---------- |
| 4.1  | Deploy to devnet                 | 🔄 Pending |
| 4.2  | Get Program ID                   | 🔄 Pending |
| 4.3  | Verify Program ID matches config | 🔄 Pending |
| 4.4  | Run tests on devnet              | 🔄 Pending |
| 4.5  | Get test transaction signatures  | 🔄 Pending |

---

## Future Tasks (Post-Implementation)

### Security Enhancements

| Task               | Priority | Description                                                           |
| ------------------ | -------- | --------------------------------------------------------------------- |
| Add Donor Tracking | HIGH     | Track individual contributions per donor for proper refund validation |
| Add Refund Bounds  | MEDIUM   | Limit refund amount to actual contribution                            |
| Add Access Control | MEDIUM   | Program-level authority for admin functions                           |
| Add Pause Feature  | LOW      | Emergency pause capability for campaigns                              |

### Additional Features

| Task                  | Priority | Description                                 |
| --------------------- | -------- | ------------------------------------------- |
| Campaign Updates      | LOW      | Creator can update campaign metadata        |
| Milestone Withdrawals | LOW      | Release funds in stages based on milestones |
| NFT Rewards           | LOW      | Tokenize campaign rewards                   |
| Token Gating          | LOW      | Restrict contributions to token holders     |

---

## Known Limitations

⚠️ **Current Implementation Limitations:**

1. **No Donor Tracking** - The `refund` instruction cannot verify if the caller actually contributed to the campaign. Anyone can call `refund()` and potentially drain the vault.

2. **No Refund Bounds** - Donors can request any amount up to the full vault balance, not just what they actually contributed.

**Recommendation:** These limitations are acceptable for a learning/project submission context. For production use, implement proper donor tracking.

---

## Technical Details

### Program ID

```
Localnet: 2bvT3M5bLbJgk8dcm3jsDkSEn8B2ntk2Eokmt2UKb7pU
Devnet:   (to be updated after deployment)
```

### File Structure

```
mancer-submission/
├── Anchor.toml              # Anchor configuration
├── Cargo.toml               # Rust workspace
├── package.json             # Node dependencies
├── tsconfig.json            # TypeScript config
├── README.md                # Main documentation
│
├── programs/
│   └── mancer-submission/
│       ├── Cargo.toml       # Program dependencies
│       └── src/
│           ├── lib.rs               # Main entry point
│           ├── errors.rs            # Custom error codes
│           ├── campaign.rs         # Campaign data struct
│           └── accounts_struct.rs  # Account validation structs
│
├── tests/
│   └── mancer-submission.ts # Test suite
│
├── docs/
│   ├── 000_project-plan.md  # This file
│   ├── 010_submission-spec.md
│   ├── 011_solana-basics.md
│   ├── 012_anchor-primer.md
│   ├── 013_pda-explained.md
│   └── 014_deployment_guide.md
│
└── migrations/
    └── deploy.ts            # Deployment script
```

---

## Success Criteria

- [x] Program compiles without errors
- [x] All 4 instructions implemented
- [ ] Tests pass locally
- [ ] Deployed to devnet
- [ ] Program ID recorded
- [ ] Test transaction signatures obtained

---

## Notes

- 1 SOL = 1,000,000,000 lamports (10^9)
- All amounts in program are in lamports
- Deadline is Unix timestamp (seconds since epoch)
- Use `Clock::get()` to get current time

---

_Last Updated: April 2026_
