#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::{Address as _, Ledger as _}, Address, Env, FromVal, String};

fn create_token_contract<'a>(env: &Env) -> (TokenContractClient<'a>, Address) {
    let contract_address = env.register_contract(None, TokenContract);
    let client = TokenContractClient::new(env, &contract_address);
    (client, contract_address)
}

fn init_token<'a>(env: &'a Env, admin: &Address) -> TokenContractClient<'a> {
    let (client, _) = create_token_contract(env);
    client.initialize(
        admin,
        &String::from_str(env, "Test Token"),
        &String::from_str(env, "TEST"),
        &18u32,
        &None,
    );
    client
}

#[test]
fn test_initialize() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let (client, contract_address) = create_token_contract(&env);
    let name = String::from_str(&env, "Test Token");
    let symbol = String::from_str(&env, "TEST");
    let decimals = 18u32;
    client.initialize(&admin, &name, &symbol, &decimals, &None);

    assert_eq!(client.admin(), admin);
    assert_eq!(client.name(), name.clone());
    assert_eq!(client.symbol(), symbol.clone());
    assert_eq!(client.decimals(), decimals);
    assert_eq!(client.total_supply(), 0i128);

    // Verify initialized event was emitted
    use soroban_sdk::{testutils::Events as _, IntoVal, Symbol};
    assert_eq!(
        env.events().all(),
        soroban_sdk::vec![
            &env,
            (
                contract_address.clone(),
                (Symbol::new(&env, "initialized"), admin.clone()).into_val(&env),
                (name, symbol, decimals).into_val(&env),
            ),
        ]
    );
}

#[test]
#[should_panic(expected = "Error(Contract, #4)")]
fn test_initialize_twice() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let (client, _) = create_token_contract(&env);
    client.initialize(
        &admin,
        &String::from_str(&env, "Test Token"),
        &String::from_str(&env, "TEST"),
        &18u32,
        &None,
    );
    client.initialize(
        &admin,
        &String::from_str(&env, "Test Token"),
        &String::from_str(&env, "TEST"),
        &18u32,
        &None,
    );
}

#[test]
fn test_mint() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let client = init_token(&env, &admin);
    client.mint(&user, &1000i128);
    assert_eq!(client.balance(&user), 1000i128);
    assert_eq!(client.total_supply(), 1000i128);
}

#[test]
fn test_total_supply() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);
    let client = init_token(&env, &admin);

    assert_eq!(client.total_supply(), 0i128);

    client.mint(&user1, &500i128);
    assert_eq!(client.total_supply(), 500i128);

    client.mint(&user2, &300i128);
    assert_eq!(client.total_supply(), 800i128);

    client.burn(&user1, &200i128);
    assert_eq!(client.total_supply(), 600i128);
}

#[test]
fn test_burn() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let client = init_token(&env, &admin);
    client.mint(&user, &1000i128);
    client.burn(&user, &300i128);
    assert_eq!(client.balance(&user), 700i128);
    assert_eq!(client.total_supply(), 700i128);
}

#[test]
fn test_transfer() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);
    let client = init_token(&env, &admin);
    client.mint(&user1, &1000i128);
    client.transfer(&user1, &user2, &300i128);
    assert_eq!(client.balance(&user1), 700i128);
    assert_eq!(client.balance(&user2), 300i128);
    assert_eq!(client.total_supply(), 1000i128);
}

#[test]
fn test_approve() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);
    let spender = Address::generate(&env);
    let client = init_token(&env, &admin);
    client.mint(&user1, &1000i128);
    let expiration = env.ledger().sequence() + 100;
    client.approve(&user1, &spender, &500i128, &expiration);
    assert_eq!(client.allowance(&user1, &spender), 500i128);
    client.transfer_from(&spender, &user1, &user2, &200i128);
    assert_eq!(client.balance(&user1), 800i128);
    assert_eq!(client.balance(&user2), 200i128);
    assert_eq!(client.allowance(&user1, &spender), 300i128);
}

#[test]
#[should_panic(expected = "Error(Contract, #2)")]
fn test_expired_allowance() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);
    let spender = Address::generate(&env);
    let client = init_token(&env, &admin);
    client.mint(&user1, &1000i128);
    // approve with expiration in the past (sequence 0, expiration 0 means already expired)
    let expiration = env.ledger().sequence() + 10;
    client.approve(&user1, &spender, &500i128, &expiration);
    // advance ledger past expiration
    env.ledger().set(soroban_sdk::testutils::LedgerInfo {
        timestamp: 0,
        protocol_version: 22,
        sequence_number: expiration + 1,
        network_id: Default::default(),
        base_reserve: 10,
        min_temp_entry_ttl: 1,
        min_persistent_entry_ttl: 1,
        max_entry_ttl: 6_312_000,
    });
    // should panic with InsufficientAllowance (#2) since allowance is expired
    client.transfer_from(&spender, &user1, &user2, &200i128);
}

#[test]
#[should_panic(expected = "Error(Contract, #6)")]
fn test_mint_zero_amount() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let client = init_token(&env, &admin);
    client.mint(&user, &0i128);
}

#[test]
fn test_set_admin() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let new_admin = Address::generate(&env);
    let (client, contract_address) = create_token_contract(&env);
    client.initialize(
        &admin,
        &String::from_str(&env, "Test Token"),
        &String::from_str(&env, "TEST"),
        &18u32,
        &None,
    );

    client.set_admin(&new_admin);

    // Admin must be updated in storage
    assert_eq!(client.admin(), new_admin);

    // Verify admin_changed event was emitted with old_admin as topic and new_admin as data
    use soroban_sdk::{testutils::Events as _, IntoVal, Symbol};
    let all_events = env.events().all();
    let n = all_events.len();
    assert!(n > 0);
    let expected = soroban_sdk::vec![
        &env,
        (
            contract_address.clone(),
            (Symbol::new(&env, "admin_changed"), admin.clone()).into_val(&env),
            new_admin.clone().into_val(&env),
        ),
    ];
    assert_eq!(all_events.slice(n - 1..), expected);
    let last = all_events.last().unwrap();
    let (addr, topics, data) = last;
    assert_eq!(addr, contract_address);
    assert_eq!(
        topics,
        (Symbol::new(&env, "admin_changed"), admin.clone()).into_val(&env)
    );
    let emitted_new_admin = Address::from_val(&env, &data);
    assert_eq!(emitted_new_admin, new_admin);
}

#[test]
#[should_panic]
fn test_unauthorized_set_admin_fails() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let attacker = Address::generate(&env);
    let new_admin = Address::generate(&env);
    let (client, _) = create_token_contract(&env);
    env.mock_all_auths();
    client.initialize(
        &admin,
        &String::from_str(&env, "Test Token"),
        &String::from_str(&env, "TEST"),
        &18u32,
        &None,
    );
    // clear mocked auths so the next call is not authorized
    env.set_auths(&[]);
    // attacker tries to set admin without authorization — should panic
    client.set_admin(&new_admin);
    let _ = attacker;
}

#[test]
fn test_approve_revoke() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let spender = Address::generate(&env);
    let (client, contract_address) = create_token_contract(&env);
    client.initialize(
        &admin,
        &String::from_str(&env, "Test Token"),
        &String::from_str(&env, "TEST"),
        &18u32,
        &None,
    );
    client.mint(&user, &1000i128);

    // Set a normal allowance first
    let expiration = env.ledger().sequence() + 100;
    client.approve(&user, &spender, &500i128, &expiration);
    assert_eq!(client.allowance(&user, &spender), 500i128);

    // Revoke by approving with amount == 0 — must emit revoke, not approve
    use soroban_sdk::{testutils::Events as _, IntoVal, Symbol};
    client.approve(&user, &spender, &0i128, &expiration);
    assert_eq!(client.allowance(&user, &spender), 0i128);

    // The last event must be revoke, not approve
    let all_events = env.events().all();
    let n = all_events.len();
    assert!(n > 0);
    let expected = soroban_sdk::vec![
        &env,
        (
            contract_address.clone(),
            (Symbol::new(&env, "revoke"), user.clone(), spender.clone()).into_val(&env),
            ().into_val(&env),
        ),
    ];
    assert_eq!(all_events.slice(n - 1..), expected);
    let last = all_events.last().unwrap();
    let (addr, topics, data) = last;
    assert_eq!(addr, contract_address);
    assert_eq!(
        topics,
        (Symbol::new(&env, "revoke"), user.clone(), spender.clone()).into_val(&env)
    );
    assert!(data.is_void());
}

#[test]
fn test_transfer_self_is_noop() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let client = init_token(&env, &admin);
    client.mint(&user, &500i128);

    client.transfer(&user, &user, &200i128);

    // Balance unchanged
    assert_eq!(client.balance(&user), 500i128);
}

#[test]
fn test_balance_of_distinguishes_unknown_from_zero() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let unknown = Address::generate(&env);
    let client = init_token(&env, &admin);

    // Unknown address has no storage entry
    assert_eq!(client.balance_of(&unknown), None);

    // After minting and burning to zero, entry exists with value 0
    client.mint(&user, &100i128);
    client.burn(&user, &100i128);
    assert_eq!(client.balance_of(&user), Some(0i128));

    // balance() returns 0 for both — indistinguishable
    assert_eq!(client.balance(&unknown), 0i128);
    assert_eq!(client.balance(&user), 0i128);
}
