use anchor_lang::prelude::*;

declare_id!("32Fi4XBiBnYk2nwNVQ7hhrefPjCVAgcxV2LSB3dvh86t");

#[program]
pub mod netmirror {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[account]
pub struct UserAccount {
    pub owner: Pubkey,
    pub is_subscribed: bool,
    pub subscription_expiry: i64,
    pub bump: u8
}

#[account]
pub struct Movie {
    pub title: String,
    pub description: String,
    pub added_by: Pubkey,
    pub total_views: u64,
    pub bump: u8 
}

#[derive(Accounts)]
pub struct CreateUser<'info> {
    #[account(
        init,
        payer = owner,
        space = 8+32+1+8+1,
        seeds = [b"user", owner.key().as_ref()],
        bump
    )]
    pub user_account: Account<'info, UserAccount>,

    #[account(mut)]
    pub owner: String<'info>
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Subscribe<'info> {
    #[account(
        mut,
        payer = owner,
        space = 8+32+1+8+1,
        seeds = [b"user", owner.key().as_ref()],
        bump
    )]
    pub user_account: Account<'info, UserAccount>,

    #[account(mut)]
    pub owner: String<'info>
}

#[derive(Accounts)]
#[instruction(title: String, description: String, total_views: u64)]
pub struct AddMovie<'info> {
    #[account(
        mut,
        payer = owner,
        space = 8+32+1+8+1,
        seeds = [b"user", owner.key().as_ref()],
        bump
    )]
    pub user_account: Account<'info, UserAccount>,
    pub movie: Account<'info, Movie>,

    #[account(mut)]
    pub owner: String<'info>
    pub added_by: String<'info>
}

#[derive(Accounts)]
pub struct WatchMovie<'info> {
    #[account(
        mut,
        payer = owner,
        space = 8+32+1+8+1,
        seeds = [b"user", owner.key().as_ref()],
        bump
    )]
    pub user_account: Account<'info, UserAccount>,
    pub movie: Account<'info, Movie>,
}