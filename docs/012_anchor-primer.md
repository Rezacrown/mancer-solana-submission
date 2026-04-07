# Anchor Framework Primer - Learning Guide

Anchor is a framework for building Solana programs that simplifies development by handling a lot of the boilerplate. Think of it as "React for Solana" - it makes building smart contracts much easier.

---

## 1. Why Anchor?

### Without Anchor (Raw Rust)

```rust
// You'd have to manually:
// - Parse instruction data
// - Validate accounts
// - Handle serialization
// - Create accounts

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: &[u8]
) -> ProgramResult {
    // Manually deserialize data
    let instruction = Instruction::unpack(data)?;

    // Manually validate accounts
    let campaign_account = &accounts[0];

    // Manually serialize/deserialize account data
    let mut campaign_data = Campaign::unpack(&campaign_account.data.borrow())?;
    campaign_data.raised += amount;
    Campaign::pack(&campaign_data, &mut campaign_account.data.borrow_mut())?;

    Ok(())
}
```

### With Anchor

```rust
#[program]
pub mod my_campaign {
    pub fn contribute(ctx: Context<Contribute>, amount: u64) -> Result<()> {
        ctx.accounts.campaign.raised += amount;
        Ok(())
    }
}
```

---

## 2. Core Concepts

### Program Module

```rust
#[program]
pub mod my_program {
    use super::*;

    pub fn my_instruction(ctx: Context<MyAccounts>, param: u64) -> Result<()> {
        // Your logic
        Ok(())
    }
}
```

Key points:

- `#[program]` - Marks this as the entry point
- `pub fn` - Public function becomes an instruction
- `Context<Accounts>` - Contains all account info
- `Result<()>` - Anchor's error handling

---

### Accounts

Accounts are defined as Rust structs with attributes:

```rust
#[derive(Accounts)]
pub struct CreateCampaign<'info> {
    #[account(
        init,                    // Create new account
        payer = creator,         // Who pays for creation
        space = 8 + Campaign::INIT_SPACE  // Account size
    )]
    pub campaign: Account<'info, Campaign>,

    #[account(mut)]
    pub creator: Signer<'info>,  // Must sign transaction

    pub system_program: Program<'info, System>,
}
```

#### Account Constraints

| Constraint      | Description                   |
| --------------- | ----------------------------- |
| `init`          | Create new account            |
| `mut`           | Account can be modified       |
| `payer = X`     | Who pays for account creation |
| `space = N`     | Data size in bytes            |
| `seeds = [...]` | For PDA derivation            |
| `bump = X`      | PDA bump seed                 |
| `has_one = X`   | Validate account owner        |

---

### Account Types

#### 1. `Account<T>`

For accounts owned by your program:

```rust
#[derive(Accounts)]
pub struct CampaignAccounts<'info> {
    #[account(mut)]
    pub campaign: Account<'info, Campaign>,  // Your program's account
}
```

#### 2. `Signer<T>`

For accounts that must sign:

```rust
pub struct ContributeAccounts<'info> {
    pub donor: Signer<'info>,  // Must sign the transaction
}
```

#### 3. `Program<T>`

For calling other programs:

```rust
pub struct WithdrawAccounts<'info> {
    pub system_program: Program<'info, System>,  // Built-in System program
}
```

#### 4. `CpiAccount<T>`

For account info from other programs:

```rust
pub struct TokenAccounts<'info> {
    pub token_account: CpiAccount<'info, TokenAccount>,
}
```

---

### Custom Errors

```rust
#[error_code]
pub enum CampaignError {
    #[msg("Campaign goal not reached")]
    GoalNotReached,

    #[msg("Campaign deadline not passed")]
    DeadlineNotPassed,

    #[msg("Campaign already claimed")]
    AlreadyClaimed,

    #[msg("Only creator can withdraw")]
    NotCreator,
}
```

Usage:

```rust
if campaign.raised < campaign.goal {
    return Err(CampaignError::GoalNotReached.into());
}
```

---

## 3. Data Structures

### Defining Account Data

```rust
#[account]
#[derive(InitSpace)]  // Auto-calculate space needed
pub struct Campaign {
    pub creator: Pubkey,
    pub goal: u64,
    pub raised: u64,
    pub deadline: i64,
    pub claimed: bool,
}
```

**Note:** `InitSpace` derives the correct space automatically. Otherwise you'd need:

```rust
const INIT_SPACE: usize = 32 + 8 + 8 + 8 + 1;  // Pubkey + u64 + u64 + i64 + bool
```

### Space Calculation

| Type   | Size                    |
| ------ | ----------------------- |
| Pubkey | 32 bytes                |
| u8     | 1 byte                  |
| u32    | 4 bytes                 |
| u64    | 8 bytes                 |
| i64    | 8 bytes                 |
| bool   | 1 byte                  |
| String | 4 + len bytes           |
| Vec<T> | 4 + (len \* size_of<T>) |

---

## 4. Context

### What is Context?

`Context` contains everything an instruction needs:

```rust
pub struct Context<'a, T> {
    pub accounts: &'a T,           // Parsed accounts
    pub bump: std::collections::HashMap<String, u8>,  // PDA bumps
    pub remaining_accounts: &'a [AccountInfo],
    pub program_id: &'a Pubkey,
}
```

### Accessing Accounts

```rust
pub fn contribute(ctx: Context<Contribute>, amount: u64) -> Result<()> {
    // Access account data
    let campaign = &mut ctx.accounts.campaign;
    campaign.raised += amount;

    // Access signer info
    let donor = ctx.accounts.donor.key();

    Ok(())
}
```

---

## 5. Cross-Program Invocation (CPI)

### Calling Another Program

```rust
use anchor_lang::system_program::{transfer, Transfer};

pub fn withdraw(ctx: Context<Withdraw>) -> Result<()> {
    let cpi_program = ctx.accounts.system_program.to_account_info();
    let cpi_accounts = Transfer {
        from: ctx.accounts.vault.to_account_info(),
        to: ctx.accounts.creator.to_account_info(),
    };

    let lamports = ctx.accounts.vault.lamports();

    anchor_lang::system_program::transfer(
        CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds),
        lamports
    )?;

    Ok(())
}
```

---

## 6. Program Derived Addresses (PDAs)

### Deriving PDAs in Anchor

```rust
#[derive(Accounts)]
pub struct Contribute<'info> {
    #[account(
        mut,
        seeds = [b"vault", campaign.key().as_ref()],
        bump
    )]
    pub vault: SystemAccount<'info>,
}
```

Anchor automatically:

- Derives the PDA
- Validates it's correct
- Provides the bump seed

---

## 7. Complete Example

### Program Code

```rust
use anchor_lang::prelude::*;
use anchor_lang::system_program::{transfer, Transfer};

declare_id!("YourProgramIdHere...");

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

        // Transfer SOL from donor to vault
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
}

#[derive(Accounts)]
pub struct CreateCampaign<'info> {
    #[account(
        init,
        payer = creator,
        space = 8 + Campaign::INIT_SPACE
    )]
    pub campaign: Account<'info, Campaign>,
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

## 8. Anchor.toml Configuration

```toml
[provider]
cluster = "devnet"           # Which network
wallet = "~/.config/solana/id.json"

[programs.localnet]
my_program = "YourProgramId"

[programs.devnet]
my_program = "YourProgramId"
```

---

## Summary

Key Anchor concepts:

1. `#[program]` - Marks the module as the program entry point
2. `#[derive(Accounts)]` - Defines account validation
3. `#[account]` - Marks struct as account data
4. `Context<T>` - Contains all account info and validation
5. `#[error_code]` - Custom error definitions
6. Constraints - `init`, `mut`, `seeds`, `bump`, etc.

---

## Next Steps

- Read `013_pda-explained.md` - Deep dive into Program Derived Addresses
- Start implementing the crowdfunding program
