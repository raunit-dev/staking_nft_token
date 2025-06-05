use anchor_lang::{prelude::*, system_program};
use anchor_spl::token::spl_token::native_mint;
use anchor_spl::token::{self, Mint, MintTo, Token, TokenAccount};
use anchor_spl::associated_token::{AssociatedToken};

use crate::state::StakeConfigAccount;
use crate::state::UserAccount;
use crate::state::StakeAccount;

use crate::error::ErrorCode;

#[derive(Accounts)]
pub struct StakeSol<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        seeds = [b"rewards", stake_config.key().as_ref()],
        bump = stake_config.reward_bump,
        mint::authority = stake_config,
        mint::decimals = 6,
    )]
    pub reward_mint: Account<'info, Mint>,

    #[account(
        mut,
        seeds = [b"config"],
        bump = stake_config.bump
    )]
    pub stake_config: Account<'info, StakeConfigAccount>,

    #[account(
        init,
        payer = user,
        space = 8 + StakeAccount::INIT_SPACE,
        seeds = [b"stake_account", user.key().as_ref(), stake_config.key().as_ref()],
        bump
    )]
    pub stake_account: Account<'info, StakeAccount>,

     #[account(
        mut,
        seeds = [b"user", user.key().as_ref()],
        bump,
    )]
    pub user_account: Account<'info, UserAccount>,

    #[account(
        mut,
        associated_token::mint = reward_mint,
        associated_token::authority = user
    )]
    pub user_reward_ata: Account<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [b"vault", user_account.key().as_ref()],
        bump,
    )]
    pub sol_vault: SystemAccount<'info>,

    pub system_program: Program<'info, System>,

    pub token_program: Program<'info, Token>

}

impl<'info> StakeSol<'info> {
    pub fn stake_sol(&mut self, stake_amount: u64, bumps: &StakeSolBumps) -> Result<()> {
        let transfer_ctx = CpiContext::new(
            self.system_program.to_account_info(),
            system_program::Transfer {
                from: self.user.to_account_info(),
                to: self.sol_vault.to_account_info(),
            },
        );
        system_program::transfer(transfer_ctx, stake_amount)?;

        let current_time = Clock::get()?.unix_timestamp;

        self.stake_account.set_inner(StakeAccount {
            owner: self.user.key(),
            mint: native_mint::id(),
            staked_at: current_time,
            bump: bumps.stake_account,
            vault_bump: bumps.sol_vault,
        });

        self.user_account.set_inner(UserAccount {
            points: self.user_account.points,
            nft_staked_amount: self.user_account.nft_staked_amount,
            spl_staked_amount: self.user_account.spl_staked_amount,
            sol_staked_amount: self
                .user_account
                .sol_staked_amount
                .checked_add(stake_amount)
                .ok_or_else(|| error!(ErrorCode::Overflow))?,
            bump: bumps.user_account,
        });

        Ok(())
    }
}