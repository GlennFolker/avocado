use crate::{
    asset::prelude::*,
    core::prelude::*,
};

mod camera;
mod config;
mod event;
mod ext;
mod frame;
mod framebuffer;
mod graph;
mod res;
mod runner;
mod sampler;
mod shader;
mod sys;
mod texture;

pub use camera::*;
pub use config::*;
pub use event::*;
pub use ext::*;
pub use frame::*;
pub use framebuffer::*;
pub use graph::*;
pub use res::*;
pub use runner::*;
pub use sampler::*;
pub use shader::*;
pub use sys::*;
pub use texture::*;

pub mod re_exports {
    pub use ::wgpu;
    pub use ::winit;
}

pub mod prelude {
    pub use crate::winit::{
        re_exports::*,
        RenderGraph,
        RenderNodeDesc, RenderInput, RenderOutput,
        Camera, GlobalCamera, CameraProj,
        Texture, Shader, SamplerDesc, FrameBuffer,
        WinitSubsystem,
        WindowConfig, WindowPosition, ClearColor,
        Renderer, SurfaceConfig, Frame,
        RenderStage, RenderLabel,
        ColorExt as _, KeyCodeExt as _,
    };
}

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
                .with_system(RenderGraph::begin_sys.at_end())
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
            .asset::<Shader>()
            .asset_loader::<Texture>(TextureLoader)
            .asset_loader::<Shader>(ShaderLoader);
    }
}
