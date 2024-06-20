use soroban_sdk::Env;

use crate::storage::{get_genesis, get_periods};

// ledgers before the current period ends 
pub(crate) fn find_x(env: &Env, current_period: i32) -> i32 {
    // current day - (genesis + [(current_period - 1) * days_in_periods])
    let genesis = get_genesis(env);
    let current_ledger = env.ledger().sequence() as i32;
    let periods = get_periods(env);
    let end_previous_period = genesis + (current_period - 1) * periods;
    let x = current_ledger - end_previous_period;
    x
}