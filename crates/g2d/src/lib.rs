use avocado_asset::prelude::*;
use avocado_core::prelude::*;
use avocado_winit::prelude::*;

use std::{
    borrow::Cow,
    path::Path,
};

mod atlas;
mod batch;
mod batch_conf;
mod vert;

pub use atlas::*;
pub use batch::*;
pub use batch_conf::*;
pub use vert::*;

pub mod prelude {
    pub use crate::{
        G2dSubsystem,
        TextureAtlas, TextureAtlasLoader, TextureAtlasData, AtlasRegion,
        Sprite, SpriteDesc, SpriteHolder,
        SpriteBatch, SpriteVertex, DefSpriteVertex,
    };
}

pub struct G2dSubsystem;
impl Subsystem for G2dSubsystem {
    fn init(app: &mut App) {
        let batch_shader = Shader::new(
            app.res::<Renderer>().unwrap(),
            include_str!("batch.wgsl"),
            Some("Sprite batch default shader"),
        );
        let batch_shader = app.res_mut::<Assets<Shader>>().unwrap()
            .add(Cow::Borrowed(Path::new("avocado/shaders/batch.wgsl")), batch_shader);

        app
            .insert_res(SpriteBatchDefShader(batch_shader))

            .asset::<TextureAtlas>()
            .asset_loader::<TextureAtlas>(TextureAtlasLoader);
    }
}
