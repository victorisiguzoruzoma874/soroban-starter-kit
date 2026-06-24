//! TokenInterface method implementations.
//!
//! Each function here maps 1-to-1 to a method on `token::TokenInterface`.
//! `lib.rs` hosts the `#[contractimpl]` block and delegates to these functions.

use soroban_sdk::{panic_with_error, Address, Env, String};

use crate::errors::TokenError;
use crate::events;
use crate::storage::{AllowanceDataKey, AllowanceValue, DataKey, MetadataKey};
use crate::TokenContract;
use soroban_common::{extend_ttl_instance, LEDGER_BUMP_AMOUNT, LEDGER_LIFETIME_THRESHOLD};

#[cfg(feature = "pausable")]
use crate::require_not_paused;

#[cfg(feature = "freeze")]
use crate::require_not_frozen;

pub fn allowance(env: Env, from: Address, spender: Address) -> i128 {
    let key = DataKey::Allowance(AllowanceDataKey { from, spender });
    let val: Option<AllowanceValue> = env.storage().temporary().get(&key);
    match val {
        Some(v) if env.ledger().sequence() <= v.expiration_ledger => v.amount,
        _ => 0,
    }
}

pub fn approve(env: Env, from: Address, spender: Address, amount: i128, expiration_ledger: u32) {
    from.require_auth();
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
    if amount == 0 {
        events::revoked(&env, &from, &spender);
    } else {
        events::approved(&env, &from, &spender, amount);
    }
}

pub fn balance(env: Env, id: Address) -> i128 {
    env.storage()
        .persistent()
        .get(&DataKey::Balance(id))
        .unwrap_or(0)
}

pub fn transfer(env: Env, from: Address, to: Address, amount: i128) {
    from.require_auth();
    #[cfg(feature = "pausable")]
    if let Err(e) = require_not_paused(&env) {
        panic_with_error!(&env, e);
    }
    #[cfg(feature = "freeze")]
    if let Err(e) = require_not_frozen(&env, &from) {
        panic_with_error!(&env, e);
    }
    if let Err(e) = TokenContract::transfer_impl(&env, from, to, amount) {
        panic_with_error!(&env, e);
    }
}

pub fn transfer_from(env: Env, spender: Address, from: Address, to: Address, amount: i128) {
    spender.require_auth();
    #[cfg(feature = "pausable")]
    if let Err(e) = require_not_paused(&env) {
        panic_with_error!(&env, e);
    }
    #[cfg(feature = "freeze")]
    if let Err(e) = require_not_frozen(&env, &from) {
        panic_with_error!(&env, e);
    }
    let key = DataKey::Allowance(AllowanceDataKey {
        from: from.clone(),
        spender: spender.clone(),
    });
    let val: Option<AllowanceValue> = env.storage().temporary().get(&key);
    let (allowance, expiration_ledger) = match val {
        Some(v) if env.ledger().sequence() <= v.expiration_ledger => (v.amount, v.expiration_ledger),
        _ => (0, 0),
    };
    if allowance < amount {
        panic_with_error!(&env, TokenError::InsufficientAllowance);
    }
    env.storage().temporary().set(
        &key,
        &AllowanceValue {
            amount: allowance - amount,
            expiration_ledger,
        },
    );
    if let Err(e) = TokenContract::transfer_impl(&env, from, to, amount) {
        panic_with_error!(&env, e);
    }
}

pub fn burn(env: Env, from: Address, amount: i128) {
    from.require_auth();
    #[cfg(feature = "pausable")]
    if let Err(e) = require_not_paused(&env) {
        panic_with_error!(&env, e);
    }
    #[cfg(feature = "freeze")]
    if let Err(e) = require_not_frozen(&env, &from) {
        panic_with_error!(&env, e);
    }
    if amount <= 0 {
        panic_with_error!(&env, TokenError::InvalidAmount);
    }
    if let Err(e) = TokenContract::update_balance(&env, &from, -amount) {
        panic_with_error!(&env, e);
    }
    let supply: i128 = env
        .storage()
        .instance()
        .get(&DataKey::TotalSupply)
        .unwrap_or(0);
    let new_supply = match supply.checked_sub(amount) {
        Some(v) => v,
        None => panic_with_error!(&env, TokenError::Overflow),
    };
    env.storage()
        .instance()
        .set(&DataKey::TotalSupply, &new_supply);
    extend_ttl_instance(&env, LEDGER_LIFETIME_THRESHOLD, LEDGER_BUMP_AMOUNT);
    events::burned(&env, &from, amount);
}

pub fn burn_from(env: Env, spender: Address, from: Address, amount: i128) {
    spender.require_auth();
    #[cfg(feature = "pausable")]
    if let Err(e) = require_not_paused(&env) {
        panic_with_error!(&env, e);
    }
    #[cfg(feature = "freeze")]
    if let Err(e) = require_not_frozen(&env, &from) {
        panic_with_error!(&env, e);
    }
    let key = DataKey::Allowance(AllowanceDataKey {
        from: from.clone(),
        spender: spender.clone(),
    });
    let val: Option<AllowanceValue> = env.storage().temporary().get(&key);
    let (allowance, expiration_ledger) = match val {
        Some(v) if env.ledger().sequence() <= v.expiration_ledger => (v.amount, v.expiration_ledger),
        _ => (0, 0),
    };
    if allowance < amount {
        panic_with_error!(&env, TokenError::InsufficientAllowance);
    }
    env.storage().temporary().set(
        &key,
        &AllowanceValue {
            amount: allowance - amount,
            expiration_ledger,
        },
    );
    if let Err(e) = TokenContract::update_balance(&env, &from, -amount) {
        panic_with_error!(&env, e);
    }
    let supply: i128 = env
        .storage()
        .instance()
        .get(&DataKey::TotalSupply)
        .unwrap_or(0);
    let new_supply = match supply.checked_sub(amount) {
        Some(v) => v,
        None => panic_with_error!(&env, TokenError::Overflow),
    };
    env.storage()
        .instance()
        .set(&DataKey::TotalSupply, &new_supply);
    extend_ttl_instance(&env, LEDGER_LIFETIME_THRESHOLD, LEDGER_BUMP_AMOUNT);
    events::burned(&env, &from, amount);
}

pub fn decimals(env: Env) -> u32 {
    env.storage()
        .instance()
        .get(&DataKey::Metadata(MetadataKey::Decimals))
        .unwrap_or_default()
}

pub fn name(env: Env) -> String {
    env.storage()
        .instance()
        .get(&DataKey::Metadata(MetadataKey::Name))
        .unwrap_or_else(|| String::from_str(&env, ""))
}

pub fn symbol(env: Env) -> String {
    env.storage()
        .instance()
        .get(&DataKey::Metadata(MetadataKey::Symbol))
        .unwrap_or_else(|| String::from_str(&env, ""))
}
