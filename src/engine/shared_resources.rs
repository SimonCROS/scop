use std::{
    collections::HashMap,
    hash::{BuildHasherDefault, DefaultHasher},
};

use crate::{
    engine::GameObject,
    renderer::{Material, MaterialInstance},
};

type DeterministicHashMap<K, V> = HashMap<K, V, BuildHasherDefault<DefaultHasher>>;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub struct GameObjectId(pub u64);

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub struct MaterialId(pub u64);

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub struct MaterialInstanceId(pub u64);

#[derive(Default)]
pub struct SharedResources {
    pub last_id: u64,
    pub materials: DeterministicHashMap<MaterialId, Material>,
    pub material_instances: DeterministicHashMap<MaterialInstanceId, MaterialInstance>,
    pub game_objects: DeterministicHashMap<GameObjectId, GameObject>,
}

pub struct ResourcesAccessor<'a>(&'a SharedResources);

pub struct ResourcesAccessorMut<'a>(&'a mut SharedResources);

impl<'a> ResourcesAccessor<'a> {
    pub fn get_materials(&self) -> &DeterministicHashMap<MaterialId, Material> {
        &self.0.materials
    }

    pub fn get_material_instances(&self) -> &DeterministicHashMap<MaterialInstanceId, MaterialInstance> {
        &self.0.material_instances
    }

    pub fn get_game_objects(&self) -> &DeterministicHashMap<GameObjectId, GameObject> {
        &self.0.game_objects
    }

    pub fn get_material(&self, id: MaterialId) -> Option<&Material> {
        self.0.materials.get(&id)
    }

    pub fn get_material_instance(&self, id: MaterialInstanceId) -> Option<&MaterialInstance> {
        self.0.material_instances.get(&id)
    }

    pub fn get_game_object(&self, id: GameObjectId) -> Option<&GameObject> {
        self.0.game_objects.get(&id)
    }
}

impl<'a> ResourcesAccessorMut<'a> {
    pub fn get_materials_mut(&mut self) -> &mut DeterministicHashMap<MaterialId, Material> {
        &mut self.0.materials
    }

    pub fn get_material_instances_mut(&mut self) -> &mut DeterministicHashMap<MaterialInstanceId, MaterialInstance> {
        &mut self.0.material_instances
    }

    pub fn get_game_objects_mut(&mut self) -> &mut DeterministicHashMap<GameObjectId, GameObject> {
        &mut self.0.game_objects
    }

    pub fn get_materials(&self) -> &DeterministicHashMap<MaterialId, Material> {
        &self.0.materials
    }

    pub fn get_material_instances(&self) -> &DeterministicHashMap<MaterialInstanceId, MaterialInstance> {
        &self.0.material_instances
    }

    pub fn get_game_objects(&self) -> &DeterministicHashMap<GameObjectId, GameObject> {
        &self.0.game_objects
    }

    pub fn get_material(&self, id: MaterialId) -> Option<&Material> {
        self.0.materials.get(&id)
    }

    pub fn get_material_instance(&self, id: MaterialInstanceId) -> Option<&MaterialInstance> {
        self.0.material_instances.get(&id)
    }

    pub fn get_game_object(&self, id: GameObjectId) -> Option<&GameObject> {
        self.0.game_objects.get(&id)
    }

    pub fn get_material_unchecked(&self, id: MaterialId) -> &Material {
        &self.0.materials[&id]
    }

    pub fn get_material_instance_unchecked(&self, id: MaterialInstanceId) -> &MaterialInstance {
        &self.0.material_instances[&id]
    }

    pub fn get_game_object_unchecked(&self, id: GameObjectId) -> &GameObject {
        &self.0.game_objects[&id]
    }

    pub fn create_material(&mut self, material: Material) -> MaterialId {
        let id = MaterialId(self.next_id());
        self.0.materials.insert(id, material);
        id
    }

    pub fn create_material_instance(
        &mut self,
        material_instance: MaterialInstance,
    ) -> MaterialInstanceId {
        let id = MaterialInstanceId(self.next_id());
        self.0.material_instances.insert(id, material_instance);
        id
    }

    pub fn create_game_object(&mut self, game_object: GameObject) -> GameObjectId {
        let id = GameObjectId(self.next_id());
        self.0.game_objects.insert(id, game_object);
        id
    }

    // pub fn create_texture(&mut self, content: ScopTexture2D) -> MaterialInstance {
    //     let material_instance = MaterialInstance(self.next_id());
    //     self.0.material_instances.insert(material_instance, content);
    //     material_instance
    // }

    fn next_id(&mut self) -> u64 {
        self.0.last_id += 1;
        self.0.last_id
    }
}

impl<'a> Into<ResourcesAccessor<'a>> for ResourcesAccessorMut<'a> {
    fn into(self) -> ResourcesAccessor<'a> {
        ResourcesAccessor(self.0)
    }
}

impl<'a> Into<ResourcesAccessor<'a>> for &'a ResourcesAccessorMut<'a> {
    fn into(self) -> ResourcesAccessor<'a> {
        ResourcesAccessor(self.0)
    }
}

impl<'a> From<&'a SharedResources> for ResourcesAccessor<'a> {
    fn from(value: &'a SharedResources) -> Self {
        Self(value)
    }
}

impl<'a> From<&'a mut SharedResources> for ResourcesAccessorMut<'a> {
    fn from(value: &'a mut SharedResources) -> Self {
        Self(value)
    }
}
