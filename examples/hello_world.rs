use avocado::incl::*;

fn main() {
    #[derive(StageLabel)]
    struct Runtime;

    App::new()
        .init::<LogSubsystem>()

        .stage(Runtime, SystemStage::parallel())
        .sys(Runtime, hello_world)

        .run()
}

fn hello_world() {
    log::info!("Hello world!");
}
