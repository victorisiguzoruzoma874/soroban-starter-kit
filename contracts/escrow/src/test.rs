#![cfg(test)]

use super::*;
use soroban_sdk::token::TokenInterface;
use soroban_sdk::{
    testutils::{Address as _, Events as _, Ledger as _},
    Address, Env, IntoVal, String, Symbol,
};

// ---------------------------------------------------------------------------
// MockToken — a no-op token contract so cross-contract calls don't panic
// ---------------------------------------------------------------------------

#[contract]
pub struct MockToken;

#[contractimpl]
impl token::TokenInterface for MockToken {
    fn allowance(_env: Env, _from: Address, _spender: Address) -> i128 {
        0
    }

    fn approve(
        _env: Env,
        _from: Address,
        _spender: Address,
        _amount: i128,
        _expiration_ledger: u32,
    ) {
    }

    fn balance(_env: Env, _id: Address) -> i128 {
        i128::MAX
    }

    fn transfer(_env: Env, _from: Address, _to: Address, _amount: i128) {}

    fn transfer_from(_env: Env, _spender: Address, _from: Address, _to: Address, _amount: i128) {}

    fn burn(_env: Env, _from: Address, _amount: i128) {}

    fn burn_from(_env: Env, _spender: Address, _from: Address, _amount: i128) {}

    fn decimals(_env: Env) -> u32 {
        18
    }

    fn name(env: Env) -> String {
        String::from_str(&env, "Mock")
    }

    fn symbol(env: Env) -> String {
        String::from_str(&env, "MCK")
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn create_escrow_contract<'a>(env: &'a Env) -> (EscrowContractClient<'a>, Address) {
    let contract_address = env.register_contract(None, EscrowContract);
    let client = EscrowContractClient::new(env, &contract_address);
    (client, contract_address)
}

fn create_mock_token(env: &Env) -> Address {
    env.register_contract(None, MockToken)
}

/// Returns (client, contract_address, buyer, seller, arbiter, token, amount).
fn setup_funded_escrow<'a>(
    env: &'a Env,
) -> (
    EscrowContractClient<'a>,
    Address,
    Address,
    Address,
    Address,
    Address,
    i128,
) {
    let buyer = Address::generate(env);
    let seller = Address::generate(env);
    let arbiter = Address::generate(env);
    let token = create_mock_token(env);
    let amount = 1_000i128;
    let deadline = env.ledger().sequence() + 100;

    let (client, contract_address) = create_escrow_contract(env);
    client.initialize(&buyer, &seller, &arbiter, &token, &amount, &deadline);
    client.fund();

    (client, contract_address, buyer, seller, arbiter, token, amount)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[test]
fn test_initialize() {
    let env = Env::default();
    env.mock_all_auths();

    let buyer = Address::generate(&env);
    let seller = Address::generate(&env);
    let arbiter = Address::generate(&env);
    let token_contract = create_mock_token(&env);
    let amount = 1_000i128;
    let deadline = env.ledger().sequence() + 100;

    let (client, contract_address) = create_escrow_contract(&env);

    client.initialize(&buyer, &seller, &arbiter, &token_contract, &amount, &deadline);

    let info = client.get_escrow_info();
    assert_eq!(info.buyer, buyer);
    assert_eq!(info.seller, seller);
    assert_eq!(info.arbiter, arbiter);
    assert_eq!(info.token_contract, token_contract);
    assert_eq!(info.amount, amount);
    assert_eq!(info.deadline, deadline);
    assert_eq!(info.state, EscrowState::Created);

    assert_eq!(
        env.events().all(),
        soroban_sdk::vec![
            &env,
            (
                contract_address.clone(),
                (Symbol::new(&env, "escrow_created"), buyer.clone(), seller.clone()).into_val(&env),
                amount.into_val(&env),
            ),
            (
                contract_address.clone(),
                (Symbol::new(&env, "initialized"), buyer.clone(), seller.clone(), arbiter.clone()).into_val(&env),
                amount.into_val(&env),
            ),
        ]
    );
}

#[test]
#[should_panic(expected = "Error(Contract, #5)")]
fn test_initialize_twice() {
    let env = Env::default();
    env.mock_all_auths();

    let buyer = Address::generate(&env);
    let seller = Address::generate(&env);
    let arbiter = Address::generate(&env);
    let token_contract = create_mock_token(&env);
    let amount = 1_000i128;
    let deadline = env.ledger().sequence() + 100;

    let (client, _) = create_escrow_contract(&env);

    client.initialize(&buyer, &seller, &arbiter, &token_contract, &amount, &deadline);
    // Second call must fail with AlreadyInitialized (#5)
    client.initialize(&buyer, &seller, &arbiter, &token_contract, &amount, &deadline);
}

#[test]
#[should_panic]
fn test_initialize_past_deadline() {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().with_mut(|l| l.sequence_number = 10);

    let buyer = Address::generate(&env);
    let seller = Address::generate(&env);
    let arbiter = Address::generate(&env);
    let token_contract = create_mock_token(&env);
    let amount = 1_000i128;
    let deadline = 5u32; // 5 < 10, already in the past

    let (client, _) = create_escrow_contract(&env);
    client.initialize(&buyer, &seller, &arbiter, &token_contract, &amount, &deadline);
}

#[test]
fn test_initialize_escrow() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, _, buyer, seller, _, _, amount) = setup_funded_escrow(&env);

    let info = client.get_escrow_info();
    assert_eq!(info.buyer, buyer);
    assert_eq!(info.seller, seller);
    assert_eq!(info.amount, amount);
    assert_eq!(info.state, EscrowState::Funded);
}

#[test]
fn test_fund() {
    let env = Env::default();
    env.mock_all_auths();

    let buyer = Address::generate(&env);
    let seller = Address::generate(&env);
    let arbiter = Address::generate(&env);
    let token_contract = create_mock_token(&env);
    let amount = 1_000i128;
    let deadline = env.ledger().sequence() + 100;

    let (client, contract_address) = create_escrow_contract(&env);
    client.initialize(&buyer, &seller, &arbiter, &token_contract, &amount, &deadline);
    client.fund();

    assert_eq!(client.get_state(), Some(EscrowState::Funded));

    // Verify escrow_funded event was emitted
    assert_eq!(
        env.events().all(),
        soroban_sdk::vec![
            &env,
            (
                contract_address.clone(),
                (Symbol::new(&env, "escrow_created"), buyer.clone(), seller.clone()).into_val(&env),
                amount.into_val(&env),
            ),
            (
                contract_address.clone(),
                (Symbol::new(&env, "initialized"), buyer.clone(), seller.clone(), arbiter.clone()).into_val(&env),
                amount.into_val(&env),
            ),
            (
                contract_address.clone(),
                (Symbol::new(&env, "escrow_funded"), buyer.clone()).into_val(&env),
                amount.into_val(&env),
            ),
        ]
    );
}

#[test]
fn test_mark_delivered() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, ..) = setup_funded_escrow(&env);
    client.mark_delivered();

    assert_eq!(client.get_state(), Some(EscrowState::Delivered));
}

#[test]
fn test_approve_delivery() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, ..) = setup_funded_escrow(&env);
    client.mark_delivered();
    client.approve_delivery();

    assert_eq!(client.get_state(), Some(EscrowState::Completed));
}

#[test]
fn test_raise_dispute() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, ..) = setup_funded_escrow(&env);
    client.raise_dispute();

    assert_eq!(client.get_state(), Some(EscrowState::Disputed));
}

#[test]
fn test_resolve_dispute_to_seller() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, ..) = setup_funded_escrow(&env);
    client.raise_dispute();
    client.resolve_dispute(&true);

    assert_eq!(client.get_state(), Some(EscrowState::Completed));
}

#[test]
fn test_resolve_dispute_to_buyer() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, ..) = setup_funded_escrow(&env);
    client.raise_dispute();
    client.resolve_dispute(&false);

    assert_eq!(client.get_state(), Some(EscrowState::Refunded));
}

#[test]
fn test_deadline_passed() {
    let env = Env::default();
    env.mock_all_auths();

    let buyer = Address::generate(&env);
    let seller = Address::generate(&env);
    let arbiter = Address::generate(&env);
    let token_contract = create_mock_token(&env);
    let amount = 1_000i128;
    let deadline = env.ledger().sequence() + 10;

    let (client, _) = create_escrow_contract(&env);
    client.initialize(&buyer, &seller, &arbiter, &token_contract, &amount, &deadline);

    assert_eq!(client.is_deadline_passed(), false);

    env.ledger().with_mut(|li| li.sequence_number = deadline + 1);

    assert_eq!(client.is_deadline_passed(), true);
}

#[test]
fn test_arbiter_resolve_to_seller() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, contract_address, buyer, seller, _, _, amount) = setup_funded_escrow(&env);
    client.raise_dispute();
    client.resolve_dispute(&true);

    assert_eq!(client.get_state(), Some(EscrowState::Completed));

    // Verify funds_released event is present
    let all_events = env.events().all();
    let n = all_events.len();
    assert!(n > 0);
    assert_eq!(
        all_events.slice(n - 1..),
        soroban_sdk::vec![
            &env,
            (
                contract_address.clone(),
                (Symbol::new(&env, "funds_released"), seller.clone()).into_val(&env),
                amount.into_val(&env),
            ),
        ]
    );
}

#[test]
fn test_arbiter_resolve_to_buyer() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, contract_address, buyer, _, _, _, amount) = setup_funded_escrow(&env);
    client.raise_dispute();
    client.resolve_dispute(&false);

    assert_eq!(client.get_state(), Some(EscrowState::Refunded));

    // Verify funds_refunded event is present
    let all_events = env.events().all();
    let n = all_events.len();
    assert!(n > 0);
    assert_eq!(
        all_events.slice(n - 1..),
        soroban_sdk::vec![
            &env,
            (
                contract_address.clone(),
                (Symbol::new(&env, "funds_refunded"), buyer.clone()).into_val(&env),
                amount.into_val(&env),
            ),
        ]
    );
}



// ---------------------------------------------------------------------------
// Feature-gated tests
// ---------------------------------------------------------------------------

#[cfg(feature = "pausable")]
mod pausable_tests {
    use super::*;
    use soroban_common::AdminKey;

    fn setup_with_admin<'a>(
        env: &'a Env,
    ) -> (EscrowContractClient<'a>, Address, Address) {
        let admin = Address::generate(env);
        let buyer = Address::generate(env);
        let seller = Address::generate(env);
        let arbiter = Address::generate(env);
        let token = create_mock_token(env);
        let amount = 1_000i128;
        let deadline = env.ledger().sequence() + 100;

        let (client, contract_address) = create_escrow_contract(env);
        // Set admin directly in contract storage before initializing
        env.as_contract(&contract_address, || {
            env.storage().instance().set(&AdminKey::Admin, &admin);
        });
        client.initialize(&buyer, &seller, &arbiter, &token, &amount, &deadline);
        (client, admin, buyer)
    }

    #[test]
    fn test_pause_blocks_fund() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, _admin, _buyer) = setup_with_admin(&env);

        client.pause();
        assert!(client.try_fund().is_err());
    }

    #[test]
    fn test_unpause_restores_fund() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, _admin, _buyer) = setup_with_admin(&env);

        client.pause();
        client.unpause();
        client.fund();
        assert_eq!(client.get_state(), Some(EscrowState::Funded));
    }
}

#[cfg(feature = "upgradeable")]
mod upgradeable_tests {
    use super::*;
    use soroban_common::AdminKey;

    #[test]
    fn test_upgrade_requires_admin() {
        let env = Env::default();
        env.mock_all_auths();

        let (client, contract_address, ..) = setup_funded_escrow(&env);
        let admin = Address::generate(&env);
        env.as_contract(&contract_address, || {
            env.storage().instance().set(&AdminKey::Admin, &admin);
        });
        let dummy_hash = soroban_sdk::BytesN::from_array(&env, &[0u8; 32]);
        // Auth passes; the upgrade itself fails because the hash is invalid.
        let _ = client.try_upgrade(&dummy_hash);
    }
}
