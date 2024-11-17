use std::path::PathBuf;

use anyhow::Ok;
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
        self.graphics = Some(Graphics::new(window).block_on());
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
                graphics.compute_pass();
                graphics.render().unwrap();
                graphics.window.request_redraw();
            }
            _ => (),
        }
    }
}

fn main() -> Result<(), anyhow::Error> {
    let data_dir = PathBuf::from("data/eclipse-10.0.42-fsrt-brain");
    let files: Vec<PathBuf> = std::fs::read_dir(data_dir)?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .collect();

    let volume = dicom_reader::load_dicom_image(&files).expect("failed to read dicom files");
    println!(
        "Loaded volume: {}x{}x{} voxels",
        volume.columns, volume.rows, volume.slices
    );

    pollster::block_on(run());
    Ok(())
}

async fn run() {
    let event_loop = EventLoop::new().expect("Failed to create event loop");
    event_loop.set_control_flow(ControlFlow::Poll); // Assuming this is better for real-time rendering
    let mut app = App::default();
    event_loop.run_app(&mut app).unwrap();
}
