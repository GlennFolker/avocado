use crate::incl::*;

#[derive(Resource)]
pub struct SurfaceConfig {
    pub surface: wgpu::Surface,
    pub config: wgpu::SurfaceConfiguration,
    pub size: winit::PhysicalSize<u32>,
}

#[derive(Resource)]
pub struct Renderer {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
}

#[derive(Deref, DerefMut)]
pub struct EventLoop(pub winit::EventLoop<()>);

#[derive(Deref, DerefMut)]
pub struct WinitWindow(pub winit::Window);
