use fixed_point_math::STROOP;
use soroban_sdk::{contract, contractimpl, contracttype, testutils::{Address as _, Ledger}, token, Address, Env};

use crate::{contract::{Pool, PoolClient}, reflector::Asset, DAY_IN_LEDGERS};

mod mock_20 {
    use soroban_sdk::{contract, contractimpl, Env};
    use crate::{reflector::Asset};
    use super::PriceData;

    #[contract]
    pub struct PricesMock20;

    #[contractimpl]
    impl PricesMock20 {
        pub fn lastprice(env: Env, asset: Asset) -> Option<PriceData> {
            Some(PriceData {
                price: 20,
                timestamp: 0
            })
        }
    }
}
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
// The price data for an asset at a given timestamp.
pub struct PriceData {
    // The price in contracts' base asset and decimals.
    pub price: i128,
    // The timestamp of the price.
    pub timestamp: u64,
}

mod mock_50 {
    use soroban_sdk::{contract, contractimpl, Env};
    use crate::{reflector::Asset};

    #[contract]
    pub struct PricesMock50;

    #[contractimpl]
    impl PricesMock50 {
        pub fn lastprice(env: Env, asset: Asset) -> Option<i128> {
            Some(20)
        }
    }
}


mod mock_70 {
    use soroban_sdk::{contract, contractimpl, Env};
    use crate::{reflector::Asset};
    #[contract]
    pub struct PricesMock70;

    #[contractimpl]
    impl PricesMock70 {
        pub fn lastprice(env: Env, asset: Asset) -> Option<i128> {
            Some(20)
        }
    }
}
extern crate std;
// NOTE: needs more coverage in the future.

#[should_panic(expected = "HostError: Error(Contract, #6)")]
#[test]
fn deposit_withdraw() {
    let env = Env::default();

    env.mock_all_auths();
    env.budget().reset_unlimited();

    let admin1 = Address::generate(&env);
    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);

    let token_id = env.register_stellar_asset_contract(admin1.clone());
    let token_admin = token::StellarAssetClient::new(&env, &token_id);
    
    token_admin.mint(&user1, &(1000 * STROOP as i128));
    token_admin.mint(&user2, &(500 * STROOP as i128));

    let pool_addr = env.register_contract(&None, Pool);
    let pool_client = PoolClient::new(&env, &pool_addr);

    let oracle_addr = env.register_contract(&None, mock_20::PricesMock20);

    pool_client.initialize(&admin1, &token_id, &oracle_addr, &30, &1000000000, &3);

    pool_client.deposit(&user1, &(1000 * STROOP as i128));
    pool_client.deposit(&user2, &(500 * STROOP as i128));

    pool_client.withdraw(&user1, &1);
}


/// check the "subscribe" function and 
/// - that the rewards are distributed correctly between liquidity provider shares
/// - that the possible premiums to pay are stored correctly 
/// - additianally we check that we can't perorm aother suscription for the current period (should fail with error "AlreadySubscribed")
#[should_panic(expected = "HostError: Error(Contract, #11)")]
#[test]
fn insurance() {
    let env = Env::default();

    env.mock_all_auths();
    env.budget().reset_unlimited();

    let admin1 = Address::generate(&env);
    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);

    let user3 = Address::generate(&env);

    let token_id = env.register_stellar_asset_contract(admin1.clone());
    let token_admin = token::StellarAssetClient::new(&env, &token_id);
    
    token_admin.mint(&user1, &(1000 * STROOP as i128));
    token_admin.mint(&user2, &(500 * STROOP as i128));
    token_admin.mint(&user3, &(200 * STROOP as i128));

    let pool_addr = env.register_contract(&None, Pool);
    let pool_client = PoolClient::new(&env, &pool_addr);

    let oracle_addr = env.register_contract(&None, mock_20::PricesMock20);

    pool_client.initialize(&admin1, &token_id, &oracle_addr, &30, &1000000000, &3);

    pool_client.deposit(&user1, &(1000 * STROOP as i128));
    pool_client.deposit(&user2, &(500 * STROOP as i128));

    env.ledger().with_mut(|ledger| {
        ledger.sequence_number += 200
    });

    std::println!("{:?}", pool_client.glob());

    pool_client.subscribe(&user3, &2000000);

    std::println!("{:?}", pool_client.glob());

    std::println!("{:?}", pool_client.fpsu());

    env.ledger().with_mut(|ledger| {
        ledger.sequence_number += 200
    });

    std::println!("{:?}", pool_client.fpsu());

    pool_client.subscribe(&user3, &2000000);
}

/// check
/// - correct succession of periods (persistent data not automatiaclly tranferrable between periods)
/// - correct withdrawal of matured fees
#[should_panic(expected = "HostError: Error(Contract, #3)")]
#[test]
fn periods() {
    let env = Env::default();

    env.mock_all_auths();
    env.budget().reset_unlimited();

    let admin1 = Address::generate(&env);
    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);

    let user3 = Address::generate(&env);

    let token_id = env.register_stellar_asset_contract(admin1.clone());
    let token_admin = token::StellarAssetClient::new(&env, &token_id);
    
    token_admin.mint(&user1, &(1000 * STROOP as i128));
    token_admin.mint(&user2, &(6000 * STROOP as i128));
    token_admin.mint(&user3, &(200 * STROOP as i128));

    let pool_addr = env.register_contract(&None, Pool);
    let pool_client = PoolClient::new(&env, &pool_addr);

    let oracle_addr = env.register_contract(&None, mock_20::PricesMock20);

    pool_client.initialize(&admin1, &token_id, &oracle_addr, &30, &1000000000, &3);

    pool_client.deposit(&user1, &(1000 * STROOP as i128));
    pool_client.deposit(&user2, &(500 * STROOP as i128));

    env.ledger().with_mut(|ledger| {
        ledger.sequence_number += 200
    });

    std::println!("{:?}", pool_client.glob());

    pool_client.subscribe(&user3, &2000000);

    std::println!("{:?}", pool_client.glob());

    std::println!("{:?}", pool_client.fpsu());

    env.ledger().with_mut(|ledger| {
        ledger.sequence_number += 200
    });

    std::println!("{:?}", pool_client.fpsu());
    std::println!("{:?}", pool_client.read_current_period());

    env.ledger().with_mut(|ledger| {
        ledger.sequence_number += 5 * DAY_IN_LEDGERS
    });

    // need to do that to bump instance and persistent several times (oterwise in tests expires before the actual threshold)
    pool_client.deposit(&user2, &(50 * STROOP as i128));

    env.ledger().with_mut(|ledger| {
        ledger.sequence_number += 5 * DAY_IN_LEDGERS
    });

    pool_client.deposit(&user2, &(50 * STROOP as i128));

    env.ledger().with_mut(|ledger| {
        ledger.sequence_number += 5 * DAY_IN_LEDGERS
    });

    pool_client.deposit(&user2, &(50 * STROOP as i128));

    env.ledger().with_mut(|ledger| {
        ledger.sequence_number += 5 * DAY_IN_LEDGERS
    });

    pool_client.deposit(&user2, &(50 * STROOP as i128));

    env.ledger().with_mut(|ledger| {
        ledger.sequence_number += 5 * DAY_IN_LEDGERS
    });

    // check before entering period 2
    std::println!("{:?}", pool_client.glob());

    pool_client.deposit(&user2, &(50 * STROOP as i128));

    env.ledger().with_mut(|ledger| {
        ledger.sequence_number += 5 * DAY_IN_LEDGERS
    });

    pool_client.deposit(&user2, &(50 * STROOP as i128));

    std::println!("{:?}", pool_client.glob());

    std::println!("{:?}", pool_client.read_current_period());

    // updating and withdrawing fees for the previous period: this works
    pool_client.update_fee_rewards(&user2, &1);
    pool_client.withdraw_matured(&user2, &1);

    // this should panic as no rewards for second period
    pool_client.update_fee_rewards(&user2, &2);
    pool_client.withdraw_matured(&user2, &2)

}

/// check
/// checking correct withdrawal of matured fees
#[test]
fn refund() {
    let env = Env::default();

    env.mock_all_auths();
    env.budget().reset_unlimited();

    let admin1 = Address::generate(&env);
    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);

    let user3 = Address::generate(&env);

    let token_id = env.register_stellar_asset_contract(admin1.clone());
    let token_admin = token::StellarAssetClient::new(&env, &token_id);
    
    token_admin.mint(&user1, &(1000 * STROOP as i128));
    token_admin.mint(&user2, &(6000 * STROOP as i128));
    token_admin.mint(&user3, &(200 * STROOP as i128));

    let pool_addr = env.register_contract(&None, Pool);
    let pool_client = PoolClient::new(&env, &pool_addr);

    let oracle_addr = env.register_contract(&None, mock_20::PricesMock20);

    pool_client.initialize(&admin1, &token_id, &oracle_addr, &30, &1000000000, &3);

    pool_client.deposit(&user1, &(1000 * STROOP as i128));
    pool_client.deposit(&user2, &(500 * STROOP as i128));

    env.ledger().with_mut(|ledger| {
        ledger.sequence_number += 200
    });

    std::println!("{:?}", pool_client.glob());

    pool_client.subscribe(&user3, &2000000);

    std::println!("{:?}", pool_client.glob());

    std::println!("{:?}", pool_client.fpsu());

    env.ledger().with_mut(|ledger| {
        ledger.sequence_number += 200
    });

    std::println!("{:?}", pool_client.fpsu());
    std::println!("{:?}", pool_client.read_current_period());

    env.ledger().with_mut(|ledger| {
        ledger.sequence_number += 5 * DAY_IN_LEDGERS
    });

    // need to do that to bump instance and persistent several times (oterwise in tests expires before the actual threshold)
    pool_client.deposit(&user2, &(50 * STROOP as i128));

}