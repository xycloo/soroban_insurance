use crate::{
    events,
    math::{compute_fee_earned, compute_fee_per_share},
    storage::*,
    token_utility::{get_token_client, transfer},
    types::Error,
};
use core::ops::AddAssign;
use soroban_sdk::{Address, Env};

// retroshade
mod retroshade {
    use retroshade_sdk::Retroshade;
    use soroban_sdk::{contracttype, Address, Symbol};

    #[derive(Retroshade)]
    #[contracttype]
    pub struct UpdateRewardsEvent {
        pub from: Address,
        pub fees: i128,
        pub ledger: u32,
        pub period: i32,
    }
}

pub(crate) fn update_rewards(e: &Env, addr: Address, period: i32) {
    let fee_per_share_universal = get_fee_per_share_universal(&e, period);
    let lender_fees = compute_fee_earned(
        read_balance(e, addr.clone(), period),
        fee_per_share_universal,
        read_fee_per_share_particular(e, addr.clone(), period),
    );

    write_fee_per_share_particular(e, addr.clone(), fee_per_share_universal, period);

    let mut matured = read_matured_fees_particular(e, addr.clone(), period);
    matured.add_assign(lender_fees);

    write_matured_fees_particular(e, addr.clone(), matured, period);

    events::new_fees(e, addr.clone(), lender_fees, period);

    // retroshade
    let current_ledger = e.ledger().sequence();

    retroshade::UpdateRewardsEvent {
        from: addr.clone(),
        fees: lender_fees,
        ledger: current_ledger,
        period,
    }
    .emit(&e);
}

pub(crate) fn update_fee_per_share_universal(e: &Env, collected: i128, period: i32) {
    let fee_per_share_universal = get_fee_per_share_universal(e, period);
    let total_supply = get_tot_supply(e, period);

    // computing the new universal fee per share in light of the collected interest
    let adjusted_fee_per_share_universal =
        compute_fee_per_share(fee_per_share_universal, collected, total_supply);

    put_fee_per_share_universal(e, adjusted_fee_per_share_universal, period);
}

pub(crate) fn pay_matured(e: &Env, addr: Address, period: i32) -> Result<i128, Error> {
    let token_client = get_token_client(e);

    // collect all the fees matured by the lender `addr`
    let matured = read_matured_fees_particular(e, addr.clone(), period);

    if matured == 0 {
        return Err(Error::NoFeesMatured);
    }

    // transfer the matured yield to `addr` and update the particular matured fees storage slot
    transfer(e, &token_client, &addr, &matured);
    write_matured_fees_particular(e, addr, 0, period);

    Ok(matured)
}
