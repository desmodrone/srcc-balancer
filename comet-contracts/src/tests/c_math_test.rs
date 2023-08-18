#![cfg(test)]

use crate::c_math::{
    calc_lp_token_amount_given_token_deposits_in,
    calc_lp_token_amount_given_token_withdrawal_amount, calc_spot_price,
    calc_token_deposits_in_given_lp_token_amount, calc_token_in_given_token_out,
    calc_token_out_given_token_in, calc_token_withdrawal_amount_given_lp_token_amount,
};

extern crate std;

#[test]
fn test_calc_spot_price() {
    let val = calc_spot_price(
        10 * 1e7 as i128,
        2 * 1e7 as i128,
        2 * 1e7 as i128,
        (0.7 * 1e7) as i128,
        (0.0001 * 1e7) as i128,
    )
    .unwrap();

    std::println!("Val1 = {}", val);
    assert!(val != 0, "result must be non-zero");
}

#[test]
fn test_calc_token_out_given_token_in() {
    let val = calc_token_out_given_token_in(
        1 * 1e7 as i128,
        (0.2 * 1e7) as i128,
        2 * 1e7 as i128,
        (0.7 * 1e7) as i128,
        (0.01 * 1e7) as i128,
        (0.001 * 1e7) as i128,
    );

    std::println!("Val2 = {}", val);
    assert!(val != 0, "result must be non-zero");
}

#[test]
fn test_calc_token_in_given_token_out() {
    let val = calc_token_in_given_token_out(
        1 * 1e7 as i128,
        (0.2 * 1e7) as i128,
        2 * 1e7 as i128,
        (0.7 * 1e7) as i128,
        (0.001 * 1e7) as i128,
        (0.0001 * 1e7) as i128,
    );

    std::println!("Val3 = {}", val);
    assert!(val != 0, "result must be non-zero");
}

#[test]
fn test_calc_lp_token_amount_given_token_deposits_in() {
    let val = calc_lp_token_amount_given_token_deposits_in(
        1 * 1e7 as i128,
        (0.2 * 1e7) as i128,
        20 * 1e7 as i128,
        (0.7 * 1e7) as i128,
        (0.001 * 1e7) as i128,
        (0.0001 * 1e7) as i128,
    );
    std::println!("Val4 = {}", val);
    assert!(val != 0, "result must be non-zero");
}

#[test]
fn test_calc_token_deposits_in_given_lp_token_amount() {
    let val = calc_token_deposits_in_given_lp_token_amount(
        1 * 1e7 as i128,
        (0.2 * 1e7) as i128,
        20 * 1e7 as i128,
        (0.7 * 1e7) as i128,
        (0.001 * 1e7) as i128,
        (0.0001 * 1e7) as i128,
    );
    std::println!("Val5 = {}", val);
    assert!(val != 0, "result must be non-zero");
}

#[test]
fn test_calc_lp_token_amount_given_token_withdrawal_amount() {
    let val = calc_lp_token_amount_given_token_withdrawal_amount(
        1 * 1e7 as i128,
        (0.2 * 1e7) as i128,
        20 * 1e7 as i128,
        (0.7 * 1e7) as i128,
        (0.001 * 1e7) as i128,
        (0.0001 * 1e7) as i128,
    );
    std::println!("Val6 = {}", val);
    assert!(val != 0, "result must be non-zero");
}

#[test]
fn test_calc_token_withdrawal_amount_given_lp_token_amount() {
    let val = calc_token_withdrawal_amount_given_lp_token_amount(
        1 * 1e7 as i128,
        (0.2 * 1e7) as i128,
        20 * 1e7 as i128,
        (0.7 * 1e7) as i128,
        (0.001 * 1e7) as i128,
        (0.0001 * 1e7) as i128,
    );
    std::println!("Val7 = {}", val);
    assert!(val != 0, "result must be non-zero");
}
