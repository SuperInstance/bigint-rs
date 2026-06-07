//! Core [`BigInt`] type and basic operations.
//!
//! Stores arbitrary-precision integers as a vector of base-10^9 digits
//! in little-endian order, with a sign flag.

use std::cmp::Ordering;
use std::fmt;

/// Base for internal digit representation (10^9).
pub(crate) const BASE: u64 = 1_000_000_000;

/// An arbitrary-precision signed integer.
///
/// Internally stored as little-endian base-10^9 digits with a sign flag.
/// Zero is always represented as positive with empty digits.
#[derive(Clone, Debug)]
pub struct BigInt {
    /// Digits in base 10^9, least significant first.
    pub(crate) digits: Vec<u64>,
    /// `true` if the number is negative.
    pub(crate) negative: bool,
}

impl BigInt {
    /// Creates a new `BigInt` representing zero.
    ///
    /// # Examples
    /// ```
    /// use bigint_rs::BigInt;
    /// let z = BigInt::zero();
    /// assert!(z.is_zero());
    /// ```
    pub fn zero() -> Self {
        BigInt { digits: vec![], negative: false }
    }

    /// Creates a `BigInt` from a `u64`.
    pub fn from_u64(n: u64) -> Self {
        if n == 0 {
            return Self::zero();
        }
        let mut digits = Vec::new();
        let mut n = n;
        while n > 0 {
            digits.push(n % BASE);
            n /= BASE;
        }
        BigInt { digits, negative: false }
    }

    /// Creates a `BigInt` from an `i64`.
    pub fn from_i64(n: i64) -> Self {
        if n == 0 {
            return Self::zero();
        }
        let negative = n < 0;
        let mut digits = Vec::new();
        let mut n = n.unsigned_abs();
        while n > 0 {
            digits.push(n % BASE);
            n /= BASE;
        }
        BigInt { digits, negative }
    }

    /// Parses a `BigInt` from a string in the given radix (2–16 supported, but 10 is primary).
    pub fn from_str_radix(s: &str, radix: u32) -> Self {
        let s = s.trim();
        let (negative, s) = if let Some(stripped) = s.strip_prefix('-') {
            (true, stripped)
        } else {
            (false, s)
        };
        let s = s.trim_start_matches('0');
        if s.is_empty() {
            return Self::zero();
        }

        // For radix 10, parse directly into base-10^9 digits.
        if radix == 10 {
            // Parse as chunks of up to 9 digits from right
            let mut digits = Vec::new();
            let bytes = s.as_bytes();
            let mut i = bytes.len();
            while i > 0 {
                let start = i.saturating_sub(9);
                let chunk = &s[start..i];
                digits.push(chunk.parse::<u64>().unwrap());
                i = start;
            }
            let mut result = BigInt { digits, negative };
            result.normalize();
            return result;
        }

        // General radix: build via repeated multiply-and-add
        let radix = radix as u64;
        let mut result = Self::zero();
        for ch in s.chars() {
            let d = ch.to_digit(radix as u32).unwrap_or(0) as u64;
            result = result.mul_u64(radix);
            result = result.add_u64(d);
        }
        result.negative = negative;
        result
    }

    /// Returns `true` if this number is zero.
    pub fn is_zero(&self) -> bool {
        self.digits.is_empty()
    }

    /// Returns `true` if this number is negative.
    pub fn is_negative(&self) -> bool {
        self.negative && !self.is_zero()
    }

    /// Returns the absolute value.
    pub fn abs(&self) -> Self {
        let mut r = self.clone();
        r.negative = false;
        r
    }

    /// Removes leading zero digits.
    pub(crate) fn normalize(&mut self) {
        while self.digits.last() == Some(&0) {
            self.digits.pop();
        }
        if self.is_zero() {
            self.negative = false;
        }
    }

    /// Adds a `u64` in place, returning a new `BigInt`.
    fn add_u64(&self, n: u64) -> Self {
        if n == 0 {
            return self.clone();
        }
        let mut result = self.clone();
        let mut carry = n;
        for i in 0..result.digits.len() {
            let sum = result.digits[i] + carry;
            result.digits[i] = sum % BASE;
            carry = sum / BASE;
            if carry == 0 {
                break;
            }
        }
        if carry > 0 {
            result.digits.push(carry);
        }
        result
    }

    /// Multiplies by a `u64`, returning a new `BigInt`.
    fn mul_u64(&self, n: u64) -> Self {
        if n == 0 || self.is_zero() {
            return Self::zero();
        }
        let mut result = BigInt {
            digits: vec![],
            negative: self.negative,
        };
        let mut carry: u64 = 0;
        for &d in &self.digits {
            let prod = d * n + carry;
            result.digits.push(prod % BASE);
            carry = prod / BASE;
        }
        while carry > 0 {
            result.digits.push(carry % BASE);
            carry /= BASE;
        }
        result
    }

    /// Schoolbook (long) multiplication. Used as a baseline for Karatsuba testing.
    pub fn schoolbook_mul(&self, other: &BigInt) -> BigInt {
        if self.is_zero() || other.is_zero() {
            return Self::zero();
        }
        let mut result = vec![0u64; self.digits.len() + other.digits.len()];
        for (i, &a) in self.digits.iter().enumerate() {
            let mut carry: u64 = 0;
            for (j, &b) in other.digits.iter().enumerate() {
                let prod = result[i + j] + a * b + carry;
                result[i + j] = prod % BASE;
                carry = prod / BASE;
            }
            let mut k = i + other.digits.len();
            while carry > 0 {
                let sum = result[k] + carry;
                result[k] = sum % BASE;
                carry = sum / BASE;
                k += 1;
            }
        }
        let mut r = BigInt {
            digits: result,
            negative: self.negative != other.negative,
        };
        r.normalize();
        r
    }

    /// Divides `self` by `other`, returning `(quotient, remainder)`.
    ///
    /// # Panics
    /// Panics if `other` is zero.
    pub fn div_rem(&self, other: &BigInt) -> (BigInt, BigInt) {
        assert!(!other.is_zero(), "division by zero");
        if self.is_zero() {
            return (Self::zero(), Self::zero());
        }
        // Work with absolute values
        let a = self.abs();
        let b = other.abs();
        if a < b {
            // quotient = 0, remainder = self (keeping self's sign)
            return (Self::zero(), self.clone());
        }

        let mut quotient = vec![0u64; a.digits.len()];
        let mut remainder = Self::zero();

        // Process from most significant digit
        for i in (0..a.digits.len()).rev() {
            // remainder = remainder * BASE + a.digits[i]
            remainder = remainder.mul_u64(BASE);
            remainder = remainder.add_u64(a.digits[i]);

            // Binary search for the largest q such that q * b <= remainder
            let mut lo: u64 = 0;
            let mut hi: u64 = BASE - 1;
            let mut q: u64 = 0;
            while lo <= hi {
                let mid = lo + (hi - lo) / 2;
                let product = b.mul_u64(mid);
                if product <= remainder {
                    q = mid;
                    lo = mid + 1;
                } else {
                    hi = mid - 1;
                }
            }
            quotient[i] = q;
            remainder = remainder - b.mul_u64(q);
        }

        let q_negative = self.negative != other.negative;
        let mut q_result = BigInt {
            digits: quotient,
            negative: q_negative && !remainder.is_zero(),
        };
        q_result.normalize();

        // Remainder sign follows dividend
        remainder.negative = if remainder.is_zero() { false } else { self.negative };

        (q_result, remainder)
    }

    /// Compares the absolute values.
    pub(crate) fn abs_cmp(&self, other: &Self) -> Ordering {
        if self.digits.len() != other.digits.len() {
            return self.digits.len().cmp(&other.digits.len());
        }
        for i in (0..self.digits.len()).rev() {
            if self.digits[i] != other.digits[i] {
                return self.digits[i].cmp(&other.digits[i]);
            }
        }
        Ordering::Equal
    }

    /// Subtracts `other` from `self` assuming `self >= other` (both positive).
    pub(crate) fn abs_sub(&self, other: &Self) -> Self {
        let mut result = Vec::with_capacity(self.digits.len());
        let mut borrow: i64 = 0;
        for i in 0..self.digits.len() {
            let a = self.digits[i] as i64;
            let b = if i < other.digits.len() { other.digits[i] as i64 } else { 0 };
            let mut diff = a - b - borrow;
            if diff < 0 {
                diff += BASE as i64;
                borrow = 1;
            } else {
                borrow = 0;
            }
            result.push(diff as u64);
        }
        let mut r = BigInt { digits: result, negative: false };
        r.normalize();
        r
    }
}

impl From<u64> for BigInt {
    fn from(n: u64) -> Self {
        Self::from_u64(n)
    }
}

impl From<i64> for BigInt {
    fn from(n: i64) -> Self {
        Self::from_i64(n)
    }
}

impl From<i32> for BigInt {
    fn from(n: i32) -> Self {
        Self::from_i64(n as i64)
    }
}

impl From<u32> for BigInt {
    fn from(n: u32) -> Self {
        Self::from_u64(n as u64)
    }
}

impl PartialEq for BigInt {
    fn eq(&self, other: &Self) -> bool {
        self.negative == other.negative && self.digits == other.digits
    }
}
impl Eq for BigInt {}

impl PartialOrd for BigInt {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for BigInt {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self.is_zero(), other.is_zero()) {
            (true, true) => return Ordering::Equal,
            (true, false) => return if other.negative { Ordering::Greater } else { Ordering::Less },
            (false, true) => return if self.negative { Ordering::Less } else { Ordering::Greater },
            _ => {}
        }
        match (self.negative, other.negative) {
            (true, false) => Ordering::Less,
            (false, true) => Ordering::Greater,
            (true, true) => other.abs_cmp(self),
            (false, false) => self.abs_cmp(other),
        }
    }
}

impl fmt::Display for BigInt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_zero() {
            return write!(f, "0");
        }
        if self.negative {
            write!(f, "-")?;
        }
        if let Some(&last) = self.digits.last() {
            write!(f, "{last}")?;
            for d in self.digits.iter().rev().skip(1) {
                write!(f, "{d:09}")?;
            }
        }
        Ok(())
    }
}

// Re-export arithmetic ops using the arithmetic module functions
use std::ops::{Add, Sub, Mul, Neg};

impl Add for &BigInt {
    type Output = BigInt;
    fn add(self, other: &BigInt) -> BigInt {
        crate::arithmetic::add(self, other)
    }
}

impl Add for BigInt {
    type Output = BigInt;
    fn add(self, other: BigInt) -> BigInt {
        &self + &other
    }
}

impl Sub for &BigInt {
    type Output = BigInt;
    fn sub(self, other: &BigInt) -> BigInt {
        crate::arithmetic::sub(self, other)
    }
}

impl Sub for BigInt {
    type Output = BigInt;
    fn sub(self, other: BigInt) -> BigInt {
        &self - &other
    }
}

impl Mul for &BigInt {
    type Output = BigInt;
    fn mul(self, other: &BigInt) -> BigInt {
        crate::arithmetic::mul(self, other)
    }
}

impl Mul for BigInt {
    type Output = BigInt;
    fn mul(self, other: BigInt) -> BigInt {
        &self * &other
    }
}

impl Neg for BigInt {
    type Output = BigInt;
    fn neg(self) -> BigInt {
        let mut r = self;
        if !r.is_zero() {
            r.negative = !r.negative;
        }
        r
    }
}

impl Neg for &BigInt {
    type Output = BigInt;
    fn neg(self) -> BigInt {
        let mut r = self.clone();
        if !r.is_zero() {
            r.negative = !r.negative;
        }
        r
    }
}
