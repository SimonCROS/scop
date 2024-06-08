use std::fmt::{Display, Formatter, Result};
use std::ops::{Add, AddAssign, Div, DivAssign, Index, IndexMut, Mul, MulAssign, Sub, SubAssign};

use crate::macros::{forward_ref_binop, forward_ref_op_assign};
use crate::Vec3;

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Mat4([[f32; 4]; 4]);

impl Mat4 {
    pub fn new() -> Self {
        Self([[0.0; 4]; 4])
    }

    pub fn identity() -> Self {
        Self([
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ])
    }

    pub fn determinant(&self) -> f32 {
        let a = self[0][0];
        let b = self[0][1];
        let c = self[0][2];
        let d = self[0][3];
        let e = self[1][0];
        let f = self[1][1];
        let g = self[1][2];
        let h = self[1][3];
        let i = self[2][0];
        let j = self[2][1];
        let k = self[2][2];
        let l = self[2][3];
        let m = self[3][0];
        let n = self[3][1];
        let o = self[3][2];
        let p = self[3][3];

        a * (f * (k * p - l * o) - g * (j * p - l * n) + h * (j * o - k * n))
            - b * (e * (k * p - l * o) - g * (i * p - l * m) + h * (i * o - k * m))
            + c * (e * (j * p - l * n) - f * (i * p - l * m) + h * (i * n - j * m))
            - d * (e * (j * o - k * n) - f * (i * o - k * m) + g * (i * n - j * m))
    }

    pub fn inverse(&self) -> Option<Self> {
        let det = self.determinant();
        if det.abs() < f32::EPSILON {
            return None;
        }

        let mut result = Self::new();

        for i in 0..4 {
            for j in 0..4 {
                let sign = if (i + j) % 2 == 0 { 1.0 } else { -1.0 };
                let minor = self.minor(i, j);
                result[j][i] = sign * minor / det;
            }
        }

        Some(result)
    }

    pub fn transpose(&self) -> Self {
        let mut result = Self::new();
        for i in 0..4 {
            for j in 0..4 {
                result.0[i][j] = self.0[j][i];
            }
        }
        result
    }

    fn minor(&self, row: usize, col: usize) -> f32 {
        let mut submatrix = [[0.0; 3]; 3];
        let mut sub_row = 0;
        let mut sub_col;

        for i in 0..4 {
            if i == row {
                continue;
            }

            sub_col = 0;

            for j in 0..4 {
                if j == col {
                    continue;
                }

                submatrix[sub_row][sub_col] = self[i][j];
                sub_col += 1;
            }

            sub_row += 1;
        }

        let a = submatrix[0][0];
        let b = submatrix[0][1];
        let c = submatrix[0][2];
        let d = submatrix[1][0];
        let e = submatrix[1][1];
        let f = submatrix[1][2];
        let g = submatrix[2][0];
        let h = submatrix[2][1];
        let i = submatrix[2][2];

        a * (e * i - f * h) - b * (d * i - f * g) + c * (d * h - e * g)
    }

    pub fn rotate(rotation: Vec3) -> Mat4 {
        let c3: f32 = rotation.z.cos();
        let s3: f32 = rotation.z.sin();
        let c2: f32 = rotation.x.cos();
        let s2: f32 = rotation.x.sin();
        let c1: f32 = rotation.y.cos();
        let s1: f32 = rotation.y.sin();

        Mat4::from([
            [
                c1 * c3 + s1 * s2 * s3,
                c2 * s3,
                c1 * s2 * s3 - c3 * s1,
                0.0f32,
            ],
            [
                c3 * s1 * s2 - c1 * s3,
                c2 * c3,
                c1 * c3 * s2 + s1 * s3,
                0.0f32,
            ],
            [c2 * s1, -s2, c1 * c2, 0.0f32],
            [0.0f32, 0.0f32, 0.0f32, 1.0f32],
        ])
    }

    pub fn scale(factor: Vec3) -> Mat4 {
        Mat4::from([
            [factor.x, 0.0f32, 0.0f32, 0.0f32],
            [0.0f32, factor.y, 0.0f32, 0.0f32],
            [0.0f32, 0.0f32, factor.z, 0.0f32],
            [0.0f32, 0.0f32, 0.0f32, 1.0f32],
        ])
    }

    pub fn translate(translation: Vec3) -> Mat4 {
        Mat4::from([
            [1.0f32, 0.0f32, 0.0f32, 0.0f32],
            [0.0f32, 1.0f32, 0.0f32, 0.0f32],
            [0.0f32, 0.0f32, 1.0f32, 0.0f32],
            [translation.x, translation.y, translation.z, 1.0f32],
        ])
    }
}

impl Add for Mat4 {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        let mut result = Self::new();
        for i in 0..4 {
            for j in 0..4 {
                result[i][j] = self[i][j] + other[i][j];
            }
        }
        result
    }
}

impl AddAssign for Mat4 {
    fn add_assign(&mut self, other: Self) {
        for i in 0..4 {
            for j in 0..4 {
                self[i][j] += other[i][j];
            }
        }
    }
}

impl Sub for Mat4 {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        let mut result = Self::new();
        for i in 0..4 {
            for j in 0..4 {
                result[i][j] = self[i][j] - other[i][j];
            }
        }
        result
    }
}

impl SubAssign for Mat4 {
    fn sub_assign(&mut self, other: Self) {
        for i in 0..4 {
            for j in 0..4 {
                self[i][j] -= other[i][j];
            }
        }
    }
}

impl Mul for Mat4 {
    type Output = Self;
    fn mul(self, other: Self) -> Self {
        let mut result = Self::new();
        for i in 0..4 {
            for j in 0..4 {
                for k in 0..4 {
                    result[i][j] += self[i][k] * other[k][j];
                }
            }
        }
        result
    }
}

impl MulAssign for Mat4 {
    fn mul_assign(&mut self, other: Self) {
        *self = *self * other;
    }
}

impl Mul<f32> for Mat4 {
    type Output = Self;

    fn mul(self, scalar: f32) -> Self {
        let mut result = Self::new();
        for i in 0..4 {
            for j in 0..4 {
                result[i][j] = self[i][j] * scalar;
            }
        }
        result
    }
}

impl Mul<Mat4> for f32 {
    type Output = Mat4;

    fn mul(self, matrix: Mat4) -> Mat4 {
        matrix * self
    }
}

impl MulAssign<f32> for Mat4 {
    fn mul_assign(&mut self, scalar: f32) {
        *self = *self * scalar;
    }
}

impl Div<f32> for Mat4 {
    type Output = Self;

    fn div(self, scalar: f32) -> Self {
        let mut result = Self::new();
        for i in 0..4 {
            for j in 0..4 {
                result[i][j] = self[i][j] / scalar;
            }
        }
        result
    }
}

impl DivAssign<f32> for Mat4 {
    fn div_assign(&mut self, scalar: f32) {
        for i in 0..4 {
            for j in 0..4 {
                self[i][j] /= scalar;
            }
        }
    }
}

impl Display for Mat4 {
    fn fmt(&self, f: &mut Formatter) -> Result {
        for i in 0..4 {
            for j in 0..4 {
                write!(f, "{} ", self.0[i][j])?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

impl Index<usize> for Mat4 {
    type Output = [f32; 4];

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl IndexMut<usize> for Mat4 {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index]
    }
}

impl From<[[f32; 4]; 4]> for Mat4 {
    fn from(arr: [[f32; 4]; 4]) -> Self {
        Self(arr)
    }
}

forward_ref_binop!(impl Add, add for Mat4, Mat4);
forward_ref_binop!(impl Sub, sub for Mat4, Mat4);
forward_ref_binop!(impl Mul, mul for Mat4, Mat4);
forward_ref_binop!(impl Mul, mul for Mat4, f32);
forward_ref_binop!(impl Mul, mul for f32, Mat4);
forward_ref_binop!(impl Div, div for Mat4, f32);
forward_ref_op_assign!(impl AddAssign, add_assign for Mat4, Mat4);
forward_ref_op_assign!(impl SubAssign, sub_assign for Mat4, Mat4);
forward_ref_op_assign!(impl MulAssign, mul_assign for Mat4, Mat4);
forward_ref_op_assign!(impl MulAssign, mul_assign for Mat4, f32);
forward_ref_op_assign!(impl DivAssign, div_assign for Mat4, f32);
