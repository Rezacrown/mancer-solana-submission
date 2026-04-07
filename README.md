# Solana Crowdfunding Smart Contract

A crowdfunding platform on Solana where users can create campaigns, accept donations, and either claim funds (if successful) or issue refunds (if failed).

---

## Overview

| Function          | Description                                        |
| ----------------- | -------------------------------------------------- |
| `create_campaign` | Create a new campaign with goal and deadline       |
| `contribute`      | Donate SOL to a campaign                           |
| `withdraw`        | Creator claims funds (goal met + deadline passed)  |
| `refund`          | Donor gets refund (goal not met + deadline passed) |

---

## Quick Start

```bash
# Install dependencies
npm install

# Build
anchor build

# Test
anchor test

# Deploy to devnet
anchor deploy --provider.cluster devnet
```

---

## Program Instructions

### 1. create_campaign(goal: u64, deadline: i64)

Creates a new crowdfunding campaign.

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

```typescript
await program.methods
  .contribute(new anchor.BN(100 * 1e9)) // 100 SOL
  .accounts({
    campaign: campaignAddress,
    vault: vaultAddress,
    donor: donor.publicKey,
  })
  .rpc();
```

---

### 3. withdraw()

Creator claims funds (requires goal met + deadline passed).

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

Donor gets refund (requires goal NOT met + deadline passed).

```typescript
await program.methods
  .refund(new anchor.BN(50 * 1e9)) // 50 SOL
  .accounts({
    campaign: campaignAddress,
    vault: vaultAddress,
    donor: donor.publicKey,
  })
  .rpc();
```

---

## Data Structures

### Campaign Account

```rust
pub struct Campaign {
    pub creator: Pubkey,
    pub goal: u64,         // in lamports
    pub raised: u64,       // in lamports
    pub deadline: i64,    // unix timestamp
    pub claimed: bool,
}
```

### Contribution Account

```rust
pub struct Contribution {
    pub campaign: Pubkey,
    pub donor: Pubkey,
    pub amount: u64,       // in lamports
}
```

---

## PDA Seeds

| Account      | Seed                                               |
| ------------ | -------------------------------------------------- |
| Campaign     | `[b"campaign", creator_pubkey]`                    |
| Vault        | `[b"vault", campaign_pubkey]`                      |
| Contribution | `[b"contribution", campaign_pubkey, donor_pubkey]` |

---

## File Structure

```
mancer-submission/
├── Anchor.toml
├── Cargo.toml
├── package.json
├── README.md
├── DELIVERABLES.md
│
├── programs/mancer-submission/src/
│   ├── lib.rs
│   ├── errors.rs
│   ├── campaign.rs
│   ├── contribution.rs
│   └── accounts_struct.rs
│
├── tests/
│   └── mancer-submission.ts
│
└── docs/
```

## Deployment Info

| Item       | Value                                                                                      |
| ---------- | ------------------------------------------------------------------------------------------ |
| Program ID | `2bvT3M5bLbJgk8dcm3jsDkSEn8B2ntk2Eokmt2UKb7pU`                                             |
| Deployer   | `7mCXNFj4N3X7Hax9FMWYBCeGpN89Qs5NrtU82dtm7Ye5`                                             |
| Signature  | `2huRwu5g3CYvVbhSGyARyKoZBWrhXCfpHEDKDHBCjvECYo7Z1hFvKPXTHr2moBCebZxbDAuNZxPLo29KAA9o4tHe` |
| Cluster    | Devnet                                                                                     |

---

## Submission Status

| Category          | Done | Pending |
| ----------------- | ---- | ------- |
| Success Criteria  | 6/6  | ✅      |
| Testing Checklist | 6/6  | ✅      |
| Common Pitfalls   | 4/4  | ✅      |
| Deploy to Devnet  | ✅   |         |

---

## Error Codes

| Code | Message                                      |
| ---- | -------------------------------------------- |
| 6000 | Deadline must be in the future               |
| 6001 | Campaign goal not reached                    |
| 6002 | Campaign deadline has not passed yet         |
| 6003 | Campaign funds already claimed               |
| 6004 | Campaign goal reached - use withdraw instead |
| 6005 | Invalid amount - must be greater than 0      |
| 6006 | Arithmetic overflow occurred                 |
| 6007 | No contribution found for this donor         |
| 6008 | Refund amount exceeds your contribution      |
| 6009 | No remaining contribution to refund          |

---

## License

MIT
