use avocado::prelude::*;

fn main() {
    App::new()
        .init::<LogSubsystem>()
        .init::<CoreSubsystem>()

        .sys(CoreStage::Update, hello_world)
        .run()
}

fn hello_world(mut exit: EventWriter<ExitEvent>) {
    log::info!("Hello world!");
    exit.send(ExitEvent::graceful());
}
