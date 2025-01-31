#![no_std]

mod contract;
mod events;
mod storage;
mod types;

mod pool {
    use soroban_sdk::contractimport;
    contractimport!(file = "../../target/wasm32-unknown-unknown/release/pools.wasm");
}
