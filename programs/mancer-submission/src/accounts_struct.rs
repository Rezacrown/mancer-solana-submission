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

// ============================================================================
// CREATE CAMPAIGN ACCOUNTS
// ============================================================================

/// Accounts required to create a new crowdfunding campaign
///
/// # What it does:
/// - Creates a new Campaign account
/// - References the vault PDA (but doesn't create it - it's auto-created on first contribution)
///
/// # Validation:
/// - campaign: Will be created (initialized) with init constraint
/// - vault: Derived using PDA with seeds [b"vault", campaign.key()]
/// - creator: Must sign (Signer) and pays for account creation
/// - system_program: Required for creating accounts
#[derive(Accounts)]
pub struct CreateCampaign<'info> {
    /// The campaign account to be created
    ///
    /// - `init`: Create a new account on-chain
    /// - `payer`: The creator pays the rent for this account
    /// - `space`: Size in bytes (8 for discriminator + Campaign::INIT_SPACE)
    #[account(
        init,
        payer = creator,
        space = 8 + Campaign::INIT_SPACE
    )]
    pub campaign: Account<'info, Campaign>,

    /// The vault PDA that will hold campaign funds
    ///
    /// IMPORTANT: We DON'T use `init` here because PDAs are created automatically
    /// when SOL is first transferred to them. This is a key Solana concept!
    ///
    /// We use `seeds` + `bump` to derive and verify this is the correct PDA:
    /// - seeds = [b"vault", campaign.key()]
    /// - bump = the extra byte that makes this a valid PDA (not on curve)
    #[account(
        seeds = [b"vault", campaign.key().as_ref()],
        bump
    )]
    pub vault: SystemAccount<'info>,

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
/// - Transfers SOL from donor to vault PDA
/// - Updates campaign.raised amount
///
/// # Validation:
/// - vault: The PDA to receive funds (derived from campaign key)
/// - campaign: The campaign being contributed to (mut to update raised)
/// - donor: Must sign to authorize the SOL transfer
#[derive(Accounts)]
pub struct Contribute<'info> {
    /// The vault PDA that holds campaign funds
    ///
    /// `mut` because this account will receive (be credited) SOL
    #[account(
        mut,
        seeds = [b"vault", campaign.key().as_ref()],
        bump
    )]
    pub vault: SystemAccount<'info>,

    /// The campaign to contribute to
    ///
    /// `mut` because we need to update the `raised` field
    #[account(mut)]
    pub campaign: Account<'info, Campaign>,

    /// The donor contributing SOL
    ///
    /// Must be a Signer to authorize the transfer from their wallet
    #[account(mut)]
    pub donor: Signer<'info>,

    /// System program for SOL transfer (CPI)
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
        has_one = creator
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
/// - Transfers specified amount from vault to donor
///
/// # Validation:
/// - vault: Source of funds (derived from campaign key)
/// - campaign: Read-only, needed to verify goal not reached
/// - donor: The signer who receives refund
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
    ///
    /// Note: We derive the PDA using campaign.creator (not has_one)
    /// because we don't have the creator as a separate account here
    #[account(
        seeds = [b"campaign", campaign.creator.as_ref()],
        bump
    )]
    pub campaign: Account<'info, Campaign>,

    /// The donor requesting refund
    #[account(mut)]
    pub donor: Signer<'info>,

    /// System program for SOL transfer
    pub system_program: Program<'info, System>,
}
