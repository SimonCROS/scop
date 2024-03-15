use crate::traits::Lerp;
use std::ops::{Add, Mul, Sub};

impl<T> Lerp<T> for T
where
    T: Add<Output = Self> + Sub<Output = Self> + Mul<f32, Output = Self> + Clone,
{
    type Output = Self;

    /// Complexity: `O(n)`
    fn lerp(&self, other: Self, t: f32) -> Self::Output {
        self.clone().add(other.sub(self.clone()).mul(t))
    }
}
