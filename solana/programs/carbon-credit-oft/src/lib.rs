use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};

declare_id!("3HZeL7L5Ep7QAe4FLnti4NfinLxLF2xb73FfRCfhf1zk");

#[program]
pub mod carbon_credit_oft {
    use super::*;

    /// Initialize the Carbon Credit OFT
    pub fn initialize(
        ctx: Context<Initialize>,
        params: InitializeParams,
    ) -> Result<()> {
        let oft_config = &mut ctx.accounts.oft_config;
        
        oft_config.admin = ctx.accounts.admin.key();
        oft_config.mint = ctx.accounts.mint.key();
        oft_config.endpoint_program = params.endpoint_program;
        oft_config.decimals = params.decimals;
        oft_config.bump = ctx.bumps.oft_config;
        
        msg!("Carbon Credit OFT initialized");
        Ok(())
    }

    /// Mint carbon credits
    pub fn mint_credits(
        ctx: Context<MintCredits>,
        amount: u64,
        project_id: String,
        vintage: u16,
    ) -> Result<()> {
        // Mint tokens to the recipient
        let cpi_accounts = token::MintTo {
            mint: ctx.accounts.mint.to_account_info(),
            to: ctx.accounts.recipient_token_account.to_account_info(),
            authority: ctx.accounts.mint_authority.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        
        token::mint_to(cpi_ctx, amount)?;

        // Record the credit details
        let credit_record = &mut ctx.accounts.credit_record;
        credit_record.project_id = project_id;
        credit_record.vintage = vintage;
        credit_record.amount = amount;
        credit_record.owner = ctx.accounts.recipient.key();
        credit_record.minted_at = Clock::get()?.unix_timestamp;
        credit_record.status = CreditStatus::Active;
        credit_record.bump = ctx.bumps.credit_record;

        msg!("Minted {} carbon credits for project {}", amount, credit_record.project_id);
        Ok(())
    }

    /// Send carbon credits to another chain via LayerZero
    pub fn send_credits(
        ctx: Context<SendCredits>,
        params: SendCreditsParams,
    ) -> Result<()> {
        // Burn tokens from sender (OFT pattern)
        let cpi_accounts = token::Burn {
            mint: ctx.accounts.mint.to_account_info(),
            from: ctx.accounts.sender_token_account.to_account_info(),
            authority: ctx.accounts.sender.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        
        token::burn(cpi_ctx, params.amount)?;

        // Prepare cross-chain message
        let credit_transfer = CreditTransferData {
            recipient: params.recipient,
            amount: params.amount,
            project_id: params.project_id.clone(),
            vintage: params.vintage,
        };

        let message = credit_transfer.try_to_vec()?;

        // Store the outbound message for LayerZero processing
        let outbound_message = &mut ctx.accounts.outbound_message;
        outbound_message.dst_eid = params.destination_chain_id;
        outbound_message.receiver = params.receiver_address;
        outbound_message.message = message;
        outbound_message.sender = ctx.accounts.sender.key();
        outbound_message.nonce = Clock::get()?.unix_timestamp as u64;
        outbound_message.bump = ctx.bumps.outbound_message;

        msg!("Sent {} credits to chain {}", params.amount, params.destination_chain_id);
        Ok(())
    }

    /// Receive carbon credits from another chain via LayerZero
    pub fn lz_receive(
        ctx: Context<LzReceiveCredits>,
        params: LzReceiveParams,
    ) -> Result<()> {
        // Decode the credit transfer data
        let credit_transfer: CreditTransferData = 
            CreditTransferData::try_from_slice(&params.message)?;

        // Mint tokens to the recipient (OFT pattern)
        let seeds = &[
            b"oft_config",
            &[ctx.accounts.oft_config.bump],
        ];
        let signer = &[&seeds[..]];

        let cpi_accounts = token::MintTo {
            mint: ctx.accounts.mint.to_account_info(),
            to: ctx.accounts.recipient_token_account.to_account_info(),
            authority: ctx.accounts.oft_config.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
        
        token::mint_to(cpi_ctx, credit_transfer.amount)?;

        // Record the received credits
        let credit_record = &mut ctx.accounts.credit_record;
        credit_record.project_id = credit_transfer.project_id;
        credit_record.vintage = credit_transfer.vintage;
        credit_record.amount = credit_transfer.amount;
        credit_record.owner = credit_transfer.recipient;
        credit_record.minted_at = Clock::get()?.unix_timestamp;
        credit_record.status = CreditStatus::Active;
        credit_record.source_chain = Some(params.src_eid);
        credit_record.bump = ctx.bumps.credit_record;

        msg!("Received {} credits from chain {}", credit_transfer.amount, params.src_eid);
        Ok(())
    }

    /// Retire carbon credits (permanently remove from circulation)
    pub fn retire_credits(
        ctx: Context<RetireCredits>,
        amount: u64,
        retirement_reason: String,
    ) -> Result<()> {
        // Burn the tokens
        let cpi_accounts = token::Burn {
            mint: ctx.accounts.mint.to_account_info(),
            from: ctx.accounts.owner_token_account.to_account_info(),
            authority: ctx.accounts.owner.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        
        token::burn(cpi_ctx, amount)?;

        // Record the retirement
        let retirement = &mut ctx.accounts.retirement_record;
        retirement.owner = ctx.accounts.owner.key();
        retirement.amount = amount;
        retirement.reason = retirement_reason;
        retirement.retired_at = Clock::get()?.unix_timestamp;
        retirement.bump = ctx.bumps.retirement_record;

        msg!("Retired {} carbon credits", amount);
        Ok(())
    }

    /// Get credit information
    pub fn get_credit_info(
        ctx: Context<GetCreditInfo>,
    ) -> Result<CreditInfo> {
        let credit_record = &ctx.accounts.credit_record;
        
        let info = CreditInfo {
            project_id: credit_record.project_id.clone(),
            vintage: credit_record.vintage,
            amount: credit_record.amount,
            owner: credit_record.owner,
            minted_at: credit_record.minted_at,
            status: credit_record.status.clone(),
            source_chain: credit_record.source_chain,
        };

        Ok(info)
    }
}

// Account Contexts
#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
        init,
        payer = admin,
        space = 8 + OftConfig::INIT_SPACE,
        seeds = [b"oft_config"],
        bump
    )]
    pub oft_config: Account<'info, OftConfig>,

    pub mint: Account<'info, Mint>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(amount: u64, project_id: String)]
pub struct MintCredits<'info> {
    #[account(mut)]
    pub mint_authority: Signer<'info>,

    #[account(mut)]
    pub mint: Account<'info, Mint>,

    /// CHECK: Recipient of the credits
    pub recipient: AccountInfo<'info>,

    #[account(mut)]
    pub recipient_token_account: Account<'info, TokenAccount>,

    #[account(
        init,
        payer = mint_authority,
        space = 8 + CreditRecord::INIT_SPACE,
        seeds = [b"credit", project_id.as_bytes(), &Clock::get()?.unix_timestamp.to_le_bytes()],
        bump
    )]
    pub credit_record: Account<'info, CreditRecord>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct SendCredits<'info> {
    #[account(mut)]
    pub sender: Signer<'info>,

    #[account(
        seeds = [b"oft_config"],
        bump = oft_config.bump
    )]
    pub oft_config: Account<'info, OftConfig>,

    #[account(mut)]
    pub mint: Account<'info, Mint>,

    #[account(mut)]
    pub sender_token_account: Account<'info, TokenAccount>,

    #[account(
        init,
        payer = sender,
        space = 8 + OutboundMessage::INIT_SPACE,
        seeds = [b"outbound", sender.key().as_ref(), &Clock::get()?.unix_timestamp.to_le_bytes()],
        bump
    )]
    pub outbound_message: Account<'info, OutboundMessage>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct LzReceiveCredits<'info> {
    #[account(
        seeds = [b"oft_config"],
        bump = oft_config.bump
    )]
    pub oft_config: Account<'info, OftConfig>,

    #[account(mut)]
    pub mint: Account<'info, Mint>,

    #[account(mut)]
    pub recipient_token_account: Account<'info, TokenAccount>,

    #[account(
        init,
        payer = payer,
        space = 8 + CreditRecord::INIT_SPACE,
        seeds = [b"received_credit", &Clock::get()?.unix_timestamp.to_le_bytes()],
        bump
    )]
    pub credit_record: Account<'info, CreditRecord>,

    #[account(mut)]
    pub payer: Signer<'info>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct RetireCredits<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(mut)]
    pub mint: Account<'info, Mint>,

    #[account(mut)]
    pub owner_token_account: Account<'info, TokenAccount>,

    #[account(
        init,
        payer = owner,
        space = 8 + RetirementRecord::INIT_SPACE,
        seeds = [b"retirement", owner.key().as_ref(), &Clock::get()?.unix_timestamp.to_le_bytes()],
        bump
    )]
    pub retirement_record: Account<'info, RetirementRecord>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct GetCreditInfo<'info> {
    pub credit_record: Account<'info, CreditRecord>,
}

// Data Structures
#[account]
#[derive(InitSpace)]
pub struct OftConfig {
    pub admin: Pubkey,
    pub mint: Pubkey,
    pub endpoint_program: Pubkey,
    pub decimals: u8,
    pub bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct CreditRecord {
    #[max_len(64)]
    pub project_id: String,
    pub vintage: u16,
    pub amount: u64,
    pub owner: Pubkey,
    pub minted_at: i64,
    pub status: CreditStatus,
    pub source_chain: Option<u32>,
    pub bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct RetirementRecord {
    pub owner: Pubkey,
    pub amount: u64,
    #[max_len(128)]
    pub reason: String,
    pub retired_at: i64,
    pub bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct OutboundMessage {
    pub dst_eid: u32,
    #[max_len(32)]
    pub receiver: Vec<u8>,
    #[max_len(512)]
    pub message: Vec<u8>,
    pub sender: Pubkey,
    pub nonce: u64,
    pub bump: u8,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq, InitSpace)]
pub enum CreditStatus {
    Active,
    Retired,
    Transferred,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct CreditTransferData {
    pub recipient: Pubkey,
    pub amount: u64,
    pub project_id: String,
    pub vintage: u16,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct CreditInfo {
    pub project_id: String,
    pub vintage: u16,
    pub amount: u64,
    pub owner: Pubkey,
    pub minted_at: i64,
    pub status: CreditStatus,
    pub source_chain: Option<u32>,
}

// Parameters
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct InitializeParams {
    pub endpoint_program: Pubkey,
    pub decimals: u8,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct SendCreditsParams {
    pub destination_chain_id: u32,
    pub receiver_address: Vec<u8>,
    pub recipient: Pubkey,
    pub amount: u64,
    pub project_id: String,
    pub vintage: u16,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct LzReceiveParams {
    pub src_eid: u32,
    pub src_address: Vec<u8>,
    pub nonce: u64,
    pub guid: Vec<u8>,
    pub message: Vec<u8>,
    pub executor: Pubkey,
}

// Errors
#[error_code]
pub enum CarbonCreditError {
    #[msg("Unauthorized sender")]
    UnauthorizedSender,
    #[msg("Insufficient credits")]
    InsufficientCredits,
    #[msg("Invalid credit data")]
    InvalidCreditData,
    #[msg("Credits already retired")]
    CreditsAlreadyRetired,
}