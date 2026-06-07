# bigint-rs

Arbitrary precision integer arithmetic library in pure Rust.

## Features
- Addition, subtraction, multiplication, division with remainder
- Karatsuba multiplication for large operands
- Modular arithmetic (add, multiply, exponentiate, inverse)
- Miller-Rabin primality testing
- GCD via Euclidean algorithm
- Zero external dependencies

## Usage
```rust
use bigint_rs::BigInt;

let a = BigInt::from(123456789);
let b = BigInt::from(987654321);
let sum = &a + &b;

// Modular exponentiation
use bigint_rs::modular::mod_exp;
let result = mod_exp(&BigInt::from(2), &BigInt::from(10), &BigInt::from(1000));
assert_eq!(result, BigInt::from(24));

// Primality testing
use bigint_rs::primality::is_probable_prime;
assert!(is_probable_prime(&BigInt::from(104729), 20));
```

License: MIT OR Apache-2.0
