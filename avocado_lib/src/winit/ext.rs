use crate::incl::*;

pub trait ColorExt: Into<wgpu::Color> {}
impl ColorExt for Color {}

impl Into<wgpu::Color> for Color {
    fn into(self) -> wgpu::Color {
        wgpu::Color {
            r: self.r as f64,
            g: self.g as f64,
            b: self.b as f64,
            a: self.a as f64,
        }
    }
}

pub trait AppExt {
    fn render_node<RenderParam, Desc>(&mut self, label: impl SystemLabel) -> &mut Self where
        RenderParam: 'static + SystemParam,
        <RenderParam as SystemParam>::Fetch: ReadOnlySystemParamFetch,

        Desc: RenderNodeDesc<RenderParam = RenderParam>;

    fn render_edge(&mut self, parent_label: impl SystemLabel, child_label: impl SystemLabel) -> &mut Self;
}

impl AppExt for App {
    fn render_node<RenderParam, Desc>(&mut self, label: impl SystemLabel) -> &mut Self where
        RenderParam: 'static + SystemParam,
        <RenderParam as SystemParam>::Fetch: ReadOnlySystemParamFetch,

        Desc: RenderNodeDesc<RenderParam = RenderParam>,
    {
        let (world, schedule) = self.unzip_mut();
        world.resource_scope(|world, mut graph: Mut<RenderGraph>| {
            graph.node::<RenderParam, Desc>(world, schedule, label);
        });

        self
    }

    fn render_edge(&mut self, parent_label: impl SystemLabel, child_label: impl SystemLabel) -> &mut Self {
        self.res_mut::<RenderGraph>().unwrap().edge(parent_label, child_label);
        self
    }
}
