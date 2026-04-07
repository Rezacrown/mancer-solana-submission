//! Contribution Account Data Structure
//!
//! Tracks each donor's contribution to a specific campaign.
//! This is CRITICAL for security - it prevents non-donors from claiming refunds.
//!
//! # PDA Seeds
//! `["contribution", campaign.key(), donor.key()]`
//!
//! # Why This Exists
//! - Prevents anyone from draining the vault via refund
//! - Ensures donors can only refund what they contributed
//! - Enables partial refunds (donor can claim back incrementally)

use anchor_lang::prelude::*;

/// Stores a single donor's contribution to a campaign
///
/// # Security Model
/// - Created on first contribution (contribute instruction)
/// - Updated on subsequent contributions by same donor
/// - Decremented when donor claims refund
/// - Prevents refund abuse by non-contributors
///
/// # Example Flow
/// 1. Alice contributes 50 SOL → Contribution PDA created with amount=50
/// 2. Alice contributes 30 SOL more → Contribution PDA updated to amount=80
/// 3. Campaign fails → Alice can refund up to 80 SOL (or less if partial)
/// 4. Alice refunds 40 SOL → Contribution PDA updated to amount=40
/// 5. Alice refunds remaining 40 SOL → Contribution PDA updated to amount=0
#[account]
#[derive(InitSpace)]
pub struct Contribution {
    /// The campaign this contribution is for
    /// Used in PDA derivation: ensures one contribution per campaign per donor
    pub campaign: Pubkey, // 32 bytes

    /// The donor who made this contribution
    /// Used in PDA derivation: ensures only donor can access this record
    pub donor: Pubkey, // 32 bytes

    /// Amount currently contributed (in lamports)
    /// DECREASES when donor claims partial refund
    /// When this reaches 0, donor has fully refunded
    pub amount: u64, // 8 bytes
}
// Total: 72 bytes + 8 byte discriminator = 80 bytes
