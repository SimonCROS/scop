use std::collections::HashMap;

use super::material_loader::MaterialLoader;

pub struct MaterialFile (pub HashMap<String, MaterialLoader>);

pub struct MaterialBank (pub HashMap<String, MaterialFile>);
