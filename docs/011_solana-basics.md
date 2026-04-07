# Solana Basics - Learning Guide

This document covers the fundamental concepts of Solana blockchain that you need to understand before building smart contracts.

---

## 1. What is Solana?

Solana is a high-performance blockchain that can process thousands of transactions per second. Unlike Ethereum, it uses a unique consensus mechanism called **Proof of History (PoH)**.

### Key Differences from Ethereum

| Aspect     | Ethereum       | Solana                            |
| ---------- | -------------- | --------------------------------- |
| Language   | Solidity       | Rust (native) / C++               |
| Consensus  | Proof of Stake | Proof of History + Proof of Stake |
| TPS        | ~15-30         | ~65,000                           |
| Block Time | ~12-15 seconds | ~400ms                            |

---

## 2. Accounts Model

In Solana, **everything is an account**. This is different from Ethereum's model where only contracts are accounts.

### Types of Accounts

1. **System Account** - Owned by the System Program, holds SOL balance
2. **Program Account** - Holds executable code (smart contracts)
3. **PDA (Program Derived Address)** - Controlled by a program, no private key
4. **Token Account** - Holds SPL tokens

### Account Data Structure

```rust
pub struct Account {
    pub lamports: u64,           // SOL balance (in lamports)
    pub data: Vec<u8>,           // Stored data
    pub owner: Pubkey,          // Program that owns this account
    pub executable: bool,        // Is this a program?
    pub rent_epoch: u64,        // When rent is due
}
```

**Important:**

- Accounts have an **owner** - only the owner program can modify their data
- Data stored in accounts is persistent
- You pay rent to store data (can be exempted with 2 years worth of rent)

---

## 3. Programs (Smart Contracts)

A Solana program is executable code that runs on-chain. There are two types:

1. **Native Programs** - Built into Solana (System, BPF Loader, etc.)
2. **User Programs** - Custom programs you deploy

### Program Structure

```rust
#[program]
pub mod my_program {
    pub fn my_instruction(ctx: Context<MyAccounts>) -> Result<()> {
        // Your logic here
        Ok(())
    }
}
```

### Key Points

- Programs are **stateless** - they don't store state
- State is stored in **accounts** passed to the program
- Programs can only modify accounts they **own**

---

## 4. Transactions & Instructions

### Transaction

A transaction is a bundle of one or more instructions that execute atomically (all or nothing).

```rust
// Example: Send transaction with 2 instructions
let transaction = Transaction::new()
    .add(instruction_1)  // Create campaign
    .add(instruction_2)  // Contribute to it
    .sign(&[payer_keypair]);
```

### Instruction

An instruction is the smallest unit of execution. Each instruction:

- Targets a specific program
- Contains command data (what to do)
- References accounts needed

```rust
// Pseudo-instruction structure
pub struct Instruction {
    pub program_id: Pubkey,    // Which program to call
    pub accounts: Vec<AccountMeta>,  // Which accounts to use
    pub data: Vec<u8>,        // Instruction data (command)
}
```

---

## 5. SOL & Lamports

### denominations

- **SOL** - Primary token
- **Lamport** - Smallest unit (10^-9 SOL)
- 1 SOL = 1,000,000,000 lamports

**Why lamports?** Since Solana uses integer math (no floating point), all amounts are stored as integers in lamports.

```rust
// Example
let amount_sol: f64 = 1.5;
let amount_lamports: u64 = (amount_sol * 1_000_000_000.0) as u64;
```

---

## 6. Getting Current Time

Solana provides time through the `Clock` sysvar:

```rust
use solana_program::clock::Clock;
use solana_program::sysvar::Sysvar;

// Get current timestamp
let clock = Clock::get()?;
let current_time = clock.unix_timestamp;  // i64
```

**Note:** Time on Solana is Unix timestamp (seconds since January 1, 1970).

---

## 7. Errors & Error Handling

### Anchor Result Type

```rust
pub type Result<T> = std::result::Result<T, Error>;
```

### Common Errors

```rust
use anchor_lang::prelude::*;

// Custom error
#[error_code]
pub enum MyError {
    #[msg("Campaign deadline has not passed yet")]
    DeadlineNotPassed,

    #[msg("Campaign goal not reached")]
    GoalNotReached,

    #[msg("Already claimed")]
    AlreadyClaimed,
}

// Using in code
if current_time < deadline {
    return Err(MyError::DeadlineNotPassed.into());
}
```

---

## 8. Key Commands

### Solana CLI

```bash
# Check version
solana --version

# Check balance
solana balance <WALLET_ADDRESS>

# Airdrop SOL (devnet)
solana airdrop 2 <WALLET_ADDRESS>

# Deploy program
solana program deploy <PROGRAM_SO_FILE>
```

### Anchor CLI

```bash
# Build program
anchor build

# Test program
anchor test

# Deploy to devnet
anchor deploy

# Run local validator
anchor run
```

---

## 9. Development Workflow

```
1. Write Rust code (lib.rs)
2. Build with anchor build
3. Test locally with anchor test
4. Deploy to devnet with anchor deploy
5. Verify with solana program show <PROGRAM_ID>
6. Run integration tests
```

---

## Summary

Key takeaways:

1. **Accounts** - All data storage; programs are stateless
2. **PDAs** - Program-controlled accounts without private keys
3. **Instructions** - Unit of execution within transactions
4. **Lamports** - All amounts in integer lamports
5. **Clock** - Use Clock sysvar for current time

---

## Next Steps

Now that you understand the basics, proceed to:

- Read `012_anchor-primer.md` - Learn how Anchor framework simplifies Solana development
- Read `013_pda-explained.md` - Deep dive into Program Derived Addresses
