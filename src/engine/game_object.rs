use std::{cell::RefCell, rc::Rc};

use crate::renderer::MaterialInstanceRef;

use super::{mesh::Mesh, Engine, Transform};

pub struct GameObject {
    pub name: Option<String>,
    pub transform: Transform,
    pub mesh: Option<Rc<Mesh>>,
    pub material: Option<MaterialInstanceRef>,
}

pub struct GameObjectBuilder<'a> {
    engine: &'a mut Engine,
    name: Option<&'a str>,
    transform: Option<Transform>,
    mesh: Option<Rc<Mesh>>,
    material: Option<MaterialInstanceRef>,
}

impl GameObject {
    pub fn builder<'a>(engine: &'a mut Engine) -> GameObjectBuilder<'a> {
        GameObjectBuilder {
            engine,
            name: None,
            transform: None,
            mesh: None,
            material: None,
        }
    }
}

impl<'a> GameObjectBuilder<'a> {
    pub fn name(mut self, name: &'a str) -> Self {
        self.name = Some(name);
        self
    }

    pub fn transform(mut self, transform: Transform) -> Self {
        self.transform = Some(transform);
        self
    }

    pub fn mesh(mut self, mesh: Rc<Mesh>) -> Self {
        self.mesh = Some(mesh);
        self
    }

    pub fn material(mut self, material: MaterialInstanceRef) -> Self {
        self.material = Some(material);
        self
    }

    pub fn build(self) -> Rc<RefCell<GameObject>> {
        self.engine.register(GameObject {
            name: self.name.map(|s| s.to_string()),
            transform: self.transform.unwrap_or(Transform::default()),
            mesh: self.mesh,
            material: self.material,
        })
    }
}
