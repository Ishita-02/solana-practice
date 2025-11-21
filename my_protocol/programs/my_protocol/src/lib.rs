use anchor_lang::prelude::*;

declare_id!("2TMPwFz4XJrgU6gDoR7riprwzvnA9zx339czJaiMV187");

#[program]
pub mod my_protocol {
    use super::*;

    pub fn create_pool(ctx: Context<CreatePool>) -> Result<()> {
        
        Ok(())
    }
}

#[account]
pub struct Pool {
    pub owner: Pubkey,            // admin/owner
    pub token_a_mint: Pubkey,     // SPL mint
    pub token_b_mint: Pubkey,
    pub lp_mint: Pubkey,          // LP token mint
    pub token_a_vault: Pubkey,    // token account holding token A
    pub token_b_vault: Pubkey,
    pub fee_vault: Pubkey,        // fee token account (could be token A or B or LP)
    pub total_lp_supply: u64,     // track for bookkeeping
    pub fee_numerator: u64,       // e.g., 30 (for 0.3%)
    pub fee_denominator: u64,     // e.g., 10000
    pub bump: u8,
}
impl Pool {
    pub const LEN: usize = 32*7 + 8*3 + 1 + 16; // compute exact
}

#[account]
pub struct StakeAccount {
    pub user: Pubkey,
    pub pool: Pubkey,
    pub amount: u64,
    pub reward_debt: u128,
    pub bump: u8,
}


#[derive(Accounts)]
#[instruction(amount: u64, item_details: String)]
pub struct CreatePool {

}
