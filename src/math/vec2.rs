use std::{
    fmt::{Display, Formatter, Result},
    ops::{Add, AddAssign, Div, DivAssign, Index, IndexMut, Mul, MulAssign, Sub, SubAssign},
};

use crate::{macros::{forward_ref_binop, forward_ref_op_assign}, Vec3, Vec4};

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn left() -> Self {
        Self { x: -1., y: 0. }
    }

    pub fn right() -> Self {
        Self { x: 1., y: 0. }
    }

    pub fn up() -> Self {
        Self { x: 0., y: 1. }
    }

    pub fn down() -> Self {
        Self { x: 0., y: -1. }
    }

    pub fn negative_infinity() -> Self {
        Self {
            x: f32::MIN,
            y: f32::MIN,
        }
    }

    pub fn positive_infinity() -> Self {
        Self {
            x: f32::MAX,
            y: f32::MAX,
        }
    }

    pub fn from_angle(angle: f32) -> Self {
        Self {
            x: angle.cos(),
            y: angle.sin(),
        }
    }

    pub fn angle(&self, v: &Self) -> f32 {
        self.dot(v) / self.length_squared()
    }

    pub fn length_squared(&self) -> f32 {
        self.x * self.x + self.y * self.y
    }

    pub fn length(&self) -> f32 {
        self.length_squared().sqrt()
    }

    pub fn normalized(&self) -> Self {
        let len = self.length();
        *self / len
    }

    pub fn dot(&self, other: &Self) -> f32 {
        self.x * other.x + self.y * other.y
    }
}

impl Add for Vec2 {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl AddAssign for Vec2 {
    fn add_assign(&mut self, other: Vec2) {
        self.x += other.x;
        self.y += other.y;
    }
}

impl Sub for Vec2 {
    type Output = Self;

    fn sub(self, other: Self) -> Self::Output {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

impl SubAssign for Vec2 {
    fn sub_assign(&mut self, other: Vec2) {
        self.x -= other.x;
        self.y -= other.y;
    }
}

impl Mul for Vec2 {
    type Output = Self;

    fn mul(self, other: Self) -> Self::Output {
        Self {
            x: self.x * other.x,
            y: self.y * other.y,
        }
    }
}

impl MulAssign for Vec2 {
    fn mul_assign(&mut self, other: Vec2) {
        self.x *= other.x;
        self.y *= other.y;
    }
}

impl Div for Vec2 {
    type Output = Self;

    fn div(self, other: Self) -> Self::Output {
        Self {
            x: self.x * other.x,
            y: self.y * other.y,
        }
    }
}

impl DivAssign for Vec2 {
    fn div_assign(&mut self, other: Vec2) {
        self.x /= other.x;
        self.y /= other.y;
    }
}

impl Mul<f32> for Vec2 {
    type Output = Self;

    fn mul(self, other: f32) -> Self::Output {
        Self {
            x: self.x * other,
            y: self.y * other,
        }
    }
}

impl Mul<Vec2> for f32 {
    type Output = Vec2;

    fn mul(self, other: Vec2) -> Self::Output {
        other * self
    }
}

impl MulAssign<f32> for Vec2 {
    fn mul_assign(&mut self, other: f32) {
        self.x *= other;
        self.y *= other;
    }
}

impl Div<f32> for Vec2 {
    type Output = Self;

    fn div(self, other: f32) -> Self::Output {
        Self {
            x: self.x / other,
            y: self.y / other,
        }
    }
}

impl DivAssign<f32> for Vec2 {
    fn div_assign(&mut self, other: f32) {
        self.x /= other;
        self.y /= other;
    }
}

impl From<[f32; 2]> for Vec2 {
    fn from(content: [f32; 2]) -> Self {
        Self {
            x: content[0],
            y: content[1],
        }
    }
}

impl Index<usize> for Vec2 {
    type Output = f32;

    fn index(&self, i: usize) -> &f32 {
        match i {
            0 => &self.x,
            1 => &self.y,
            _ => panic!("unknown field: {}", i),
        }
    }
}

impl IndexMut<usize> for Vec2 {
    fn index_mut(&mut self, i: usize) -> &mut f32 {
        match i {
            0 => &mut self.x,
            1 => &mut self.y,
            _ => panic!("unknown field: {}", i),
        }
    }
}

impl Display for Vec2 {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "[{}, {}]", self.x, self.y)?;
        Ok(())
    }
}

impl From<Vec3> for Vec2 {
    fn from(v: Vec3) -> Self {
        Self { x: v.x, y: v.y }
    }
}

impl From<Vec4> for Vec2 {
    fn from(v: Vec4) -> Self {
        Self { x: v.x, y: v.y }
    }
}

forward_ref_binop!(impl Add, add for Vec2, Vec2);
forward_ref_binop!(impl Sub, sub for Vec2, Vec2);
forward_ref_binop!(impl Mul, mul for Vec2, Vec2);
forward_ref_binop!(impl Div, div for Vec2, Vec2);
forward_ref_binop!(impl Mul, mul for Vec2, f32);
forward_ref_binop!(impl Mul, mul for f32, Vec2);
forward_ref_binop!(impl Div, div for Vec2, f32);
forward_ref_op_assign!(impl AddAssign, add_assign for Vec2, Vec2);
forward_ref_op_assign!(impl SubAssign, sub_assign for Vec2, Vec2);
forward_ref_op_assign!(impl MulAssign, mul_assign for Vec2, Vec2);
forward_ref_op_assign!(impl DivAssign, div_assign for Vec2, Vec2);
forward_ref_op_assign!(impl MulAssign, mul_assign for Vec2, f32);
forward_ref_op_assign!(impl DivAssign, div_assign for Vec2, f32);
