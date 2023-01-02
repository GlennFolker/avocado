use crate::incl::*;

// TODO terrible
#[derive(Debug)]
pub struct FrameBuffer {
    pub color_attachments: Vec<(wgpu::Texture, wgpu::TextureView)>,
    pub width: u32,
    pub height: u32,
}

impl FrameBuffer {
    pub fn new(renderer: &Renderer, width: u32, height: u32) -> Self {
        Self {
            color_attachments: vec![Self::create_color_attachment(renderer, width, height)],
            width, height,
        }
    }

    pub fn resize(&mut self, renderer: &Renderer, new_width: u32, new_height: u32) -> bool {
        if self.width != new_width || self.height != new_height {
            self.width = new_width;
            self.height = new_height;

            let len = self.color_attachments.len();
            self.color_attachments.clear();

            for _ in 0..len {
                self.color_attachments.push(Self::create_color_attachment(renderer, new_width, new_height));
            }

            true
        } else {
            false
        }
    }

    pub fn create_color_attachment(renderer: &Renderer, width: u32, height: u32) -> (wgpu::Texture, wgpu::TextureView) {
        let texture = renderer.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Frame buffer color attachment"),
            size: wgpu::Extent3d {
                width, height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT,
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        (texture, view)
    }
}
