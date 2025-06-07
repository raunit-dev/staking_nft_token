use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct StakeConfigAccount{
    pub points_per_nft_stake: u8,
    pub points_per_sol_stake: u8,
    pub points_per_spl_stake: u8,
    pub min_freeze_period: u32,
    pub reward_bump: u8,
    pub bump: u8
}