use anchor_lang::prelude::*;

declare_id!("2bvT3M5bLbJgk8dcm3jsDkSEn8B2ntk2Eokmt2UKb7pU");

#[program]
pub mod mancer_submission {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
