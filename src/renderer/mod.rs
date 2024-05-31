pub mod debug;
pub mod device;
pub mod pipeline;
pub mod scop_buffer;
pub mod scop_command_pool;
pub mod scop_image;
pub mod shader;
pub mod swapchain;
pub mod window;

use core::slice;
use std::{
    cell::RefCell,
    collections::HashMap,
    ffi::{self, c_char, CString},
    rc::Rc,
};

use anyhow::Result;
use ash::{
    extensions::ext,
    vk::{self, CommandPoolCreateFlags, PipelineStageFlags, QueueFlags, ShaderStageFlags},
};

use crate::engine::GameObject;

use self::{
    debug::RendererDebug,
    device::RendererDevice,
    pipeline::{RendererPipeline, SimplePushConstantData},
    scop_command_pool::ScopCommandPool,
    swapchain::RendererSwapchain,
};
use raw_window_handle::HasRawDisplayHandle;
use window::RendererWindow;

pub struct Renderer {
    #[allow(
        dead_code,
        reason = "Segfaults when destroy device if not in the struct"
    )]
    entry: ash::Entry,
    pub instance: Rc<ash::Instance>,
    pub debug: RendererDebug,
    pub window: RendererWindow,
    pub main_device: Rc<RendererDevice>,
    pub swapchain: RendererSwapchain,
    pub render_pass: vk::RenderPass,
    pub graphics_pipeline: RendererPipeline,
    pub graphic_command_pool: ScopCommandPool,
    pub frame_count: u32,
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
        let instance = Rc::new(instance);

        let window = RendererWindow::new(event_loop, window, &entry, &instance)?;

        let debug = RendererDebug::new(&entry, &instance)?;

        let main_device = Rc::new(RendererDevice::new(&instance)?);

        let render_pass = Self::create_render_pass(&main_device, &window)?;

        let mut swapchain = RendererSwapchain::new(&instance, &main_device, &window)?;
        unsafe { swapchain.create_depth_resources(&main_device)? };
        swapchain.create_framebuffers(&main_device, render_pass)?;

        let graphics_pipeline = RendererPipeline::new(&main_device, swapchain.extent, render_pass)?;

        let mut graphic_command_pool = ScopCommandPool::new(
            main_device.clone(),
            main_device
                .get_queue_family_with(QueueFlags::GRAPHICS)
                .unwrap(),
            CommandPoolCreateFlags::RESET_COMMAND_BUFFER,
        )?;

        graphic_command_pool.create_command_buffers(swapchain.image_count as u32)?;

        Ok(Self {
            entry,
            instance,
            debug,
            main_device,
            window,
            swapchain,
            render_pass,
            graphics_pipeline,
            graphic_command_pool,
            frame_count: 0,
        })
    }

    pub fn handle_draw_request(
        &mut self,
        game_objects: &HashMap<u32, Rc<RefCell<GameObject>>>,
    ) -> Result<()> {
        self.frame_count += 1;

        // acquiring next image:
        let (image_index, image_available, rendering_finished, may_begin_drawing, framebuffer) =
            unsafe { self.swapchain.next_image(&self.main_device) }.unwrap();

        // commands:
        let command_buffer = self.graphic_command_pool.get_command_buffer(image_index);

        // let buffer_barrier = BufferMemoryBarrier2::builder()
        //     .src_access_mask(AccessFlags2::HOST_WRITE)
        //     .dst_access_mask(AccessFlags2::SHADER_READ)
        //     .src_queue_family_index(QUEUE_FAMILY_IGNORED)
        //     .dst_queue_family_index(QUEUE_FAMILY_IGNORED)
        //     .buffer(vertex_buffer.buffer)
        //     .offset(0)
        //     .size(WHOLE_SIZE);
        // let dependency_info = DependencyInfo::builder()
        //     .buffer_memory_barriers(slice::from_ref(&buffer_barrier));
        // unsafe {
        //     self.main_device.logical_device.cmd_pipeline_barrier2(command_buffer, &dependency_info)
        // };

        self.main_device.begin_command_buffer(command_buffer)?;
        self.add_render_pass(
            command_buffer,
            self.render_pass,
            framebuffer,
            |command_buffer| unsafe {
                self.main_device.logical_device.cmd_bind_pipeline(
                    command_buffer,
                    vk::PipelineBindPoint::GRAPHICS,
                    self.graphics_pipeline.pipeline,
                );

                for go in game_objects.values() {
                    let game_object = go.borrow();

                    if let Some(mesh) = &game_object.mesh {
                        let push = SimplePushConstantData {
                            model_matrix: game_object.transform.mat(),
                            normal_matrix: game_object.transform.normal_matrix(),
                        };

                        self.main_device.logical_device.cmd_push_constants(
                            command_buffer,
                            self.graphics_pipeline.pipeline_layout,
                            ShaderStageFlags::VERTEX | ShaderStageFlags::FRAGMENT,
                            0,
                            crate::utils::any_as_u8_slice(&push),
                        );

                        mesh.bind(command_buffer);
                        mesh.draw(command_buffer);
                    }
                }
            },
        );
        self.main_device.end_command_buffer(command_buffer)?;
        self.graphic_command_pool.submit(
            &[command_buffer],
            &[image_available],
            &[rendering_finished],
            &[PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT],
            may_begin_drawing,
        )?;
        self.swapchain.present_image(
            self.graphic_command_pool.get_queue_family().queues[0],
            image_index,
            &[rendering_finished],
        )?;

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
        let depth_format = unsafe {
            device.find_supported_format(
                vec![
                    vk::Format::D32_SFLOAT,
                    vk::Format::D32_SFLOAT_S8_UINT,
                    vk::Format::D24_UNORM_S8_UINT,
                ],
                vk::ImageTiling::OPTIMAL,
                vk::FormatFeatureFlags::DEPTH_STENCIL_ATTACHMENT,
            )?
        };

        let attachments = [
            vk::AttachmentDescription::builder()
                .format(surface_format.format)
                .samples(vk::SampleCountFlags::TYPE_1)
                .load_op(vk::AttachmentLoadOp::CLEAR)
                .store_op(vk::AttachmentStoreOp::STORE)
                .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
                .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
                .initial_layout(vk::ImageLayout::UNDEFINED)
                .final_layout(vk::ImageLayout::PRESENT_SRC_KHR)
                .build(),
            vk::AttachmentDescription::builder()
                .format(depth_format)
                .samples(vk::SampleCountFlags::TYPE_1)
                .load_op(vk::AttachmentLoadOp::CLEAR)
                .store_op(vk::AttachmentStoreOp::DONT_CARE)
                .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
                .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
                .initial_layout(vk::ImageLayout::UNDEFINED)
                .final_layout(vk::ImageLayout::ATTACHMENT_OPTIMAL)
                .build(),
        ];

        let color_attachment_references = [vk::AttachmentReference::builder()
            .attachment(0)
            .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
            .build()];

        let depth_attachment_references = vk::AttachmentReference::builder()
            .attachment(1)
            .layout(vk::ImageLayout::STENCIL_ATTACHMENT_OPTIMAL)
            .build();

        let subpasses = [vk::SubpassDescription::builder()
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
            .color_attachments(&color_attachment_references)
            .depth_stencil_attachment(&depth_attachment_references)
            .build()];

        let subpass_dependencies = [vk::SubpassDependency::builder()
            .src_subpass(vk::SUBPASS_EXTERNAL)
            .src_stage_mask(
                vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT
                    | vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS,
            )
            .dst_stage_mask(
                vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT
                    | vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS,
            )
            .dst_access_mask(
                vk::AccessFlags::COLOR_ATTACHMENT_WRITE
                    | vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
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

    fn add_render_pass<F: FnOnce(vk::CommandBuffer)>(
        &self,
        command_buffer: vk::CommandBuffer,
        render_pass: vk::RenderPass,
        framebuffer: vk::Framebuffer,
        f: F,
    ) {
        let clear_values = [
            vk::ClearValue {
                color: vk::ClearColorValue {
                    float32: [0.0, 0.0, 0.0, 1.0],
                },
            },
            vk::ClearValue {
                depth_stencil: vk::ClearDepthStencilValue {
                    depth: 1f32,
                    stencil: 0,
                },
            },
        ];

        let render_pass_info = vk::RenderPassBeginInfo::builder()
            .render_pass(render_pass)
            .framebuffer(framebuffer)
            .render_area(vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: self.swapchain.extent,
            })
            .clear_values(&clear_values);

        unsafe {
            self.main_device.logical_device.cmd_begin_render_pass(
                command_buffer,
                &render_pass_info,
                vk::SubpassContents::INLINE,
            );
            f(command_buffer);
            self.main_device
                .logical_device
                .cmd_end_render_pass(command_buffer);
        }
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        unsafe {
            let _ = self.main_device.logical_device.device_wait_idle();

            self.graphic_command_pool.cleanup();
            self.graphics_pipeline
                .cleanup(&self.main_device.logical_device);
            self.swapchain.cleanup(&self.main_device.logical_device);
            self.main_device
                .logical_device
                .destroy_render_pass(self.render_pass, None);
            self.main_device.cleanup();
            self.debug.cleanup();
            self.window.cleanup();
            self.instance.destroy_instance(None);
        }
    }
}
