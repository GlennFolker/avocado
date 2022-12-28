use crate::incl::*;

#[cfg(feature = "winit_2d")]
mod g2d;
#[cfg(feature = "winit_2d")]
pub use g2d::*;

pub mod render;

mod config;
mod event;
mod ext;
mod frame;
mod res;
mod runner;
mod sys;

pub use config::*;
pub use event::*;
pub use ext::{};
pub use frame::*;
pub use res::*;
pub use runner::*;
pub use sys::*;

pub struct WinitSubsystem;
impl Subsystem for WinitSubsystem {
    fn init(app: &mut App) {
        app
            .set_runner(WinitRunner::run)

            .stage(RenderStage::Begin, SystemStage::parallel()
                .with_system(Frame::prepare_sys.label(RenderLabel::PrepareFrame))
                .with_system(Frame::init_sys
                    .label(RenderLabel::InitFrame)
                    .after(RenderLabel::PrepareFrame)
                )
            )

            .stage(RenderStage::Queue, SystemStage::parallel())
            .stage(RenderStage::Render, SystemStage::parallel())

            .stage(RenderStage::End, SystemStage::parallel()
                .with_system(Frame::present_sys.label(RenderLabel::PresentFrame))
                .with_system(WindowConfig::visible_sys
                    .run_if(run_once)
                    .after(RenderLabel::PresentFrame)
                )
            )

            .init_res::<Frame>()

            .event::<WindowResizedEvent>()
            .event::<WindowMovedEvent>()
            .event::<SuspendEvent>()
            .event::<ResumeEvent>()

            .asset::<Texture>()
            .asset_loader::<Texture>(TextureLoader);

        #[cfg(feature = "winit_2d")]
        app.init::<Winit2dSubsystem>();
    }
}
