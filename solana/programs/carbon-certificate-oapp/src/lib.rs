use anchor_lang::prelude::*;

declare_id!("6DTmAjesQy49eeZPR3iQ4ZKPHzUU9uAsRzCuD9uGbHfw");

#[program]
pub mod carbon_certificate_oapp {
    use super::*;

    /// Initialize the Carbon Certificate OApp
    pub fn initialize(
        ctx: Context<Initialize>,
        params: InitializeParams,
    ) -> Result<()> {
        let oapp_config = &mut ctx.accounts.oapp_config;
        
        oapp_config.admin = ctx.accounts.admin.key();
        oapp_config.endpoint_program = params.endpoint_program;
        oapp_config.bump = ctx.bumps.oapp_config;
        
        msg!("Carbon Certificate OApp initialized");
        Ok(())
    }

    /// Issue a new carbon certificate
    pub fn issue_certificate(
        ctx: Context<IssueCertificate>,
        params: IssueCertificateParams,
    ) -> Result<()> {
        let certificate = &mut ctx.accounts.certificate;
        let clock = Clock::get()?;
        
        certificate.id = params.certificate_id;
        certificate.issuer = ctx.accounts.issuer.key();
        certificate.total_emissions = params.total_emissions;
        certificate.data_hash = params.data_hash;
        certificate.issue_timestamp = clock.unix_timestamp;
        certificate.valid_until = params.valid_until;
        certificate.status = CertificateStatus::Active;
        certificate.bump = ctx.bumps.certificate;
        
        msg!("Certificate issued: {}", params.certificate_id);
        Ok(())
    }

    /// Send certificate verification to another chain via LayerZero
    pub fn send_certificate_verification(
        ctx: Context<SendCertificateVerification>,
        params: SendVerificationParams,
    ) -> Result<()> {
        let certificate = &ctx.accounts.certificate;
        
        // Verify certificate is valid
        require!(
            certificate.status == CertificateStatus::Active,
            CarbonCertificateError::CertificateInactive
        );
        
        let clock = Clock::get()?;
        require!(
            clock.unix_timestamp <= certificate.valid_until,
            CarbonCertificateError::CertificateExpired
        );

        // Prepare cross-chain message payload
        let verification_data = CertificateVerificationData {
            certificate_id: certificate.id.clone(),
            issuer: certificate.issuer,
            total_emissions: certificate.total_emissions,
            data_hash: certificate.data_hash.clone(),
            issue_timestamp: certificate.issue_timestamp,
            valid_until: certificate.valid_until,
        };

        // Serialize the message
        let message = verification_data.try_to_vec()?;

        // Store the outbound message for LayerZero processing
        let outbound_message = &mut ctx.accounts.outbound_message;
        outbound_message.dst_eid = params.destination_chain_id;
        outbound_message.receiver = params.receiver_address;
        outbound_message.message = message;
        outbound_message.sender = ctx.accounts.sender.key();
        outbound_message.nonce = clock.unix_timestamp as u64;
        outbound_message.bump = ctx.bumps.outbound_message;

        msg!("Certificate verification prepared for chain {}", params.destination_chain_id);
        Ok(())
    }

    /// Receive certificate verification from another chain via LayerZero
    pub fn lz_receive(
        ctx: Context<LzReceive>,
        params: LzReceiveParams,
    ) -> Result<()> {
        // Decode the verification data
        let verification_data: CertificateVerificationData = 
            CertificateVerificationData::try_from_slice(&params.message)?;

        // Store the verification
        let verification = &mut ctx.accounts.verification;
        verification.certificate_id = verification_data.certificate_id;
        verification.source_chain = params.src_eid;
        verification.issuer = verification_data.issuer;
        verification.total_emissions = verification_data.total_emissions;
        verification.data_hash = verification_data.data_hash;
        verification.verified_at = Clock::get()?.unix_timestamp;
        verification.bump = ctx.bumps.verification;

        msg!("Certificate verification received from chain {}", params.src_eid);
        Ok(())
    }

    /// Verify a certificate exists and is valid
    pub fn verify_certificate(
        ctx: Context<VerifyCertificate>,
        certificate_id: String,
    ) -> Result<CertificateVerificationResult> {
        let certificate = &ctx.accounts.certificate;
        let clock = Clock::get()?;

        let is_valid = certificate.status == CertificateStatus::Active &&
                      clock.unix_timestamp <= certificate.valid_until;

        let result = CertificateVerificationResult {
            exists: true,
            is_valid,
            total_emissions: certificate.total_emissions,
            issue_timestamp: certificate.issue_timestamp,
            valid_until: certificate.valid_until,
        };

        msg!("Certificate {} verification result: valid={}", certificate_id, is_valid);
        Ok(result)
    }

    /// Revoke a certificate
    pub fn revoke_certificate(
        ctx: Context<RevokeCertificate>,
        certificate_id: String,
    ) -> Result<()> {
        let certificate = &mut ctx.accounts.certificate;
        
        require!(
            certificate.issuer == ctx.accounts.issuer.key(),
            CarbonCertificateError::UnauthorizedSender
        );

        certificate.status = CertificateStatus::Revoked;
        
        msg!("Certificate {} revoked", certificate_id);
        Ok(())
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
        space = 8 + OAppConfig::INIT_SPACE,
        seeds = [b"oapp_config"],
        bump
    )]
    pub oapp_config: Account<'info, OAppConfig>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(params: IssueCertificateParams)]
pub struct IssueCertificate<'info> {
    #[account(mut)]
    pub issuer: Signer<'info>,

    #[account(
        init,
        payer = issuer,
        space = 8 + CarbonCertificate::INIT_SPACE,
        seeds = [b"certificate", params.certificate_id.as_bytes()],
        bump
    )]
    pub certificate: Account<'info, CarbonCertificate>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct SendCertificateVerification<'info> {
    #[account(mut)]
    pub sender: Signer<'info>,

    #[account(
        seeds = [b"oapp_config"],
        bump = oapp_config.bump
    )]
    pub oapp_config: Account<'info, OAppConfig>,

    #[account(
        seeds = [b"certificate", certificate.id.as_bytes()],
        bump = certificate.bump
    )]
    pub certificate: Account<'info, CarbonCertificate>,

    #[account(
        init,
        payer = sender,
        space = 8 + OutboundMessage::INIT_SPACE,
        seeds = [b"outbound", sender.key().as_ref(), &Clock::get()?.unix_timestamp.to_le_bytes()],
        bump
    )]
    pub outbound_message: Account<'info, OutboundMessage>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct LzReceive<'info> {
    #[account(
        seeds = [b"oapp_config"],
        bump = oapp_config.bump
    )]
    pub oapp_config: Account<'info, OAppConfig>,

    #[account(
        init,
        payer = payer,
        space = 8 + CertificateVerification::INIT_SPACE,
        seeds = [b"verification", &Clock::get()?.unix_timestamp.to_le_bytes()],
        bump
    )]
    pub verification: Account<'info, CertificateVerification>,

    #[account(mut)]
    pub payer: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(certificate_id: String)]
pub struct VerifyCertificate<'info> {
    #[account(
        seeds = [b"certificate", certificate_id.as_bytes()],
        bump = certificate.bump
    )]
    pub certificate: Account<'info, CarbonCertificate>,
}

#[derive(Accounts)]
#[instruction(certificate_id: String)]
pub struct RevokeCertificate<'info> {
    #[account(mut)]
    pub issuer: Signer<'info>,

    #[account(
        mut,
        seeds = [b"certificate", certificate_id.as_bytes()],
        bump = certificate.bump
    )]
    pub certificate: Account<'info, CarbonCertificate>,
}

// Data Structures
#[account]
#[derive(InitSpace)]
pub struct OAppConfig {
    pub admin: Pubkey,
    pub endpoint_program: Pubkey,
    pub bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct CarbonCertificate {
    #[max_len(64)]
    pub id: String,
    pub issuer: Pubkey,
    pub total_emissions: u64, // in kg CO2e
    #[max_len(64)]
    pub data_hash: String,
    pub issue_timestamp: i64,
    pub valid_until: i64,
    pub status: CertificateStatus,
    pub bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct CertificateVerification {
    #[max_len(64)]
    pub certificate_id: String,
    pub source_chain: u32,
    pub issuer: Pubkey,
    pub total_emissions: u64,
    #[max_len(64)]
    pub data_hash: String,
    pub verified_at: i64,
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
pub enum CertificateStatus {
    Active,
    Revoked,
    Expired,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct CertificateVerificationData {
    pub certificate_id: String,
    pub issuer: Pubkey,
    pub total_emissions: u64,
    pub data_hash: String,
    pub issue_timestamp: i64,
    pub valid_until: i64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct CertificateVerificationResult {
    pub exists: bool,
    pub is_valid: bool,
    pub total_emissions: u64,
    pub issue_timestamp: i64,
    pub valid_until: i64,
}

// Parameters
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct InitializeParams {
    pub endpoint_program: Pubkey,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct IssueCertificateParams {
    pub certificate_id: String,
    pub total_emissions: u64,
    pub data_hash: String,
    pub valid_until: i64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct SendVerificationParams {
    pub destination_chain_id: u32,
    pub receiver_address: Vec<u8>,
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
pub enum CarbonCertificateError {
    #[msg("Certificate is not active")]
    CertificateInactive,
    #[msg("Certificate has expired")]
    CertificateExpired,
    #[msg("Unauthorized sender")]
    UnauthorizedSender,
    #[msg("Invalid certificate data")]
    InvalidCertificateData,
}