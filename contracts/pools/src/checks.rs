use soroban_sdk::{token::Client, Env};
use crate::{math::actual_period, types::Error};

// This function was introduced as an extra measure under the advice of auditors
// in order to avoid potentially undesired events due to rounding errors.

/// Extra-check that the pool balance never goes below the total supply.
/*
pub(crate) fn check_balance_ge_supply(env: &Env, token_client: &Client) -> Result<(), Error> {
    let pool_balance = token_client.balance(&env.current_contract_address());
    let total_supply = get_tot_supply(env);

    if pool_balance < total_supply {
        return Err(Error::BalanceLtSupply)
    };

    Ok(())
}
    */

/// Make sure that we're dealing with amounts > 0
pub(crate) fn check_amount_gt_0(amount: i128) -> Result<(), Error> {
    if amount <= 0 {
        return Err(Error::InvalidAmount);
    }

    Ok(())
}

/* 
pub(crate) fn check_update_period(env: &Env) {
    let actual_period = actual_period(&env);
        let current_period = get_period(&env);
        if actual_period > current_period {
            write_period(&env, actual_period)
        }
} 
*/