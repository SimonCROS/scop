pub mod command_pools;
pub mod debug;
pub mod device;
pub mod index_buffer;
pub mod pipeline;
pub mod shader;
pub mod swapchain;
pub mod vertex_buffer;
pub mod window;
pub mod scop_buffer;

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
    vk::{self, ShaderStageFlags},
};

use crate::engine::GameObject;

use self::{
    command_pools::CommandPools, debug::RendererDebug, device::RendererDevice,
    pipeline::{RendererPipeline, SimplePushConstantData}, swapchain::RendererSwapchain,
};
use raw_window_handle::HasRawDisplayHandle;
use window::RendererWindow;

pub struct Renderer {
    #[allow(
        dead_code,
        reason = "Segfaults when destroy device if not in the struct"
    )]
    entry: ash::Entry,
    pub instance: ash::Instance,
    pub debug: RendererDebug,
    pub window: RendererWindow,
    pub main_device: Rc<RendererDevice>,
    pub swapchain: RendererSwapchain,
    pub render_pass: vk::RenderPass,
    pub graphics_pipeline: RendererPipeline,
    pub command_pools: CommandPools,
    pub graphics_command_buffers: Vec<vk::CommandBuffer>,
    pub graphics_queue: vk::Queue,
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

        let window = RendererWindow::new(event_loop, window, &entry, &instance)?;

        let debug = RendererDebug::new(&entry, &instance)?;

        let main_device = RendererDevice::new(&instance)?;

        let render_pass = Self::create_render_pass(&main_device, &window)?;

        let mut swapchain = RendererSwapchain::new(&instance, &main_device, &window)?;
        swapchain.create_framebuffers(&main_device, render_pass)?;

        let graphics_pipeline = RendererPipeline::new(&main_device, swapchain.extent, render_pass)?;

        let command_pools = CommandPools::new(&main_device)?;

        let graphics_command_buffers = CommandPools::create_command_buffers(
            &main_device,
            command_pools.graphics,
            swapchain.image_count as u32,
        )?;

        let graphics_queue = main_device.main_graphics_queue_family().queues[0];

        Ok(Self {
            entry,
            instance,
            debug,
            main_device: Rc::new(main_device),
            window,
            swapchain,
            render_pass,
            graphics_pipeline,
            command_pools,
            graphics_command_buffers,
            graphics_queue,
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
        let command_buffer = self.graphics_command_buffers[image_index as usize];

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

        self.fill_command_buffer(command_buffer, |command_buffer: vk::CommandBuffer| {
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
        })
        .unwrap();

        self.submit_command_buffer(
            self.graphics_queue,
            command_buffer,
            image_available,
            rendering_finished,
            may_begin_drawing,
        );

        self.present_command_buffer(self.graphics_queue, image_index, rendering_finished);

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
        // let surface_format = surface_formats.iter().find(|s| ).first().unwrap();

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

    fn fill_command_buffer<F: FnOnce(vk::CommandBuffer)>(
        &self,
        command_buffer: vk::CommandBuffer,
        f: F,
    ) -> Result<()> {
        let begin_info = vk::CommandBufferBeginInfo::builder();

        unsafe {
            self.main_device
                .logical_device
                .begin_command_buffer(command_buffer, &begin_info)?;
            f(command_buffer);
            self.main_device
                .logical_device
                .end_command_buffer(command_buffer)?;
        }

        Ok(())
    }

    fn submit_command_buffer(
        &self,
        queue: vk::Queue,
        command_buffer: vk::CommandBuffer,
        image_available: vk::Semaphore,
        rendering_finished: vk::Semaphore,
        may_begin_drawing: vk::Fence,
    ) {
        let wait_semaphores = [image_available];
        let wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        let command_buffers = [command_buffer];
        let signal_semaphores = [rendering_finished];

        let submit_info = vk::SubmitInfo::builder()
            .wait_semaphores(&wait_semaphores)
            .wait_dst_stage_mask(&wait_stages)
            .command_buffers(&command_buffers)
            .signal_semaphores(&signal_semaphores);

        unsafe {
            self.main_device
                .logical_device
                .queue_submit(queue, &[submit_info.build()], may_begin_drawing)
                .unwrap();
        }
    }

    fn present_command_buffer(
        &self,
        queue: vk::Queue,
        image_index: u32,
        rendering_finished: vk::Semaphore,
    ) {
        let swapchains = [self.swapchain.swapchain];
        let image_indices = [image_index];

        let present_info = vk::PresentInfoKHR::builder()
            .wait_semaphores(slice::from_ref(&rendering_finished))
            .swapchains(&swapchains)
            .image_indices(&image_indices);

        unsafe {
            self.swapchain
                .swapchain_loader
                .queue_present(queue, &present_info)
                .unwrap();
        }
    }

    fn add_render_pass<F: FnOnce(vk::CommandBuffer)>(
        &self,
        command_buffer: vk::CommandBuffer,
        render_pass: vk::RenderPass,
        framebuffer: vk::Framebuffer,
        f: F,
    ) {
        let clear_values = [vk::ClearValue {
            color: vk::ClearColorValue {
                float32: [0.0, 0.0, 0.0, 1.0],
            },
        }];

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
        dbg!("Drop Renderer");
        unsafe {
            let _ = self.main_device.logical_device.device_wait_idle();

            self.command_pools.cleanup(
                &self.main_device.logical_device,
                &self.graphics_command_buffers,
            );
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
