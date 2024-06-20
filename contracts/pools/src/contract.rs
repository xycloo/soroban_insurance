use crate::{
    balance::{burn_shares, get_withdrawable_amount, mint_shares}, checks::check_amount_gt_0, events, math::{actual_period, calculate_principal_value, calculate_refund, find_x}, reflector, rewards::{pay_matured, update_fee_per_share_universal, update_rewards}, storage::*, token_utility::{get_token_client, transfer, transfer_in_pool}, types::{BalanceObject, Error, InstanceDataKey, PersistentDataKey}, DAY_IN_LEDGERS
};
use soroban_sdk::{contract, contractimpl, symbol_short, Address, BytesN, Env};

#[contract]
pub struct Pool;

pub trait SubscribeInsurance {
    /// The entry point for executing a flash loan, the initiator (or borrower) provides:
    /// `receiver_id: Address` The address of the receiver contract which contains the borrowing logic.
    /// `amount` Amount of `token_id` to borrow (`token_id` is set when the contract is initialized).
    fn subscribe(e: Env, initiator: Address, amount: i128, q: i32) -> Result<(), Error>;

    fn claim_reward(env: Env, claimant: Address) -> Result<(), Error>;

}

pub trait Vault {
    /// deposit

    /// Allows to deposit into the pool and mints liquidity provider shares to the lender.
    /// This action currently must be authorized by the `admin`, so the proxy contract.
    /// This allows a pool to be only funded when the pool is part of the wider protocol, and is not an old pool.
    /// This design decision may be removed in the next release, follow https://github.com/xycloo/xycloans/issues/16

    /// `deposit()` must be provided with:
    /// `from: Address` Address of the liquidity provider.
    /// `amount: i128` Amount of `token_id` that `from` wants to deposit in the pool.
    fn deposit(env: Env, from: Address, amount: i128) -> Result<(), Error>;

    /// update_fee_rewards

    /// Updates the matured rewards for a certain user `addr`
    /// This function may be called by anyone.

    /// `update_fee_rewards()` must be provided with:
    /// `addr: Address` The address that is udpating its fee rewards.
    fn update_fee_rewards(e: Env, addr: Address, period: i32) -> Result<(), Error>;

    /// withdraw_matured

    /// Allows a certain user `addr` to withdraw the matured fees.
    /// Before calling `withdraw_matured()` the user should call `update_fee_rewards`.
    /// If not, the matured fees that were not updated will not be lost, just not included in the payment.

    /// `withdraw_matured()` must be provided with:
    /// `addr: Address` The address that is withdrawing its fee rewards.
    fn withdraw_matured(e: Env, addr: Address, period: i32) -> Result<(), Error>;

    /// withdraw

    /// Allows to withdraw liquidity from the pool by burning liquidity provider shares.
    /// Will result in a cross contract call to the flash loan, which holds the funds that are being withdrawn.
    /// The liquidity provider can also withdraw only a portion of its shares.

    /// withdraw() must be provided with:
    /// `addr: Address` Address of the liquidity provider
    /// `amount: i28` Amount of shares that are being withdrawn
    fn withdraw(env: Env, addr: Address, period: i32) -> Result<(), Error>;

    /// Returns the amount of shares that an address holds.
    fn shares(e: Env, addr: Address, period: i32) -> i128;

    /// Returns the amount of matured fees for an address.
    fn matured(env: Env, addr: Address, period: i32) -> i128;

    fn withdrawable_amount(env: Env, addr: Address, period: i32) -> i128 ;
}

pub trait Initializable {
    /// initialize

    /// Constructor function, only to be callable once

    /// `initialize()` must be provided with:
    /// `token_id: Address` The pool's token.
    /// `flash_loan` The address of the associated flash loan contract. `flash_loan` should have `current_contract_address()` as `lp`.
    fn initialize(env: Env, admin: Address, token: Address, oracle: Address, periods_in_days: i32) -> Result<(), Error>;
}

#[contractimpl]
impl Pool {
    pub fn glob(e: Env ) -> (i128, i128) {
        (read_refund_global(&e, actual_period(&e)), get_tot_liquidity(&e, actual_period(&e)))
    }

    pub fn fpsu(e: Env) -> i128 {
        get_fee_per_share_universal(&e, actual_period(&e))
    }

    pub fn update(env: Env, hash: BytesN<32>) {
        env.storage().persistent().get::<InstanceDataKey, Address>(&InstanceDataKey::Admin).unwrap();

        env.deployer().update_current_contract_wasm(hash);
    }
}

#[contractimpl]
impl Initializable for Pool {
    fn initialize(env: Env, admin: Address, token: Address, oracle: Address, periods_in_days: i32) -> Result<(), Error> {
        if has_token_id(&env) {
            return Err(Error::AlreadyInitialized);
        }

        if has_oracle(&env) {
            return Err(Error::AlreadyInitialized);
        }

        let periods_in_ledgers = periods_in_days * DAY_IN_LEDGERS as i32;

        env.storage().persistent().set(&InstanceDataKey::Admin, &admin);

        put_oracle_id(&env, oracle);
        put_token_id(&env, token);
        write_genesis(&env);
        write_periods(&env, periods_in_ledgers);
        Ok(())
    }
}

#[contractimpl]
impl Vault for Pool {
    fn deposit(env: Env, from: Address, amount: i128) -> Result<(), Error> {
        check_amount_gt_0(amount)?;

        // finds current period
        let period = actual_period(&env);

        from.require_auth();

        bump_instance(&env);

        // we update the rewards for the current period before the deposit to avoid the abuse of the collected fees by withdrawing them with liquidity that didn't contribute to their generation.
        update_rewards(&env, from.clone(), period);
        // update_losses(&env, from.clone());

        // transfer the funds into the isurance pool
        let token_client = get_token_client(&env);
        transfer_in_pool(&env, &token_client, &from, &amount);

         // mint the new shares to the lender.
        // shares to mint will depend on: f(x) = amount_deposited * tot_supply_shares / tot_liquidity
        mint_shares(&env, from.clone(), amount, period);

        // after having calculated the right amount to mint, we van update the liquidity in the pool
        put_tot_liquidity(&env, get_tot_liquidity(&env, period) + amount, period);

        events::deposited(&env, from, amount, period);
        Ok(())
    }

    fn withdraw_matured(env: Env, addr: Address, period: i32) -> Result<(), Error> {
        // require lender auth for withdrawal
        addr.require_auth();

        bump_instance(&env);

        // pay the matured yield
        let paid = pay_matured(&env, addr.clone(), period)?;

        events::matured_withdrawn(&env, addr, paid, period);
        Ok(())
    }

    // neened to return result or error?
    fn update_fee_rewards(env: Env, addr: Address, period: i32) -> Result<(), Error> {
        bump_instance(&env);

        update_rewards(&env, addr, period);

        Ok(())
    }

    // withdraw the principal you had in a certain period
    // 1) can only be done after the period 
    // 2) it's separated from the rewards
    // 3) can be less than what you deposited if the pool had some losses
    fn withdraw(env: Env, addr: Address, period: i32) -> Result<(), Error> {

        // enforce that you can't withdraw the principal for the current period
        let current_period = actual_period(&env);
        if period >= current_period {
            return Err(Error::CannotWithdraw);
        }

        let addr_balance = read_balance(&env, addr.clone(), period);
        // if the amount is 0 return an error
        if addr_balance == 0 {
            return Err(Error::NoBalance);
        }
        
        // require lender auth for withdrawal
        addr.require_auth();

        bump_instance(&env);

        // update addr's rewards
        update_rewards(&env, addr.clone(), period);

        let tot_liquidity = get_tot_liquidity(&env, period);
        let tot_shares = get_tot_supply(&env, period);
        let principal_value = calculate_principal_value(addr_balance, tot_liquidity, tot_shares);

        // pay out the corresponding deposit
        let token_client = get_token_client(&env);
        transfer(&env, &token_client, &addr, &principal_value);

        // burn the shares
        burn_shares(&env, addr.clone(), addr_balance, period);

        events::withdrawn(&env, addr, addr_balance, period);
        Ok(())
    }

    fn shares(e: Env, addr: Address, period: i32) -> i128 {
        read_balance(&e, addr, period)
    }

    fn matured(env: Env, addr: Address, period: i32) -> i128 {
        read_matured_fees_particular(&env, addr, period)
    }

    fn withdrawable_amount(env: Env, addr: Address, period: i32) -> i128 {
        get_withdrawable_amount(&env, addr, period)
    }
}

#[contractimpl]
impl SubscribeInsurance for Pool {
    fn subscribe(e: Env, initiator: Address, amount: i128, q: i32) -> Result<(), Error> {
        initiator.require_auth();
        
        let current_period = actual_period(&e);
        
        let tot_liquidity = get_tot_liquidity(&e, current_period);
        let refund_global = read_refund_global(&e, current_period);

        // the time until the end of the period
        let time_to_end = find_x(&e, current_period);
        // calculated as y = amount * (1 + (1 / (q * time_to_end)))
        // the greater q, the greater the differential in refund prize if enter later
        let possible_amount_to_refund = calculate_refund(time_to_end as i128, amount, q as i128);
        
        if refund_global + possible_amount_to_refund > tot_liquidity {
            return Err(Error::NotEnoughLiquidity);
        }
        
        transfer_in_pool(&e, &get_token_client(&e), &initiator, &amount);
        
        update_fee_per_share_universal(&e, amount, current_period);
        
        let reflector_price: i128 = reflector::Client::new(&e, &get_oracle_id(&e)?).lastprice(&reflector::Asset::Other(symbol_short!("UNI"))).ok_or(Error::NoPrice)?.price;
        
        write_refund_particular(&e, initiator, possible_amount_to_refund, reflector_price, current_period);
        write_refund_global(&e, refund_global + possible_amount_to_refund, current_period);
        
       // bump_instance(&e);
        
        Ok(())
    }

    // can only claim suring the period you subscribed the insurance 
    fn claim_reward(e: Env, claimant: Address) -> Result<(), Error> {
        let current_period = actual_period(&e);
        
        // check if you had an available possible refund in the current period
        let refund = read_refund_particular(&e, claimant.clone(), current_period).ok_or(Error::NoInsurance)?;
        let reflector_price = reflector::Client::new(&e, &get_oracle_id(&e)?).lastprice(&reflector::Asset::Other(symbol_short!("UNI"))).ok_or(Error::NoPrice)?.price;
        let volatility = get_volatility(&e)?;

        if refund.price + volatility < reflector_price || refund.price - volatility > reflector_price {
            transfer(&e, &get_token_client(&e), &claimant, &refund.amount);
            let tot_liquidity = get_tot_liquidity(&e, current_period);
            let refund_global = read_refund_global(&e, current_period);

            write_refund_global(&e, refund_global - refund.amount, current_period);
            put_tot_liquidity(&e, tot_liquidity - refund.amount, current_period);
            e.storage().persistent().remove(&PersistentDataKey::RefundParticular(BalanceObject::new(claimant, current_period)))
        }

        bump_instance(&e);

        Ok(())
    }

}