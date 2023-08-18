use soroban_sdk::{contracttype, Address, Env, Symbol};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SwapEvent {
    pub caller: Address,
    pub token_in: Address,
    pub token_out: Address,
    pub token_amount_in: i128,
    pub token_amount_out: i128,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct JoinEvent {
    pub caller: Address,
    pub token_in: Address,
    pub token_amount_in: i128,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExitEvent {
    pub caller: Address,
    pub token_out: Address,
    pub token_amount_out: i128,
}


// Token Events 

pub fn incr_allow_event(e: &Env, from: Address, to: Address, amount: i128) {
    let topics = (Symbol::new(e, "incr_allow"), from, to);
    e.events().publish(topics, amount);
}


pub fn decr_allow_event(e: &Env, from: Address, to: Address, amount: i128) {
    let topics = (Symbol::new(e, "decr_allow"), from, to);
    e.events().publish(topics, amount);
}

pub fn transfer_event(e: &Env, from: Address, to: Address, amount: i128) {
    let topics = (Symbol::short("transfer"), from, to);
    e.events().publish(topics, amount);
}

pub fn mint_event(e: &Env, admin: Address, to: Address, amount: i128) {
    let topics = (Symbol::short("mint"), admin, to);
    e.events().publish(topics, amount);
}

pub fn clawback_event(e: &Env, admin: Address, from: Address, amount: i128) {
    let topics = (Symbol::short("clawback"), admin, from);
    e.events().publish(topics, amount);
}

pub fn set_auth_event(e: &Env, admin: Address, id: Address, authorize: bool) {
    let topics = (Symbol::short("set_auth"), admin, id);
    e.events().publish(topics, authorize);
}

pub fn set_admin_event(e: &Env, admin: Address, new_admin: Address) {
    let topics = (Symbol::short("set_admin"), admin);
    e.events().publish(topics, new_admin);
}

pub fn burn_event(e: &Env, from: Address, amount: i128) {
    let topics = (Symbol::short("burn"), from);
    e.events().publish(topics, amount);
}

