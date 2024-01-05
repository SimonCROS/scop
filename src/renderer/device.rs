use anyhow::{bail, Context, Error, Result};
use ash::{
    prelude::VkResult,
    vk::{
        DeviceCreateInfo, DeviceQueueCreateInfo, PhysicalDevice, PhysicalDeviceType, Queue,
        QueueFlags,
    },
    Instance, {self},
};

pub struct QueueFamily {
    pub index: u32,
    pub flags: QueueFlags,
    pub queues: Vec<Queue>,
}

pub struct RendererDevice {
    pub physical_device: PhysicalDevice,
    pub logical_device: ash::Device,
    queue_families: Vec<QueueFamily>,
}

impl RendererDevice {
    fn pick_physical_device(instance: &Instance) -> Result<Option<PhysicalDevice>> {
        let physical_devices = unsafe { instance.enumerate_physical_devices() }?;

        let mut choosen = None;

        for physical_device in physical_devices {
            let props = unsafe { instance.get_physical_device_properties(physical_device) };

            if props.device_type == PhysicalDeviceType::DISCRETE_GPU {
                choosen = Some(physical_device)
            }
        }

        Ok(choosen)
    }

    fn pick_queue_families(
        instance: &Instance,
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
        instance: &Instance,
        physical_device: PhysicalDevice,
        queue_families: Vec<QueueFamily>,
    ) -> VkResult<ash::Device> {
        let queue_priorities = [1.0f32];

        let mut queue_create_infos: Vec<DeviceQueueCreateInfo> = queue_families
            .iter()
            .map(|family| {
                DeviceQueueCreateInfo::builder()
                    .queue_family_index(family.index)
                    .queue_priorities(&queue_priorities)
                    .build()
            })
            .collect();

        let create_info = DeviceCreateInfo::builder()
            .queue_create_infos(&queue_create_infos)
            .enabled_extension_names(&[ash::extensions::khr::Swapchain::name().as_ptr()]);

        unsafe { instance.create_device(physical_device, &create_info, None) }
    }

    pub fn new(instance: &Instance) -> Result<Self> {
        let physical_device =
            Self::pick_physical_device(instance)?.context("No physical device found")?;

        let mut queue_families = Self::pick_queue_families(instance, physical_device);
        if queue_families.is_empty() {
            bail!("No suitable queue family found");
        }

        let logical_device =
            Self::create_logical_device(instance, physical_device, queue_families)?;

        queue_families.iter_mut().for_each(|family| {
            family
                .queues
                .push(unsafe { logical_device.get_device_queue(family.index, 0) })
        });

        Ok(Self {
            physical_device,
            logical_device,
            queue_families,
        })
    }

    pub fn queue_family(&self, flags: QueueFlags) -> Option<&QueueFamily> {
        self.queue_families.iter().find(|f| f.flags.contains(flags))
    }

    pub fn main_graphics_queue_family(&self) -> &QueueFamily {
        self.queue_family(QueueFlags::GRAPHICS).unwrap()
    }

    pub unsafe fn cleanup(&self) {
        self.logical_device.destroy_device(None);
    }
}
