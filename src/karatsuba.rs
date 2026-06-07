//! Karatsuba multiplication algorithm.
//!
//! Implements the divide-and-conquer Karatsuba algorithm with O(n^log₂3) complexity,
//! falling back to schoolbook multiplication for small operands.

use crate::bigint::BigInt;

/// Threshold below which schoolbook multiplication is used.
const KARATSUBA_THRESHOLD: usize = 4;

/// Multiplies two non-negative `BigInt` values using Karatsuba's algorithm.
///
/// For operands with fewer than `KARATSUBA_THRESHOLD` digits, falls back to schoolbook.
/// Otherwise, splits each operand at the midpoint and recursively computes:
///   z₀ = low(a) × low(b)
///   z₂ = high(a) × high(b)
///   z₁ = (low(a) + high(a)) × (low(b) + high(b)) - z₀ - z₂
///   result = z₂ × B^(2m) + z₁ × B^m + z₀
pub fn karatsuba_mul(a: &BigInt, b: &BigInt) -> BigInt {
    if a.is_zero() || b.is_zero() {
        return BigInt::zero();
    }
    let result = karatsuba_inner(&a.digits, &b.digits);
    let mut r = BigInt { digits: result, negative: a.negative != b.negative };
    r.normalize();
    r
}

/// Inner Karatsuba on raw digit vectors (base 10^9).
fn karatsuba_inner(a: &[u64], b: &[u64]) -> Vec<u64> {
    let n = a.len().max(b.len());
    if n < KARATSUBA_THRESHOLD {
        return schoolbook_digits(a, b);
    }
    let m = n / 2;
    let (a_low, a_high) = split_at(a, m);
    let (b_low, b_high) = split_at(b, m);

    let z0 = karatsuba_inner(&a_low, &b_low);
    let z2 = karatsuba_inner(&a_high, &b_high);

    let a_sum = add_digits(&a_low, &a_high);
    let b_sum = add_digits(&b_low, &b_high);
    let z1_full = karatsuba_inner(&a_sum, &b_sum);
    let z1 = sub_digits(&sub_digits(&z1_full, &z0), &z2);

    // result = z0 + z1 * BASE^m + z2 * BASE^(2m)
    let mut result = vec![0u64; a.len() + b.len() + 1];
    add_to(&mut result, &z0, 0);
    add_to(&mut result, &z1, m);
    add_to(&mut result, &z2, 2 * m);
    trim_zeros(result)
}

/// Splits a digit slice at position `m`, padding the high part.
fn split_at(digits: &[u64], m: usize) -> (Vec<u64>, Vec<u64>) {
    let low = if digits.len() > m {
        digits[..m].to_vec()
    } else {
        let mut v = digits.to_vec();
        v.resize(m, 0);
        v
    };
    let high = if digits.len() > m {
        digits[m..].to_vec()
    } else {
        vec![]
    };
    (low, high)
}

/// Adds two digit vectors.
fn add_digits(a: &[u64], b: &[u64]) -> Vec<u64> {
    let max_len = a.len().max(b.len());
    let mut result = Vec::with_capacity(max_len + 1);
    let mut carry: u64 = 0;
    for i in 0..max_len {
        let da = if i < a.len() { a[i] } else { 0 };
        let db = if i < b.len() { b[i] } else { 0 };
        let sum = da + db + carry;
        result.push(sum % crate::bigint::BASE);
        carry = sum / crate::bigint::BASE;
    }
    if carry > 0 {
        result.push(carry);
    }
    result
}

/// Subtracts digit vector `b` from `a` (assumes a >= b).
fn sub_digits(a: &[u64], b: &[u64]) -> Vec<u64> {
    let mut result = Vec::with_capacity(a.len());
    let mut borrow: i64 = 0;
    let base = crate::bigint::BASE as i64;
    for i in 0..a.len() {
        let da = a[i] as i64;
        let db = if i < b.len() { b[i] as i64 } else { 0 };
        let mut diff = da - db - borrow;
        if diff < 0 {
            diff += base;
            borrow = 1;
        } else {
            borrow = 0;
        }
        result.push(diff as u64);
    }
    trim_zeros(result)
}

/// Schoolbook multiplication on raw digit vectors.
fn schoolbook_digits(a: &[u64], b: &[u64]) -> Vec<u64> {
    let base = crate::bigint::BASE;
    let mut result = vec![0u64; a.len() + b.len()];
    for (i, &da) in a.iter().enumerate() {
        let mut carry: u64 = 0;
        for (j, &db) in b.iter().enumerate() {
            let prod = result[i + j] + da * db + carry;
            result[i + j] = prod % base;
            carry = prod / base;
        }
        let mut k = i + b.len();
        while carry > 0 {
            let sum = result[k] + carry;
            result[k] = sum % base;
            carry = sum / base;
            k += 1;
        }
    }
    trim_zeros(result)
}

/// Adds `src` to `dst` starting at offset `offset`.
fn add_to(dst: &mut [u64], src: &[u64], offset: usize) {
    let base = crate::bigint::BASE;
    let mut carry: u64 = 0;
    for (i, &s) in src.iter().enumerate() {
        let sum = dst[offset + i] + s + carry;
        dst[offset + i] = sum % base;
        carry = sum / base;
    }
    let mut k = offset + src.len();
    while carry > 0 && k < dst.len() {
        let sum = dst[k] + carry;
        dst[k] = sum % base;
        carry = sum / base;
        k += 1;
    }
}

/// Removes trailing zeros from a digit vector.
fn trim_zeros(mut v: Vec<u64>) -> Vec<u64> {
    while v.last() == Some(&0) {
        v.pop();
    }
    v
}
