pub mod camera;
mod game_object;
pub mod mesh;
mod transform;

use std::{cell::RefCell, collections::HashMap, rc::Rc};

use anyhow::Result;
use camera::Camera;
pub use game_object::*;
pub use transform::*;
use winit_input_helper::WinitInputHelper;

use crate::renderer::{Renderer, RendererWindow};

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
        camera: &Camera,
        mut on_update: F,
    ) -> Result<()> {
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
