use solana_program::program_error::ProgramError;
use thiserror::Error;

#[derive(Error, Debug, Copy, Clone)]
pub enum SolcatError {
    #[error("Invalid address format")]
    InvalidAddress,
    
    #[error("Invalid risk score")]
    InvalidRiskScore,
    
    #[error("Insufficient stake")]
    InsufficientStake,
    
    #[error("Report already exists")]
    ReportAlreadyExists,
    
    #[error("Not authorized")]
    NotAuthorized,
    
    #[error("Invalid report data")]
    InvalidReportData,

    #[error("Insufficient reputation")]
    InsufficientReputation,

    #[error("Time lock active")]
    TimeLockActive,

    #[error("Invalid vote weight")]
    InvalidVoteWeight,

    #[error("Report limit exceeded")]
    ReportLimitExceeded,

    #[error("Cooldown period active")]
    CooldownActive,

    #[error("Insufficient token balance")]
    InsufficientTokenBalance,

    #[error("Invalid token mint")]
    InvalidTokenMint,

    #[error("Staking disabled")]
    StakingDisabled,

    #[error("Invalid stake amount")]
    InvalidStakeAmount,

    #[error("Stake locked")]
    StakeLocked,

    #[error("Invalid reward calculation")]
    InvalidRewardCalculation,

    #[error("Treasury account mismatch")]
    TreasuryMismatch,

    #[error("Invalid batch report")]
    InvalidBatchReport,

    #[error("Batch verification pending")]
    BatchVerificationPending,

    #[error("Address already blacklisted")]
    AddressAlreadyBlacklisted,

    #[error("Invalid blacklist operation")]
    InvalidBlacklistOperation,

    #[error("History update failed")]
    HistoryUpdateFailed,
}

impl From<SolcatError> for ProgramError {
    fn from(e: SolcatError) -> Self {
        ProgramError::Custom(e as u32)
    }
} 