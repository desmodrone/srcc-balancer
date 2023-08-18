use c_consts::BONE;

use crate::c_consts::{self, CPOW_PRECISION, MAX_CPOW_BASE, MIN_CPOW_BASE};

fn c_toi(a: i128) -> i128 {
    a.checked_div(BONE).unwrap()
}

fn c_floor(a: i128) -> i128 {
    c_toi(a).checked_mul(BONE).unwrap()
}

// pub fn c_add(a: i128, b: i128) -> Result<i128, &'static str> {
//     let c = a.checked_add(b).ok_or("ERR_ADD_OVERFLOW")?;
//     Ok(c)
// }

pub fn c_add(a: i128, b: i128) -> Result<i128, &'static str> {
    if a < 0 && b < 0 {
        let c = a.checked_sub(-b).ok_or("ERR_ADD_OVERFLOW")?;
        Ok(c)
    } else {
        let c = a.checked_add(b).ok_or("ERR_ADD_OVERFLOW")?;
        Ok(c)
    }
}




// updated function
pub fn c_sub(a: i128, b: i128) -> Result<i128, &'static str> {
    match a.checked_sub(b) {
        Some(result) => Ok(result),
        None => {
            if a > 0 && b < 0 {
                Err("ERR_SUB_OVERFLOW")
            } else {
                Err("ERR_SUB_UNDERFLOW")
            }
        }
    }
}

pub fn c_sub_sign(a: i128, b: i128) -> (i128, bool) {
    if a >= b {
        (a.checked_sub(b).unwrap(), false)
    } else {
        (b.checked_sub(a).unwrap(), true)
    }
}

pub fn c_mul(a: i128, b: i128) -> Result<i128, &'static str> {
    let c0 = a.checked_mul(b).ok_or("ERR_MUL_OVERFLOW")?;
    let c1 = c0
        .checked_add(BONE.checked_div(2).unwrap())
        .ok_or("ERR_MUL_OVERFLOW")?;
    if c1 < c0 {
        return Err("ERR_MUL_OVERFLOW");
    }
    let c2 = c1.checked_div(BONE).unwrap();
    Ok(c2)
}

pub fn c_div(a: i128, b: i128) -> Result<i128, &'static str> {
    if b == 0 {
        return Err("ERR_DIV_ZERO");
    }
    let c0 = a.checked_mul(BONE).ok_or("ERR_DIV_INTERNAL")?;
    let c1 = c0
        .checked_add(b.checked_div(2).unwrap())
        .ok_or("ERR_DIV_INTERNAL")?;
    let c2 = c1.checked_div(b).ok_or("ERR_DIV_INTERNAL")?;

    Ok(c2)
}

pub fn c_powi(a: i128, n: i128) -> i128 {
    let mut z = if n % 2 != 0 { a } else { BONE };

    let mut a = a;
    let mut n = n.checked_div(2).unwrap();

    while n != 0 {
        a = c_mul(a, a).unwrap();

        if n % 2 != 0 {
            z = c_mul(z, a).unwrap();
        }

        n = n.checked_div(2).unwrap();
    }

    z
}

pub fn c_pow(base: i128, exp: i128) -> Result<i128, &'static str> {
    if base < MIN_CPOW_BASE {
        return Err("ERR_CPOW_BASE_TOO_LOW");
    }

    if base > MAX_CPOW_BASE {
        return Err("ERR_CPOW_BASE_TOO_HIGH");
    }

    let whole = c_floor(exp);

    let remain = c_sub(exp, whole).unwrap();

    let whole_pow = c_powi(base, c_toi(whole));

    if remain == 0 {
        return Ok(whole_pow);
    }

    let partial_result = c_pow_approx(base, remain, CPOW_PRECISION);
    Ok(c_mul(whole_pow, partial_result).unwrap())
}

pub fn c_pow_approx(base: i128, exp: i128, precision: i128) -> i128 {
    let a = exp;
    let (x, xneg) = c_sub_sign(base, BONE);
    let mut term = BONE;
    let mut sum = term;
    let mut negative = false;
    let mut i: i128 = 1;
    while term >= precision {
        let big_k = i.checked_mul(BONE).unwrap();
        let (c, cneg) = c_sub_sign(a, c_sub(big_k, BONE).unwrap());
        term = c_mul(term, c_mul(c, x).unwrap()).unwrap();
        term = c_div(term, big_k).unwrap();

        if term == 0 {
            break;
        }

        if xneg {
            negative = !negative;
        }

        if cneg {
            negative = !negative;
        }

        if negative {
            sum = c_sub(sum, term).unwrap();
        } else {
            sum = c_add(sum, term).unwrap();
        }

        i = i.checked_add(1).unwrap();
    }

    sum
}
