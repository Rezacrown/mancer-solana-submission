//! Solana Crowdfunding Smart Contract - Main Entry Point
//!
//! ============================================================================
//! MODULE STRUCTURE
//! ============================================================================
//!
//! lib.rs       - Entry point, declares program ID and instruction handlers
//! errors.rs    - Custom error codes (CampaignError enum)
//! campaign.rs  - Campaign account data structure  
//! accounts.rs  - All #[derive(Accounts)] validation structs
//!
//! ============================================================================
//! IMPORT EXPLANATIONS
//! ============================================================================
//!
//! 1. `use anchor_lang::prelude::*` - Imports common Anchor types:
//!    - Result<T> - Anchor's error handling type (alias for std::result::Result<T, Error>)
//!    - Context<T> - Context for instructions (contains all accounts)
//!    - msg! - Macro for logging messages on-chain (viewable in explorer)
//!    - Program, Account, Signer, SystemAccount, InterfaceAccount - Account types
//!    - Pubkey - Solana public key (32 bytes)
//!    - AccountMeta, RemainingAccount - For advanced CPI
//!    - CpiContext, CpiContext::new, CpiContext::new_with_signer - For cross-program calls
//!    - Clock - Sysvar for getting current time
//!    - System - The system program
//!
//! 2. `use anchor_lang::system_program::{transfer, Transfer}`
//!    Imports specifically from the system_program module:
//!    - transfer() - Function to transfer SOL via CPI to System program
//!    - Transfer - Struct defining {from, to} accounts for the transfer
//!    This is used for moving SOL between accounts.
//!
//! 3. `mod crate::errors::CampaignError` - Our custom error enum
//!
//! 4. `mod crate::campaign::Campaign` - Campaign data struct
//!
//! 5. `mod crate::accounts::*` - Account validation structs

// ============================================================================
// MODULE DECLARATIONS
// ============================================================================

mod accounts_struct;
mod campaign; // Campaign data structure
mod errors; // Custom error codes // Account validation structs

// ============================================================================
// IMPORTS
// ============================================================================

// Import common Anchor types (prelude includes most used types)
use anchor_lang::prelude::*;

// Import system program transfer function for moving SOL
// This is used for CPI (Cross-Program Invocation) to the System program
use anchor_lang::system_program::{transfer, Transfer};

// Import our custom errors from errors.rs
use crate::errors::CampaignError;

// Import our data structures
use crate::campaign::Campaign;

// Import our account validation structs from accounts.rs
use crate::accounts_struct::*;

// ============================================================================
// PROGRAM ID
// ============================================================================

/// The program ID - this MUST match what's in Anchor.toml
/// This identifies this specific program on Solana
declare_id!("2bvT3M5bLbJgk8dcm3jsDkSEn8B2ntk2Eokmt2UKb7pU");

// ============================================================================
// INSTRUCTION HANDLERS
// ============================================================================

/// The main program module containing all instruction functions
#[program]
pub mod mancer_submission {
    // Bring parent module types into scope
    use super::*;

    /// Create a new crowdfunding campaign
    ///
    /// # Arguments
    /// * `goal` - Target amount in lamports (1 SOL = 1,000,000,000 lamports)
    /// * `deadline` - Unix timestamp when the campaign ends
    ///
    /// # Flow
    /// 1. Get current time from Solana's Clock sysvar
    /// 2. Validate deadline is in the future
    /// 3. Initialize campaign with creator, goal, deadline, raised=0, claimed=false
    ///
    /// # Accounts (via Context<CreateCampaign>)
    /// - campaign: New account to store campaign data (created by this instruction)
    /// - vault: PDA that will hold funds (derived, not created here)
    /// - creator: Must sign and pays for account creation
    /// - system_program: Required for account operations
    pub fn create_campaign(
        ctx: Context<CreateCampaign>, // Contains all validated accounts
        goal: u64,                    // Target amount in lamports
        deadline: i64,                // Unix timestamp
    ) -> Result<()> {
        // Get current time from Solana's Clock sysvar
        // Clock is a special account that always has the current time
        let clock = Clock::get()?;
        let current_time = clock.unix_timestamp;

        // Validation: Deadline must be in the future
        if deadline <= current_time {
            return Err(CampaignError::DeadlineMustBeInFuture.into());
        }

        // Initialize campaign data
        let campaign = &mut ctx.accounts.campaign;
        campaign.creator = ctx.accounts.creator.key(); // Store creator's public key
        campaign.goal = goal; // Set target amount
        campaign.raised = 0; // Start with 0 raised
        campaign.deadline = deadline; // Set end time
        campaign.claimed = false; // Not claimed yet

        msg!(
            "Campaign created: goal={} lamports, deadline={}",
            goal,
            deadline
        );
        Ok(())
    }

    /// Contribute SOL to a campaign
    ///
    /// # Arguments
    /// * `amount` - Amount to donate in lamports
    ///
    /// # Flow
    /// 1. Validate amount > 0
    /// 2. Update campaign.raised += amount
    /// 3. Transfer SOL from donor to vault PDA
    ///
    /// # Accounts (via Context<Contribute>)
    /// - vault: PDA receiving the funds (derived from campaign key)
    /// - campaign: Campaign to contribute to (mut - updating raised amount)
    /// - donor: Must sign to authorize transfer from their wallet
    /// - system_program: Required for the transfer
    pub fn contribute(
        ctx: Context<Contribute>, // Contains vault, campaign, donor, system_program
        amount: u64,              // Amount in lamports
    ) -> Result<()> {
        // Validation: Amount must be positive
        if amount == 0 {
            return Err(CampaignError::InvalidAmount.into());
        }

        // Update the campaign's raised amount
        let campaign = &mut ctx.accounts.campaign;
        campaign.raised += amount;

        // Transfer SOL from donor to the vault PDA
        // This is a CPI (Cross-Program Invocation) to the System program
        // We call system_program::transfer to move SOL
        transfer(
            CpiContext::new(
                ctx.accounts.system_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.donor.to_account_info(),
                    to: ctx.accounts.vault.to_account_info(),
                },
            ),
            amount,
        )?;

        msg!(
            "Contributed: {} lamports, total raised={}",
            amount,
            campaign.raised
        );
        Ok(())
    }

    /// Withdraw funds from a successful campaign
    ///
    /// # Conditions (ALL must be true)
    /// - Campaign raised >= goal (goal reached)
    /// - Current time >= deadline (deadline passed)
    /// - Caller is the campaign creator
    /// - Campaign not already claimed (prevent double withdrawal)
    ///
    /// # Flow
    /// 1. Get current time
    /// 2. Validate goal reached
    /// 3. Validate deadline passed
    /// 4. Validate not already claimed
    /// 5. Transfer all SOL from vault to creator
    /// 6. Mark campaign as claimed
    ///
    /// # Accounts (via Context<Withdraw>)
    /// - vault: PDA holding the funds (transfer OUT from here)
    /// - campaign: Validates creator and claimed status
    /// - creator: Receives funds and must sign
    /// - system_program: Required for transfer
    pub fn withdraw(ctx: Context<Withdraw>) -> Result<()> {
        let campaign = &ctx.accounts.campaign;
        let clock = Clock::get()?;
        let current_time = clock.unix_timestamp;

        // Validation 1: Goal must be reached
        if campaign.raised < campaign.goal {
            return Err(CampaignError::GoalNotReached.into());
        }

        // Validation 2: Deadline must have passed
        if current_time < campaign.deadline {
            return Err(CampaignError::DeadlineNotPassed.into());
        }

        // Validation 3: Not already claimed
        if campaign.claimed {
            return Err(CampaignError::AlreadyClaimed.into());
        }

        // Get the vault's balance (all SOL held in the vault PDA)
        let vault = &ctx.accounts.vault;
        let amount = vault.lamports();

        // Prepare the seeds for PDA signing
        // IMPORTANT: We must use invoke_signed because vault is a PDA
        // PDAs don't have private keys - our program signs on their behalf
        let campaign_key = ctx.accounts.campaign.key();
        let seeds = &[b"vault", campaign_key.as_ref(), &[ctx.bumps.vault]];

        // Transfer from vault PDA to creator
        // Using CpiContext::new_with_signer to sign on behalf of the PDA
        transfer(
            CpiContext::new_with_signer(
                ctx.accounts.system_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.vault.to_account_info(),
                    to: ctx.accounts.creator.to_account_info(),
                },
                &[seeds],
            ),
            amount,
        )?;

        // Mark as claimed to prevent double withdrawal
        ctx.accounts.campaign.claimed = true;

        msg!("Withdrawn: {} lamports", amount);
        Ok(())
    }

    /// Get a refund if campaign failed (goal not met after deadline)
    ///
    /// # Conditions (ALL must be true)
    /// - Current time >= deadline (deadline passed)
    /// - Campaign raised < goal (goal NOT reached - campaign failed)
    /// - Donor specifies amount to refund
    ///
    /// # Flow
    /// 1. Get current time
    /// 2. Validate deadline passed
    /// 3. Validate goal NOT reached (campaign failed)
    /// 4. Validate amount > 0
    /// 5. Transfer amount from vault to donor
    ///
    /// # Accounts (via Context<Refund>)
    /// - vault: PDA holding funds (transfer OUT from here)
    /// - campaign: Read-only, needed to check deadline and goal
    /// - donor: Receives refund and must sign
    /// - system_program: Required for transfer
    pub fn refund(ctx: Context<Refund>, amount: u64) -> Result<()> {
        let campaign = &ctx.accounts.campaign;
        let clock = Clock::get()?;
        let current_time = clock.unix_timestamp;

        // Validation 1: Deadline must have passed
        if current_time < campaign.deadline {
            return Err(CampaignError::DeadlineNotPassed.into());
        }

        // Validation 2: Goal must NOT be reached (campaign failed)
        if campaign.raised >= campaign.goal {
            return Err(CampaignError::GoalReached.into());
        }

        // Validation 3: Amount must be positive
        if amount == 0 {
            return Err(CampaignError::InvalidAmount.into());
        }

        // Prepare seeds for PDA signing
        let campaign_key = ctx.accounts.campaign.key();
        let seeds = &[b"vault", campaign_key.as_ref(), &[ctx.bumps.vault]];

        // Transfer from vault PDA to donor
        transfer(
            CpiContext::new_with_signer(
                ctx.accounts.system_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.vault.to_account_info(),
                    to: ctx.accounts.donor.to_account_info(),
                },
                &[seeds],
            ),
            amount,
        )?;

        msg!("Refunded: {} lamports", amount);
        Ok(())
    }
}
