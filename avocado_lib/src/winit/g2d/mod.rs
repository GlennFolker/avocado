use crate::incl::*;

mod atlas;
mod batch;
mod batch_conf;
mod vert;

pub use atlas::*;
pub use batch::*;
pub use batch_conf::*;
pub use vert::*;

pub struct Winit2dSubsystem;
impl Subsystem for Winit2dSubsystem {
    fn init(app: &mut App) {
        let batch_shader = Shader::new(
            app.res::<Renderer>().unwrap(),
            include_str!("batch.wgsl"),
            Some("Sprite batch default shader")
        );
        let batch_shader = app.res_mut::<Assets<Shader>>().unwrap()
            .add(Cow::Borrowed(Path::new("avocado/shaders/batch.wgsl")), batch_shader);

        app
            .insert_res(SpriteBatchDefShader(batch_shader))

            .asset::<TextureAtlas>()
            .asset_loader::<TextureAtlas>(TextureAtlasLoader);
    }
}
