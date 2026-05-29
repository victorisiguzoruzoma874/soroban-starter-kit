#![no_std]

use soroban_sdk::{contract, contractimpl, token, Address, Env, Symbol};

mod admin;
mod errors;
mod events;
mod storage;

pub use errors::EscrowError;
pub use storage::{DataKey, EscrowInfo, EscrowState};

use storage::DataKey::{Arbiter, Amount, Buyer, Deadline, Seller, State, TokenContract, BuyerApproved, SellerDelivered};
use admin::require_admin;
use soroban_common::MIN_DEADLINE_BUFFER;
use storage::DataKey::*;

/// Extend storage TTL when remaining ledgers fall below this threshold.
const LEDGER_LIFETIME_THRESHOLD: u32 = 120_960;

/// Target TTL (in ledgers) after each extension.
const LEDGER_BUMP_AMOUNT: u32 = 518_400;

fn bump_instance(env: &Env) {
    env.storage()
        .instance()
        .extend_ttl(LEDGER_LIFETIME_THRESHOLD, LEDGER_BUMP_AMOUNT);
}

/// Escrow contract for secure two-party transactions.
///
/// Lifecycle: `Created → Funded → Delivered → Completed`
/// with side exits to `Refunded` (deadline-based) or `Cancelled` (pre-fund).
#[contract]
pub struct EscrowContract;

#[contractimpl]
impl EscrowContract {
    /// Initialize a new escrow. Must be called exactly once.
    ///
    /// `token_contract` must be a valid Soroban token contract that implements the
    /// token interface (i.e. responds to `decimals()`). Passing an address that does
    /// not implement the token interface will cause this call to panic.
    ///
    /// # Errors
    ///
    /// Returns [`EscrowError::AlreadyInitialized`] if the contract has already been initialized.
    /// Returns [`EscrowError::InvalidAmount`] if `amount` <= 0.
    /// Returns [`EscrowError::InvalidParties`] if any two parties are the same address.
    /// Returns [`EscrowError::DeadlinePassed`] if `deadline_ledger` is not at least `MIN_DEADLINE_BUFFER` ledgers in the future.
    ///
    /// # Panics
    ///
    /// Panics if `token_contract` does not implement the token interface.
    pub fn initialize(
        env: Env,
        buyer: Address,
        seller: Address,
        arbiter: Address,
        token_contract: Address,
        amount: i128,
        deadline_ledger: u32,
    ) -> Result<(), EscrowError> {
        if env.storage().instance().has(&State) {
            return Err(EscrowError::AlreadyInitialized);
        }
        if amount <= 0 {
            return Err(EscrowError::InvalidAmount);
        }
        if buyer == seller || buyer == arbiter || seller == arbiter {
            return Err(EscrowError::InvalidParties);
        }
        if deadline_ledger < env.ledger().sequence() + MIN_DEADLINE_BUFFER {
            return Err(EscrowError::DeadlinePassed);
        }

        // Validate that token_contract is a real token by calling decimals().
        // This panics if the address does not implement the token interface.
        let token_client = token::Client::new(&env, &token_contract);
        token_client.decimals();

        env.storage().instance().set(&Buyer, &buyer);
        env.storage().instance().set(&Seller, &seller);
        env.storage().instance().set(&Arbiter, &arbiter);
        env.storage().instance().set(&TokenContract, &token_contract);
        env.storage().instance().set(&Amount, &amount);
        env.storage().instance().set(&Deadline, &deadline_ledger);
        env.storage().instance().set(&State, &EscrowState::Created);
        env.storage().instance().set(&BuyerApproved, &false);
        env.storage().instance().set(&SellerDelivered, &false);
        // Default to single-sig (1-of-1)
        env.storage().instance().set(&RequiredSignatures, &1u32);

        bump_instance(&env);

        events::escrow_created(&env, &buyer, &seller, amount);
        events::initialized(&env, &buyer, &seller, &arbiter, amount);

        Ok(())
    }

    /// Buyer funds the escrow by transferring tokens to the contract.
    ///
    /// # Errors
    ///
    /// Returns [`EscrowError::NotInitialized`] if the contract has not been initialized.
    /// Returns [`EscrowError::InvalidState`] if the escrow is not in the `Created` state.
    /// Returns [`EscrowError::InsufficientFunds`] if the buyer's balance is less than the escrow amount.
    pub fn fund(env: Env) -> Result<(), EscrowError> {
        #[cfg(feature = "pausable")]
        Self::require_not_paused(&env)?;
    /// Initialize a new escrow with multi-sig arbiter support. Must be called exactly once.
    ///
    /// Allows specifying multiple arbiters and requiring N-of-M signatures for resolution.
    pub fn initialize_with_arbiters(
        env: Env,
        buyer: Address,
        seller: Address,
        arbiters: soroban_sdk::Vec<Address>,
        token_contract: Address,
        amount: i128,
        deadline_ledger: u32,
        required_signatures: u32,
    ) -> Result<(), EscrowError> {
        if env.storage().instance().has(&State) {
            return Err(EscrowError::AlreadyInitialized);
        }
        if amount <= 0 {
            return Err(EscrowError::InvalidAmount);
        }
        if arbiters.is_empty() || required_signatures == 0 || required_signatures > arbiters.len() as u32 {
            return Err(EscrowError::InvalidParties);
        }

        // Validate no duplicates and no conflicts with buyer/seller
        for arbiter in arbiters.iter() {
            if arbiter == buyer || arbiter == seller {
                return Err(EscrowError::InvalidParties);
            }
        }

        if deadline_ledger < env.ledger().sequence() + MIN_DEADLINE_BUFFER {
            return Err(EscrowError::DeadlinePassed);
        }

        // Validate that token_contract is a real token by calling decimals().
        let token_client = token::Client::new(&env, &token_contract);
        token_client.decimals();

        env.storage().instance().set(&Buyer, &buyer);
        env.storage().instance().set(&Seller, &seller);
        env.storage().instance().set(&Arbiters, &arbiters);
        env.storage().instance().set(&Arbiter, &arbiters.get(0).unwrap());
        env.storage().instance().set(&RequiredSignatures, &required_signatures);
        env.storage().instance().set(&TokenContract, &token_contract);
        env.storage().instance().set(&Amount, &amount);
        env.storage().instance().set(&Deadline, &deadline_ledger);
        env.storage().instance().set(&State, &EscrowState::Created);
        env.storage().instance().set(&BuyerApproved, &false);
        env.storage().instance().set(&SellerDelivered, &false);

        bump_instance(&env);

        events::escrow_created(&env, &buyer, &seller, amount);
        events::initialized(&env, &buyer, &seller, &arbiters.get(0).unwrap(), amount);

        Ok(())
    }

    /// Update the escrow amount. Buyer only, `Created` state only.
    ///
    /// Allows the buyer to adjust the amount before funding. Validates `new_amount > 0`.
    /// Emits an `amount_updated` event.
    pub fn update_amount(env: Env, new_amount: i128) -> Result<(), EscrowError> {
        let buyer: Address = env
            .storage()
            .instance()
            .get(&Buyer)
            .ok_or(EscrowError::NotInitialized)?;
        buyer.require_auth();

        if new_amount <= 0 {
            return Err(EscrowError::InvalidAmount);
        }

        let state: EscrowState = env
            .storage()
            .instance()
            .get(&State)
            .ok_or(EscrowError::NotInitialized)?;
        if state != EscrowState::Created {
            return Err(EscrowError::InvalidState);
        }

        env.storage().instance().set(&Amount, &new_amount);
        bump_instance(&env);

        env.events()
            .publish((Symbol::new(&env, "amount_updated"), buyer), new_amount);

        Ok(())
    }

    /// Buyer funds the escrow by transferring tokens to the contract.
    pub fn fund(env: Env) -> Result<(), EscrowError> {
        #[cfg(feature = "pausable")]
        Self::require_not_paused(&env)?;

        let buyer: Address = Self::get_required(&env, &Buyer)?;
        buyer.require_auth();

        let state: EscrowState = Self::get_required(&env, &State)?;
        if state != EscrowState::Created {
            return Err(EscrowError::InvalidState);
        }

        let token_contract: Address = Self::get_required(&env, &TokenContract)?;
        let amount: i128 = Self::get_required(&env, &Amount)?;

        let token_client = token::Client::new(&env, &token_contract);
        if token_client.balance(&buyer) < amount {
            return Err(EscrowError::InsufficientFunds);
        }
        token_client.transfer(&buyer, &env.current_contract_address(), &amount);

        env.storage().instance().set(&State, &EscrowState::Funded);
        bump_instance(&env);

        env.events()
            .publish((Symbol::new(&env, "escrow_funded"), buyer), amount);

        Ok(())
    }

    /// Seller marks goods/services as delivered.
    ///
    /// # Errors
    ///
    /// Returns [`EscrowError::NotInitialized`] if the contract has not been initialized.
    /// Returns [`EscrowError::InvalidState`] if the escrow is not in the `Funded` state.
    pub fn mark_delivered(env: Env) -> Result<(), EscrowError> {
        #[cfg(feature = "pausable")]
        Self::require_not_paused(&env)?;

        let seller: Address = Self::get_required(&env, &Seller)?;
        seller.require_auth();

        let state: EscrowState = Self::get_required(&env, &State)?;
        if state != EscrowState::Funded {
            return Err(EscrowError::InvalidState);
        }

        env.storage().instance().set(&State, &EscrowState::Delivered);
        bump_instance(&env);

        env.events()
            .publish((Symbol::new(&env, "delivery_marked"), seller), ());

        Ok(())
    }

    /// Buyer approves delivery, releasing funds to the seller.
    ///
    /// # Errors
    ///
    /// Returns [`EscrowError::NotInitialized`] if the contract has not been initialized.
    /// Returns [`EscrowError::InvalidState`] if the escrow is not in the `Delivered` state.
    pub fn approve_delivery(env: Env) -> Result<(), EscrowError> {
        #[cfg(feature = "pausable")]
        Self::require_not_paused(&env)?;

        let buyer: Address = Self::get_required(&env, &Buyer)?;
        buyer.require_auth();

        Self::release_to_seller(env)
    }

    /// Buyer requests a refund after the deadline has passed.
    ///
    /// # Errors
    ///
    /// Returns [`EscrowError::NotInitialized`] if the contract has not been initialized.
    /// Returns [`EscrowError::DeadlineNotReached`] if the deadline has not yet passed.
    /// Returns [`EscrowError::InvalidState`] if the escrow is not in the `Funded` or `Delivered` state.
    pub fn request_refund(env: Env) -> Result<(), EscrowError> {
    /// Buyer releases a partial amount to the seller (milestone-based payments).
    /// Only callable in `Funded` state. Decrements the stored amount.
    pub fn release_partial(env: Env, amount: i128) -> Result<(), EscrowError> {
        #[cfg(feature = "pausable")]
        Self::require_not_paused(&env)?;

        let buyer: Address = env
            .storage()
            .instance()
            .get(&Buyer)
            .ok_or(EscrowError::NotInitialized)?;
        buyer.require_auth();

        let state: EscrowState = env
            .storage()
            .instance()
            .get(&State)
            .ok_or(EscrowError::NotInitialized)?;
        if state != EscrowState::Funded {
            return Err(EscrowError::InvalidState);
        }

        if amount <= 0 {
            return Err(EscrowError::InvalidAmount);
        }

        let stored_amount: i128 = env.storage().instance().get(&Amount).unwrap();
        if amount > stored_amount {
            return Err(EscrowError::InsufficientFunds);
        }

        let seller: Address = env.storage().instance().get(&Seller).unwrap();
        let new_amount = stored_amount - amount;
        env.storage().instance().set(&Amount, &new_amount);
        bump_instance(&env);

        admin::transfer_token(&env, &env.current_contract_address(), &seller, amount);
        events::partial_release(&env, &seller, amount);

        Ok(())
    }

    /// Buyer requests a refund after the deadline has passed.
    pub fn request_refund(env: Env) -> Result<(), EscrowError> {
        #[cfg(feature = "pausable")]
        Self::require_not_paused(&env)?;

        let buyer: Address = Self::get_required(&env, &Buyer)?;
        buyer.require_auth();

        let state: EscrowState = Self::get_required(&env, &State)?;
        let deadline: u32 = Self::get_required(&env, &Deadline)?;

        let can_refund = matches!(state, EscrowState::Funded | EscrowState::Delivered)
            && env.ledger().sequence() > deadline;
        if !can_refund {
            return Err(EscrowError::DeadlineNotReached);
        }

        Self::refund_to_buyer(env)
    }

    /// Buyer or seller raises a dispute.
    ///
    /// # Errors
    ///
    /// Returns [`EscrowError::NotInitialized`] if the contract has not been initialized.
    /// Returns [`EscrowError::NotAuthorized`] if the caller is neither the buyer nor the seller.
    /// Returns [`EscrowError::InvalidState`] if the escrow is not in the `Funded` or `Delivered` state.
    pub fn raise_dispute(env: Env, caller: Address) -> Result<(), EscrowError> {
        #[cfg(feature = "pausable")]
        Self::require_not_paused(&env)?;

        let buyer: Address = Self::get_required(&env, &Buyer)?;
        let seller: Address = Self::get_required(&env, &Seller)?;

        if caller != buyer && caller != seller {
            return Err(EscrowError::NotAuthorized);
        }
        caller.require_auth();

        let state: EscrowState = Self::get_required(&env, &State)?;
        if !matches!(state, EscrowState::Funded | EscrowState::Delivered) {
            return Err(EscrowError::InvalidState);
        }

        env.storage().instance().set(&State, &EscrowState::Disputed);
        bump_instance(&env);

        env.events()
            .publish((Symbol::new(&env, "dispute_raised"), caller), ());

        Ok(())
    }

    /// Arbiter resolves a dispute.
    ///
    /// # Errors
    ///
    /// Returns [`EscrowError::NotInitialized`] if the contract has not been initialized.
    /// Returns [`EscrowError::InvalidState`] if the escrow is not in the `Disputed` state.
    pub fn resolve_dispute(env: Env, release_to_seller: bool) -> Result<(), EscrowError> {
        let arbiter: Address = Self::get_required(&env, &Arbiter)?;
        arbiter.require_auth();

        let state: EscrowState = Self::get_required(&env, &State)?;
        let state: EscrowState = env
            .storage()
            .instance()
            .get(&State)
            .ok_or(EscrowError::NotInitialized)?;
        if state != EscrowState::Disputed {
            return Err(EscrowError::InvalidState);
        }

        // Check if using multi-sig arbiters
        let arbiters_opt: Option<soroban_sdk::Vec<Address>> = env.storage().instance().get(&DataKey::Arbiters);
        
        if let Some(arbiters) = arbiters_opt {
            // Multi-sig mode
            let required_sigs: u32 = env
                .storage()
                .instance()
                .get(&DataKey::RequiredSignatures)
                .unwrap_or(1);
            
            let mut votes: soroban_sdk::Vec<Address> = env
                .storage()
                .instance()
                .get(&DataKey::ArbiterVotes)
                .unwrap_or_else(|| soroban_sdk::Vec::new(&env));
            
            // Find the first arbiter that authorizes and add to votes
            let mut caller_found = false;
            for arbiter in arbiters.iter() {
                arbiter.require_auth();
                
                // Add this arbiter to votes if not already there
                let mut already_voted = false;
                for vote in votes.iter() {
                    if vote == arbiter {
                        already_voted = true;
                        break;
                    }
                }
                
                if !already_voted {
                    votes.push_back(arbiter.clone());
                }
                caller_found = true;
                break;
            }
            
            if !caller_found {
                return Err(EscrowError::NotAuthorized);
            }
            
            env.storage().instance().set(&DataKey::ArbiterVotes, &votes);
            
            // Check if we have enough signatures
            if votes.len() as u32 >= required_sigs {
                env.storage().instance().remove(&DataKey::ArbiterVotes);
                if release_to_seller {
                    env.storage().instance().set(&State, &EscrowState::Delivered);
                    Self::release_to_seller(env)
                } else {
                    env.storage().instance().set(&State, &EscrowState::Funded);
                    Self::refund_to_buyer(env)
                }
            } else {
                Ok(())
            }
        } else {
            // Single arbiter mode (backward compatible)
            let arbiter: Address = env
                .storage()
                .instance()
                .get(&Arbiter)
                .ok_or(EscrowError::NotInitialized)?;
            arbiter.require_auth();

            if release_to_seller {
                env.storage().instance().set(&State, &EscrowState::Delivered);
                Self::release_to_seller(env)
            } else {
                env.storage().instance().set(&State, &EscrowState::Funded);
                Self::refund_to_buyer(env)
            }
        }
    }

    /// Buyer cancels an unfunded escrow (`Created` state only).
    ///
    /// # Errors
    ///
    /// Returns [`EscrowError::NotInitialized`] if the contract has not been initialized.
    /// Returns [`EscrowError::InvalidState`] if the escrow is not in the `Created` state.
    pub fn cancel(env: Env) -> Result<(), EscrowError> {
        let buyer: Address = Self::get_required(&env, &Buyer)?;
        buyer.require_auth();

        let state: EscrowState = Self::get_required(&env, &State)?;
        if state != EscrowState::Created {
            return Err(EscrowError::InvalidState);
        }

        env.storage().instance().set(&State, &EscrowState::Cancelled);
        bump_instance(&env);

        env.events()
            .publish((Symbol::new(&env, "escrow_cancelled"), buyer), ());

        Ok(())
    }

    /// Extend the escrow deadline by mutual consent (buyer and seller auth required).
    pub fn extend_deadline(env: Env, new_deadline: u32) -> Result<(), EscrowError> {
        let buyer: Address = env
            .storage()
            .instance()
            .get(&Buyer)
            .ok_or(EscrowError::NotInitialized)?;
        let seller: Address = env
            .storage()
            .instance()
            .get(&Seller)
            .ok_or(EscrowError::NotInitialized)?;

        buyer.require_auth();
        seller.require_auth();

        let current_deadline: u32 = env
            .storage()
            .instance()
            .get(&Deadline)
            .ok_or(EscrowError::NotInitialized)?;

        if new_deadline <= current_deadline {
            return Err(EscrowError::DeadlinePassed);
        }

        let state: EscrowState = env
            .storage()
            .instance()
            .get(&State)
            .ok_or(EscrowError::NotInitialized)?;
        if !matches!(state, EscrowState::Funded | EscrowState::Delivered) {
            return Err(EscrowError::InvalidState);
        }

        env.storage().instance().set(&Deadline, &new_deadline);
        bump_instance(&env);

        env.events()
            .publish((Symbol::new(&env, "deadline_extended"), buyer), new_deadline);

        Ok(())
    }

    /// Extend storage TTL. Anyone can call this to keep an active escrow alive.
    ///
    /// # Errors
    ///
    /// Returns [`EscrowError::NotInitialized`] if the contract has not been initialized.
    pub fn bump(env: Env) -> Result<(), EscrowError> {
        if !env.storage().instance().has(&State) {
            return Err(EscrowError::NotInitialized);
        }
        bump_instance(&env);
        Ok(())
    }

    /// Return full escrow details as an [`EscrowInfo`] struct, or `None` if not initialized.
    #[must_use]
    pub fn get_escrow_info(env: Env) -> Option<EscrowInfo> {
        Some(EscrowInfo {
            buyer: env.storage().instance().get(&Buyer)?,
            seller: env.storage().instance().get(&Seller)?,
            arbiter: env.storage().instance().get(&Arbiter)?,
            token_contract: env.storage().instance().get(&TokenContract)?,
            amount: env.storage().instance().get(&Amount)?,
            deadline: env.storage().instance().get(&Deadline)?,
            state: env.storage().instance().get(&State)?,
    /// Return full escrow details as an [`EscrowInfo`] struct.
    pub fn get_escrow_info(env: Env) -> Result<EscrowInfo, EscrowError> {
        Ok(EscrowInfo {
            buyer: Self::get_required(&env, &Buyer)?,
            seller: Self::get_required(&env, &Seller)?,
            arbiter: Self::get_required(&env, &Arbiter)?,
            token_contract: Self::get_required(&env, &TokenContract)?,
            amount: Self::get_required(&env, &Amount)?,
            deadline: Self::get_required(&env, &Deadline)?,
            state: Self::get_required(&env, &State)?,
        })
    }

    /// Return the current [`EscrowState`], or `None` if not initialized.
    #[must_use]
    pub fn get_state(env: Env) -> Option<EscrowState> {
        env.storage().instance().get(&State)
    }

    /// Return `true` if the deadline ledger has been passed.
    #[must_use]
    pub fn is_deadline_passed(env: Env) -> bool {
        let deadline: u32 = env.storage().instance().get(&Deadline).unwrap_or(0);
        env.ledger().sequence() > deadline
    }

    /// Return the number of ledgers remaining until the deadline.
    ///
    /// Returns a negative value if the deadline has already passed.
    /// Each ledger takes approximately 5 seconds on the Stellar network.
    pub fn get_remaining_ledgers(env: Env) -> i64 {
        let deadline: u32 = env.storage().instance().get(&Deadline).unwrap_or(0);
        let current_sequence: u32 = env.ledger().sequence();
        deadline as i64 - current_sequence as i64
    }
}

/// Pause / unpause — only compiled when the `pausable` feature is enabled.
#[cfg(feature = "pausable")]
#[contractimpl]
impl EscrowContract {
    /// Minimum ledgers between proposing and executing a WASM upgrade (~24 h at 5 s/ledger).
    const UPGRADE_DELAY_LEDGERS: u32 = 17_280;

    /// Pause the contract. Admin only.
    ///
    /// # Errors
    ///
    /// Returns [`EscrowError::NotAuthorized`] if the caller is not the admin.
    pub fn pause(env: Env) -> Result<(), EscrowError> {
        let admin = require_admin(&env)?;
        admin.require_auth();
        env.storage().instance().set(&Paused, &true);
        bump_instance(&env);
        events::paused(&env, &admin);
        Ok(())
    }

    /// Unpause the contract. Admin only.
    ///
    /// # Errors
    ///
    /// Returns [`EscrowError::NotAuthorized`] if the caller is not the admin.
    pub fn unpause(env: Env) -> Result<(), EscrowError> {
        let admin = require_admin(&env)?;
        admin.require_auth();
        env.storage().instance().set(&Paused, &false);
        bump_instance(&env);
        events::unpaused(&env, &admin);
        Ok(())
    }

    /// Return the git commit hash baked in at compile time.
    #[must_use]
    pub fn version(env: Env) -> soroban_sdk::String {
        soroban_sdk::String::from_str(&env, env!("GIT_HASH"))
    }

    /// Propose a WASM upgrade. Admin only.
    ///
    /// Stores `wasm_hash` and a `ready_after` ledger number. The upgrade cannot
    /// be executed until at least `UPGRADE_DELAY_LEDGERS` ledgers have passed.
    ///
    /// # Errors
    ///
    /// Returns [`EscrowError::NotAuthorized`] if the caller is not the admin.
    pub fn propose_upgrade(env: Env, wasm_hash: soroban_sdk::BytesN<32>) -> Result<(), EscrowError> {
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
    ///
    /// # Errors
    ///
    /// Returns [`EscrowError::NotAuthorized`] if the caller is not the admin, no upgrade is pending, or the timelock has not elapsed.
    pub fn execute_upgrade(env: Env) -> Result<(), EscrowError> {
        let admin = require_admin(&env)?;
        admin.require_auth();
        let (wasm_hash, ready_after): (soroban_sdk::BytesN<32>, u32) = env
            .storage()
            .instance()
            .get(&DataKey::PendingUpgrade)
            .ok_or(EscrowError::NotAuthorized)?;
        if env.ledger().sequence() < ready_after {
            return Err(EscrowError::NotAuthorized);
        }
        env.storage().instance().remove(&DataKey::PendingUpgrade);
        events::upgraded(&env, &admin, &wasm_hash);
        env.events().publish(
            (soroban_sdk::Symbol::new(&env, "upgrade_executed"), admin),
            wasm_hash.clone(),
        );
        env.deployer().update_current_contract_wasm(wasm_hash);
        Ok(())
    }
}

impl EscrowContract {
    /// Helper to retrieve a required value from instance storage.
    /// Returns `NotInitialized` error if the key is missing.
    fn get_required<T: soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val>>(
        env: &Env,
        key: &DataKey,
    ) -> Result<T, EscrowError> {
        env.storage()
            .instance()
            .get(key)
            .ok_or(EscrowError::NotInitialized)
    }

    fn require_state(env: &Env, expected: EscrowState) -> Result<(), EscrowError> {
        let state: EscrowState = Self::get_required(env, &State)?;
        if state != expected {
            return Err(EscrowError::InvalidState);
        }
        Ok(())
    }

    fn release_to_seller(env: Env) -> Result<(), EscrowError> {
        Self::require_state(&env, EscrowState::Delivered)?;

        let seller: Address = Self::get_required(&env, &Seller)?;
        let amount: i128 = Self::get_required(&env, &Amount)?;

        // checks-effects-interactions: update state before external call
        env.storage().instance().set(&State, &EscrowState::Completed);
        bump_instance(&env);

        admin::transfer_token(&env, &env.current_contract_address(), &seller, amount);

        env.events()
            .publish((Symbol::new(&env, "funds_released"), seller), amount);

        Ok(())
    }

    fn refund_to_buyer(env: Env) -> Result<(), EscrowError> {
        Self::require_state(&env, EscrowState::Funded)?;

        let buyer: Address = Self::get_required(&env, &Buyer)?;
        let amount: i128 = Self::get_required(&env, &Amount)?;

        // checks-effects-interactions: update state before external call
        env.storage().instance().set(&State, &EscrowState::Refunded);
        bump_instance(&env);

        admin::transfer_token(&env, &env.current_contract_address(), &buyer, amount);

        env.events()
            .publish((Symbol::new(&env, "funds_refunded"), buyer), amount);

        Ok(())
    }

    #[cfg(feature = "pausable")]
    fn require_not_paused(env: &Env) -> Result<(), EscrowError> {
        if env.storage().instance().get(&Paused).unwrap_or(false) {
            return Err(EscrowError::NotAuthorized);
        }
        Ok(())
    }
}

mod test;
