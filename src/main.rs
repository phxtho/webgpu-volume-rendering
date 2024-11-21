use std::path::PathBuf;

use anyhow::{Error, Ok};
use dicom_reader::ImageVolume;
use graphics::Graphics;
use pollster::FutureExt;
use winit::application::ApplicationHandler;
use winit::event::{ElementState, KeyEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::WindowId;

mod dicom_reader;
mod graphics;

#[derive(Default)]
struct App {
    graphics: Option<Graphics>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = event_loop.create_window(Default::default()).unwrap();
        let image_volume = load_image_volume("data/eclipse-10.0.42-fsrt-brain").unwrap();
        self.graphics = Some(Graphics::new(window, image_volume).block_on());
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested
            | WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        state: ElementState::Pressed,
                        physical_key: PhysicalKey::Code(KeyCode::Escape),
                        ..
                    },
                ..
            } => {
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                let graphics = self.graphics.as_mut().unwrap();
                let sliders = vec![1.; 6];
                graphics.render(sliders.as_slice()).unwrap();
                graphics.window.request_redraw();
            }
            _ => (),
        }
    }
}

fn load_image_volume(path: &str) -> Result<ImageVolume, Error> {
    let data_dir = PathBuf::from(path);
    let files: Vec<PathBuf> = std::fs::read_dir(data_dir)?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .collect();

    dicom_reader::load_dicom_image(&files)
}

fn main() -> Result<(), anyhow::Error> {
    pollster::block_on(run());
    Ok(())
}

async fn run() {
    let event_loop = EventLoop::new().expect("Failed to create event loop");
    event_loop.set_control_flow(ControlFlow::Poll); // Assuming this is better for real-time rendering
    let mut app = App::default();
    event_loop.run_app(&mut app).unwrap();
}
