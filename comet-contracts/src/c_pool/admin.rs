// who owns the token TODO Documents

use soroban_sdk::{Address, Env};

use super::storage_types::DataKeyToken;
// dele
pub fn has_administrator(e: &Env) -> bool {
    let key = DataKeyToken::Admin;
    e.storage().has(&key)
}

fn read_administrator(e: &Env) -> Address {
    let key = DataKeyToken::Admin;
    e.storage().get_unchecked(&key).unwrap()
}

pub fn write_administrator(e: &Env, id: &Address) {
    let key = DataKeyToken::Admin;
    e.storage().set(&key, id);
}

pub fn check_admin(e: &Env, admin: &Address) {
    if admin != &read_administrator(e) {
        panic!("not authorized by admin")
    }
}
