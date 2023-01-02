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
pub use ext::{
    ColorExt as _,
    AppExt as _,
};
pub use frame::*;
pub use res::*;
pub use runner::*;
pub use sys::*;

pub struct WinitSubsystem;
impl Subsystem for WinitSubsystem {
    fn init(app: &mut App) {
        WinitRunner::init(app)
            .set_runner(WinitRunner::run)

            .stage(RenderStage::Begin, SystemStage::parallel()
                .with_system(Frame::prepare_sys.label(RenderLabel::PrepareFrame))
                .with_system(Frame::init_sys
                    .label(RenderLabel::InitFrame)
                    .after(RenderLabel::PrepareFrame)
                )
                .with_system(GlobalCamera::update_sys
                    .label(RenderLabel::ComputeGlobalCamera)
                )
            )

            .stage(RenderStage::Queue, SystemStage::parallel())

            .stage(RenderStage::Render, SystemStage::parallel()
                .with_system(RenderGraph::render_sys.at_end())
            )

            .stage(RenderStage::End, SystemStage::parallel()
                .with_system(Frame::present_sys.label(RenderLabel::PresentFrame))
                .with_system(WindowConfig::visible_sys
                    .with_run_criteria(ShouldRun::once)
                    .after(RenderLabel::PresentFrame)
                )
            )

            .init_res::<Frame>()
            .init_res::<RenderGraph>()
            .init_res::<GlobalCamera>()

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
