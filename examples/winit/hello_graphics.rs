use avocado::prelude::*;

fn main() {
    App::new()
        .init::<AVocado>()

        .sys(RenderStage::Begin, change_color)
        .run();
}

fn change_color(time: Res<Time>, mut color: ResMut<ClearColor>) {
    **color = Color::rgb(1.0, 0.0, 0.0).shift_hue((time.elapsed_sec() * 12.) as f32);
}
