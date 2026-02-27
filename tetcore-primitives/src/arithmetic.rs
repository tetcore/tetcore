// File: arithmetic.rs - This file is part of Tetcore
// Copyright (c) 2026 Dust LLC, and Contributors
// Description:
// Arithmetic primitives for deterministic computation in Tetcore.
// Includes FixedU128 for 18-decimal token amounts, Perbill/PerU16 for
// fractional representation, Gas for resource metering, and safe math
// traits for checked/saturating operations. All operations ensure
// consensus-critical determinism.

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct FixedU128 {
    raw: u128,
    decimals: u8,
}

impl FixedU128 {
    pub const DECIMALS: u8 = 18;
    pub const ONE: Self = Self {
        raw: 10u128.pow(18),
        decimals: 18,
    };
    pub const ZERO: Self = Self {
        raw: 0,
        decimals: 18,
    };

    pub fn new(raw: u128) -> Self {
        Self {
            raw,
            decimals: Self::DECIMALS,
        }
    }

    pub fn from_parts(integer: u128, fraction: u128) -> Self {
        let raw = integer * 10u128.pow(18) + fraction % 10u128.pow(18);
        Self {
            raw,
            decimals: Self::DECIMALS,
        }
    }

    pub fn from_token_amount(amount: u128) -> Self {
        Self {
            raw: amount,
            decimals: Self::DECIMALS,
        }
    }

    pub fn to_token_amount(&self) -> u128 {
        self.raw
    }

    pub fn as_u128(&self) -> u128 {
        self.raw
    }

    pub fn integer_part(&self) -> u128 {
        self.raw / 10u128.pow(self.decimals as u32)
    }

    pub fn fractional_part(&self) -> u128 {
        self.raw % 10u128.pow(self.decimals as u32)
    }

    pub fn saturating_add(self, other: Self) -> Self {
        Self {
            raw: self.raw.saturating_add(other.raw),
            decimals: self.decimals,
        }
    }

    pub fn saturating_sub(self, other: Self) -> Self {
        Self {
            raw: self.raw.saturating_sub(other.raw),
            decimals: self.decimals,
        }
    }

    pub fn checked_add(self, other: Self) -> Option<Self> {
        Some(Self {
            raw: self.raw.checked_add(other.raw)?,
            decimals: self.decimals,
        })
    }

    pub fn checked_sub(self, other: Self) -> Option<Self> {
        Some(Self {
            raw: self.raw.checked_sub(other.raw)?,
            decimals: self.decimals,
        })
    }

    pub fn mul(self, other: Self) -> Self {
        let result = (self.raw * other.raw) / 10u128.pow(self.decimals as u32);
        Self {
            raw: result,
            decimals: self.decimals,
        }
    }

    pub fn mul_truncate(self, other: Self) -> Self {
        let result = (self.raw * other.raw) / 10u128.pow(self.decimals as u32);
        Self {
            raw: result,
            decimals: self.decimals,
        }
    }

    pub fn div(self, other: Self) -> Option<Self> {
        if other.raw == 0 {
            return None;
        }
        let scaled = self.raw * 10u128.pow(self.decimals as u32);
        let result = scaled / other.raw;
        Some(Self {
            raw: result,
            decimals: self.decimals,
        })
    }

    pub fn ratio(numerator: u128, denominator: u128) -> Option<Self> {
        if denominator == 0 {
            return None;
        }
        let raw = (numerator * 10u128.pow(Self::DECIMALS as u32)) / denominator;
        Some(Self {
            raw,
            decimals: Self::DECIMALS,
        })
    }

    pub fn percentage(basis_points: u32) -> Self {
        Self {
            raw: basis_points as u128 * 10u128.pow((Self::DECIMALS - 2) as u32),
            decimals: Self::DECIMALS,
        }
    }

    pub fn from_basis_points(bp: u16) -> Self {
        Self {
            raw: bp as u128 * 10u128.pow((Self::DECIMALS - 4) as u32),
            decimals: Self::DECIMALS,
        }
    }

    pub fn to_basis_points(&self) -> u16 {
        (self.raw / 10u128.pow((self.decimals - 4) as u32)) as u16
    }
}

impl Default for FixedU128 {
    fn default() -> Self {
        Self::ZERO
    }
}

impl core::ops::Add for FixedU128 {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        self.saturating_add(other)
    }
}

impl core::ops::Sub for FixedU128 {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        self.saturating_sub(other)
    }
}

impl core::ops::Mul for FixedU128 {
    type Output = Self;
    fn mul(self, other: Self) -> Self {
        self.mul(other)
    }
}

impl core::ops::Div for FixedU128 {
    type Output = Option<Self>;
    fn div(self, other: Self) -> Option<Self> {
        self.div(other)
    }
}

#[derive(Clone, Debug, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CheckedAdd;

pub trait SafeAdd: Sized {
    fn checked_add(self, other: Self) -> Option<Self>;
    fn saturating_add(self, other: Self) -> Self;
    fn checked_add_signed(self, other: i128) -> Option<Self>;
}

pub trait SafeSub: Sized {
    fn checked_sub(self, other: Self) -> Option<Self>;
    fn saturating_sub(self, other: Self) -> Self;
}

pub trait SafeMul: Sized {
    fn checked_mul(self, other: Self) -> Option<Self>;
    fn saturating_mul(self, other: Self) -> Self;
}

pub trait SafeDiv: Sized {
    fn checked_div(self, other: Self) -> Option<Self>;
    fn saturating_div(self, other: Self) -> Self;
}

impl SafeAdd for u8 {
    fn checked_add(self, other: Self) -> Option<Self> {
        self.checked_add(other)
    }
    fn saturating_add(self, other: Self) -> Self {
        self.saturating_add(other)
    }
    fn checked_add_signed(self, other: i128) -> Option<Self> {
        if other >= 0 {
            self.checked_add(other as u8)
        } else {
            let abs = (-other) as u8;
            self.checked_sub(abs)
        }
    }
}

impl SafeSub for u8 {
    fn checked_sub(self, other: Self) -> Option<Self> {
        self.checked_sub(other)
    }
    fn saturating_sub(self, other: Self) -> Self {
        self.saturating_sub(other)
    }
}

impl SafeMul for u8 {
    fn checked_mul(self, other: Self) -> Option<Self> {
        self.checked_mul(other)
    }
    fn saturating_mul(self, other: Self) -> Self {
        self.saturating_mul(other)
    }
}

impl SafeDiv for u8 {
    fn checked_div(self, other: Self) -> Option<Self> {
        if other == 0 {
            None
        } else {
            Some(self / other)
        }
    }
    fn saturating_div(self, other: Self) -> Self {
        if other == 0 {
            0
        } else {
            self / other
        }
    }
}

impl SafeAdd for u16 {
    fn checked_add(self, other: Self) -> Option<Self> {
        self.checked_add(other)
    }
    fn saturating_add(self, other: Self) -> Self {
        self.saturating_add(other)
    }
    fn checked_add_signed(self, other: i128) -> Option<Self> {
        if other >= 0 {
            self.checked_add(other as u16)
        } else {
            let abs = (-other) as u16;
            self.checked_sub(abs)
        }
    }
}

impl SafeSub for u16 {
    fn checked_sub(self, other: Self) -> Option<Self> {
        self.checked_sub(other)
    }
    fn saturating_sub(self, other: Self) -> Self {
        self.saturating_sub(other)
    }
}

impl SafeMul for u16 {
    fn checked_mul(self, other: Self) -> Option<Self> {
        self.checked_mul(other)
    }
    fn saturating_mul(self, other: Self) -> Self {
        self.saturating_mul(other)
    }
}

impl SafeDiv for u16 {
    fn checked_div(self, other: Self) -> Option<Self> {
        if other == 0 {
            None
        } else {
            Some(self / other)
        }
    }
    fn saturating_div(self, other: Self) -> Self {
        if other == 0 {
            0
        } else {
            self / other
        }
    }
}

impl SafeAdd for u32 {
    fn checked_add(self, other: Self) -> Option<Self> {
        self.checked_add(other)
    }
    fn saturating_add(self, other: Self) -> Self {
        self.saturating_add(other)
    }
    fn checked_add_signed(self, other: i128) -> Option<Self> {
        if other >= 0 {
            self.checked_add(other as u32)
        } else {
            let abs = (-other) as u32;
            self.checked_sub(abs)
        }
    }
}

impl SafeSub for u32 {
    fn checked_sub(self, other: Self) -> Option<Self> {
        self.checked_sub(other)
    }
    fn saturating_sub(self, other: Self) -> Self {
        self.saturating_sub(other)
    }
}

impl SafeMul for u32 {
    fn checked_mul(self, other: Self) -> Option<Self> {
        self.checked_mul(other)
    }
    fn saturating_mul(self, other: Self) -> Self {
        self.saturating_mul(other)
    }
}

impl SafeDiv for u32 {
    fn checked_div(self, other: Self) -> Option<Self> {
        if other == 0 {
            None
        } else {
            Some(self / other)
        }
    }
    fn saturating_div(self, other: Self) -> Self {
        if other == 0 {
            0
        } else {
            self / other
        }
    }
}

impl SafeAdd for u64 {
    fn checked_add(self, other: Self) -> Option<Self> {
        self.checked_add(other)
    }
    fn saturating_add(self, other: Self) -> Self {
        self.saturating_add(other)
    }
    fn checked_add_signed(self, other: i128) -> Option<Self> {
        if other >= 0 {
            self.checked_add(other as u64)
        } else {
            let abs = (-other) as u64;
            self.checked_sub(abs)
        }
    }
}

impl SafeSub for u64 {
    fn checked_sub(self, other: Self) -> Option<Self> {
        self.checked_sub(other)
    }
    fn saturating_sub(self, other: Self) -> Self {
        self.saturating_sub(other)
    }
}

impl SafeMul for u64 {
    fn checked_mul(self, other: Self) -> Option<Self> {
        self.checked_mul(other)
    }
    fn saturating_mul(self, other: Self) -> Self {
        self.saturating_mul(other)
    }
}

impl SafeDiv for u64 {
    fn checked_div(self, other: Self) -> Option<Self> {
        if other == 0 {
            None
        } else {
            Some(self / other)
        }
    }
    fn saturating_div(self, other: Self) -> Self {
        if other == 0 {
            0
        } else {
            self / other
        }
    }
}

impl SafeAdd for u128 {
    fn checked_add(self, other: Self) -> Option<Self> {
        self.checked_add(other)
    }
    fn saturating_add(self, other: Self) -> Self {
        self.saturating_add(other)
    }
    fn checked_add_signed(self, other: i128) -> Option<Self> {
        if other >= 0 {
            self.checked_add(other as u128)
        } else {
            let abs = (-other) as u128;
            self.checked_sub(abs)
        }
    }
}

impl SafeSub for u128 {
    fn checked_sub(self, other: Self) -> Option<Self> {
        self.checked_sub(other)
    }
    fn saturating_sub(self, other: Self) -> Self {
        self.saturating_sub(other)
    }
}

impl SafeMul for u128 {
    fn checked_mul(self, other: Self) -> Option<Self> {
        self.checked_mul(other)
    }
    fn saturating_mul(self, other: Self) -> Self {
        self.saturating_mul(other)
    }
}

impl SafeDiv for u128 {
    fn checked_div(self, other: Self) -> Option<Self> {
        if other == 0 {
            None
        } else {
            Some(self / other)
        }
    }
    fn saturating_div(self, other: Self) -> Self {
        if other == 0 {
            0
        } else {
            self / other
        }
    }
}

#[derive(Clone, Debug, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Perbill(u32);

impl Perbill {
    pub const ZERO: Self = Self(0);
    pub const ONE: Self = Self(1_000_000_000);

    pub fn from_parts(parts: u32) -> Self {
        Self(parts.min(1_000_000_000))
    }

    pub fn from_percent(x: u32) -> Self {
        Self(x.saturating_mul(10_000_000))
    }

    pub fn from_basis_points(x: u16) -> Self {
        Self(x as u32 * 10_000)
    }

    pub fn from_rational(numer: u32, denom: u32) -> Option<Self> {
        if numer == 0 {
            return Some(Self::ZERO);
        }
        if denom == 0 {
            return None;
        }
        let result = (numer as u64 * 1_000_000_000u64 / denom as u64) as u32;
        Some(Self(result))
    }

    pub fn as_u32(self) -> u32 {
        self.0
    }

    pub fn as_percent(self) -> u32 {
        self.0 / 10_000_000
    }

    pub fn as_basis_points(self) -> u16 {
        (self.0 / 10_000) as u16
    }

    pub fn multiply(self, value: u128) -> u128 {
        (value * self.0 as u128) / 1_000_000_000
    }

    pub fn multiply_ceil(self, value: u128) -> u128 {
        ((value * self.0 as u128) + 1_000_000_000 - 1) / 1_000_000_000
    }
}

impl core::ops::Mul<u128> for Perbill {
    type Output = u128;
    fn mul(self, value: u128) -> u128 {
        self.multiply(value)
    }
}

#[derive(Clone, Debug, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PerU16(u16);

impl PerU16 {
    pub const ZERO: Self = Self(0);
    pub const ONE: Self = Self(65535);

    pub fn from_parts(parts: u16) -> Self {
        Self(parts.min(65535))
    }

    pub fn from_percent(x: u32) -> Self {
        Self(((x.saturating_mul(65535) / 100) as u16).min(65535))
    }

    pub fn from_basis_points(x: u16) -> Self {
        Self((x.saturating_mul(65535) / 10000).min(65535))
    }

    pub fn from_rational(numer: u16, denom: u16) -> Option<Self> {
        if numer == 0 {
            return Some(Self::ZERO);
        }
        if denom == 0 {
            return None;
        }
        let result = (numer as u32 * 65535u32 / denom as u32) as u16;
        Some(Self(result))
    }

    pub fn as_u16(self) -> u16 {
        self.0
    }

    pub fn as_percent(self) -> u32 {
        (self.0 as u32 * 100) / 65535
    }

    pub fn as_basis_points(self) -> u16 {
        (self.0 as u32 * 10000 / 65535) as u16
    }

    pub fn multiply(self, value: u128) -> u128 {
        (value * self.0 as u128) / 65535
    }
}

impl core::ops::Mul<u128> for PerU16 {
    type Output = u128;
    fn mul(self, value: u128) -> u128 {
        self.multiply(value)
    }
}

#[derive(Clone, Debug, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Ratio {
    pub numerator: u128,
    pub denominator: u128,
}

impl Ratio {
    pub fn new(numerator: u128, denominator: u128) -> Option<Self> {
        if denominator == 0 {
            None
        } else {
            Some(Self {
                numerator,
                denominator,
            })
        }
    }

    pub fn from_rational(numer: u128, denom: u128) -> Option<Self> {
        Self::new(numer, denom)
    }

    pub fn percent(x: u32) -> Self {
        Self {
            numerator: x as u128,
            denominator: 100,
        }
    }

    pub fn basis_points(x: u16) -> Self {
        Self {
            numerator: x as u128,
            denominator: 10000,
        }
    }

    pub fn multiply(&self, value: u128) -> u128 {
        (value * self.numerator) / self.denominator
    }

    pub fn multiply_ceil(&self, value: u128) -> u128 {
        ((value * self.numerator) + self.denominator - 1) / self.denominator
    }

    pub fn inverse(&self) -> Option<Self> {
        Self::new(self.denominator, self.numerator)
    }
}

impl Default for Ratio {
    fn default() -> Self {
        Self {
            numerator: 1,
            denominator: 1,
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Gas {
    gas: u64,
}

impl Gas {
    pub const ZERO: Self = Self { gas: 0 };
    pub const MAX: Self = Self { gas: u64::MAX };

    pub fn new(gas: u64) -> Self {
        Self { gas }
    }

    pub fn as_u64(&self) -> u64 {
        self.gas
    }

    pub fn checked_add(self, other: Self) -> Option<Self> {
        self.gas.checked_add(other.gas).map(|g| Self { gas: g })
    }

    pub fn checked_sub(self, other: Self) -> Option<Self> {
        self.gas.checked_sub(other.gas).map(|g| Self { gas: g })
    }

    pub fn saturating_add(self, other: Self) -> Self {
        Self {
            gas: self.gas.saturating_add(other.gas),
        }
    }

    pub fn saturating_sub(self, other: Self) -> Self {
        Self {
            gas: self.gas.saturating_sub(other.gas),
        }
    }

    pub fn checked_mul(self, rhs: u64) -> Option<Self> {
        self.gas.checked_mul(rhs).map(|g| Self { gas: g })
    }

    pub fn saturating_mul(self, rhs: u64) -> Self {
        Self {
            gas: self.gas.saturating_mul(rhs),
        }
    }
}

impl core::ops::Add for Gas {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        self.saturating_add(other)
    }
}

impl core::ops::Sub for Gas {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        self.saturating_sub(other)
    }
}

impl core::ops::Mul<u64> for Gas {
    type Output = Self;
    fn mul(self, rhs: u64) -> Self {
        self.saturating_mul(rhs)
    }
}

#[cfg(feature = "std")]
impl std::fmt::Display for FixedU128 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let integer = self.integer_part();
        let fraction = self.fractional_part();
        write!(f, "{}.{:0>18}", integer, fraction)
    }
}
