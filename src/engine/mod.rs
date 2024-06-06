pub mod camera;
mod components;
mod game_object;
mod material_bank;
pub mod mesh;
mod transform;

use std::{cell::RefCell, collections::HashMap, rc::Rc};

use anyhow::Result;
use ash::vk;
use camera::Camera;
pub use components::*;
pub use game_object::*;
use matrix::traits::Zero;
pub use transform::*;

use crate::{
    math::{Up, Vector3},
    parsing::read_obj_file,
    renderer::{Material, Renderer, RendererWindow, ScopDescriptorSetLayout, ScopTexture2D},
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
            "./resources/teapot2.obj",
        )?);

        let texture = ScopTexture2D::from_tga_r8g8b8a8_file(
            self.renderer.main_device.clone(),
            &self.renderer.graphic_command_pools[0],
            "./textures/earth.tga",
        )?;

        let set_layouts = vec![ScopDescriptorSetLayout::builder(&self.renderer.main_device)
            .add_binding(
                0,
                vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                vk::ShaderStageFlags::FRAGMENT,
            )
            .build()?];

        let material = Rc::new(Material::new(&self.renderer, set_layouts)?);
        material
            .get_writer_for_all(0)
            .set_texture2d(0, &texture)
            .write();

        GameObject::builder(self)
            .name("Hello World")
            .mesh(mesh.clone())
            .material(material)
            .build();

        let mut camera = Camera::empty();
        camera.set_perspective_projection(60.0, 1.0, 0.0, 100.0);
        camera.set_view_target([0.0, 0.0, -8.0].into(), Vector3::zero(), Vector3::up());

        let event_loop = self.renderer.window.acquire_event_loop()?;
        RendererWindow::run(event_loop, || {
            self.renderer
                .handle_draw_request(&camera, &self.game_objects)?;
            let yaw =
                (std::f32::consts::PI * 2f32 / 542f32) * (self.renderer.frame_count % 542) as f32;
            let roll =
                (std::f32::consts::PI * 2f32 / 1000f32) * (self.renderer.frame_count % 1000) as f32;
            // let roll = 0f32;
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
