#![allow(unexpected_cfgs)]
pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("jgvGJazuaFWvM2185R6C19e1DKqer3kMZz3gJBP1eA9");

#[program]
pub mod staking_nft_token {
    use super::*;

    pub fn initialize_config(ctx: Context<InitializeConfig>,points_per_nft_stake: u8,
        points_per_spl_stake: u8,
        points_per_sol_stake: u8,
        min_freeze_period: u32) -> Result<()> {
        ctx.accounts.initialize_config(points_per_nft_stake, points_per_spl_stake, points_per_sol_stake, min_freeze_period, &ctx.bumps)
    }

    pub fn initialize_user(ctx: Context<InitializeUser>) -> Result<()> {
        ctx.accounts.initialize_user(&ctx.bumps)
    }

    pub fn stake_nft(ctx: Context<StakeNFT>, seed: u64) -> Result<()> {
        ctx.accounts.stake_nft(seed, &ctx.bumps)
    }

    pub fn stake_sol(ctx: Context<StakeSol>,seed: u64,amount: u64) -> Result<()> {
        ctx.accounts.stake_sol(seed,amount, &ctx.bumps)
    }

    pub fn stake_spl(ctx: Context<StakeSpl>,seed: u64,amount: u64) -> Result<()> {
        ctx.accounts.stake_spl(seed,amount, &ctx.bumps)
    }

    pub fn unstake_nft(ctx: Context<UnStakeNFT>) -> Result<()> {
        ctx.accounts.unstake_nft()
    }

    pub fn unstake_sol(ctx: Context<UnStakeSoL>) -> Result<()> {
        ctx.accounts.unstake_sol()
    }

    pub fn unstake_spl(ctx: Context<UnStakeSpL>) -> Result<()> {
        ctx.accounts.unstake_spl()
    }

}