pub mod camera;
mod components;
mod game_object;
pub mod material;
pub mod mesh;
mod transform;

use std::{cell::RefCell, collections::HashMap, mem::ManuallyDrop, rc::Rc};

use anyhow::Result;
use camera::Camera;
pub use components::*;
pub use game_object::*;
use matrix::traits::{One, Zero};
pub use transform::*;

use crate::{math::{Up, Vector2, Vector3}, renderer::{window::RendererWindow, Renderer}};

use self::mesh::{Mesh, Vertex};

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
        let vertices = vec![
            Vertex {
                position: [-0.2, 0.2, -0.2].into(),
                color: [0.0, 0.0, 1.0].into(),
                normal: Vector3::up(),
                uv: Vector2::zero(),
            },
            Vertex {
                position: [0.2, 0.2, -0.2].into(),
                color: [0.0, 1.0, 0.0].into(),
                normal: Vector3::up(),
                uv: Vector2::zero(),
            },
            Vertex {
                position: [0.2, 0.2, 0.2].into(),
                color: [1.0, 0.0, 0.0].into(),
                normal: Vector3::up(),
                uv: Vector2::zero(),
            },
            Vertex {
                position: [-0.2, 0.2, 0.2].into(),
                color: [0.0, 0.0, 0.0].into(),
                normal: Vector3::up(),
                uv: Vector2::zero(),
            },
            Vertex {
                position: [0.0, -0.2, 0.0].into(),
                color: [1.0, 1.0, 1.0].into(),
                normal: Vector3::up(),
                uv: Vector2::zero(),
            },
        ];

        let indices = vec![0, 1, 2, 0, 3, 2, 0, 1, 4, 1, 2, 4, 2, 3, 4, 3, 0, 4];

        let m = Rc::new(Mesh::builder(self.renderer.main_device.clone())
            .vertices(&vertices)
            .indices(&indices)
            .build()?);

        let mut camera = Camera::empty();
        camera.set_perspective_projection(60.0, 16f32 / 9f32, 0.0, 100.0);

        let xs: [f32; 40] = [0.02, 0.59, 0.88, 0.43, 0.01, 0.21, 0.10, 0.98, 0.62, 0.15, 0.12, 0.42, 0.61, 0.41, 0.22, 0.58, 0.99, 0.33, 0.79, 0.90, 0.39, 0.48, 0.31, 0.28, 0.80, 0.51, 0.30, 0.07, 0.53, 0.35, 0.32, 0.27, 0.60, 0.73, 0.49, 0.91, 0.25, 0.45, 0.40, 0.92];
        let ys: [f32; 40] = [0.50, 0.95, 0.68, 0.55, 0.00, 0.82, 0.13, 0.85, 0.40, 0.60, 0.25, 0.09, 0.89, 0.05, 0.27, 0.22, 0.20, 0.04, 0.44, 0.74, 0.14, 0.65, 0.03, 0.48, 0.07, 0.45, 0.62, 0.23, 0.99, 0.24, 0.17, 0.00, 0.08, 0.54, 0.80, 0.98, 0.59, 0.83, 0.96, 0.61];
        for i in 0..40 {
            GameObject::builder(self)
                .name("Hello World")
                .mesh(m.clone())
                .transform(Transform { translation: [xs[i] * 2. - 1., ys[i] * 2. - 1., 0.6].into(), scale: Vector3::one(), rotation: Vector3::zero()})
                .build();
        }

        let event_loop = self.renderer.window.acquire_event_loop()?;
        RendererWindow::run(event_loop, || {
            self.renderer.handle_draw_request(&camera, &self.game_objects)?;
            let yaw = (std::f32::consts::PI * 2f32 / 542f32) * (self.renderer.frame_count % 542) as f32;
            let roll = (std::f32::consts::PI * 2f32 / 1000f32) * (self.renderer.frame_count % 1000) as f32;
            // let roll = 0f32;
            // self.game_objects.values_mut().for_each(|e| {
            //     e.borrow_mut().transform.rotation = [0., yaw, roll].into();
            // });
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
