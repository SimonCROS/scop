pub mod debug;
pub mod device;
pub mod pipeline;
pub mod scop_buffer;
pub mod scop_command_pool;
pub mod scop_framebuffer;
pub mod scop_image;
pub mod scop_render_pass;
pub mod shader;
pub mod scop_swapchain;
pub mod window;

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
    scop_render_pass::ScopRenderPass,
    scop_swapchain::ScopSwapchain,
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
    pub swapchain: ScopSwapchain,
    pub defaut_render_pass: ScopRenderPass,
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

        let swapchain = ScopSwapchain::new(&instance, main_device.clone(), &window)?;

        let defaut_render_pass = ScopRenderPass::new(main_device.clone(), &window, &swapchain)?;

        let graphics_pipeline = RendererPipeline::new(
            main_device.clone(),
            swapchain.extent,
            defaut_render_pass.render_pass,
        )?;

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
            defaut_render_pass,
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

        let (image_index, image_available, rendering_finished, may_begin_drawing) =
            self.swapchain.next_image()?;

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
        self.defaut_render_pass.begin(command_buffer, image_index);
        self.graphics_pipeline
            .bind(command_buffer, vk::PipelineBindPoint::GRAPHICS);

        self.draw_game_objects(game_objects, command_buffer);

        self.defaut_render_pass.end(command_buffer);
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

    pub fn wait_gpu(&self) {
        let _ = unsafe { self.main_device.logical_device.device_wait_idle() };
    }

    fn draw_game_objects(
        &self,
        game_objects: &HashMap<u32, Rc<RefCell<GameObject>>>,
        command_buffer: vk::CommandBuffer,
    ) {
        for go in game_objects.values() {
            let game_object = go.borrow();

            if let Some(mesh) = &game_object.mesh {
                let push = SimplePushConstantData {
                    model_matrix: game_object.transform.mat(),
                    normal_matrix: game_object.transform.normal_matrix(),
                };

                unsafe {
                    self.main_device.logical_device.cmd_push_constants(
                        command_buffer,
                        self.graphics_pipeline.pipeline_layout,
                        ShaderStageFlags::VERTEX | ShaderStageFlags::FRAGMENT,
                        0,
                        crate::utils::any_as_u8_slice(&push),
                    );
                }

                mesh.bind(command_buffer);
                mesh.draw(command_buffer);
            }
        }
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
}

impl Drop for Renderer {
    fn drop(&mut self) {
        self.wait_gpu();

        self.graphic_command_pool.cleanup();
        self.graphics_pipeline.cleanup();
        self.swapchain.cleanup();
        self.defaut_render_pass.cleanup();
        self.main_device.cleanup();
        self.debug.cleanup();
        self.window.cleanup();

        unsafe { self.instance.destroy_instance(None) };
    }
}
