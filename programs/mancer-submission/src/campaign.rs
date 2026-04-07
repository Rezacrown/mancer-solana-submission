//! Campaign Account Data Structure
//!
//! This file defines the Campaign account which stores all the state
//! for a single crowdfunding campaign.
//!
//! Key Data:
//! - creator: Who created the campaign
//! - goal: Target amount to raise
//! - raised: Current amount raised
//! - deadline: When campaign ends
//! - claimed: Whether funds have been withdrawn

use anchor_lang::prelude::*;

/// The Campaign account - stores all state for a crowdfunding campaign
///
/// This account is created when someone calls create_campaign().
/// It's persisted on-chain and can be read by anyone.
///
/// # Data Fields
/// - `creator`: Pubkey of the campaign creator
/// - `goal`: Target amount in lamports (1 SOL = 1e9 lamports)
/// - `raised`: Current amount raised in lamports
/// - `deadline`: Unix timestamp when campaign ends
/// - `claimed`: Boolean - has creator withdrawn the funds?
///
/// # Space Calculation (InitSpace)
/// Using `#[derive(InitSpace)]` lets Anchor calculate the required space:
/// - Pubkey: 32 bytes
/// - u64 (goal): 8 bytes
/// - u64 (raised): 8 bytes  
/// - i64 (deadline): 8 bytes
/// - bool (claimed): 1 byte
/// Total: ~57 bytes + 8 byte discriminator = 65 bytes
#[account]
#[derive(InitSpace)]
pub struct Campaign {
    /// The public key of the account that created this campaign
    /// This is who can withdraw funds if the campaign succeeds
    pub creator: Pubkey,

    /// The target amount to raise, in lamports
    /// If raised >= goal, the campaign is successful
    pub goal: u64,

    /// The current amount that has been raised, in lamports
    /// Updated every time someone contributes
    pub raised: u64,

    /// Unix timestamp (seconds since Jan 1, 1970)
    /// Campaign ends at this time - after which:
    /// - If goal met: creator can withdraw
    /// - If goal not met: donors can request refunds
    pub deadline: i64,

    /// Whether the campaign funds have been claimed
    /// Set to true when creator successfully withdraws
    /// Prevents double-withdrawal attack
    pub claimed: bool,
}
