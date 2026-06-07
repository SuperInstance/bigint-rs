//! Addition, subtraction, and multiplication for [`BigInt`](crate::BigInt).
//!
//! Implements schoolbook algorithms. For large multiplication,
//! see the [`karatsuba`](crate::karatsuba) module.

use crate::bigint::{BigInt, BASE};

/// Adds two `BigInt` values.
pub fn add(a: &BigInt, b: &BigInt) -> BigInt {
    // Handle signs
    match (a.negative, b.negative) {
        (false, false) => add_abs(a, b),
        (true, true) => {
            let mut r = add_abs(a, b);
            r.negative = true;
            r
        }
        (false, true) => sub(a, &b.abs()),
        (true, false) => sub(b, &a.abs()),
    }
}

/// Subtracts `b` from `a`.
pub fn sub(a: &BigInt, b: &BigInt) -> BigInt {
    match (a.negative, b.negative) {
        (false, false) => {
            match a.abs_cmp(b) {
                std::cmp::Ordering::Equal => BigInt::zero(),
                std::cmp::Ordering::Greater => a.abs_sub(b),
                std::cmp::Ordering::Less => {
                    let mut r = b.abs_sub(a);
                    r.negative = true;
                    r
                }
            }
        }
        (true, true) => sub(&b.abs(), &a.abs()),
        (false, true) => add(a, &b.abs()),
        (true, false) => {
            let mut r = add(&a.abs(), b);
            r.negative = true;
            r
        }
    }
}

/// Multiplies two `BigInt` values using schoolbook algorithm.
pub fn mul(a: &BigInt, b: &BigInt) -> BigInt {
    a.schoolbook_mul(b)
}

/// Adds the absolute values of two positive `BigInt` values.
fn add_abs(a: &BigInt, b: &BigInt) -> BigInt {
    let max_len = a.digits.len().max(b.digits.len());
    let mut result = Vec::with_capacity(max_len + 1);
    let mut carry: u64 = 0;
    for i in 0..max_len {
        let da = if i < a.digits.len() { a.digits[i] } else { 0 };
        let db = if i < b.digits.len() { b.digits[i] } else { 0 };
        let sum = da + db + carry;
        result.push(sum % BASE);
        carry = sum / BASE;
    }
    if carry > 0 {
        result.push(carry);
    }
    let mut r = BigInt { digits: result, negative: false };
    r.normalize();
    r
}
