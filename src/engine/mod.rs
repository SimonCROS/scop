pub mod camera;
mod components;
mod game_object;
mod shared_resources;
mod transform;

use std::{cell::RefCell, collections::HashMap, rc::Rc, sync::RwLock};

use anyhow::Result;
use ash::vk;
use camera::Camera;
pub use components::*;
pub use game_object::*;
use math::Vec3;
pub use shared_resources::{
    GameObjectId, MaterialId, MaterialInstanceId, ResourcesAccessor, ResourcesAccessorMut,
    SharedResources,
};
pub use transform::*;
use winit_input_helper::WinitInputHelper;

use crate::{
    parsing::{read_obj_file, read_tga_r8g8b8a8_srgb_file},
    renderer::{Renderer, RendererWindow, ScopDescriptorSetLayout},
};

pub struct FrameInfo<'a> {
    pub input: &'a WinitInputHelper,
    pub frame_count: u32,
    pub image_index: u32,
}

pub struct Engine {
    pub game_objects: HashMap<GameObjectId, Rc<RefCell<GameObject>>>,
    pub renderer: Renderer,
    input: WinitInputHelper,
    resources: SharedResources,
}

impl<'a> Engine {
    pub fn new() -> Result<Self> {
        Ok(Engine {
            input: WinitInputHelper::new(),
            renderer: Renderer::new()?,
            game_objects: HashMap::new(),
            resources: SharedResources::default(),
        })
    }

    pub fn run(&mut self) -> Result<()> {
        let mut camera = Camera::empty();
        let aspect = self.renderer.window.window.inner_size().width as f32
            / self.renderer.window.window.inner_size().height as f32;
        camera.set_perspective_projection(60f32.to_radians(), aspect, 0.0, 100.0);
        // camera.set_view_target([20.0, 0.0, 0.0].into(), Vector3::zero(), Vector3::up());
        camera.set_view_target([0.0, 0.0, -20.0].into(), Vec3::default(), Vec3::up());

        let event_loop = self.renderer.window.acquire_event_loop()?;

        RendererWindow::run(event_loop, &mut self.input, |input| {
            self.renderer.handle_draw_request(
                &camera,
                &self.resources_accessor_mut(),
                |renderer, resources, image_index| {
                    let frame_info = FrameInfo {
                        input,
                        frame_count: renderer.frame_count,
                        image_index,
                    };

                    for game_object in self.game_objects.values() {
                        game_object.borrow_mut().update(&frame_info);
                    }
                },
            )?;
            Ok(())
        })?;

        Ok(())
    }

    pub fn resources_accessor(&'a self) -> ResourcesAccessor<'a> {
        ResourcesAccessor::from(&self.resources)
    }

    pub fn resources_accessor_mut(&'a mut self) -> ResourcesAccessorMut<'a> {
        ResourcesAccessorMut::from(&mut self.resources)
    }
}

// impl Drop for Engine {
//     fn drop(&mut self) {
//         self.renderer.wait_gpu();
//     }
// }
