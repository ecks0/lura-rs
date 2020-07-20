use num::pow;
use num_traits::{
  int::PrimInt,
  sign::Unsigned,
  cast::{FromPrimitive, ToPrimitive},
};
use std::{
  thread,
  time,
  ops::AddAssign,
};
use async_std::task;

/////
// Expo, exponential decay

pub struct Expo<I: PrimInt + Unsigned> {
  base: I,
  factor: I,
  power: usize,
  max: Option<I>,
}

impl<I> Expo<I>
where
  I: PrimInt + Unsigned,
  u8: Into<I>,
{

  pub fn new(max: Option<I>) -> Self {
    Self { base: 2.into(), factor: 1.into(), power: 0, max }
  }
}

impl<I: PrimInt + Unsigned> Expo<I> {

  pub fn with(base: I, factor: I, power: usize, max: Option<I>) -> Self {
    Self { base, factor, power, max }
  }

  pub fn sleep(&mut self) -> bool {
    match self.next() {
      Some(value) => {
        thread::sleep(time::Duration::from_secs(value.to_u64().unwrap()));
        return true;
      },
      None => return false,
    }
  }

  pub async fn async_sleep(&mut self) -> bool {
    match self.next() {
      Some(value) => {
        task::sleep(time::Duration::from_secs(value.to_u64().unwrap())).await;
        return true;
      },
      None => return false,
    }
  }
}

impl<I: PrimInt + Unsigned + ToPrimitive> Iterator for Expo<I> {

  type Item = I;

  fn next(&mut self) -> Option<I> {
    let a: I = self.factor * pow(self.base, self.power);
    if self.max.is_none() || a < self.max.unwrap() {
      self.power += 1;
      Some(a)
    } else {
      None
    }
  }
}

/////
// Fibo, fibonaccial decay

pub struct Fibo<I: PrimInt + Unsigned> {
  a: I,
  b: I,
  max: Option<I>,
}

impl<I> Fibo<I>
where
  I: PrimInt + Unsigned,
  u8: Into<I>,
{

  pub fn new(max: Option<I>) -> Self {
    Self { a: 0.into(), b: 1.into(), max }
  }
}

impl<I: PrimInt + Unsigned> Fibo<I> {

  pub fn with(a: I, b: I, max: Option<I>) -> Self {
    Self { a, b, max }
  }

  pub fn sleep(&mut self) -> bool {
    match self.next() {
      Some(value) => {
        thread::sleep(time::Duration::from_secs(value.to_u64().unwrap()));
        return true;
      },
      None => return false,
    }
  }

  pub async fn async_sleep(&mut self) -> bool {
    match self.next() {
      Some(value) => {
        task::sleep(time::Duration::from_secs(value.to_u64().unwrap())).await;
        return true;
      },
      None => return false,
    }
  }
}

impl<I: PrimInt + Unsigned + ToPrimitive> Iterator for Fibo<I> {

  type Item = I;

  fn next(&mut self) -> Option<I> {
    if self.max.is_none() || self.a < self.max.unwrap() {
      let a = self.a;
      self.a = self.b;
      self.b = a + self.b;
      Some(a)
    } else {
      None
    }
  }
}

/////
// Constant, constant decay

pub struct Constant<I: PrimInt + Unsigned + AddAssign> {
  multiplier: I,
  multiplicand: I,
  max: Option<I>,
}

impl<I> Constant<I>
where
  I: PrimInt + Unsigned + AddAssign,
  u8: Into<I>,
{

  pub fn new(multiplicand: I, max: Option<I>) -> Self {
    Self { multiplier: 0.into(), multiplicand, max }
  }
}

impl<I> Constant<I>
where
  I: PrimInt + Unsigned + AddAssign + FromPrimitive,
  u8: Into<I>,
{

  pub fn with(multiplier: I, multiplicand: I, max: Option<I>) -> Self {
    Self { multiplier, multiplicand, max }
  }

  pub fn sleep(&mut self) -> bool {
    match self.next() {
      Some(value) => {
        thread::sleep(time::Duration::from_secs(value.to_u64().unwrap()));
        return true;
      },
      None => return false,
    }
  }

  pub async fn async_sleep(&mut self) -> bool {
    match self.next() {
      Some(value) => {
        task::sleep(time::Duration::from_secs(value.to_u64().unwrap())).await;
        return true;
      },
      None => return false,
    }
  }
}

impl<I> Iterator for Constant<I>
where
  I: PrimInt + Unsigned + FromPrimitive + AddAssign,
  u8: Into<I>,
{
  type Item = I;

  fn next(&mut self) -> Option<I> {
    let value = self.multiplier * self.multiplicand;
    if self.max.is_none() || value < self.max.unwrap() {
      self.multiplier += 1.into();
      Some(value)
    } else {
      None
    }
  }
}

#[cfg(test1)]
mod tests {

  use anyhow::Error;
  use std::{thread, time};
  use crate::backoff::{Expo, Fibo, Constant};

  ///// expo

  #[test]
  fn test_expo_new() -> Result<(), Error> {
    let expo: Expo<u16> = Expo::new(Some(10000));
    expo.for_each(|i| println!("{:?}", i));
    Ok(())
  }

  #[test]
  fn test_expo_with() -> Result<(), Error> {
    let expo: Expo<u32> = Expo::with(2, 1, 0, Some(10000));
    expo.for_each(|i| println!("{:?}", i));
    Ok(())
  }

  #[test]
  fn test_expo_sleep() -> Result<(), Error> {
    let expo: Expo<u64> = Expo::new(Some(9));
    expo.for_each(|i| {
      let now = time::Instant::now();
      println!("{:?} sleep start {:?}", i, now);
      thread::sleep(time::Duration::from_secs(i));
      println!("{:?} sleep end {:?}", i, time::Instant::now());
      println!("{:?} slept for {:?}\n", i, now.elapsed());
    });
    Ok(())
  }

  ///// fibo

  #[test]
  fn test_fibo_new() -> Result<(), Error> {
    let fibo: Fibo<u16> = Fibo::new(Some(500));
    fibo.for_each(|i| println!("{:?}", i));
    Ok(())
  }

  #[test]
  fn test_fibo_with() -> Result<(), Error> {
    let fibo: Fibo<u32> = Fibo::with(0, 1, Some(500));
    fibo.for_each(|i| println!("{:?}", i));
    Ok(())
  }

  #[test]
  fn test_fibo_sleep() -> Result<(), Error> {
    let fibo: Fibo<u64> = Fibo::new(Some(9));
    fibo.for_each(|i| {
      let now = time::Instant::now();
      println!("{:?} sleep start {:?}", i, now);
      thread::sleep(time::Duration::from_secs(i));
      println!("{:?} sleep end {:?}", i, time::Instant::now());
      println!("{:?} slept for {:?}\n", i, now.elapsed());
    });
    Ok(())
  }

  ///// constant

  #[test]
  fn test_constant_new() -> Result<(), Error> {
    let constant: Constant<u16> = Constant::new(1, Some(10));
    constant.for_each(|i| println!("{:?}", i));
    Ok(())
  }

  #[test]
  fn test_constant_with() -> Result<(), Error> {
    let constant: Constant<u32> = Constant::with(0, 2, Some(20));
    constant.for_each(|i| println!("{:?}", i));
    Ok(())
  }

  #[test]
  fn test_constant_sleep() -> Result<(), Error> {
    let constant: Constant<u64> = Constant::new(1, Some(4));
    constant.for_each(|i| {
      let now = time::Instant::now();
      println!("{:?} sleep start {:?}", i, now);
      thread::sleep(time::Duration::from_secs(i));
      println!("{:?} sleep end {:?}", i, time::Instant::now());
      println!("{:?} slept for {:?}\n", i, now.elapsed());
    });
    Ok(())
  }
}
