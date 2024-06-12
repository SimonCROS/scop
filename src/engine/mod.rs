pub mod camera;
mod game_object;
pub mod mesh;
mod transform;

use std::{cell::RefCell, collections::HashMap, rc::Rc};

use anyhow::Result;
use ash::vk;
use camera::Camera;
pub use game_object::*;
use math::Vec3;
pub use transform::*;
use winit_input_helper::WinitInputHelper;

use crate::{
    parsing::{read_obj_file, read_tga_r8g8b8a8_srgb_file},
    renderer::{Material, MaterialInstance, Renderer, RendererWindow, ScopDescriptorSetLayout},
};

pub type GameObjectId = u32;

pub struct Engine {
    last_used_id: GameObjectId,
    pub game_objects: HashMap<GameObjectId, Rc<RefCell<GameObject>>>,
    pub renderer: Renderer,
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

    pub fn run<F: FnMut(&mut Engine, &WinitInputHelper, u32)>(
        &mut self,
        mut on_update: F,
    ) -> Result<()> {
        let mut camera = Camera::empty();
        let aspect = self.renderer.window.window.inner_size().width as f32
            / self.renderer.window.window.inner_size().height as f32;
        camera.set_perspective_projection(60.0, aspect, 0.0, 100.0);
        // camera.set_view_target([20.0, 0.0, 0.0].into(), Vector3::zero(), Vector3::up());
        camera.set_view_target([0.0, 0.0, -20.0].into(), Vec3::default(), Vec3::up());

        let event_loop = self.renderer.window.acquire_event_loop()?;
        RendererWindow::run(event_loop, |input| {
            let next_frame_infos = self.renderer.handle_draw_request()?;

            if let Some((image_index, image_available, rendering_finished, may_begin_drawing)) =
                next_frame_infos
            {
                on_update(self, input, image_index);

                self.renderer.draw(
                    &camera,
                    &self.game_objects,
                    image_index,
                    image_available,
                    rendering_finished,
                    may_begin_drawing,
                )?;
            }
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
