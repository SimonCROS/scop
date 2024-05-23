use matrix::traits::{One, Zero};

use crate::math::{Matrix3, Matrix4, Up, Vector3};

#[derive(Clone, Debug, PartialEq)]
pub struct Transform {
    translation: Vector3,
    scale: Vector3,
    rotation: Vector3,
}

impl Transform {
    pub fn new() -> Transform {
        Transform {
            translation: Vector3::zero(),
            scale: Vector3::one(),
            rotation: Vector3::up(),
        }
    }

    // Matrix corrsponds to Translate * Ry * Rx * Rz * Scale
    // Rotations correspond to Tait-bryan angles of Y(1), X(2), Z(3)
    // https://en.wikipedia.org/wiki/Euler_angles#Rotation_matrix
    pub fn mat(&self) -> Matrix4 {
        let c3: f32 = self.rotation.z().cos();
        let s3: f32 = self.rotation.z().sin();
        let c2: f32 = self.rotation.x().cos();
        let s2: f32 = self.rotation.x().sin();
        let c1: f32 = self.rotation.y().cos();
        let s1: f32 = self.rotation.y().sin();
        return Matrix4::from([
            [
                self.scale.x() * (c1 * c3 + s1 * s2 * s3),
                self.scale.x() * (c2 * s3),
                self.scale.x() * (c1 * s2 * s3 - c3 * s1),
                0.0f32,
            ],
            [
                self.scale.y() * (c3 * s1 * s2 - c1 * s3),
                self.scale.y() * (c2 * c3),
                self.scale.y() * (c1 * c3 * s2 + s1 * s3),
                0.0f32,
            ],
            [
                self.scale.z() * (c2 * s1),
                self.scale.z() * (-s2),
                self.scale.z() * (c1 * c2),
                0.0f32,
            ],
            [
                self.translation.x(),
                self.translation.y(),
                self.translation.z(),
                1.0f32,
            ],
        ]);
    }

    pub fn normal_matrix(&self) -> Matrix3 {
        let c3: f32 = self.rotation.z().cos();
        let s3: f32 = self.rotation.z().sin();
        let c2: f32 = self.rotation.x().cos();
        let s2: f32 = self.rotation.x().sin();
        let c1: f32 = self.rotation.y().cos();
        let s1: f32 = self.rotation.y().sin();
        let inv_scale: Vector3 = Vector3::from([
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