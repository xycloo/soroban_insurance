use soroban_sdk::{Address, Env};
use fixed_point_math;

use crate::{
    math::calculate_period, types::{BalanceObject, Error, InstanceDataKey, Insurance, PersistentDataKey}, INSTANCE_LEDGER_LIFE, INSTANCE_LEDGER_TTL_THRESHOLD, PERSISTENT_LEDGER_LIFE, PERSISTENT_LEDGER_TTL_THRESHOLD
};

// User specific state.

pub(crate) fn bump_persistent(e: &Env, key: &PersistentDataKey) {
    e.storage()
        .persistent()
        .extend_ttl(key, PERSISTENT_LEDGER_TTL_THRESHOLD, PERSISTENT_LEDGER_LIFE);
}

pub(crate) fn write_balance(e: &Env, addr: Address, balance: i128, period: i32) {
    let balance_object = BalanceObject::new(addr, period);
    let key = PersistentDataKey::Balance(balance_object);
    e.storage().persistent().set(&key, &balance);
    bump_persistent(e, &key);
}

pub(crate) fn read_balance(e: &Env, addr: Address, period: i32) -> i128 {
    let balance_object = BalanceObject::new(addr, period);
    let key = PersistentDataKey::Balance(balance_object);

    if let Some(balance) = e.storage().persistent().get(&key) {
        bump_persistent(e, &key);
        balance
    } else {
        0
    }
}

pub(crate) fn write_fee_per_share_particular(e: &Env, addr: Address, amount: i128, period: i32) {
    let balance_object = BalanceObject::new(addr, period);
    let key = PersistentDataKey::FeePerShareParticular(balance_object);
    e.storage().persistent().set(&key, &amount);
    bump_persistent(e, &key);
}

pub(crate) fn read_fee_per_share_particular(e: &Env, addr: Address, period: i32) -> i128 {
    let balance_object = BalanceObject::new(addr, period);
    let key = PersistentDataKey::FeePerShareParticular(balance_object);

    if let Some(particular) = e.storage().persistent().get(&key) {
        bump_persistent(e, &key);
        particular
    } else {
        0
    }
}

pub(crate) fn write_matured_fees_particular(e: &Env, addr: Address, amount: i128, period: i32) {
    let balance_object = BalanceObject::new(addr, period);
    let key = PersistentDataKey::MaturedFeesParticular(balance_object);
    e.storage().persistent().set(&key, &amount);
    bump_persistent(e, &key);
}

pub(crate) fn read_matured_fees_particular(e: &Env, addr: Address, period: i32) -> i128 {
    let balance_object = BalanceObject::new(addr, period);
    let key = PersistentDataKey::MaturedFeesParticular(balance_object);

    if let Some(matured) = e.storage().persistent().get(&key) {
        bump_persistent(e, &key);
        matured
    } else {
        0
    }
}

pub(crate) fn write_refund_particular(e: &Env, addr: Address, amount: i128, price: i128, period: i32) {
    let balance_object = BalanceObject::new(addr, period);
    let key = PersistentDataKey::RefundParticular(balance_object);
    e.storage().persistent().set(&key, &Insurance {
        amount,
        price
    });
    bump_persistent(e, &key);
}

pub(crate) fn read_refund_particular(e: &Env, addr: Address, period: i32) -> Option<Insurance> {
    let balance_object = BalanceObject::new(addr, period);
    let key = PersistentDataKey::RefundParticular(balance_object);
    
    e.storage().persistent().get(&key)
}

pub(crate) fn write_refund_global(e: &Env, amount: i128, period: i32) {
    let key = PersistentDataKey::RefundGlobal(period);
    e.storage().persistent().set(&key, &amount);
    bump_persistent(e, &key);
}

pub(crate) fn read_refund_global(e: &Env, period: i32) -> i128 {
    let key = PersistentDataKey::RefundGlobal(period);
    let refund = e.storage().persistent().get(&key).unwrap_or(0);
    refund
}

pub(crate) fn bump_instance(env: &Env) {                                                    // used
    env.storage()
        .instance()
        .extend_ttl(INSTANCE_LEDGER_TTL_THRESHOLD, INSTANCE_LEDGER_LIFE);
}

pub(crate) fn put_tot_supply(e: &Env, supply: i128, period: i32) {
    let key = PersistentDataKey::TotSupply(period);
    e.storage().persistent().set(&key, &supply);

    bump_persistent(e, &key);
}

// update it only when you have a deposit or when you have a loss inside the period
pub(crate) fn put_tot_liquidity(e: &Env, liquidity: i128, period: i32) {
    let key = PersistentDataKey::TotLiquidity(period);
    e.storage().persistent().set(&key, &liquidity);

    bump_persistent(e, &key);
}

// total amount of minted shares for the period
pub(crate) fn get_tot_supply(e: &Env, period: i32) -> i128 {
    let key = PersistentDataKey::TotSupply(period);
    e.storage().persistent().get(&key).unwrap_or(0)
}

pub(crate) fn get_tot_liquidity(e: &Env, period: i32) -> i128 {
    let key = PersistentDataKey::TotLiquidity(period);
    e.storage().persistent().get(&key).unwrap_or(0)
}

pub(crate) fn put_fee_per_share_universal(e: &Env, last_recorded: i128, period: i32) {
    let key = PersistentDataKey::FeePerShareUniversal(period);
    e.storage().instance().set(&key, &last_recorded);
}

pub(crate) fn get_fee_per_share_universal(e: &Env, period: i32) -> i128 {                // used
    let key = PersistentDataKey::FeePerShareUniversal(period);
    e.storage().instance().get(&key).unwrap_or(0)
}

pub(crate) fn has_token_id(e: &Env) -> bool {     // used
    let key = InstanceDataKey::TokenId;
    e.storage().instance().has(&key)
}

pub(crate) fn put_token_id(e: &Env, token_id: Address) {      // used
    let key = InstanceDataKey::TokenId;
    e.storage().instance().set(&key, &token_id);
}

pub(crate) fn get_token_id(e: &Env) -> Result<Address, Error> {
    let key = InstanceDataKey::TokenId;

    if let Some(token) = e.storage().instance().get(&key) {
        Ok(token)
    } else {
        return Err(Error::NotInitialized);
    }
}

pub(crate) fn has_oracle(e: &Env) -> bool {     // used
    let key = InstanceDataKey::Oracle;
    e.storage().instance().has(&key)
}

pub(crate) fn put_oracle_id(e: &Env, oracle: Address) {      // used
    let key = InstanceDataKey::Oracle;
    e.storage().instance().set(&key, &oracle);
}

pub(crate) fn get_oracle_id(e: &Env) -> Result<Address, Error> {
    let key = InstanceDataKey::Oracle;

    if let Some(oracle) = e.storage().instance().get(&key) {
        Ok(oracle)
    } else {
        return Err(Error::NotInitialized);
    }
}


pub(crate) fn put_volatility(e: &Env, amount: i128) {      // used
    let key = InstanceDataKey::Volatility;
    e.storage().instance().set(&key, &amount);
}

pub(crate) fn get_volatility(e: &Env) -> Result<i128, Error> {
    let key = InstanceDataKey::Volatility;

    if let Some(oracle) = e.storage().instance().get(&key) {
        Ok(oracle)
    } else {
        return Err(Error::NotInitialized);
    }
}

pub(crate) fn write_genesis(e: &Env) {                              // used
    let key = InstanceDataKey::GenesisPeriod;
    let current_ledger = e.ledger().sequence() as i32;
    e.storage().instance().set(&key, &current_ledger);
}

pub(crate) fn get_genesis(e: &Env) -> i32 {                             // used
    let key = InstanceDataKey::GenesisPeriod;
    let genesis = e.storage().instance().get(&key).unwrap();
    genesis
}

// writes and gives the time-span of a single period of insurance, set at beginning (ex: 30d)
pub(crate) fn write_periods(e: &Env, periods_in_ledgers: i32) {            // used
    let key = InstanceDataKey::Periods;
    e.storage().instance().set(&key, &periods_in_ledgers);
}

pub(crate) fn get_periods(e: &Env) -> i32 {                        // used
    let key = InstanceDataKey::Periods;
    let periods: i32 = e.storage().instance().get(&key).unwrap();
    periods
}
