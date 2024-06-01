use super::vector::Vector;
use crate::traits::{Dot, Field, Transpose};
use std::fmt::{self, Debug, Display, Formatter};
use std::ops::{Add, AddAssign, Mul, MulAssign, Sub, SubAssign, Index, IndexMut};

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Matrix<const ROWS: usize, const COLS: usize, K>([Vector<COLS, K>; ROWS])
where
    K: Field;

impl<const ROWS: usize, const COLS: usize, K> Matrix<ROWS, COLS, K>
where
    K: Field,
{
    /// Returns the size of the matrix in a tuple
    /// (rows: usize, cols: usize)
    pub const fn size(&self) -> (usize, usize) {
        (ROWS, COLS)
    }

    pub const fn is_square(&self) -> bool {
        ROWS == COLS
    }

    pub fn iter(&self) -> core::slice::Iter<Vector<COLS, K>> {
        self.0.iter()
    }

    pub fn row_echelon(&self) -> Self {
        let mut left = self.clone();
        let mut lead = 0;

        for r in 0..ROWS {
            if COLS <= lead {
                break;
            }
            let mut i = r;
            while left.0[i].0[lead] == K::zero() {
                i += 1;
                if ROWS == i {
                    i = r;
                    lead += 1;
                    if COLS == lead {
                        return left;
                    }
                }
            }
            left.0.swap(i, r);

            if left.0[r].0[lead] != K::zero() {
                left.0[r] /= left.0[r].0[lead];
            }
            for i in 0..ROWS {
                if i != r {
                    left.0[i] -= left.0[r].clone() * left.0[i].0[lead];
                }
            }
            lead += 1
        }

        left
    }

    pub fn rank(&self) -> usize {
        let echelon = self.row_echelon();

        echelon.0.iter().filter(|v| v.0 != Vector::<COLS, K>::default().0).count()
    }
}

impl<const ROWS: usize, K> Matrix<ROWS, ROWS, K>
where
    K: Field,
{
    pub fn identity() -> Self {
        let mut mat = Self::default();

        for i in 0..ROWS {
            mat.0[i].0[i] = K::one()
        }
        mat
    }

    pub fn trace(&self) -> K {
        let mut val = K::zero();

        for i in 0..ROWS {
            val += self.0[i].0[i];
        }
        val
    }

    fn determinant_step(&self, row: usize, cols: &mut [usize; ROWS]) -> K {
        let mut ret: K = K::zero();
        let mut min: bool = false;

        for (col, e) in cols.clone().iter().enumerate() {
            if *e == 0 {
                continue;
            }

            if row == ROWS - 1 {
                return self.0[row].0[col];
            }

            cols[col] = 0;
            let scl = self.0[row].0[col] * self.determinant_step(row + 1, cols);
            if min {
                ret -= scl;
            } else {
                ret += scl;
            }
            min = !min;
            cols[col] = 1;
        }

        return ret;
    }

    pub fn determinant(&self) -> K {
        let mut cols: [usize; ROWS] = [1; ROWS];
        self.determinant_step(0, &mut cols)
    }

    pub fn inverse(&self) -> Result<Self, String> {
        if self.determinant() == K::zero() {
            return Err("This matrix does not have inverse.".to_owned());
        }

        let mut left: Self = self.clone();
        let mut right: Self = Matrix::identity();

        for j in 0..ROWS {
            let mut greater: Option<(K, f32, usize)> = Option::default();

            for i in 0..ROWS {
                let num = left.0[i].0[j];
                let abs = num.norm();
                if greater.map_or(0., |f| f.1) < abs {
                    greater = Option::Some((num, abs,  i));
                }
            }

            if greater == None {
                return Err("This matrix does not have inverse.".to_owned())
            }

            let (value, _, k): (K, f32, usize) = greater.unwrap();
            left.0[k] /= value;
            right.0[k] /= value;

            if k != j {
                left.0.swap(k, j);
                right.0.swap(k, j);
            }

            for i in 0..ROWS {
                if i != j {
                    let ratio = left.0[i].0[j] / left.0[j].0[j];
                    right.0[i] -= right.0[j].clone() * ratio;
                    left.0[i] -= left.0[j].clone() * ratio;
                }
            }
        }

        Ok(right)
    }
}

impl<const ROWS: usize, const COLS: usize, K> From<[[K; COLS]; ROWS]> for Matrix<ROWS, COLS, K>
where
    K: Field,
{
    fn from(content: [[K; COLS]; ROWS]) -> Self {
        Self(content.map(Vector::from))
    }
}

impl<const SIZE: usize, K> From<Vector<SIZE, K>> for Matrix<1, SIZE, K>
where
    K: Field,
{
    fn from(v: Vector<SIZE, K>) -> Self {
        Self([v])
    }
}

impl<const ROWS: usize, const COLS: usize, K> Default for Matrix<ROWS, COLS, K>
where
    K: Field,
{
    fn default() -> Self {
        Self([(); ROWS].map(|_| Vector::default()))
    }
}

impl<const ROWS: usize, const COLS: usize, K> Display for Matrix<ROWS, COLS, K>
where
    K: Field + Display,
{
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        for line in &self.0 {
            writeln!(f, "{}", line)?;
        }
        Ok(())
    }
}

impl<const ROWS: usize, const COLS: usize, K> Transpose for Matrix<ROWS, COLS, K>
where
    K: Field,
{
    type Output = Matrix<COLS, ROWS, K>;

    fn transpose(&self) -> Self::Output {
        let mut i = 0;

        Matrix::<COLS, ROWS, K>([(); COLS].map(|_| {
            let mut j = 0;

            i += 1;
            Vector([(); ROWS].map(|_| {
                j += 1;
                self.0[j - 1].0[i - 1]
            }))
        }))
    }
}

impl<const ROWS: usize, const COLS: usize, K> Add<Matrix<ROWS, COLS, K>> for Matrix<ROWS, COLS, K>
where
    K: Field,
{
    type Output = Matrix<ROWS, COLS, K>;

    fn add(self, other: Matrix<ROWS, COLS, K>) -> Self::Output {
        let mut result = self;
        result += other;
        result
    }
}

impl<const ROWS: usize, const COLS: usize, K> Sub<Matrix<ROWS, COLS, K>> for Matrix<ROWS, COLS, K>
where
    K: Field,
{
    type Output = Matrix<ROWS, COLS, K>;

    fn sub(self, other: Matrix<ROWS, COLS, K>) -> Self::Output {
        let mut result = self;
        result -= other;
        result
    }
}

impl<const ROWS: usize, const COLS: usize, K, S> Mul<S> for Matrix<ROWS, COLS, K>
where
    K: Field + MulAssign<S>,
    S: Field,
{
    type Output = Matrix<ROWS, COLS, K>;

    fn mul(self, other: S) -> Self::Output {
        let mut result = self;
        result *= other;
        result
    }
}

impl<const ROWS: usize, const COLS: usize, K> Dot<&Vector<COLS, K>> for Matrix<ROWS, COLS, K>
where
    K: Field,
{
    type Output = Vector<ROWS, K>;

    fn dot(&self, other: &Vector<COLS, K>) -> Self::Output {
        let mut i = 0;

        Vector::<ROWS, K>([(); ROWS].map(|_| {
            i += 1;
            self.0[i - 1].dot(other)
        }))
    }
}

impl<const ROWS: usize, const COLS: usize, const OCOLS: usize, K> Dot<&Matrix<COLS, OCOLS, K>>
    for Matrix<ROWS, COLS, K>
where
    K: Field,
{
    type Output = Matrix<ROWS, OCOLS, K>;

    fn dot(&self, other: &Matrix<COLS, OCOLS, K>) -> Self::Output {
        let other = other.transpose();
        let mut i = 0;

        Matrix::<ROWS, OCOLS, K>([(); ROWS].map(|_| {
            i += 1;

            let mut j = 0;
            Vector([(); OCOLS].map(|_| {
                j += 1;
                self.0[i - 1].dot(&other.0[j - 1])
            }))
        }))
    }
}

impl<const ROWS: usize, const COLS: usize, K> AddAssign<Matrix<ROWS, COLS, K>>
    for Matrix<ROWS, COLS, K>
where
    K: Field,
{
    fn add_assign(&mut self, other: Matrix<ROWS, COLS, K>) {
        self.0
            .iter_mut()
            .zip(other.0.into_iter())
            .for_each(|(v1, v2)| *v1 += v2)
    }
}

impl<const ROWS: usize, const COLS: usize, K> SubAssign<Matrix<ROWS, COLS, K>>
    for Matrix<ROWS, COLS, K>
where
    K: Field,
{
    fn sub_assign(&mut self, other: Matrix<ROWS, COLS, K>) {
        self.0
            .iter_mut()
            .zip(other.0.into_iter())
            .for_each(|(v1, v2)| *v1 -= v2)
    }
}

impl<const ROWS: usize, const COLS: usize, K, S> MulAssign<S> for Matrix<ROWS, COLS, K>
where
    K: Field + MulAssign<S>,
    S: Field,
{
    fn mul_assign(&mut self, other: S) {
        for line in &mut self.0 {
            line.mul_assign(other);
        }
    }
}

impl<const ROWS: usize, const COLS: usize, K> Index<usize> for Matrix<ROWS, COLS, K>
where
    K: Field {
    type Output = Vector<COLS, K>;
    fn index<'a>(&'a self, i: usize) -> &'a Vector<COLS, K> {
        &self.0[i]
    }
}

impl<const ROWS: usize, const COLS: usize, K> IndexMut<usize> for Matrix<ROWS, COLS, K>
where
    K: Field {
    fn index_mut<'a>(&'a mut self, i: usize) -> &'a mut Vector<COLS, K> {
        &mut self.0[i]
    }
}

impl Matrix<4, 4, f32> {
    pub fn projection(fov: f32, ratio: f32, near: f32, far: f32) -> Matrix<4, 4, f32> {
        let fov_rad = (std::f32::consts::PI / 180.) * fov;
        let b = (fov_rad / 2.).tan();

        return Matrix::from([
            [1. / (ratio * b), 0., 0., 0.],
            [0., 1. / b, 0., 0.],
            [0., 0., far / (far - near), -(far * near) / (far - near)],
            [0., 0., 1., 0.],
        ]);
    }
}
