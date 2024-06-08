use crate::Vec3;

#[derive(Copy, Clone, Default, Debug, PartialEq)]
pub struct BoundingBox {
    pub min: Vec3,
    pub max: Vec3,
}

impl BoundingBox {
    pub fn get_middle_point(&self) -> Vec3 {
        self.min + (self.max - self.min) / 2.
    }
}
