use crate::incl::*;

#[derive(Debug, Resource, Default, TypeUuid)]
#[uuid = "842a1988-c51e-457b-aea5-f4e0893e79b1"]
pub struct TextureAtlas {
    pub pages: Vec<Handle<Texture>>,
    pub samplers: Vec<wgpu::Sampler>,

    pub mapping: HashMap<PathBuf, AtlasRegion>,
    pub sampler_mapping: HashMap<usize, usize>,
}

impl TextureAtlas {
    pub fn region(&self, path: &Path) -> AtlasRegion {
        self.mapping[path]
    }
}

#[derive(Debug, Copy, Clone)]
pub struct AtlasRegion {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,

    pub page_index: usize,
    pub u: f32,
    pub u2: f32,
    pub v: f32,
    pub v2: f32,
}

#[derive(Debug)]
pub struct Sprite<T: SpriteVertex> {
    pub region: AtlasRegion,
    pub color: Color,
    pub desc: SpriteDesc<T>,
}

#[derive(Debug)]
pub enum SpriteDesc<T: SpriteVertex> {
    Direct {
        z: f32,
        vertices: SmallVec<[T; 4]>,
        indices: SmallVec<[u16; 6]>,
    },
    Transform {
        pos: Vec2,
        z: f32,
        anchor: Vec2,
        size: Vec2,
        rotation: f32,
        data: T::Data,
    },
}

impl<T: SpriteVertex> SpriteDesc<T> {
    #[inline]
    pub fn z(&self) -> f32 {
        match self {
            Self::Direct { z, .. } => *z,
            Self::Transform { z, .. } => *z,
        }
    }

    #[inline]
    pub fn vert_len(&self) -> u16 {
        (match self {
            Self::Direct { vertices, .. } => vertices.len(),
            Self::Transform { .. } => 4,
        }) as u16
    }

    #[inline]
    pub fn ind_len(&self) -> u32 {
        (match self {
            Self::Direct { indices, .. } => indices.len(),
            Self::Transform { .. } => 6,
        }) as u32
    }
}

#[derive(Component)]
pub struct SpriteHolder<T: SpriteVertex> {
    pub sprites: Vec<Sprite<T>>,
}

/// Necessary data for texture atlas building. Given image handles must be strong otherwise a panic is expected.
#[derive(Debug)]
pub struct TextureAtlasData {
    pub min_width: u32,
    pub min_height: u32,
    pub max_width: u32,
    pub max_height: u32,
    pub categories: HashMap<String, (SamplerDesc, Vec<Handle<Image>>)>,
}

pub struct TextureAtlasLoader;
impl AssetLoader for TextureAtlasLoader {
    fn load(
        &self,
        _: Arc<dyn AssetReader>, handle_path: Cow<'static, Path>,
        data: Option<Box<dyn AssetData>>,
        load_sync: AssetLoadSync,
    ) -> Result<Box<dyn AssetDyn>, anyhow::Error> {
        let data = data
            .ok_or(AssetLoaderError::NoData)?
            .downcast::<TextureAtlasData>().or(Err(AssetLoaderError::WrongType))?;

        let mapped = Arc::new(RwLock::new(HashMap::default()));
        {
            let mut categories = data.categories;
            let mapped = Arc::clone(&mapped);

            load_sync(Box::new(move |world| {
                let assets = SystemState::<Res<Assets<Image>>>::new(world).get(world);
                for (group, (desc, images)) in categories.drain() {
                    mapped.write().insert(group, (desc, images
                        .iter()
                        .map(|handle| (handle.path().to_path_buf(), assets.get(&handle).unwrap().clone()))
                        .collect::<Vec<_>>()
                    ));
                }
            }))?;
        }

        let mut packer = BinPack::<String, PathBuf>::new(data.min_width, data.min_height, data.max_width, data.max_height);
        let mut mapped = mapped.write();

        for (group, (_, images)) in &*mapped {
            packer.group(group.clone());
            for (path, image) in images {
                packer.insert(&group, path.clone(), image.width, image.height)?;
            }
        }

        let mut pages = vec![];
        let mut sampler_descs = vec![];
        let mut mapping = HashMap::default();
        let mut sampler_mapping = HashMap::default();

        let mut bins = packer.finish();
        for (group, mut bins) in bins.drain() {
            let (desc, mut images) = mapped.remove(&group).ok_or(AssetLoaderError::Other("Group not found".to_string()))?;
            sampler_descs.push(desc);

            for bin in bins.drain(..) {
                let page_width = bin.width();
                let page_height = bin.height();

                let mut page = Image::new(page_width, page_height);
                for (path, image) in images.drain(..) {
                    let rect = bin
                        .get(&path)
                        .ok_or(AssetLoaderError::Other(format!("{:?} not found", &path)))?;

                    page.draw(&image, rect.x, rect.y);
                    mapping.insert(path, AtlasRegion {
                        x: rect.x,
                        y: rect.y,
                        width: rect.width,
                        height: rect.height,

                        page_index: pages.len(),
                        u: (rect.x as f32) / (page_width as f32),
                        u2: ((rect.x + rect.width) as f32) / (page_width as f32),
                        v: (rect.y as f32) / (page_height as f32),
                        v2: ((rect.y + rect.height) as f32) / (page_height as f32),
                    });
                }

                pages.push(page);
                sampler_mapping.insert(pages.len() - 1, sampler_descs.len() - 1);
            }
        }

        let textures = Arc::new(RwLock::new(Vec::with_capacity(pages.len())));
        let samplers = Arc::new(RwLock::new(Vec::with_capacity(sampler_descs.len())));

        {
            let textures = Arc::clone(&textures);
            let samplers = Arc::clone(&samplers);

            let mut pages = pages;
            let mut sampler_descs = sampler_descs;
            load_sync(Box::new(move |world| {
                let (renderer, mut assets) = SystemState::<(
                    Res<Renderer>,
                    ResMut<Assets<Texture>>
                )>::new(world).get_mut(world);

                let mut i = 0;
                for page in pages.drain(..) {
                    let path = handle_path
                        .parent().unwrap_or(Path::new(""))
                        .join(format!("{}#page{}", handle_path.to_string_lossy(), i));

                    i += 1;

                    let tex = Texture::from_image(&renderer, &page, None);
                    let handle = assets.add(Cow::Owned(path), tex);
                    textures.write().push(handle);
                }

                for desc in sampler_descs.drain(..) {
                    let sampler = desc.create_sampler(&renderer);
                    samplers.write().push(sampler);
                }
            }))?;
        }

        let pages = {
            let mut textures = textures.write();

            let mut pages = Vec::with_capacity(textures.len());
            pages.append(&mut textures);
            pages
        };

        let samplers = {
            let mut samplers = samplers.write();

            let mut inner = Vec::with_capacity(samplers.len());
            inner.append(&mut samplers);
            inner
        };

        Ok(Box::new(TextureAtlas { pages, samplers, mapping, sampler_mapping, }))
    }
}
