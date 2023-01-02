use avocado::incl::*;

#[derive(Debug, TypeUuid)]
#[uuid = "056db7a3-e928-4ccd-8f20-43fd30f7c465"]
struct Message(String);

struct MessageLoader;
impl AssetLoader for MessageLoader {
    fn load(
        &self,
        _: Arc<dyn AssetReader>, handle_path: Cow<'static, Path>,
        _: Option<Box<dyn AssetData>>,
        _: AssetLoadSync,
    ) -> Result<Box<dyn AssetDyn>, anyhow::Error> {
        Ok(Box::new(Message(handle_path.to_string_lossy().into_owned())))
    }
}

#[derive(Resource)]
struct ReceivedMessage(Handle<Message>);

fn main() {
    App::new()
        .set_runner(App::headless_runner())

        .init::<LogSubsystem>()
        .init::<CoreSubsystem>()
        .init::<AssetSubsystem>()

        .asset::<Message>()
        .asset_loader::<Message>(MessageLoader)

        .insert_res({
            let mut builder = AssetGraphBuilder::default();

            builder.node("first",
            |In(_): In<AssetGraphIn>, mut server: ResMut<AssetServer>| {
                let asset = server.load::<Message>(Path::new("AVocado")).as_dyn();
                Ok(vec![asset])
            });

            builder.node("first_a",
            |In(data): In<AssetGraphIn>, mut server: ResMut<AssetServer>, messages: Res<Assets<Message>>| {
                let handle = data["first"][0].clone_weak_typed()?;
                let message = messages.get(&handle).ok_or(AssetLoaderError::NoAsset)?;

                let asset = server.load::<Message>(Path::new(&format!("{} is terrible", &message.0)).to_path_buf()).as_dyn();
                Ok(vec![asset])
            });

            builder.node("first_b",
            |In(data): In<AssetGraphIn>, mut server: ResMut<AssetServer>, messages: Res<Assets<Message>>| {
                let handle = data["first"][0].clone_weak_typed()?;
                let message = messages.get(&handle).ok_or(AssetLoaderError::NoAsset)?;

                let asset = server.load::<Message>(Path::new(&format!("{} is disgusting", &message.0)).to_path_buf()).as_dyn();
                Ok(vec![asset])
            });

            builder.node("second",
            |In(data): In<AssetGraphIn>, mut commands: Commands, mut server: ResMut<AssetServer>, messages: Res<Assets<Message>>| {
                let a = data["first_a"][0].clone_weak_typed()?;
                let a = messages.get(&a).ok_or(AssetLoaderError::NoAsset)?;

                let b = data["first_b"][0].clone_weak_typed()?;
                let b = messages.get(&b).ok_or(AssetLoaderError::NoAsset)?;

                let asset = server.load::<Message>(Path::new(&format!("\n{}\n{}", &a.0, &b.0)).to_path_buf());
                commands.insert_resource(ReceivedMessage(asset.clone()));

                Ok(vec![asset.as_dyn()])
            });

            builder.edge("first", "first_a");
            builder.edge("first", "first_b");

            builder.edge("first_a", "second");
            builder.edge("first_b", "second");

            builder.build()
        })

        .sys(CoreStage::Update, done)
        .run();
}

fn done(
    done: EventReader<AssetGraphDoneEvent>,
    message: Option<Res<ReceivedMessage>>, messages: Res<Assets<Message>>,
    mut exit: EventWriter<ExitEvent>
) {
    if !done.is_empty() {
        done.clear();

        log::info!("{}", &messages.get(&message.unwrap().0).unwrap().0);
        exit.send(ExitEvent::graceful());
    }
}
