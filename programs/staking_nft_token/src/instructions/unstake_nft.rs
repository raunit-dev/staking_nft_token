use anchor_lang::prelude::*;
use anchor_spl::{metadata::{mpl_token_metadata::instructions::{ ThawDelegatedAccountCpi, ThawDelegatedAccountCpiAccounts}, MasterEditionAccount, Metadata, MetadataAccount}, token::{ revoke, Mint, Revoke, Token, TokenAccount}};

use crate::{error::ErrorCode, StakeAccount, StakeConfigAccount, UserAccount};

#[derive(Accounts)]
pub struct UnStakeNFT<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    pub mint: Account<'info, Mint>,

    pub collection_mint: Account<'info, Mint>,

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
        seeds = [
            b"metadata",
            metadata_program.key().as_ref(),
            mint.key().as_ref()
        ],
        bump,
        seeds::program = metadata_program.key(),
        constraint = metadata.collection.as_ref().unwrap().key.as_ref() == collection_mint.key().as_ref(),
        constraint = metadata.collection.as_ref().unwrap().verified == true

    )]
    pub metadata: Account<'info, MetadataAccount>,

    #[account(
        seeds = [
            b"metadata",
            metadata_program.key().as_ref(),
            mint.key().as_ref(),
            b"edition"
        ],
        bump,
        seeds::program = metadata_program.key()
    )]
    pub master_edition: Account<'info, MasterEditionAccount>,

    #[account(
        mut,
        close = user,
        has_one = mint,
        seeds = [b"stake", config.key().as_ref(), mint.key().as_ref(), stake_account.seed.to_le_bytes().as_ref()],
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
        seeds = [b"user", user.key().as_ref()],
        bump = user_account.bump
    )]
    pub user_account: Account<'info, UserAccount>,

    pub token_program: Program<'info, Token>,
    pub metadata_program: Program<'info, Metadata>,
    pub system_program: Program<'info, System>,
}

impl<'info> UnStakeNFT<'info> {
    pub fn unstake_nft(&mut self) -> Result<()> {

        let staked_at = self.stake_account.staked_at;
        let current = Clock::get()?.unix_timestamp;

        require!(current.checked_sub(staked_at).unwrap() >= self.config.min_freeze_period as i64, ErrorCode::FreezePeriodeNotPassed);

        let seeds = &[
            b"stake",
            self.config.to_account_info().key.as_ref(),
            self.mint.to_account_info().key.as_ref(),
            &self.stake_account.seed.to_le_bytes()[..],
            &[self.stake_account.bump],
        ];

        let signer_seeds = &[&seeds[..]];

        let delegate = &self.stake_account.to_account_info();
        let token_account = &self.mint_ata.to_account_info();
        let edition = &self.master_edition.to_account_info();
        let mint = &self.mint.to_account_info();
        let token_program = &self.token_program.to_account_info();
        let metadata_program = &self.metadata_program.to_account_info();

        ThawDelegatedAccountCpi::new(
            metadata_program,
            ThawDelegatedAccountCpiAccounts {
                delegate,
                token_account,
                edition,
                mint,
                token_program,
            },
        )
        .invoke_signed(signer_seeds)?;

       let cpi_program = self.token_program.to_account_info();

        let cpi_accounts = Revoke{
            source: self.mint_ata.to_account_info(),
            authority: self.user.to_account_info()
        };

        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

        revoke(cpi_ctx)?;

        self.user_account.nft_staked_amount = self.user_account.nft_staked_amount.checked_sub(1).ok_or(ErrorCode::Overflow)?;

        Ok(())
    }

}