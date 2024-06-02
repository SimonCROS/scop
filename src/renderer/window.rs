use anyhow::Result;
use ash::{extensions::khr, vk};
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};
use winit::{
    dpi::LogicalSize,
    event_loop::EventLoop,
    window::{Window, WindowBuilder},
};

pub struct RendererWindow {
    pub event_loop: Option<EventLoop<()>>,
    pub window: Window,
    pub surface: vk::SurfaceKHR,
    pub surface_loader: khr::Surface,
}

impl RendererWindow {
    pub fn create_window() -> Result<(EventLoop<()>, Window)> {
        let event_loop = EventLoop::new()?;
        let window = WindowBuilder::new()
            .with_title("scop")
            .with_inner_size(LogicalSize::new(1024, 768))
            .with_resizable(false)
            .build(&event_loop)?;

        Ok((event_loop, window))
    }

    pub fn new(
        event_loop: EventLoop<()>,
        window: Window,
        entry: &ash::Entry,
        instance: &ash::Instance,
    ) -> Result<Self> {
        let surface = unsafe {
            ash_window::create_surface(
                entry,
                instance,
                window.raw_display_handle(),
                window.raw_window_handle(),
                None,
            )
        }?;

        let surface_loader = khr::Surface::new(entry, instance);

        Ok(Self {
            event_loop: Some(event_loop),
            window,
            surface,
            surface_loader,
        })
    }

    pub fn capabilities(
        &self,
        physical_device: vk::PhysicalDevice,
    ) -> Result<vk::SurfaceCapabilitiesKHR, vk::Result> {
        unsafe {
            self.surface_loader
                .get_physical_device_surface_capabilities(physical_device, self.surface)
        }
    }

    pub fn formats(
        &self,
        physical_device: vk::PhysicalDevice,
    ) -> Result<Vec<vk::SurfaceFormatKHR>, vk::Result> {
        unsafe {
            self.surface_loader
                .get_physical_device_surface_formats(physical_device, self.surface)
        }
    }

    pub fn acquire_event_loop(&mut self) -> Result<winit::event_loop::EventLoop<()>> {
        match self.event_loop.take() {
            None => anyhow::bail!("EventLoop was acquired before"),
            Some(el) => Ok(el),
        }
    }

    pub fn run<F: FnMut() -> Result<()>>(
        event_loop: EventLoop<()>,
        mut draw_request: F,
    ) -> Result<()> {
        event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
        event_loop.run(move |event, elwt| match event {
            winit::event::Event::WindowEvent {
                event:
                    winit::event::WindowEvent::CloseRequested
                    | winit::event::WindowEvent::KeyboardInput {
                        event:
                            winit::event::KeyEvent {
                                state: winit::event::ElementState::Pressed,
                                logical_key:
                                    winit::keyboard::Key::Named(winit::keyboard::NamedKey::Escape),
                                ..
                            },
                        ..
                    },
                ..
            } => elwt.exit(),
            winit::event::Event::NewEvents(winit::event::StartCause::Poll) => {
                match draw_request() {
                    Ok(_) => (),
                    Err(e) => {
                        dbg!(e);
                        elwt.exit();
                    }
                }
            }
            _ => (),
        })?;

        Ok(())
    }

    pub fn cleanup(&self) {
        unsafe { self.surface_loader.destroy_surface(self.surface, None) };
    }
}
