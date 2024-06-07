pub mod camera;
mod components;
mod game_object;
pub mod mesh;
mod transform;

use std::{cell::RefCell, collections::HashMap, rc::Rc};

use anyhow::Result;
use ash::vk;
use camera::Camera;
pub use components::*;
pub use game_object::*;
use matrix::traits::{One, Zero};
pub use transform::*;

use crate::{
    math::{Left, Right, Up, Vector3},
    parsing::{read_obj_file, read_tga_r8g8b8a8_file},
    renderer::{Material, MaterialInstance, Renderer, RendererWindow, ScopDescriptorSetLayout},
};

pub type GameObjectId = u32;

pub struct Engine {
    last_used_id: GameObjectId,
    pub game_objects: HashMap<GameObjectId, Rc<RefCell<GameObject>>>,
    renderer: Renderer,
}

impl Engine {
    pub fn new() -> Result<Self> {
        Ok(Engine {
            last_used_id: 0,
            renderer: Renderer::new()?,
            game_objects: HashMap::new(),
        })
    }

    pub fn register(&mut self, game_object: GameObject) -> Rc<RefCell<GameObject>> {
        self.last_used_id += 1;

        let id = self.last_used_id;
        let go = Rc::new(RefCell::new(game_object));
        self.game_objects.insert(id, go.clone());
        go
    }

    pub fn run(&mut self) -> Result<()> {
        let mesh = Rc::new(read_obj_file(
            self.renderer.main_device.clone(),
            "./resources/42.obj",
        )?);

        let texture_1 = read_tga_r8g8b8a8_file(
            self.renderer.main_device.clone(),
            &self.renderer.graphic_command_pools[0],
            "./textures/earth.tga",
        )?;

        let texture_2 = read_tga_r8g8b8a8_file(
            self.renderer.main_device.clone(),
            &self.renderer.graphic_command_pools[0],
            "./textures/ponies.tga",
        )?;

        let set_layouts = vec![ScopDescriptorSetLayout::builder(&self.renderer.main_device)
            .add_binding(
                0,
                vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                vk::ShaderStageFlags::FRAGMENT,
            )
            .build()?];

        let material = Material::new(&self.renderer, set_layouts)?;

        let material_instance_1 = MaterialInstance::instanciate(&self.renderer, material.clone())?;
        material_instance_1
            .get_writer_for_all(0)
            .set_texture2d(0, &texture_1)
            .write();

        let material_instance_2 = MaterialInstance::instanciate(&self.renderer, material.clone())?;
        material_instance_2
            .get_writer_for_all(0)
            .set_texture2d(0, &texture_2)
            .write();

        let go0 = GameObject::builder(self)
            .name("Hello World")
            .mesh(mesh.clone())
            .material(material_instance_2.clone())
            .build();

        go0.borrow_mut().transform.pivot = mesh.bounding_box.get_middle_point();
        go0.borrow_mut().transform.scale = Vector3::one() * 2.;

        // let go1 = GameObject::builder(self)
        //     .name("Hello World")
        //     .mesh(mesh.clone())
        //     .material(material_instance_1.clone())
        //     .build();
        // go1.borrow_mut().transform.translation = Vector3::left() * 7.;

        // let go2 = GameObject::builder(self)
        //     .name("Hello World")
        //     .mesh(mesh.clone())
        //     .material(material_instance_1.clone())
        //     .build();
        // go2.borrow_mut().transform.translation = Vector3::right() * 7.;

        let mut camera = Camera::empty();
        camera.set_perspective_projection(60.0, 1.0, 0.0, 100.0);
        camera.set_view_target([0.0, 0.0, -10.0].into(), Vector3::zero(), Vector3::up());

        let event_loop = self.renderer.window.acquire_event_loop()?;
        RendererWindow::run(event_loop, || {
            self.renderer
                .handle_draw_request(&camera, &self.game_objects)?;
            let yaw =
                (std::f32::consts::PI * 2f32 / 542f32) * (self.renderer.frame_count % 542) as f32;
            let roll =
                (std::f32::consts::PI * 2f32 / 1000f32) * (self.renderer.frame_count % 1000) as f32;
            let roll = 0f32;
            self.game_objects.values_mut().for_each(|e| {
                e.borrow_mut().transform.rotation = [0., yaw, roll].into();
            });
            Ok(())
        })?;

        Ok(())
    }
}

impl Drop for Engine {
    fn drop(&mut self) {
        self.renderer.wait_gpu();
    }
}
