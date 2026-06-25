//! Allowance read/write/deduction helpers for the token contract.
//!
//! All three operations are centralised here so that `approve`, `transfer_from`,
//! and `burn_from` share a single code path, eliminating the duplication that
//! previously existed across `token_interface.rs`.

use soroban_sdk::{Env, panic_with_error};

use crate::errors::TokenError;
use crate::storage::{AllowanceDataKey, AllowanceValue, DataKey};

/// Returns the active allowance (amount) for `(from, spender)`, or `0` if the
/// allowance is absent or expired.
pub fn get_allowance(env: &Env, from: soroban_sdk::Address, spender: soroban_sdk::Address) -> i128 {
    let key = DataKey::Allowance(AllowanceDataKey { from, spender });
    let val: Option<AllowanceValue> = env.storage().temporary().get(&key);
    match val {
        Some(v) if env.ledger().sequence() <= v.expiration_ledger => v.amount,
        _ => 0,
    }
}

/// Writes a new allowance entry.  Extends the TTL when `expiration_ledger` is in
/// the future so the entry survives until the approval expires.
pub fn set_allowance(
    env: &Env,
    from: soroban_sdk::Address,
    spender: soroban_sdk::Address,
    amount: i128,
    expiration_ledger: u32,
) {
    let key = DataKey::Allowance(AllowanceDataKey {
        from: from.clone(),
        spender: spender.clone(),
    });
    env.storage()
        .temporary()
        .set(&key, &AllowanceValue { amount, expiration_ledger });
    if expiration_ledger > env.ledger().sequence() {
        env.storage()
            .temporary()
            .extend_ttl(&key, expiration_ledger, expiration_ledger);
    }
}

/// Deducts `amount` from the `(from, spender)` allowance, **preserving the
/// original `expiration_ledger`** so the entry's TTL is not accidentally reset.
///
/// Panics with [`TokenError::InsufficientAllowance`] when the active allowance
/// is less than `amount`.
pub fn deduct_allowance(
    env: &Env,
    from: soroban_sdk::Address,
    spender: soroban_sdk::Address,
    amount: i128,
) {
    let key = DataKey::Allowance(AllowanceDataKey {
        from: from.clone(),
        spender: spender.clone(),
    });
    let val: Option<AllowanceValue> = env.storage().temporary().get(&key);
    let (current, expiration_ledger) = match val {
        Some(v) if env.ledger().sequence() <= v.expiration_ledger => {
            (v.amount, v.expiration_ledger)
        }
        _ => (0, 0),
    };
    if current < amount {
        panic_with_error!(env, TokenError::InsufficientAllowance);
    }
    env.storage().temporary().set(
        &key,
        &AllowanceValue {
            amount: current - amount,
            // Preserve the original expiration so the TTL is not reset.
            expiration_ledger,
        },
    );
}
