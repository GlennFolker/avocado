use avocado::incl::*;

#[derive(Resource, Default, Deref, DerefMut)]
struct AtlasHandle(Option<Handle<TextureAtlas>>);

#[derive(SystemLabel)]
struct SpriteBatchLabel;

fn main() {
    App::new()
        .init::<AVocado>()
        .init_res::<AtlasHandle>()

        .insert_res({
            let mut builder = AssetGraphBuilder::default();

            builder.node("ball", load_ball);
            builder.node("atlas", load_atlas);
            builder.edge("ball", "atlas");

            builder.build()
        })

        .render_node::<_, SpriteBatch<1>>(SpriteBatchLabel)

        .sys(CoreStage::Update, check)
        .sys(RenderStage::Begin, resize)
        .run();
}

fn load_ball(_: In<AssetGraphIn>, mut server: ResMut<AssetServer>) -> AssetGraphResult {
    let handle = server.load::<Image>(Path::new("ball.png"));
    Ok(vec![handle.as_dyn()])
}

fn load_atlas(
    In(parents): In<AssetGraphIn>,
    mut atlas: ResMut<AtlasHandle>,
    renderer: Res<Renderer>, mut server: ResMut<AssetServer>,
) -> AssetGraphResult {
    let max_size = renderer.device.limits().max_texture_dimension_2d;
    let data = TextureAtlasData {
        min_width: 256,
        min_height: 256,
        max_width: max_size,
        max_height: max_size,
        categories: {
            let mut map = HashMap::default();
            let mut images = Vec::with_capacity(parents["ball"].len());
            for handle in &parents["ball"] {
                let image = handle.clone_typed()?;
                images.push(image);
            }

            map.insert("sprites".to_string(), (SamplerDesc::default(), images));
            map
        },
    };

    let handle = server.load_with(Path::new("atlas"), Some(data));
    **atlas = Some(handle.clone());

    Ok(vec![handle.as_dyn()])
}

fn check(
    events: EventReader<AssetGraphDoneEvent>,
    mut commands: Commands,
    mut atlas: ResMut<AtlasHandle>, mut atlases: ResMut<Assets<TextureAtlas>>,

    mut global_camera: ResMut<GlobalCamera>, window: NonSend<WinitWindow>,
) {
    if !events.is_empty() {
        events.clear();

        let handle = atlas.take().unwrap();
        let atlas = atlases.remove(handle).unwrap();

        let winit::PhysicalSize { width, height, } = window.inner_size().cast::<f32>();
        global_camera.entity = commands.spawn(Camera {
            position: Vec3 { x: -0., y: -0., z: 10., },
            near: -5.,
            far: 5.,
            proj: CameraProj::Orthographic { width, height, },
        }).id();

        commands.spawn(SpriteHolder {
            sprites: vec![Sprite {
                region: atlas.region(Path::new("ball.png")),
                color: Color::rgb(1., 1., 1.),
                trns: SpriteTransform {
                    x: -32.,
                    y: -32.,
                    z: 0.,

                    pivot_x: 32.,
                    pivot_y: 32.,

                    width: 64.,
                    height: 64.,

                    rotation: 0.,
                },
                mask: !0,
            }]
        });

        commands.insert_resource(atlas);
    }
}

fn resize(
    mut graph: ResMut<RenderGraph>, renderer: Res<Renderer>, mut cameras: Query<&mut Camera>,
    window: NonSend<WinitWindow>,
) {
    let winit::PhysicalSize { width, height, } = window.inner_size();
    graph.output_mut(SpriteBatchLabel).buffer.resize(&renderer, width, height);

    let (width, height) = (width as f32, height as f32);
    for mut camera in &mut cameras {
        camera.proj = CameraProj::Orthographic { width, height, };
    }
}
