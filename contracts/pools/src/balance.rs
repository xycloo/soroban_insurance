use crate::{math::{calculate_to_mint, calculate_principal_value}, storage::{get_tot_liquidity, get_tot_supply, put_tot_supply, read_balance, read_matured_fees_particular, write_balance}};
use soroban_sdk::{Address, Env};

pub(crate) fn mint_shares(e: &Env, to: Address, amount: i128, period: i32) {
    let to_mint = calculate_to_mint(e, amount, get_tot_supply(e, period), get_tot_liquidity(e, period));

    // add to total supply
    put_tot_supply(e, get_tot_supply(e, period) + to_mint, period);  

    // add to user balance
    write_balance(e, to.clone(), read_balance(e, to, period) + to_mint, period);
}

// needs to be rewritten
pub(crate) fn burn_shares(e: &Env, to: Address, shares: i128, period: i32) {
    // update the total supply
    let tot_supply = get_tot_supply(e, period);
    put_tot_supply(e, tot_supply - shares, period);

    write_balance(e, to, 0, period);
}

pub(crate) fn get_withdrawable_amount(env: &Env, addr: Address, period: i32) -> i128 {
    let shares = read_balance(env, addr.clone(), period);
    let tot_liquidity = get_tot_liquidity(env, period);
    let tot_shares = get_tot_supply(env, period);
    let principal = calculate_principal_value(shares, tot_liquidity, tot_shares); 
    let accrued_fees = read_matured_fees_particular(env, addr.clone(), period);
    let withdrawable_amount = principal + accrued_fees;
    withdrawable_amount
}