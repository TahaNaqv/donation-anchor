use anchor_lang::prelude::*;
use anchor_lang::system_program::{transfer, Transfer};

declare_id!("FikJ34DGa5VMMckRxaPdkR3bWHGiyXiQBYZEGNZyWigS");

#[program]
pub mod donation_anchor {
    use super::*;

    pub fn init_jar(ctx: Context<InitJar>) -> Result<()> {
        let jar = &mut ctx.accounts.jar;
        jar.creator = ctx.accounts.creator.key();
        jar.total_raised = 0;
        jar.donation_count = 0;
        jar.last_donor = Pubkey::default();
        jar.bump = ctx.bumps.jar;
        msg!("Initialized Jar Created By: {:?}", ctx.accounts.creator.key());
        Ok(())
    }

    pub fn donate(ctx: Context<Donate>, amount: u64) -> Result<()> {
        require!(amount > 0, JarError::InvalidAmount);

        let cpi_ctx = CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            Transfer {
                from: ctx.accounts.donor.to_account_info(),
                to: ctx.accounts.jar.to_account_info(),
            },
        );
        transfer(cpi_ctx, amount)?;

        let jar = &mut ctx.accounts.jar;
        jar.total_raised = jar
            .total_raised
            .checked_add(amount)
            .ok_or(JarError::Overflow)?;
        jar.donation_count = jar
            .donation_count
            .checked_add(1)
            .ok_or(JarError::Overflow)?;
        jar.last_donor = ctx.accounts.donor.key();
        msg!(
            "Donation of {} lamports from {:?} to Jar {:?}",
            amount,
            ctx.accounts.donor.key(),
            jar.key()
        );
        Ok(())
    }

    pub fn withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
        require!(amount > 0, JarError::InvalidAmount);
        require!(amount <= ctx.accounts.jar.total_raised, JarError::InvalidAmount);

        let jar_info = &mut ctx.accounts.jar.to_account_info();
        let creator_info = &mut ctx.accounts.creator.to_account_info();

        let rent_minimum = Rent::get()?.minimum_balance(jar_info.data_len());
        let available_balance = jar_info.lamports().saturating_sub(rent_minimum);

        let jar = &mut ctx.accounts.jar;
        jar.total_raised = jar
            .total_raised
            .checked_sub(amount)
            .ok_or(JarError::Overflow)?;

        require!(amount <= available_balance, JarError::InvalidAmount);

        **jar_info.try_borrow_mut_lamports()? -= amount;
        **creator_info.try_borrow_mut_lamports()? += amount;
        
        msg!(
            "Withdrew {} lamports from Jar {:?} to Creator {:?}",
            amount,
            jar_info.key(),
            creator_info.key()
        );
        Ok(())
    }
}

#[derive(Accounts)]
pub struct InitJar<'info> {
    #[account(
        init,
        payer = creator,
        space = 8 + Jar::INIT_SPACE,
        seeds = [b"jar", creator.key().as_ref()],
        bump
    )]
    pub jar: Account<'info, Jar>,
    #[account(mut)]
    pub creator: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Donate<'info> {
    #[account(
        mut,
        seeds = [b"jar", jar.creator.as_ref()],
        bump = jar.bump
    )]
    pub jar: Account<'info, Jar>,
    #[account(mut)]
    pub donor: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(
        mut,
        seeds = [b"jar", jar.creator.as_ref()],
        bump = jar.bump,
        has_one = creator
    )]
    pub jar: Account<'info, Jar>,
    #[account(mut)]
    pub creator: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[account]
#[derive(InitSpace)]
pub struct Jar {
    pub creator: Pubkey,
    pub total_raised: u64,
    pub donation_count: u64,
    pub last_donor: Pubkey,
    pub bump: u8,
}

#[error_code]
pub enum JarError {
    #[msg("Invalid donation amount")]
    InvalidAmount,
    #[msg("Arithmetic overflow")]
    Overflow,
}