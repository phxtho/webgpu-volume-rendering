use std::iter;
use std::path::PathBuf;

use anyhow::Ok;
use wgpu::{
    CommandEncoderDescriptor, ComputePassDescriptor, ComputePipelineDescriptor, DeviceDescriptor,
    Instance, InstanceDescriptor, RequestAdapterOptions, ShaderModuleDescriptor, ShaderSource,
};
use winit::application::ApplicationHandler;
use winit::event::{ElementState, KeyEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowId};

mod dicom_reader;

#[derive(Default)]
struct App {
    window: Option<Window>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.window = Some(
            event_loop
                .create_window(Window::default_attributes())
                .unwrap(),
        );
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
                pollster::block_on(async { render(self.window.as_ref().unwrap()).await });
                self.window.as_ref().unwrap().request_redraw();
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

async fn render(window: &Window) {
    let instance = Instance::new(InstanceDescriptor::default());
    let surface = instance
        .create_surface(window)
        .expect("Failed to create surface");
    let adapter = instance
        .request_adapter(&RequestAdapterOptions {
            compatible_surface: Some(&surface),
            ..Default::default()
        })
        .await
        .expect("Failed to get an adapter");

    let (device, queue) = adapter
        .request_device(&DeviceDescriptor::default(), None)
        .await
        .expect("Failed to get device and cmd queue");

    let shader_module = device.create_shader_module(ShaderModuleDescriptor {
        label: Some("Compute shader"),
        source: ShaderSource::Wgsl(include_str!("compute.wgsl").into()),
    });

    let pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
        label: Some("Compute pipeline"),
        layout: None,
        module: &shader_module,
        entry_point: Some("main"),
        compilation_options: Default::default(),
        cache: None,
    });

    let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
        label: Some("Compute Pass"),
    });
    {
        let mut compute_pass = encoder.begin_compute_pass(&ComputePassDescriptor {
            label: Some("Compute Pass"),
            timestamp_writes: None,
        });

        compute_pass.set_pipeline(&pipeline);
        compute_pass.dispatch_workgroups(1, 1, 64);
    }

    queue.submit(iter::once(encoder.finish()));
}
