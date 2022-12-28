use crate::incl::*;

#[derive(Debug, TypeUuid)]
#[uuid = "df04defc-ea6d-47dd-83eb-f87086909935"]
pub struct Image {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
}

impl Image {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width, height,
            data: Vec::with_capacity((width as usize) * (height as usize)),
        }
    }

    pub fn from_memory(bytes: &[u8]) -> Result<Self, image::error::ImageError> {
        let image = image::load_from_memory(&bytes)?;

        let (width, height) = image.dimensions();
        let data = image.into_rgba8();

        Ok(Self {
            width, height,
            data: data.to_vec(),
        })
    }

    pub fn draw(&mut self, other: &Image, x: u32, y: u32) -> &mut Self {
        let src_row = (other.width.min(self.width - x) * 4) as usize;
        let dst_row = (self.width * 4) as usize;

        let end_y = (y + other.height).min(self.height) as usize;
        let y = y as usize;
        for src_y in y..end_y {
            let dst = &mut self.data[(dst_row * src_y)..(dst_row * src_y + src_row)];
            let src = &other.data[(src_row * (src_y - y))..(src_row * (src_y - (y - 1)))];
            dst.copy_from_slice(src);
        }

        self
    }
}

#[cfg(feature = "asset")]
pub struct ImageLoader;
#[cfg(feature = "asset")]
impl AssetLoader for ImageLoader {
    fn load(
        &self,
        reader: Arc<dyn AssetReader>, handle_path: Cow<'static, Path>,
        _: Option<Box<dyn AssetData>>,
        _: AssetLoadSync,
    ) -> Result<Box<dyn AssetDyn>, anyhow::Error> {
        let bytes = reader.read_file(&handle_path)?;
        let image = Image::from_memory(&bytes)?;

        Ok(Box::new(image))
    }
}
