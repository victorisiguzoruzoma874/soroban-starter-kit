use soroban_sdk::{Address, Env, Symbol};

pub fn initialized(env: &Env, admin: &Address, staleness_threshold: u32) {
    env.events().publish(
        (Symbol::new(env, "initialized"), admin.clone()),
        staleness_threshold,
    );
}

pub fn price_updated(env: &Env, admin: &Address, price: i128, ledger: u32) {
    env.events().publish(
        (Symbol::new(env, "price_updated"), admin.clone()),
        (price, ledger),
    );
}
