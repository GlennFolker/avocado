use crate::{
    core::prelude::*,
    winit::{
        WindowResizedEvent,
        SurfaceConfig, Renderer,
    },
};

/// Contains informations about the platform surface that'll be rendered to.
#[derive(Resource, Default)]
pub struct Frame {
    pub valid: bool,
    pub output: Option<wgpu::SurfaceTexture>,
    pub view: Option<wgpu::TextureView>,
}

impl Frame {
    pub fn prepare_sys(mut resized: EventReader<WindowResizedEvent>, mut surface: ResMut<SurfaceConfig>, renderer: Res<Renderer>) {
        if !resized.is_empty() {
            let new_size = **resized.iter().next_back().unwrap();
            resized.clear();

            if surface.size.width != new_size.width && surface.size.height != new_size.height {
                surface.size = new_size;
                surface.config.width = new_size.width;
                surface.config.height = new_size.height;

                surface.surface.configure(&renderer.device, &surface.config);
            }
        }
    }

    pub fn init_sys(
        mut exit: EventWriter<ExitEvent>,
        mut frame: ResMut<Frame>, surface: Res<SurfaceConfig>, renderer: Res<Renderer>,
    ) {
        let output = match surface.surface.get_current_texture() {
            Ok(output) => Some(output),
            Err(wgpu::SurfaceError::Lost) => {
                log::warn!("Skipping frame due to surface loss");

                surface.surface.configure(&renderer.device, &surface.config);
                None
            },
            Err(wgpu::SurfaceError::OutOfMemory) => {
                exit.send(ExitEvent::error("Out of memory"));
                None
            },
            Err(error) => {
                log::warn!("Skipping frame due to {:?}", error);
                None
            }
        };

        if output.is_none() {
            frame.valid = false;
            frame.output = None;
            frame.view = None;
            return;
        }

        let output = output.unwrap();
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        frame.valid = true;
        frame.output = Some(output);
        frame.view = Some(view);
    }

    pub fn present_sys(mut frame: ResMut<Frame>) {
        if frame.valid {
            let output = frame.output.take().unwrap();
            output.present();

            frame.view = None;
        }
    }
}
