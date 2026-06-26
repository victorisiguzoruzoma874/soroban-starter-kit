use soroban_sdk::{Address, Env};

pub fn initialized(env: &Env, admin: &Address, token: &Address) {
    let topics = (Symbol::new(env, "initialized"),);
    env.events().publish(topics, (admin.clone(), token.clone()));
}

pub fn wrapped(env: &Env, user: &Address, amount: i128, total: i128) {
    let topics = (Symbol::new(env, "wrapped"),);
    env.events().publish(topics, (user.clone(), amount, total));
}

pub fn unwrapped(env: &Env, user: &Address, amount: i128, total: i128) {
    let topics = (Symbol::new(env, "unwrapped"),);
    env.events().publish(topics, (user.clone(), amount, total));
}

use soroban_sdk::Symbol;
