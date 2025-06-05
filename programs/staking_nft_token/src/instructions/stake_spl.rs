use anchor_lang::prelude;
use anchor_spl::{associated_token::associated_token, token::{mint_to, transfer_checked, Mint, MintTo, Token, TokenAccount, TransferChecked}};

use crate::state::StakeConfigAccount;
use crate::state::UserAccount;
use crate::state::StakeAccount;

#[derive(Accounts)]
pub struct StakeSPL <'info> {

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
        bump = config.rewards_bump,
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
        init,
        payer = user,
        seeds = [b"stake", config.key().as_ref(), user.key().as_ref(), mint.key().as_ref()], // seed so that user can stake multiple ammounts
        bump,
        space = 8 + StakeAccount::INIT_SPACE
    )]
    pub stake_account: Account<'info, StakeAccount>,
    
    #[account(
        seeds = [b"config"],
        bump = config.bump,
    )]
    pub config: Account<'info, StateConfig>,

    #[account(
        init,
        payer = user,
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
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
} 
