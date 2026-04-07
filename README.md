# Solana Crowdfunding Smart Contract

A crowdfunding platform on Solana where users can create campaigns, accept donations, and either claim funds (if successful) or issue refunds (if failed).

Think Kickstarter, but on-chain.

---

## Table of Contents

- [Overview](#overview)
- [Features](#features)
- [Prerequisites](#prerequisites)
- [Quick Start](#quick-start)
- [Program Instructions](#program-instructions)
- [Security Considerations](#security-considerations)
- [File Structure](#file-structure)
- [API Reference](#api-reference)
- [Troubleshooting](#troubleshooting)
- [Resources](#resources)

---

## Overview

This Anchor program implements a complete crowdfunding solution with the following capabilities:

- **Create Campaign**: Users can start fundraising campaigns with a specific goal and deadline
- **Contribute**: Donors can contribute SOL to campaigns
- **Withdraw**: Campaign creators can claim funds if the goal is met after the deadline
- **Refund**: Donors can get refunds if the campaign fails (goal not met after deadline)

---

## Features

### Core Features

| Feature                      | Description                                                    |
| ---------------------------- | -------------------------------------------------------------- |
| PDA Vault                    | Funds are held in a Program Derived Address for security       |
| Deadline Enforcement         | Funds can only be claimed after deadline passes                |
| Goal Validation              | Withdraw requires goal to be met, refunds require goal NOT met |
| Double Withdrawal Prevention | Campaign tracks claimed status                                 |
| Comprehensive Logging        | All actions are logged on-chain                                |

### Security Features

- Creator authorization via `has_one` constraint
- PDA-based fund holding (no direct transfer to creator)
- Proper validation of all conditions before fund release
- Custom error codes for clear feedback

---

## Prerequisites

Before running this project, ensure you have:

1. **Rust** - Install from [rustup.rs](https://rustup.rs)
2. **Solana CLI** - Install from [Solana documentation](https://docs.solana.com/cli/install-solana-cli-tools)
3. **Anchor** - Install via `cargo install anchor-cli` or use AVM
4. **Node.js** - v18 or higher
5. **Yarn** or **npm**

### Recommended Tools

- **Phantom Wallet** - For testing with browser
- **VS Code** - With rust-analyzer extension

---

## Quick Start

### 1. Install Dependencies

```bash
# Navigate to project directory
cd mancer-submission

# Install Node dependencies
npm install
```

### 2. Build the Program

```bash
anchor build
```

### 3. Run Tests

```bash
anchor test
```

### 4. Deploy to Devnet

```bash
anchor deploy --provider.cluster devnet
```

---

## Program Instructions

### 1. create_campaign(goal: u64, deadline: i64)

Creates a new crowdfunding campaign.

**Parameters:**

- `goal`: Target amount in lamports (1 SOL = 1,000,000,000 lamports)
- `deadline`: Unix timestamp when the campaign ends

**Validation:**

- Deadline must be in the future

**Example:**

```typescript
const goal = 1000 * 1e9; // 1000 SOL
const deadline = Math.floor(Date.now() / 1000) + 86400; // Tomorrow

await program.methods
  .createCampaign(new anchor.BN(goal), new anchor.BN(deadline))
  .accounts({ creator: creator.publicKey })
  .rpc();
```

---

### 2. contribute(amount: u64)

Contributes SOL to a campaign.

**Parameters:**

- `amount`: Amount to donate in lamports

**Validation:**

- Amount must be greater than 0

**Example:**

```typescript
const amount = 100 * 1e9; // 100 SOL

await program.methods
  .contribute(new anchor.BN(amount))
  .accounts({
    campaign: campaignAddress,
    vault: vaultAddress,
    donor: donor.publicKey,
  })
  .rpc();
```

---

### 3. withdraw()

Creator claims funds from a successful campaign.

**Conditions (ALL must be true):**

- Campaign raised >= goal (goal reached)
- Current time >= deadline (deadline passed)
- Caller is the campaign creator
- Campaign not already claimed

**Example:**

```typescript
await program.methods
  .withdraw()
  .accounts({
    campaign: campaignAddress,
    vault: vaultAddress,
    creator: creator.publicKey,
  })
  .rpc();
```

---

### 4. refund(amount: u64)

Donor gets a refund if the campaign failed.

**Conditions (ALL must be true):**

- Current time >= deadline (deadline passed)
- Campaign raised < goal (goal NOT reached)
- Amount > 0

**Example:**

```typescript
const refundAmount = 50 * 1e9; // 50 SOL

await program.methods
  .refund(new anchor.BN(refundAmount))
  .accounts({
    campaign: campaignAddress,
    vault: vaultAddress,
    donor: donor.publicKey,
  })
  .rpc();
```

---

## Security Considerations

### Current Implementation Strengths

1. **Creator Authorization**: The `has_one = creator` constraint ensures only the original creator can withdraw funds.

2. **Double Withdrawal Prevention**: The `claimed` boolean prevents funds from being withdrawn multiple times.

3. **PDA-based Vault**: Funds are held in a PDA, not directly in the creator's account. The program controls when funds can be released.

4. **Proper Validation**: All conditions (goal, deadline, claimed status) are checked before any transfer.

### Known Limitations

⚠️ **Important**: The current implementation has these limitations:

1. **No Donor Tracking**: The refund instruction cannot verify if the caller actually contributed. Any signer could potentially call refund.

2. **No Refund Bounds**: Donors can request any amount up to the full vault, not just what they contributed.

**Recommendation**: These limitations are acceptable for a learning/project context. For production use, implement proper donor tracking.

---

## File Structure

```
mancer-submission/
├── Anchor.toml              # Anchor project configuration
├── Cargo.toml               # Rust workspace configuration
├── package.json             # Node.js dependencies
├── README.md                # This file
│
├── programs/
│   └── mancer-submission/
│       ├── Cargo.toml       # Program dependencies
│       └── src/
│           ├── lib.rs              # Main program entry point
│           ├── errors.rs           # Custom error codes
│           ├── campaign.rs         # Campaign account struct
│           └── accounts_struct.rs # Account validation structs
│
├── tests/
│   └── mancer-submission.ts # Test suite
│
├── docs/                    # Learning documentation
│   ├── 000_project_plan.md  # Project plan
│   ├── 010_submission_spec.md
│   ├── 011_solana_basics.md
│   ├── 012_anchor_primer.md
│   ├── 013_pda_explained.md
│   └── 014_deployment_guide.md
│
└── migrations/
    └── deploy.ts            # Deployment script
```

---

## API Reference

### Data Structures

#### Campaign Account

```rust
pub struct Campaign {
    pub creator: Pubkey,    // Who created this campaign
    pub goal: u64,          // Target amount in lamports
    pub raised: u64,        // Current amount raised
    pub deadline: i64,     // Unix timestamp when campaign ends
    pub claimed: bool,      // Whether funds have been claimed
}
```

### Error Codes

| Code                     | Message                                      | Description                                 |
| ------------------------ | -------------------------------------------- | ------------------------------------------- |
| `DeadlineMustBeInFuture` | Deadline must be in the future               | Tried to create campaign with past deadline |
| `GoalNotReached`         | Campaign goal not reached                    | Tried to withdraw before goal was met       |
| `DeadlineNotPassed`      | Campaign deadline has not passed yet         | Tried to withdraw/refund before deadline    |
| `AlreadyClaimed`         | Campaign funds already claimed               | Tried to withdraw twice                     |
| `GoalReached`            | Campaign goal reached - use withdraw instead | Tried to refund when goal was met           |
| `InvalidAmount`          | Invalid amount - must be greater than 0      | Amount was zero                             |

---

## Troubleshooting

### Build Errors

**Error: `anchor-cli` not found**

```bash
# Install Anchor CLI
cargo install anchor-cli

# Or use AVM
avm install latest
avm use latest
```

**Error: GLIBC version mismatch**

See [deployment guide](./docs/014_deployment_guide.md) for solutions.

### Runtime Errors

**Error: Campaign goal not reached**

- Wait for more contributions to meet the goal, OR
- Wait for deadline to pass and use refund instead

**Error: Campaign deadline has not passed yet**

- Wait until the deadline timestamp has passed

---

## Resources

### Official Documentation

- [Solana Documentation](https://docs.solana.com)
- [Anchor Framework](https://www.anchor-lang.com)
- [Solana Cookbook](https://solanacookbook.com)

### Learning Materials

- [Solana Basics](./docs/011_solana_basics.md)
- [Anchor Primer](./docs/012_anchor_primer.md)
- [PDA Explained](./docs/013_pda_explained.md)
- [Deployment Guide](./docs/014_deployment_guide.md)

---

## License

MIT

---

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

---

_Built with Anchor Framework for Solana_
