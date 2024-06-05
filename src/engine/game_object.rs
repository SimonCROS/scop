use std::{cell::RefCell, collections::HashSet, rc::Rc};

use crate::renderer::Material;

use super::{mesh::Mesh, Component, Engine, Transform};

pub struct GameObject {
    pub name: Option<String>,
    pub transform: Transform,
    pub components: HashSet<Box<dyn Component>>,
    pub mesh: Option<Rc<Mesh>>,
    pub material: Option<Rc<Material>>,
}

pub struct GameObjectBuilder<'a> {
    engine: &'a mut Engine,
    name: Option<&'a str>,
    transform: Option<Transform>,
    components: HashSet<Box<dyn Component>>,
    mesh: Option<Rc<Mesh>>,
    material: Option<Rc<Material>>,
}

impl GameObject {
    pub fn builder<'a>(engine: &'a mut Engine) -> GameObjectBuilder<'a> {
        GameObjectBuilder {
            engine,
            name: None,
            transform: None,
            components: HashSet::new(),
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

    pub fn material(mut self, material: Rc<Material>) -> Self {
        self.material = Some(material);
        self
    }

    pub fn build(self) -> Rc<RefCell<GameObject>> {
        self.engine.register(GameObject {
            name: self.name.map(|s| s.to_string()),
            transform: self.transform.unwrap_or(Transform::default()),
            components: self.components,
            mesh: self.mesh,
            material: self.material,
        })
    }
}
