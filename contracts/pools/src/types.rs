use soroban_sdk::{contracterror, contracttype, Address};

#[derive(Clone)]
#[contracttype]
pub struct BalanceObject {
    address: Address,
    period: i32
}

#[derive(Clone)]
#[contracttype]
pub struct Insurance {
    pub amount: i128,
    pub price: i128
}

impl BalanceObject {
    pub fn new(address: Address, period: i32) -> Self {
        Self { 
            address, 
            period 
        }
    }
}

#[derive(Clone)]
#[contracttype]
pub enum InstanceDataKey {
    TokenId,
    GenesisPeriod,
    Periods,
    Oracle,
    Volatility,
    Admin,
    Multiplier
}

#[derive(Clone)]
#[contracttype]
pub enum PersistentDataKey {
    Balance(BalanceObject),
    Principal(BalanceObject),
    TotLiquidity(i32),
    TotSupply(i32),
    FeePerShareUniversal(i32),
    FeePerShareParticular(BalanceObject),
    MaturedFeesParticular(BalanceObject),
    RefundParticular(BalanceObject),
    RefundGlobal(i32)
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    AlreadyInitialized = 0,
    NotInitialized = 1,
    InvalidShareBalance = 2,
    NoFeesMatured = 3,
    BalanceLtSupply = 4,
    InvalidAmount = 5,
    CannotWithdraw = 6,
    NoBalance = 7,
    NotEnoughLiquidity = 8,
    NoInsurance = 9,
    NoPrice = 10,
    AlreadySubscribed = 11,
    UnmetCondition = 12
}