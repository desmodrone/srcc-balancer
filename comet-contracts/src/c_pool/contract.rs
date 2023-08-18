use core::ops::Add;

use super::{
    metadata::{
        get_token_share, get_total_shares, put_total_shares, read_controller, read_factory,
        read_record, read_swap_fee, read_tokens, read_total_weight, write_record, write_tokens,
        write_total_weight,
    },
    storage_types::{DataKey, Record},
};

use super::{
    admin::{check_admin, has_administrator, write_administrator},
    allowance::{read_allowance, spend_allowance, write_allowance},
    balance::{is_authorized, read_balance, receive_balance, spend_balance, write_authorization},
    events::{
        burn_event, clawback_event, decr_allow_event, incr_allow_event, mint_event,
        set_admin_event, set_auth_event, transfer_event,
    },
    metadata::{read_decimal, read_name, read_symbol, write_decimal, write_name, write_symbol},
};

use crate::{
    c_consts::{
        EXIT_FEE, INIT_POOL_SUPPLY, MAX_BOUND_TOKENS, MAX_FEE, MAX_IN_RATIO, MAX_OUT_RATIO,
        MAX_TOTAL_WEIGHT, MAX_WEIGHT, MIN_BALANCE, MIN_BOUND_TOKENS, MIN_FEE, MIN_WEIGHT,
    },
    c_math::{
        self, calc_lp_token_amount_given_token_deposits_in,
        calc_lp_token_amount_given_token_withdrawal_amount, calc_spot_price,
        calc_token_deposits_in_given_lp_token_amount, calc_token_in_given_token_out,
        calc_token_out_given_token_in, calc_token_withdrawal_amount_given_lp_token_amount,
    },
    c_num::{c_add, c_div, c_mul, c_sub},
    c_pool::{
        events::{ExitEvent, JoinEvent, SwapEvent},
        metadata::{
            check_record_bound, put_token_share, read_finalize, read_public_swap, write_controller,
            write_factory, write_finalize, write_public_swap, write_swap_fee,
        },
    },
};
use soroban_sdk::{
    contractimpl, log, unwrap::UnwrapOptimized, vec, xdr::SurveyMessageResponseType, Address,
    Bytes, BytesN, Env, Map, Symbol, Vec,
};
// Token Interface
mod token {
    // interects with the Host
    soroban_sdk::contractimport!(file = "../soroban_token_spec.wasm");
}

pub struct CometPoolContract;

// abstraction from the code
pub trait TokenTrait {
    fn initialize(e: Env, admin: Address, decimal: u32, name: Bytes, symbol: Bytes);

    fn allowance(e: Env, from: Address, spender: Address) -> i128;

    fn incr_allow(e: Env, from: Address, spender: Address, amount: i128);

    fn decr_allow(e: Env, from: Address, spender: Address, amount: i128);

    fn balance(e: Env, id: Address) -> i128;

    fn spendable(e: Env, id: Address) -> i128;

    fn authorized(e: Env, id: Address) -> bool;

    fn xfer(e: Env, from: Address, to: Address, amount: i128);

    fn xfer_from(e: Env, spender: Address, from: Address, to: Address, amount: i128);

    fn burn(e: Env, from: Address, amount: i128);

    fn burn_from(e: Env, spender: Address, from: Address, amount: i128);

    fn clawback(e: Env, admin: Address, from: Address, amount: i128);

    fn set_auth(e: Env, admin: Address, id: Address, authorize: bool);

    fn mint(e: Env, admin: Address, to: Address, amount: i128);

    fn set_admin(e: Env, admin: Address, new_admin: Address);

    fn decimals(e: Env) -> u32;

    fn name(e: Env) -> Bytes;

    fn symbol(e: Env) -> Bytes;

    fn get_num_tokens(e: Env) -> u32;

    fn get_current_tokens(e: Env) -> Vec<Address>;

    fn get_final_tokens(e: Env) -> Vec<Address>;

    fn get_balance(e: Env, token: Address) -> i128;

    fn get_total_denormalized_weight(e: Env) -> i128;

    fn get_denormalized_weight(e: Env, token: Address) -> i128;

    fn get_normalized_weight(e: Env, token: Address) -> i128;

    fn get_spot_price(e: Env, token_in: Address, token_out: Address) -> i128;

    fn get_swap_fee(e: Env) -> i128;

    fn is_bound(e: Env, t: Address) -> bool;

    fn share_id(e: Env) -> BytesN<32>;

    fn is_public_swap(e: Env) -> bool;

    fn is_finalized(e: Env) -> bool;

    fn get_spot_price_sans_fee(e: Env, token_in: Address, token_out: Address) -> i128;

    fn set_swap_fee(e: Env, fee: i128, caller: Address);

    fn set_controller(e: Env, caller: Address, manager: Address);

    fn set_public_swap(e: Env, caller: Address, val: bool);

    fn init(e: Env, factory: Address, controller: Address, token_wasm_hash: BytesN<32>);

    fn get_controller(e: Env) -> Address;

    fn bind(e: Env, token: Address, balance: i128, denorm: i128, admin: Address);

    fn rebind(e: Env, token: Address, balance: i128, denorm: i128, admin: Address);

    fn finalize(e: Env);

    fn join_pool(e: Env, pool_amount_out: i128, max_amounts_in: Vec<i128>, user: Address);

    fn exit_pool(e: Env, pool_amount_in: i128, min_amounts_out: Vec<i128>, user: Address);

    fn swap_exact_amount_in(
        e: Env,
        token_in: Address,
        token_amount_in: i128,
        token_out: Address,
        min_amount_out: i128,
        max_price: i128,
        user: Address,
    ) -> (i128, i128);

    fn swap_exact_amount_out(
        e: Env,
        token_in: Address,
        max_amount_in: i128,
        token_out: Address,
        token_amount_out: i128,
        max_price: i128,
        user: Address,
    ) -> (i128, i128);

    fn dep_lp_tokn_amt_out_get_tokn_in(
        e: Env,
        token_in: Address,
        pool_amount_out: i128,
        max_amount_in: i128,
        user: Address,
    ) -> i128;

    fn wdr_tokn_amt_in_get_lp_tokns_out(
        e: Env,
        token_out: Address,
        pool_amount_in: i128,
        min_amount_out: i128,
        user: Address,
    ) -> i128;

    fn wdr_tokn_amt_out_get_lp_tokns_in(
        e: Env,
        token_out: Address,
        token_amount_out: i128,
        max_pool_amount_in: i128,
        user: Address,
    ) -> i128;

    fn dep_tokn_amt_in_get_lp_tokns_out(
        e: Env,
        token_in: Address,
        token_amount_in: i128,
        min_pool_amount_out: i128,
        user: Address,
    ) -> i128;
}

#[contractimpl]
impl TokenTrait for CometPoolContract {
    fn init(e: Env, factory: Address, controller: Address, token_wasm_hash: BytesN<32>) {
        assert!(!e.storage().has(&DataKey::Factory), "already initialized");

        write_factory(&e, factory);
        write_controller(&e, controller);
        let val: &Address = &e.current_contract_address();
        // Another Contract on Soroban
        let name = Bytes::from_slice(&e, b"Comet Pool Token");
        let symbol = Bytes::from_slice(&e, b"CPAL");

        put_token_share(&e, val.contract_id().unwrap());
        put_total_shares(&e, 0);
        write_swap_fee(&e, MIN_FEE);
        write_finalize(&e, false);
        write_public_swap(&e, false);

        Self::initialize(e, val.clone(), 7u32, name, symbol);
    }

    fn get_controller(e: Env) -> Address {
        read_controller(&e)
    }

    fn bind(e: Env, token: Address, balance: i128, denorm: i128, admin: Address) {
        assert!(!read_finalize(&e), "ERR_FINALIZED");
        assert!(!check_record_bound(&e, token.clone()), "ERR_IS_BOUND");
        let controller = read_controller(&e);
        controller.require_auth();
        assert!(read_tokens(&e).len() < MAX_BOUND_TOKENS, "ERR_MAX_TOKENS");
        let key = DataKey::AllTokenVec;
        let key_rec = DataKey::AllRecordData;

        let index = read_tokens(&e).len();

        let mut tokens_arr = read_tokens(&e);

        let mut record_map = e
            .storage()
            .get(&key_rec)
            .unwrap_or(Ok(Map::<Address, Record>::new(&e))) // if no members on vector
            .unwrap();

        let record = Record {
            bound: true,
            index,
            denorm: 0,
            balance: 0,
        };
        record_map.set(token.clone(), record);
        write_record(&e, record_map);
        tokens_arr.push_back(token.clone());
        write_tokens(&e, tokens_arr);

        Self::rebind(e, token, balance, denorm, admin);
    }

    fn rebind(e: Env, token: Address, balance: i128, denorm: i128, admin: Address) {
        assert!(!read_finalize(&e), "ERR_FINALIZED");
        let controller = read_controller(&e);
        controller.require_auth();
        assert!(read_tokens(&e).len() < MAX_BOUND_TOKENS, "ERR_MAX_TOKENS");
        assert!(check_record_bound(&e, token.clone()), "ERR_NOT_BOUND");
        assert!(denorm >= MIN_WEIGHT, "ERR_MIN_WEIGHT");
        assert!(denorm <= MAX_WEIGHT, "ERR_MAX_WEIGHT");
        assert!(balance >= MIN_BALANCE, "ERR_MIN_BALANCE");

        let mut record_map: Map<Address, Record> = read_record(&e);
        let mut record = record_map.get(token.clone()).unwrap().unwrap();
        let old_weight = record.denorm;
        let mut total_weight = read_total_weight(&e);

        if denorm > old_weight {
            total_weight = c_add(total_weight, c_sub(denorm, old_weight).unwrap()).unwrap();
            write_total_weight(&e, total_weight);
            if total_weight > MAX_TOTAL_WEIGHT {
                panic!("ERR_MAX_TOTAL_WEIGHT");
            }
        } else if denorm < old_weight {
            total_weight = c_sub(total_weight, c_sub(old_weight, denorm).unwrap()).unwrap();
            write_total_weight(&e, total_weight);
        }

        record.denorm = denorm;

        let old_balance = record.balance;
        record.balance = balance;

        if balance > old_balance {
            pull_underlying(&e, &token, admin, c_sub(balance, old_balance).unwrap());
        } else if balance < old_balance {
            let token_balance_withdrawn = c_sub(old_balance, balance).unwrap();
            let token_exit_fee = c_mul(token_balance_withdrawn, 0).unwrap();
            push_underlying(
                &e,
                &token,
                admin,
                c_sub(token_balance_withdrawn, token_exit_fee).unwrap(),
            );
            let factory = read_factory(&e);
            push_underlying(&e, &token, factory, token_exit_fee)
        }

        record_map.set(token, record);
        write_record(&e, record_map);
    }

    fn finalize(e: Env) {
        assert!(!read_finalize(&e), "ERR_FINALIZED");
        assert!(read_tokens(&e).len() > MIN_BOUND_TOKENS, "ERR_MIN_TOKENS");
        let controller = read_controller(&e);

        controller.require_auth();
        write_finalize(&e, true);
        write_public_swap(&e, true);
        mint_shares(e, controller, INIT_POOL_SUPPLY);
    }

    fn join_pool(e: Env, pool_amount_out: i128, max_amounts_in: Vec<i128>, user: Address) {
        assert!(read_finalize(&e), "ERR_NOT_FINALIZED");

        user.require_auth();

        let pool_total = get_total_shares(&e);
        let ratio = c_div(pool_amount_out, pool_total).unwrap();

        if ratio == 0 {
            panic!("ERR_MATH_APPROX")
        }
        let tokens = read_tokens(&e);
        let mut records = read_record(&e);
        for i in 0..tokens.len() {
            let t = tokens.get(i).unwrap().unwrap();
            let mut rec = records.get(t.clone()).unwrap().unwrap();
            let token_amount_in = c_mul(ratio, rec.balance).unwrap();
            if token_amount_in == 0 {
                panic!("ERR_MATH_APPROX")
            }

            if token_amount_in > max_amounts_in.get(i).unwrap().unwrap() {
                panic!("ERR_LIMIT_IN")
            }
            rec.balance = c_add(rec.balance, token_amount_in).unwrap();
            records.set(t.clone(), rec);
            // emit LOG_JOIN(msg.sender, t, tokenAmountIn);
            let event: JoinEvent = JoinEvent {
                caller: user.clone(),
                token_in: t.clone(),
                token_amount_in: token_amount_in,
            };
            e.events()
                .publish((Symbol::short("LOG"), Symbol::short("JOIN")), event);
            pull_underlying(&e, &t, user.clone(), token_amount_in);
        }

        write_record(&e, records);
        mint_shares(e, user, pool_amount_out);
    }

    fn exit_pool(e: Env, pool_amount_in: i128, min_amounts_out: Vec<i128>, user: Address) {
        user.require_auth();
        assert!(read_finalize(&e), "ERR_NOT_FINALIZED");
        let pool_total = get_total_shares(&e);
        let exit_fee = c_mul(pool_amount_in, EXIT_FEE).unwrap();
        let pai_after_exit_fee = c_sub(pool_amount_in, EXIT_FEE).unwrap();
        let ratio = c_div(pai_after_exit_fee, pool_total).unwrap();
        assert!(ratio != 0, "ERR_MATH_APPROX");

        pull_shares(&e, user.clone(), pool_amount_in);
        let share_contract_id = get_token_share(&e);
        push_shares(
            &e,
            Address::from_contract_id(&e, &share_contract_id),
            EXIT_FEE,
        );
        burn_shares(&e, pai_after_exit_fee);
        let tokens = read_tokens(&e);
        let mut records = read_record(&e);
        for i in 0..tokens.len() {
            let t = tokens.get(i).unwrap().unwrap();
            let mut rec = records.get(t.clone()).unwrap().unwrap();
            let token_amount_out = c_mul(ratio, rec.balance).unwrap();
            assert!(token_amount_out != 0, "ERR_MATH_APPROX");
            assert!(
                token_amount_out >= min_amounts_out.get(i).unwrap().unwrap(),
                "ERR_LIMIT_OUT"
            );
            rec.balance = c_sub(rec.balance, token_amount_out).unwrap();
            records.set(t.clone(), rec);
            let event: ExitEvent = ExitEvent {
                caller: user.clone(),
                token_out: t.clone(),
                token_amount_out: token_amount_out,
            };
            e.events()
                .publish((Symbol::short("LOG"), Symbol::short("EXIT")), event);
            push_underlying(&e, &t, user.clone(), token_amount_out)
        }

        write_record(&e, records);
    }

    fn swap_exact_amount_in(
        e: Env,
        token_in: Address,
        token_amount_in: i128,
        token_out: Address,
        min_amount_out: i128,
        max_price: i128,
        user: Address,
    ) -> (i128, i128) {
        assert!(read_public_swap(&e), "ERR_SWAP_NOT_PUBLIC");
        assert!(check_record_bound(&e, token_in.clone()), "ERR_NOT_BOUND");
        assert!(check_record_bound(&e, token_out.clone()), "ERR_NOT_BOUND");

        user.require_auth();
        let mut in_record = read_record(&e).get(token_in.clone()).unwrap().unwrap();
        let mut out_record = read_record(&e).get(token_out.clone()).unwrap().unwrap();
        assert!(
            token_amount_in <= c_mul(in_record.balance, MAX_IN_RATIO).unwrap(),
            "ERR_MAX_IN_RATIO"
        );

        let spot_price_before = calc_spot_price(
            in_record.balance,
            in_record.denorm,
            out_record.balance,
            out_record.denorm,
            read_swap_fee(&e),
        )
        .unwrap();
        assert!(spot_price_before <= max_price, "ERR_BAD_LIMIT_PRICE");
        let token_amount_out = calc_token_out_given_token_in(
            in_record.balance,
            in_record.denorm,
            out_record.balance,
            out_record.denorm,
            token_amount_in,
            read_swap_fee(&e),
        );
        assert!(token_amount_out >= min_amount_out, "ERR_LIMIT_OUT");

        in_record.balance = c_add(in_record.balance, token_amount_in).unwrap();
        out_record.balance = c_sub(out_record.balance, token_amount_out).unwrap();

        let spot_price_after = calc_spot_price(
            in_record.balance,
            in_record.denorm,
            out_record.balance,
            out_record.denorm,
            read_swap_fee(&e),
        )
        .unwrap();

        assert!(spot_price_after >= spot_price_before, "ERR_MATH_APPROX");
        assert!(spot_price_after <= max_price, "ERR_LIMIT_PRICE");
        assert!(
            spot_price_before <= c_div(token_amount_in, token_amount_out).unwrap(),
            "ERR_MATH_APPROX"
        );

        let event: SwapEvent = SwapEvent {
            caller: user.clone(),
            token_in: token_in.clone(),
            token_out: token_out.clone(),
            token_amount_in: token_amount_in,
            token_amount_out: token_amount_out,
        };
        e.events()
            .publish((Symbol::short("LOG"), Symbol::short("SWAP")), event);

        pull_underlying(&e, &token_in, user.clone(), token_amount_in);
        push_underlying(&e, &token_out, user, token_amount_out);

        let mut record_map = read_record(&e);
        record_map.set(token_in, in_record);
        record_map.set(token_out, out_record);

        write_record(&e, record_map);

        (token_amount_out, spot_price_after)
    }

    fn swap_exact_amount_out(
        e: Env,
        token_in: Address,
        max_amount_in: i128,
        token_out: Address,
        token_amount_out: i128,
        max_price: i128,
        user: Address,
    ) -> (i128, i128) {
        assert!(check_record_bound(&e, token_in.clone()), "ERR_NOT_BOUND");
        assert!(check_record_bound(&e, token_out.clone()), "ERR_NOT_BOUND");
        assert!(read_public_swap(&e), "ERR_SWAP_NOT_PUBLIC");

        user.require_auth();
        let mut in_record = read_record(&e).get(token_in.clone()).unwrap().unwrap();
        let mut out_record = read_record(&e).get(token_out.clone()).unwrap().unwrap();
        assert!(
            token_amount_out <= c_mul(out_record.balance, MAX_OUT_RATIO).unwrap(),
            "ERR_MAX_IN_RATIO"
        );

        let spot_price_before = calc_spot_price(
            in_record.balance,
            in_record.denorm,
            out_record.balance,
            out_record.denorm,
            read_swap_fee(&e),
        )
        .unwrap();
        assert!(spot_price_before <= max_price, "ERR_BAD_LIMIT_PRICE");
        let token_amount_in = calc_token_in_given_token_out(
            in_record.balance,
            in_record.denorm,
            out_record.balance,
            out_record.denorm,
            token_amount_out,
            read_swap_fee(&e),
        );

        assert!(token_amount_in <= max_amount_in, "ERR_LIMIT_IN");

        in_record.balance = c_add(in_record.balance, token_amount_in).unwrap();
        out_record.balance = c_sub(out_record.balance, token_amount_out).unwrap();

        let spot_price_after = calc_spot_price(
            in_record.balance,
            in_record.denorm,
            out_record.balance,
            out_record.denorm,
            read_swap_fee(&e),
        )
        .unwrap();

        assert!(spot_price_after >= spot_price_before, "ERR_MATH_APPROX");
        assert!(spot_price_after <= max_price, "ERR_LIMIT_PRICE");
        assert!(
            spot_price_before <= c_div(token_amount_in, token_amount_out).unwrap(),
            "ERR_MATH_APPROX"
        );

        let event: SwapEvent = SwapEvent {
            caller: user.clone(),
            token_in: token_in.clone(),
            token_out: token_out.clone(),
            token_amount_in: token_amount_in,
            token_amount_out: token_amount_out,
        };
        e.events()
            .publish((Symbol::short("LOG"), Symbol::short("SWAP")), event);

        pull_underlying(&e, &token_in, user.clone(), token_amount_in);
        push_underlying(&e, &token_out, user, token_amount_out);

        let mut record_map = read_record(&e);
        record_map.set(token_in, in_record);
        record_map.set(token_out, out_record);

        write_record(&e, record_map);

        (token_amount_in, spot_price_after)
    }

    fn dep_tokn_amt_in_get_lp_tokns_out(
        e: Env,
        token_in: Address,
        token_amount_in: i128,
        min_pool_amount_out: i128,
        user: Address,
    ) -> i128 {
        assert!(read_finalize(&e), "ERR_NOT_FINALIZED");
        assert!(check_record_bound(&e, token_in.clone()), "ERR_NOT_BOUND");
        assert!(
            token_amount_in
                <= c_mul(
                    read_record(&e)
                        .get(token_in.clone())
                        .unwrap()
                        .unwrap()
                        .balance,
                    MAX_IN_RATIO
                )
                .unwrap(),
            "ERR_MAX_IN_RATIO"
        );
        let mut in_record = read_record(&e).get(token_in.clone()).unwrap().unwrap();
        let pool_amount_out = calc_lp_token_amount_given_token_deposits_in(
            in_record.balance,
            in_record.denorm,
            get_total_shares(&e),
            read_total_weight(&e),
            token_amount_in,
            read_swap_fee(&e),
        );
        assert!(pool_amount_out >= min_pool_amount_out, "ERR_LIMIT_OUT");
        in_record.balance = c_add(in_record.balance, token_amount_in).unwrap();

        let mut record_map = read_record(&e);
        record_map.set(token_in.clone(), in_record);
        write_record(&e, record_map);

        let event: JoinEvent = JoinEvent {
            caller: user.clone(),
            token_in: token_in.clone(),
            token_amount_in: token_amount_in,
        };
        e.events()
            .publish((Symbol::short("LOG"), Symbol::short("JOIN")), event);

        pull_underlying(&e, &token_in, user.clone(), token_amount_in);
        mint_shares(e, user.clone(), pool_amount_out);

        pool_amount_out
    }

    fn dep_lp_tokn_amt_out_get_tokn_in(
        e: Env,
        token_in: Address,
        pool_amount_out: i128,
        max_amount_in: i128,
        user: Address,
    ) -> i128 {
        assert!(read_finalize(&e), "ERR_NOT_FINALIZED");
        assert!(check_record_bound(&e, token_in.clone()), "ERR_NOT_BOUND");

        let mut in_record: Record = read_record(&e).get(token_in.clone()).unwrap().unwrap();

        let token_amount_in = calc_token_deposits_in_given_lp_token_amount(
            in_record.balance,
            in_record.denorm,
            get_total_shares(&e),
            read_total_weight(&e),
            pool_amount_out,
            read_swap_fee(&e),
        );
        assert!(token_amount_in != 0, "ERR_MATH_APPROX");
        assert!(token_amount_in <= max_amount_in, "ERR_LIMIT_IN");
        assert!(
            token_amount_in
                <= c_mul(
                    read_record(&e)
                        .get(token_in.clone())
                        .unwrap()
                        .unwrap()
                        .balance,
                    MAX_IN_RATIO
                )
                .unwrap(),
            "ERR_MAX_IN_RATIO"
        );
        in_record.balance = c_add(in_record.balance, token_amount_in).unwrap();

        let mut record_map = read_record(&e);
        record_map.set(token_in.clone(), in_record);
        write_record(&e, record_map);

        let event: JoinEvent = JoinEvent {
            caller: user.clone(),
            token_in: token_in.clone(),
            token_amount_in: token_amount_in,
        };
        e.events()
            .publish((Symbol::short("LOG"), Symbol::short("JOIN")), event);

        pull_underlying(&e, &token_in, user.clone(), token_amount_in);
        mint_shares(e, user.clone(), pool_amount_out);

        token_amount_in
    }

    fn wdr_tokn_amt_in_get_lp_tokns_out(
        e: Env,
        token_out: Address,
        pool_amount_in: i128,
        min_amount_out: i128,
        user: Address,
    ) -> i128 {
        user.require_auth();
        assert!(read_finalize(&e), "ERR_NOT_FINALIZED");
        assert!(check_record_bound(&e, token_out.clone()), "ERR_NOT_BOUND");

        let mut out_record: Record = read_record(&e).get(token_out.clone()).unwrap().unwrap();

        let token_amount_out = calc_token_withdrawal_amount_given_lp_token_amount(
            out_record.balance,
            out_record.denorm,
            get_total_shares(&e),
            read_total_weight(&e),
            pool_amount_in,
            read_swap_fee(&e),
        );

        assert!(token_amount_out >= min_amount_out, "ERR_LIMIT_OUT");
        assert!(
            token_amount_out
                <= c_mul(
                    read_record(&e)
                        .get(token_out.clone())
                        .unwrap()
                        .unwrap()
                        .balance,
                    MAX_OUT_RATIO
                )
                .unwrap(),
            "ERR_MAX_OUT_RATIO"
        );
        out_record.balance = c_sub(out_record.balance, token_amount_out).unwrap();
        let exit_fee = c_mul(pool_amount_in, EXIT_FEE).unwrap();

        let event: ExitEvent = ExitEvent {
            caller: user.clone(),
            token_out: token_out.clone(),
            token_amount_out: token_amount_out,
        };
        e.events()
            .publish((Symbol::short("LOG"), Symbol::short("EXIT")), event);

        pull_shares(&e, user.clone(), pool_amount_in);
        burn_shares(&e, c_sub(pool_amount_in, EXIT_FEE).unwrap());
        let factory = read_factory(&e);
        push_shares(&e, factory, EXIT_FEE);
        push_underlying(&e, &token_out, user, token_amount_out);

        let mut record_map = read_record(&e);
        record_map.set(token_out, out_record);
        write_record(&e, record_map);

        token_amount_out
    }

    fn wdr_tokn_amt_out_get_lp_tokns_in(
        e: Env,
        token_out: Address,
        token_amount_out: i128,
        max_pool_amount_in: i128,
        user: Address,
    ) -> i128 {
        user.require_auth();
        assert!(read_finalize(&e), "ERR_NOT_FINALIZED");
        assert!(check_record_bound(&e, token_out.clone()), "ERR_NOT_BOUND");
        assert!(
            token_amount_out
                <= c_mul(
                    read_record(&e)
                        .get(token_out.clone())
                        .unwrap()
                        .unwrap()
                        .balance,
                    MAX_OUT_RATIO
                )
                .unwrap(),
            "ERR_MAX_OUT_RATIO"
        );
        let mut out_record: Record = read_record(&e).get(token_out.clone()).unwrap().unwrap();
        let pool_amount_in = calc_lp_token_amount_given_token_withdrawal_amount(
            out_record.balance,
            out_record.denorm,
            get_total_shares(&e),
            read_total_weight(&e),
            token_amount_out,
            read_swap_fee(&e),
        );

        assert!(pool_amount_in != 0, "ERR_MATH_APPROX");
        assert!(pool_amount_in <= max_pool_amount_in, "ERR_LIMIT_IN");
        out_record.balance = c_sub(out_record.balance, token_amount_out).unwrap();
        let exit_fee = c_mul(pool_amount_in, EXIT_FEE).unwrap();
        let event: ExitEvent = ExitEvent {
            caller: user.clone(),
            token_out: token_out.clone(),
            token_amount_out: token_amount_out,
        };
        e.events()
            .publish((Symbol::short("LOG"), Symbol::short("EXIT")), event);

        pull_shares(&e, user.clone(), pool_amount_in);
        burn_shares(&e, c_sub(pool_amount_in, EXIT_FEE).unwrap());
        let factory = read_factory(&e);
        push_shares(&e, factory, EXIT_FEE);
        push_underlying(&e, &token_out, user, token_amount_out);

        pool_amount_in
    }

    fn set_swap_fee(e: Env, fee: i128, caller: Address) {
        assert!(!read_finalize(&e), "ERR_FINALIZED");
        assert!(fee >= MIN_FEE, "ERR_MIN_FEE");
        assert!(fee <= MAX_FEE, "ERR_MAX_FEED");
        assert!(caller == read_controller(&e), "ERR_NOT_CONTROLLER");
        caller.require_auth();
        write_swap_fee(&e, fee);
    }

    fn set_controller(e: Env, caller: Address, manager: Address) {
        assert!(caller == read_controller(&e), "ERR_NOT_CONTROLLER");
        caller.require_auth();
        write_controller(&e, manager);
    }

    fn set_public_swap(e: Env, caller: Address, val: bool) {
        assert!(caller == read_controller(&e), "ERR_NOT_CONTROLLER");
        assert!(read_finalize(&e), "ERR_NOT_FINALIZED");
        caller.require_auth();
        write_public_swap(&e, val);
    }

    fn get_total_denormalized_weight(e: Env) -> i128 {
        read_total_weight(&e)
    }

    fn get_num_tokens(e: Env) -> u32 {
        let token_vec = read_tokens(&e);
        token_vec.len()
    }

    fn get_current_tokens(e: Env) -> Vec<Address> {
        read_tokens(&e)
    }

    fn get_final_tokens(e: Env) -> Vec<Address> {
        assert!(read_finalize(&e), "ERR_NOT_FINALIZED");
        read_tokens(&e)
    }

    fn get_balance(e: Env, token: Address) -> i128 {
        let val = read_record(&e).get(token).unwrap().unwrap();
        assert!(val.bound, "ERR_NOT_BOUND");
        val.balance
    }

    fn get_denormalized_weight(e: Env, token: Address) -> i128 {
        assert!(check_record_bound(&e, token.clone()), "ERR_NOT_BOUND");
        let val = read_record(&e).get(token).unwrap().unwrap();
        val.denorm
    }

    fn get_normalized_weight(e: Env, token: Address) -> i128 {
        assert!(check_record_bound(&e, token.clone()), "ERR_NOT_BOUND");
        let val = read_record(&e).get(token).unwrap().unwrap();
        c_div(val.denorm, read_total_weight(&e)).unwrap()
    }

    fn get_spot_price(e: Env, token_in: Address, token_out: Address) -> i128 {
        let in_record = read_record(&e).get(token_in).unwrap().unwrap();
        let out_record: Record = read_record(&e).get(token_out).unwrap().unwrap();
        calc_spot_price(
            in_record.balance,
            in_record.denorm,
            out_record.balance,
            out_record.denorm,
            read_swap_fee(&e),
        )
        .unwrap()
    }

    fn get_swap_fee(e: Env) -> i128 {
        read_swap_fee(&e)
    }

    fn get_spot_price_sans_fee(e: Env, token_in: Address, token_out: Address) -> i128 {
        let in_record = read_record(&e).get(token_in).unwrap().unwrap();
        let out_record = read_record(&e).get(token_out).unwrap().unwrap();
        calc_spot_price(
            in_record.balance,
            in_record.denorm,
            out_record.balance,
            out_record.denorm,
            0,
        )
        .unwrap()
    }

    fn share_id(e: Env) -> BytesN<32> {
        get_token_share(&e)
    }

    fn is_public_swap(e: Env) -> bool {
        read_public_swap(&e)
    }

    fn is_finalized(e: Env) -> bool {
        read_finalize(&e)
    }

    fn is_bound(e: Env, t: Address) -> bool {
        read_record(&e).get(t).unwrap().unwrap().bound
    }

    fn initialize(e: Env, admin: Address, decimal: u32, name: Bytes, symbol: Bytes) {
        if has_administrator(&e) {
            panic!("already initialized")
        }
        write_administrator(&e, &admin);

        write_decimal(&e, u8::try_from(decimal).expect("Decimal must fit in a u8"));
        write_name(&e, name);
        write_symbol(&e, symbol);
    }

    fn allowance(e: Env, from: Address, spender: Address) -> i128 {
        read_allowance(&e, from, spender)
    }

    fn incr_allow(e: Env, from: Address, spender: Address, amount: i128) {
        from.require_auth();

        check_nonnegative_amount(amount);

        let allowance = read_allowance(&e, from.clone(), spender.clone());
        let new_allowance = allowance
            .checked_add(amount)
            .expect("Updated allowance doesn't fit in an i128");

        write_allowance(&e, from.clone(), spender.clone(), new_allowance);
        incr_allow_event(&e, from, spender, amount);
    }

    fn decr_allow(e: Env, from: Address, spender: Address, amount: i128) {
        from.require_auth();

        check_nonnegative_amount(amount);

        let allowance = read_allowance(&e, from.clone(), spender.clone());
        if amount >= allowance {
            write_allowance(&e, from.clone(), spender.clone(), 0);
        } else {
            write_allowance(&e, from.clone(), spender.clone(), allowance - amount);
        }
        decr_allow_event(&e, from, spender, amount);
    }

    fn balance(e: Env, id: Address) -> i128 {
        read_balance(&e, id)
    }

    fn spendable(e: Env, id: Address) -> i128 {
        read_balance(&e, id)
    }

    fn authorized(e: Env, id: Address) -> bool {
        is_authorized(&e, id)
    }

    fn xfer(e: Env, from: Address, to: Address, amount: i128) {
        from.require_auth();

        check_nonnegative_amount(amount);
        spend_balance(&e, from.clone(), amount);
        receive_balance(&e, to.clone(), amount);
        transfer_event(&e, from, to, amount);
    }

    fn xfer_from(e: Env, spender: Address, from: Address, to: Address, amount: i128) {
        spender.require_auth();

        check_nonnegative_amount(amount);
        spend_allowance(&e, from.clone(), spender, amount);
        spend_balance(&e, from.clone(), amount);
        receive_balance(&e, to.clone(), amount);
        transfer_event(&e, from, to, amount)
    }

    fn burn(e: Env, from: Address, amount: i128) {
        from.require_auth();

        check_nonnegative_amount(amount);
        spend_balance(&e, from.clone(), amount);
        burn_event(&e, from, amount);
    }

    fn burn_from(e: Env, spender: Address, from: Address, amount: i128) {
        spender.require_auth();

        check_nonnegative_amount(amount);
        spend_allowance(&e, from.clone(), spender, amount);
        spend_balance(&e, from.clone(), amount);
        burn_event(&e, from, amount)
    }

    fn clawback(e: Env, admin: Address, from: Address, amount: i128) {
        check_nonnegative_amount(amount);
        check_admin(&e, &admin);
        admin.require_auth();
        spend_balance(&e, from.clone(), amount);
        clawback_event(&e, admin, from, amount);
    }

    fn set_auth(e: Env, admin: Address, id: Address, authorize: bool) {
        check_admin(&e, &admin);
        admin.require_auth();
        write_authorization(&e, id.clone(), authorize);
        set_auth_event(&e, admin, id, authorize);
    }

    fn mint(e: Env, admin: Address, to: Address, amount: i128) {
        check_nonnegative_amount(amount);
        check_admin(&e, &admin);
        admin.require_auth();
        receive_balance(&e, to.clone(), amount);
        mint_event(&e, admin, to, amount);
    }

    fn set_admin(e: Env, admin: Address, new_admin: Address) {
        check_admin(&e, &admin);
        admin.require_auth();
        write_administrator(&e, &new_admin);
        set_admin_event(&e, admin, new_admin);
    }

    fn decimals(e: Env) -> u32 {
        read_decimal(&e)
    }

    fn name(e: Env) -> Bytes {
        read_name(&e)
    }

    fn symbol(e: Env) -> Bytes {
        read_symbol(&e)
    }
}

fn pull_underlying(e: &Env, token: &Address, from: Address, amount: i128) {
    token::Client::new(e, &token.contract_id().unwrap()).xfer_from(
        &e.current_contract_address(),
        &from,
        &e.current_contract_address(),
        &amount,
    );
}

fn push_underlying(e: &Env, token: &Address, to: Address, amount: i128) {
    token::Client::new(e, &token.contract_id().unwrap()).xfer(
        &e.current_contract_address(),
        &to,
        &amount,
    );
}

fn mint_shares(e: Env, to: Address, amount: i128) {
    let total = get_total_shares(&e);
    put_total_shares(&e, total + amount);
    let contract_address = e.current_contract_address();
    CometPoolContract::mint(e, contract_address, to, amount);
}

fn pull_shares(e: &Env, from: Address, amount: i128) {
    let share_contract_id = get_token_share(e);
    token::Client::new(e, &share_contract_id).xfer(&from, &e.current_contract_address(), &amount);
}

fn burn_shares(e: &Env, amount: i128) {
    let total = get_total_shares(e);
    let share_contract_id = get_token_share(e);
    token::Client::new(e, &share_contract_id).burn(&e.current_contract_address(), &amount);
    put_total_shares(e, total - amount);
}

fn push_shares(e: &Env, to: Address, amount: i128) {
    let share_contract_id = get_token_share(e);
    token::Client::new(e, &share_contract_id).xfer(&e.current_contract_address(), &to, &amount);
}

fn check_nonnegative_amount(amount: i128) {
    if amount < 0 {
        panic!("negative amount is not allowed: {}", amount)
    }
}
