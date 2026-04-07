# Solana Crowdfunding Platform - Specification

## Project Overview

A crowdfunding smart contract on Solana where users can create campaigns, accept donations, and either claim funds (if successful) or issue refunds (if failed).

Think Kickstarter, but on-chain.

---

## The Problem

Right now, there's no way for creators to:

- Accept donations without immediately receiving them
- Guarantee refunds if they don't hit their goal
- Prove to donors that funds are locked until conditions are met

---

## Task Overview

Build a Solana program with 4 functions:

### 1. Create Campaign

**What it does:** Creator sets up a new fundraising campaign

**Inputs:**

- `goal` (u64): Target amount in lamports
- `deadline` (i64): Unix timestamp when campaign ends

**Logic:**

- Store campaign data (creator, goal, deadline, raised=0, claimed=false)
- Validate deadline is in the future
- Log: "Campaign created: goal={goal}, deadline={deadline}"

---

### 2. Contribute

**What it does:** Donor sends SOL to the campaign

**Inputs:**

- `amount` (u64): How much to donate in lamports

**Logic:**

- Transfer SOL from donor to campaign vault (PDA)
- Update campaign.raised += amount
- Log: "Contributed: {amount} lamports, total={raised}"

---

### 3. Withdraw

**What it does:** Creator claims funds if campaign succeeded

**Conditions (all must be true):**

- Campaign raised >= goal
- Current time >= deadline
- Caller is the campaign creator
- Campaign not already claimed

**Logic:**

- Transfer all SOL from vault to creator
- Mark campaign.claimed = true
- Log: "Withdrawn: {amount} lamports"

---

### 4. Refund

**What it does:** Donor gets money back if campaign failed

**Conditions (all must be true):**

- Campaign raised < goal
- Current time >= deadline

**Logic:**

- Transfer donor's contribution back from vault
- Log: "Refunded: {amount} lamports"

---

## Technical Specification

### Data Structure

```rust
pub struct Campaign {
    pub creator: Pubkey,    // Who created this
    pub goal: u64,          // Target amount
    pub raised: u64,        // Current amount
    pub deadline: i64,     // When it ends
    pub claimed: bool,      // Already withdrawn?
}
```

---

### The Vault (Important!)

Don't send donations directly to the creator. Use a Program Derived Address (PDA) as a vault:

```rust
// Derive the vault address
let (vault_pda, bump) = Pubkey::find_program_address(
    &[b"vault", campaign_account.key.as_ref()],
    program_id
);

// Later, when transferring FROM the vault, use invoke_signed:
invoke_signed(
    &system_instruction::transfer(vault_pda, recipient, amount),
    &[vault_account, recipient_account, system_program],
    &[&[b"vault", campaign_account.key.as_ref(), &[bump]]]
)?;
```

**Why PDA?** It's an account your program controls. No private key needed - your program can "sign" for it.

---

### Getting Current Time

```rust
use solana_program::clock::Clock;
use solana_program::sysvar::Sysvar;

let clock = Clock::get()?;
let current_time = clock.unix_timestamp;
```

---

## Success Criteria

Your program should:

- ✅ Accept campaign creation with goal and deadline
- ✅ Accept contributions and track total raised
- ✅ Allow withdrawal only if goal reached after deadline
- ✅ Allow refunds only if goal NOT reached after deadline
- ✅ Prevent double withdrawals
- ✅ Use PDA for vault (not direct transfers)

---

## Testing Checklist

1. Create a campaign with goal=1000 SOL, deadline=tomorrow
2. Contribute 600 SOL → should succeed, raised=600
3. Contribute 500 SOL → should succeed, raised=1100
4. Try withdraw before deadline → should fail
5. Wait until after deadline → withdraw should succeed
6. Try withdraw again → should fail (already claimed)

---

## Common Pitfalls

| Don't                              | Do                         |
| ---------------------------------- | -------------------------- |
| Send donations directly to creator | Use PDA vault              |
| Allow withdrawal before deadline   | Check both goal AND time   |
| Forget to mark claimed=true        | Prevent double withdrawals |
| Use unwrap() everywhere            | Handle errors properly     |

---

## Deliverables

- [ ] Rust program code
- [ ] Deployed to Solana Devnet
- [ ] Program ID
- [ ] Test transaction signatures
