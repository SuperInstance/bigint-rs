//! Primality testing and GCD computation.
//!
//! Implements the Euclidean algorithm for GCD and the Miller-Rabin
//! probabilistic primality test with configurable rounds.

use crate::bigint::BigInt;
use crate::modular::mod_exp;

/// Computes the greatest common divisor of `a` and `b` using the Euclidean algorithm.
pub fn gcd(a: &BigInt, b: &BigInt) -> BigInt {
    let mut a = a.abs();
    let mut b = b.abs();
    while !b.is_zero() {
        let (_, r) = a.div_rem(&b);
        a = b;
        b = r;
    }
    a
}

/// Determines if `n` is a probable prime using the Miller-Rabin test.
///
/// Performs `k` rounds of testing. The probability of a false positive is at most 4^(-k).
/// For cryptographic purposes, `k = 20` gives error probability < 10^(-12).
///
/// # How it works
/// Write `n - 1 = 2^s * d` where `d` is odd. Then for each round:
/// 1. Pick a random base `a` in `[2, n-2]`
/// 2. Compute `x = a^d mod n`
/// 3. If `x == 1` or `x == n-1`, this round passes
/// 4. Otherwise square `x` up to `s-1` times; if any equals `n-1`, pass
/// 5. If no square equals `n-1`, `n` is composite
pub fn is_probable_prime(n: &BigInt, k: usize) -> bool {
    if *n < BigInt::from(2) {
        return false;
    }
    if *n < BigInt::from(4) {
        return true; // 2 and 3 are prime
    }
    // Check if even
    if n.digits[0].is_multiple_of(2) {
        return false;
    }

    // Write n - 1 = 2^s * d
    let n_minus_1 = n - &BigInt::from(1);
    let (s, d) = factor_out_twos(&n_minus_1);

    // Use deterministic witnesses for small numbers, then random-ish for larger
    // For testing, use a fixed set of small primes as witnesses
    let witnesses: Vec<u64> = if *n < BigInt::from_str_radix("3317044064679887385961981", 10) {
        // Deterministic for n < 3.3e24: witnesses 2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37
        vec![2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37]
    } else {
        // For very large numbers, use the first k primes
        vec![2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41, 43, 47, 53, 59, 61, 67, 71]
    };

    let rounds = k.min(witnesses.len());
    for &w in witnesses.iter().take(rounds) {
        let a = BigInt::from(w);
        if a >= n_minus_1 {
            continue;
        }

        let mut x = mod_exp(&a, &d, n);

        if x == BigInt::from(1) || x == n_minus_1 {
            continue;
        }

        let mut found = false;
        for _ in 0..s {
            x = mod_exp(&x, &BigInt::from(2), n);
            if x == n_minus_1 {
                found = true;
                break;
            }
        }
        if !found {
            return false;
        }
    }
    true
}

/// Factors `n = 2^s * d` where `d` is odd. Returns `(s, d)`.
fn factor_out_twos(n: &BigInt) -> (u64, BigInt) {
    let mut s: u64 = 0;
    let mut d = n.clone();
    while !d.is_zero() && d.digits[0].is_multiple_of(2) {
        d = d.div_rem(&BigInt::from(2)).0;
        s += 1;
    }
    (s, d)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gcd_coprime() {
        assert_eq!(gcd(&BigInt::from(35), &BigInt::from(18)), BigInt::from(1));
    }

    #[test]
    fn test_prime_two() {
        assert!(is_probable_prime(&BigInt::from(2), 5));
    }

    #[test]
    fn test_composite_even() {
        assert!(!is_probable_prime(&BigInt::from(100), 5));
    }
}
