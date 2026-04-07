# Program Derived Addresses (PDA) - Deep Dive

PDAs are one of the most powerful features in Solana. They're essential for the crowdfunding contract, so let's understand them deeply.

---

## 1. What is a PDA?

A **Program Derived Address (PDA)** is a special type of account address that:

- Is derived from a program ID and some seeds
- **Has no private key** - only the program can control it
- Is used to create "program-controlled" accounts

### Analogy

Think of a PDA like a safety deposit box:

- Regular account = Your personal safe (you have the key)
- PDA = Bank's safe (only the bank can open it with the right code)

---

## 2. Why Use PDAs?

### Without PDA (Direct Transfer Problem)

```rust
// BAD: Direct transfer to creator
pub fn contribute_bad(ctx: Context<Contribute>, amount: u64) -> Result<()> {
    // This is insecure!
    // Creator gets funds immediately
    // No way to refund if goal not met
    // Creator could run away with money

    transfer(
        CpiContext::new(...),
        amount  // Goes directly to creator
    )?;
    Ok(())
}
```

### With PDA (Secure Escrow)

```rust
// GOOD: Use PDA as vault
pub fn contribute_good(ctx: Context<Contribute>, amount: u64) -> Result<()> {
    // Funds go to PDA vault
    // Program controls when funds can be released
    // Can implement refund logic

    transfer(
        CpiContext::new(...),
        amount  // Goes to PDA vault
    )?;
    Ok(())
}
```

**Benefits:**

- ✅ Funds are locked until conditions are met
- ✅ Can implement refund logic
- ✅ Donor can verify funds are secured
- ✅ Program acts as escrow

---

## 3. How PDAs Work

### Derivation Algorithm

```
PDA = hash(
    "Program ID" +
    "User-provided seeds" +
    "bump seed"
)
```

The "bump seed" is a value (0-255) that ensures the address doesn't fall on the curve (i.e., doesn't have a valid private key).

### Finding a PDA

```rust
use anchor_lang::prelude::*;

// Method 1: Using find_program_address (raw)
let (pda, bump) = Pubkey::find_program_address(
    &[b"vault", campaign.key().as_ref()],
    program_id
);

// Method 2: Using Anchor's bump constraint (preferred)
#[account(
    seeds = [b"vault", campaign.key().as_ref()],
    bump
)]
pub vault: SystemAccount<'info>,
```

---

## 4. PDA in Anchor

### Defining a PDA Account

```rust
#[derive(Accounts)]
pub struct Contribute<'info> {
    // PDA vault account
    #[account(
        mut,                                    // Can be modified
        seeds = [b"vault", campaign.key().as_ref()],  // Seeds
        bump                                    // Auto-fill bump
    )]
    pub vault: SystemAccount<'info>,

    // The campaign account
    #[account(
        mut,
        seeds = [b"campaign", creator.key().as_ref()],
        bump
    )]
    pub campaign: Account<'info, Campaign>,

    // Donor who contributes
    #[account(mut)]
    pub donor: Signer<'info>,

    // System program
    pub system_program: Program<'info, System>,
}
```

---

## 5. Signing with PDA

The magic of PDAs: your program can "sign" for them using `invoke_signed`.

### Why Signing Matters

When transferring FROM a PDA, the Solana runtime needs to verify the program has authority to do so. This is done via `invoke_signed`.

### How to Sign

```rust
use anchor_lang::system_program::{transfer, Transfer};

pub fn withdraw(ctx: Context<Withdraw>) -> Result<()> {
    // Create seeds for signing
    let seeds = &[
        b"vault",
        ctx.accounts.campaign.key().as_ref(),
        &[ctx.bumps.vault]  // Must use bump from accounts
    ];

    // Create signer (program acts on behalf of PDA)
    let signer = &[seeds.as_slice()];

    // Transfer from PDA to creator
    transfer(
        CpiContext::new_with_signer(
            ctx.accounts.system_program.to_account_info(),
            Transfer {
                from: ctx.accounts.vault.to_account_info(),
                to: ctx.accounts.creator.to_account_info(),
            },
            signer  // Seeds for signing
        ),
        amount
    )?;

    Ok(())
}
```

### The Signer Seeds

The key insight:

```rust
// These seeds must MATCH the PDA derivation
let seeds = &[
    b"vault",                    // Must match
    campaign.key().as_ref(),     // Must match
    &[bump]                      // Must match
];

// Solana will verify:
// hash(program_id, seeds) = PDA address
// If match, program is authorized to sign
```

---

## 6. Complete PDA Example

### Program

```rust
use anchor_lang::prelude::*;
use anchor_lang::system_program::{transfer, Transfer};

declare_id!("YourProgramId...");

#[program]
pub mod crowdfunding {
    use super::*;

    pub fn create_campaign(
        ctx: Context<CreateCampaign>,
        goal: u64,
        deadline: i64
    ) -> Result<()> {
        let campaign = &mut ctx.accounts.campaign;
        campaign.creator = ctx.accounts.creator.key();
        campaign.goal = goal;
        campaign.raised = 0;
        campaign.deadline = deadline;
        campaign.claimed = false;

        msg!("Campaign created: goal={}, deadline={}", goal, deadline);
        Ok(())
    }

    pub fn contribute(ctx: Context<Contribute>, amount: u64) -> Result<()> {
        let campaign = &mut ctx.accounts.campaign;
        campaign.raised += amount;

        // Transfer from donor TO vault (PDA)
        transfer(
            CpiContext::new(
                ctx.accounts.system_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.donor.to_account_info(),
                    to: ctx.accounts.vault.to_account_info(),
                }
            ),
            amount
        )?;

        msg!("Contributed: {} lamports, total={}", amount, campaign.raised);
        Ok(())
    }

    pub fn withdraw(ctx: Context<Withdraw>) -> Result<()> {
        let campaign = &ctx.accounts.campaign;
        let vault = &ctx.accounts.vault;
        let amount = vault.lamports;

        // Transfer from vault (PDA) TO creator
        // Must use invoke_signed because vault is a PDA
        transfer(
            CpiContext::new_with_signer(
                ctx.accounts.system_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.vault.to_account_info(),
                    to: ctx.accounts.creator.to_account_info(),
                },
                &[&[
                    b"vault",
                    ctx.accounts.campaign.key().as_ref(),
                    &[ctx.bumps.vault]
                ]]
            ),
            amount
        )?;

        ctx.accounts.campaign.claimed = true;

        msg!("Withdrawn: {} lamports", amount);
        Ok(())
    }
}

// ============ ACCOUNTS ============

#[derive(Accounts)]
pub struct CreateCampaign<'info> {
    #[account(
        init,
        payer = creator,
        space = 8 + Campaign::INIT_SPACE,
        seeds = [b"campaign", creator.key().as_ref()],
        bump
    )]
    pub campaign: Account<'info, Campaign>,
    #[account(mut, seeds = [b"vault", creator.key().as_ref()], bump)]
    pub vault: SystemAccount<'info>,
    #[account(mut)]
    pub creator: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Contribute<'info> {
    #[account(
        mut,
        seeds = [b"vault", campaign.key().as_ref()],
        bump
    )]
    pub vault: SystemAccount<'info>,
    #[account(mut)]
    pub campaign: Account<'info, Campaign>,
    #[account(mut)]
    pub donor: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(
        mut,
        seeds = [b"vault", campaign.key().as_ref()],
        bump
    )]
    pub vault: SystemAccount<'info>,
    #[account(
        mut,
        seeds = [b"campaign", creator.key().as_ref()],
        bump
    )]
    pub campaign: Account<'info, Campaign>,
    #[account(mut)]
    pub creator: Signer<'info>,
    pub system_program: Program<'info, System>,
}

// ============ DATA STRUCTURES ============

#[account]
#[derive(InitSpace)]
pub struct Campaign {
    pub creator: Pubkey,
    pub goal: u64,
    pub raised: u64,
    pub deadline: i64,
    pub claimed: bool,
}
```

---

## 7. Common PDA Patterns

### Pattern 1: Single PDA per User

```rust
// One vault per campaign creator
let (vault, bump) = Pubkey::find_program_address(
    &[b"vault", creator.key().as_ref()],
    program_id
);
```

### Pattern 2: PDA per Campaign

```rust
// Unique vault for each campaign
let (vault, bump) = Pubkey::find_program_address(
    &[b"vault", campaign.key().as_ref()],
    program_id
);
```

### Pattern 3: Nested PDAs

```rust
// PDA derived from another PDA
let (inner_pda, _) = Pubkey::find_program_address(
    &[b"inner", outer_pda.key().as_ref()],
    program_id
);
```

---

## 8. Debugging PDAs

### Common Errors

| Error                                         | Cause                   | Fix                              |
| --------------------------------------------- | ----------------------- | -------------------------------- |
| `AnchorError: The given account is not a PDA` | Wrong seeds used        | Verify seeds match derivation    |
| `Signature constraint required`               | Missing bump constraint | Add `bump` to account constraint |
| `Cross-program invocation failed`             | Signer seeds wrong      | Check seeds match exactly        |

### Debug Tips

```rust
// Add debug logging
msg!("PDA derivation:");
msg!("  seeds: {:?}", seeds);
msg!("  bump: {}", bump);
msg!("  derived: {}", pda);
msg!("  actual: {}", actual_account.key());
```

---

## 9. Security Considerations

### DO ✅

- Always use unique seeds per account
- Validate PDA matches expected address
- Use `bump` constraint for safety
- Log operations for debugging

### DON'T ❌

- Don't use predictable seeds (could be pre-computed)
- Don't forget to use `invoke_signed` for PDA transfers
- Don't skip bump validation
- Don't allow arbitrary seeds (could cause collisions)

---

## Summary

Key PDA concepts:

1. **No private key** - Only program can control
2. **Derived using seeds** - Program ID + seeds = address
3. **Bump seed** - Ensures address is off-curve
4. **`invoke_signed`** - Program signs for PDA
5. **Vault pattern** - Use for escrowing funds

---

## Next Steps

- Read `014_deployment-guide.md` - Learn how to deploy and test
- Start implementing the crowdfunding program
