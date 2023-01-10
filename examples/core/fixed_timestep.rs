use avocado::prelude::*;

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
        .init::<LogSubsystem>()
        .init::<CoreSubsystem>()

        .init_res::<FPS>()
        .fixed_timestep_sec::<PollUpdate>(CoreStage::Update, Poll, SystemStage::parallel(), 1.0)

        .sys(CoreStage::Update, incr)
        .sys(Poll, poll.run_if(FixedUpdate::qualified_sys::<PollUpdate>))
        .sys(Poll, exit
            .run_if(FixedUpdate::qualified_sys::<PollUpdate>)
            .run_if(should_exit)
        )

        .run();
}

fn incr(mut frame: ResMut<FPS>) {
    **frame += 1;
}

fn poll(mut frame: ResMut<FPS>) {
    log::info!("FPS: {}", frame.0);
    **frame = 0;
}

fn exit(mut exit_event: EventWriter<ExitEvent>) {
    exit_event.send(ExitEvent::graceful());
}

fn should_exit(mut count: Local<u8>) -> bool {
    *count += 1;
    *count >= 5
}
