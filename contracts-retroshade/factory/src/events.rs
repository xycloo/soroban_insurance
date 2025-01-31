use soroban_sdk::{symbol_short, Address, Env};

pub(crate) fn deployed_pool(env: &Env, contract: &Address) {
    let topics = (symbol_short!("deployed"),);
    env.events().publish(topics, contract);
}
