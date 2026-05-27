use soroban_sdk::{Address, BytesN, Env, Symbol};

pub fn initialized(env: &Env, buyer: &Address, seller: &Address, arbiter: &Address, amount: i128) {
    env.events().publish(
        (Symbol::new(env, "initialized"), buyer.clone(), seller.clone(), arbiter.clone()),
        amount,
    );
}

pub fn escrow_created(env: &Env, buyer: &Address, seller: &Address, amount: i128) {
    env.events().publish((Symbol::new(env, "escrow_created"), buyer.clone(), seller.clone()), amount);
}

pub fn escrow_funded(env: &Env, buyer: &Address, amount: i128) {
    env.events().publish((Symbol::new(env, "escrow_funded"), buyer.clone()), amount);
}

pub fn delivery_marked(env: &Env, seller: &Address) {
    env.events().publish((Symbol::new(env, "delivery_marked"), seller.clone()), ());
}

pub fn funds_released(env: &Env, seller: &Address, amount: i128) {
    env.events().publish((Symbol::new(env, "funds_released"), seller.clone()), amount);
}

pub fn funds_refunded(env: &Env, buyer: &Address, amount: i128) {
    env.events().publish((Symbol::new(env, "funds_refunded"), buyer.clone()), amount);
}

pub fn paused(env: &Env, admin: &Address) {
    env.events().publish((Symbol::new(env, "paused"), admin.clone()), ());
}

pub fn unpaused(env: &Env, admin: &Address) {
    env.events().publish((Symbol::new(env, "unpaused"), admin.clone()), ());
}

pub fn upgraded(env: &Env, admin: &Address, new_wasm_hash: &soroban_sdk::BytesN<32>) {
    env.events().publish((Symbol::new(env, "upgraded"), admin.clone()), new_wasm_hash.clone());
}
