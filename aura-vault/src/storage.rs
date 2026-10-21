use soroban_sdk::{contracttype, Address, Env, Vec};
use crate::errors::VaultError;
#[contracttype]
pub enum DataKey {
    Admin,
    UnderlyingToken,
    TotalShares,
    TotalDeposited,
    Balance(Address),
    Version,
    LayoutVersion,
    /// Emergency pause flag — when true, deposit/withdraw/harvest are blocked.
    Paused,
    Treasury,
    PerfFeeBps,
    MgmtFeeBps,
    TotalFeeCollected,
    LastMgmtFeeTime,
    /// Whitelisted alternative yield tokens (Issue #48)
    YieldToken(Address),
    /// Maximum total assets allowed (0 = unlimited). Issue #467.
    TvlCap,
    /// Timestamp of the last successful harvest (ledger timestamp). Issue #471.
    LastHarvestTime,
    /// Minimum seconds between harvests (0 = no cooldown). Issue #471.
    HarvestCooldownSecs,
    // -----------------------------------------------------------------------
    // Yield-distribution per-share accumulator (Issue #YD)
    // -----------------------------------------------------------------------
    /// Global cumulative yield-per-share (YPS) accumulator (scaled by YIELD_PRECISION).
    CumulativeYps,
    /// Per-user YPS checkpoint: the value of CumulativeYps at the user's last
    /// collect_pending_yield or deposit call.
    UserCheckpoint(Address),
    /// Stored (unsettled) pending yield tokens for a user, in underlying units.
    UserPendingYield(Address),
    /// Monotonically increasing epoch counter; bumped on every distribution.
    DistributionEpoch,
    // -----------------------------------------------------------------------
    // Withdrawal queue (Issue #WQ)
    // -----------------------------------------------------------------------
    /// Minimum underlying-token amount that triggers queue instead of instant withdrawal.
    WithdrawalQueueThreshold,
    /// Minimum seconds a queued withdrawal must wait before it can be claimed.
    WithdrawalUnbondingSecs,
    /// Per-user withdrawal fee in basis points (0 = no fee, max 500 = 5%).
    WithdrawalFeeBps,
    /// Next sequential withdrawal queue ID.
    WithdrawalNextId,
    /// Individual withdrawal queue entry keyed by queue ID.
    WithdrawalEntry(u64),
    // -----------------------------------------------------------------------
    // Withdrawal rate limit (Issue #913)
    // -----------------------------------------------------------------------
    MaxWithdrawalBps,
    LastWithdrawalLedger,
    CurrentLedgerWithdrawals,
    // -----------------------------------------------------------------------
    // Circuit breaker — share-price movement limit (Issue #371)
    // -----------------------------------------------------------------------
    /// Maximum allowed share-price movement per harvest, in basis points.
    /// 0 = check disabled. Covers both up and down movements.
    PriceMovementLimit,
    // -----------------------------------------------------------------------
    // Whitelist-only deposit mode (Issue #349)
    // -----------------------------------------------------------------------
    /// Whether whitelist-only mode is enabled for deposits.
    WhitelistEnabled,
    /// Per-address whitelist status (persistent storage).
    Whitelist(Address),
    // -----------------------------------------------------------------------
    // Minimum deposit amount (Issue #355)
    // -----------------------------------------------------------------------
    /// Minimum deposit amount in underlying token units.
    MinDeposit,
    // -----------------------------------------------------------------------
    // Contract metadata (Issue #350, Issue #347)
    // -----------------------------------------------------------------------
    /// Vault name (set at initialization).
    VaultName,
    /// Vault share symbol (set at initialization).
    VaultSymbol,
    /// Contract version integer (set at initialization).
    VaultVersion,
    /// Vault share decimals (set at initialization, immutable). Issue #347.
    Decimals,
    // -----------------------------------------------------------------------
    // Reentrancy guard (Issue #345)
    // -----------------------------------------------------------------------
    /// Reentrancy guard lock flag.
    ReentrancyGuard,
    // ---------------------------------------------------------------------------
    // Multi-sig admin operations (Issue #375)
    // ---------------------------------------------------------------------------
    /// Ordered list of current multi-sig signers
    MultiSigSigners,
    /// M (threshold) — number of signatures required to execute an operation
    MultiSigThreshold,
    /// Monotonically increasing operation counter
    MultiSigOpCount,
    /// Per-signer vote record — prevents double-signing. Tuple: (op_id, signer).
    MultiSigVote(u64, Address),
    /// Per-address role bitmask used by admin/keeper/guardian checks.
    Role(Address),
    /// Pending admin nominated for a two-step transfer.
    PendingAdmin,
    /// Unix timestamp at which a pending admin transfer expires.
    PendingAdminExpiry,
}
pub const ADMIN_ROLE: u32 = 1 << 0;
pub const KEEPER_ROLE: u32 = 1 << 1;
pub const GUARDIAN_ROLE: u32 = 1 << 2;
pub const DAY_IN_LEDGERS: u32 = 17_280;
pub const INSTANCE_LIFETIME_THRESHOLD: u32 = DAY_IN_LEDGERS * 7;
pub const INSTANCE_BUMP_AMOUNT: u32 = DAY_IN_LEDGERS * 30;
pub const PERSISTENT_LIFETIME_THRESHOLD: u32 = DAY_IN_LEDGERS * 7;
pub const PERSISTENT_BUMP_AMOUNT: u32 = DAY_IN_LEDGERS * 30;
// ---------------------------------------------------------------------------
// Instance-storage helpers
// ---------------------------------------------------------------------------
pub fn is_initialized(env: &Env) -> bool {
    env.storage().instance().has(&DataKey::UnderlyingToken)
}
pub fn get_admin(env: &Env) -> Option<Address> {
    env.storage().instance().get(&DataKey::Admin)
}
pub fn set_admin(env: &Env, admin: &Address) {
    env.storage().instance().set(&DataKey::Admin, admin);
}
pub fn get_token(env: &Env) -> Option<Address> {
    env.storage().instance().get(&DataKey::UnderlyingToken)
}
pub fn set_token(env: &Env, token: &Address) {
    env.storage().instance().set(&DataKey::UnderlyingToken, token);
}
pub fn get_total_shares(env: &Env) -> i128 {
    env.storage().instance().get(&DataKey::TotalShares).unwrap_or(0)
}
pub fn set_total_shares(env: &Env, val: i128) {
    env.storage().instance().set(&DataKey::TotalShares, &val);
}
pub fn get_total_deposited(env: &Env) -> i128 {
    env.storage().instance().get(&DataKey::TotalDeposited).unwrap_or(0)
}
pub fn set_total_deposited(env: &Env, val: i128) {
    env.storage().instance().set(&DataKey::TotalDeposited, &val);
}
// ---------------------------------------------------------------------------
// Persistent-storage helpers
// ---------------------------------------------------------------------------
pub fn get_balance(env: &Env, addr: &Address) -> i128 {
    env.storage()
        .persistent()
        .get(&DataKey::Balance(addr.clone()))
        .unwrap_or(0)
}
pub fn set_balance(env: &Env, addr: &Address, val: i128) {
    env.storage()
        .persistent()
        .set(&DataKey::Balance(addr.clone()), &val);
}
// ---------------------------------------------------------------------------
// Fee storage helpers (instance storage)
// ---------------------------------------------------------------------------
pub fn get_treasury(env: &Env) -> Option<Address> {
    env.storage().instance().get(&DataKey::Treasury)
}
pub fn set_treasury(env: &Env, treasury: &Address) {
    env.storage().instance().set(&DataKey::Treasury, treasury);
}
pub fn get_perf_fee_bps(env: &Env) -> u32 {
    env.storage().instance().get(&DataKey::PerfFeeBps).unwrap_or(1000)
}
pub fn set_perf_fee_bps(env: &Env, bps: u32) {
    env.storage().instance().set(&DataKey::PerfFeeBps, &bps);
}
pub fn get_mgmt_fee_bps(env: &Env) -> u32 {
    env.storage().instance().get(&DataKey::MgmtFeeBps).unwrap_or(0)
}
pub fn set_mgmt_fee_bps(env: &Env, bps: u32) {
    env.storage().instance().set(&DataKey::MgmtFeeBps, &bps);
}
pub fn get_total_fee_collected(env: &Env) -> i128 {
    env.storage().instance().get(&DataKey::TotalFeeCollected).unwrap_or(0)
}
pub fn set_total_fee_collected(env: &Env, val: i128) {
    env.storage().instance().set(&DataKey::TotalFeeCollected, &val);
}
pub fn get_last_mgmt_fee_time(env: &Env) -> u64 {
    env.storage().instance().get(&DataKey::LastMgmtFeeTime).unwrap_or(0)
}
pub fn set_last_mgmt_fee_time(env: &Env, time: u64) {
    env.storage().instance().set(&DataKey::LastMgmtFeeTime, &time);
}
// ---------------------------------------------------------------------------
// Yield-token whitelist helpers (instance storage — Issue #48)
// ---------------------------------------------------------------------------
pub fn is_yield_token(env: &Env, token: &Address) -> bool {
    env.storage()
        .instance()
        .get(&DataKey::YieldToken(token.clone()))
        .unwrap_or(false)
}
pub fn set_yield_token(env: &Env, token: &Address, enabled: bool) {
    env.storage()
        .instance()
        .set(&DataKey::YieldToken(token.clone()), &enabled);
}
// ---------------------------------------------------------------------------
// TTL bump helpers
// ---------------------------------------------------------------------------
pub fn bump_instance(env: &Env) {
    env.storage()
        .instance()
        .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
}
pub fn bump_persistent(env: &Env, addr: &Address) {
    env.storage()
        .persistent()
        .extend_ttl(
            &DataKey::Balance(addr.clone()),
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );
}
// ---------------------------------------------------------------------------
// Version helpers (instance storage — same TTL as the rest of state)
// ---------------------------------------------------------------------------
pub const CURRENT_LAYOUT_VERSION: u32 = 1;
pub fn get_version(env: &Env) -> u32 {
    env.storage().instance().get(&DataKey::Version).unwrap_or(0)
}
pub fn set_version(env: &Env, v: u32) {
    env.storage().instance().set(&DataKey::Version, &v);
}
pub fn get_layout_version(env: &Env) -> u32 {
    env.storage()
        .instance()
        .get(&DataKey::LayoutVersion)
        .unwrap_or(0)
}
pub fn set_layout_version(env: &Env, v: u32) {
    env.storage().instance().set(&DataKey::LayoutVersion, &v);
}
// ---------------------------------------------------------------------------
// Pause helpers (instance storage)
// ---------------------------------------------------------------------------
pub fn is_paused(env: &Env) -> bool {
    env.storage().instance().get(&DataKey::Paused).unwrap_or(false)
}
pub fn set_paused(env: &Env, paused: bool) {
    env.storage().instance().set(&DataKey::Paused, &paused);
}
// ---------------------------------------------------------------------------
// TVL cap helpers (instance storage) — Issue #467
// ---------------------------------------------------------------------------
pub fn get_tvl_cap(env: &Env) -> i128 {
    env.storage().instance().get(&DataKey::TvlCap).unwrap_or(0)
}
pub fn set_tvl_cap(env: &Env, cap: i128) {
    env.storage().instance().set(&DataKey::TvlCap, &cap);
}
// ---------------------------------------------------------------------------
// Harvest cooldown helpers (instance storage) — Issue #471
// ---------------------------------------------------------------------------
pub fn get_last_harvest_time(env: &Env) -> u64 {
    env.storage().instance().get(&DataKey::LastHarvestTime).unwrap_or(0)
}
pub fn set_last_harvest_time(env: &Env, ts: u64) {
    env.storage().instance().set(&DataKey::LastHarvestTime, &ts);
}
pub fn get_harvest_cooldown_secs(env: &Env) -> u64 {
    env.storage().instance().get(&DataKey::HarvestCooldownSecs).unwrap_or(0)
}
pub fn set_harvest_cooldown_secs(env: &Env, secs: u64) {
    env.storage().instance().set(&DataKey::HarvestCooldownSecs, &secs);
}
// ---------------------------------------------------------------------------
// Yield-distribution accumulator helpers (Issue #YD)
// ---------------------------------------------------------------------------
pub fn get_cumulative_yps(env: &Env) -> i128 {
    env.storage().instance().get(&DataKey::CumulativeYps).unwrap_or(0)
}
pub fn set_cumulative_yps(env: &Env, val: i128) {
    env.storage().instance().set(&DataKey::CumulativeYps, &val);
}
pub fn get_user_checkpoint(env: &Env, addr: &Address) -> i128 {
    env.storage()
        .persistent()
        .get(&DataKey::UserCheckpoint(addr.clone()))
        .unwrap_or(0)
}
pub fn set_user_checkpoint(env: &Env, addr: &Address, val: i128) {
    env.storage()
        .persistent()
        .set(&DataKey::UserCheckpoint(addr.clone()), &val);
}
pub fn get_user_pending_yield(env: &Env, addr: &Address) -> i128 {
    env.storage()
        .persistent()
        .get(&DataKey::UserPendingYield(addr.clone()))
        .unwrap_or(0)
}
pub fn set_user_pending_yield(env: &Env, addr: &Address, val: i128) {
    env.storage()
        .persistent()
        .set(&DataKey::UserPendingYield(addr.clone()), &val);
}
pub fn get_distribution_epoch(env: &Env) -> u64 {
    env.storage().instance().get(&DataKey::DistributionEpoch).unwrap_or(0)
}
pub fn set_distribution_epoch(env: &Env, epoch: u64) {
    env.storage().instance().set(&DataKey::DistributionEpoch, &epoch);
}
pub fn bump_user_yield_ttl(env: &Env, addr: &Address) {
    let ck = DataKey::UserCheckpoint(addr.clone());
    if env.storage().persistent().has(&ck) {
        env.storage().persistent().extend_ttl(&ck, PERSISTENT_LIFETIME_THRESHOLD, PERSISTENT_BUMP_AMOUNT);
    }
    let py = DataKey::UserPendingYield(addr.clone());
    if env.storage().persistent().has(&py) {
        env.storage().persistent().extend_ttl(&py, PERSISTENT_LIFETIME_THRESHOLD, PERSISTENT_BUMP_AMOUNT);
    }
}
// ---------------------------------------------------------------------------
// Withdrawal queue helpers (Issue #WQ)
// ---------------------------------------------------------------------------
pub fn get_withdrawal_queue_threshold(env: &Env) -> i128 {
    env.storage().instance().get(&DataKey::WithdrawalQueueThreshold).unwrap_or(0)
}
pub fn set_withdrawal_queue_threshold(env: &Env, threshold: i128) {
    env.storage().instance().set(&DataKey::WithdrawalQueueThreshold, &threshold);
}
pub fn get_withdrawal_unbonding_secs(env: &Env) -> u64 {
    env.storage().instance().get(&DataKey::WithdrawalUnbondingSecs).unwrap_or(0)
}
pub fn set_withdrawal_unbonding_secs(env: &Env, secs: u64) {
    env.storage().instance().set(&DataKey::WithdrawalUnbondingSecs, &secs);
}
pub fn get_withdrawal_fee_bps(env: &Env) -> u32 {
    env.storage().instance().get(&DataKey::WithdrawalFeeBps).unwrap_or(0)
}
pub fn set_withdrawal_fee_bps(env: &Env, bps: u32) {
    env.storage().instance().set(&DataKey::WithdrawalFeeBps, &bps);
}
pub fn get_withdrawal_next_id(env: &Env) -> u64 {
    env.storage().instance().get(&DataKey::WithdrawalNextId).unwrap_or(1)
}
pub fn set_withdrawal_next_id(env: &Env, id: u64) {
    env.storage().instance().set(&DataKey::WithdrawalNextId, &id);
}
pub fn get_withdrawal_entry(env: &Env, id: u64) -> Option<WithdrawalEntry> {
    env.storage().persistent().get(&DataKey::WithdrawalEntry(id))
}
pub fn set_withdrawal_entry(env: &Env, id: u64, entry: &WithdrawalEntry) {
    env.storage()
        .persistent()
        .set(&DataKey::WithdrawalEntry(id), entry);
    let key = DataKey::WithdrawalEntry(id);
    env.storage().persistent().extend_ttl(&key, PERSISTENT_LIFETIME_THRESHOLD, PERSISTENT_BUMP_AMOUNT);
}
pub fn remove_withdrawal_entry(env: &Env, id: u64) {
    env.storage().persistent().remove(&DataKey::WithdrawalEntry(id));
}

// ---------------------------------------------------------------------------
// Withdrawal rate limit helpers (Issue #913)
// ---------------------------------------------------------------------------
pub fn get_max_withdrawal_bps(env: &Env) -> u32 {
    env.storage().instance().get(&DataKey::MaxWithdrawalBps).unwrap_or(0)
}
pub fn set_max_withdrawal_bps(env: &Env, bps: u32) {
    env.storage().instance().set(&DataKey::MaxWithdrawalBps, &bps);
}
pub fn get_last_withdrawal_ledger(env: &Env) -> u32 {
    env.storage().instance().get(&DataKey::LastWithdrawalLedger).unwrap_or(0)
}
pub fn set_last_withdrawal_ledger(env: &Env, ledger: u32) {
    env.storage().instance().set(&DataKey::LastWithdrawalLedger, &ledger);
}
pub fn get_current_ledger_withdrawals(env: &Env) -> i128 {
    env.storage().instance().get(&DataKey::CurrentLedgerWithdrawals).unwrap_or(0)
}
pub fn set_current_ledger_withdrawals(env: &Env, amount: i128) {
    env.storage().instance().set(&DataKey::CurrentLedgerWithdrawals, &amount);
}

// ---------------------------------------------------------------------------
// Circuit-breaker helpers (instance storage) — Issue #371
// ---------------------------------------------------------------------------
pub fn get_price_movement_limit(env: &Env) -> u32 {
    env.storage()
        .instance()
        .get(&DataKey::PriceMovementLimit)
        .unwrap_or(0)
}
pub fn set_price_movement_limit(env: &Env, bps: u32) {
    env.storage()
        .instance()
        .set(&DataKey::PriceMovementLimit, &bps);
}
#[soroban_sdk::contracttype]
#[derive(Clone, Debug)]
pub struct WithdrawalEntry {
    pub owner: Address,
    pub shares: i128,
    pub redeem_amount: i128,
    pub claimable_after: u64,
    pub claimed: bool,
}
pub fn get_whitelist_enabled(env: &Env) -> bool {
    env.storage().instance().get(&DataKey::WhitelistEnabled).unwrap_or(false)
}
pub fn set_whitelist_enabled(env: &Env, enabled: bool) {
    env.storage().instance().set(&DataKey::WhitelistEnabled, &enabled);
}
pub fn is_whitelisted(env: &Env, addr: &Address) -> bool {
    env.storage()
        .persistent()
        .get(&DataKey::Whitelist(addr.clone()))
        .unwrap_or(false)
}
pub fn set_whitelisted(env: &Env, addr: &Address, whitelisted: bool) {
    env.storage()
        .persistent()
        .set(&DataKey::Whitelist(addr.clone()), &whitelisted);
    let key = DataKey::Whitelist(addr.clone());
    env.storage().persistent().extend_ttl(&key, PERSISTENT_LIFETIME_THRESHOLD, PERSISTENT_BUMP_AMOUNT);
}
pub fn get_min_deposit(env: &Env) -> i128 {
    env.storage().instance().get(&DataKey::MinDeposit).unwrap_or(10_000)
}
pub fn set_min_deposit(env: &Env, amount: i128) {
    env.storage().instance().set(&DataKey::MinDeposit, &amount);
}
pub fn get_vault_name(env: &Env) -> Option<soroban_sdk::String> {
    env.storage().instance().get(&DataKey::VaultName)
}
pub fn set_vault_name(env: &Env, name: &soroban_sdk::String) {
    env.storage().instance().set(&DataKey::VaultName, name);
}
pub fn get_vault_symbol(env: &Env) -> Option<soroban_sdk::String> {
    env.storage().instance().get(&DataKey::VaultSymbol)
}
pub fn set_vault_symbol(env: &Env, symbol: &soroban_sdk::String) {
    env.storage().instance().set(&DataKey::VaultSymbol, symbol);
}
pub fn get_vault_version(env: &Env) -> u32 {
    env.storage().instance().get(&DataKey::VaultVersion).unwrap_or(1u32)
}
pub fn set_vault_version(env: &Env, version: u32) {
    env.storage().instance().set(&DataKey::VaultVersion, &version);
}
pub fn get_decimals(env: &Env) -> u32 {
    env.storage().instance().get(&DataKey::Decimals).unwrap_or(7u32)
}
pub fn set_decimals(env: &Env, decimals: u32) {
    env.storage().instance().set(&DataKey::Decimals, &decimals);
}
pub fn is_reentrancy_locked(env: &Env) -> bool {
    env.storage().instance().get(&DataKey::ReentrancyGuard).unwrap_or(false)
}
pub fn set_reentrancy_lock(env: &Env, locked: bool) {
    env.storage().instance().set(&DataKey::ReentrancyGuard, &locked);
}
pub fn enter_reentrancy_guard(env: &Env) -> Result<(), VaultError> {
    if is_reentrancy_locked(env) {
        return Err(VaultError::Reentrancy);
    }
    set_reentrancy_lock(env, true);
    Ok(())
}
pub fn exit_reentrancy_guard(env: &Env) {
    set_reentrancy_lock(env, false);
}
pub const DEFAULT_THRESHOLD: u32 = 2;
pub const MULTISIG_EXPIRY_SECS: u64 = 72 * 60 * 60;
pub fn get_multisig_signers(env: &Env) -> Vec<Address> {
    env.storage()
        .instance()
        .get(&DataKey::MultiSigSigners)
        .unwrap_or_else(|| Vec::new(env))
}
pub fn set_multisig_signers(env: &Env, signers: &Vec<Address>) {
    env.storage()
        .instance()
        .set(&DataKey::MultiSigSigners, signers);
}
pub fn get_multisig_threshold(env: &Env) -> u32 {
    env.storage()
        .instance()
        .get(&DataKey::MultiSigThreshold)
        .unwrap_or(DEFAULT_THRESHOLD)
}
pub fn set_multisig_threshold(env: &Env, threshold: u32) {
    env.storage()
        .instance()
        .set(&DataKey::MultiSigThreshold, &threshold);
}
pub fn get_multisig_op_count(env: &Env) -> u64 {
    env.storage()
        .instance()
        .get(&DataKey::MultiSigOpCount)
        .unwrap_or(0)
}
pub fn set_multisig_op_count(env: &Env, count: u64) {
    env.storage()
        .instance()
        .set(&DataKey::MultiSigOpCount, &count);
}
pub fn get_role_mask(env: &Env, addr: &Address) -> u32 {
    env.storage()
        .persistent()
        .get(&DataKey::Role(addr.clone()))
        .unwrap_or(0)
}
pub fn set_role_mask(env: &Env, addr: &Address, mask: u32) {
    env.storage().persistent().set(&DataKey::Role(addr.clone()), &mask);
    let key = DataKey::Role(addr.clone());
    env.storage().persistent().extend_ttl(&key, PERSISTENT_LIFETIME_THRESHOLD, PERSISTENT_BUMP_AMOUNT);
}
pub fn has_role(env: &Env, addr: &Address, role: u32) -> bool {
    (get_role_mask(env, addr) & role) == role
}
pub fn get_pending_admin(env: &Env) -> Option<Address> {
    env.storage().instance().get(&DataKey::PendingAdmin)
}
pub fn set_pending_admin(env: &Env, addr: &Address) {
    env.storage().instance().set(&DataKey::PendingAdmin, addr);
}
pub fn clear_pending_admin(env: &Env) {
    env.storage().instance().remove(&DataKey::PendingAdmin);
}
pub fn get_pending_admin_expiry(env: &Env) -> u64 {
    env.storage().instance().get(&DataKey::PendingAdminExpiry).unwrap_or(0)
}
pub fn set_pending_admin_expiry(env: &Env, expiry: u64) {
    env.storage().instance().set(&DataKey::PendingAdminExpiry, &expiry);
}
pub fn clear_pending_admin_expiry(env: &Env) {
    env.storage().instance().remove(&DataKey::PendingAdminExpiry);
}
pub fn has_multisig_signed(env: &Env, op_id: u64, signer: &Address) -> bool {
    env.storage()
        .instance()
        .get::<DataKey, bool>(&DataKey::MultiSigVote(op_id, signer.clone()))
        .unwrap_or(false)
}
pub fn record_multisig_vote(env: &Env, op_id: u64, signer: &Address) {
    env.storage().instance().set(
        &DataKey::MultiSigVote(op_id, signer.clone()),
        &true,
    );
}