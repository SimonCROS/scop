use matrix::traits::{Dot, One};

use crate::math::{Matrix3, Matrix4, Vec3};

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Transform {
    pub pivot: Vec3,
    pub translation: Vec3,
    pub scale: Vec3,
    pub rotation: Vec3,
}

impl Transform {
    fn rotate(rotation: Vec3) -> Matrix4 {
        let c3: f32 = rotation.z().cos();
        let s3: f32 = rotation.z().sin();
        let c2: f32 = rotation.x().cos();
        let s2: f32 = rotation.x().sin();
        let c1: f32 = rotation.y().cos();
        let s1: f32 = rotation.y().sin();

        Matrix4::from([
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

    fn scale(scale: Vec3) -> Matrix4 {
        Matrix4::from([
            [scale.x(), 0.0f32, 0.0f32, 0.0f32],
            [0.0f32, scale.y(), 0.0f32, 0.0f32],
            [0.0f32, 0.0f32, scale.z(), 0.0f32],
            [0.0f32, 0.0f32, 0.0f32, 1.0f32],
        ])
    }

    fn translate(translation: Vec3) -> Matrix4 {
        Matrix4::from([
            [1.0f32, 0.0f32, 0.0f32, 0.0f32],
            [0.0f32, 1.0f32, 0.0f32, 0.0f32],
            [0.0f32, 0.0f32, 1.0f32, 0.0f32],
            [translation.x(), translation.y(), translation.z(), 1.0f32],
        ])
    }

    // Matrix corrsponds to Translate * Ry * Rx * Rz * Scale
    // Rotations correspond to Tait-bryan angles of Y(1), X(2), Z(3)
    // https://en.wikipedia.org/wiki/Euler_angles#Rotation_matrix
    pub fn mat(&self) -> Matrix4 {
        let rotate = Self::translate(self.pivot * -1.)
            .dot(&Self::rotate(self.rotation))
            .dot(&Self::translate(self.pivot));

        let mut pivot_to_mul = self.pivot; // TODO Rewrite Vectors to allow Vector * Vector
        pivot_to_mul *= self.scale;

        rotate.dot(&Self::scale(self.scale)).dot(&Self::translate(self.translation - pivot_to_mul))
    }

    pub fn normal_matrix(&self) -> Matrix3 {
        let c3: f32 = self.rotation.z().cos();
        let s3: f32 = self.rotation.z().sin();
        let c2: f32 = self.rotation.x().cos();
        let s2: f32 = self.rotation.x().sin();
        let c1: f32 = self.rotation.y().cos();
        let s1: f32 = self.rotation.y().sin();
        let inv_scale: Vec3 = Vec3::from([
            1.0f32 / self.scale[0],
            1.0f32 / self.scale[1],
            1.0f32 / self.scale[2],
        ]);

        return Matrix3::from([
            [
                inv_scale.x() * (c1 * c3 + s1 * s2 * s3),
                inv_scale.x() * (c2 * s3),
                inv_scale.x() * (c1 * s2 * s3 - c3 * s1),
            ],
            [
                inv_scale.y() * (c3 * s1 * s2 - c1 * s3),
                inv_scale.y() * (c2 * c3),
                inv_scale.y() * (c1 * c3 * s2 + s1 * s3),
            ],
            [
                inv_scale.z() * (c2 * s1),
                inv_scale.z() * (-s2),
                inv_scale.z() * (c1 * c2),
            ],
        ]);
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            pivot: Default::default(),
            translation: Default::default(),
            scale: Vec3::one(),
            rotation: Default::default(),
        }
    }
}
