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
pub struct AppObjects {
    last_frame_move: u32,
    texture_target_fade: f32,
    texture_change_frame: u32,
}

impl AppObjects {
    pub fn start(&mut self, engine: &mut Engine) -> Result<()> {
        // --------------------
        // Meshs
        // --------------------

        let mesh_sphere = read_obj_file(engine, "./resources/sphere.obj")?;

        let mesh_42 = read_obj_file(engine, "./resources/42.obj")?;

        let mesh_teapot_1 = read_obj_file(engine, "./resources/teapot.obj")?;

        let mesh_teapot_2 = read_obj_file(engine, "./resources/teapot2.obj")?;

        // --------------------
        // Textures
        // --------------------

        let mut texture_earth = read_tga_r8g8b8a8_srgb_file(engine, "./textures/earth.tga")?;

        let mut texture_mars = read_tga_r8g8b8a8_srgb_file(engine, "./textures/mars.tga")?;

        let mut texture_ponies = read_tga_r8g8b8a8_srgb_file(engine, "./textures/ponies.tga")?;

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

        let material_instance_earth =
            MaterialInstance::instanciate(&engine.renderer, material.clone())?;
        material_instance_earth
            .writer(0)
            .set_texture2d(0, &texture_earth)
            .write();

        let material_instance_ponies =
            MaterialInstance::instanciate(&engine.renderer, material.clone())?;
        material_instance_ponies
            .writer(0)
            .set_texture2d(0, &texture_ponies)
            .write();

        let material_instance_mars =
            MaterialInstance::instanciate(&engine.renderer, material.clone())?;
        material_instance_mars
            .writer(0)
            .set_texture2d(0, &texture_mars)
            .write();

        // --------------------
        // GameObjects
        // --------------------

        let go = GameObject::builder(engine)
            .name("Earth")
            .mesh(mesh_sphere.clone())
            .material(material_instance_earth.clone())
            .transform(Transform {
                scale: Vec3::one() * 2.,
                ..Default::default()
            })
            .build();
        go.borrow_mut().transform.translation = Vec3::from([7., -7., 0.]);

        let go = GameObject::builder(engine)
            .name("Mars")
            .mesh(mesh_sphere.clone())
            .material(material_instance_mars.clone())
            .transform(Transform {
                scale: Vec3::one() * 1.5,
                ..Default::default()
            })
            .build();
        go.borrow_mut().transform.translation = Vec3::from([-7., -7., 0.]);

        let go = GameObject::builder(engine)
            .name("42")
            .mesh(mesh_42.clone())
            .material(material_instance_ponies.clone())
            .transform(Transform {
                pivot: mesh_42.bounding_box.get_middle_point(),
                scale: Vec3::one() * 2.5,
                rotation: Vec3::up() * std::f32::consts::PI / 2.,
                ..Default::default()
            })
            .build();
        go.borrow_mut().transform.translation = Vec3::from([0., 0., 0.]);

        let go = GameObject::builder(engine)
            .name("Teapot 1")
            .mesh(mesh_teapot_1.clone())
            .material(material_instance_ponies.clone())
            .transform(Transform {
                pivot: mesh_teapot_1.bounding_box.get_middle_point(),
                ..Default::default()
            })
            .build();
        go.borrow_mut().transform.translation = Vec3::from([7., 7., 0.]);

        let go = GameObject::builder(engine)
            .name("Teapot 2")
            .mesh(mesh_teapot_2.clone())
            .material(material_instance_ponies.clone())
            .transform(Transform {
                pivot: mesh_teapot_2.bounding_box.get_middle_point(),
                ..Default::default()
            })
            .build();
        go.borrow_mut().transform.translation = Vec3::from([-7., 7., 0.]);

        // --------------------
        // Logic
        // --------------------

        let mut camera = Camera::empty();
        let aspect = engine.renderer.window.window.inner_size().width as f32
            / engine.renderer.window.window.inner_size().height as f32;
        camera.set_perspective_projection(60.0, aspect, 1.0, 100.0);
        camera.set_view_target([0.0, 0.0, -20.0].into(), Vec3::default(), Vec3::up());
        
        engine.run(&camera, |engine, input, _image_index| {
            let mut movement = Vec3::default();
            let mut rotation = Vec3::default();
            if input.key_held_logical(Key::Named(NamedKey::ArrowLeft)) {
                rotation.y -= 0.02;
                self.last_frame_move = engine.renderer.frame_count;
            }
            if input.key_held_logical(Key::Named(NamedKey::ArrowRight)) {
                rotation.y += 0.02;
                self.last_frame_move = engine.renderer.frame_count;
            }
            if input.key_held_logical(Key::Named(NamedKey::ArrowUp)) {
                rotation.z += 0.02;
                self.last_frame_move = engine.renderer.frame_count;
            }
            if input.key_held_logical(Key::Named(NamedKey::ArrowDown)) {
                rotation.z -= 0.02;
                self.last_frame_move = engine.renderer.frame_count;
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

            if self.last_frame_move == 0 || engine.renderer.frame_count - self.last_frame_move > 200
            {
                rotation.y += 0.02;
            }

            if engine.renderer.flat_texture_interpolation < self.texture_target_fade {
                engine.renderer.flat_texture_interpolation =
                    (engine.renderer.flat_texture_interpolation + 0.016).clamp(0., 1.);
            } else if engine.renderer.flat_texture_interpolation > self.texture_target_fade {
                engine.renderer.flat_texture_interpolation =
                    (engine.renderer.flat_texture_interpolation - 0.016).clamp(0., 1.);
            }

            engine.game_objects.values_mut().for_each(|e| {
                e.borrow_mut().transform.rotation += rotation;
                e.borrow_mut().transform.translation += movement;
            });
        })?;

        engine.renderer.wait_gpu();

        texture_earth.cleanup();
        texture_mars.cleanup();
        texture_ponies.cleanup();

        engine.game_objects.clear();

        Ok(())
    }
}
