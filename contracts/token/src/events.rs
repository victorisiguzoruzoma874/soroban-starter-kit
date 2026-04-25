use soroban_sdk::{Address, Env, String, Symbol};

pub fn initialized(env: &Env, admin: &Address, name: String, symbol: String, decimals: u32) {
    env.events().publish((Symbol::new(env, "initialized"), admin.clone()), (name, symbol, decimals));
}

pub fn minted(env: &Env, to: &Address, amount: i128) {
    env.events().publish((Symbol::new(env, "mint"), to.clone()), amount);
}

pub fn burned(env: &Env, from: &Address, amount: i128) {
    env.events().publish((Symbol::new(env, "burn"), from.clone()), amount);
}

pub fn admin_changed(env: &Env, old_admin: &Address, new_admin: &Address) {
    env.events().publish((Symbol::new(env, "admin_changed"), old_admin.clone()), new_admin.clone());
}

pub fn approved(env: &Env, from: &Address, spender: &Address, amount: i128) {
    env.events().publish((Symbol::new(env, "approve"), from.clone(), spender.clone()), amount);
}

pub fn revoked(env: &Env, from: &Address, spender: &Address) {
    env.events().publish((Symbol::new(env, "revoke"), from.clone(), spender.clone()), ());
}

pub fn transferred(env: &Env, from: &Address, to: &Address, amount: i128) {
    env.events().publish((Symbol::new(env, "transfer"), from.clone(), to.clone()), amount);
}
