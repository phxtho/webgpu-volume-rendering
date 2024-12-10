use std::f32::consts::PI;
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
    uniforms: [f32; 6],
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = event_loop.create_window(Default::default()).unwrap();
        let image_volume = load_image_volume("data/eclipse-10.0.42-fsrt-brain").unwrap();
        self.uniforms = [0.01, 1., 1., 1., 1., 0.];
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
                graphics.render(self.uniforms.as_slice()).unwrap();
                graphics.window.request_redraw();
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key:
                            key @ (PhysicalKey::Code(KeyCode::ArrowLeft)
                            | PhysicalKey::Code(KeyCode::ArrowRight)
                            | PhysicalKey::Code(KeyCode::ArrowUp)
                            | PhysicalKey::Code(KeyCode::ArrowDown)),
                        state: ElementState::Pressed,
                        ..
                    },
                ..
            } => {
                handle_user_input(key, &mut self.uniforms);
            }
            _ => (),
        }
    }
}

fn handle_user_input(key: PhysicalKey, uniforms: &mut [f32; 6]) {
    let slice_delta: f32 = 0.05;
    let rotation_delta = 0.1;

    match key {
        PhysicalKey::Code(KeyCode::ArrowLeft) => uniforms[5] += rotation_delta,
        PhysicalKey::Code(KeyCode::ArrowRight) => uniforms[5] -= rotation_delta,
        PhysicalKey::Code(KeyCode::ArrowUp) => uniforms[0] += slice_delta,
        PhysicalKey::Code(KeyCode::ArrowDown) => uniforms[0] -= slice_delta,
        _ => (),
    }
    uniforms[5] = uniforms[5].clamp(-PI, PI);
    uniforms[0] = uniforms[0].clamp(0., 1.);
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
    event_loop.set_control_flow(ControlFlow::Poll);
    let mut app = App::default();
    event_loop.run_app(&mut app).unwrap();
}
