# Security & Requirements Audit Report

**Date:** 2026-04-07
**Status:** 🔴 Critical Issues Found

---

## 📊 Current Score Analysis

| Category      | Max     | Current | Status      |
| ------------- | ------- | ------- | ----------- |
| Functionality | 30      | 22      | ⚠️ Yellow   |
| Code Quality  | 20      | 16      | ⚠️ Yellow   |
| Design        | 20      | 17      | ⚠️ Yellow   |
| Documentation | 15      | 13      | ✅ Good     |
| Security      | 10      | 4       | 🔴 Critical |
| Innovation    | 5       | 2       | ⚠️ Yellow   |
| **TOTAL**     | **100** | **74**  | ⚠️ Yellow   |

---

## 🔴 CRITICAL ISSUES (Must Fix)

### Issue #1: Refund Has No Donor Tracking

**Severity:** 🔴 Critical
**Category:** Security
**Score Impact:** -4 points

#### Problem

The `refund()` function allows ANYONE to claim ANY amount, even if they never contributed to the campaign. There is no verification that the caller actually made a contribution.

#### Current Code

```rust
// lib.rs:273-311
pub fn refund(ctx: Context<Refund>, amount: u64) -> Result<()> {
    // No check: Did this donor actually contribute?
    // No check: Is amount <= their contribution?

    // Just validates:
    // 1. Deadline passed
    // 2. Goal not reached
    // 3. Amount > 0

    // Then transfers amount from vault to donor!
}
```

#### Attack Vector

1. Alice contributes 100 SOL to a campaign
2. Campaign fails (goal not reached)
3. Bob (who NEVER contributed) calls `refund(100 SOL)`
4. Bob receives Alice's 100 SOL
5. Alice loses her funds

#### Why This Happens

- `Refund` accounts struct only has `donor: Signer<'info>`
- No verification that donor has a contribution record
- No check that `amount` matches their actual contribution

#### Solution

Implement a `Contribution` PDA that tracks each donor's contribution:

```rust
// New file: contribution.rs
#[account]
#[derive(InitSpace)]
pub struct Contribution {
    pub campaign: Pubkey,  // 32 bytes
    pub donor: Pubkey,     // 32 bytes
    pub amount: u64,       // 8 bytes
}
// PDA seeds: [b"contribution", campaign.key(), donor.key()]
```

#### Files to Modify

1. **NEW:** `programs/mancer-submission/src/contribution.rs`
2. `accounts_struct.rs` - Add contribution account to Contribute and Refund
3. `lib.rs` - Update contribute() to create Contribution PDA
4. `lib.rs` - Update refund() to verify contribution exists and check amount
5. `errors.rs` - Add new error codes

---

### Issue #2: Withdraw PDA Seeds Mismatch

**Severity:** 🔴 Critical
**Category:** Functionality
**Score Impact:** -3 points

#### Problem

The `Withdraw` struct expects `campaign` to be derived with `seeds = [b"campaign", creator.key()]`, but `CreateCampaign` doesn't use these seeds when creating the campaign.

#### Current Code - CreateCampaign (accounts_struct.rs:35-68)

```rust
#[derive(Accounts)]
pub struct CreateCampaign<'info> {
    // NO seeds specified here!
    #[account(
        init,
        payer = creator,
        space = 8 + Campaign::INIT_SPACE
    )]
    pub campaign: Account<'info, Campaign>,
    // ...
}
```

#### Current Code - Withdraw (accounts_struct.rs:127-154)

```rust
#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(
        mut,
        seeds = [b"campaign", creator.key().as_ref()],  // Expects seeds!
        bump,
        has_one = creator
    )]
    pub campaign: Account<'info, Campaign>,
    // ...
}
```

#### Why This Breaks

- In Solana, PDAs must be derived consistently
- `CreateCampaign` creates campaign without seeds → it's a regular account
- `Withdraw` expects campaign with seeds → PDA derivation will fail
- Result: `withdraw()` will ALWAYS fail with signature verification error

#### Solution

Add seeds to `CreateCampaign`:

```rust
#[account(
    init,
    payer = creator,
    space = 8 + Campaign::INIT_SPACE,
    seeds = [b"campaign", creator.key().as_ref()],
    bump
)]
pub campaign: Account<'info, Campaign>,
```

#### Files to Modify

1. `accounts_struct.rs` - Line 41-45, add seeds and bump

---

### Issue #3: Refund Account Seeds Mismatch

**Severity:** 🟡 Medium
**Category:** Functionality
**Score Impact:** -2 points

#### Problem

The `Refund` struct uses `campaign.creator` for seeds derivation, but this might not match if campaign was created without seeds (Issue #2).

#### Current Code (accounts_struct.rs:183-187)

```rust
#[account(
    seeds = [b"campaign", campaign.creator.as_ref()],
    bump
)]
pub campaign: Account<'info, Campaign>,
```

#### Why This Is Risky

- If Issue #2 isn't fixed, this will also fail
- If Issue #2 is fixed, this should work correctly
- But we need consistency across all account validations

#### Solution

Same as Issue #2 - fix CreateCampaign to use seeds.

---

## ⚠️ WARNINGS (Should Fix)

### Issue #4: No Goal Bounds Validation

**Severity:** 🟡 Medium
**Category:** Security
**Score Impact:** -1 point

#### Problem

Creator can set `goal = u64::MAX` which could cause:

- Overflow in `campaign.raised += amount`
- Unrealistic expectations
- Potential denial of service

#### Current Code (lib.rs:101-130)

```rust
pub fn create_campaign(
    ctx: Context<CreateCampaign>,
    goal: u64,
    deadline: i64,
) -> Result<()> {
    // No validation on goal bounds!
    if deadline <= current_time {
        return Err(CampaignError::DeadlineMustBeInFuture.into());
    }
    // ...
}
```

#### Recommendation

Add reasonable bounds:

- Minimum goal: 0.1 SOL (100,000,000 lamports) - prevent spam
- Maximum goal: 1,000,000 SOL - reasonable limit

#### Note

User chose "Only fix critical bugs" so this is deferred.

---

### Issue #5: No Deadline Maximum

**Severity:** 🟡 Medium
**Category:** Security
**Score Impact:** -1 point

#### Problem

Creator can set deadline decades in the future, locking funds indefinitely.

#### Recommendation

Add maximum duration:

- Max duration: 365 days (1 year)
- Min duration: 1 hour

#### Note

User chose "Only fix critical bugs" so this is deferred.

---

### Issue #6: Overflow Risk in Raised Amount

**Severity:** 🟡 Medium
**Category:** Security
**Score Impact:** -1 point

#### Problem

Using `campaign.raised += amount` can overflow if raised is near u64::MAX.

#### Current Code (lib.rs:157-158)

```rust
let campaign = &mut ctx.accounts.campaign;
campaign.raised += amount;  // Can overflow!
```

#### Solution

Use checked arithmetic:

```rust
campaign.raised = campaign.raised.checked_add(amount)
    .ok_or(CampaignError::AmountOverflow)?;
```

#### Note

User chose "Only fix critical bugs" so this is deferred.

---

## 📋 IMPLEMENTATION CHECKLIST

### Phase 1: Critical Fixes (REQUIRED)

- [ ] **Issue #1: Implement Contribution PDA**
  - [ ] Create `contribution.rs` file
  - [ ] Update `accounts_struct.rs` for Contribute accounts
  - [ ] Update `accounts_struct.rs` for Refund accounts
  - [ ] Update `lib.rs` - contribute() to create Contribution PDA
  - [ ] Update `lib.rs` - contribute() to update existing Contribution
  - [ ] Update `lib.rs` - refund() to verify Contribution ownership
  - [ ] Update `lib.rs` - refund() to verify amount <= contribution.amount
  - [ ] Update `lib.rs` - refund() to decrease contribution.amount
  - [ ] Add error codes to `errors.rs`

- [ ] **Issue #2: Fix CreateCampaign Seeds**
  - [ ] Update `accounts_struct.rs` - Add seeds to CreateCampaign
  - [ ] Update `lib.rs` - Update create_campaign signature if needed

- [ ] **Issue #3: Verify All PDAs Consistent**
  - [ ] Test CreateCampaign PDA derivation
  - [ ] Test Withdraw PDA derivation
  - [ ] Test Refund PDA derivation
  - [ ] Test Contribution PDA derivation

### Phase 2: Testing (REQUIRED)

- [ ] Update test file with contribution tracking tests
- [ ] Add test: Non-donor cannot refund
- [ ] Add test: Donor cannot refund more than contributed
- [ ] Add test: Multiple contributions from same donor
- [ ] Add test: Partial refund works correctly

### Phase 3: Verification (REQUIRED)

- [ ] Run `anchor build` - must compile without errors
- [ ] Run `anchor test` - all tests pass
- [ ] Manual test on devnet

---

## 📁 FILES TO MODIFY

| File                         | Changes                                         |
| ---------------------------- | ----------------------------------------------- |
| `contribution.rs`            | **NEW FILE** - Contribution PDA struct          |
| `campaign.rs`                | No changes needed                               |
| `accounts_struct.rs`         | Fix CreateCampaign seeds, add Contribution PDAs |
| `errors.rs`                  | Add 3 new error codes                           |
| `lib.rs`                     | Fix seeds, implement contribution tracking      |
| `tests/mancer-submission.ts` | Add contribution tracking tests                 |

---

## 🎯 EXPECTED SCORE AFTER FIXES

| Category      | Before | After  | Change  |
| ------------- | ------ | ------ | ------- |
| Functionality | 22     | 28     | +6      |
| Code Quality  | 16     | 18     | +2      |
| Design        | 17     | 18     | +1      |
| Documentation | 13     | 14     | +1      |
| Security      | 4      | 9      | +5      |
| Innovation    | 2      | 4      | +2      |
| **TOTAL**     | **74** | **91** | **+17** |

---

## 📝 NEW ERROR CODES TO ADD

```rust
// errors.rs - Add these new error codes

/// Error: Caller trying to refund has no contribution record
#[msg("No contribution found for this donor")]
NotADonor,

/// Error: Refund amount exceeds donor's contribution
#[msg("Refund amount exceeds your contribution")]
InsufficientContribution,

/// Error: Contribution has already been fully refunded
#[msg("No remaining contribution to refund")]
NoContributionToRefund,
```

---

## 🔧 DETAILED CODE CHANGES

### 1. NEW FILE: `contribution.rs`

```rust
//! Contribution Account Data Structure
//!
//! Tracks each donor's contribution to a specific campaign.
//! PDA seeds: [b"contribution", campaign.key(), donor.key()]

use anchor_lang::prelude::*;

/// Stores a single donor's contribution to a campaign
///
/// # Why We Need This
/// - Prevents non-donors from claiming refunds
/// - Ensures donors can only refund what they contributed
/// - Enables partial refunds
///
/// # PDA Derivation
/// seeds = [b"contribution", campaign.key(), donor.key()]
/// - campaign: Ensures contribution is tied to specific campaign
/// - donor: Ensures only the donor can access this record
#[account]
#[derive(InitSpace)]
pub struct Contribution {
    /// The campaign this contribution is for
    pub campaign: Pubkey, // 32 bytes

    /// The donor who made this contribution
    pub donor: Pubkey, // 32 bytes

    /// Amount contributed in lamports
    /// Decreases when partial refunds are claimed
    pub amount: u64, // 8 bytes
}
// Total: 72 bytes + 8 discriminator = 80 bytes
```

### 2. MODIFY: `accounts_struct.rs`

#### Change 1: Fix CreateCampaign (Line 41-45)

**Before:**

```rust
#[account(
    init,
    payer = creator,
    space = 8 + Campaign::INIT_SPACE
)]
pub campaign: Account<'info, Campaign>,
```

**After:**

```rust
#[account(
    init,
    payer = creator,
    space = 8 + Campaign::INIT_SPACE,
    seeds = [b"campaign", creator.key().as_ref()],
    bump
)]
pub campaign: Account<'info, Campaign>,
```

#### Change 2: Update Contribute struct

Add contribution PDA account and update campaign to use seeds.

#### Change 3: Update Refund struct

Add contribution PDA account and verify ownership.

### 3. MODIFY: `errors.rs`

Add the 3 new error codes listed above.

### 4. MODIFY: `lib.rs`

#### Change 1: Update create_campaign signature

Add bump to function signature if needed.

#### Change 2: Update contribute()

- Create Contribution PDA on first contribution
- Update existing Contribution PDA on subsequent contributions
- Use checked_add for amount tracking

#### Change 3: Update refund()

- Verify contribution account exists
- Verify donor owns the contribution account
- Verify amount <= contribution.amount
- Decrease contribution.amount by refunded amount

---

## ✅ ACCEPTANCE CRITERIA

After implementing all fixes:

1. ✅ `anchor build` compiles without errors
2. ✅ `anchor test` passes all tests
3. ✅ Non-donors cannot call refund (transaction fails)
4. ✅ Donors can only refund up to their contribution amount
5. ✅ Multiple contributions from same donor are tracked correctly
6. ✅ Partial refunds work (donor can refund less than total)
7. ✅ Withdraw works correctly after seeds fix
8. ✅ All PDA derivations are consistent

---

## 📚 REFERENCES

- [Solana Cookbook - PDAs](https://solanacookbook.com/core-concepts/pdas.html)
- [Anchor Documentation - Accounts](https://docs.anchor-lang.com/core-concepts/accounts)
- [Sealevel Attacks - Security Patterns](https://github.com/coral-xyz/sealevel-attacks)

---

**End of Report**
