use crate::traits::{Dot, Field, One, Zero};
use std::fmt::{self, Debug, Display, Formatter};
use std::ops::{Add, AddAssign, Div, DivAssign, Index, IndexMut, Mul, MulAssign, Sub, SubAssign};

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Vector<const SIZE: usize, K>(pub(super) [K; SIZE]);

impl<const SIZE: usize, K> Vector<SIZE, K>
where
    K: Field,
{
    /// Complexity `O(1)`
    pub fn size(&self) -> usize {
        SIZE
    }

    /// Complexity `O(n)`
    pub fn linear_combination<const LEN: usize>(
        u: [Vector<SIZE, K>; LEN],
        coefs: [K; LEN],
    ) -> Vector<SIZE, K> {
        let iter = u.into_iter().zip(coefs);
        iter.fold(Self::default(), |acc, x| acc + x.0.mul(x.1))
    }

    pub fn iter(&self) -> core::slice::Iter<K> {
        self.0.iter()
    }

    pub fn norm_1(&self) -> f32 {
        self.0.iter().fold(0.0, |acc, f| acc + (*f).norm())
    }

    pub fn norm(&self) -> f32 {
        self.0
            .iter()
            .fold(0.0, |acc, f| acc + (*f * *f).norm())
            .sqrt()
    }

    pub fn norm_inf(&self) -> f32 {
        self.0
            .iter()
            .map(|f| (*f).norm())
            .reduce(f32::max)
            .unwrap_or_default()
    }
}

impl<const SIZE: usize, K> Zero for Vector<SIZE, K>
where
    K: Field,
{
    fn zero() -> Self {
        Self([K::zero(); SIZE])
    }
}

impl<const SIZE: usize, K> One for Vector<SIZE, K>
where
    K: Field,
{
    fn one() -> Self {
        Self([K::one(); SIZE])
    }
}

impl<K> Vector<3, K>
where
    K: Field,
{
    pub fn cross(&self, v: &Self) -> Self {
        Self([
            (self.0[1] * v.0[2]) - (self.0[2] * v.0[1]),
            (self.0[2] * v.0[0]) - (self.0[0] * v.0[2]),
            (self.0[0] * v.0[1]) - (self.0[1] * v.0[0]),
        ])
    }
}

impl<const SIZE: usize, K> Vector<SIZE, K>
where
    K: Field + Div<f32, Output = f32>,
{
    pub fn angle_cos(&self, v: &Self) -> f32 {
        self.dot(v) / (self.norm() * v.norm())
    }
}

impl<const SIZE: usize, K> From<[K; SIZE]> for Vector<SIZE, K>
where
    K: Field,
{
    /// Complexity `O(1)`
    fn from(content: [K; SIZE]) -> Self {
        Self(content)
    }
}

impl<K> From<[K; 3]> for Vector<4, K>
where
    K: Field,
{
    /// Complexity `O(1)`
    fn from(content: [K; 3]) -> Self {
        Self([content[0], content[1], content[2], K::one()])
    }
}

impl<K> From<[K; 2]> for Vector<3, K>
where
    K: Field,
{
    /// Complexity `O(1)`
    fn from(content: [K; 2]) -> Self {
        Self([content[0], content[1], K::zero()])
    }
}

impl<const SIZE: usize, K> Default for Vector<SIZE, K>
where
    K: Field,
{
    /// Complexity `O(n)`
    fn default() -> Self {
        Self([(); SIZE].map(|_| K::zero()))
    }
}

impl<const SIZE: usize, K> Display for Vector<SIZE, K>
where
    K: Field + Display,
{
    /// Complexity `O(n)`
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "[{}]", self.0.map(|f| f.to_string()).join(", "))?;
        Ok(())
    }
}

impl<const SIZE: usize, K> Add<Vector<SIZE, K>> for Vector<SIZE, K>
where
    K: Field,
{
    type Output = Vector<SIZE, K>;

    /// Complexity `O(n)`
    fn add(self, other: Vector<SIZE, K>) -> Self::Output {
        let mut result = self;
        result += other;
        result
    }
}

impl<const SIZE: usize, K> Sub<Vector<SIZE, K>> for Vector<SIZE, K>
where
    K: Field,
{
    type Output = Vector<SIZE, K>;

    /// Complexity `O(n)`
    fn sub(self, other: Vector<SIZE, K>) -> Self::Output {
        let mut result = self;
        result -= other;
        result
    }
}

impl<const SIZE: usize, K, S> Mul<S> for Vector<SIZE, K>
where
    K: Field + MulAssign<S>,
    S: Field,
{
    type Output = Vector<SIZE, K>;

    fn mul(self, other: S) -> Self::Output {
        let mut result = self;
        result *= other;
        result
    }
}

impl<const SIZE: usize, K, S> Div<S> for Vector<SIZE, K>
where
    K: Field + DivAssign<S>,
    S: Field,
{
    type Output = Vector<SIZE, K>;

    fn div(self, other: S) -> Self::Output {
        let mut result = self;
        result /= other;
        result
    }
}

impl<const SIZE: usize, K> Dot<&Vector<SIZE, K>> for Vector<SIZE, K>
where
    K: Field,
{
    type Output = K;

    /// Complexity: `O(n)`
    fn dot(&self, other: &Vector<SIZE, K>) -> Self::Output {
        self.0
            .into_iter()
            .zip(other.0)
            .fold(K::zero(), |acc, (v1, v2)| acc + (v1 * v2))
    }
}

impl<const SIZE: usize, K> AddAssign<Vector<SIZE, K>> for Vector<SIZE, K>
where
    K: Field,
{
    /// Complexity: `O(n)`
    fn add_assign(&mut self, other: Vector<SIZE, K>) {
        for cell in self.0.iter_mut().zip(other.0.into_iter()) {
            *cell.0 += cell.1;
        }
    }
}

impl<const SIZE: usize, K> SubAssign<Vector<SIZE, K>> for Vector<SIZE, K>
where
    K: Field,
{
    /// Complexity: `O(n)`
    fn sub_assign(&mut self, other: Vector<SIZE, K>) {
        for cell in self.0.iter_mut().zip(other.0.into_iter()) {
            *cell.0 -= cell.1;
        }
    }
}

impl<const SIZE: usize, K, S> MulAssign<S> for Vector<SIZE, K>
where
    K: Field + MulAssign<S>,
    S: Field,
{
    /// Complexity: `O(n)`
    fn mul_assign(&mut self, other: S) {
        for cell in &mut self.0 {
            *cell *= other;
        }
    }
}

impl<const SIZE: usize, K> MulAssign<Vector<SIZE, K>> for Vector<SIZE, K>
where
    K: Field,
{
    /// Complexity: `O(n)`
    fn mul_assign(&mut self, other: Self) {
        for (i, o) in self.0.iter_mut().zip(other.0) {
            *i *= o;
        }
    }
}

impl<const SIZE: usize, K, S> DivAssign<S> for Vector<SIZE, K>
where
    K: Field + DivAssign<S>,
    S: Field,
{
    /// Complexity: `O(n)`
    fn div_assign(&mut self, other: S) {
        for cell in &mut self.0 {
            *cell /= other;
        }
    }
}

impl<const SIZE: usize, K> Index<usize> for Vector<SIZE, K>
where
    K: Field,
{
    type Output = K;
    fn index<'a>(&'a self, i: usize) -> &'a K {
        &self.0[i]
    }
}

impl<const SIZE: usize, K> IndexMut<usize> for Vector<SIZE, K>
where
    K: Field,
{
    fn index_mut<'a>(&'a mut self, i: usize) -> &'a mut K {
        &mut self.0[i]
    }
}

impl Vector<2, f32> {
    pub fn x(&self) -> f32 {
        self.0[0]
    }

    pub fn y(&self) -> f32 {
        self.0[1]
    }
}

impl Vector<3, f32> {
    pub fn x(&self) -> f32 {
        self.0[0]
    }

    pub fn y(&self) -> f32 {
        self.0[1]
    }

    pub fn z(&self) -> f32 {
        self.0[2]
    }

    pub fn from_angle_z(angle: f32) -> Self {
        Self([angle.cos(), angle.sin(), 0.0])
    }

    pub fn from_angle_y(angle: f32) -> Self {
        Self([angle.cos(), 0.0, angle.sin()])
    }

    pub fn from_angle_x(angle: f32) -> Self {
        Self([0.0, angle.cos(), angle.sin()])
    }

    pub fn from_angle_axis(angle: f32, axis: &Self) -> Self {
        let c = angle.cos();
        let s = angle.sin();
        let t = 1.0 - c;
        let x = axis.0[0];
        let y = axis.0[1];
        let z = axis.0[2];
        Self([(t * x * x + c), (t * x * y - s * z), (t * x * z + s * y)])
    }

    pub fn normalize(&self) -> Self {
        let norm = self.norm();
        Self([self.0[0] / norm, self.0[1] / norm, self.0[2] / norm])
    }
}

impl Vector<4, f32> {
    pub fn x(&self) -> f32 {
        self.0[0]
    }

    pub fn y(&self) -> f32 {
        self.0[1]
    }

    pub fn z(&self) -> f32 {
        self.0[2]
    }

    pub fn w(&self) -> f32 {
        self.0[3]
    }
}
