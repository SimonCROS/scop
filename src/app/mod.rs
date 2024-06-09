use anyhow::Result;
use math::Vec3;

use crate::{
    engine::{Engine, GameObject, Transform},
    parsing::{read_frag_spv_file, read_obj_file, read_tga_r8g8b8a8_srgb_file, read_vert_spv_file},
    renderer::{MaterialId, ScopDescriptorSetLayout},
};

pub struct App {}

impl App {
    pub fn main(engine: &Engine) -> Result<()> {
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
                .add_texture_binding(0)
                .build()?,
        ];

        let material = MaterialId::new(&engine.renderer, set_layouts, &vert_shader, &frag_shader)?;

        // --------------------
        // Material instances
        // --------------------

        let material_instance_earth = material.instanciate(&engine.renderer)?;
        material_instance_earth
            .writer(0)
            .set_texture2d(0, &texture_earth)
            .write();

        let material_instance_ponies = material.instanciate(&engine.renderer)?;
        material_instance_ponies
            .writer(0)
            .set_texture2d(0, &texture_ponies)
            .write();

        let material_instance_mars = material.instanciate(&engine.renderer)?;
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

        Ok(())
    }
}
