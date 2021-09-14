use core::borrow::Borrow;
use core::iter::{Product, Sum};
use ff::{Field, PrimeField};
use ff_derive_num::Num;
use serde::{Deserialize, Serialize};

// T&E Phase2 254-bit scalar
#[derive(PrimeField, Num, Serialize, Deserialize)]
#[PrimeFieldModulus = "21888242871839275222246405745257275088548364400416034343698204186575808495617"]
#[PrimeFieldGenerator = "5"]
#[PrimeFieldReprEndianness = "little"]
pub struct Scalar([u64; 4]);

// 128-bit scalar
//#[derive(PrimeField, Serialize, Deserialize)]
//#[PrimeFieldModulus = "70386805592835581672624750593"]
//#[PrimeFieldGenerator = "17"]
//#[PrimeFieldReprEndianness = "little"]
//pub struct Scalar([u64; 2]);

/*impl FieldHash for Scalar {
  type HashRepr = <Scalar as PrimeField>::Repr;

  fn to_hash_repr(&self) -> Self::HashRepr {
    PrimeField::to_repr(self)
  }
}*/

impl<T> Product<T> for Scalar
where
  T: Borrow<Scalar>,
{
  fn product<I>(iter: I) -> Self
  where
    I: Iterator<Item = T>,
  {
    iter.fold(Scalar::one(), |acc, item| acc * item.borrow())
  }
}

impl<T> Sum<T> for Scalar
where
  T: Borrow<Scalar>,
{
  fn sum<I>(iter: I) -> Self
  where
    I: Iterator<Item = T>,
  {
    iter.fold(Scalar::zero(), |acc, item| acc + item.borrow())
  }
}

pub trait ScalarFromPrimitives {
  fn to_scalar(self) -> Scalar;
}

impl ScalarFromPrimitives for usize {
  #[inline]
  fn to_scalar(self) -> Scalar {
    (0..self).map(|_i| Scalar::one()).sum()
  }
}

impl ScalarFromPrimitives for bool {
  #[inline]
  fn to_scalar(self) -> Scalar {
    if self {
      Scalar::one()
    } else {
      Scalar::zero()
    }
  }
}
