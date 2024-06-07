use matrix::traits::Dot;

use crate::math::{Matrix4, Vector3};

pub struct Camera {
    projection_matrix: Matrix4,
    view_matrix: Matrix4,
    inverse_view_matrix: Matrix4,
}

impl Camera {
    pub fn empty() -> Camera {
        Self {
            projection_matrix: Matrix4::identity(),
            view_matrix: Matrix4::identity(),
            inverse_view_matrix: Matrix4::identity(),
        }
    }

    pub fn set_orthographic_projection(
        &mut self,
        _left: f32,
        _right: f32,
        _top: f32,
        _bottom: f32,
        _near: f32,
        _far: f32,
    ) {
        unimplemented!()
    }

    pub fn set_perspective_projection(&mut self, fovy: f32, aspect: f32, near: f32, far: f32) {
        assert!((aspect - f32::EPSILON).abs() > 0f32);

        let fovy_rad = (std::f32::consts::PI / 180.) * fovy;
        let tan_half_fovy = (fovy_rad / 2f32).tan();
        self.projection_matrix[0][0] = 1f32 / (aspect * tan_half_fovy);
        self.projection_matrix[1][1] = 1f32 / (tan_half_fovy);
        self.projection_matrix[2][2] = far / (far - near);
        self.projection_matrix[2][3] = 1f32;
        self.projection_matrix[3][2] = -(far * near) / (far - near);
    }

    pub fn get_projection(&self) -> &Matrix4 {
        &self.projection_matrix
    }

    pub fn get_view(&self) -> &Matrix4 {
        &self.view_matrix
    }

    pub fn get_inverse_view(&self) -> &Matrix4 {
        &self.inverse_view_matrix
    }

    pub fn get_position(&self) -> Vector3 {
        Vector3::from([
            self.inverse_view_matrix[0][0],
            self.inverse_view_matrix[1][0],
            self.inverse_view_matrix[2][0],
        ])
    }

    pub fn set_view_direction(&mut self, position: Vector3, direction: Vector3, up: Vector3) {
        let w = direction.normalize();
        let u = w.cross(&up).normalize();
        let v = w.cross(&u);

        self.view_matrix[0][0] = u.x();
        self.view_matrix[1][0] = u.y();
        self.view_matrix[2][0] = u.z();
        self.view_matrix[0][1] = v.x();
        self.view_matrix[1][1] = v.y();
        self.view_matrix[2][1] = v.z();
        self.view_matrix[0][2] = w.x();
        self.view_matrix[1][2] = w.y();
        self.view_matrix[2][2] = w.z();
        self.view_matrix[3][0] = -u.dot(&position);
        self.view_matrix[3][1] = -v.dot(&position);
        self.view_matrix[3][2] = -w.dot(&position);

        self.inverse_view_matrix[0][0] = u.x();
        self.inverse_view_matrix[0][1] = u.y();
        self.inverse_view_matrix[0][2] = u.z();
        self.inverse_view_matrix[1][0] = v.x();
        self.inverse_view_matrix[1][1] = v.y();
        self.inverse_view_matrix[1][2] = v.z();
        self.inverse_view_matrix[2][0] = w.x();
        self.inverse_view_matrix[2][1] = w.y();
        self.inverse_view_matrix[2][2] = w.z();
        self.inverse_view_matrix[3][0] = position.x();
        self.inverse_view_matrix[3][1] = position.y();
        self.inverse_view_matrix[3][2] = position.z();
    }

    pub fn set_view_target(&mut self, position: Vector3, target: Vector3, up: Vector3) {
        self.set_view_direction(position, target - position, up);
    }

    pub fn set_view_yxz(&mut self, position: Vector3, rotation: Vector3) {
        let c3 = rotation.z().cos();
        let s3 = rotation.z().sin();
        let c2 = rotation.x().cos();
        let s2 = rotation.x().sin();
        let c1 = rotation.y().cos();
        let s1 = rotation.y().sin();

        let u = Vector3::from([c1 * c3 + s1 * s2 * s3, c2 * s3, c1 * s2 * s3 - c3 * s1]);
        let v = Vector3::from([c3 * s1 * s2 - c1 * s3, c2 * c3, c1 * c3 * s2 + s1 * s3]);
        let w = Vector3::from([c2 * s1, -s2, c1 * c2]);

        self.view_matrix[0][0] = u.x();
        self.view_matrix[1][0] = u.y();
        self.view_matrix[2][0] = u.z();
        self.view_matrix[0][1] = v.x();
        self.view_matrix[1][1] = v.y();
        self.view_matrix[2][1] = v.z();
        self.view_matrix[0][2] = w.x();
        self.view_matrix[1][2] = w.y();
        self.view_matrix[2][2] = w.z();
        self.view_matrix[3][0] = -u.dot(&position);
        self.view_matrix[3][1] = -v.dot(&position);
        self.view_matrix[3][2] = -w.dot(&position);

        self.inverse_view_matrix[0][0] = u.x();
        self.inverse_view_matrix[0][1] = u.y();
        self.inverse_view_matrix[0][2] = u.z();
        self.inverse_view_matrix[1][0] = v.x();
        self.inverse_view_matrix[1][1] = v.y();
        self.inverse_view_matrix[1][2] = v.z();
        self.inverse_view_matrix[2][0] = w.x();
        self.inverse_view_matrix[2][1] = w.y();
        self.inverse_view_matrix[2][2] = w.z();
        self.inverse_view_matrix[3][0] = position.x();
        self.inverse_view_matrix[3][1] = position.y();
        self.inverse_view_matrix[3][2] = position.z();
    }
}
