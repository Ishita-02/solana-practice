use anchor_lang::{prelude::*, solana_program::example_mocks::solana_sdk::system_program};

declare_id!("32Fi4XBiBnYk2nwNVQ7hhrefPjCVAgcxV2LSB3dvh86t");

#[program]
pub mod netmirror {
    use anchor_lang::system_program;

    use super::*;

    pub fn create_account(ctx: Context<CreateUser>) -> Result<()> {
        let account = &mut ctx.accounts.user_account ;
        account.owner = ctx.accounts.owner.key();
        account.is_subscribed = false;
        account.subscription_expiry = 0;
        account.bump = ctx.bumps.user_account;
        Ok(())
    }

    pub fn subscribe(ctx: Context<Subscribe>, duration: i64) -> Result<()> {
        let userAccount = &mut ctx.accounts.user_account;
        userAccount.is_subscribed = true;
        let current_time = Clock::get()?.unix_timestamp;
        userAccount.subscription_expiry = current_time + duration;

        let cost = ctx.accounts.platform_state.subscription_price;
        let cpi_contxt = CpiContext::new(ctx.accounts.system_program.to_account_info(), system_program::Transfer{
            from: ctx.accounts.owner.to_account_info(),
            to: ctx.accounts.treasury.to_account_info()
        });
        system_program::transfer(cpi_contxt, cost)?;
        Ok(())
    }

    pub fn add_movie(ctx: Context<AddMovie>, title: String, description: String, video_url: String, genre: String) -> Result<()> {
        let movie = &mut ctx.accounts.movie;
        movie.title = title;
        movie.description = description;
        movie.added_by = ctx.accounts.admin.key();
        movie.total_views = 0;
        movie.genre = genre;
        movie.video_url = video_url;
        movie.rating_count = 0;
        movie.rating_sum = 0;
        movie.bump = ctx.bumps.movie;
        Ok(())
    }

    pub fn watch_movie(ctx: Context<WatchMovie>) -> Result<()> {
        let movie = &mut ctx.accounts.movie;
        let user = &mut ctx.accounts.user_account;
        if (user.is_subscribed == false) {
            return err!(NetMirrorError::UserNotSubscribed);
        }
        movie.total_views += 1;
        Ok(())
    }

    pub fn update_movie(ctx: Context<UpdateMovie>, title: String) -> Result<()> {
        let movie = &mut ctx.accounts.movie;
        movie.title = title;
        Ok(())
    }

    pub fn delete_movie(ctx: Context<DeleteMovie>) -> Result<()> {
        Ok(())
    }

    pub fn rate_movie(ctx: Context<RateMovie>, rating_count: u64) -> Result<()> {
        let movie = &mut ctx.accounts.movie;
        let review = &mut ctx.accounts.review;
        movie.rating_count == 1;
        movie.rating_sum += review.rating;
        review.reviewer = ctx.accounts.user_account.key();
        Ok(())
    }
}

#[error_code]
pub enum NetMirrorError {
    #[msg("User is not subscribed")]
    UserNotSubscribed,
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
    pub video_url: String,
    pub genre: String,
    pub added_by: Pubkey,
    pub total_views: u64,
    pub rating_sum: u64,
    pub rating_count: u64,
    pub bump: u8 
}

#[account]
pub struct PlatformState {
    pub admin: Pubkey,
    pub treasury: Pubkey,
    pub subscription_price: u64,
    pub bump: u8
}

#[account]
pub struct Review {
    pub reviewer: Pubkey,
    pub movie: Pubkey,
    pub rating: u64,
    pub bump: u8
}

#[derive(Accounts)]
pub struct InitializePlatform<'info> {
    #[account(
        init,
        payer = admin,
        space = 8+32+32+8+1,
        seeds = [b"state"],
        bump
    )]
    pub platform_state: Account<'info, PlatformState>,

    #[account(mut)]
    pub admin: Signer<'info>,
    pub treasury: AccountInfo<'info>,
    pub system_program: Program<'info,System>,
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
    pub owner: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(duration: i64)]
pub struct Subscribe<'info> {
    #[account(
        mut,
        seeds = [b"user", owner.key().as_ref()],
        bump
    )]
    pub user_account: Account<'info, UserAccount>,
    pub platform_state: Account<'info, PlatformState>,

    #[account(mut)]
    pub owner: Signer<'info>,
    pub treasury: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(title: String, description: String, video_url: String, genre: String)]
pub struct AddMovie<'info> {
    #[account(
        init,
        payer = admin,
        space = 8 + 4 + 100 + 4 + 200 + 32 + 8 + 1,
        seeds = [b"movie", admin.key().as_ref(), title.as_bytes()],
        bump
    )]
    pub movie: Account<'info, Movie>,
    #[account(mut)]
    pub admin: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(title: String)]
pub struct WatchMovie<'info> {
    #[account(
        mut,
        seeds = [b"user", owner.key().as_ref()],
        bump = user_account.bump,
        has_one = owner
    )]
    pub user_account: Account<'info, UserAccount>,

    #[account(
        mut,
        seeds = [b"movie", movie.added_by.as_ref(), title.as_bytes()],
        bump = movie.bump
    )]
    pub movie: Account<'info, Movie>,

    pub owner: Signer<'info>,
}

#[derive(Accounts)]
#[instruction(title: String)]
pub struct UpdateMovie<'info> {
    #[account(
        mut, 
        seeds = [b"movie", added_by.key().as_ref(), title.as_bytes()],
        bump = movie.bump,
        has_one = added_by
    )]
    pub movie: Account<'info, Movie>,
    pub added_by: Signer<'info>
}

#[derive(Accounts)]
#[instruction(title: String)]
pub struct DeleteMovie<'info> {
    #[account(
        mut, 
        seeds = [b"movie", added_by.key().as_ref(), title.as_bytes()],
        bump = movie.bump,
        close = added_by
    )]
    pub movie: Account<'info, Movie>,
    pub added_by: Signer<'info>,
}

#[derive(Accounts)]
pub struct RateMovie<'info> {
    #[account(mut)]
    pub movie: Account<'info, Movie>,

    #[account(
        seeds = [b"user", owner.key().as_ref()],
        bump
    )]

    pub user_account: Account<'info, UserAccount>,

    #[account(
        init, 
        payer = owner,
        space = 8 + 32 + 32 + 1 + 1,
        seeds = [b"review", movie.key().as_ref(), owner.key().as_ref()],
        bump
    )]
    pub review: Account<'info, Review>,

    #[account(mut)]
    pub owner: Signer<'info>,
    pub system_program: Program<'info, System>
}