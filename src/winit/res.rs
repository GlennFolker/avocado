use crate::core::prelude::*;
use derive_more::*;
use winit::{
    dpi::PhysicalSize,
    event_loop::EventLoop,
    window::Window,
};

#[derive(Resource)]
pub struct SurfaceConfig {
    pub surface: wgpu::Surface,
    pub config: wgpu::SurfaceConfiguration,
    pub size: PhysicalSize<u32>,
}

#[derive(Resource)]
pub struct Renderer {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
}

#[derive(Deref, DerefMut)]
pub struct WinitEventLoop(pub EventLoop<()>);

#[derive(Deref, DerefMut)]
pub struct WinitWindow(pub Window);
