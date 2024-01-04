pub mod window;

use anyhow::Result;
use ash::vk;

use window::RendererWindow;

pub struct Renderer {
    pub instance: ash::Instance,
    // pub debug: RendererDebug,
    // pub main_device: RendererDevice,
    pub window: RendererWindow,
    // pub swapchain: RendererSwapchain,
    pub render_pass: vk::RenderPass,
    // pub graphics_pipeline: RendererPipeline,
    // pub command_pools: CommandPools,
    pub graphics_command_buffers: Vec<vk::CommandBuffer>,
}

impl Renderer {
    fn new(
        layer_name_pts: &Vec<*const i8>,
        extension_name_pts: &Vec<*const i8>,
    ) -> Result<Self> {
        let entry = unsafe { ash::Entry::load() }?;
        let app_info = vk::ApplicationInfo::builder().api_version(vk::API_VERSION_1_3);

        let instance_info = vk::InstanceCreateInfo::builder()
            .application_info(&app_info)
            .enabled_extension_names(extension_name_pts)
            .enabled_layer_names(layer_name_pts);

        let instance = unsafe { entry.create_instance(&instance_info, None)? };

        
    }

    /// Renders a frame for our Vulkan app.
    unsafe fn render(&mut self, window: &RendererWindow) -> Result<()> {
        unimplemented!()
    }

    /// Destroys our Vulkan app.
    unsafe fn destroy(&mut self) {}
}
