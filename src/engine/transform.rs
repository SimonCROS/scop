use math::{Mat3, Mat4, Vec3};

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Transform {
    pub pivot: Vec3,
    pub translation: Vec3,
    pub scale: Vec3,
    pub rotation: Vec3,
}

impl Transform {
    // Matrix corrsponds to Translate * Ry * Rx * Rz * Scale
    // Rotations correspond to Tait-bryan angles of Y(1), X(2), Z(3)
    // https://en.wikipedia.org/wiki/Euler_angles#Rotation_matrix
    pub fn mat(&self) -> Mat4 {
        let rotate = Mat4::translate(self.pivot * -1.)
            * Mat4::rotate(self.rotation)
            * Mat4::translate(self.pivot);

        rotate * Mat4::scale(self.scale) * Mat4::translate(self.translation - (self.pivot * self.scale))
    }

    pub fn normal_matrix(&self) -> Mat3 {
        let c3: f32 = self.rotation.z.cos();
        let s3: f32 = self.rotation.z.sin();
        let c2: f32 = self.rotation.x.cos();
        let s2: f32 = self.rotation.x.sin();
        let c1: f32 = self.rotation.y.cos();
        let s1: f32 = self.rotation.y.sin();
        let inv_scale: Vec3 = Vec3::from([
            1.0f32 / self.scale[0],
            1.0f32 / self.scale[1],
            1.0f32 / self.scale[2],
        ]);

        return Mat3::from([
            [
                inv_scale.x * (c1 * c3 + s1 * s2 * s3),
                inv_scale.x * (c2 * s3),
                inv_scale.x * (c1 * s2 * s3 - c3 * s1),
            ],
            [
                inv_scale.y * (c3 * s1 * s2 - c1 * s3),
                inv_scale.y * (c2 * c3),
                inv_scale.y * (c1 * c3 * s2 + s1 * s3),
            ],
            [
                inv_scale.z * (c2 * s1),
                inv_scale.z * (-s2),
                inv_scale.z * (c1 * c2),
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
