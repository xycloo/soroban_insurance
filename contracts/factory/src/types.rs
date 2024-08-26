use soroban_sdk::{contracterror, contracttype, Address};

#[contracttype]
pub enum DataKey {
    PoolHash,
}

#[derive(Copy, Clone, Debug)]
#[contracterror]
#[repr(u32)]
pub enum Error {
    AlreadyInitialized = 0,
    NotInitialized = 1,
    NotAdmin = 2,
    PoolExists = 3,
    NoPool = 4,
}
