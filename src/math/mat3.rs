use std::fmt::{Display, Formatter, Result};
use std::ops::{Add, AddAssign, Div, DivAssign, Index, IndexMut, Mul, MulAssign, Sub, SubAssign};

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Mat3([[f32; 3]; 3]);

impl Mat3 {
    pub fn new() -> Self {
        Self([[0.0; 3]; 3])
    }

    pub fn identity() -> Self {
        Self([[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]])
    }

    pub fn determinant(&self) -> f32 {
        let a = self[0][0];
        let b = self[0][1];
        let c = self[0][2];
        let d = self[1][0];
        let e = self[1][1];
        let f = self[1][2];
        let g = self[2][0];
        let h = self[2][1];
        let i = self[2][2];

        a * (e * i - f * h) - b * (d * i - f * g) + c * (d * h - e * g)
    }

    pub fn inverse(&self) -> Option<Self> {
        let det = self.determinant();
        if det.abs() < f32::EPSILON {
            return None;
        }

        let mut result = Self::new();

        // Calculate the adjugate matrix
        for i in 0..3 {
            for j in 0..3 {
                let sign = if (i + j) % 2 == 0 { 1.0 } else { -1.0 };
                let minor = self.minor(i, j);
                result[j][i] = sign * minor / det;
            }
        }

        Some(result)
    }

    pub fn transpose(&self) -> Self {
        let mut result = Self::new();
        for i in 0..3 {
            for j in 0..3 {
                result.0[i][j] = self.0[j][i];
            }
        }
        result
    }

    fn minor(&self, row: usize, col: usize) -> f32 {
        let mut submatrix = [[0.0; 2]; 2];
        let mut sub_row = 0;
        let mut sub_col;

        for i in 0..3 {
            if i == row {
                continue;
            }

            sub_col = 0;

            for j in 0..3 {
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
        let c = submatrix[1][0];
        let d = submatrix[1][1];

        a * d - b * c
    }
}

impl Add for Mat3 {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        let mut result = Self::new();
        for i in 0..3 {
            for j in 0..3 {
                result[i][j] = self[i][j] + other[i][j];
            }
        }
        result
    }
}

impl AddAssign for Mat3 {
    fn add_assign(&mut self, other: Self) {
        for i in 0..3 {
            for j in 0..3 {
                self[i][j] += other[i][j];
            }
        }
    }
}

impl Sub for Mat3 {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        let mut result = Self::new();
        for i in 0..3 {
            for j in 0..3 {
                result[i][j] = self[i][j] - other[i][j];
            }
        }
        result
    }
}

impl SubAssign for Mat3 {
    fn sub_assign(&mut self, other: Self) {
        for i in 0..3 {
            for j in 0..3 {
                self[i][j] -= other[i][j];
            }
        }
    }
}

impl Mul for Mat3 {
    type Output = Self;
    fn mul(self, other: Self) -> Self {
        let mut result = Self::new();
        for i in 0..3 {
            for j in 0..3 {
                for k in 0..3 {
                    result[i][j] += self[i][k] * other[k][j];
                }
            }
        }
        result
    }
}

impl MulAssign for Mat3 {
    fn mul_assign(&mut self, other: Self) {
        *self = *self * other;
    }
}

impl Mul<f32> for Mat3 {
    type Output = Self;

    fn mul(self, scalar: f32) -> Self {
        let mut result = Self::new();
        for i in 0..3 {
            for j in 0..3 {
                result[i][j] = self[i][j] * scalar;
            }
        }
        result
    }
}

impl Mul<Mat3> for f32 {
    type Output = Mat3;

    fn mul(self, matrix: Mat3) -> Mat3 {
        matrix * self
    }
}

impl MulAssign<f32> for Mat3 {
    fn mul_assign(&mut self, scalar: f32) {
        *self = *self * scalar;
    }
}

impl Div<f32> for Mat3 {
    type Output = Self;

    fn div(self, scalar: f32) -> Self {
        let mut result = Self::new();
        for i in 0..3 {
            for j in 0..3 {
                result[i][j] = self[i][j] / scalar;
            }
        }
        result
    }
}

impl DivAssign<f32> for Mat3 {
    fn div_assign(&mut self, scalar: f32) {
        for i in 0..3 {
            for j in 0..3 {
                self[i][j] /= scalar;
            }
        }
    }
}

impl Display for Mat3 {
    fn fmt(&self, f: &mut Formatter) -> Result {
        for i in 0..3 {
            for j in 0..3 {
                write!(f, "{} ", self.0[i][j])?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

impl Index<usize> for Mat3 {
    type Output = [f32; 3];

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl IndexMut<usize> for Mat3 {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index]
    }
}

impl From<[[f32; 3]; 3]> for Mat3 {
    fn from(arr: [[f32; 3]; 3]) -> Self {
        Self(arr)
    }
}
