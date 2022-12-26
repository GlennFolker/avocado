use crate::incl::*;

mod ext;
mod sys;
mod time;

pub use ext::AppExt as _;
pub use sys::*;
pub use time::*;

pub struct CoreSubsystem;
impl Subsystem for CoreSubsystem {
    fn init(app: &mut App) {
        app
            .init_res::<Time>()

            .stage(CoreStage::SysUpdate, SystemStage::parallel()
                .with_system(Time::update_sys
                    .label(CoreLabel::TimeUpdate)
                )
            )

            .stage(CoreStage::PreUpdate, SystemStage::parallel())
            .stage(CoreStage::Update, SystemStage::parallel())
            .stage(CoreStage::PostUpdate, SystemStage::parallel())
            .stage(CoreStage::SysPostUpdate, SystemStage::parallel());
    }
}
