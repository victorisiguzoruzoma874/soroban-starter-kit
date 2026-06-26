use soroban_sdk::{Address, Env, Symbol};

pub fn initialized(env: &Env, admin: &Address, token: &Address) {
    let topics = (Symbol::new(env, "initialized"),);
    env.events().publish(topics, (admin.clone(), token.clone()));
}

pub fn bought(env: &Env, buyer: &Address, tokens: i128, cost: i128) {
    let topics = (Symbol::new(env, "bought"),);
    env.events().publish(topics, (buyer.clone(), tokens, cost));
}

pub fn sold(env: &Env, seller: &Address, tokens: i128, proceeds: i128) {
    let topics = (Symbol::new(env, "sold"),);
    env.events().publish(topics, (seller.clone(), tokens, proceeds));
}
