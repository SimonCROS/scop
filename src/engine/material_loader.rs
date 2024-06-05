use anyhow::Result;

use super::material::Material;

pub struct MaterialLoader {
    outdated: bool,
    material: Option<Material>,
}

impl MaterialLoader {
    pub fn is_outdated() -> bool {
        todo!()
    }

    pub fn get(&self) -> &Option<Material> {
        &self.material
    }

    pub fn get_or_build(&mut self) -> &Result<Material> {
        todo!()
    }

    pub fn rebuild(&mut self) -> &Result<Material> {
        todo!()
    }
}
