use avocado_asset::prelude::*;
use avocado_core::prelude::*;
use avocado_graphics::prelude::*;
use avocado_utils::prelude::*;

use crate::Renderer;
use std::{
    borrow::Cow,
    num::NonZeroU32,
    path::Path,
    sync::Arc,
};

#[derive(Debug, TypeUuid)]
#[uuid = "3552c34a-4292-42ca-892c-d598c99bef4e"]
pub struct Texture {
    pub texture: wgpu::Texture,
    pub size: wgpu::Extent3d,
    pub view: wgpu::TextureView,
}

impl Texture {
    pub fn from_image(renderer: &Renderer, image: &Image, label: Option<&str>) -> Texture {
        let size = wgpu::Extent3d {
            width: image.width,
            height: image.height,
            depth_or_array_layers: 1,
        };

        let texture = renderer.device.create_texture(&wgpu::TextureDescriptor {
            label, size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        });

        renderer.queue.write_texture(
            wgpu::ImageCopyTexture {
                aspect: wgpu::TextureAspect::All,
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            &image.data,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: NonZeroU32::new(4 * image.width),
                rows_per_image: NonZeroU32::new(image.height),
            },
            size,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        Self { texture, size, view, }
    }
}

/// Optional texture data for loading textures. Given image handle, if any, must be strong otherwise a panic is expected.
#[derive(Debug, Default)]
pub struct TextureData {
    pub image: Option<Handle<Image>>,
}

pub struct TextureLoader;
impl AssetLoader for TextureLoader {
    fn load(
        &self,
        reader: Arc<dyn AssetReader>, handle_path: Cow<'static, Path>,
        data: Option<Box<dyn AssetData>>,
        load_sync: AssetLoadSync,
    ) -> Result<Box<dyn AssetDyn>, anyhow::Error> {
        let data = data
            .unwrap_or_else(|| Box::new(TextureData::default())).downcast::<TextureData>()
            .or(Err(AssetLoaderError::WrongType))?;

        let result = Arc::new(RwLock::new(None));
        if let Some(handle) = data.image {
            let result = Arc::clone(&result);
            load_sync(Box::new(move |world| {
                let (assets, renderer) = SystemState::<(
                    Res<Assets<Image>>,
                    Res<Renderer>,
                )>::new(world).get(world);

                *result.write() = Some(Texture::from_image(&renderer, &assets.get(&handle).unwrap(), None));
            }))?;
        } else {
            let result = Arc::clone(&result);
            let bytes = reader.read_file(&handle_path)?;
            let image = Image::from_memory(&bytes)?;

            load_sync(Box::new(move |world| {
                let renderer = SystemState::<Res<Renderer>>::new(world).get(world);
                *result.write() = Some(Texture::from_image(&renderer, &image, None));
            }))?;
        }

        let texture = result.write().take().unwrap();
        Ok(Box::new(texture))
    }
}
