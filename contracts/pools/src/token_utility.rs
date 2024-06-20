use crate::{
    storage::get_token_id,
};
use soroban_sdk::{token, Address, Env};

pub(crate) fn transfer(e: &Env, client: &token::Client, to: &Address, amount: &i128) {
    client.transfer(&e.current_contract_address(), to, amount);
}

pub(crate) fn get_token_client(e: &Env) -> token::Client {
    token::Client::new(
        e,
        &get_token_id(e).unwrap(), // safe
                                   // only called when
                                   // execution already
                                   // knows that the contract
                                   // is initialized
    )
}

pub(crate) fn transfer_in_pool(env: &Env, client: &token::Client, from: &Address, amount: &i128) {
    client.transfer(from, &env.current_contract_address(), amount);
}
