pub mod device;
pub mod swapchain;
pub mod window;

use std::ffi::CString;

use anyhow::Result;
use ash::vk;

use raw_window_handle::HasRawDisplayHandle;
use window::RendererWindow;

use self::device::RendererDevice;

pub struct Renderer {
    pub instance: ash::Instance,
    // pub debug: RendererDebug,
    pub window: RendererWindow,
    pub main_device: RendererDevice,
    // pub swapchain: RendererSwapchain,
    // pub render_pass: vk::RenderPass,
    // pub graphics_pipeline: RendererPipeline,
    // pub command_pools: CommandPools,
    // pub graphics_command_buffers: Vec<vk::CommandBuffer>,
}

impl Renderer {
    fn new() -> Result<Self> {
        let (event_loop, window) = RendererWindow::create_window()?;

        let entry = unsafe { ash::Entry::load() }?;
        let app_info = vk::ApplicationInfo::builder().api_version(vk::API_VERSION_1_3);

        let extension_name_pts =
            ash_window::enumerate_required_extensions(window.raw_display_handle())?;

        let instance = Self::create_instance(&entry, &vec![], extension_name_pts)?;

        let window = RendererWindow::new(event_loop, window, &entry, &instance)?;

        let main_device = RendererDevice::new(&instance)?;

        Ok(Self { instance, window, main_device })
    }

    fn create_instance(
        entry: &ash::Entry,
        layer_name_pts: &[*const i8],
        extension_name_pts: &[*const i8],
    ) -> Result<ash::Instance> {
        let app_name = CString::new("Vulkan App")?;
        let engine_name = CString::new("Vulkan Engine")?;

        let app_info = vk::ApplicationInfo::builder()
            .application_name(&app_name)
            .engine_name(&engine_name)
            .application_version(vk::make_api_version(0, 1, 0, 0))
            .engine_version(vk::make_api_version(0, 1, 0, 0))
            .api_version(vk::API_VERSION_1_3);

        let instance_info = vk::InstanceCreateInfo::builder()
            .application_info(&app_info)
            .enabled_extension_names(extension_name_pts)
            .enabled_layer_names(layer_name_pts);

        let instance = unsafe { entry.create_instance(&instance_info, None)? };

        Ok(instance)
    }

    /// Renders a frame for our Vulkan app.
    unsafe fn render(&mut self, window: &RendererWindow) -> Result<()> {
        unimplemented!()
    }

    /// Destroys our Vulkan app.
    unsafe fn destroy(&mut self) {}
}
