use crate::types::*;
use soroban_sdk::{BytesN, Env};

pub fn set_pool_hash(env: &Env, pool_hash: BytesN<32>) {
    let key = DataKey::PoolHash;
    env.storage().instance().set(&key, &pool_hash);
}

pub fn get_pool_hash(env: &Env) -> BytesN<32> {
    let key = DataKey::PoolHash;
    env.storage().instance().get(&key).unwrap()
}

pub fn has_pool_hash(env: &Env) -> bool {
    let key = DataKey::PoolHash;
    env.storage().instance().has(&key)
}
