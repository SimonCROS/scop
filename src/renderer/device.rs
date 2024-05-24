use std::rc::Rc;

use anyhow::{bail, Context, Result};
use ash::{
    prelude::VkResult,
    vk::{
        self, DeviceCreateInfo, DeviceQueueCreateInfo, PhysicalDevice, PhysicalDeviceType, Queue,
        QueueFlags,
    },
    Instance,
};

pub struct QueueFamily {
    pub index: u32,
    pub flags: QueueFlags,
    pub queues: Vec<Queue>,
}

pub struct RendererDevice {
    pub instance: Rc<Instance>,
    pub physical_device: PhysicalDevice,
    pub logical_device: ash::Device,
    pub memory_properties: vk::PhysicalDeviceMemoryProperties,
    queue_families: Vec<QueueFamily>,
}

impl RendererDevice {
    fn pick_physical_device(instance: &Rc<Instance>) -> Result<Option<PhysicalDevice>> {
        let physical_devices = unsafe { instance.enumerate_physical_devices() }?;

        let mut choosen = None;

        for physical_device in physical_devices {
            let props = unsafe { instance.get_physical_device_properties(physical_device) };

            if props.device_type == PhysicalDeviceType::DISCRETE_GPU
                || props.device_type == PhysicalDeviceType::INTEGRATED_GPU
            {
                choosen = Some(physical_device)
            }
        }

        Ok(choosen)
    }

    fn pick_queue_families(
        instance: &Rc<Instance>,
        physical_device: PhysicalDevice,
    ) -> Vec<QueueFamily> {
        let props =
            unsafe { instance.get_physical_device_queue_family_properties(physical_device) };

        props
            .into_iter()
            .enumerate()
            .filter(|(_, qf)| qf.queue_count > 0 && qf.queue_flags.contains(QueueFlags::GRAPHICS))
            .map(|(i, qf)| QueueFamily {
                index: i as u32,
                flags: qf.queue_flags,
                queues: vec![],
            })
            .collect()
    }

    fn create_logical_device(
        instance: &Rc<Instance>,
        physical_device: PhysicalDevice,
        queue_families: &Vec<QueueFamily>,
    ) -> VkResult<ash::Device> {
        let queue_priorities = [1.0f32];

        let queue_create_infos: Vec<DeviceQueueCreateInfo> = queue_families
            .iter()
            .map(|family| {
                DeviceQueueCreateInfo::builder()
                    .queue_family_index(family.index)
                    .queue_priorities(&queue_priorities)
                    .build()
            })
            .collect();

        let extensions = [ash::extensions::khr::Swapchain::name().as_ptr()];

        let create_info = DeviceCreateInfo::builder()
            .queue_create_infos(&queue_create_infos)
            .enabled_extension_names(&extensions);

        unsafe { instance.create_device(physical_device, &create_info, None) }
    }

    pub fn new(instance: &Rc<Instance>) -> Result<Self> {
        dbg!("New device");

        let physical_device =
            Self::pick_physical_device(instance)?.context("No physical device found")?;

        let mut queue_families = Self::pick_queue_families(instance, physical_device);
        if queue_families.is_empty() {
            bail!("No suitable queue family found");
        }

        let logical_device =
            Self::create_logical_device(instance, physical_device, &queue_families)?;

        queue_families.iter_mut().for_each(|family| {
            family
                .queues
                .push(unsafe { logical_device.get_device_queue(family.index, 0) })
        });

        let device_memory_properties =
            unsafe { instance.get_physical_device_memory_properties(physical_device) };

        Ok(Self {
            instance: instance.clone(),
            physical_device,
            logical_device,
            memory_properties: device_memory_properties,
            queue_families,
        })
    }

    pub fn queue_family(&self, flags: QueueFlags) -> Option<&QueueFamily> {
        self.queue_families.iter().find(|f| f.flags.contains(flags))
    }

    pub fn main_graphics_queue_family(&self) -> &QueueFamily {
        self.queue_family(QueueFlags::GRAPHICS).unwrap()
    }

    pub fn find_memorytype_index(
        memory_req: &vk::MemoryRequirements,
        memory_prop: &vk::PhysicalDeviceMemoryProperties,
        flags: vk::MemoryPropertyFlags,
    ) -> Option<u32> {
        memory_prop.memory_types[..memory_prop.memory_type_count as _]
            .iter()
            .enumerate()
            .find(|(index, memory_type)| {
                (1 << index) & memory_req.memory_type_bits != 0
                    && memory_type.property_flags & flags == flags
            })
            .map(|(index, _memory_type)| index as _)
    }

    pub unsafe fn find_supported_format(
        &self,
        formats: Vec<vk::Format>,
        tiling: vk::ImageTiling,
        features: vk::FormatFeatureFlags,
    ) -> vk::Format {
        for format in formats {
            let properties = self
                .instance
                .get_physical_device_format_properties(self.physical_device, format);

            if tiling == vk::ImageTiling::LINEAR
                && (properties.linear_tiling_features & features) == features
            {
                return format;
            } else if tiling == vk::ImageTiling::OPTIMAL
                && (properties.optimal_tiling_features & features) == features
            {
                return format;
            }
        }
        unimplemented!()
    }

    pub unsafe fn cleanup(&self) {
        dbg!("Cleanup device");

        self.logical_device.destroy_device(None);
    }
}
