use avocado::prelude::*;
use winit::dpi::PhysicalSize;
use std::path::Path;

#[derive(Resource, Default, Deref, DerefMut)]
struct AtlasHandle(Option<Handle<TextureAtlas>>);

#[derive(SystemLabel)]
struct SpriteBatchLabel;

#[derive(StageLabel)]
struct Poll;

#[derive(Resource, Deref, DerefMut)]
struct PollUpdate(FixedUpdate);
impl FixedUpdateWrap for PollUpdate {
    fn new(_: &mut World, updater: FixedUpdate) -> Self {
        Self(updater)
    }
}

#[derive(Resource, Default, Deref, DerefMut)]
struct FPS(usize);

fn main() {
    App::new()
        .init::<AVocado>()
        .init_res::<AtlasHandle>()

        .init::<SpriteBatch<DefSpriteVertex>>()

        .insert_res({
            let mut builder = AssetGraphBuilder::default();

            builder.node("ball", load_ball);
            builder.node("atlas", load_atlas);
            builder.edge("ball", "atlas");

            builder.build()
        })

        .init_res::<FPS>()
        .fixed_timestep_sec::<PollUpdate>(CoreStage::Update, Poll, SystemStage::parallel(), 1.0)
        .sys(CoreStage::Update, incr)
        .sys(Poll, poll.run_if(FixedUpdate::qualified_sys::<PollUpdate>))

        .sys(CoreStage::Update, check)
        .sys(CoreStage::Update, behave)
        .sys(RenderStage::Begin, resize
            .label(RenderLabel::InitFrame)
            .after(RenderLabel::PrepareFrame)
        )

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

fn incr(mut frame: ResMut<FPS>) {
    **frame += 1;
}

fn poll(mut frame: ResMut<FPS>) {
    log::info!("FPS: {}", **frame);
    **frame = 0;
}

fn check(
    events: EventReader<AssetGraphDoneEvent>,
    mut commands: Commands,
    mut atlas: ResMut<AtlasHandle>, mut atlases: ResMut<Assets<TextureAtlas>>,
    mut graph: ResMut<RenderGraph>, mut global_camera: ResMut<GlobalCamera>,
    renderer: Res<Renderer>, surface: Res<SurfaceConfig>,
) {
    if !events.is_empty() {
        events.clear();

        let handle = atlas.take().unwrap();
        let atlas = atlases.remove(handle).unwrap();

        let PhysicalSize { width, height, } = surface.size.cast::<f32>();
        global_camera.entity = commands.spawn(Camera {
            position: Vec3 { x: -0., y: -0., z: 10., },
            near: -5.,
            far: 5.,
            proj: CameraProj::Orthographic { width, height, },
        }).id();

        commands.spawn(SpriteHolder {
            sprites: vec![Sprite::<DefSpriteVertex> {
                region: atlas.region(Path::new("ball.png")),
                color: Color::rgb(1., 1., 1.),
                desc: SpriteDesc::Transform {
                    pos: Vec2::splat(-64.),
                    z: 0.,
                    anchor: Vec2::splat(64.),
                    size: Vec2::splat(128.),
                    rotation: 0.,
                    data: (),
                },
            }]
        });

        commands.insert_resource(atlas);

        graph.node::<SpriteBatch<DefSpriteVertex>>(SpriteBatchLabel, &renderer);
    }
}

fn behave(time: Res<Time>, mut holders: Query<&mut SpriteHolder<DefSpriteVertex>>) {
    let (sin, cos) = (time.elapsed_sec() as f32).sin_cos();
    let new_pos = Vec2 { x: cos * 256. - 64., y: sin * 256. - 64. };
    let delta = time.delta_sec() as f32 * 60.;

    for mut holder in &mut holders {
        for sprite in &mut holder.sprites {
            let SpriteDesc::Transform {
                pos,
                rotation,
                ..
            } = &mut sprite.desc else {
                continue;
            };

            *pos = new_pos;
            *rotation += delta;
        }
    }
}

fn resize(
    mut graph: ResMut<RenderGraph>, renderer: Res<Renderer>, mut cameras: Query<&mut Camera>,
    surface: Res<SurfaceConfig>,
) {
    let PhysicalSize { width, height, } = surface.size;
    if let Some(output) = graph.output_mut(SpriteBatchLabel) {
        output.buffer.resize(&renderer, width, height);
    }

    let (width, height) = (width as f32, height as f32);
    for mut camera in &mut cameras {
        camera.proj = CameraProj::Orthographic { width, height, };
    }
}
