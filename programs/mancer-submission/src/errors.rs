//! Custom Error Codes for the Crowdfunding Program
//!
//! This file defines all custom error codes that the program can return.
//! Each error has a user-friendly message that will be displayed when the error occurs.

use anchor_lang::prelude::*;

/// Custom error codes for the crowdfunding program
///
/// These errors provide specific feedback when operations fail:
/// - Validation errors (deadline, amount, goal)
/// - State errors (already claimed, goal reached)
/// - Authorization errors (not creator)
#[error_code]
pub enum CampaignError {
    /// Error: Attempted to create campaign with deadline in the past
    ///
    /// Fix: Set deadline to a future timestamp
    #[msg("Deadline must be in the future")]
    DeadlineMustBeInFuture,

    /// Error: Attempted to withdraw but campaign didn't reach its goal
    ///
    /// Fix: Wait for more contributions or the campaign fails
    #[msg("Campaign goal not reached")]
    GoalNotReached,

    /// Error: Attempted to withdraw/claim before deadline passed
    ///
    /// Fix: Wait until deadline has passed
    #[msg("Campaign deadline has not passed yet")]
    DeadlineNotPassed,

    /// Error: Attempted to withdraw twice from the same campaign
    ///
    /// Fix: This is prevented by design - campaign can only be claimed once
    #[msg("Campaign funds already claimed")]
    AlreadyClaimed,

    /// Error: Attempted to refund when goal was reached (campaign succeeded)
    ///
    /// Fix: Use withdraw() to claim funds instead
    #[msg("Campaign goal reached - use withdraw instead")]
    GoalReached,

    /// Error: Amount is zero or negative
    ///
    /// Fix: Provide a positive amount
    #[msg("Invalid amount - must be greater than 0")]
    InvalidAmount,

    /// Error: Arithmetic overflow when adding to raised amount
    ///
    /// Fix: This shouldn't happen normally - campaign has reasonable limits
    #[msg("Arithmetic overflow occurred")]
    AmountOverflow,

    /// Error: Caller trying to refund has no contribution record
    ///
    /// Fix: Only donors who contributed can request refunds
    #[msg("No contribution found for this donor")]
    NotADonor,

    /// Error: Refund amount exceeds donor's remaining contribution
    ///
    /// Fix: Specify an amount <= your remaining contribution
    #[msg("Refund amount exceeds your contribution")]
    InsufficientContribution,

    /// Error: Contribution has already been fully refunded
    ///
    /// Fix: You have no remaining contribution to refund
    #[msg("No remaining contribution to refund")]
    NoContributionToRefund,
}
