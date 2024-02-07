pub mod device;
pub mod swapchain;
pub mod window;

use std::ffi::{self, c_char, CString};

use anyhow::Result;
use ash::{
    extensions::ext::{self},
    vk,
};

use raw_window_handle::HasRawDisplayHandle;
use window::RendererWindow;
use winit::{
    event::{ElementState, Event, KeyEvent, WindowEvent},
    event_loop::ControlFlow,
    keyboard::{Key, NamedKey},
};

use self::{device::RendererDevice, swapchain::RendererSwapchain};

pub struct Renderer {
    pub instance: ash::Instance,
    // pub debug: RendererDebug,
    pub window: RendererWindow,
    pub main_device: RendererDevice,
    pub swapchain: RendererSwapchain,
    // pub render_pass: vk::RenderPass,
    // pub graphics_pipeline: RendererPipeline,
    // pub command_pools: CommandPools,
    // pub graphics_command_buffers: Vec<vk::CommandBuffer>,
}

impl Renderer {
    fn used_layer_names() -> Vec<ffi::CString> {
        vec![ffi::CString::new("VK_LAYER_KHRONOS_validation").unwrap()]
    }

    fn used_extensions() -> Vec<*const c_char> {
        vec![ext::DebugUtils::name().as_ptr()]
    }

    pub fn new() -> Result<Self> {
        let (event_loop, window) = RendererWindow::create_window()?;

        let entry = unsafe { ash::Entry::load() }?;

        let layers_names = Self::used_layer_names();
        let layers_names_raw: Vec<*const c_char> =
            layers_names.iter().map(|l| l.as_ptr()).collect();

        let mut extension_names = Self::used_extensions();
        extension_names.extend_from_slice(ash_window::enumerate_required_extensions(
            window.raw_display_handle(),
        )?);

        let instance = Self::create_instance(&entry, &layers_names_raw, &extension_names)?;

        let window = RendererWindow::new(event_loop, window, &entry, &instance)?;

        let main_device = RendererDevice::new(&instance)?;

        let render_pass = Self::create_render_pass(&main_device, &window)?;

        let mut swapchain = RendererSwapchain::new(&instance, &main_device, &window)?;
        swapchain.create_framebuffers(&main_device, render_pass)?;

        Ok(Self {
            instance,
            window,
            main_device,
            swapchain,
        })
    }

    pub fn run(&mut self) -> Result<()> {
        // TODO Wrapper in window with close already set
        let event_loop = self.window.acquire_event_loop()?;
        event_loop.set_control_flow(ControlFlow::Poll);
        event_loop.run(move |event, elwt| match event {
            Event::WindowEvent {
                event:
                    WindowEvent::CloseRequested
                    | WindowEvent::KeyboardInput {
                        event:
                            KeyEvent {
                                state: ElementState::Pressed,
                                logical_key: Key::Named(NamedKey::Escape),
                                ..
                            },
                        ..
                    },
                ..
            } => elwt.exit(),
            _ => (),
        })?;

        Ok(())
    }

    fn create_instance(
        entry: &ash::Entry,
        used_layer_names: &Vec<*const c_char>,
        used_extensions: &Vec<*const c_char>,
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
            .enabled_extension_names(used_extensions)
            .enabled_layer_names(used_layer_names);

        let instance = unsafe { entry.create_instance(&instance_info, None)? };

        Ok(instance)
    }

    fn create_render_pass(
        device: &RendererDevice,
        window: &RendererWindow,
    ) -> Result<vk::RenderPass> {
        let surface_formats = window.formats(device.physical_device)?;
        let surface_format = surface_formats.first().unwrap();

        let attachments = [vk::AttachmentDescription::builder()
            .format(surface_format.format)
            .samples(vk::SampleCountFlags::TYPE_1)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::STORE)
            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(vk::ImageLayout::PRESENT_SRC_KHR)
            .build()];

        let color_attachment_references = [vk::AttachmentReference::builder()
            .attachment(0)
            .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
            .build()];

        let subpasses = [vk::SubpassDescription::builder()
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
            .color_attachments(&color_attachment_references)
            .build()];

        let subpass_dependencies = [vk::SubpassDependency::builder()
            .src_subpass(vk::SUBPASS_EXTERNAL)
            .src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
            .dst_subpass(0)
            .dst_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
            .dst_access_mask(
                vk::AccessFlags::COLOR_ATTACHMENT_READ | vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
            )
            .build()];

        let render_pass_info = vk::RenderPassCreateInfo::builder()
            .attachments(&attachments)
            .subpasses(&subpasses)
            .dependencies(&subpass_dependencies);

        let render_pass = unsafe {
            device
                .logical_device
                .create_render_pass(&render_pass_info, None)
        }?;

        Ok(render_pass)
    }

    /// Renders a frame for our Vulkan app.
    unsafe fn render(&mut self, _window: &RendererWindow) -> Result<()> {
        unimplemented!()
    }

    /// Destroys our Vulkan app.
    unsafe fn destroy(&mut self) {}
}
