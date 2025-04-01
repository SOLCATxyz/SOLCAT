use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub enum RiskType {
    Scam,
    Phishing,
    Malware,
    Ransomware,
    MoneyLaundering,
    MarketManipulation,
    Unknown,
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct RiskAssessment {
    pub base_score: u8,
    pub risk_types: Vec<RiskType>,
    pub confidence_score: u8,
    pub evidence_count: u32,
    pub last_update: i64,
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct RiskMetrics {
    pub transaction_volume: u64,
    pub unique_interactions: u32,
    pub age_of_account: i64,
    pub suspicious_patterns: Vec<String>,
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct AddressReport {
    pub reporter: Pubkey,
    pub reported_address: Pubkey,
    pub risk_score: u8,
    pub stake_amount: u64,
    pub timestamp: i64,
    pub description: String,
    pub vote_weight: u32,
    pub last_update_time: i64,
    pub time_lock_end: i64,
    pub risk_assessment: RiskAssessment,
    pub risk_metrics: RiskMetrics,
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct AddressStats {
    pub total_reports: u32,
    pub risk_scores: Vec<u8>,
    pub total_stake: u64,
    pub last_update: i64,
    pub weighted_risk_score: u32,
    pub total_vote_weight: u32,
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct ReporterStats {
    pub total_reports: u32,
    pub successful_reports: u32,
    pub total_stake: u64,
    pub reputation_score: u8,
    pub last_report_time: i64,
    pub reports_in_window: u32,
    pub cooldown_end_time: i64,
    pub token_balance: u64,
    pub rewards_claimed: u64,
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct GlobalConfig {
    pub min_stake_amount: u64,
    pub reward_rate: u64,
    pub token_mint: Pubkey,
    pub treasury: Pubkey,
    pub total_supply: u64,
    pub circulating_supply: u64,
    pub staking_enabled: bool,
    pub min_lock_duration: i64,
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct StakePool {
    pub total_staked: u64,
    pub reward_per_token: u64,
    pub last_update_time: i64,
    pub reward_rate: u64,
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct UserStake {
    pub owner: Pubkey,
    pub amount: u64,
    pub rewards_earned: u64,
    pub reward_per_token_paid: u64,
    pub lock_end_time: i64,
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct ReportHistory {
    pub address: Pubkey,
    pub reports: Vec<HistoricalReport>,
    pub is_blacklisted: bool,
    pub blacklist_reason: String,
    pub blacklist_timestamp: i64,
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct HistoricalReport {
    pub timestamp: i64,
    pub risk_score: u8,
    pub reporter: Pubkey,
    pub description: String,
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct BatchReport {
    pub reporter: Pubkey,
    pub addresses: Vec<Pubkey>,
    pub risk_scores: Vec<u8>,
    pub timestamp: i64,
    pub verification_status: VerificationStatus,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, PartialEq)]
pub enum VerificationStatus {
    Pending,
    Verified,
    Rejected,
} 