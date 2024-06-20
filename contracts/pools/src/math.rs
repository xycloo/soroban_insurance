use soroban_sdk::Env;
use crate::storage::{get_periods, get_genesis};
use core::ops::{Add, Sub};
use fixed_point_math::{FixedPoint, STROOP};

pub fn compute_fee_per_share(
    fee_per_share_universal: i128,
    accrued_interest: i128,
    total_supply: i128,
) -> i128 {
    let interest_by_supply = accrued_interest.fixed_div_floor(total_supply, STROOP.into()).unwrap();
    let computed_floored = fee_per_share_universal.add(interest_by_supply);
    
    computed_floored
}

// used
pub fn compute_fee_earned(
    user_balance: i128,
    fee_per_share_universal: i128,
    fee_per_share_particular: i128,
) -> i128 {
    user_balance
        .fixed_mul_floor(
            fee_per_share_universal.sub(fee_per_share_particular),
            STROOP.into(),
        )
        .unwrap()
}

// used
pub fn compute_losses(
    user_balance: i128,
    loss_per_share_universal: i128,
    loss_per_share_particular: i128,
) -> i128 {
    user_balance
        .fixed_mul_floor(
            loss_per_share_universal.sub(loss_per_share_particular),
            STROOP.into(),
        )
        .unwrap()
}

// used
pub(crate) fn calculate_period(current: i128, genesis: i128, periods: i128) -> i32 {    
    let current_ledger = current;
    let genesis_ledger = genesis;
    let diff = current_ledger - genesis_ledger;
    let div = diff.fixed_div_ceil(periods, 1).unwrap();
    
    if div == 0 {
        return 1
    }
    
    div as i32
}

// gives the current period (ex: 1)
// used
pub(crate) fn actual_period(e: &Env) -> i32 {                
    let current_ledger = e.ledger().sequence();
    let genesis_ledger = get_genesis(&e);
    let periods = get_periods(&e);
    //calculate_period(current_ledger as i128, genesis_ledger as i128, periods as i128);

    1
}

pub(crate) fn calculate_to_mint(e: &Env, amount: i128, total_supply: i128, total_liquidity: i128) -> i128 {
    if total_liquidity == 0 {
        return amount
    }

    let multiplier = total_supply
    .fixed_div_floor(
        total_liquidity,
        STROOP.into(),
    )
    .unwrap();

    amount
        .fixed_mul_floor(
            multiplier,
            STROOP.into()
        )
        .unwrap()
}

pub(crate) fn calculate_principal_value(shares: i128, tot_liquidity: i128, tot_shares: i128) -> i128 {
    let multiplier = tot_liquidity
    .fixed_div_floor(
        tot_shares,
        STROOP.into(),
    )
    .unwrap();

    shares
        .fixed_mul_floor(
            multiplier,
            STROOP.into(),
        )
        .unwrap()
}

pub(crate) fn find_x(env: &Env, current_period: i32) -> i32 {
    // current day - (genesis + [(current_period - 1) * days_in_periods])
    let genesis = get_genesis(env);
    let current_ledger = env.ledger().sequence() as i32;
    let periods = get_periods(env);
    let end_previous_period = genesis + (current_period - 1) * periods;
    let x = current_ledger - end_previous_period;
    x
}

pub(crate) fn calculate_refund(time_to_end: i128, amount: i128) -> i128 {
    let coefficient = amount / time_to_end;

    amount + coefficient
}
