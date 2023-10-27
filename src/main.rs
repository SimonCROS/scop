use anyhow::{Context, Result};
use ash::{
    self,
    vk::{self, PhysicalDevice},
    Device, Instance,
};
use winit::{
    event::{ElementState, Event, KeyEvent, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    keyboard::{Key, NamedKey},
    window::{Window, WindowBuilder},
};

fn destroy_instance(instance: Instance) {
    unsafe { instance.destroy_instance(None) }
}

fn init_window() -> Result<Window> {
    let event_loop = EventLoop::new()?;
    let window = WindowBuilder::new().with_title("scop").build(&event_loop)?;

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

    Ok(window)
}

fn main() -> Result<()> {
    let instance = {
        let entry = unsafe { ash::Entry::load() }?;
        let application_info = vk::ApplicationInfo::builder()
            .api_version(vk::API_VERSION_1_3)
            .build();
        let create_info = vk::InstanceCreateInfo::builder()
            .application_info(&application_info)
            .build();
        unsafe { entry.create_instance(&create_info, None) }?
    };

    let device = {
        let physical_device = unsafe { instance.enumerate_physical_devices() }?
            .into_iter()
            .next()
            .context("No physical device found")?;
        let create_info = vk::DeviceCreateInfo::builder().build();
        unsafe { instance.create_device(physical_device, &create_info, None) }
    }?;

    let window = init_window().unwrap();

    destroy_instance(instance);
    Ok(())
}
