use anchor_lang::prelude::*;

declare_id!("9dSFULzzXU51YNMDeiYt9Xk6bJYZFCanSqbdrubV2ZMc");

#[program]
pub mod event_ticketing_system {
    use super::*;

    pub fn create_event(ctx: Context<CreateEvent>, title: String) -> Result<()> {
        let event = &mut ctx.accounts.event;

        event.title = title;
        event.date = Clock::get()?.unix_timestamp;
        event.total_tickets = 0;
        event.sold_count = 0;
        event.creator = ctx.accounts.creator.key();
        event.bump = ctx.bumps.event;
        Ok(())
    }

    pub fn buy_ticket(ctx: Context<BuyTicket>, seat_no: u32) -> Result<()> {
        let ticket = &mut ctx.accounts.ticket;
        let event = &mut ctx.accounts.event;

        ticket.owner = ticket.owner.key();
        ticket.event = event.key();
        ticket.seat_no = seat_no;
        ticket.is_used = false;
        ticket.bump = ctx.bumps.ticket;

        event.total_tickets += 1;
        event.sold_count += 1;
        Ok(())
    }

    pub fn transfer_ticket(ctx: Context<TransferTicket>, new_owner: Pubkey) -> Result<()> {
        let ticket = &mut ctx.accounts.ticket;

        ticket.owner = new_owner;
        Ok(())
    }

    pub fn verify_ticket(ctx: Context<VerifyTicket>) -> Result<()> {
        let ticket = &mut ctx.accounts.ticket;
        ticket.is_used = true;

        Ok(())
    }

    pub fn cancel_event(ctx: Context<CancelEvent>) -> Result<()> {
        Ok(())
    }
}

#[account]
#[derive(InitSpace)]
pub struct Event {
    #[max_len(50)]
    pub title: String,
    pub date: i64,
    pub total_tickets: u64,
    pub sold_count: u64,
    pub creator: Pubkey,
    pub bump: u8
}

#[account]
#[derive(InitSpace)]
pub struct Ticket {
    pub owner: Pubkey,
    pub event: Pubkey,
    pub seat_no: u32,
    pub is_used: bool,
    pub bump: u8
}

#[derive(Accounts)]
#[instruction(title: String)]
pub struct CreateEvent<'info> {
    #[account(
        init,
        payer = creator,
        space = 8 + Event::INIT_SPACE,
        seeds = [b"event", title.as_bytes(), creator.key().as_ref()],
        bump
    )]
    pub event: Account<'info, Event>,

    #[account(mut)]
    pub creator: Signer<'info>,
    pub system_program: Program<'info, System>
}

// Ticket → (owner, event, seat_no, QR_hash, bump) 

#[derive(Accounts)]
#[instruction(seat_no: i64)]
pub struct BuyTicket<'info> {
    #[account(
        init,
        payer = owner,
        space = 8 + Ticket::INIT_SPACE,
        seeds = [b"ticket", owner.key().as_ref(), event.key().as_ref(), &seat_no.to_le_bytes()],
        bump
    )]

    pub ticket: Account<'info, Ticket>,
    pub event: Account<'info, Event>,

    #[account(mut)]
    pub owner: Signer<'info>,
    pub system_program: Program<'info, System>
}

// transfer_ticket

#[derive(Accounts)]
pub struct TransferTicket<'info> {
    #[account(
        mut, 
        seeds = [b"ticket", owner.key().as_ref(), event.key().as_ref(), &ticket.seat_no.to_le_bytes()],
        bump = ticket.bump
    )]

    pub ticket: Account<'info, Ticket>,
    pub event: Account<'info, Event>,

    #[account(mut)]
    pub owner: Signer<'info>,
    pub new_owner: Signer<'info>
}

#[derive(Accounts)]
pub struct VerifyTicket<'info> {
    #[account(
        mut, 
        seeds = [b"ticket", owner.key().as_ref(), event.key().as_ref(), &ticket.seat_no.to_le_bytes()],
        bump = ticket.bump
    )]

    pub ticket: Account<'info, Ticket>,
    pub event: Account<'info, Event>,

    #[account(mut)]
    pub owner: Signer<'info>,
}

#[derive(Accounts)]
#[instruction(title: String)]
pub struct CancelEvent<'info> {
    #[account(
        mut,
        seeds = [b"event", title.as_bytes(), creator.key().as_ref()],
        bump = event.bump,
        close = creator
    )]
    pub event: Account<'info, Event>,
    pub creator: Signer<'info>
}