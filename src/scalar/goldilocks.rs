//! ff_derive does not work with Goldilock scalars, so we implement our own.

use ff::{Field, PrimeField};
use rand_core::RngCore;
use serde::{Deserialize, Serialize};
use std::ops::{Add, AddAssign, Mul, MulAssign, Neg, Sub, SubAssign};
use subtle::{Choice, ConditionallySelectable, ConstantTimeEq, CtOption};

/// 2^64 - 2^32 + 1
pub const PRIME: u64 = 0xffffffff00000001;

/// Goldilocks field with modulus 2^64 - 2^32 + 1.
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, Eq, PartialEq)]
pub struct Goldilocks(pub u64);

#[derive(Clone, Copy, Default)]
pub struct GoldilocksRepr(pub [u8; 8]);

impl AsRef<[u8]> for GoldilocksRepr {
  fn as_ref(&self) -> &[u8] {
    &self.0
  }
}

impl AsMut<[u8]> for GoldilocksRepr {
  fn as_mut(&mut self) -> &mut [u8] {
    &mut self.0
  }
}

impl Add for Goldilocks {
  type Output = Self;

  fn add(self, other: Self) -> Self {
    let sum = self.0.wrapping_add(other.0);
    Self(if sum < self.0 || sum < other.0 || sum >= PRIME {
      sum.wrapping_sub(PRIME)
    } else {
      sum
    })
  }
}

impl<'a> Add<&'a Goldilocks> for Goldilocks {
  type Output = Self;

  fn add(self, other: &'a Goldilocks) -> Self {
    self + *other
  }
}

impl AddAssign for Goldilocks {
  fn add_assign(&mut self, other: Self) {
    *self = *self + other;
  }
}

impl<'a> AddAssign<&'a Goldilocks> for Goldilocks {
  fn add_assign(&mut self, other: &'a Goldilocks) {
    *self = *self + *other;
  }
}

impl Sub for Goldilocks {
  type Output = Self;

  fn sub(self, other: Self) -> Self {
    let diff = self.0.wrapping_sub(other.0);
    Self(if diff > self.0 {
      diff.wrapping_add(PRIME)
    } else {
      diff
    })
  }
}

impl<'a> Sub<&'a Goldilocks> for Goldilocks {
  type Output = Self;

  fn sub(self, other: &'a Goldilocks) -> Self {
    self - *other
  }
}

impl SubAssign for Goldilocks {
  fn sub_assign(&mut self, other: Self) {
    *self = *self - other;
  }
}

impl<'a> SubAssign<&'a Goldilocks> for Goldilocks {
  fn sub_assign(&mut self, other: &'a Goldilocks) {
    *self = *self - *other;
  }
}

impl Mul for Goldilocks {
  type Output = Self;

  fn mul(self, other: Self) -> Self {
    let a0 = self.0 as u32;
    let a1 = (self.0 >> 32) as u32;
    let b0 = other.0 as u32;
    let b1 = (other.0 >> 32) as u32;

    let p0 = (a0 as u64).wrapping_mul(b0 as u64);
    let p1 = (a0 as u64).wrapping_mul(b1 as u64);
    let p2 = (a1 as u64).wrapping_mul(b0 as u64);
    let p3 = (a1 as u64).wrapping_mul(b1 as u64);

    let cy = (((p0 >> 32)
      .wrapping_add(p1 as u32 as u64)
      .wrapping_add(p2 as u32 as u64))
      >> 32) as u32;
    let x = p0.wrapping_add(p1 << 32).wrapping_add(p2 << 32);
    let y = p3
      .wrapping_add(p1 >> 32)
      .wrapping_add(p2 >> 32)
      .wrapping_add(cy as u64);

    let c0 = x as u32;
    let c1 = (x >> 32) as u32;
    let c2 = y as u32;
    let c3 = (y >> 32) as u32;

    (Self(c0 as u64) - Self(c2 as u64) - Self(c3 as u64))
      + (Self((c1 as u64) << 32) + Self((c2 as u64) << 32))
  }
}

impl<'a> Mul<&'a Goldilocks> for Goldilocks {
  type Output = Self;

  fn mul(self, other: &'a Goldilocks) -> Self {
    self * *other
  }
}

impl MulAssign for Goldilocks {
  fn mul_assign(&mut self, other: Self) {
    *self = *self * other;
  }
}

impl<'a> MulAssign<&'a Goldilocks> for Goldilocks {
  fn mul_assign(&mut self, other: &'a Goldilocks) {
    *self = *self * *other;
  }
}

impl Neg for Goldilocks {
  type Output = Self;

  fn neg(self) -> Self {
    Self(PRIME - self.0)
  }
}

impl ConditionallySelectable for Goldilocks {
  fn conditional_select(a: &Self, b: &Self, choice: Choice) -> Self {
    Self(u64::conditional_select(&a.0, &b.0, choice))
  }
}

impl ConstantTimeEq for Goldilocks {
  fn ct_eq(&self, other: &Self) -> Choice {
    self.0.ct_eq(&other.0)
  }
}

impl Field for Goldilocks {
  fn random(mut rng: impl RngCore) -> Self {
    Self(loop {
      let x = rng.next_u64();
      if x < PRIME {
        break x;
      }
    })
  }

  fn zero() -> Self {
    Self(0)
  }

  fn one() -> Self {
    Self(1)
  }

  fn square(&self) -> Self {
    *self * *self
  }

  fn double(&self) -> Self {
    *self + *self
  }

  fn invert(&self) -> CtOption<Self> {
    if self.0 == 0 {
      CtOption::new(*self, Choice::from(0))
    } else {
      let p = self.pow_vartime(&[PRIME - 2]);
      CtOption::new(p, Choice::from(1))
    }
  }

  fn sqrt(&self) -> CtOption<Self> {
    unimplemented!()
  }
}

impl From<u64> for Goldilocks {
  fn from(value: u64) -> Self {
    Self(value)
  }
}

impl PrimeField for Goldilocks {
  type Repr = GoldilocksRepr;

  const NUM_BITS: u32 = 64;
  const CAPACITY: u32 = 63;
  const S: u32 = 32;

  fn from_repr(repr: Self::Repr) -> CtOption<Self> {
    let value = u64::from_le_bytes(repr.0);
    if value < PRIME {
      CtOption::new(Self(value), Choice::from(1))
    } else {
      CtOption::new(Self(value), Choice::from(0))
    }
  }

  fn to_repr(&self) -> Self::Repr {
    GoldilocksRepr(self.0.to_le_bytes())
  }

  fn is_odd(&self) -> Choice {
    Choice::from((self.0 & 1) as u8)
  }

  fn multiplicative_generator() -> Self {
    Self(2717)
  }

  fn root_of_unity() -> Self {
    Self(3549159659317374600)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use rand_chacha::ChaCha20Rng;
  use rand_core::SeedableRng;

  #[test]
  fn test_mul() {
    let mut rng = ChaCha20Rng::from_seed([0; 32]);
    for _ in 0..1024 {
      let a = Goldilocks::random(&mut rng);
      let b = Goldilocks::random(&mut rng);
      let prod = a * b;
      let prod_naive = Goldilocks(((a.0 as u128) * (b.0 as u128) % (PRIME as u128)) as u64);
      assert_eq!(prod, prod_naive);
    }
  }

  #[test]
  fn test_invert() {
    let mut rng = ChaCha20Rng::from_seed([0; 32]);
    for _ in 0..1024 {
      let a = loop {
        let a = Goldilocks::random(&mut rng);
        if a.0 != 0 {
          break a;
        }
      };
      let a_inv = a.invert();
      assert_eq!(a * a_inv.unwrap(), Goldilocks::one());
    }
  }

  #[test]
  fn test_root_unity() {
    let root_of_unity = Goldilocks::root_of_unity();
    let should_be_one = root_of_unity.pow_vartime([1u64 << Goldilocks::S]);
    let one = Goldilocks::one();
    assert_eq!(should_be_one, one);
  }
}
