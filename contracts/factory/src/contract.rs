use crate::{
    events, pool,
    storage::{get_pool_hash, has_pool_hash, set_pool_hash},
    types::Error,
};
use soroban_sdk::{contract, contractimpl, Address, BytesN, Env, Symbol};

#[contract]
pub struct Factory;

pub trait Interface {
    fn init_factory(env: Env, pool_hash: BytesN<32>) -> Result<(), Error>;

    fn initialize(
        env: Env,
        admin: Address,
        salt: BytesN<32>,
        token: Address,
        oracle: Address,
        symbol: Symbol,
        external_asset: bool,
        oracle_asset: Option<Address>,
        periods_in_days: i32,
        volatility: i128,
        multiplier: i32,
    ) -> Result<Address, Error>;
}

#[contractimpl]
impl Interface for Factory {
    fn init_factory(env: Env, pool_hash: BytesN<32>) -> Result<(), Error> {
        if has_pool_hash(&env) {
            return Err(Error::AlreadyInitialized);
        }

        set_pool_hash(&env, pool_hash);

        Ok(())
    }

    fn initialize(
        env: Env,
        admin: Address,
        salt: BytesN<32>,
        token: Address,
        oracle: Address,
        symbol: Symbol,
        external_asset: bool,
        oracle_asset: Option<Address>,
        periods_in_days: i32,
        volatility: i128,
        multiplier: i32,
    ) -> Result<Address, Error> {
        let pool_hash = get_pool_hash(&env);
        let pool_address = env.deployer().with_current_contract(salt).deploy(pool_hash);
        let pool = pool::Client::new(&env, &pool_address);

        pool.initialize(
            &admin,
            &token,
            &oracle,
            &symbol,
            &external_asset,
            &oracle_asset,
            &periods_in_days,
            &volatility,
            &multiplier,
        );

        events::deployed_pool(&env, &pool_address);

        Ok(pool_address)
    }
}
