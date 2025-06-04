use anchor_lang::prelude::*;
use anchor_spl::token::{Mint,Token, TokenAccount};

use crate::state::user_account::UserAccount;

#[derive(Accounts)]
pub struct InitUser<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        init,
        payer = user,
        seeds = [b"user", user.key().as_ref()],
        bump,
        space = UserAccount::INIT_SPACE,
    )]
    pub user_account: Account<'info, UserAccount>,

    pub system_program: Program<'info, System>,
}

impl<'info> InitUser<'info> {
    pub fn init_user(&mut self, bump: &InitUserBumps) -> Result<()> {
        self.user_account.set_inner(UserAccount { 
            points: 0, 
            nft_staked_amount: 0, 
            spl_staked_amount: 0, 
            sol_staked_amount: 0, 
            bump:  bump.user_account
        });
        Ok(())
    }
}