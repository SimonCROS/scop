use ash::vk::DescriptorSet;

use crate::renderer::RendererPipeline;

enum MaterialPass {
    MainColor,
    Transparent,
    Other,
}

struct Material {
    pipeline: RendererPipeline,
    material_set: DescriptorSet,
    pass_type: MaterialPass,
}
