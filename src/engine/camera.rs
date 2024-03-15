use matrix::traits::Dot;

use crate::{Matrix4, Vector3};

pub struct Camera {
    velocity: Vector3,
    position: Vector3,
    view_matrix: Matrix4,
}

impl Camera {
    // pub fn get_view_matrix(&self) -> Matrix4 {
    //     // to create a correct model view, we need to move the world in opposite
    //     // direction to the camera
    //     //  so we will create the camera model matrix and invert
    //     Matrix::<4, 4, f32>::identity()
    //         .translate(&-self.position)
    //         .rotate_y(-self.yaw)
    //         .rotate_x(-self.pitch);
    //     let camera_translation = translate(&Mat4::identity(), self.position);
    //     let camera_rotation = self.get_rotation_matrix();
    //     inverse(&(camera_translation * camera_rotation))
    // }

    // fn get_rotation_matrix(&self) -> Matrix4 {
    //     // fairly typical FPS style camera. we join the pitch and yaw rotations into
    //     // the final rotation matrix
    //     let pitch_rotation = Vector3::from([1.0, 0.0, 0.0]).(self.pitch);
    //     let yaw_rotation = angle_axis(self.yaw, &glm::vec3(0.0, -1.0, 0.0));
    //     to_mat4(&yaw_rotation) * to_mat4(&pitch_rotation)
    // }

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
    }
}
