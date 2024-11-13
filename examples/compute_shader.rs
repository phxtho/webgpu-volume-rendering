use std::iter;
use wgpu::{
    CommandEncoderDescriptor, ComputePassDescriptor, ComputePipelineDescriptor, DeviceDescriptor,
    Instance, InstanceDescriptor, RequestAdapterOptions, ShaderModuleDescriptor, ShaderSource,
};

fn main() {
    pollster::block_on(run());
}
async fn run() {
    let instance = Instance::new(InstanceDescriptor::default());
    let adapter = instance
        .request_adapter(&RequestAdapterOptions::default())
        .await
        .expect("Failed to get an adapter");

    let (device, queue) = adapter
        .request_device(&DeviceDescriptor::default(), None)
        .await
        .expect("Failed to get device and cmd queue");

    let shader_module = device.create_shader_module(ShaderModuleDescriptor {
        label: Some("Compute shader"),
        source: ShaderSource::Wgsl(
            r#"
             @compute @workgroup_size(64)
            fn main() {
            // Pointless!
            }
            "#
            .into(),
        ),
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
