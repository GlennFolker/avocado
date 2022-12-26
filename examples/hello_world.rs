use avocado::incl::*;

fn main() {
    App::new()
        .init::<LogSubsystem>()
        .run();
}
