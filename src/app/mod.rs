use anyhow::Result;
use ash::vk;
use math::Vec3;
use winit::keyboard::{Key, NamedKey};

use crate::{
    engine::{Engine, GameObject, Transform},
    parsing::{read_frag_spv_file, read_obj_file, read_tga_r8g8b8a8_srgb_file, read_vert_spv_file},
    renderer::{Material, MaterialInstance, ScopDescriptorSetLayout},
};

#[derive(Default)]
pub struct App {
    last_frame_move: u32,
    texture_target_fade: f32,
    texture_change_frame: u32,
    current_pitch: f32,
    current_yaw: f32,
    current_roll: f32,
}

impl App {
    pub fn start(&mut self, engine: &mut Engine) -> Result<()> {
        // --------------------
        // Meshs
        // --------------------

        let mesh_teapot = read_obj_file(engine, "./resources/teapot2.obj")?;

        let mesh_42 = read_obj_file(engine, "./resources/42.obj")?;

        // --------------------
        // Textures
        // --------------------

        let texture_earth = read_tga_r8g8b8a8_srgb_file(engine, "./textures/earth.tga")?;

        let texture_mars = read_tga_r8g8b8a8_srgb_file(engine, "./textures/mars.tga")?;

        let texture_ponies = read_tga_r8g8b8a8_srgb_file(engine, "./textures/ponies.tga")?;

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

        let shared_42_transform = Transform {
            pivot: mesh_42.bounding_box.get_middle_point(),
            scale: Vec3::one() * 2.,
            ..Default::default()
        };

        // Left
        {
            let go = GameObject::builder(engine)
                .name("Hello World")
                .mesh(mesh_42.clone())
                .material(material_instance_mars.clone())
                .transform(shared_42_transform)
                .build();
            go.borrow_mut().transform.translation = Vec3::from([7., 7., 0.]);

            let go = GameObject::builder(engine)
                .name("Hello World")
                .mesh(mesh_teapot.clone())
                .material(material_instance_mars.clone())
                .build();
            go.borrow_mut().transform.translation = Vec3::right() * 7.;

            let go = GameObject::builder(engine)
                .name("Hello World")
                .mesh(mesh_42.clone())
                .material(material_instance_mars.clone())
                .transform(shared_42_transform)
                .build();
            go.borrow_mut().transform.translation = Vec3::from([7., -7., 0.]);
        }

        // Center
        {
            let go = GameObject::builder(engine)
                .name("Hello World")
                .mesh(mesh_teapot.clone())
                .material(material_instance_ponies.clone())
                .build();
            go.borrow_mut().transform.translation = Vec3::from([0., 7., 0.]);

            GameObject::builder(engine)
                .name("Hello World")
                .mesh(mesh_42.clone())
                .material(material_instance_ponies.clone())
                .transform(shared_42_transform)
                .build();

            let go = GameObject::builder(engine)
                .name("Hello World")
                .mesh(mesh_teapot.clone())
                .material(material_instance_ponies.clone())
                .build();
            go.borrow_mut().transform.translation = Vec3::from([0., -7., 0.]);
        }

        // Right
        {
            let go = GameObject::builder(engine)
                .name("Hello World")
                .mesh(mesh_42.clone())
                .material(material_instance_earth.clone())
                .transform(shared_42_transform)
                .build();
            go.borrow_mut().transform.translation = Vec3::from([-7., 7., 0.]);

            let go = GameObject::builder(engine)
                .name("Hello World")
                .mesh(mesh_teapot.clone())
                .material(material_instance_earth.clone())
                .build();
            go.borrow_mut().transform.translation = Vec3::left() * 7.;

            let go = GameObject::builder(engine)
                .name("Hello World")
                .mesh(mesh_42.clone())
                .material(material_instance_earth.clone())
                .transform(shared_42_transform)
                .build();
            go.borrow_mut().transform.translation = Vec3::from([-7., -7., 0.]);
        }

        engine.run(|engine, input, image_index| {
            if input.key_held_logical(Key::Named(NamedKey::ArrowLeft)) {
                self.current_yaw -= 0.02;
                self.last_frame_move = engine.renderer.frame_count;
            }
            if input.key_held_logical(Key::Named(NamedKey::ArrowRight)) {
                self.current_yaw += 0.02;
                self.last_frame_move = engine.renderer.frame_count;
            }
            if input.key_held_logical(Key::Named(NamedKey::ArrowUp)) {
                self.current_roll += 0.02;
                self.last_frame_move = engine.renderer.frame_count;
            }
            if input.key_held_logical(Key::Named(NamedKey::ArrowDown)) {
                self.current_roll -= 0.02;
                self.last_frame_move = engine.renderer.frame_count;
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
                self.current_yaw += 0.02;
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
            });
        })
    }
}
