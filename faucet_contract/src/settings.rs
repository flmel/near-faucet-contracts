use near_sdk::Balance;

// settings
// 20NEAR
pub const MAX_WITHDRAW_AMOUNT: Balance = 20_000_000_000_000_000_000_000_000;
// 1 hour in MS
pub const REQUEST_GAP_LIMITER: u64 = 3_600_000;
// 5000NEAR
pub const MIN_BALANCE_THRESHOLD: Balance = 5_000_000_000_000_000_000_000_000_000;
pub const VAULT_ID: &str = "vault.nonofficial.testnet";
