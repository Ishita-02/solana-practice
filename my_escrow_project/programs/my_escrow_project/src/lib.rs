use anchor_lang::prelude::*;

declare_id!("g6u4N2d2vwrihSzUGDFYWA12sNNiHFZF7eXy9i3VX5v");

#[program]
pub mod escrow {
    use super::*;

    pub fn initialize_escrow(ctx: Context<InitializeEscrow>, amount: u64, item_details: String) -> Result<()> {
        let escrow = &mut ctx.accounts.escrow;

        let ix = anchor_lang::system_program::Transfer {
            from: ctx.accounts.seller.to_account_info(),
            to: escrow.to_account_info(),
        };

        anchor_lang::system_program::transfer(
            CpiContext::new(ctx.accounts.system_program.to_account_info(), ix),
            amount,
        )?;

        escrow.amount = amount;
        escrow.item_details = item_details;
        escrow.is_active = true;
        escrow.seller = *ctx.accounts.seller.key;
        escrow.buyer = Pubkey::default();
        escrow.bump = ctx.bumps.escrow;
        Ok(())
    }

    pub fn accept_escrow(ctx: Context<AcceptEscrow>) -> Result<()> {
        let escrow = &mut ctx.accounts.escrow;
        require!(escrow.is_active, EscrowError::EscrowNotActive);
        require!(escrow.buyer == Pubkey::default(), EscrowError::EscrowAlreadyAccepted);
        
        escrow.buyer = *ctx.accounts.buyer.key;

        **ctx.accounts.buyer.to_account_info().try_borrow_mut_lamports()? += escrow.amount;
        **ctx.accounts.escrow.to_account_info().try_borrow_mut_lamports()? -= escrow.amount;

        Ok(())
    }

    pub fn cancel_escrow(ctx: Context<CancelEscrow>) -> Result<()> {
        let escrow = &mut ctx.accounts.escrow;
        require!(escrow.is_active, EscrowError::EscrowNotActive);
        require!(escrow.buyer == Pubkey::default(), EscrowError::EscrowAlreadyAccepted);
        
        escrow.is_active = false;

        **ctx.accounts.seller.to_account_info().try_borrow_mut_lamports()? += escrow.amount;
        **ctx.accounts.escrow.to_account_info().try_borrow_mut_lamports()? -= escrow.amount;
        Ok(())
    }
}

#[account]
pub struct Escrow {
    pub seller: Pubkey,      // 32
    pub buyer: Pubkey,       // 32
    pub amount: u64,         // 8
    pub item_details: String, // 4 + len
    pub is_active: bool,     // 1
    pub bump: u8             // 1
}

impl Escrow {
    pub const LEN: usize = 32 + 32 + 8 + (4 + 200) + 1 + 1; // 278 bytes (assuming max 200 chars for item_details)
}

#[derive(Accounts)]
#[instruction(amount: u64, item_details: String)]
pub struct InitializeEscrow<'info> {
    #[account(
        init,
        payer = seller,
        space = 8 + Escrow::LEN,
        seeds = [b"escrow", seller.key().as_ref()],
        bump
    )]
    pub escrow: Account<'info, Escrow>,

    #[account(mut)]
    pub seller: Signer<'info>,

    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
pub struct AcceptEscrow<'info> {
    #[account(
        mut,
        seeds = [b"escrow", escrow.seller.as_ref()],
        bump = escrow.bump
    )]
    pub escrow: Account<'info, Escrow>,

    pub buyer: Signer<'info>,
}

#[derive(Accounts)]
pub struct CancelEscrow<'info> {
    #[account(
        mut,
        seeds = [b"escrow", seller.key().as_ref()],
        bump = escrow.bump,
        has_one = seller
    )]
    pub escrow: Account<'info, Escrow>,

    pub seller: Signer<'info>,
}

#[error_code]
pub enum EscrowError {
    #[msg("Escrow is not active")]
    EscrowNotActive,
    #[msg("Escrow has already been accepted")]
    EscrowAlreadyAccepted,
}