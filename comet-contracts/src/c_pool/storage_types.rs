use soroban_sdk::{contracttype, Address, Map, Vec};

#[contracttype]
#[derive(Clone, Default, Debug, Eq, PartialEq)]
pub struct Record {
    pub bound: bool,
    pub index: u32,
    pub denorm: i128,
    pub balance: i128,
}

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Factory,     // Address of the Factory Contract
    Controller,  // Address of the Controller Account
    SwapFee,     // i128
    TotalWeight, // i128
    AllTokenVec,
    AllRecordData,
    TokenShare,
    TotalShares,
    PublicSwap,
    Finalize,
}

#[derive(Clone)]
#[contracttype]
pub struct AllowanceDataKey {
    pub from: Address,
    pub spender: Address,
}

#[derive(Clone)]
#[contracttype]
pub enum DataKeyToken {
    Allowance(AllowanceDataKey),
    Balance(Address),
    Nonce(Address),
    State(Address),
    Admin,
    Decimals,
    Name,
    Symbol,
}
