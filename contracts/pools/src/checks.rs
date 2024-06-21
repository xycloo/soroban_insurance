use crate::{math::actual_period, types::Error};
use soroban_sdk::{token::Client, Env};

/// Make sure that we're dealing with amounts > 0
pub(crate) fn check_amount_gt_0(amount: i128) -> Result<(), Error> {
    if amount <= 0 {
        return Err(Error::InvalidAmount);
    }

    Ok(())
}
