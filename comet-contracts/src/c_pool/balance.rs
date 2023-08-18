use soroban_sdk::{Address, Env};

use super::storage_types::DataKeyToken;

pub fn read_balance(e: &Env, addr: Address) -> i128 {
    let key = DataKeyToken::Balance(addr);
    if let Some(balance) = e.storage().get(&key) {
        balance.unwrap()
    } else {
        0
    }
}

fn write_balance(e: &Env, addr: Address, amount: i128) {
    let key = DataKeyToken::Balance(addr);
    e.storage().set(&key, &amount);
}

pub fn receive_balance(e: &Env, addr: Address, amount: i128) {
    let balance = read_balance(e, addr.clone());
    if !is_authorized(e, addr.clone()) {
        panic!("can't receive when deauthorized");
    }
    write_balance(e, addr, balance + amount);
}
// addr is account
pub fn spend_balance(e: &Env, addr: Address, amount: i128) {
    let balance = read_balance(e, addr.clone());
    if !is_authorized(e, addr.clone()) {
        panic!("can't spend when deauthorized");
    }
    if balance < amount {
        panic!("insufficient balance");
    }
    write_balance(e, addr, balance - amount);
}

// why is this used?
pub fn is_authorized(e: &Env, addr: Address) -> bool {
    let key = DataKeyToken::State(addr);
    if let Some(state) = e.storage().get(&key) {
        state.unwrap()
    } else {
        true
    }
}

pub fn write_authorization(e: &Env, addr: Address, is_authorized: bool) {
    let key = DataKeyToken::State(addr);
    e.storage().set(&key, &is_authorized);
}
