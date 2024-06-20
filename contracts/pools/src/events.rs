use soroban_sdk::{symbol_short, Address, Env};

pub(crate) fn deposited(env: &Env, from: Address, amount: i128, period: i32) {
    let topics = (symbol_short!("deposit"), from, period);
    env.events().publish(topics, amount);
}

pub(crate) fn matured_withdrawn(env: &Env, addr: Address, withdrawn: i128, period: i32) {
    let topics = (symbol_short!("collect"), addr, period);
    env.events().publish(topics, withdrawn);
}

pub(crate) fn new_fees(env: &Env, addr: Address, matured: i128, period: i32) {
    let topics = (symbol_short!("newfee"), addr, period);
    env.events().publish(topics, matured);
}

pub(crate) fn new_loss(env: &Env, addr: Address, matured: i128, period: i32) {
    let topics = (symbol_short!("newloss"), addr, period);
    env.events().publish(topics, matured);
}

pub(crate) fn withdrawn(env: &Env, from: Address, amount: i128, period: i32) {
    let topics = (symbol_short!("withdrawn"), from, period);
    env.events().publish(topics, amount);
}

pub(crate) fn loan_successful(env: &Env, receiver_contract: Address, amount: i128) {
    let topics = (symbol_short!("borrow"), receiver_contract);
    env.events().publish(topics, amount);
}