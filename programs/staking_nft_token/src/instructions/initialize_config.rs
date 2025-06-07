use anchor_lang::prelude::*;
use anchor_spl::token::{Mint,Token};

use crate::state::stake_config_account::StakeConfigAccount;

#[derive(Accounts)]

pub struct InitializeConfig<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(
        init,
        payer = admin,
        space = 8 + StakeConfigAccount::INIT_SPACE,
        seeds = [b"config"],
        bump
    )]
    pub config: Account<'info,StakeConfigAccount>,

    #[account(
        init,
        payer = admin,
        seeds = [b"rewards", config.key().as_ref()],
        bump,
        mint::decimals = 6,
        mint::authority = config
    )]
    pub reward_mint: Account<'info,Mint>,

    pub token_program: Program<'info,Token>,
    pub system_program: Program<'info,System>
}

impl<'info> InitializeConfig<'info> {
    pub fn initialize_config(&mut self,points_per_nft_stake: u8,points_per_spl_stake: u8,points_per_sol_stake: u8,min_freeze_period: u32,bumps: &InitializeConfigBumps) -> Result<()> {
        self.config.set_inner(StakeConfigAccount{
            points_per_nft_stake,
            points_per_sol_stake,
            points_per_spl_stake,
            min_freeze_period,
            reward_bump: bumps.reward_mint,
            bump: bumps.config,  
        });
        Ok(())
    }
}

