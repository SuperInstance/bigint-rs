//! # bigint-rs
//!
//! Arbitrary precision integer arithmetic library implementing Karatsuba multiplication,
//! modular exponentiation, and Miller-Rabin primality testing.
//!
//! ## Modules
//! - [`bigint`] - Core `BigInt` type and basic operations
//! - [`arithmetic`] - Addition, subtraction, multiplication, division
//! - [`karatsuba`] - Karatsuba multiplication algorithm
//! - [`modular`] - Modular arithmetic, modular exponentiation, modular inverse
//! - [`primality`] - Miller-Rabin primality testing and GCD

pub mod arithmetic;
pub mod bigint;
pub mod karatsuba;
pub mod modular;
pub mod primality;

pub use bigint::BigInt;

#[cfg(test)]
mod tests {
    use super::*;

    // ── Arithmetic tests ──────────────────────────────────────

    #[test]
    fn test_addition_simple() {
        let a = BigInt::from(123);
        let b = BigInt::from(456);
        assert_eq!(a + b, BigInt::from(579));
    }

    #[test]
    fn test_addition_with_carry() {
        let a = BigInt::from(999);
        let b = BigInt::from(1);
        assert_eq!(a + b, BigInt::from(1000));
    }

    #[test]
    fn test_addition_large() {
        let a = BigInt::from_str_radix("123456789012345678901234567890", 10);
        let b = BigInt::from_str_radix("987654321098765432109876543210", 10);
        let sum = &a + &b;
        assert_eq!(sum, BigInt::from_str_radix("1111111110111111111011111111100", 10));
    }

    #[test]
    fn test_subtraction_simple() {
        let a = BigInt::from(500);
        let b = BigInt::from(200);
        assert_eq!(a - b, BigInt::from(300));
    }

    #[test]
    fn test_subtraction_result_negative() {
        let a = BigInt::from(100);
        let b = BigInt::from(200);
        let result = &a - &b;
        assert!(result.is_negative());
        assert_eq!(result.abs(), BigInt::from(100));
    }

    #[test]
    fn test_subtraction_zero() {
        let a = BigInt::from(42);
        assert_eq!(&a - &a, BigInt::zero());
    }

    #[test]
    fn test_multiplication_simple() {
        let a = BigInt::from(12);
        let b = BigInt::from(34);
        assert_eq!(a * b, BigInt::from(408));
    }

    #[test]
    fn test_multiplication_by_zero() {
        let a = BigInt::from(12345);
        assert_eq!(&a * &BigInt::zero(), BigInt::zero());
    }

    #[test]
    fn test_multiplication_large() {
        let a = BigInt::from(99999);
        let b = BigInt::from(99999);
        assert_eq!(a * b, BigInt::from(9999800001u64));
    }

    #[test]
    fn test_division_simple() {
        let a = BigInt::from(100);
        let b = BigInt::from(7);
        let (q, r) = a.div_rem(&b);
        assert_eq!(q, BigInt::from(14));
        assert_eq!(r, BigInt::from(2));
    }

    #[test]
    fn test_division_exact() {
        let a = BigInt::from(100);
        let b = BigInt::from(25);
        let (q, r) = a.div_rem(&b);
        assert_eq!(q, BigInt::from(4));
        assert_eq!(r, BigInt::zero());
    }

    #[test]
    fn test_division_by_one() {
        let a = BigInt::from(42);
        let (q, r) = a.div_rem(&BigInt::from(1));
        assert_eq!(q, BigInt::from(42));
        assert_eq!(r, BigInt::zero());
    }

    #[test]
    fn test_negation() {
        let a = BigInt::from(42);
        let neg = -a;
        assert!(neg.is_negative());
        assert_eq!(-neg, BigInt::from(42));
    }

    // ── Karatsuba tests ───────────────────────────────────────

    #[test]
    fn test_karatsuba_matches_schoolbook_small() {
        let a = BigInt::from(12345);
        let b = BigInt::from(67890);
        let school = a.schoolbook_mul(&b);
        let karat = karatsuba::karatsuba_mul(&a, &b);
        assert_eq!(school, karat);
    }

    #[test]
    fn test_karatsuba_matches_schoolbook_medium() {
        let a = BigInt::from_str_radix("12345678901234567890", 10);
        let b = BigInt::from_str_radix("98765432109876543210", 10);
        let school = a.schoolbook_mul(&b);
        let karat = karatsuba::karatsuba_mul(&a, &b);
        assert_eq!(school, karat);
    }

    #[test]
    fn test_karatsuba_identity() {
        let a = BigInt::from(42);
        let result = karatsuba::karatsuba_mul(&a, &BigInt::from(1));
        assert_eq!(result, BigInt::from(42));
    }

    #[test]
    fn test_karatsuba_zero() {
        let a = BigInt::from(99999);
        let result = karatsuba::karatsuba_mul(&a, &BigInt::zero());
        assert_eq!(result, BigInt::zero());
    }

    // ── Modular arithmetic tests ──────────────────────────────

    #[test]
    fn test_mod_add() {
        let a = BigInt::from(17);
        let b = BigInt::from(20);
        let m = BigInt::from(10);
        assert_eq!(modular::mod_add(&a, &b, &m), BigInt::from(7));
    }

    #[test]
    fn test_mod_mul() {
        let a = BigInt::from(12);
        let b = BigInt::from(7);
        let m = BigInt::from(10);
        assert_eq!(modular::mod_mul(&a, &b, &m), BigInt::from(4));
    }

    #[test]
    fn test_mod_exp() {
        // 2^10 mod 1000 = 1024 mod 1000 = 24
        let base = BigInt::from(2);
        let exp = BigInt::from(10);
        let modulus = BigInt::from(1000);
        assert_eq!(
            modular::mod_exp(&base, &exp, &modulus),
            BigInt::from(24)
        );
    }

    #[test]
    fn test_mod_exp_fermat() {
        // Fermat's little theorem: 3^(17-1) mod 17 = 1
        let base = BigInt::from(3);
        let exp = BigInt::from(16);
        let modulus = BigInt::from(17);
        assert_eq!(
            modular::mod_exp(&base, &exp, &modulus),
            BigInt::from(1)
        );
    }

    #[test]
    fn test_mod_exp_large() {
        // 2^100 mod 10000007
        let base = BigInt::from(2);
        let exp = BigInt::from(100);
        let modulus = BigInt::from(10000007);
        let result = modular::mod_exp(&base, &exp, &modulus);
        assert!(result < modulus);
        // Verify via repeated squaring: 2^10 = 1024, 2^20 mod 10000007 = 1048576, etc.
        assert!(result > BigInt::zero());
    }

    #[test]
    fn test_mod_inverse() {
        // 3 * x ≡ 1 (mod 7) => x = 5
        let a = BigInt::from(3);
        let m = BigInt::from(7);
        let inv = modular::mod_inverse(&a, &m).unwrap();
        assert_eq!(inv, BigInt::from(5));
        let product = modular::mod_mul(&a, &inv, &m);
        assert_eq!(product, BigInt::from(1));
    }

    #[test]
    fn test_mod_inverse_no_inverse() {
        // 2 and 4 are not coprime, no inverse
        let a = BigInt::from(2);
        let m = BigInt::from(4);
        assert!(modular::mod_inverse(&a, &m).is_none());
    }

    // ── Primality tests ───────────────────────────────────────

    #[test]
    fn test_gcd() {
        assert_eq!(primality::gcd(&BigInt::from(48), &BigInt::from(18)), BigInt::from(6));
        assert_eq!(primality::gcd(&BigInt::from(17), &BigInt::from(13)), BigInt::from(1));
        assert_eq!(primality::gcd(&BigInt::from(100), &BigInt::from(0)), BigInt::from(100));
    }

    #[test]
    fn test_gcd_large() {
        let a = BigInt::from_str_radix("123456789012345678901234567890", 10);
        let b = BigInt::from_str_radix("987654321098765432109876543210", 10);
        let g = primality::gcd(&a, &b);
        assert!(g > BigInt::zero());
    }

    #[test]
    fn test_miller_rabin_small_primes() {
        let primes = [2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41, 43, 47];
        for p in primes {
            assert!(
                primality::is_probable_prime(&BigInt::from(p), 20),
                "{p} should be prime"
            );
        }
    }

    #[test]
    fn test_miller_rabin_composites() {
        let composites = [4, 6, 8, 9, 10, 12, 14, 15, 16, 18, 20, 21, 22, 24, 25];
        for c in composites {
            assert!(
                !primality::is_probable_prime(&BigInt::from(c), 20),
                "{c} should not be prime"
            );
        }
    }

    #[test]
    fn test_miller_rabin_carmichael() {
        // Carmichael numbers are false positives for Fermat but Miller-Rabin catches them
        // 561 = 3 * 11 * 17 (smallest Carmichael number)
        assert!(!primality::is_probable_prime(&BigInt::from(561), 20));
        // 1105 = 5 * 13 * 17
        assert!(!primality::is_probable_prime(&BigInt::from(1105), 20));
    }

    #[test]
    fn test_miller_rabin_medium_prime() {
        // Known prime: 104729 (the 10000th prime)
        assert!(primality::is_probable_prime(&BigInt::from(104729), 20));
    }

    #[test]
    fn test_miller_rabin_large_composite() {
        // 1000001 = 101 * 9901
        assert!(!primality::is_probable_prime(&BigInt::from(1000001), 20));
    }

    #[test]
    fn test_zero_and_one() {
        assert!(!primality::is_probable_prime(&BigInt::zero(), 5));
        assert!(!primality::is_probable_prime(&BigInt::from(1), 5));
    }

    #[test]
    fn test_bigint_from_str() {
        let n = BigInt::from_str_radix("123456789", 10);
        assert_eq!(n, BigInt::from(123456789));
    }

    #[test]
    fn test_bigint_comparison() {
        let a = BigInt::from(100);
        let b = BigInt::from(200);
        assert!(a < b);
        assert!(b > a);
        assert_eq!(a, BigInt::from(100));
    }

    #[test]
    fn test_bigint_display() {
        let n = BigInt::from(42);
        assert_eq!(format!("{n}"), "42");
        let neg = -BigInt::from(42);
        assert_eq!(format!("{neg}"), "-42");
    }
}
