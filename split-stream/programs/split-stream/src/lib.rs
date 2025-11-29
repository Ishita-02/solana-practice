use anchor_lang::prelude::*;

declare_id!("HHn9AjC1DzYJamkpuu7rLAbqrT6fScA5cCLKq65brT9J");

#[program]
pub mod split_stream {
    use super::*;

    pub fn initialize_split(ctx: Context<InitializeSplit>, nft_mint: Pubkey, recipients_data: Vec<RecipientInput>) -> Result<()> {
        let mut total: u8 = 0;
        for recipient_input in &recipients_data {
            total += recipient_input.percentage;
        }

        require!(total == 100, EventErrors::SumExceed100Error);

        let recipients: Vec<Recipient> = recipients_data.iter().map(|input | Recipient {
            wallet: input.wallet,
            percentage: input.percentage,
            claimed: 0
        }).collect();

        let royalty_split = &mut ctx.accounts.royalty_split;
        royalty_split.nft_mint = nft_mint;
        royalty_split.creator = ctx.accounts.creator.key();
        royalty_split.recipients = recipients;
        royalty_split.total_collected = 0;
        royalty_split.is_active = true;
        royalty_split.bump = ctx.bumps.royalty_split;

        Ok(())
    }

    pub fn deposit_royalty(ctx: Context<DepositRoyalty>, amount: u64) -> Result<()> {

        let cpi_context = CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            anchor_lang::system_program::Transfer {
                from: ctx.accounts.depositor.to_account_info(),
                to: ctx.accounts.royalty_split.to_account_info()
            },
        );

        anchor_lang::system_program::transfer(cpi_context, amount)?;

        let royalty = &mut ctx.accounts.royalty_split;
        royalty.total_collected += amount;

        emit!(DepositEvent {
            nft_mint: royalty.nft_mint,
            amount,
            depositor: ctx.accounts.depositor.key()
        });

        Ok(())
    }

    pub fn claim_share(ctx: Context<ClaimShare>, recipient_index: u64) -> Result<()> {
        let royalty_split = &mut ctx.accounts.royalty_split;

        require!(
            (recipient_index as usize) < royalty_split.recipients.len(),
            EventErrors::InvalidRecipientIndex
        ); 

        let recipient = &royalty_split.recipients[recipient_index as usize];

        require!(recipient.wallet == ctx.accounts.recipient.key(), EventErrors::UnauthorizedRecipient);

        let total_share = (royalty_split.total_collected as u128)
            .checked_mul(recipient.percentage as u128)
            .unwrap()
            .checked_div(100)
            .unwrap() as u64;

        let claimable = total_share.checked_sub(recipient.claimed).unwrap();
        require!(claimable > 0, EventErrors::NothingToClaim);

        **royalty_split.to_account_info().try_borrow_mut_lamports()? -= claimable;
        **ctx.accounts.recipient.to_account_info().try_borrow_mut_lamports()? += claimable;
        
        royalty_split.recipients[recipient_index as usize].claimed += claimable;

        Ok(())
    }

    pub fn close_split(ctx: Context<CloseSplit>) -> Result<()> {
        let royalty_split = &ctx.accounts.royalty_split;

        require!(royalty_split.creator == ctx.accounts.creator.key(),
            EventErrors::UnauthorizedClose
        );
        
        let mut total_claimed: u64 = 0;
        for recipient in &royalty_split.recipients {
            total_claimed += recipient.claimed;
        }
        
        require!(
            total_claimed == royalty_split.total_collected,
            EventErrors::UnclaimedFundsRemaining
        );
        Ok(())
    }
}

#[event]
pub struct DepositEvent {
    pub nft_mint: Pubkey,
    pub amount: u64,
    pub depositor: Pubkey,
}

#[error_code]
pub enum EventErrors {
    #[msg("The sum of percentages exceed 100.")]
    SumExceed100Error,

    #[msg("Invalid recipient index")]
    InvalidRecipientIndex,

    #[msg("Recipient is unauthorized")]
    UnauthorizedRecipient,

    #[msg("Nothing to claim, the amount is 0")]
    NothingToClaim,

    #[msg("Only creator can close the split.")]
    UnauthorizedClose,
    
    #[msg("Cannot close with unclaimed funds remaining.")]
    UnclaimedFundsRemaining,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct RecipientInput {
    pub wallet: Pubkey,
    pub percentage: u8,
}

#[account]
pub struct RoyaltySplit {
    pub nft_mint: Pubkey,
    pub creator: Pubkey,
    pub recipients: Vec<Recipient>,
    pub total_collected: u64,
    pub is_active: bool,
    pub bump: u8
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct Recipient {
    pub wallet: Pubkey,
    pub percentage: u8,
    pub claimed: u64
}

#[derive(Accounts)]
#[instruction(nft_mint: Pubkey, recipients_data: Vec<RecipientInput>)]
pub struct InitializeSplit<'info> {
    #[account(
        init,
        payer = creator,
        space = 8 + 32 + 32 + 4 + 8 + 1 + 1 + (recipients_data.len() * (32 + 1 + 8)),
        seeds = [b"royalty_split" , nft_mint.as_ref()],
        bump
    )]

    pub royalty_split: Account <'info, RoyaltySplit>,

    #[account(mut)]
    pub creator: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(amount: u64)]
pub struct DepositRoyalty<'info> {
    #[account(mut)]
    pub royalty_split: Account<'info, RoyaltySplit>,

    #[account(mut)]
    pub depositor: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(recipient_index: u8)]
pub struct ClaimShare<'info> {
    #[account(
        mut,
        seeds = [b"royalty_split" , royalty_split.nft_mint.as_ref()],
        bump = royalty_split.bump
    )]

    pub royalty_split: Account<'info, RoyaltySplit>,

    #[account(mut)]
    pub recipient: Signer<'info>,
}

#[derive(Accounts)]
pub struct CloseSplit<'info> {
    #[account(
        mut,
        close = creator,
        seeds = [b"royalty_split", royalty_split.nft_mint.as_ref()],
        bump = royalty_split.bump,
        has_one = creator
    )]

    pub royalty_split: Account<'info, RoyaltySplit>,

    #[account(mut)]
    pub creator: Signer<'info>,
}
