use anchor_lang::prelude::*;
use anchor_spl::token::{Mint,Token, TokenAccount};
use anchor_spl::associated_token::{AssociatedToken};

use crate::state::StakeConfigAccount;
use crate::state::UserAccount;
use crate::state::StakeAccount;

#[derive(Accounts)]
pub struct StakeSol<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        seeds = [b"rewards", stake_config.key().as_ref()],
        bump = stake_config.reward_bump,
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
    pub user_stake_account: Account<'info, StakeAccount>,

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
    pub fn stake_sol(&mut self, stake_amount: u64) -> Result<()> {

        let from = &mut self.user;
        let to = &mut self.sol_vault;
        let system_program = self.system_program.to_account_info();

        Ok(())
    }
}