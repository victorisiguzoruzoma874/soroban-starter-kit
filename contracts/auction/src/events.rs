use soroban_sdk::{Address, Env, Symbol};

pub fn started(env: &Env, seller: &Address, start_price: i128, deadline: u32) {
    env.events().publish(
        (Symbol::new(env, "started"), seller.clone()),
        (start_price, deadline),
    );
}

pub fn bid_placed(env: &Env, bidder: &Address, amount: i128) {
    env.events()
        .publish((Symbol::new(env, "bid_placed"), bidder.clone()), amount);
}

pub fn ended(env: &Env, winner: &Address, amount: i128) {
    env.events()
        .publish((Symbol::new(env, "ended"), winner.clone()), amount);
}

pub fn ended_no_bids(env: &Env) {
    env.events()
        .publish((Symbol::new(env, "ended_no_bids"),), ());
}

pub fn withdrawn(env: &Env, bidder: &Address, amount: i128) {
    env.events()
        .publish((Symbol::new(env, "withdrawn"), bidder.clone()), amount);
}
