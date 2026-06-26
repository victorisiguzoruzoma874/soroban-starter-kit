use soroban_sdk::{symbol_short, Address, Bytes, Env};

pub fn root_set(env: &Env, root: &Bytes) {
    env.events().publish((symbol_short!("root_set"),), root.clone());
}

pub fn claimed(env: &Env, recipient: &Address, amount: i128) {
    env.events()
        .publish((symbol_short!("claimed"),), (recipient.clone(), amount));
}
