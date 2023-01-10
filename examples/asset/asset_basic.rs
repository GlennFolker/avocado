use avocado::prelude::*;
use std::{
    borrow::Cow,
    path::Path,
    sync::Arc,
};

#[derive(Debug, TypeUuid, Deref, DerefMut)]
#[uuid = "ae58f163-e781-4a7e-9f6a-8258a5bd672d"]
struct SecretAsset(String);

struct SecretAssetLoader;
impl AssetLoader for SecretAssetLoader {
    fn load(
        &self,
        reader: Arc<dyn AssetReader>, handle_path: Cow<'static, Path>,
        _: Option<Box<dyn AssetData>>,
        _: AssetLoadSync,
    ) -> Result<Box<dyn AssetDyn>, anyhow::Error> {
        let data = reader.read_file(&handle_path)?;
        let message = String::from_utf8(data)?;

        Ok(Box::new(SecretAsset(message)))
    }
}

#[derive(Resource, Deref, DerefMut)]
struct Secret(Handle<SecretAsset>);

fn main() {
    App::new()
        .init::<LogSubsystem>()
        .init::<CoreSubsystem>()
        .init::<AssetSubsystem>()

        .asset::<SecretAsset>()
        .asset_loader::<SecretAsset>(SecretAssetLoader {})

        .startup_sys(startup)
        .sys(CoreStage::Update, check)

        .run();
}

fn startup(mut commands: Commands, mut server: ResMut<AssetServer>) {
    let handle = server.load::<SecretAsset>(Path::new("secret_message.txt"));
    commands.insert_resource(Secret(handle));
}

fn check(assets: Res<Assets<SecretAsset>>, secret: Res<Secret>, mut exit: EventWriter<ExitEvent>) {
    if let Some(asset) = assets.get(&secret) {
        log::info!("{}", &**asset);
        exit.send(ExitEvent::graceful());
    }
}
