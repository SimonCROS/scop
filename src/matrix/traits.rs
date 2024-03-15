use std::ops::{Add, AddAssign, Mul, MulAssign, Sub, SubAssign, Div, DivAssign};

pub trait Field = Add<Output = Self>
    + Sub<Output = Self>
    + Mul<Output = Self>
    + Div<Output = Self>
    + AddAssign
    + SubAssign
    + MulAssign
    + DivAssign
    + Copy
    + Sized
    + Zero
    + One
    + Norm
    + PartialEq<Self>;

pub trait Zero {
    fn zero() -> Self;
}

pub trait One {
    fn one() -> Self;
}

pub trait Transpose<Rhs = Self> {
    type Output;

    fn transpose(&self) -> Self::Output;
}

pub trait Dot<Rhs = Self> {
    type Output;

    fn dot(&self, rhs: Rhs) -> Self::Output;
}

pub trait Lerp<Rhs = Self> {
    type Output;

    fn lerp(&self, other: Self, t: f32) -> Self::Output;
}

pub trait Norm {
    fn norm(&self) -> f32;
}

impl Zero for i8 { fn zero() -> Self { 0 } }
impl Zero for i16 { fn zero() -> Self { 0 } }
impl Zero for i32 { fn zero() -> Self { 0 } }
impl Zero for i64 { fn zero() -> Self { 0 } }
impl Zero for f32 { fn zero() -> Self { 0. } }
impl Zero for f64 { fn zero() -> Self { 0. } }

impl One for i8 { fn one() -> Self { 1 } }
impl One for i16 { fn one() -> Self { 1 } }
impl One for i32 { fn one() -> Self { 1 } }
impl One for i64 { fn one() -> Self { 1 } }
impl One for f32 { fn one() -> Self { 1. } }
impl One for f64 { fn one() -> Self { 1. } }

impl Norm for i8 { fn norm(&self) -> f32 { self.abs() as f32 } }
impl Norm for i16 { fn norm(&self) -> f32 { self.abs() as f32 } }
impl Norm for i32 { fn norm(&self) -> f32 { self.abs() as f32 } }
impl Norm for i64 { fn norm(&self) -> f32 { self.abs() as f32 } }
impl Norm for f32 { fn norm(&self) -> f32 { self.abs() as f32 } }
impl Norm for f64 { fn norm(&self) -> f32 { self.abs() as f32 } }
