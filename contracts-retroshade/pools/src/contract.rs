#![no_std]
use crate::{
    balance::{burn_shares, get_withdrawable_amount, mint_shares},
    checks::check_amount_gt_0,
    events,
    math::{actual_period, calculate_principal_value, calculate_refund, calculate_to_mint, find_x},
    reflector,
    rewards::{pay_matured, update_fee_per_share_universal, update_rewards},
    storage::*,
    token_utility::{get_token_client, transfer, transfer_in_pool},
    types::{BalanceObject, Error, InstanceDataKey, Insurance, PersistentDataKey},
    MIN_IN_LEDGERS,
};
use soroban_sdk::{contract, contractimpl, Address, BytesN, Env, Symbol};

// ---------------- Retroshades types (only when --features mercury) ----------------
#[cfg(feature = "mercury")]
mod retroshade {
    use retroshade_sdk::Retroshade;        // crate name retroshade-sdk -> module retroshade_sdk
    use soroban_sdk::{contracttype, Address, Symbol};

    #[derive(Retroshade)]
    #[contracttype]
    pub struct DepositEvent {
        pub from: Address,
        pub amount: i128,
        pub ledger: u32,
        pub period: i32,
        pub new_shares_minted: i128,
    }

    #[derive(Retroshade)]
    #[contracttype]
    pub struct WithdrawMaturedEvent {
        pub from: Address,
        pub paid: i128,
        pub ledger: u32,
        pub period: i32,
    }

    #[derive(Retroshade)]
    #[contracttype]
    pub struct WithdrawEvent {
        pub from: Address,
        pub burnt_shares: i128,
        pub amount_withdrawn: i128,
        pub ledger: u32,
        pub period: i32,
    }
}
// -------------------------------------------------------------------------------

#[contract]
pub struct Pool;

pub trait SubscribeInsurance {
    fn subscribe(e: Env, initiator: Address, amount: i128) -> Result<(), Error>;
    fn claim_reward(env: Env, claimant: Address) -> Result<(), Error>;
}

pub trait Vault {
    fn deposit(env: Env, from: Address, amount: i128) -> Result<(), Error>;
    fn update_fee_rewards(e: Env, addr: Address, period: i32) -> Result<(), Error>;
    fn withdraw_matured(e: Env, addr: Address, period: i32) -> Result<(), Error>;
    fn withdraw(env: Env, addr: Address, period: i32) -> Result<(), Error>;
    fn shares(e: Env, addr: Address, period: i32) -> i128;
    fn matured(env: Env, addr: Address, period: i32) -> i128;
    fn withdrawable_amount(env: Env, addr: Address, period: i32) -> i128;
}

pub trait Initializable {
    fn initialize(
        env: Env,
        admin: Address,
        token: Address,
        oracle: Address,
        symbol: Symbol,
        external_asset: bool,
        oracle_asset: Option<Address>,
        periods_in_min: i32,
        volatility: i128,
        multiplier: i32,
    ) -> Result<(), Error>;
}

#[contractimpl]
impl Pool {
    pub fn glob(e: Env) -> (i128, i128, i128, i128) {
        (
            read_refund_global(&e, actual_period(&e)),
            get_tot_liquidity(&e, actual_period(&e)),
            get_fee_per_share_universal(&e, actual_period(&e)),
            get_tot_supply(&e, actual_period(&e)),
        )
    }

    pub fn particular(e: Env, user: Address) -> (i128, i128, i128, Option<Insurance>) {
        (
            read_balance(&e, user.clone(), actual_period(&e)),
            read_fee_per_share_particular(&e, user.clone(), actual_period(&e)),
            read_matured_fees_particular(&e, user.clone(), actual_period(&e)),
            read_refund_particular(&e, user.clone(), actual_period(&e)),
        )
    }

    pub fn get_price(e: Env) -> Option<i128> {
        let oracle_id = get_oracle_id(&e).ok()?;
        let client = reflector::Client::new(&e, &oracle_id);

        let symbol = get_symbol(&e);
        let external = get_external(&e);

        let last_price = if external {
            client.lastprice(&reflector::Asset::Other(symbol))
        } else {
            if let Some(asset) = get_oracle_asset(&e) {
                client.lastprice(&reflector::Asset::Stellar(asset))
            } else {
                None
            }
        };

        last_price.map(|lp| lp.price)
    }

    pub fn fpsu(e: Env) -> i128 {
        get_fee_per_share_universal(&e, actual_period(&e))
    }

    pub fn fpsp(e: Env, user: Address) -> i128 {
        read_fee_per_share_particular(&e, user, actual_period(&e))
    }

    pub fn read_current_period(e: Env) -> i32 {
        actual_period(&e)
    }

    pub fn update(env: Env, hash: BytesN<32>) {
        env.storage()
            .instance()
            .get::<InstanceDataKey, Address>(&InstanceDataKey::Admin)
            .unwrap()
            .require_auth();

        env.deployer().update_current_contract_wasm(hash);
    }
}

#[contractimpl]
impl Initializable for Pool {
    fn initialize(
        env: Env,
        admin: Address,
        token: Address,
        oracle: Address,
        symbol: Symbol,
        external_asset: bool,
        oracle_asset: Option<Address>,
        periods_in_min: i32,
        volatility: i128,
        multiplier: i32,
    ) -> Result<(), Error> {
        if has_token_id(&env) {
            return Err(Error::AlreadyInitialized);
        }

        let periods_in_ledgers = periods_in_min * MIN_IN_LEDGERS as i32;

        env.storage().instance().set(&InstanceDataKey::Admin, &admin);
        put_oracle_id(&env, oracle);
        put_token_id(&env, token);
        write_genesis(&env);
        write_periods(&env, periods_in_ledgers);
        put_volatility(&env, volatility);
        put_multiplier(&env, multiplier);
        put_symbol(&env, symbol);
        put_external(&env, external_asset);
        put_oracle_asset(&env, oracle_asset);
        Ok(())
    }
}

#[contractimpl]
impl Vault for Pool {
    fn deposit(env: Env, from: Address, amount: i128) -> Result<(), Error> {
        check_amount_gt_0(amount)?;

        let period = actual_period(&env);
        from.require_auth();
        bump_instance(&env);

        update_rewards(&env, from.clone(), period);
        transfer_in_pool(&env, &get_token_client(&env), &from, &amount);

        mint_shares(&env, from.clone(), amount, period);
        put_tot_liquidity(&env, get_tot_liquidity(&env, period) + amount, period);

        events::deposited(&env, from.clone(), amount, period);

        // ---- Retroshades: only when --features mercury ---------------------
        #[cfg(feature = "mercury")]
        {
            use crate::retroshade::DepositEvent;

            let current_ledger: u32 = env.ledger().sequence();
            let shares_to_mint = calculate_to_mint(
                &env,
                amount,
                get_tot_supply(&env, period),
                get_tot_liquidity(&env, period),
            );

            DepositEvent {
                from: from.clone(),
                amount,
                ledger: current_ledger,
                period,
                new_shares_minted: shares_to_mint,
            }
            .emit(&env);
        }
        // --------------------------------------------------------------------

        Ok(())
    }

    fn withdraw_matured(env: Env, addr: Address, period: i32) -> Result<(), Error> {
        addr.require_auth();
        bump_instance(&env);

        let paid = pay_matured(&env, addr.clone(), period)?;
        events::matured_withdrawn(&env, addr.clone(), paid, period);

        #[cfg(feature = "mercury")]
        {
            use crate::retroshade::WithdrawMaturedEvent;
            let current_ledger: u32 = env.ledger().sequence();

            WithdrawMaturedEvent {
                from: addr.clone(),
                paid,
                ledger: current_ledger,
                period,
            }
            .emit(&env);
        }

        Ok(())
    }

    fn update_fee_rewards(env: Env, addr: Address, period: i32) -> Result<(), Error> {
        bump_instance(&env);
        update_rewards(&env, addr, period);
        // (Your events & retroshades for rewards are handled inside update_rewards, per your comment.)
        Ok(())
    }

    fn withdraw(env: Env, addr: Address, period: i32) -> Result<(), Error> {
        let current_period = actual_period(&env);
        if period >= current_period {
            return Err(Error::CannotWithdraw);
        }

        let addr_balance = read_balance(&env, addr.clone(), period);
        if addr_balance == 0 {
            return Err(Error::NoBalance);
        }

        addr.require_auth();
        bump_instance(&env);

        update_rewards(&env, addr.clone(), period);

        let tot_liquidity = get_tot_liquidity(&env, period);
        let tot_shares = get_tot_supply(&env, period);
        let principal_value = calculate_principal_value(addr_balance, tot_liquidity, tot_shares);

        transfer(&env, &get_token_client(&env), &addr, &principal_value);
        burn_shares(&env, addr.clone(), addr_balance, period);

        events::withdrawn(&env, addr.clone(), addr_balance, period);

        #[cfg(feature = "mercury")]
        {
            use crate::retroshade::WithdrawEvent;
            let current_ledger: u32 = env.ledger().sequence();

            WithdrawEvent {
                from: addr.clone(),
                burnt_shares: addr_balance,
                amount_withdrawn: principal_value,
                ledger: current_ledger,
                period,
            }
            .emit(&env);
        }

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
    fn subscribe(e: Env, initiator: Address, amount: i128) -> Result<(), Error> {
        let current_period: i32 = actual_period(&e);

        if has_refund_particular(&e, initiator.clone(), current_period) {
            return Err(Error::AlreadySubscribed);
        }

        initiator.require_auth();

        let multiplier = get_multiplier(&e);
        let tot_liquidity = get_tot_liquidity(&e, current_period);
        let refund_global = read_refund_global(&e, current_period);
        let time_to_end = find_x(&e, current_period);

        let possible_amount_to_refund =
            calculate_refund(time_to_end as i128, amount, multiplier as i128);

        if refund_global + possible_amount_to_refund > tot_liquidity {
            return Err(Error::NotEnoughLiquidity);
        }

        transfer_in_pool(&e, &get_token_client(&e), &initiator, &amount);
        update_fee_per_share_universal(&e, amount, current_period);

        let symbol = get_symbol(&e);
        let external = get_external(&e);

        let reflector_price = if external {
            reflector::Client::new(&e, &get_oracle_id(&e)?)
                .lastprice(&reflector::Asset::Other(symbol))
                .ok_or(Error::NoPrice)?
                .price
        } else {
            if let Some(asset) = get_oracle_asset(&e) {
                reflector::Client::new(&e, &get_oracle_id(&e)?)
                    .lastprice(&reflector::Asset::Stellar(asset))
                    .ok_or(Error::NoPrice)?
                    .price
            } else {
                return Err(Error::NoPrice);
            }
        };

        write_refund_particular(
            &e,
            initiator.clone(),
            possible_amount_to_refund,
            reflector_price,
            current_period,
        );

        write_refund_global(&e, refund_global + possible_amount_to_refund, current_period);

        events::policy_purchase(&e, initiator, amount, current_period);
        Ok(())
    }

    fn claim_reward(e: Env, claimant: Address) -> Result<(), Error> {
        let current_period = actual_period(&e);
        claimant.require_auth();

        let refund = read_refund_particular(&e, claimant.clone(), current_period)
            .ok_or(Error::NoInsurance)?;

        let symbol = get_symbol(&e);
        let external = get_external(&e);

        let reflector_price = if external {
            reflector::Client::new(&e, &get_oracle_id(&e)?)
                .lastprice(&reflector::Asset::Other(symbol))
                .ok_or(Error::NoPrice)?
                .price
        } else {
            if let Some(asset) = get_oracle_asset(&e) {
                reflector::Client::new(&e, &get_oracle_id(&e)?)
                    .lastprice(&reflector::Asset::Stellar(asset))
                    .ok_or(Error::NoPrice)?
                    .price
            } else {
                return Err(Error::NoPrice);
            }
        };

        let volatility = get_volatility(&e)?;

        if refund.price + volatility < reflector_price || refund.price - volatility > reflector_price
        {
            transfer(&e, &get_token_client(&e), &claimant, &refund.amount);
            let tot_liquidity = get_tot_liquidity(&e, current_period);
            let refund_global = read_refund_global(&e, current_period);

            write_refund_global(&e, refund_global - refund.amount, current_period);
            put_tot_liquidity(&e, tot_liquidity - refund.amount, current_period);
            e.storage().persistent().remove(
                &PersistentDataKey::RefundParticular(BalanceObject::new(claimant.clone(), current_period)),
            );
        } else {
            return Err(Error::UnmetCondition);
        }

        bump_instance(&e);
        events::befenit_payout(&e, claimant, refund.amount, current_period);
        Ok(())
    }
}
