//! Account Structs for All Instructions
//!
//! This file defines all the account validation structs used by each instruction.
//! Each struct uses Anchor's derive macros to automatically validate accounts.
//!
//! Key Concepts:
//! - #[derive(Accounts)] - macro that generates account validation logic
//! - #[account(...)] - constraints like init, mut, seeds, bump, has_one
//! - Signer<'info> - an account that must sign the transaction
//! - SystemAccount<'info> - represents a PDA or system-owned account
//! - Account<'info, T> - an account owned by this program with specific data type

use anchor_lang::prelude::*;

/// Imports the Campaign struct from campaign.rs
/// This is needed because Account<'info, Campaign> expects the Campaign type
pub use crate::campaign::Campaign;

/// Imports the Contribution struct from contribution.rs
/// This tracks each donor's contribution amount
pub use crate::contribution::Contribution;
use crate::errors::CampaignError;

// ============================================================================
// CREATE CAMPAIGN ACCOUNTS
// ============================================================================

/// Accounts required to create a new crowdfunding campaign
///
/// # What it does:
/// - Creates a new Campaign account as a PDA
///
/// # Note on Vault:
/// The vault PDA is NOT created here. It's automatically created
/// on first contribution when SOL is transferred to it.
/// This saves compute units during campaign creation.
///
/// # Validation:
/// - campaign: Created as PDA with seeds [b"campaign", creator.key()]
/// - creator: Must sign and pays for account creation
/// - system_program: Required for account creation
#[derive(Accounts)]
pub struct CreateCampaign<'info> {
    /// The campaign account to be created (PDA)
    ///
    /// - `init`: Create a new account on-chain
    /// - `payer`: The creator pays the rent for this account
    /// - `space`: Size in bytes (8 for discriminator + Campaign::INIT_SPACE)
    /// - `seeds`: PDA derivation [b"campaign", creator.key()] - ensures consistent address
    /// - `bump`: Anchor finds the canonical bump seed
    #[account(
    init,
    payer = creator,
    space = 8 + Campaign::INIT_SPACE,
    seeds = [b"campaign", creator.key().as_ref()],
    bump
    )]
    pub campaign: Account<'info, Campaign>,

    /// The campaign creator - must sign and pays for account creation
    #[account(mut)]
    pub creator: Signer<'info>,

    /// The System program - required for account operations
    pub system_program: Program<'info, System>,
}

// ============================================================================
// CONTRIBUTE ACCOUNTS
// ============================================================================

/// Accounts required to contribute SOL to a campaign
///
/// # What it does:
/// - Creates Contribution PDA on first contribution (init_if_needed)
/// - Updates existing Contribution PDA on subsequent contributions
/// - Transfers SOL from donor to vault PDA
/// - Updates campaign.raised amount
///
/// # Security Note:
/// - Uses `init_if_needed` to handle both first and subsequent contributions
/// - First contribution: Creates new Contribution account
/// - Subsequent contributions: Updates existing account
/// - Seeds [b"contribution", campaign, donor] ensure only donor can modify
#[derive(Accounts)]
#[instruction(amount: u64)]
pub struct Contribute<'info> {
    /// The vault PDA that holds campaign funds
    #[account(
        mut,
        seeds = [b"vault", campaign.key().as_ref()],
        bump
    )]
    pub vault: SystemAccount<'info>,

    /// The campaign to contribute to (PDA)
    #[account(
        mut,
        seeds = [b"campaign", campaign.creator.as_ref()],
        bump
    )]
    pub campaign: Account<'info, Campaign>,

    /// The contribution PDA - tracks this donor's contribution amount
    ///
    /// Uses `init_if_needed`:
    /// - First contribution: Creates new Contribution account (init)
    /// - Subsequent contributions: Updates existing account (no init)
    ///
    /// Seeds ensure only the correct donor can create/modify this account
    #[account(
        init_if_needed,
        payer = donor,
        space = 8 + Contribution::INIT_SPACE,
        seeds = [b"contribution", campaign.key().as_ref(), donor.key().as_ref()],
        bump
    )]
    pub contribution: Account<'info, Contribution>,

    /// The donor contributing SOL (pays for Contribution PDA creation on first time)
    #[account(mut)]
    pub donor: Signer<'info>,

    /// System program for SOL transfer and account creation
    pub system_program: Program<'info, System>,
}

// ============================================================================
// WITHDRAW ACCOUNTS
// ============================================================================

/// Accounts required to withdraw funds (creator claims money)
///
/// # What it does:
/// - Transfers all SOL from vault to creator
/// - Marks campaign as claimed
///
/// # Validation:
/// - vault: Source of funds (derived from campaign key)
/// - campaign: Validates creator is the real creator, checks claimed status
/// - creator: The signer who receives funds (must be campaign creator)
#[derive(Accounts)]
pub struct Withdraw<'info> {
    /// The vault PDA - source of funds
    #[account(
        mut,
        seeds = [b"vault", campaign.key().as_ref()],
        bump
    )]
    pub vault: SystemAccount<'info>,

    /// The campaign account
    ///
    /// `has_one = creator` ensures only the original creator can withdraw
    /// This is an important security check!
    #[account(
        mut,
        seeds = [b"campaign", creator.key().as_ref()],
        bump,
        has_one = creator,
    )]
    pub campaign: Account<'info, Campaign>,

    /// The creator - receives the funds and must sign
    #[account(mut)]
    pub creator: Signer<'info>,

    /// System program for SOL transfer
    pub system_program: Program<'info, System>,
}

// ============================================================================
// REFUND ACCOUNTS
// ============================================================================

/// Accounts required to get a refund (donor gets money back)
///
/// # What it does:
/// - Verifies donor has a valid contribution record
/// - Transfers requested amount from vault to donor
/// - Decrements contribution amount to prevent double-refund
///
/// # Security:
/// - contribution PDA ensures only actual donors can refund
/// - Seeds tie contribution to specific campaign and donor
/// - Prevents non-donors from draining the vault
#[derive(Accounts)]
pub struct Refund<'info> {
    /// The vault PDA - source of funds
    #[account(
        mut,
        seeds = [b"vault", campaign.key().as_ref()],
        bump
    )]
    pub vault: SystemAccount<'info>,

    /// The campaign - needed to check deadline and goal status
    #[account(
        seeds = [b"campaign", campaign.creator.as_ref()],
        bump
    )]
    pub campaign: Account<'info, Campaign>,

    /// The contribution record - MUST exist and be owned by this donor
    ///
    /// This is the KEY security check:
    /// - Seeds ensure this contribution belongs to THIS donor for THIS campaign
    /// - Amount check in instruction handler ensures refund <= contributed
    /// - After refund, amount is decremented to prevent double-refund
    #[account(
        mut,
        seeds = [b"contribution", campaign.key().as_ref(), donor.key().as_ref()],
        bump,
        has_one = donor @CampaignError::NotADonor,
    )]
    pub contribution: Account<'info, Contribution>,

    /// The donor requesting refund (must match contribution.donor)
    #[account(mut)]
    pub donor: Signer<'info>,

    /// System program for SOL transfer
    pub system_program: Program<'info, System>,
}
