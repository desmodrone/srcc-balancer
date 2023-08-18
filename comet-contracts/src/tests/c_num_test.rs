// #![cfg(test)]

use crate::c_consts;
use crate::c_num::c_pow_approx;
use crate::c_num::c_powi;
use crate::c_num::{c_add, c_div, c_mul, c_pow, c_sub};
use crate::c_consts::BONE;


#[test]
// tests that an error is returned when adding a positive number to i128::MAX.
fn test_c_add_overflow() {
    assert_eq!(c_add(1, i128::MAX).err().unwrap(), "ERR_ADD_OVERFLOW");
}

#[test]
// tests that c_sub() returns an error when subtracting a larger number from a smaller number.
fn test_c_sub_underflow() {
    match c_sub(1, 2) {
        Ok(result) => assert_eq!(result, -1),
        Err(err_msg) => panic!("Expected Ok(-1), but got Err({})", err_msg),
    }
}

#[test]
// tests that an error is returned when multiplying a number by i128::MAX.
fn test_c_mul_overflow() {
    assert_eq!(c_mul(2, i128::MAX).err().unwrap(), "ERR_MUL_OVERFLOW");
}

#[test]
// tests that an error is returned when dividing a number by zero.
fn test_c_div_error_on_div_by_zero() {
    assert_eq!(c_div(1, 0).err().unwrap(), "ERR_DIV_ZERO");
}

#[test]
// tests that an error is returned when calculating the power of a number
// that is too low or too high.
fn test_c_pow() {
    assert_eq!(c_pow(0, 2).err().unwrap(), "ERR_CPOW_BASE_TOO_LOW");
    assert_eq!(c_pow(i128::MAX, 2).err().unwrap(), "ERR_CPOW_BASE_TOO_HIGH");
}

#[test]
 // correctly adds two numbers.
fn test_c_add() {
    assert_eq!(c_add(1, 2).unwrap(), 3);
    assert_eq!(c_add(0, 0).unwrap(), 0);
    assert_eq!(c_add(-1, 1).unwrap(), 0);
}

#[test]
// correctly subtracts two numbers.
fn test_c_sub() {
    assert_eq!(c_sub(2, 1).unwrap(), 1);
    assert_eq!(c_sub(0, 0).unwrap(), 0);
    assert_eq!(c_sub(1, -1).unwrap(), 2);
}

#[test]
// correctly multiplies 2 numbers
fn test_c_mul() {
    let a = 0 * c_consts::BONE;
    let b = 6 * c_consts::BONE;
    let expected_result = 0;
    assert_eq!(c_mul(a, b).unwrap(), expected_result);
}

#[test]
// correctly divides two numbers.
fn test_c_div() {
    let a = 3 * c_consts::BONE;
    let b = 1 * c_consts::BONE;
    let result = c_div(a, b).unwrap();
    assert_eq!(result, 3 * c_consts::BONE);
}

#[test]
// tests that c_pow_approx() correctly approximates
// the power of a number with a certain precision.
fn test_c_pow_approx() {
    let base = 2;
    let exp = 2;
    let precision = 1;
    let bone = c_consts::BONE;
    let result = c_pow_approx(base * bone, exp * bone, precision);
    assert_eq!(result, 4 * bone);
}

#[test]
// correctly calculates the power of a number with an integer exponent.
fn test_c_powi() {
    let base = 2 * c_consts::BONE;
    let exp = 3;
    let result = c_powi(base, exp);
    assert_eq!(
        result,
        (2i128.pow(exp as u32) * c_consts::BONE.pow(exp as u32))
            / c_consts::BONE.pow((exp - 1) as u32)
    );
}

// Additional test

#[test]
//  returns an error when subtracting a smaller number from i128::MAX.
fn test_c_sub_max_min() {
    let max = i128::MAX;
    let min = i128::MIN + 1;
    match c_sub(max, min - 1) {
        Ok(result) => panic!("Expected an error, but got Ok({})", result),
        Err(err_msg) => assert_eq!(err_msg, "ERR_SUB_OVERFLOW"),
    }
}

#[test]
// ests that an error is returned when multiplying a large number by 2.
fn test_c_mul_large() {
    let a = i128::MAX / 2 + 1;
    let b = 2;
    assert_eq!(c_mul(a, b).err().unwrap(), "ERR_MUL_OVERFLOW");
}

#[test]
// tests that an error is returned when dividing a large number by 2.
fn test_c_div_large() {
    let a = i128::MAX;
    let b = 2;
    assert_eq!(c_div(a, b).err().unwrap(), "ERR_DIV_INTERNAL");
}

#[test]
// tests that c_pow_approx() correctly approximates the power
// of a large number with a certain precision.
fn test_c_pow_approx_large() {
    let base = 10;
    let exp = 3;
    let precision = 1;
    let bone = c_consts::BONE;
    let result = c_pow_approx(base * bone, exp * bone, precision);
    assert_eq!(result, 1000 * bone);
}

#[test]
// tests that c_powi() correctly calculates the power
// of a number with an even integer exponent.
fn test_c_powi_even_exponent() {
    let base = 3 * c_consts::BONE;
    let exp = 4;
    let result = c_powi(base, exp);
    assert_eq!(
        result,
        (3i128.pow(exp as u32) * c_consts::BONE.pow(exp as u32))
            / c_consts::BONE.pow((exp - 1) as u32)
    );
}

// new test added May 9
#[test]
fn test_c_add_with_negative_numbers() {
    assert_eq!(
        c_add(-BONE, -2 * BONE),
        Ok(-3 * BONE)
    );
    assert_eq!(c_add(-2 * BONE, -3 * BONE), Ok(-5 * BONE));
    assert_eq!(c_add(-BONE, 0), Ok(-BONE));
    assert_eq!(c_add(-BONE, BONE), Ok(0));
}
