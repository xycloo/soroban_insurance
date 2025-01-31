use crate::{
    events, pool,
    storage::{get_pool_hash, has_pool_hash, set_pool_hash},
    types::Error,
};
use retroshade_sdk::Retroshade;
use soroban_sdk::{contract, contractimpl, contracttype, Address, BytesN, Env, Symbol};

#[derive(Retroshade)]
#[contracttype]
pub struct DeployedLiquidityPools {
    pub pool: Address,
    pub pool_admin: Address,
    pub pool_token: Address,
    pub pool_oracle: Address,
    pub asset_symbol: Symbol,
    pub periods_in_days: i32,
    pub volatility: i128,
    pub multiplier: i32,
}

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
    // set the pool hash before initializing any pools
    fn init_factory(env: Env, pool_hash: BytesN<32>) -> Result<(), Error> {
        if has_pool_hash(&env) {
            return Err(Error::AlreadyInitialized);
        }

        set_pool_hash(&env, pool_hash);

        Ok(())
    }

    // currently works only with external assets
    fn initialize(
        env: Env,
        admin: Address,
        salt: BytesN<32>,
        token: Address, // the token used by rthe pool to collect insurance premiums and pay amounts
        oracle: Address, // must be a Reflector oracle address
        symbol: Symbol,
        external_asset: bool, // true/false: internal if using an oracle that sources datat from Stellar pubnet (for Stellar Classic and Soroban assets), external if oracle's data source is external CEX/DEX (for any external tokens/assets/symbols)
        oracle_asset: Option<Address>, // if the asset is external, set this to None or whatever value (it will not be taken into account)
        periods_in_days: i32,          // days each "insurance period" will last
        volatility: i128, // amount in stroops that the price of the followed asse needs to move with respect to the base asset (USD/USDC) in order for the triggering condition to happen
        multiplier: i32, // see section "Insurance Premiums and how Benefits are Calculated" of the whitepaper
    ) -> Result<Address, Error> {
        let pool_hash = get_pool_hash(&env);
        let pool_address = env.deployer().with_current_contract(salt).deploy(pool_hash);
        let pool = pool::Client::new(&env, &pool_address.clone());

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

        // retroshades
        DeployedLiquidityPools {
            pool: pool_address.clone(),
            pool_admin: admin.clone(),
            pool_token: token.clone(),
            pool_oracle: oracle.clone(),
            asset_symbol: symbol.clone(),
            periods_in_days: periods_in_days.clone(),
            volatility: volatility.clone(),
            multiplier: multiplier.clone(),
        }
        .emit(&env);

        Ok(pool_address)
    }
}
