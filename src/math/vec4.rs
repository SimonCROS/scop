use std::fmt::{self, Display, Formatter};
use std::ops::{Add, AddAssign, Div, DivAssign, Index, IndexMut, Mul, MulAssign, Sub, SubAssign};

use crate::macros::{forward_ref_binop, forward_ref_op_assign};
use crate::{Vec2, Vec3};

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Vec4 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Vec4 {
    pub fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self { x, y, z, w }
    }

    pub fn one() -> Self {
        Self {
            x: 1.,
            y: 1.,
            z: 1.,
            w: 1.,
        }
    }

    pub fn left() -> Self {
        Self {
            x: -1.,
            y: 0.,
            z: 0.,
            w: 0.,
        }
    }

    pub fn right() -> Self {
        Self {
            x: 1.,
            y: 0.,
            z: 0.,
            w: 0.,
        }
    }

    pub fn up() -> Self {
        Self {
            x: 0.,
            y: 1.,
            z: 0.,
            w: 0.,
        }
    }

    pub fn down() -> Self {
        Self {
            x: 0.,
            y: -1.,
            z: 0.,
            w: 0.,
        }
    }

    pub fn forward() -> Self {
        Self {
            x: 0.,
            y: 0.,
            z: -1.,
            w: 0.,
        }
    }

    pub fn backward() -> Self {
        Self {
            x: 0.,
            y: 0.,
            z: 1.,
            w: 0.,
        }
    }

    pub fn negative_infinity() -> Self {
        Self {
            x: f32::MIN,
            y: f32::MIN,
            z: f32::MIN,
            w: f32::MIN,
        }
    }

    pub fn positive_infinity() -> Self {
        Self {
            x: f32::MAX,
            y: f32::MAX,
            z: f32::MAX,
            w: f32::MAX,
        }
    }

    pub fn angle(&self, v: &Self) -> f32 {
        self.dot(v) / self.length_squared()
    }

    pub fn length_squared(&self) -> f32 {
        self.x * self.x + self.y * self.y + self.z * self.z + self.w * self.w
    }

    pub fn length(&self) -> f32 {
        self.length_squared().sqrt()
    }

    pub fn normalized(&self) -> Self {
        let len = self.length();
        *self / len
    }

    pub fn dot(&self, other: &Self) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z + self.w * other.w
    }
}

impl Add for Vec4 {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
            w: self.w + other.w,
        }
    }
}

impl AddAssign for Vec4 {
    fn add_assign(&mut self, other: Vec4) {
        self.x += other.x;
        self.y += other.y;
        self.z += other.z;
        self.w += other.w;
    }
}

impl Sub for Vec4 {
    type Output = Self;

    fn sub(self, other: Self) -> Self::Output {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
            w: self.w - other.w,
        }
    }
}

impl SubAssign for Vec4 {
    fn sub_assign(&mut self, other: Vec4) {
        self.x -= other.x;
        self.y -= other.y;
        self.z -= other.z;
        self.w -= other.w;
    }
}

impl Mul for Vec4 {
    type Output = Self;

    fn mul(self, other: Self) -> Self::Output {
        Self {
            x: self.x * other.x,
            y: self.y * other.y,
            z: self.z * other.z,
            w: self.w * other.w,
        }
    }
}

impl MulAssign for Vec4 {
    fn mul_assign(&mut self, other: Vec4) {
        self.x *= other.x;
        self.y *= other.y;
        self.z *= other.z;
        self.w *= other.w;
    }
}

impl Div for Vec4 {
    type Output = Self;

    fn div(self, other: Self) -> Self::Output {
        Self {
            x: self.x / other.x,
            y: self.y / other.y,
            z: self.z / other.z,
            w: self.w / other.w,
        }
    }
}

impl DivAssign for Vec4 {
    fn div_assign(&mut self, other: Vec4) {
        self.x /= other.x;
        self.y /= other.y;
        self.z /= other.z;
        self.w /= other.w;
    }
}

impl Mul<f32> for Vec4 {
    type Output = Self;

    fn mul(self, other: f32) -> Self::Output {
        Self {
            x: self.x * other,
            y: self.y * other,
            z: self.z * other,
            w: self.w * other,
        }
    }
}

impl Mul<Vec4> for f32 {
    type Output = Vec4;

    fn mul(self, other: Vec4) -> Self::Output {
        other * self
    }
}

impl MulAssign<f32> for Vec4 {
    fn mul_assign(&mut self, other: f32) {
        self.x *= other;
        self.y *= other;
        self.z *= other;
        self.w *= other;
    }
}

impl Div<f32> for Vec4 {
    type Output = Self;

    fn div(self, other: f32) -> Self::Output {
        Self {
            x: self.x / other,
            y: self.y / other,
            z: self.z / other,
            w: self.w / other,
        }
    }
}

impl DivAssign<f32> for Vec4 {
    fn div_assign(&mut self, other: f32) {
        self.x /= other;
        self.y /= other;
        self.z /= other;
        self.w /= other;
    }
}

impl From<[f32; 4]> for Vec4 {
    fn from(content: [f32; 4]) -> Self {
        Self {
            x: content[0],
            y: content[1],
            z: content[2],
            w: content[3],
        }
    }
}

impl Index<usize> for Vec4 {
    type Output = f32;

    fn index(&self, i: usize) -> &f32 {
        match i {
            0 => &self.x,
            1 => &self.y,
            2 => &self.z,
            3 => &self.w,
            _ => panic!("unknown field: {}", i),
        }
    }
}

impl IndexMut<usize> for Vec4 {
    fn index_mut(&mut self, i: usize) -> &mut f32 {
        match i {
            0 => &mut self.x,
            1 => &mut self.y,
            2 => &mut self.z,
            3 => &mut self.w,
            _ => panic!("unknown field: {}", i),
        }
    }
}

impl Display for Vec4 {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "[{}, {}, {}, {}]", self.x, self.y, self.z, self.w)?;
        Ok(())
    }
}

impl From<Vec2> for Vec4 {
    fn from(v: Vec2) -> Self {
        Self {
            x: v.x,
            y: v.y,
            z: 0.0,
            w: 0.0,
        }
    }
}

impl From<Vec3> for Vec4 {
    fn from(v: Vec3) -> Self {
        Self {
            x: v.x,
            y: v.y,
            z: v.z,
            w: 0.0,
        }
    }
}

forward_ref_binop!(impl Add, add for Vec4, Vec4);
forward_ref_binop!(impl Sub, sub for Vec4, Vec4);
forward_ref_binop!(impl Mul, mul for Vec4, Vec4);
forward_ref_binop!(impl Div, div for Vec4, Vec4);
forward_ref_binop!(impl Mul, mul for Vec4, f32);
forward_ref_binop!(impl Mul, mul for f32, Vec4);
forward_ref_binop!(impl Div, div for Vec4, f32);
forward_ref_op_assign!(impl AddAssign, add_assign for Vec4, Vec4);
forward_ref_op_assign!(impl SubAssign, sub_assign for Vec4, Vec4);
forward_ref_op_assign!(impl MulAssign, mul_assign for Vec4, Vec4);
forward_ref_op_assign!(impl DivAssign, div_assign for Vec4, Vec4);
forward_ref_op_assign!(impl MulAssign, mul_assign for Vec4, f32);
forward_ref_op_assign!(impl DivAssign, div_assign for Vec4, f32);
