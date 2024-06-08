use std::fmt::{self, Display, Formatter};
use std::ops::{Add, AddAssign, Div, DivAssign, Index, IndexMut, Mul, MulAssign, Sub, SubAssign};

use crate::macros::{forward_ref_binop, forward_ref_op_assign};
use crate::{Vec2, Vec4};

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3 {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    pub fn one() -> Self {
        Self { x: 1., y: 1., z: 1. }
    }

    pub fn left() -> Self {
        Self {
            x: -1.,
            y: 0.,
            z: 0.,
        }
    }

    pub fn right() -> Self {
        Self {
            x: 1.,
            y: 0.,
            z: 0.,
        }
    }

    pub fn up() -> Self {
        Self {
            x: 0.,
            y: 1.,
            z: 0.,
        }
    }

    pub fn down() -> Self {
        Self {
            x: 0.,
            y: -1.,
            z: 0.,
        }
    }

    pub fn forward() -> Self {
        Self {
            x: 0.,
            y: 0.,
            z: 1.,
        }
    }

    pub fn backward() -> Self {
        Self {
            x: 0.,
            y: 0.,
            z: -1.,
        }
    }

    pub fn negative_infinity() -> Self {
        Self {
            x: f32::MIN,
            y: f32::MIN,
            z: f32::MIN,
        }
    }

    pub fn positive_infinity() -> Self {
        Self {
            x: f32::MAX,
            y: f32::MAX,
            z: f32::MAX,
        }
    }

    pub fn from_angle_x(angle: f32) -> Self {
        Self {
            x: 0.0,
            y: angle.cos(),
            z: angle.sin(),
        }
    }

    pub fn from_angle_y(angle: f32) -> Self {
        Self {
            x: angle.cos(),
            y: 0.0,
            z: angle.sin(),
        }
    }

    pub fn from_angle_z(angle: f32) -> Self {
        Self {
            x: angle.cos(),
            y: angle.sin(),
            z: 0.0,
        }
    }

    pub fn from_axis(axis: Vec3, angle: f32) -> Self {
        let cos_angle = angle.cos();
        let sin_angle = angle.sin();
        let x = axis.x;
        let y = axis.y;
        let z = axis.z;
        Self {
            x: (cos_angle + (1.0 - cos_angle) * x * x) * axis.x
                + ((1.0 - cos_angle) * x * y - sin_angle * z) * axis.y
                + ((1.0 - cos_angle) * x * z + sin_angle * y) * axis.z,
            y: ((1.0 - cos_angle) * x * y + sin_angle * z) * axis.x
                + (cos_angle + (1.0 - cos_angle) * y * y) * axis.y
                + ((1.0 - cos_angle) * y * z - sin_angle * x) * axis.z,
            z: ((1.0 - cos_angle) * x * z - sin_angle * y) * axis.x
                + ((1.0 - cos_angle) * y * z + sin_angle * x) * axis.y
                + (cos_angle + (1.0 - cos_angle) * z * z) * axis.z,
        }
    }

    pub fn angle(&self, v: &Self) -> f32 {
        self.dot(v) / self.length_squared()
    }

    pub fn length_squared(&self) -> f32 {
        self.x * self.x + self.y * self.y + self.z * self.z
    }

    pub fn length(&self) -> f32 {
        self.length_squared().sqrt()
    }

    pub fn normalized(&self) -> Self {
        let len = self.length();
        *self / len
    }

    pub fn dot(&self, other: &Self) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }
}

impl Add for Vec3 {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        }
    }
}

impl AddAssign for Vec3 {
    fn add_assign(&mut self, other: Vec3) {
        self.x += other.x;
        self.y += other.y;
        self.z += other.z;
    }
}

impl Sub for Vec3 {
    type Output = Self;

    fn sub(self, other: Self) -> Self::Output {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
        }
    }
}

impl SubAssign for Vec3 {
    fn sub_assign(&mut self, other: Vec3) {
        self.x -= other.x;
        self.y -= other.y;
        self.z -= other.z;
    }
}

impl Mul for Vec3 {
    type Output = Self;

    fn mul(self, other: Self) -> Self::Output {
        Self {
            x: self.x * other.x,
            y: self.y * other.y,
            z: self.z * other.z,
        }
    }
}

impl MulAssign for Vec3 {
    fn mul_assign(&mut self, other: Vec3) {
        self.x *= other.x;
        self.y *= other.y;
        self.z *= other.z;
    }
}

impl DivAssign for Vec3 {
    fn div_assign(&mut self, other: Vec3) {
        self.x /= other.x;
        self.y /= other.y;
        self.z /= other.z;
    }
}

impl Mul<f32> for Vec3 {
    type Output = Self;

    fn mul(self, other: f32) -> Self::Output {
        Self {
            x: self.x * other,
            y: self.y * other,
            z: self.z * other,
        }
    }
}

impl Mul<Vec3> for f32 {
    type Output = Vec3;

    fn mul(self, other: Vec3) -> Self::Output {
        other * self
    }
}

impl MulAssign<f32> for Vec3 {
    fn mul_assign(&mut self, other: f32) {
        self.x *= other;
        self.y *= other;
        self.z *= other;
    }
}

impl Div for Vec3 {
    type Output = Self;

    fn div(self, other: Self) -> Self::Output {
        Self {
            x: self.x / other.x,
            y: self.y / other.y,
            z: self.z / other.z,
        }
    }
}

impl Div<f32> for Vec3 {
    type Output = Self;

    fn div(self, other: f32) -> Self::Output {
        Self {
            x: self.x / other,
            y: self.y / other,
            z: self.z / other,
        }
    }
}

impl DivAssign<f32> for Vec3 {
    fn div_assign(&mut self, other: f32) {
        self.x /= other;
        self.y /= other;
        self.z /= other;
    }
}

impl From<[f32; 3]> for Vec3 {
    fn from(content: [f32; 3]) -> Self {
        Self {
            x: content[0],
            y: content[1],
            z: content[2],
        }
    }
}

impl Index<usize> for Vec3 {
    type Output = f32;

    fn index(&self, i: usize) -> &f32 {
        match i {
            0 => &self.x,
            1 => &self.y,
            2 => &self.z,
            _ => panic!("unknown field: {}", i),
        }
    }
}

impl IndexMut<usize> for Vec3 {
    fn index_mut(&mut self, i: usize) -> &mut f32 {
        match i {
            0 => &mut self.x,
            1 => &mut self.y,
            2 => &mut self.z,
            _ => panic!("unknown field: {}", i),
        }
    }
}

impl Display for Vec3 {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "[{}, {}, {}]", self.x, self.y, self.z)?;
        Ok(())
    }
}

impl From<Vec2> for Vec3 {
    fn from(v: Vec2) -> Self {
        Self {
            x: v.x,
            y: v.y,
            z: 0.0,
        }
    }
}

impl From<Vec4> for Vec3 {
    fn from(v: Vec4) -> Self {
        Self {
            x: v.x,
            y: v.y,
            z: v.z,
        }
    }
}

forward_ref_binop!(impl Add, add for Vec3, Vec3);
forward_ref_binop!(impl Sub, sub for Vec3, Vec3);
forward_ref_binop!(impl Mul, mul for Vec3, Vec3);
forward_ref_binop!(impl Div, div for Vec3, Vec3);
forward_ref_binop!(impl Mul, mul for Vec3, f32);
forward_ref_binop!(impl Mul, mul for f32, Vec3);
forward_ref_binop!(impl Div, div for Vec3, f32);
forward_ref_op_assign!(impl AddAssign, add_assign for Vec3, Vec3);
forward_ref_op_assign!(impl SubAssign, sub_assign for Vec3, Vec3);
forward_ref_op_assign!(impl MulAssign, mul_assign for Vec3, Vec3);
forward_ref_op_assign!(impl DivAssign, div_assign for Vec3, Vec3);
forward_ref_op_assign!(impl MulAssign, mul_assign for Vec3, f32);
forward_ref_op_assign!(impl DivAssign, div_assign for Vec3, f32);
