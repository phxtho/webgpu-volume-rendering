use wgpu::{DeviceDescriptor, RequestAdapterOptions};

fn main() {
    pollster::block_on(run());
}

async fn run() {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::default());
    let adapter = instance
        .request_adapter(&RequestAdapterOptions::default())
        .await
        .expect("failed to get adapter");
    let (device, _) = adapter
        .request_device(&DeviceDescriptor::default(), None)
        .await
        .expect("failed to get device");
    println!("{:?}", device);
}
