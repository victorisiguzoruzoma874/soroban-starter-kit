#![no_std]

use soroban_sdk::{
    contract, contractimpl, panic_with_error, token, Address, Env, String,
};
use token::TokenInterface as _;

mod admin;
mod errors;
mod events;
mod storage;

#[cfg(test)]
mod test;

use admin::require_admin;
use errors::TokenError;
use storage::{AllowanceDataKey, AllowanceValue, DataKey, MetadataKey};

const LEDGER_LIFETIME_THRESHOLD: u32 = 120_960;
const LEDGER_BUMP_AMOUNT: u32 = 518_400;

fn bump_instance(env: &Env) {
    env.storage()
        .instance()
        .extend_ttl(LEDGER_LIFETIME_THRESHOLD, LEDGER_BUMP_AMOUNT);
}

fn bump_persistent(env: &Env, key: &DataKey) {
    env.storage()
        .persistent()
        .extend_ttl(key, LEDGER_LIFETIME_THRESHOLD, LEDGER_BUMP_AMOUNT);
}

#[cfg(feature = "pausable")]
fn require_not_paused(env: &Env) -> Result<(), TokenError> {
    if env
        .storage()
        .instance()
        .get(&DataKey::Paused)
        .unwrap_or(false)
    {
        return Err(TokenError::Unauthorized);
    }
    Ok(())
}

#[contract]
pub struct TokenContract;

#[contractimpl]
impl TokenContract {
    /// Initialize the token with metadata and an admin. Must be called once.
    ///
    /// `max_supply` is only enforced when the `capped-supply` feature is enabled.
    pub fn initialize(
        env: Env,
        admin: Address,
        name: String,
        symbol: String,
        decimals: u32,
        max_supply: Option<i128>,
    ) -> Result<(), TokenError> {
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(TokenError::AlreadyInitialized);
        }
        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage()
            .instance()
            .set(&DataKey::Metadata(MetadataKey::Name), &name);
        env.storage()
            .instance()
            .set(&DataKey::Metadata(MetadataKey::Symbol), &symbol);
        env.storage()
            .instance()
            .set(&DataKey::Metadata(MetadataKey::Decimals), &decimals);
        env.storage().instance().set(&DataKey::TotalSupply, &0i128);
        #[cfg(feature = "capped-supply")]
        if let Some(cap) = max_supply {
            if cap <= 0 {
                return Err(TokenError::InvalidAmount);
            }
            env.storage().instance().set(&DataKey::MaxSupply, &cap);
        }
        #[cfg(not(feature = "capped-supply"))]
        let _ = max_supply;
        bump_instance(&env);
        events::initialized(&env, &admin, name, symbol, decimals);
        Ok(())
    }

    /// Mint `amount` tokens to `to`. Admin only.
    pub fn mint(env: Env, to: Address, amount: i128) -> Result<(), TokenError> {
        #[cfg(feature = "pausable")]
        require_not_paused(&env)?;
        let admin = require_admin(&env)?;
        admin.require_auth();
        if amount <= 0 {
            return Err(TokenError::InvalidAmount);
        }
        #[cfg(feature = "capped-supply")]
        {
            let supply: i128 = env
                .storage()
                .instance()
                .get(&DataKey::TotalSupply)
                .unwrap_or(0);
            if let Some(cap) = env
                .storage()
                .instance()
                .get::<DataKey, i128>(&DataKey::MaxSupply)
            {
                if supply.checked_add(amount).ok_or(TokenError::Overflow)? > cap {
                    return Err(TokenError::InvalidAmount);
                }
            }
        }
        let balance: i128 = env
            .storage()
            .persistent()
            .get(&DataKey::Balance(to.clone()))
            .unwrap_or(0);
        let new_balance = balance.checked_add(amount).ok_or(TokenError::Overflow)?;
        env.storage()
            .persistent()
            .set(&DataKey::Balance(to.clone()), &new_balance);
        bump_persistent(&env, &DataKey::Balance(to.clone()));
        let supply: i128 = env
            .storage()
            .instance()
            .get(&DataKey::TotalSupply)
            .unwrap_or(0);
        env.storage()
            .instance()
            .set(&DataKey::TotalSupply, &(supply + amount));
        bump_instance(&env);
        events::minted(&env, &to, amount);
        Ok(())
    }

    /// Burn `amount` tokens from `from`. Admin only.
    pub fn admin_burn(env: Env, from: Address, amount: i128) -> Result<(), TokenError> {
        #[cfg(feature = "pausable")]
        require_not_paused(&env)?;
        let admin = require_admin(&env)?;
        admin.require_auth();
        if amount <= 0 {
            return Err(TokenError::InvalidAmount);
        }
        let balance: i128 = env
            .storage()
            .persistent()
            .get(&DataKey::Balance(from.clone()))
            .unwrap_or(0);
        if balance < amount {
            return Err(TokenError::InsufficientBalance);
        }
        env.storage()
            .persistent()
            .set(&DataKey::Balance(from.clone()), &(balance - amount));
        bump_persistent(&env, &DataKey::Balance(from.clone()));
        let supply: i128 = env
            .storage()
            .instance()
            .get(&DataKey::TotalSupply)
            .unwrap_or(0);
        env.storage()
            .instance()
            .set(&DataKey::TotalSupply, &(supply.checked_sub(amount).ok_or(TokenError::Overflow)?));
        bump_instance(&env);
        events::burned(&env, &from, amount);
        Ok(())
    }

    /// Propose a new admin. Current admin only.
    ///
    /// The transfer is not final until `new_admin` calls [`accept_admin`].
    /// Replaces the one-step `set_admin` to prevent accidental loss of admin
    /// access from a typo in the new address.
    pub fn propose_admin(env: Env, new_admin: Address) -> Result<(), TokenError> {
        let admin = require_admin(&env)?;
        admin.require_auth();
        env.storage().instance().set(&DataKey::PendingAdmin, &new_admin);
        bump_instance(&env);
        env.events().publish(
            (soroban_sdk::Symbol::new(&env, "admin_proposed"), admin),
            new_admin,
        );
        Ok(())
    }

    /// Accept a pending admin transfer. Must be called by the pending admin.
    pub fn accept_admin(env: Env) -> Result<(), TokenError> {
        let pending: Address = env
            .storage()
            .instance()
            .get(&DataKey::PendingAdmin)
            .ok_or(TokenError::Unauthorized)?;
        pending.require_auth();
        let old_admin = require_admin(&env)?;
        env.storage().instance().set(&DataKey::Admin, &pending);
        env.storage().instance().remove(&DataKey::PendingAdmin);
        bump_instance(&env);
        events::admin_changed(&env, &old_admin, &pending);
        Ok(())
    }

    /// Cancel a pending admin transfer. Current admin only.
    pub fn cancel_admin_transfer(env: Env) -> Result<(), TokenError> {
        let admin = require_admin(&env)?;
        admin.require_auth();
        env.storage().instance().remove(&DataKey::PendingAdmin);
        bump_instance(&env);
        Ok(())
    }

    /// Deprecated: use `propose_admin` + `accept_admin` instead.
    ///
    /// Kept for backwards compatibility. Will be removed in a future version.
    pub fn set_admin(env: Env, new_admin: Address) -> Result<(), TokenError> {
        let old_admin = require_admin(&env)?;
        old_admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &new_admin);
        bump_instance(&env);
        events::admin_changed(&env, &old_admin, &new_admin);
        Ok(())
    }

    /// Return the current admin address.
    pub fn admin(env: Env) -> Address {
        env.storage()
            .instance()
            .get(&DataKey::Admin)
            .unwrap()
    }

    /// Return the current total token supply.
    pub fn total_supply(env: Env) -> i128 {
        env.storage()
            .instance()
            .get(&DataKey::TotalSupply)
            .unwrap_or(0)
    }

    /// Return the git commit hash baked in at compile time.
    pub fn version(env: Env) -> String {
        String::from_str(&env, env!("GIT_HASH"))
    }
}

/// Pause / unpause — only compiled when the `pausable` feature is enabled.
#[cfg(feature = "pausable")]
#[contractimpl]
impl TokenContract {
    /// Pause all token operations. Admin only.
    pub fn pause(env: Env) -> Result<(), TokenError> {
        let admin = require_admin(&env)?;
        admin.require_auth();
        env.storage().instance().set(&DataKey::Paused, &true);
        bump_instance(&env);
        Ok(())
    }

    /// Resume all token operations. Admin only.
    pub fn unpause(env: Env) -> Result<(), TokenError> {
        let admin = require_admin(&env)?;
        admin.require_auth();
        env.storage().instance().set(&DataKey::Paused, &false);
        bump_instance(&env);
        Ok(())
    }
}

/// Upgrade path — only compiled when the `upgradeable` feature is enabled.
#[cfg(feature = "upgradeable")]
#[contractimpl]
impl TokenContract {
    /// Minimum ledgers between proposing and executing a WASM upgrade (~24 h at 5 s/ledger).
    const UPGRADE_DELAY_LEDGERS: u32 = 17_280;

    /// Propose a WASM upgrade. Admin only.
    ///
    /// Stores `wasm_hash` and a `ready_after` ledger number. The upgrade cannot
    /// be executed until at least `UPGRADE_DELAY_LEDGERS` ledgers have passed.
    pub fn propose_upgrade(env: Env, wasm_hash: soroban_sdk::BytesN<32>) -> Result<(), TokenError> {
        let admin = require_admin(&env)?;
        admin.require_auth();
        let ready_after = env.ledger().sequence() + Self::UPGRADE_DELAY_LEDGERS;
        env.storage()
            .instance()
            .set(&DataKey::PendingUpgrade, &(wasm_hash.clone(), ready_after));
        bump_instance(&env);
        env.events().publish(
            (soroban_sdk::Symbol::new(&env, "upgrade_proposed"), admin),
            (wasm_hash, ready_after),
        );
        Ok(())
    }

    /// Execute a previously proposed WASM upgrade. Admin only.
    ///
    /// Fails if no upgrade has been proposed or if the timelock has not yet elapsed.
    pub fn execute_upgrade(env: Env) -> Result<(), TokenError> {
        let admin = require_admin(&env)?;
        admin.require_auth();
        let (wasm_hash, ready_after): (soroban_sdk::BytesN<32>, u32) = env
            .storage()
            .instance()
            .get(&DataKey::PendingUpgrade)
            .ok_or(TokenError::Unauthorized)?;
        if env.ledger().sequence() < ready_after {
            return Err(TokenError::Unauthorized);
        }
        env.storage().instance().remove(&DataKey::PendingUpgrade);
        env.events().publish(
            (soroban_sdk::Symbol::new(&env, "upgrade_executed"), admin),
            wasm_hash.clone(),
        );
        env.deployer().update_current_contract_wasm(wasm_hash);
        Ok(())
    }
}

/// Supply cap — only compiled when the `capped-supply` feature is enabled.
#[cfg(feature = "capped-supply")]
#[contractimpl]
impl TokenContract {
    /// Return the configured maximum supply cap, or `None` if uncapped.
    pub fn max_supply(env: Env) -> Option<i128> {
        env.storage().instance().get(&DataKey::MaxSupply)
    }
}

#[contractimpl]
impl token::TokenInterface for TokenContract {
    fn allowance(env: Env, from: Address, spender: Address) -> i128 {
        let key = DataKey::Allowance(AllowanceDataKey {
            from: from.clone(),
            spender: spender.clone(),
        });
        let val: Option<AllowanceValue> = env.storage().temporary().get(&key);
        match val {
            Some(v) if env.ledger().sequence() <= v.expiration_ledger => v.amount,
            _ => 0,
        }
    }

    fn approve(
        env: Env,
        from: Address,
        spender: Address,
        amount: i128,
        expiration_ledger: u32,
    ) {
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

    fn balance(env: Env, id: Address) -> i128 {
        // Returns 0 for both unknown addresses and addresses with a zero balance.
        // Use `balance_of` to distinguish between the two cases.
        env.storage()
            .persistent()
            .get(&DataKey::Balance(id))
            .unwrap_or(0)
    }

    fn transfer(env: Env, from: Address, to: Address, amount: i128) {
        from.require_auth();
        #[cfg(feature = "pausable")]
        if let Err(e) = require_not_paused(&env) {
            panic_with_error!(&env, e);
        }
        if let Err(e) = Self::transfer_impl(&env, from, to, amount) {
            panic_with_error!(&env, e);
        }
    }

    fn transfer_from(env: Env, spender: Address, from: Address, to: Address, amount: i128) {
        spender.require_auth();
        #[cfg(feature = "pausable")]
        if let Err(e) = require_not_paused(&env) {
            panic_with_error!(&env, e);
        }
        let key = DataKey::Allowance(AllowanceDataKey {
            from: from.clone(),
            spender: spender.clone(),
        });
        let val: Option<AllowanceValue> = env.storage().temporary().get(&key);
        let allowance = match val {
            Some(v) if env.ledger().sequence() <= v.expiration_ledger => v.amount,
            _ => 0,
        };
        if allowance < amount {
            panic_with_error!(&env, TokenError::InsufficientAllowance);
        }
        env.storage().temporary().set(
            &key,
            &AllowanceValue {
                amount: allowance - amount,
                expiration_ledger: env.ledger().sequence() + LEDGER_BUMP_AMOUNT,
            },
        );
        if let Err(e) = Self::transfer_impl(&env, from, to, amount) {
            panic_with_error!(&env, e);
        }
    }

    fn burn(env: Env, from: Address, amount: i128) {
        from.require_auth();
        #[cfg(feature = "pausable")]
        if let Err(e) = require_not_paused(&env) {
            panic_with_error!(&env, e);
        }
        if amount <= 0 {
            panic_with_error!(&env, TokenError::InvalidAmount);
        }
        let balance: i128 = env
            .storage()
            .persistent()
            .get(&DataKey::Balance(from.clone()))
            .unwrap_or(0);
        if balance < amount {
            panic_with_error!(&env, TokenError::InsufficientBalance);
        }
        env.storage()
            .persistent()
            .set(&DataKey::Balance(from.clone()), &(balance - amount));
        bump_persistent(&env, &DataKey::Balance(from.clone()));
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
        bump_instance(&env);
        events::burned(&env, &from, amount);
    }

    fn burn_from(env: Env, spender: Address, from: Address, amount: i128) {
        spender.require_auth();
        #[cfg(feature = "pausable")]
        if let Err(e) = require_not_paused(&env) {
            panic_with_error!(&env, e);
        }
        let key = DataKey::Allowance(AllowanceDataKey {
            from: from.clone(),
            spender: spender.clone(),
        });
        let val: Option<AllowanceValue> = env.storage().temporary().get(&key);
        let allowance = match val {
            Some(v) if env.ledger().sequence() <= v.expiration_ledger => v.amount,
            _ => 0,
        };
        if allowance < amount {
            panic_with_error!(&env, TokenError::InsufficientAllowance);
        }
        env.storage().temporary().set(
            &key,
            &AllowanceValue {
                amount: allowance - amount,
                expiration_ledger: env.ledger().sequence() + LEDGER_BUMP_AMOUNT,
            },
        );
        let balance: i128 = env
            .storage()
            .persistent()
            .get(&DataKey::Balance(from.clone()))
            .unwrap_or(0);
        if balance < amount {
            panic_with_error!(&env, TokenError::InsufficientBalance);
        }
        env.storage()
            .persistent()
            .set(&DataKey::Balance(from.clone()), &(balance - amount));
        bump_persistent(&env, &DataKey::Balance(from.clone()));
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
        bump_instance(&env);
        events::burned(&env, &from, amount);
    }

    fn decimals(env: Env) -> u32 {
        env.storage()
            .instance()
            .get(&DataKey::Metadata(MetadataKey::Decimals))
            .unwrap()
    }

    fn name(env: Env) -> String {
        env.storage()
            .instance()
            .get(&DataKey::Metadata(MetadataKey::Name))
            .unwrap()
    }

    fn symbol(env: Env) -> String {
        env.storage()
            .instance()
            .get(&DataKey::Metadata(MetadataKey::Symbol))
            .unwrap()
    }
}

impl TokenContract {
    /// Move `amount` tokens from `from` to `to`, updating persistent storage and emitting an event.
    ///
    /// # Preconditions (caller must ensure before calling)
    /// - Caller authorization for `from` has already been checked (`from.require_auth()` or
    ///   allowance deducted).
    /// - `amount` is positive (this function also enforces it, but callers should pre-validate).
    ///
    /// # What this function validates
    /// - Returns `Ok(())` immediately when `from == to` (no-op, no event emitted).
    /// - Returns [`TokenError::InvalidAmount`] if `amount <= 0`.
    /// - Returns [`TokenError::InsufficientBalance`] if `from`'s balance is less than `amount`.
    ///
    /// # What this function does NOT validate
    /// - Does not check authorization — that is the caller's responsibility.
    /// - Does not enforce allowances — `transfer_from` deducts the allowance before calling here.
    /// - Does not check or update `TotalSupply` — supply only changes on mint/burn.
    fn transfer_impl(env: &Env, from: Address, to: Address, amount: i128) -> Result<(), TokenError> {
        if from == to {
            return Ok(());
        }
        if amount <= 0 {
            return Err(TokenError::InvalidAmount);
        }
        let from_balance: i128 = env
            .storage()
            .persistent()
            .get(&DataKey::Balance(from.clone()))
            .unwrap_or(0);
        if from_balance < amount {
            return Err(TokenError::InsufficientBalance);
        }
        env.storage()
            .persistent()
            .set(&DataKey::Balance(from.clone()), &(from_balance - amount));
        bump_persistent(env, &DataKey::Balance(from.clone()));
        let to_balance: i128 = env
            .storage()
            .persistent()
            .get(&DataKey::Balance(to.clone()))
            .unwrap_or(0);
        env.storage()
            .persistent()
            .set(&DataKey::Balance(to.clone()), &(to_balance + amount));
        bump_persistent(env, &DataKey::Balance(to.clone()));
        events::transferred(env, &from, &to, amount);
        Ok(())
    }
}
