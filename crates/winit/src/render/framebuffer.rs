use crate::{
    render::Texture,
    Renderer,
};

#[derive(Debug)]
pub struct FrameBuffer {
    pub colors: Vec<Texture>,
    pub width: u32,
    pub height: u32,
}

impl FrameBuffer {
    pub fn new(renderer: &Renderer, width: u32, height: u32) -> Self {
        Self {
            colors: vec![Self::create_color(renderer, width, height)],
            width, height,
        }
    }

    pub fn resize(&mut self, renderer: &Renderer, new_width: u32, new_height: u32) -> bool {
        if self.width != new_width || self.height != new_height {
            self.width = new_width;
            self.height = new_height;

            let len = self.colors.len();
            self.colors.clear();

            for _ in 0..len {
                self.colors.push(Self::create_color(renderer, new_width, new_height));
            }

            true
        } else {
            false
        }
    }

    pub fn create_color(renderer: &Renderer, width: u32, height: u32) -> Texture {
        let size = wgpu::Extent3d {
            width, height,
            depth_or_array_layers: 1,
        };

        let texture = renderer.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Frame buffer color attachment"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT,
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        Texture { texture, size, view, }
    }
}
