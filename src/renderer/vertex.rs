use std::mem::{offset_of, size_of};

use ash::vk::{self, VertexInputAttributeDescription, VertexInputBindingDescription};
use math::{BoundingBox, Vec2, Vec3};

#[derive(Copy, Clone, Default, Debug, PartialEq)]
pub struct Vertex {
    pub position: Vec3,
    pub color: Vec3,
    pub normal: Vec3,
    pub uv: Vec2,
}

impl Vertex {
    pub fn get_bounding_box(vertices: &[Vertex]) -> BoundingBox {
        let mut min = Vec3::from([f32::MAX, f32::MAX, f32::MAX]);
        let mut max = Vec3::from([f32::MIN, f32::MIN, f32::MIN]);

        for vert in vertices {
            for i in 0..3 {
                if vert.position[i] < min[i] {
                    min[i] = vert.position[i];
                }
                if vert.position[i] > max[i] {
                    max[i] = vert.position[i];
                }
            }
        }

        BoundingBox { min, max }
    }

    pub fn get_vertex_input_attribute_descriptions() -> Vec<VertexInputAttributeDescription> {
        vec![
            vk::VertexInputAttributeDescription {
                location: 0,
                binding: 0,
                format: vk::Format::R32G32B32_SFLOAT,
                offset: offset_of!(Vertex, position) as u32,
            },
            vk::VertexInputAttributeDescription {
                location: 1,
                binding: 0,
                format: vk::Format::R32G32B32_SFLOAT,
                offset: offset_of!(Vertex, color) as u32,
            },
            vk::VertexInputAttributeDescription {
                location: 2,
                binding: 0,
                format: vk::Format::R32G32B32_SFLOAT,
                offset: offset_of!(Vertex, normal) as u32,
            },
            vk::VertexInputAttributeDescription {
                location: 3,
                binding: 0,
                format: vk::Format::R32G32_SFLOAT,
                offset: offset_of!(Vertex, uv) as u32,
            },
        ]
    }

    pub fn get_vertex_input_binding_descriptions() -> Vec<VertexInputBindingDescription> {
        vec![vk::VertexInputBindingDescription {
            binding: 0,
            stride: size_of::<Vertex>() as u32,
            input_rate: vk::VertexInputRate::VERTEX,
        }]
    }
}
