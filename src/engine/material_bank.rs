use std::collections::HashMap;

use crate::renderer::Material;

pub struct MaterialFile(pub HashMap<String, Material>);

pub struct MaterialBank(pub HashMap<String, MaterialFile>);
