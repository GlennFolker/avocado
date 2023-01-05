use avocado::prelude::*;

#[derive(Resource, Default)]
struct FPS(usize);

#[derive(StageLabel)]
struct Poll;

const INTERVAL: f64 = 1.0;
const SECS: u64 = Time::secs(INTERVAL);
const NANOS: u32 = Time::nanos(INTERVAL);
type PollUpdate = FixedUpdate<SECS, NANOS>;

fn main() {
    App::new()
        .init::<LogSubsystem>()
        .init::<CoreSubsystem>()

        .init_res::<FPS>()
        .fixed_timestep::<SECS, NANOS>(CoreStage::Update, Poll, SystemStage::parallel())

        .sys(CoreStage::Update, incr)
        .sys(Poll, poll.run_if(PollUpdate::qualified_sys))
        .sys(Poll, exit
            .run_if(PollUpdate::qualified_sys)
            .run_if(should_exit)
        )

        .run();
}

fn incr(mut frame: ResMut<FPS>) {
    frame.0 += 1;
}

fn poll(mut frame: ResMut<FPS>) {
    log::info!("FPS: {}", frame.0);
    frame.0 = 0;
}

fn exit(mut exit_event: EventWriter<ExitEvent>) {
    exit_event.send(ExitEvent::graceful());
}

fn should_exit(mut count: Local<u8>) -> bool {
    *count += 1;
    *count >= 5
}
