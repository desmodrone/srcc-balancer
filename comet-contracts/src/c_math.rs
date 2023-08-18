use crate::{
    c_consts::{BONE, EXIT_FEE},
    c_num::{c_add, c_div, c_mul, c_pow, c_sub},
};

// Calculates the spot price for a token pair based on weights and balances for that pair of tokens, accounting for fees
pub fn calc_spot_price(
    token_balance_in: i128,
    token_weight_in: i128,
    token_balance_out: i128,
    token_weight_out: i128,
    swap_fee: i128,
) -> Result<i128, &'static str> {
    let numer = c_div(token_balance_in, token_weight_in)?;
    let denom = c_div(token_balance_out, token_weight_out)?;
    let ratio = c_div(numer, denom)?;
    let scale = c_div(BONE, c_sub(BONE, swap_fee)?)?;
    c_mul(ratio, scale)
}

// Calculates the amount of token B you get after a swap, given amount of token A are you swapping
pub fn calc_token_out_given_token_in(
    token_balance_in: i128,
    token_weight_in: i128,
    token_balance_out: i128,
    token_weight_out: i128,
    token_amount_in: i128,
    swap_fee: i128,
) -> i128 {
    let weight_ratio = c_div(token_weight_in, token_weight_out).unwrap();
    let adjusted_in = c_sub(BONE, swap_fee).unwrap();
    let adjusted_in = c_mul(token_amount_in, adjusted_in).unwrap();
    let y = c_div(
        token_balance_in,
        c_add(token_balance_in, adjusted_in).unwrap(),
    )
    .unwrap();
    let foo = c_pow(y, weight_ratio).unwrap();
    let bar = c_sub(BONE, foo).unwrap();
    let token_amount_out = c_mul(token_balance_out, bar).unwrap();
    token_amount_out
}

// Calculates the amount of token A you need to have, given amount of token B you want to get
pub fn calc_token_in_given_token_out(
    token_balance_in: i128,
    token_weight_in: i128,
    token_balance_out: i128,
    token_weight_out: i128,
    token_amount_out: i128,
    swap_fee: i128,
) -> i128 {
    let weight_ratio = c_div(token_weight_out, token_weight_in).unwrap();
    let diff = c_sub(token_balance_out, token_amount_out).unwrap();
    let y = c_div(token_balance_out, diff).unwrap();
    let mut foo = c_pow(y, weight_ratio).unwrap();
    foo = c_sub(foo, BONE).unwrap();
    let mut token_amount_in = c_sub(BONE, swap_fee).unwrap();
    token_amount_in = c_div(c_mul(token_balance_in, foo).unwrap(), token_amount_in).unwrap();
    token_amount_in
}

// Calculates the amount of LP tokens being minted to user, given how many deposit tokens a user deposits
pub fn calc_lp_token_amount_given_token_deposits_in(
    token_balance_in: i128,
    token_weight_in: i128,
    pool_supply: i128,
    total_weight: i128,
    token_amount_in: i128,
    swap_fee: i128,
) -> i128 {
    let normalized_weight = c_div(token_weight_in, total_weight).unwrap();
    let zaz = c_mul(c_sub(BONE, normalized_weight).unwrap(), swap_fee).unwrap();
    let token_amount_in_after_fee = c_mul(token_amount_in, c_sub(BONE, zaz).unwrap()).unwrap();

    let new_token_balance_in = c_add(token_balance_in, token_amount_in_after_fee).unwrap();
    let token_in_ratio = c_div(new_token_balance_in, token_balance_in).unwrap();

    let pool_ratio = c_pow(token_in_ratio, normalized_weight).unwrap();
    let new_pool_supply = c_mul(pool_ratio, pool_supply).unwrap();
    let pool_amount_out = c_sub(new_pool_supply, pool_supply).unwrap();
    pool_amount_out
}

// If a user wants some amount of LP tokens, this is how many tokens to deposit into the pool
pub fn calc_token_deposits_in_given_lp_token_amount(
    token_balance_in: i128,
    token_weight_in: i128,
    pool_supply: i128,
    total_weight: i128,
    pool_amount_out: i128,
    swap_fee: i128,
) -> i128 {
    let normalized_weight = c_div(token_weight_in, total_weight).unwrap();
    let new_pool_supply = c_add(pool_supply, pool_amount_out).unwrap();
    let pool_ratio = c_div(new_pool_supply, pool_supply).unwrap();

    let boo = c_div(BONE, normalized_weight).unwrap();
    let token_in_ratio = c_pow(pool_ratio, boo).unwrap();
    let new_token_balance_in = c_mul(token_in_ratio, token_balance_in).unwrap();
    let token_amount_in_after_fee = c_sub(new_token_balance_in, token_balance_in).unwrap();

    let zar = c_mul(c_sub(BONE, normalized_weight).unwrap(), swap_fee).unwrap();
    let token_amount_in = c_div(token_amount_in_after_fee, c_sub(BONE, zar).unwrap()).unwrap();
    token_amount_in
}

// Calculating the amount of LP tokens a user needs to burn, given how many deposit tokens they want to receive
pub fn calc_lp_token_amount_given_token_withdrawal_amount(
    token_balance_out: i128,
    token_weight_out: i128,
    pool_supply: i128,
    total_weight: i128,
    token_amount_out: i128,
    swap_fee: i128,
) -> i128 {
    let normalized_weight = c_div(token_weight_out, total_weight).unwrap();
    let zoo = c_sub(BONE, normalized_weight).unwrap();
    let zar = c_mul(zoo, swap_fee).unwrap();
    let token_amount_out_before_swap_fee =
        c_div(token_amount_out, c_sub(BONE, zar).unwrap()).unwrap();

    let new_token_balance_out = c_sub(token_balance_out, token_amount_out_before_swap_fee).unwrap();
    let token_out_ratio = c_div(new_token_balance_out, token_balance_out).unwrap();

    let pool_ratio = c_pow(token_out_ratio, normalized_weight).unwrap();
    let new_pool_supply = c_mul(pool_ratio, pool_supply).unwrap();
    let pool_amount_in_after_exit_fee = c_sub(pool_supply, new_pool_supply).unwrap();

    let pool_amount_in = c_div(
        pool_amount_in_after_exit_fee,
        c_sub(BONE, EXIT_FEE).unwrap(),
    )
    .unwrap();
    pool_amount_in
}

// Calculating the amount of deposit token returned, given how many LP tokens the user wants to burn
pub fn calc_token_withdrawal_amount_given_lp_token_amount(
    token_balance_out: i128,
    token_weight_out: i128,
    pool_supply: i128,
    total_weight: i128,
    pool_amount_in: i128,
    swap_fee: i128,
) -> i128 {
    let normalized_weight = c_div(token_weight_out, total_weight).unwrap();

    let pool_amount_in_after_exit_fee =
        c_mul(pool_amount_in, c_sub(BONE, EXIT_FEE).unwrap()).unwrap();
    let new_pool_supply = c_sub(pool_supply, pool_amount_in_after_exit_fee).unwrap();
    let pool_ratio = c_div(new_pool_supply, pool_supply).unwrap();

    let token_out_ratio = c_pow(pool_ratio, c_div(BONE, normalized_weight).unwrap()).unwrap();
    let new_token_balance_out = c_mul(token_out_ratio, token_balance_out).unwrap();

    let token_amount_out_before_swap_fee = c_sub(token_balance_out, new_token_balance_out).unwrap();

    let zaz = c_mul(c_sub(BONE, normalized_weight).unwrap(), swap_fee).unwrap();
    let token_amount_out =
        c_mul(token_amount_out_before_swap_fee, c_sub(BONE, zaz).unwrap()).unwrap();

    token_amount_out
}
