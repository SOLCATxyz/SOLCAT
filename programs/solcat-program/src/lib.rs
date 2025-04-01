use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
    clock::Clock,
    sysvar::Sysvar,
};

mod error;
mod state;

use error::SolcatError;
use state::{
    AddressReport, AddressStats, ReporterStats, GlobalConfig, UserStake, StakePool,
    ReportHistory, BatchReport, VerificationStatus, HistoricalReport, RiskAssessment, RiskMetrics
};

// Constants for anti-Sybil mechanisms
const MIN_REPUTATION_SCORE: u8 = 10;
const REPORT_COOLDOWN_PERIOD: i64 = 3600; // 1 hour in seconds
const MAX_REPORTS_PER_WINDOW: u32 = 5;
const REPORT_WINDOW_DURATION: i64 = 86400; // 24 hours in seconds
const TIME_LOCK_DURATION: i64 = 604800; // 7 days in seconds

// Constants for token economics
const REWARD_MULTIPLIER: u64 = 100; // Base reward multiplier
const MIN_STAKE_DURATION: i64 = 604800; // 7 days in seconds
const MAX_STAKE_DURATION: i64 = 31536000; // 365 days in seconds
const STAKE_DURATION_MULTIPLIER: u64 = 10; // Multiplier for longer stake duration

// Constants for batch reporting
const MAX_BATCH_SIZE: usize = 10;
const MIN_VERIFICATION_STAKE: u64 = 100_000_000; // 0.1 SOL

// Constants for risk assessment
const RISK_WEIGHT_TRANSACTION_VOLUME: f32 = 0.3;
const RISK_WEIGHT_INTERACTIONS: f32 = 0.2;
const RISK_WEIGHT_ACCOUNT_AGE: f32 = 0.1;
const RISK_WEIGHT_PATTERNS: f32 = 0.4;

entrypoint!(process_instruction);

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let instruction = instruction_data[0];

    match instruction {
        0 => report_address(program_id, accounts_iter, &instruction_data[1..]),
        1 => update_report(program_id, accounts_iter, &instruction_data[1..]),
        2 => stake_on_report(program_id, accounts_iter, &instruction_data[1..]),
        3 => update_reporter_stats(program_id, accounts_iter, &instruction_data[1..]),
        4 => stake_tokens(program_id, accounts_iter, &instruction_data[1..]),
        5 => unstake_tokens(program_id, accounts_iter, &instruction_data[1..]),
        6 => claim_rewards(program_id, accounts_iter, &instruction_data[1..]),
        7 => distribute_rewards(program_id, accounts_iter, &instruction_data[1..]),
        8 => submit_batch_report(program_id, accounts_iter, &instruction_data[1..]),
        9 => verify_batch_report(program_id, accounts_iter, &instruction_data[1..]),
        10 => blacklist_address(program_id, accounts_iter, &instruction_data[1..]),
        11 => update_history(program_id, accounts_iter, &instruction_data[1..]),
        _ => Err(ProgramError::InvalidInstructionData),
    }
}

fn report_address(
    program_id: &Pubkey,
    accounts_iter: &mut std::slice::Iter<AccountInfo>,
    data: &[u8],
) -> ProgramResult {
    let reporter_info = next_account_info(accounts_iter)?;
    let reported_address_info = next_account_info(accounts_iter)?;
    let report_account_info = next_account_info(accounts_iter)?;
    let stats_account_info = next_account_info(accounts_iter)?;
    let reporter_stats_info = next_account_info(accounts_iter)?;
    let clock_sysvar_info = next_account_info(accounts_iter)?;

    // Verify reporter is signer
    if !reporter_info.is_signer {
        return Err(SolcatError::NotAuthorized.into());
    }

    // Check reporter stats and anti-Sybil conditions
    let mut reporter_stats = ReporterStats::try_from_slice(&reporter_stats_info.data.borrow())?;
    let clock = Clock::from_account_info(clock_sysvar_info)?;
    
    // Check reputation score
    if reporter_stats.reputation_score < MIN_REPUTATION_SCORE {
        return Err(SolcatError::InsufficientReputation.into());
    }

    // Check cooldown period
    if clock.unix_timestamp < reporter_stats.cooldown_end_time {
        return Err(SolcatError::CooldownActive.into());
    }

    // Check report limit in time window
    if clock.unix_timestamp - reporter_stats.last_report_time < REPORT_WINDOW_DURATION {
        if reporter_stats.reports_in_window >= MAX_REPORTS_PER_WINDOW {
            return Err(SolcatError::ReportLimitExceeded.into());
        }
    } else {
        // Reset window if it has expired
        reporter_stats.reports_in_window = 0;
    }

    // Parse report data
    let risk_score = data[0];
    if risk_score > 100 {
        return Err(SolcatError::InvalidRiskScore.into());
    }

    let description_len = data[1] as usize;
    let description = String::from_utf8_lossy(&data[2..2+description_len]).to_string();
    
    // Parse risk assessment data
    let mut offset = 2 + description_len;
    let num_risk_types = data[offset] as usize;
    offset += 1;
    
    let mut risk_types = Vec::with_capacity(num_risk_types);
    for _ in 0..num_risk_types {
        let risk_type_index = data[offset];
        offset += 1;
        risk_types.push(match risk_type_index {
            0 => RiskType::Scam,
            1 => RiskType::Phishing,
            2 => RiskType::Malware,
            3 => RiskType::Ransomware,
            4 => RiskType::MoneyLaundering,
            5 => RiskType::MarketManipulation,
            _ => RiskType::Unknown,
        });
    }

    let confidence_score = data[offset];
    offset += 1;

    let evidence_count = u32::from_le_bytes(data[offset..offset+4].try_into().unwrap());
    offset += 4;

    // Parse risk metrics
    let transaction_volume = u64::from_le_bytes(data[offset..offset+8].try_into().unwrap());
    offset += 8;

    let unique_interactions = u32::from_le_bytes(data[offset..offset+4].try_into().unwrap());
    offset += 4;

    let age_of_account = i64::from_le_bytes(data[offset..offset+8].try_into().unwrap());
    offset += 8;

    let num_patterns = data[offset] as usize;
    offset += 1;
    
    let mut suspicious_patterns = Vec::with_capacity(num_patterns);
    for _ in 0..num_patterns {
        let pattern_len = data[offset] as usize;
        offset += 1;
        suspicious_patterns.push(
            String::from_utf8_lossy(&data[offset..offset+pattern_len]).to_string()
        );
        offset += pattern_len;
    }

    let clock = Clock::from_account_info(clock_sysvar_info)?;

    let risk_assessment = RiskAssessment {
        base_score: risk_score,
        risk_types,
        confidence_score,
        evidence_count,
        last_update: clock.unix_timestamp,
    };

    let risk_metrics = RiskMetrics {
        transaction_volume,
        unique_interactions,
        age_of_account,
        suspicious_patterns,
    };

    // Calculate final risk score
    let final_risk_score = calculate_risk_score(&risk_assessment, &risk_metrics);
    
    let report = AddressReport {
        reporter: *reporter_info.key,
        reported_address: *reported_address_info.key,
        risk_score: final_risk_score,
        stake_amount: 0,
        timestamp: clock.unix_timestamp,
        description,
        vote_weight: calculate_vote_weight(&reporter_stats),
        last_update_time: clock.unix_timestamp,
        time_lock_end: clock.unix_timestamp + TIME_LOCK_DURATION,
        risk_assessment,
        risk_metrics,
    };

    // Save report
    report.serialize(&mut *report_account_info.data.borrow_mut())?;

    // Update stats
    let mut stats = if let Ok(s) = AddressStats::try_from_slice(&stats_account_info.data.borrow()) {
        s
    } else {
        AddressStats {
            total_reports: 0,
            risk_scores: Vec::new(),
            total_stake: 0,
            last_update: 0,
            weighted_risk_score: 0,
            total_vote_weight: 0,
        }
    };

    stats.total_reports += 1;
    stats.risk_scores.push(risk_score);
    stats.last_update = clock.unix_timestamp;
    stats.weighted_risk_score += (risk_score as u32) * report.vote_weight;
    stats.total_vote_weight += report.vote_weight;

    stats.serialize(&mut *stats_account_info.data.borrow_mut())?;

    // Update reporter stats
    reporter_stats.total_reports += 1;
    reporter_stats.last_report_time = clock.unix_timestamp;
    reporter_stats.reports_in_window += 1;
    reporter_stats.cooldown_end_time = clock.unix_timestamp + REPORT_COOLDOWN_PERIOD;

    reporter_stats.serialize(&mut *reporter_stats_info.data.borrow_mut())?;

    msg!("Address reported successfully");
    Ok(())
}

fn update_report(
    program_id: &Pubkey,
    accounts_iter: &mut std::slice::Iter<AccountInfo>,
    data: &[u8],
) -> ProgramResult {
    let reporter_info = next_account_info(accounts_iter)?;
    let report_account_info = next_account_info(accounts_iter)?;
    let stats_account_info = next_account_info(accounts_iter)?;
    let clock_sysvar_info = next_account_info(accounts_iter)?;

    // Verify reporter is signer
    if !reporter_info.is_signer {
        return Err(SolcatError::NotAuthorized.into());
    }

    // Load existing report
    let mut report = AddressReport::try_from_slice(&report_account_info.data.borrow())?;

    // Verify reporter owns the report
    if report.reporter != *reporter_info.key {
        return Err(SolcatError::NotAuthorized.into());
    }

    let clock = Clock::from_account_info(clock_sysvar_info)?;

    // Check time lock
    if clock.unix_timestamp < report.time_lock_end {
        return Err(SolcatError::TimeLockActive.into());
    }

    // Parse update data
    let risk_score = data[0];
    if risk_score > 100 {
        return Err(SolcatError::InvalidRiskScore.into());
    }

    let description_len = data[1] as usize;
    let description = String::from_utf8_lossy(&data[2..2+description_len]).to_string();

    // Update report
    report.risk_score = risk_score;
    report.description = description;
    report.last_update_time = clock.unix_timestamp;
    report.time_lock_end = clock.unix_timestamp + TIME_LOCK_DURATION;

    // Save updated report
    report.serialize(&mut *report_account_info.data.borrow_mut())?;

    // Update stats
    let mut stats = AddressStats::try_from_slice(&stats_account_info.data.borrow())?;
    stats.weighted_risk_score = stats.weighted_risk_score
        .saturating_sub((report.risk_score as u32) * report.vote_weight)
        .saturating_add((risk_score as u32) * report.vote_weight);
    stats.last_update = clock.unix_timestamp;

    stats.serialize(&mut *stats_account_info.data.borrow_mut())?;

    msg!("Report updated successfully");
    Ok(())
}

fn stake_on_report(
    program_id: &Pubkey,
    accounts_iter: &mut std::slice::Iter<AccountInfo>,
    data: &[u8],
) -> ProgramResult {
    let staker_info = next_account_info(accounts_iter)?;
    let report_account_info = next_account_info(accounts_iter)?;
    let stats_account_info = next_account_info(accounts_iter)?;
    let reporter_stats_info = next_account_info(accounts_iter)?;
    let system_program_info = next_account_info(accounts_iter)?;
    let config_info = next_account_info(accounts_iter)?;

    // Verify staker is signer
    if !staker_info.is_signer {
        return Err(SolcatError::NotAuthorized.into());
    }

    // Load global config
    let config = GlobalConfig::try_from_slice(&config_info.data.borrow())?;

    // Parse stake amount
    let stake_amount = u64::from_le_bytes(data[0..8].try_into().unwrap());
    if stake_amount < config.min_stake_amount {
        return Err(SolcatError::InsufficientStake.into());
    }

    // Load existing report and reporter stats
    let mut report = AddressReport::try_from_slice(&report_account_info.data.borrow())?;
    let mut reporter_stats = ReporterStats::try_from_slice(&reporter_stats_info.data.borrow())?;

    // Transfer SOL from staker to program account
    let ix = solana_program::system_instruction::transfer(
        staker_info.key,
        report_account_info.key,
        stake_amount,
    );

    solana_program::program::invoke(
        &ix,
        &[
            staker_info.clone(),
            report_account_info.clone(),
            system_program_info.clone(),
        ],
    )?;

    // Update report stake amount
    report.stake_amount += stake_amount;
    report.vote_weight = calculate_vote_weight(&reporter_stats);

    // Save updated report
    report.serialize(&mut *report_account_info.data.borrow_mut())?;

    // Update stats
    let mut stats = AddressStats::try_from_slice(&stats_account_info.data.borrow())?;
    stats.total_stake += stake_amount;
    stats.weighted_risk_score = stats.weighted_risk_score
        .saturating_sub((report.risk_score as u32) * report.vote_weight)
        .saturating_add((report.risk_score as u32) * calculate_vote_weight(&reporter_stats));
    stats.serialize(&mut *stats_account_info.data.borrow_mut())?;

    // Update reporter stats
    reporter_stats.total_stake += stake_amount;
    reporter_stats.serialize(&mut *reporter_stats_info.data.borrow_mut())?;

    msg!("Stake added successfully");
    Ok(())
}

fn update_reporter_stats(
    program_id: &Pubkey,
    accounts_iter: &mut std::slice::Iter<AccountInfo>,
    data: &[u8],
) -> ProgramResult {
    let authority_info = next_account_info(accounts_iter)?;
    let reporter_stats_info = next_account_info(accounts_iter)?;

    // Verify authority is signer and is program upgrade authority
    if !authority_info.is_signer || authority_info.key != program_id {
        return Err(SolcatError::NotAuthorized.into());
    }

    let mut reporter_stats = ReporterStats::try_from_slice(&reporter_stats_info.data.borrow())?;
    
    // Update reputation score based on successful reports
    let new_reputation_score = data[0];
    if new_reputation_score > 100 {
        return Err(SolcatError::InvalidReportData.into());
    }

    reporter_stats.reputation_score = new_reputation_score;
    reporter_stats.successful_reports += 1;

    reporter_stats.serialize(&mut *reporter_stats_info.data.borrow_mut())?;

    msg!("Reporter stats updated successfully");
    Ok(())
}

// Helper function to calculate vote weight based on reporter stats
fn calculate_vote_weight(reporter_stats: &ReporterStats) -> u32 {
    let base_weight = 100u32;
    let reputation_multiplier = (reporter_stats.reputation_score as u32).max(1);
    let stake_multiplier = ((reporter_stats.total_stake / 1_000_000_000) as u32).max(1); // Convert lamports to SOL
    let success_rate = if reporter_stats.total_reports > 0 {
        (reporter_stats.successful_reports as f32 / reporter_stats.total_reports as f32 * 100.0) as u32
    } else {
        0
    };

    base_weight
        .saturating_mul(reputation_multiplier)
        .saturating_mul(stake_multiplier)
        .saturating_mul(success_rate.max(1))
}

fn stake_tokens(
    program_id: &Pubkey,
    accounts_iter: &mut std::slice::Iter<AccountInfo>,
    data: &[u8],
) -> ProgramResult {
    let staker_info = next_account_info(accounts_iter)?;
    let stake_pool_info = next_account_info(accounts_iter)?;
    let user_stake_info = next_account_info(accounts_iter)?;
    let token_mint_info = next_account_info(accounts_iter)?;
    let config_info = next_account_info(accounts_iter)?;
    let clock_sysvar_info = next_account_info(accounts_iter)?;

    // Verify staker is signer
    if !staker_info.is_signer {
        return Err(SolcatError::NotAuthorized.into());
    }

    // Load config and verify staking is enabled
    let config = GlobalConfig::try_from_slice(&config_info.data.borrow())?;
    if !config.staking_enabled {
        return Err(SolcatError::StakingDisabled.into());
    }

    // Verify token mint
    if config.token_mint != *token_mint_info.key {
        return Err(SolcatError::InvalidTokenMint.into());
    }

    // Parse stake amount and duration
    let amount = u64::from_le_bytes(data[0..8].try_into().unwrap());
    let duration = i64::from_le_bytes(data[8..16].try_into().unwrap());

    if amount == 0 {
        return Err(SolcatError::InvalidStakeAmount.into());
    }

    if duration < MIN_STAKE_DURATION || duration > MAX_STAKE_DURATION {
        return Err(SolcatError::InvalidStakeAmount.into());
    }

    let clock = Clock::from_account_info(clock_sysvar_info)?;
    
    // Load or create user stake account
    let mut user_stake = if let Ok(stake) = UserStake::try_from_slice(&user_stake_info.data.borrow()) {
        stake
    } else {
        UserStake {
            owner: *staker_info.key,
            amount: 0,
            rewards_earned: 0,
            reward_per_token_paid: 0,
            lock_end_time: 0,
        }
    };

    // Update stake pool
    let mut stake_pool = StakePool::try_from_slice(&stake_pool_info.data.borrow())?;
    
    // Calculate rewards before updating stake
    let pending_reward = calculate_pending_rewards(&user_stake, &stake_pool);
    user_stake.rewards_earned = user_stake.rewards_earned.saturating_add(pending_reward);
    
    // Update user stake
    user_stake.amount = user_stake.amount.saturating_add(amount);
    user_stake.lock_end_time = clock.unix_timestamp + duration;
    user_stake.reward_per_token_paid = stake_pool.reward_per_token;

    // Update stake pool
    stake_pool.total_staked = stake_pool.total_staked.saturating_add(amount);
    stake_pool.last_update_time = clock.unix_timestamp;

    // Save state
    user_stake.serialize(&mut *user_stake_info.data.borrow_mut())?;
    stake_pool.serialize(&mut *stake_pool_info.data.borrow_mut())?;

    msg!("Tokens staked successfully");
    Ok(())
}

fn unstake_tokens(
    program_id: &Pubkey,
    accounts_iter: &mut std::slice::Iter<AccountInfo>,
    data: &[u8],
) -> ProgramResult {
    let staker_info = next_account_info(accounts_iter)?;
    let stake_pool_info = next_account_info(accounts_iter)?;
    let user_stake_info = next_account_info(accounts_iter)?;
    let clock_sysvar_info = next_account_info(accounts_iter)?;

    // Verify staker is signer
    if !staker_info.is_signer {
        return Err(SolcatError::NotAuthorized.into());
    }

    let clock = Clock::from_account_info(clock_sysvar_info)?;
    
    // Load user stake
    let mut user_stake = UserStake::try_from_slice(&user_stake_info.data.borrow())?;

    // Verify ownership
    if user_stake.owner != *staker_info.key {
        return Err(SolcatError::NotAuthorized.into());
    }

    // Check lock period
    if clock.unix_timestamp < user_stake.lock_end_time {
        return Err(SolcatError::StakeLocked.into());
    }

    // Load stake pool
    let mut stake_pool = StakePool::try_from_slice(&stake_pool_info.data.borrow())?;

    // Calculate final rewards
    let pending_reward = calculate_pending_rewards(&user_stake, &stake_pool);
    user_stake.rewards_earned = user_stake.rewards_earned.saturating_add(pending_reward);

    // Update stake pool
    stake_pool.total_staked = stake_pool.total_staked.saturating_sub(user_stake.amount);
    stake_pool.last_update_time = clock.unix_timestamp;

    // Clear user stake
    let amount = user_stake.amount;
    user_stake.amount = 0;
    user_stake.lock_end_time = 0;

    // Save state
    user_stake.serialize(&mut *user_stake_info.data.borrow_mut())?;
    stake_pool.serialize(&mut *stake_pool_info.data.borrow_mut())?;

    msg!("Tokens unstaked successfully: {}", amount);
    Ok(())
}

fn claim_rewards(
    program_id: &Pubkey,
    accounts_iter: &mut std::slice::Iter<AccountInfo>,
    _data: &[u8],
) -> ProgramResult {
    let claimer_info = next_account_info(accounts_iter)?;
    let stake_pool_info = next_account_info(accounts_iter)?;
    let user_stake_info = next_account_info(accounts_iter)?;
    let treasury_info = next_account_info(accounts_iter)?;
    let config_info = next_account_info(accounts_iter)?;

    // Verify claimer is signer
    if !claimer_info.is_signer {
        return Err(SolcatError::NotAuthorized.into());
    }

    // Load config and verify treasury
    let config = GlobalConfig::try_from_slice(&config_info.data.borrow())?;
    if config.treasury != *treasury_info.key {
        return Err(SolcatError::TreasuryMismatch.into());
    }

    // Load user stake
    let mut user_stake = UserStake::try_from_slice(&user_stake_info.data.borrow())?;

    // Verify ownership
    if user_stake.owner != *claimer_info.key {
        return Err(SolcatError::NotAuthorized.into());
    }

    // Load stake pool
    let stake_pool = StakePool::try_from_slice(&stake_pool_info.data.borrow())?;

    // Calculate pending rewards
    let pending_reward = calculate_pending_rewards(&user_stake, &stake_pool);
    let total_rewards = user_stake.rewards_earned.saturating_add(pending_reward);

    if total_rewards == 0 {
        return Err(SolcatError::InvalidRewardCalculation.into());
    }

    // Update user stake
    user_stake.rewards_earned = 0;
    user_stake.reward_per_token_paid = stake_pool.reward_per_token;

    // Save state
    user_stake.serialize(&mut *user_stake_info.data.borrow_mut())?;

    msg!("Rewards claimed successfully: {}", total_rewards);
    Ok(())
}

fn distribute_rewards(
    program_id: &Pubkey,
    accounts_iter: &mut std::slice::Iter<AccountInfo>,
    data: &[u8],
) -> ProgramResult {
    let authority_info = next_account_info(accounts_iter)?;
    let stake_pool_info = next_account_info(accounts_iter)?;
    let config_info = next_account_info(accounts_iter)?;
    let clock_sysvar_info = next_account_info(accounts_iter)?;

    // Verify authority is signer and program upgrade authority
    if !authority_info.is_signer || authority_info.key != program_id {
        return Err(SolcatError::NotAuthorized.into());
    }

    let clock = Clock::from_account_info(clock_sysvar_info)?;
    
    // Load config and stake pool
    let config = GlobalConfig::try_from_slice(&config_info.data.borrow())?;
    let mut stake_pool = StakePool::try_from_slice(&stake_pool_info.data.borrow())?;

    // Calculate new rewards
    let time_elapsed = clock.unix_timestamp - stake_pool.last_update_time;
    if time_elapsed > 0 && stake_pool.total_staked > 0 {
        let reward_amount = config.reward_rate.saturating_mul(time_elapsed as u64);
        let reward_per_token = reward_amount.saturating_div(stake_pool.total_staked);
        
        stake_pool.reward_per_token = stake_pool.reward_per_token.saturating_add(reward_per_token);
        stake_pool.last_update_time = clock.unix_timestamp;

        // Save state
        stake_pool.serialize(&mut *stake_pool_info.data.borrow_mut())?;
    }

    msg!("Rewards distributed successfully");
    Ok(())
}

// Helper function to calculate pending rewards
fn calculate_pending_rewards(user_stake: &UserStake, stake_pool: &StakePool) -> u64 {
    if user_stake.amount == 0 {
        return 0;
    }

    let reward_per_token_delta = stake_pool.reward_per_token.saturating_sub(user_stake.reward_per_token_paid);
    user_stake.amount.saturating_mul(reward_per_token_delta).saturating_div(1_000_000_000)
}

fn submit_batch_report(
    program_id: &Pubkey,
    accounts_iter: &mut std::slice::Iter<AccountInfo>,
    data: &[u8],
) -> ProgramResult {
    let reporter_info = next_account_info(accounts_iter)?;
    let batch_report_info = next_account_info(accounts_iter)?;
    let reporter_stats_info = next_account_info(accounts_iter)?;
    let clock_sysvar_info = next_account_info(accounts_iter)?;

    // Verify reporter is signer
    if !reporter_info.is_signer {
        return Err(SolcatError::NotAuthorized.into());
    }

    // Check reporter stats
    let reporter_stats = ReporterStats::try_from_slice(&reporter_stats_info.data.borrow())?;
    if reporter_stats.reputation_score < MIN_REPUTATION_SCORE {
        return Err(SolcatError::InsufficientReputation.into());
    }

    // Parse batch data
    let num_addresses = data[0] as usize;
    if num_addresses == 0 || num_addresses > MAX_BATCH_SIZE {
        return Err(SolcatError::InvalidBatchReport.into());
    }

    let mut addresses = Vec::with_capacity(num_addresses);
    let mut risk_scores = Vec::with_capacity(num_addresses);

    let mut offset = 1;
    for _ in 0..num_addresses {
        let pubkey_bytes: [u8; 32] = data[offset..offset+32].try_into().unwrap();
        addresses.push(Pubkey::new_from_array(pubkey_bytes));
        offset += 32;

        let risk_score = data[offset];
        if risk_score > 100 {
            return Err(SolcatError::InvalidRiskScore.into());
        }
        risk_scores.push(risk_score);
        offset += 1;
    }

    let clock = Clock::from_account_info(clock_sysvar_info)?;
    
    let batch_report = BatchReport {
        reporter: *reporter_info.key,
        addresses,
        risk_scores,
        timestamp: clock.unix_timestamp,
        verification_status: VerificationStatus::Pending,
    };

    batch_report.serialize(&mut *batch_report_info.data.borrow_mut())?;

    msg!("Batch report submitted successfully");
    Ok(())
}

fn verify_batch_report(
    program_id: &Pubkey,
    accounts_iter: &mut std::slice::Iter<AccountInfo>,
    data: &[u8],
) -> ProgramResult {
    let verifier_info = next_account_info(accounts_iter)?;
    let batch_report_info = next_account_info(accounts_iter)?;
    let verifier_stats_info = next_account_info(accounts_iter)?;
    let system_program_info = next_account_info(accounts_iter)?;

    // Verify verifier is signer
    if !verifier_info.is_signer {
        return Err(SolcatError::NotAuthorized.into());
    }

    // Check verifier stats
    let verifier_stats = ReporterStats::try_from_slice(&verifier_stats_info.data.borrow())?;
    if verifier_stats.total_stake < MIN_VERIFICATION_STAKE {
        return Err(SolcatError::InsufficientStake.into());
    }

    // Load batch report
    let mut batch_report = BatchReport::try_from_slice(&batch_report_info.data.borrow())?;
    if batch_report.verification_status != VerificationStatus::Pending {
        return Err(SolcatError::BatchVerificationPending.into());
    }

    // Parse verification decision
    let is_verified = data[0] != 0;
    batch_report.verification_status = if is_verified {
        VerificationStatus::Verified
    } else {
        VerificationStatus::Rejected
    };

    batch_report.serialize(&mut *batch_report_info.data.borrow_mut())?;

    msg!("Batch report verification completed");
    Ok(())
}

fn blacklist_address(
    program_id: &Pubkey,
    accounts_iter: &mut std::slice::Iter<AccountInfo>,
    data: &[u8],
) -> ProgramResult {
    let authority_info = next_account_info(accounts_iter)?;
    let address_history_info = next_account_info(accounts_iter)?;
    let clock_sysvar_info = next_account_info(accounts_iter)?;

    // Verify authority is signer and program upgrade authority
    if !authority_info.is_signer || authority_info.key != program_id {
        return Err(SolcatError::NotAuthorized.into());
    }

    // Parse blacklist data
    let reason_len = data[0] as usize;
    let reason = String::from_utf8_lossy(&data[1..1+reason_len]).to_string();

    let clock = Clock::from_account_info(clock_sysvar_info)?;

    // Load or create history
    let mut history = if let Ok(h) = ReportHistory::try_from_slice(&address_history_info.data.borrow()) {
        if h.is_blacklisted {
            return Err(SolcatError::AddressAlreadyBlacklisted.into());
        }
        h
    } else {
        ReportHistory {
            address: *address_history_info.key,
            reports: Vec::new(),
            is_blacklisted: false,
            blacklist_reason: String::new(),
            blacklist_timestamp: 0,
        }
    };

    // Update blacklist status
    history.is_blacklisted = true;
    history.blacklist_reason = reason;
    history.blacklist_timestamp = clock.unix_timestamp;

    history.serialize(&mut *address_history_info.data.borrow_mut())?;

    msg!("Address blacklisted successfully");
    Ok(())
}

fn update_history(
    program_id: &Pubkey,
    accounts_iter: &mut std::slice::Iter<AccountInfo>,
    data: &[u8],
) -> ProgramResult {
    let report_info = next_account_info(accounts_iter)?;
    let history_info = next_account_info(accounts_iter)?;
    let clock_sysvar_info = next_account_info(accounts_iter)?;

    // Load report and history
    let report = AddressReport::try_from_slice(&report_info.data.borrow())?;
    let mut history = if let Ok(h) = ReportHistory::try_from_slice(&history_info.data.borrow()) {
        h
    } else {
        ReportHistory {
            address: report.reported_address,
            reports: Vec::new(),
            is_blacklisted: false,
            blacklist_reason: String::new(),
            blacklist_timestamp: 0,
        }
    };

    let clock = Clock::from_account_info(clock_sysvar_info)?;

    // Add report to history
    history.reports.push(HistoricalReport {
        timestamp: clock.unix_timestamp,
        risk_score: report.risk_score,
        reporter: report.reporter,
        description: report.description.clone(),
    });

    history.serialize(&mut *history_info.data.borrow_mut())?;

    msg!("History updated successfully");
    Ok(())
}

// Helper function to calculate comprehensive risk score
fn calculate_risk_score(risk_assessment: &RiskAssessment, risk_metrics: &RiskMetrics) -> u8 {
    let base_score = risk_assessment.base_score as f32;
    let confidence_multiplier = risk_assessment.confidence_score as f32 / 100.0;
    
    // Calculate metrics-based score
    let volume_score = if risk_metrics.transaction_volume > 1_000_000_000_000 { // 1000 SOL
        100.0
    } else {
        (risk_metrics.transaction_volume as f32 / 1_000_000_000_000.0) * 100.0
    };

    let interaction_score = if risk_metrics.unique_interactions > 1000 {
        100.0
    } else {
        (risk_metrics.unique_interactions as f32 / 1000.0) * 100.0
    };

    let age_score = if risk_metrics.age_of_account < 86400 { // 1 day
        100.0
    } else if risk_metrics.age_of_account < 2592000 { // 30 days
        75.0
    } else if risk_metrics.age_of_account < 31536000 { // 1 year
        50.0
    } else {
        25.0
    };

    let pattern_score = (risk_metrics.suspicious_patterns.len() as f32 / 5.0).min(1.0) * 100.0;

    // Calculate weighted score
    let metrics_score = 
        volume_score * RISK_WEIGHT_TRANSACTION_VOLUME +
        interaction_score * RISK_WEIGHT_INTERACTIONS +
        age_score * RISK_WEIGHT_ACCOUNT_AGE +
        pattern_score * RISK_WEIGHT_PATTERNS;

    // Combine base score with metrics score
    let final_score = (base_score * 0.6 + metrics_score * 0.4) * confidence_multiplier;

    final_score.round().min(100.0) as u8
}

// Helper function to assess risk type severity
fn get_risk_type_severity(risk_type: &RiskType) -> u8 {
    match risk_type {
        RiskType::Ransomware => 100,
        RiskType::Malware => 90,
        RiskType::Scam => 80,
        RiskType::MoneyLaundering => 85,
        RiskType::Phishing => 75,
        RiskType::MarketManipulation => 70,
        RiskType::Unknown => 50,
    }
} 