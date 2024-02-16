use ash::vk::DescriptorSet;

use crate::renderer::pipeline::RendererPipeline;

enum MaterialPass {
    MainColor,
    Transparent,
    Other
}

struct Material {
	pipeline: RendererPipeline,
	material_set: DescriptorSet,
	pass_type: MaterialPass,
}

// struct MaterialInstance {
//     Ren* pipeline;
//     VkDescriptorSet materialSet;
//     MaterialPass passType;
// };

// struct RenderObject {
// 	Mesh* mesh;

// 	Material* material;

// 	glm::mat4 transformMatrix;
// };