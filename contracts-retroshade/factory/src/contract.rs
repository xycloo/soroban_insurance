#![no_std]
use soroban_sdk::{contract, contractimpl, Env};

use crate::{
    events, pool,
    storage::{get_pool_hash, has_pool_hash, set_pool_hash},
    types::Error,
};

// ---- Retroshades types behind the "mercury" feature flag -------------------
#[cfg(feature = "mercury")]
mod mercury_types {
    use retroshade_sdk::Retroshade;                 // crate `retroshade-sdk` -> `retroshade_sdk` in code
    use soroban_sdk::{contracttype, Address, Symbol};

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
}
// ---------------------------------------------------------------------------

use soroban_sdk::{Address, BytesN, Symbol};

#[contract]
pub struct Factory;

pub trait Interface {
    fn init_factory(env: Env, pool_hash: BytesN<32>) -> Result<(), Error>;

    #[allow(clippy::too_many_arguments)]
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
    // Set the pool hash before initializing any pools
    fn init_factory(env: Env, pool_hash: BytesN<32>) -> Result<(), Error> {
        if has_pool_hash(&env) {
            return Err(Error::AlreadyInitialized);
        }
        set_pool_hash(&env, pool_hash);
        Ok(())
    }

    // Currently works only with external assets
    #[allow(clippy::too_many_arguments)]
    fn initialize(
        env: Env,
        admin: Address,
        salt: BytesN<32>,
        token: Address,           // token used by the pool to collect premiums and pay benefits
        oracle: Address,          // Reflector oracle address
        symbol: Symbol,
        external_asset: bool,     // true if oracle sources data from CEX/DEX/etc.
        oracle_asset: Option<Address>, // ignored when external_asset = true
        periods_in_days: i32,     // length of each insurance period
        volatility: i128,         // stroops movement threshold for trigger
        multiplier: i32,          // see whitepaper’s premium/benefit section
    ) -> Result<Address, Error> {
        let pool_hash = get_pool_hash(&env);

        let pool_address = env.deployer()
            .with_current_contract(salt)
            .deploy(pool_hash);

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

        // Your existing contract event (traditional Soroban event)
        events::deployed_pool(&env, &pool_address);

        // ---- Retroshades emission (only compiled when --features mercury) ---
        #[cfg(feature = "mercury")]
        {
            use mercury_types::DeployedLiquidityPools;

            // NOTE: .emit(&env) — takes &Env
            DeployedLiquidityPools {
                pool: pool_address.clone(),
                pool_admin: admin.clone(),
                pool_token: token.clone(),
                pool_oracle: oracle.clone(),
                asset_symbol: symbol,
                periods_in_days,              
                volatility,                   
                multiplier,                   
            }
            .emit(&env);
        }
        // ---------------------------------------------------------------------

        Ok(pool_address)
    }
}
