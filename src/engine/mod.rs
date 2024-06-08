pub mod camera;
mod components;
mod game_object;
pub mod mesh;
mod transform;

use std::{cell::RefCell, collections::HashMap, rc::Rc, sync::RwLock};

use anyhow::Result;
use ash::vk;
use camera::Camera;
pub use components::*;
pub use game_object::*;
use math::Vec3;
pub use transform::*;
use winit_input_helper::WinitInputHelper;

use crate::{
    parsing::{read_obj_file, read_tga_r8g8b8a8_srgb_file},
    renderer::{Material, MaterialInstance, Renderer, RendererWindow, ScopDescriptorSetLayout},
};

pub type GameObjectId = u32;

pub struct FrameInfo<'a> {
    pub input: &'a WinitInputHelper,
    pub frame_count: u32,
    pub image_index: u32,
}

pub struct Engine {
    last_used_id: GameObjectId,
    pub game_objects: HashMap<GameObjectId, Rc<RefCell<GameObject>>>,
    input: WinitInputHelper,
    renderer: Renderer,
}

impl<'a> Engine {
    pub fn new() -> Result<Self> {
        Ok(Engine {
            last_used_id: 0,
            input: WinitInputHelper::new(),
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
        {
            let renderer = &self.renderer;

            let mesh_teapot = Rc::new(read_obj_file(
                renderer.main_device.clone(),
                "./resources/teapot2.obj",
            )?);

            let mesh_42 = Rc::new(read_obj_file(
                renderer.main_device.clone(),
                "./resources/42.obj",
            )?);

            let texture_earth = read_tga_r8g8b8a8_srgb_file(
                renderer.main_device.clone(),
                &renderer.graphic_command_pools[0],
                "./textures/earth.tga",
            )?;

            let texture_mars = read_tga_r8g8b8a8_srgb_file(
                renderer.main_device.clone(),
                &renderer.graphic_command_pools[0],
                "./textures/mars.tga",
            )?;

            let texture_ponies = read_tga_r8g8b8a8_srgb_file(
                renderer.main_device.clone(),
                &renderer.graphic_command_pools[0],
                "./textures/ponies.tga",
            )?;

            let set_layouts = vec![ScopDescriptorSetLayout::builder(&renderer.main_device)
                .add_binding(
                    0,
                    vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                    vk::ShaderStageFlags::FRAGMENT,
                )
                .build()?];

            let material = Material::new(&renderer, set_layouts)?;

            let material_instance_earth =
                MaterialInstance::instanciate(&renderer, material.clone())?;
            material_instance_earth
                .get_writer_for_all(0)
                .set_texture2d(0, &texture_earth)
                .write();

            let material_instance_ponies =
                MaterialInstance::instanciate(&renderer, material.clone())?;
            material_instance_ponies
                .get_writer_for_all(0)
                .set_texture2d(0, &texture_ponies)
                .write();

            let material_instance_mars =
                MaterialInstance::instanciate(&renderer, material.clone())?;
            material_instance_mars
                .get_writer_for_all(0)
                .set_texture2d(0, &texture_mars)
                .write();

            let shared_42 = Transform {
                pivot: mesh_42.bounding_box.get_middle_point(),
                scale: Vec3::one() * 2.,
                ..Default::default()
            };
        }

        // {
        //     let go = GameObject::builder(&mut self)
        //         .name("Hello World")
        //         .mesh(mesh_42.clone())
        //         .material(material_instance_mars.clone())
        //         .transform(shared_42)
        //         .build();
        //     go.borrow_mut().transform.translation = Vec3::from([7., 7., 0.]);

        //     let go = GameObject::builder(&mut self)
        //         .name("Hello World")
        //         .mesh(mesh_teapot.clone())
        //         .material(material_instance_mars.clone())
        //         .build();
        //     go.borrow_mut().transform.translation = Vec3::right() * 7.;

        //     let go = GameObject::builder(&mut self)
        //         .name("Hello World")
        //         .mesh(mesh_42.clone())
        //         .material(material_instance_mars.clone())
        //         .transform(shared_42)
        //         .build();
        //     go.borrow_mut().transform.translation = Vec3::from([7., -7., 0.]);
        // }

        // {
        //     let go = GameObject::builder(&mut self)
        //         .name("Hello World")
        //         .mesh(mesh_teapot.clone())
        //         .material(material_instance_ponies.clone())
        //         .build();
        //     go.borrow_mut().transform.translation = Vec3::from([0., 7., 0.]);

        //     GameObject::builder(&mut self)
        //         .name("Hello World")
        //         .mesh(mesh_42.clone())
        //         .material(material_instance_ponies.clone())
        //         .transform(shared_42)
        //         .build();

        //     let go = GameObject::builder(&mut self)
        //         .name("Hello World")
        //         .mesh(mesh_teapot.clone())
        //         .material(material_instance_ponies.clone())
        //         .build();
        //     go.borrow_mut().transform.translation = Vec3::from([0., -7., 0.]);
        // }

        // {
        //     let go = GameObject::builder(&mut self)
        //         .name("Hello World")
        //         .mesh(mesh_42.clone())
        //         .material(material_instance_earth.clone())
        //         .transform(shared_42)
        //         .build();
        //     go.borrow_mut().transform.translation = Vec3::from([-7., 7., 0.]);

        //     let go = GameObject::builder(&mut self)
        //         .name("Hello World")
        //         .mesh(mesh_teapot.clone())
        //         .material(material_instance_earth.clone())
        //         .build();
        //     go.borrow_mut().transform.translation = Vec3::left() * 7.;

        //     let go = GameObject::builder(&mut self)
        //         .name("Hello World")
        //         .mesh(mesh_42.clone())
        //         .material(material_instance_earth.clone())
        //         .transform(shared_42)
        //         .build();
        //     go.borrow_mut().transform.translation = Vec3::from([-7., -7., 0.]);
        // }

        let mut camera = Camera::empty();
        let aspect = self.renderer.window.window.inner_size().width as f32
            / self.renderer.window.window.inner_size().height as f32;
        camera.set_perspective_projection(60f32.to_radians(), aspect, 0.0, 100.0);
        // camera.set_view_target([20.0, 0.0, 0.0].into(), Vector3::zero(), Vector3::up());
        camera.set_view_target([0.0, 0.0, -20.0].into(), Vec3::default(), Vec3::up());

        let event_loop = self.renderer.window.acquire_event_loop()?;

        let on_update = |image_index| {
            let frame_info = FrameInfo {
                input: &self.input,
                frame_count: self.renderer.frame_count,
                image_index,
            };

            for game_object in self.game_objects.values() {
                game_object.borrow_mut().update(&frame_info);
            }
        };

        RendererWindow::run(event_loop, &mut self.input, || {
            self.renderer.handle_draw_request(&camera, &self.game_objects, on_update)?;
            Ok(())
        })?;

        Ok(())
    }
}

// impl Drop for Engine {
//     fn drop(&mut self) {
//         self.renderer.wait_gpu();
//     }
// }
