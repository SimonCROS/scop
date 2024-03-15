use crate::{Matrix4, Vector3};

pub struct Camera {
    velocity: Vector3,
    position: Vector3,
    pitch: f32, // vertical rotation
    yaw: f32, // horizontal rotation
}

impl Camera {
    pub fn get_view_matrix(&self) -> Matrix4 {
        // to create a correct model view, we need to move the world in opposite
        // direction to the camera
        //  so we will create the camera model matrix and invert
        Matrix::<4, 4, f32>::identity()
            .translate(&-self.position)
            .rotate_y(-self.yaw)
            .rotate_x(-self.pitch);
        let camera_translation = translate(&Mat4::identity(), self.position);
        let camera_rotation = self.get_rotation_matrix();
        inverse(&(camera_translation * camera_rotation))
    }

    fn get_rotation_matrix(&self) -> Matrix4 {
        // fairly typical FPS style camera. we join the pitch and yaw rotations into
        // the final rotation matrix
        let pitch_rotation = Vector3::from([1.0, 0.0, 0.0]).(self.pitch);
        let yaw_rotation = angle_axis(self.yaw, &glm::vec3(0.0, -1.0, 0.0));
        to_mat4(&yaw_rotation) * to_mat4(&pitch_rotation)
    }

    // pub fn getViewMatrix(&self) -> [f32; 16] {
    //     let mut view = [0.0; 16];
    //     let cos_pitch = self.pitch.cos();
    //     let sin_pitch = self.pitch.sin();
    //     let cos_yaw = self.yaw.cos();
    //     let sin_yaw = self.yaw.sin();
    //     let cos_pitch_sin_yaw = cos_pitch * sin_yaw;
    //     let sin_pitch_sin_yaw = sin_pitch * sin_yaw;
    //     let cos_pitch_cos_yaw = cos_pitch * cos_yaw;
    //     let sin_pitch_cos_yaw = sin_pitch * cos_yaw;
    //     view[0] = cos_yaw;
    //     view[1] = 0.0;
    //     view[2] = -sin_yaw;
    //     view[3] = 0.0;
    //     view[4] = sin_pitch_sin_yaw;
    //     view[5] = cos_pitch;
    //     view[6] = cos_pitch_sin_yaw;
    //     view[7] = 0.0;
    //     view[8] = sin_pitch_cos_yaw;
    //     view[9] = -sin_pitch;
    //     view[10] = cos_pitch_cos_yaw;
    //     view[11] = 0.0;
    //     view[12] = -self.position[0] * view[0] - self.position[1] * view[4] - self.position[2] * view[8];
    //     view[13] = -self.position[0] * view[1] - self.position[1] * view[5] - self.position[2] * view[9];
    //     view[14] = -self.position[0] * view[2] - self.position[1] * view[6] - self.position[2] * view[10];
    //     view[15] = 1.0;
    //     view
    // }

    // pub fn getRotationMatrix(&self) -> [f32; 16] {
    //     let mut rotation = [0.0; 16];
    //     let cos_pitch = self.pitch.cos();
    //     let sin_pitch = self.pitch.sin();
    //     let cos_yaw = self.yaw.cos();
    //     let sin_yaw = self.yaw.sin();
    //     let cos_pitch_sin_yaw = cos_pitch * sin_yaw;
    //     let sin_pitch_sin_yaw = sin_pitch * sin_yaw;
    //     let cos_pitch_cos_yaw = cos_pitch * cos_yaw;
    //     let sin_pitch_cos_yaw = sin_pitch * cos_yaw;
    //     rotation[0] = cos_yaw;
    //     rotation[1] = 0.0;
    //     rotation[2] = -sin_yaw;
    //     rotation[3] = 0.0;
    //     rotation[4] = sin_pitch_sin_yaw;
    //     rotation[5] = cos_pitch;
    //     rotation[6] = cos_pitch_sin_yaw;
    //     rotation[7] = 0.0;
    //     rotation[8] = sin_pitch_cos_yaw;
    //     rotation[9] = -sin_pitch;
    //     rotation[10] = cos_pitch_cos_yaw;
    //     rotation[11] = 0.0;
    //     rotation[12] = 0.0;
    //     rotation[13] = 0.0;
    //     rotation[14] = 0.0;
    //     rotation[15] = 1.0;
    //     rotation
    // }
}
