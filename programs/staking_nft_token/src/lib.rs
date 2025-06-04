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

    pub fn init_config(ctx: Context<InitializeConfig>, points_per_nft_stake: u8, points_per_spl_stake: u8, points_per_sol_stake: u8, min_freeze_period: u32) -> Result<()> {
        ctx.accounts.init_config(points_per_nft_stake, points_per_spl_stake, points_per_sol_stake, min_freeze_period, &ctx.bumps)
    }

    pub fn init_user(ctx: Context<InitUser>) -> Result<()> {
        ctx.accounts.init_user(&ctx.bumps)
    }

    
}
