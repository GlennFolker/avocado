use crate::incl::*;

#[derive(Debug, TypeUuid, Deref, DerefMut)]
#[uuid = "ac636f76-1e5a-48a0-8535-53a2d396efc8"]
pub struct Shader(pub wgpu::ShaderModule);

impl Shader {
    pub fn new<R: AsRef<str>>(renderer: &Renderer, source: R, label: Option<&str>) -> Self {
        Self(renderer.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label,
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(source.as_ref())),
        }))
    }
}

pub struct ShaderLoader;
impl AssetLoader for ShaderLoader {
    fn load(
        &self,
        reader: Arc<dyn AssetReader>, handle_path: Cow<'static, Path>,
        _: Option<Box<dyn AssetData>>,
        load_sync: AssetLoadSync,
    ) -> Result<Box<dyn AssetDyn>, anyhow::Error> {
        let bytes = reader.read_file(&handle_path)?;
        let source = String::from_utf8(bytes)?;

        let shader = Arc::new(RwLock::new(None));
        {
            let shader = Arc::clone(&shader);
            load_sync(Box::new(move |world| {
                let renderer = SystemState::<Res<Renderer>>::new(world).get(world);
                *shader.write() = Some(Shader::new(&renderer, source, Some(&handle_path.to_string_lossy())));
            }))?;
        }

        let shader = shader.write().take().unwrap();
        Ok(Box::new(shader))
    }
}
