use anchor_lang::prelude::*;
use anchor_spl::token::{transfer_checked, close_account, Mint, Token, TokenAccount, TransferChecked, CloseAccount};

use crate::state::{StakeConfigAccount, UserAccount, StakeAccount};
use crate::error::ErrorCode;

#[derive(Accounts)]
pub struct UnStakeSpL<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    pub mint: Account<'info, Mint>,

    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = user
    )]
    pub mint_ata: Account<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [b"rewards", config.key().as_ref()],
        bump = config.reward_bump,
        mint::authority = config,
    )]
    pub reward_mint: Account<'info, Mint>,

    #[account(
        mut,
        associated_token::mint = reward_mint,
        associated_token::authority = user,
    )]
    pub user_reward_ata: Account<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [b"stake", config.key().as_ref(), user.key().as_ref(), mint.key().as_ref()],
        bump = stake_account.bump,
    )]
    pub stake_account: Account<'info, StakeAccount>,
    
    #[account(
        seeds = [b"config"],
        bump = config.bump,
    )]
    pub config: Account<'info, StakeConfigAccount>,

    #[account(
        mut,
        close = user,
        associated_token::mint = mint,
        associated_token::authority = stake_account,
    )]
    pub vault_ata: Account<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [b"user", user.key().as_ref()],
        bump = user_account.bump
    )]
    pub user_account: Account<'info, UserAccount>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
} 

impl<'info> UnStakeSpL<'info> {
    pub fn unstake_spl(&mut self) -> Result<()> {
        let staked_at = self.stake_account.staked_at;
        let current = Clock::get()?.unix_timestamp;
        require!(current.checked_sub(staked_at).unwrap() >= self.config.min_freeze_period as i64, ErrorCode::FreezePeriodeNotPassed);

        let seeds = &[
            b"stake",
            self.config.to_account_info().key.as_ref(),
            self.user.to_account_info().key.as_ref(),
            self.mint.to_account_info().key.as_ref(),
            &[self.stake_account.bump],
        ];
        let signer_seeds = &[&seeds[..]];

        let cpi_program = self.token_program.to_account_info();
        let cpi_accounts = TransferChecked {
            from: self.vault_ata.to_account_info(),
            mint: self.mint.to_account_info(),
            to: self.mint_ata.to_account_info(),
            authority: self.stake_account.to_account_info(),
        };
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);
        transfer_checked(cpi_ctx, self.vault_ata.amount, self.mint.decimals)?;

        self.user_account.spl_staked_amount = self.user_account.spl_staked_amount.checked_add(self.vault_ata.amount).ok_or(ErrorCode::Overflow)?;

        let close_accounts = CloseAccount {
            account: self.vault_ata.to_account_info(),
            destination: self.user.to_account_info(),
            authority: self.stake_account.to_account_info(),
        };
        let close_cpi_ctx = CpiContext::new_with_signer(self.token_program.to_account_info(), close_accounts, signer_seeds);
        close_account(close_cpi_ctx)?;
        Ok(())
    }
}
