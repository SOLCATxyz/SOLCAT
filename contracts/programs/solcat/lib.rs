use anchor_lang::prelude::*;

declare_id!("your_program_id_here");

#[program]
pub mod solcat {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        Ok(())
    }

    pub fn mark_suspicious_address(ctx: Context<MarkSuspiciousAddress>, address: Pubkey, reason: String) -> Result<()> {
        let suspicious_address = &mut ctx.accounts.suspicious_address;
        suspicious_address.address = address;
        suspicious_address.reporter = ctx.accounts.reporter.key();
        suspicious_address.reason = reason;
        suspicious_address.timestamp = Clock::get()?.unix_timestamp;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}

#[derive(Accounts)]
pub struct MarkSuspiciousAddress<'info> {
    #[account(mut)]
    pub suspicious_address: Account<'info, SuspiciousAddress>,
    #[account(mut)]
    pub reporter: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[account]
pub struct SuspiciousAddress {
    pub address: Pubkey,
    pub reporter: Pubkey,
    pub reason: String,
    pub timestamp: i64,
} 