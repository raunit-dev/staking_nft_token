use anchor_lang::{prelude::*, system_program};
use anchor_spl::token::{self, Mint, MintTo, Token, TokenAccount};
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

    pub stake_mint: Account<'info, Mint>,

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
    pub fn stake_sol(&mut self, stake_amount: u64, bumps: &StakeSolBumps) -> Result<()> {

        let from = &mut self.user;
        let to = &mut self.sol_vault;
        let system_program = self.system_program.to_account_info();

        let transfer_account = system_program::Transfer {
            from: from.to_account_info(),
            to: to.to_account_info(),
        };

        let cpi_context = CpiContext::new(system_program, transfer_account);

        system_program::transfer(cpi_context, stake_amount)?;

        let current_time = Clock::get()?.unix_timestamp;
        let staking_start_time = self.user_stake_account.staked_at;

        let staking_duration = current_time - staking_start_time;

        let rewards = staking_duration as u64 * stake_amount * self.stake_config.points_per_sol_stake as u64;

        let signer_seeds: &[&[&[u8]]] = &[
            &[
                b"config",
                &[self.stake_config.bump]
            ]
        ];

        let mint_account = MintTo {
            mint: self.reward_mint.to_account_info(),
            to: self.user_reward_ata.to_account_info(),
            authority: self.stake_config.to_account_info()
        };

        let mint_cpi_context = CpiContext::new_with_signer(self.token_program.to_account_info(), mint_account, signer_seeds);

        token::mint_to(mint_cpi_context, rewards)?;

        let stake_account = &mut self.user_stake_account;

        stake_account.set_inner(StakeAccount {
            owner: self.user.key(),
            mint: self.stake_mint.key(),
            staked_at: Clock::get()?.unix_timestamp,
            bump: bumps.user_stake_account,
            vault_bump: bumps.sol_vault,
        });

        let user_account = &mut self.user_account;

        user_account.set_inner(UserAccount {
            points: user_account.points.checked_add(rewards).unwrap(),
            nft_staked_amount: user_account.nft_staked_amount,
            spl_staked_amount: user_account.spl_staked_amount,
            sol_staked_amount: user_account.sol_staked_amount.checked_add(stake_amount).unwrap(),
            bump: bumps.user_account
        });

        Ok(())
    }
}