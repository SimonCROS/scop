use anyhow::{Context, Result};
use ash::{
    prelude::VkResult,
    vk::{self, DeviceQueueCreateInfo, PhysicalDevice},
    Instance,
};

pub struct RendererDevice {
    pub physical_device: vk::PhysicalDevice,
    pub logical_device: ash::Device,
    pub graphics_queue: vk::Queue,
    pub graphics_queue_family: u32,
}

impl RendererDevice {
    fn pick_physical_device(instance: &Instance) -> Result<Option<PhysicalDevice>> {
        let physical_devices = unsafe { instance.enumerate_physical_devices() }?;

        let mut choosen = None;

        for physical_device in physical_devices {
            let props = unsafe { instance.get_physical_device_properties(physical_device) };

            if props.device_type == vk::PhysicalDeviceType::DISCRETE_GPU {
                choosen = Some(physical_device)
            }
        }

        Ok(choosen)
    }

    fn pick_queue_family(instance: &Instance, physical_device: vk::PhysicalDevice) -> Option<u32> {
        let props =
            unsafe { instance.get_physical_device_queue_family_properties(physical_device) };

        props
            .into_iter()
            .position(|qf| qf.queue_flags.contains(vk::QueueFlags::GRAPHICS))
            .map(|i| i as u32)
    }

    fn create_logical_device(
        instance: &Instance,
        physical_device: vk::PhysicalDevice,
        graphics_queue_family: u32,
    ) -> VkResult<ash::Device> {
        let queue_priorities = [1.0f32];

        let queue_create_infos = [DeviceQueueCreateInfo::builder()
            .queue_family_index(graphics_queue_family)
            .queue_priorities(&queue_priorities)
            .build()];

        let create_info = vk::DeviceCreateInfo::builder().queue_create_infos(&queue_create_infos);

        unsafe { instance.create_device(physical_device, &create_info, None) }
    }

    pub fn new(instance: &Instance, layer_pts: &Vec<*const i8>) -> Result<Self> {
        let physical_device =
            Self::pick_physical_device(instance)?.context("No physical device found")?;

        let graphics_queue_family = Self::pick_queue_family(instance, physical_device)
            .context("No suitable queue family")?;

        let logical_device =
            Self::create_logical_device(instance, physical_device, graphics_queue_family)?;

        let graphics_queue = unsafe { logical_device.get_device_queue(graphics_queue_family, 0) };

        Ok(Self {
            physical_device,
            logical_device,
            graphics_queue,
            graphics_queue_family,
        })
    }

    pub unsafe fn cleanup(&self) {}
}
