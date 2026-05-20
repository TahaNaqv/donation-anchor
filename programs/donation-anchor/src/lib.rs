use anchor_lang::prelude::*;

declare_id!("FikJ34DGa5VMMckRxaPdkR3bWHGiyXiQBYZEGNZyWigS");

#[program]
pub mod donation_anchor {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
