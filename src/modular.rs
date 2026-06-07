//! Modular arithmetic operations.
//!
//! Provides modular addition, multiplication, exponentiation (square-and-multiply),
//! and modular inverse via the extended Euclidean algorithm.

use crate::bigint::BigInt;
use crate::primality::gcd;

/// Computes `(a + b) mod m`.
///
/// All inputs are treated as non-negative; the result is in `[0, m)`.
pub fn mod_add(a: &BigInt, b: &BigInt, m: &BigInt) -> BigInt {
    let sum = crate::arithmetic::add(a, b);
    let result = mod_rem(&sum, m);
    if result.negative {
        crate::arithmetic::add(&result, m)
    } else {
        result
    }
}

/// Computes `(a * b) mod m`.
pub fn mod_mul(a: &BigInt, b: &BigInt, m: &BigInt) -> BigInt {
    let prod = crate::arithmetic::mul(a, b);
    let result = mod_rem(&prod, m);
    if result.negative {
        crate::arithmetic::add(&result, m)
    } else {
        result
    }
}

/// Computes `base^exp mod modulus` using binary exponentiation (square-and-multiply).
///
/// # Panics
/// Panics if `modulus` is zero.
pub fn mod_exp(base: &BigInt, exp: &BigInt, modulus: &BigInt) -> BigInt {
    assert!(!modulus.is_zero(), "modulus must be non-zero");
    if modulus == &BigInt::from(1) {
        return BigInt::zero();
    }
    let base = mod_rem(base, modulus);
    let mut result = BigInt::from(1);
    let mut base = base;
    // Iterate through bits of exp
    let exp = exp.abs();
    let mut bit_index: usize = 0;
    let total_bits = exp.digits.len() * 30; // conservative upper bound
    while bit_index < total_bits || bit_index < 1 {
        if get_bit(&exp, bit_index) {
            result = mod_mul(&result, &base, modulus);
        }
        base = mod_mul(&base, &base, modulus);
        bit_index += 1;
        // Check if remaining bits are all zero
        if bit_index >= total_bits {
            break;
        }
    }
    result
}

/// Computes the modular inverse of `a` modulo `m`, i.e. finds `x` such that `a*x ≡ 1 (mod m)`.
///
/// Returns `None` if `a` and `m` are not coprime.
pub fn mod_inverse(a: &BigInt, m: &BigInt) -> Option<BigInt> {
    let a = mod_rem(a, m);
    if gcd(&a, m) != BigInt::from(1) {
        return None;
    }
    // Extended Euclidean algorithm
    let (mut old_r, mut r) = (a.clone(), m.clone());
    let (mut old_s, mut _s) = (BigInt::from(1), BigInt::zero());

    while !r.is_zero() {
        let (q, rem) = old_r.div_rem(&r);
        old_r = r;
        r = rem;
        let new_s = &old_s - &(&q * &_s);
        old_s = _s;
        _s = new_s;
    }
    let mut result = old_s;
    if result.is_negative() {
        result = crate::arithmetic::add(&result, m);
    }
    Some(result)
}

/// Computes `a mod m` (remainder), with `a` possibly negative.
fn mod_rem(a: &BigInt, m: &BigInt) -> BigInt {
    let (_, r) = a.abs().div_rem(m);
    if a.negative && !r.is_zero() {
        m - &r
    } else {
        r
    }
}

/// Gets bit `i` from a positive BigInt's binary representation.
fn get_bit(n: &BigInt, i: usize) -> bool {
    let digit_idx = i / 30; // Each base-10^9 digit holds ~30 bits
    let bit_idx = i % 30;
    if digit_idx >= n.digits.len() {
        return false;
    }
    (n.digits[digit_idx] >> bit_idx) & 1 == 1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mod_rem_basic() {
        assert_eq!(mod_rem(&BigInt::from(17), &BigInt::from(5)), BigInt::from(2));
    }

    #[test]
    fn test_mod_rem_negative() {
        assert_eq!(mod_rem(&BigInt::from(-3), &BigInt::from(7)), BigInt::from(4));
    }
}
