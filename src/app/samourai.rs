use anyhow::Result;
use ash::vk;
use math::Vec3;
use winit::keyboard::{Key, KeyCode, NamedKey};

use crate::{
    engine::{camera::Camera, Engine, GameObject, Transform},
    parsing::{read_frag_spv_file, read_obj_file, read_tga_r8g8b8a8_srgb_file, read_vert_spv_file},
    renderer::{Material, MaterialInstance, ScopDescriptorSetLayout},
};

#[derive(Default)]
pub struct AppSamourai {
    texture_target_fade: f32,
    texture_change_frame: u32,
    current_pitch: f32,
    current_yaw: f32,
    current_roll: f32,
}

impl AppSamourai {
    pub fn start(&mut self, engine: &mut Engine) -> Result<()> {
        // --------------------
        // Meshs
        // --------------------

        let mesh_samourai = read_obj_file(engine, "./resources/samourai2.obj")?;

        let mesh_socle = read_obj_file(engine, "./resources/socle_samourai.obj")?;

        let mesh_katana = read_obj_file(engine, "./resources/katana.obj")?;

        // --------------------
        // Textures
        // --------------------

        let mut texture_samourai = read_tga_r8g8b8a8_srgb_file(engine, "./textures/samourai.tga")?;

        let mut texture_katana = read_tga_r8g8b8a8_srgb_file(engine, "./textures/katana.tga")?;

        // --------------------
        // Shaders
        // --------------------

        let vert_shader = read_vert_spv_file(engine, "./shaders/default.vert.spv")?;

        let frag_shader = read_frag_spv_file(engine, "./shaders/default.frag.spv")?;

        // --------------------
        // Materials
        // --------------------

        let set_layouts = vec![
            ScopDescriptorSetLayout::builder(&engine.renderer.main_device)
                .add_texture_binding(0, vk::ShaderStageFlags::FRAGMENT)
                .build()?,
        ];

        let material = Material::new(&engine.renderer, set_layouts, &vert_shader, &frag_shader)?;

        // --------------------
        // Material instances
        // --------------------

        let material_instance_samourai =
            MaterialInstance::instanciate(&engine.renderer, material.clone())?;
        material_instance_samourai
            .writer(0)
            .set_texture2d(0, &texture_samourai)
            .write();

        let material_instance_katana =
            MaterialInstance::instanciate(&engine.renderer, material.clone())?;
        material_instance_katana
            .writer(0)
            .set_texture2d(0, &texture_katana)
            .write();

        // --------------------
        // GameObjects
        // --------------------

        GameObject::builder(engine)
            .name("Samourai")
            .mesh(mesh_samourai.clone())
            .material(material_instance_samourai.clone())
            .build();

        GameObject::builder(engine)
            .name("Socle Samourai")
            .mesh(mesh_socle.clone())
            .material(material_instance_samourai.clone())
            .build();

        GameObject::builder(engine)
            .name("Katana")
            .mesh(mesh_katana.clone())
            .material(material_instance_katana.clone())
            .build();

        // --------------------
        // Logic
        // --------------------

        let mut camera = Camera::empty();
        let aspect = engine.renderer.window.window.inner_size().width as f32
            / engine.renderer.window.window.inner_size().height as f32;
        camera.set_perspective_projection(60.0, aspect, 0.0, 100.0);
        camera.set_view_direction([0.0, 10.0, 25.0].into(), Vec3::backward(), Vec3::up());
        
        engine.run(&camera, |engine, input, image_index| {
            let mut movement = Vec3::default();
            if input.key_held_logical(Key::Named(NamedKey::ArrowLeft)) {
                self.current_yaw -= 0.02;
            }
            if input.key_held_logical(Key::Named(NamedKey::ArrowRight)) {
                self.current_yaw += 0.02;
            }
            if input.key_held(KeyCode::KeyA) {
                movement.x -= 0.084;
            }
            if input.key_held(KeyCode::KeyD) {
                movement.x += 0.084;
            }
            if input.key_held(KeyCode::KeyW) {
                movement.z -= 0.084;
            }
            if input.key_held(KeyCode::KeyS) {
                movement.z += 0.084;
            }
            if input.key_held(KeyCode::KeyQ) {
                movement.y -= 0.084;
            }
            if input.key_held(KeyCode::KeyE) {
                movement.y += 0.084;
            }

            if input.key_pressed_logical(Key::Character(&"t")) {
                self.texture_target_fade = if self.texture_target_fade == 1. {
                    0.
                } else {
                    1.
                };
                self.texture_change_frame = engine.renderer.frame_count;
            }

            if engine.renderer.flat_texture_interpolation < self.texture_target_fade {
                engine.renderer.flat_texture_interpolation =
                    (engine.renderer.flat_texture_interpolation + 0.016).clamp(0., 1.);
            } else if engine.renderer.flat_texture_interpolation > self.texture_target_fade {
                engine.renderer.flat_texture_interpolation =
                    (engine.renderer.flat_texture_interpolation - 0.016).clamp(0., 1.);
            }

            engine.game_objects.values_mut().for_each(|e| {
                e.borrow_mut().transform.rotation =
                    [self.current_pitch, self.current_yaw, self.current_roll].into();
                e.borrow_mut().transform.translation += movement;
            });
        })?;

        engine.renderer.wait_gpu();

        texture_samourai.cleanup();
        texture_katana.cleanup();

        engine.game_objects.clear();

        Ok(())
    }
}
