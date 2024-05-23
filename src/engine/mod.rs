pub mod camera;
mod components;
mod game_object;
pub mod material;
pub mod mesh;
mod transform;

use std::{cell::RefCell, collections::HashMap, rc::Rc};

use anyhow::Result;
pub use components::*;
pub use game_object::*;
use matrix::traits::{One, Zero};
pub use transform::*;

use crate::{math::{Up, Vector2, Vector3}, renderer::{window::RendererWindow, Renderer}};

use self::mesh::{Mesh, Vertex};

pub struct Engine {
    last_go_id: u32,
    renderer: Renderer,
    pub game_objects: HashMap<u32, Rc<RefCell<GameObject>>>,
}

impl Engine {
    pub fn new() -> Result<Self> {
        Ok(Engine {
            last_go_id: 0,
            renderer: Renderer::new()?,
            game_objects: HashMap::new(),
        })
    }

    pub fn register(&mut self, game_object: GameObject) -> Rc<RefCell<GameObject>> {
        self.last_go_id += 1;

        let id = self.last_go_id;
        let go: Rc<RefCell<GameObject>> = Rc::new(RefCell::new(game_object));
        self.game_objects.insert(id, go.clone());
        go
    }

    pub fn run(&mut self) -> Result<()> {
        let vertices = vec![
            Vertex {
                position: [-0.2, 0.2, 0.0].into(),
                color: [0.0, 0.0, 1.0].into(),
                normal: Vector3::up(),
                uv: Vector2::zero(),
            },
            Vertex {
                position: [0.2, 0.2, 0.0].into(),
                color: [0.0, 1.0, 0.0].into(),
                normal: Vector3::up(),
                uv: Vector2::zero(),
            },
            Vertex {
                position: [0.0, -0.2, 0.0].into(),
                color: [1.0, 0.0, 0.0].into(),
                normal: Vector3::up(),
                uv: Vector2::zero(),
            },
        ];

        let indices = vec![0, 1, 2];

        let m = Rc::new(Mesh::builder(self.renderer.main_device.clone())
            .vertices(&vertices)
            .indices(&indices)
            .build()?);

        GameObject::builder(self)
            .name("Hello World 1")
            .mesh(m.clone())
            .transform(Transform { translation: [0.2, 0., 0.].into(), scale: Vector3::one(), rotation: Vector3::up() })
            .build();
        GameObject::builder(self)
            .name("Hello World 2")
            .mesh(m.clone())
            .transform(Transform { translation: [-0.8, 0.4, 0.].into(), scale: Vector3::one(), rotation: Vector3::up() })
            .build();
        GameObject::builder(self)
            .name("Hello World 3")
            .mesh(m.clone())
            .build();

        let event_loop = self.renderer.window.acquire_event_loop()?;
        RendererWindow::run(event_loop, || {
            self.renderer.handle_draw_request(&self.game_objects)?;
            Ok(())
        })?;

        Ok(())
    }
}
