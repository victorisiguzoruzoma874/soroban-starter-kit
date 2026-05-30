use soroban_sdk::{Address, Env, Symbol};

pub fn initialized(env: &Env, admin: &Address, stake_token: &Address, reward_token: &Address) {
    env.events().publish(
        (Symbol::new(env, "initialized"), admin.clone()),
        (stake_token.clone(), reward_token.clone()),
    );
}

pub fn staked(env: &Env, staker: &Address, amount: i128, new_total: i128) {
    env.events().publish(
        (Symbol::new(env, "staked"), staker.clone()),
        (amount, new_total),
    );
}

pub fn unstaked(env: &Env, staker: &Address, amount: i128, remaining: i128) {
    env.events().publish(
        (Symbol::new(env, "unstaked"), staker.clone()),
        (amount, remaining),
    );
}

pub fn rewards_claimed(env: &Env, staker: &Address, amount: i128) {
    env.events().publish(
        (Symbol::new(env, "rewards_claimed"), staker.clone()),
        amount,
    );
}

pub fn rewards_added(env: &Env, admin: &Address, amount: i128, new_total: i128) {
    env.events().publish(
        (Symbol::new(env, "rewards_added"), admin.clone()),
        (amount, new_total),
    );
}
