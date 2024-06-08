use std::{
    cell::RefCell,
    collections::HashMap,
    ffi::{self, c_char, CString},
    mem::size_of,
    rc::Rc,
};

use anyhow::Result;
use ash::{
    extensions::ext,
    vk::{self, CommandPoolCreateFlags, PipelineStageFlags, QueueFlags, ShaderStageFlags},
};

use crate::engine::{camera::Camera, mesh::Mesh, GameObject};

use raw_window_handle::HasRawDisplayHandle;

use super::{
    Material, MaterialInstance, RendererDebug, RendererDevice, RendererWindow, ScopBuffer, ScopCommandPool, ScopDescriptorPool, ScopDescriptorSetLayout, ScopDescriptorWriter, ScopGpuCameraData, ScopRenderPass, ScopSwapchain, SimplePushConstantData
};

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
    pub global_descriptor_pool: ScopDescriptorPool,
    pub global_descriptor_set_layout: ScopDescriptorSetLayout,
    pub global_descriptor_sets: Vec<vk::DescriptorSet>,
    pub graphic_command_pools: Vec<ScopCommandPool>,
    pub camera_buffers: Vec<ScopBuffer>,
    pub frame_count: u32,
    pub flat_texture_interpolation: f32,
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

        let global_descriptor_pool = ScopDescriptorPool::builder(&main_device)
            .add_size(
                vk::DescriptorType::UNIFORM_BUFFER,
                swapchain.image_count as u32,
            )
            .add_size(
                vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                (swapchain.image_count * 3) as u32,
            )
            .max_sets((swapchain.image_count * 10) as u32)
            .build()?;

        let global_descriptor_set_layout = ScopDescriptorSetLayout::builder(&main_device)
            .add_binding(
                0,
                vk::DescriptorType::UNIFORM_BUFFER,
                vk::ShaderStageFlags::VERTEX,
            )
            .build()?;

        let mut graphic_command_pools =
            Vec::<ScopCommandPool>::with_capacity(swapchain.image_count);
        let mut camera_buffers = Vec::<ScopBuffer>::with_capacity(swapchain.image_count);
        for _ in 0..swapchain.image_count {
            let mut graphic_command_pool = ScopCommandPool::new(
                main_device.clone(),
                main_device
                    .get_queue_family_with(QueueFlags::GRAPHICS)
                    .unwrap(),
                CommandPoolCreateFlags::RESET_COMMAND_BUFFER,
            )?;
            graphic_command_pool.create_command_buffers(1 as u32)?;
            graphic_command_pools.push(graphic_command_pool);

            camera_buffers.push(ScopBuffer::new(
                main_device.clone(),
                1,
                size_of::<ScopGpuCameraData>() as u64,
                vk::BufferUsageFlags::UNIFORM_BUFFER,
                vk::MemoryPropertyFlags::HOST_VISIBLE,
                1,
            )?);
        }

        let mut global_descriptor_sets =
            Vec::<vk::DescriptorSet>::with_capacity(swapchain.image_count);
        for i in 0..swapchain.image_count {
            let allocate_info = vk::DescriptorSetAllocateInfo::builder()
                .descriptor_pool(global_descriptor_pool.descriptor_pool)
                .set_layouts(&[global_descriptor_set_layout.set_layout])
                .build();

            let set = unsafe {
                main_device
                    .logical_device
                    .allocate_descriptor_sets(&allocate_info)?[0]
            };

            ScopDescriptorWriter::new(&main_device, &global_descriptor_set_layout)
                .descriptors(&[set])
                .set_buffer(0, &camera_buffers[i])
                .write();

            global_descriptor_sets.push(set);
        }

        Ok(Self {
            entry,
            instance,
            debug,
            main_device,
            window,
            swapchain,
            defaut_render_pass,
            global_descriptor_pool,
            global_descriptor_set_layout,
            global_descriptor_sets,
            graphic_command_pools,
            camera_buffers,
            frame_count: 0,
            flat_texture_interpolation: 0.
        })
    }

    pub fn handle_draw_request(
        &mut self,
        camera: &Camera,
        game_objects: &HashMap<u32, Rc<RefCell<GameObject>>>
    ) -> Result<()> {
        self.frame_count += 1;

        let (image_index, image_available, rendering_finished, may_begin_drawing) =
            self.swapchain.next_image()?;

        let camera_data = ScopGpuCameraData {
            projection: *camera.get_projection(),
            view: *camera.get_view(),
        };

        let camera_buffer = &mut self.camera_buffers[image_index as usize];
        camera_buffer.map(vk::WHOLE_SIZE, 0)?;
        camera_buffer.write_to_buffer(&[camera_data], 0);
        camera_buffer.flush(vk::WHOLE_SIZE, 0)?;
        camera_buffer.unmap();

        let command_pool = &self.graphic_command_pools[image_index as usize];
        let command_buffer = command_pool.get_command_buffer(0);

        self.main_device.begin_command_buffer(command_buffer)?;
        self.defaut_render_pass.begin(command_buffer, image_index);

        self.draw_game_objects(game_objects, command_buffer, image_index);

        self.defaut_render_pass.end(command_buffer);
        self.main_device.end_command_buffer(command_buffer)?;
        command_pool.submit(
            &[command_buffer],
            &[image_available],
            &[rendering_finished],
            &[PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT],
            may_begin_drawing,
        )?;
        self.swapchain.queue_present(
            command_pool.get_queue_family().queues[0],
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
        image_index: u32,
    ) {
        let mut previous_mesh_ptr: *const Mesh = std::ptr::null();
        let mut previous_material_ptr: *const Material = std::ptr::null();
        let mut previous_material_instance_ptr: *const MaterialInstance = std::ptr::null();

        for go in game_objects.values() {
            let game_object = go.borrow();

            if let Some(mesh) = &game_object.mesh {
                let material_instance = game_object.material.as_ref().unwrap();

                if previous_material_ptr != Rc::as_ptr(&material_instance.material) {
                    previous_material_ptr = Rc::as_ptr(&material_instance.material);

                    material_instance
                        .material
                        .pipeline
                        .bind(command_buffer, vk::PipelineBindPoint::GRAPHICS);
                }

                if previous_material_instance_ptr != Rc::as_ptr(material_instance) {
                    previous_material_instance_ptr = Rc::as_ptr(material_instance);

                    material_instance.material.pipeline.bind_descriptor_sets(
                        command_buffer,
                        vk::PipelineBindPoint::GRAPHICS,
                        &[
                            self.global_descriptor_sets[image_index as usize],
                            material_instance.material_sets[image_index as usize],
                        ],
                    );
                }

                let push = SimplePushConstantData {
                    model_matrix: game_object.transform.mat(),
                    normal_matrix: game_object.transform.normal_matrix(),
                    dummy0: 0.0,
                    dummy1: 0.0,
                    dummy2: 0.0,
                    flat_texture_interpolation: self.flat_texture_interpolation,
                };

                unsafe {
                    self.main_device.logical_device.cmd_push_constants(
                        command_buffer,
                        material_instance.material.pipeline.pipeline_layout,
                        ShaderStageFlags::VERTEX | ShaderStageFlags::FRAGMENT,
                        0,
                        crate::utils::any_as_u8_slice(&push),
                    );
                }

                if previous_mesh_ptr != Rc::as_ptr(mesh) {
                    previous_mesh_ptr = Rc::as_ptr(mesh);

                    mesh.bind(command_buffer);
                }

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

        self.camera_buffers.iter_mut().for_each(ScopBuffer::cleanup);
        self.graphic_command_pools
            .iter_mut()
            .for_each(ScopCommandPool::cleanup);
        self.global_descriptor_pool.cleanup();
        self.global_descriptor_set_layout.cleanup(&self.main_device);
        self.swapchain.cleanup();
        self.defaut_render_pass.cleanup();
        self.main_device.cleanup();
        self.debug.cleanup();
        self.window.cleanup();

        unsafe { self.instance.destroy_instance(None) };
    }
}
