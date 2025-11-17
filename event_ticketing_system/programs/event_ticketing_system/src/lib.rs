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

    pub fn buy_ticket(ctx: Context<BuyTicket>, title: String, seat_no: u32) -> Result<()> {
        let ticket = &mut ctx.accounts.ticket;
        let event = &mut ctx.accounts.event;

        ticket.owner = ctx.accounts.buyer.key();
        ticket.event = event.key();
        ticket.seat_no = seat_no;
        ticket.is_used = false;
        ticket.bump = ctx.bumps.ticket;

        event.total_tickets = event
            .total_tickets
            .checked_add(1)
            .ok_or(ProgramError::Custom(0))?;
        event.sold_count = event
            .sold_count
            .checked_add(1)
            .ok_or(ProgramError::Custom(0))?;

        Ok(())
    }

    pub fn transfer_ticket(
        ctx: Context<TransferTicket>,
        _title: String,
    ) -> Result<()> {
        let ticket = &mut ctx.accounts.ticket;

        require_keys_eq!(ticket.owner, ctx.accounts.current_owner.key(), EventErrors::NotOwner);

        ticket.owner = ctx.accounts.new_owner.key();
        Ok(())
    }

    pub fn verify_ticket(
        ctx: Context<VerifyTicket>,
        _title: String, 
        _seat_no: u32,
    ) -> Result<()> {
        let ticket = &mut ctx.accounts.ticket;
        let event = &ctx.accounts.event;

        require!(!ticket.is_used, EventErrors::TicketAlreadyUsed);
        require_keys_eq!(ticket.event, event.key(), EventErrors::InvalidEvent);

        ticket.is_used = true;

        Ok(())
    }

    pub fn cancel_event(ctx: Context<CancelEvent>, title: String) -> Result<()> {
        msg!("Event {} closed", title);
        Ok(())
    }
}

#[error_code]
pub enum EventErrors {
    #[msg("You are not the owner of the ticket.")]
    NotOwner,
    #[msg("Ticket does not exist.")]
    TicketDoesNotExist,
    #[msg("Ticket is already used.")]
    TicketAlreadyUsed,
    #[msg("The event is invalid")]
    InvalidEvent,
}

#[account]
pub struct Event {
    pub title: String,
    pub date: i64,
    pub total_tickets: u64,
    pub sold_count: u64,
    pub creator: Pubkey,
    pub bump: u8,
}

impl Event {
    pub const INIT_SPACE: usize = 4 + 50 + 8 + 8 + 8 + 32 + 1;
}

#[account]
pub struct Ticket {
    pub owner: Pubkey,
    pub event: Pubkey,
    pub seat_no: u32,
    pub is_used: bool,
    pub bump: u8,
}

impl Ticket {
    pub const INIT_SPACE: usize = 32 + 32 + 4 + 1 + 1;
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
    pub system_program: Program<'info, System>,
}


#[derive(Accounts)]
#[instruction(title: String, seat_no: u32)]
pub struct BuyTicket<'info> {
    #[account(
        mut,
        seeds = [b"event", title.as_bytes(), event.creator.as_ref()],
        bump = event.bump,
    )]
    pub event: Account<'info, Event>,

    #[account(
        init,
        payer = buyer,
        space = 8 + Ticket::INIT_SPACE,
        seeds = [b"ticket", event.key().as_ref(), &seat_no.to_le_bytes()],
        bump
    )]
    pub ticket: Account<'info, Ticket>,

    #[account(mut)]
    pub buyer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(title: String, seat_no: u32)]
pub struct TransferTicket<'info> {
    #[account(
        mut,
        seeds = [b"ticket", event.key().as_ref(), &seat_no.to_le_bytes()],
        bump = ticket.bump
    )]
    pub ticket: Account<'info, Ticket>,

    #[account(
        seeds = [b"event", title.as_bytes(), event.creator.as_ref()],
        bump = event.bump
    )]
    pub event: Account<'info, Event>,

    pub current_owner: Signer<'info>,

    pub new_owner: Signer<'info>,
}

//
//
#[derive(Accounts)]
#[instruction(title: String, seat_no: u32)]
pub struct VerifyTicket<'info> {
    #[account(
        mut,
        seeds = [b"ticket", event.key().as_ref(), &seat_no.to_le_bytes()],
        bump = ticket.bump
    )]
    pub ticket: Account<'info, Ticket>,

    #[account(
        seeds = [b"event", title.as_bytes(), event.creator.as_ref()],
        bump = event.bump
    )]
    pub event: Account<'info, Event>,

    pub event_creator: Signer<'info>,
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

    pub creator: Signer<'info>,
}
