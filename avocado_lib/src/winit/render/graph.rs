use crate::incl::*;

pub struct RenderInput {
    pub output: &'static mut RenderOutput,
    pub parent_outputs: HashMap<SystemLabelId, &'static RenderOutput>,
}

impl RenderInput {
    pub fn parent(&self, label: impl SystemLabel) -> &'static RenderOutput {
        self.parent_outputs[&label.as_label()]
    }
}

pub struct RenderOutput {
    pub buffer: FrameBuffer,
}

impl RenderOutput {
    pub fn new(renderer: &Renderer) -> Self {
        Self {
            buffer: FrameBuffer::new(renderer, 2, 2),
        }
    }
}

struct OutputChannel {
    sender: SenderAsync<HashMap<SystemLabelId, &'static RenderOutput>>,
    receiver: ReceiverAsync<HashMap<SystemLabelId, &'static RenderOutput>>,
}

impl Default for OutputChannel {
    fn default() -> Self {
        let (sender, receiver) = async_channel::unbounded();
        Self { sender, receiver, }
    }
}

struct DoneChannel {
    sender: SenderAsync<usize>,
    receiver: ReceiverAsync<usize>,
}

impl Default for DoneChannel {
    fn default() -> Self {
        let (sender, receiver) = async_channel::unbounded();
        Self { sender, receiver, }
    }
}

struct RenderState {
    screen: wgpu::Buffer,
    bind_layout: wgpu::BindGroupLayout,
    bind_group: Option<wgpu::BindGroup>,
    pipeline: wgpu::RenderPipeline,
    sampler: wgpu::Sampler,
}

impl RenderState {
    fn new(renderer: &Renderer, surface: &SurfaceConfig) -> Self {
        #[repr(C)]
        #[derive(Copy, Clone, Pod, Zeroable)]
        struct QuadVertex {
            pos: [f32; 2],
            uv: [f32; 2],
        }

        impl QuadVertex {
            const fn new(x: f32, y: f32, u: f32, v: f32) -> Self {
                Self {
                    pos: [x, y],
                    uv: [u, v],
                }
            }
        }

        let screen = renderer.device.create_buffer_init(&wgpu::BufferInitDescriptor {
            label: Some("Render graph output quad"),
            contents: bytemuck::cast_slice(&[
                QuadVertex::new(-1., 1., 0., 0.),
                QuadVertex::new(-1., -1., 0., 1.),
                QuadVertex::new(1., -1., 1., 1.),
                QuadVertex::new(1., -1., 1., 1.),
                QuadVertex::new(1., 1., 1., 0.),
                QuadVertex::new(-1., 1., 0., 0.),
            ]),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let shader = renderer.device.create_shader_module(include_wgsl!("screen_quad.wgsl"));

        let bind_layout = renderer.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Render graph texture attachment layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    multisampled: false,
                    view_dimension: wgpu::TextureViewDimension::D2,
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                },
                count: None,
            }, wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            }],
        });

        let bind_group = None;

        let pipeline = renderer.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render graph output pipeline"),
            layout: Some(&renderer.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&bind_layout],
                push_constant_ranges: &[],
            })),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: mem::size_of::<QuadVertex>() as wgpu::BufferAddress,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &vertex_attr_array![
                        0 => Float32x2,
                        1 => Float32x2,
                    ],
                }],
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            fragment: Some(wgpu::FragmentState { // 3.
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState { // 4.
                    format: surface.config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview: None,
        });

        let sampler = SamplerDesc::default().create_sampler(renderer);
        Self { screen, bind_layout, bind_group, pipeline, sampler, }
    }

    fn update_bind_group(
        &mut self,
        renderer: &Renderer,
        view: &wgpu::TextureView,
    ) {
        self.bind_group = Some(renderer.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Render graph texture attachment"),
            layout: &self.bind_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(view),
            }, wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&self.sampler),
            }]
        }));
    }
}

#[derive(Resource, Default)]
pub struct RenderGraph {
    nodes: Vec<Box<dyn RenderNodeDyn>>,
    labels: HashMap<SystemLabelId, usize>,
    done_channel: DoneChannel,

    ran: HashSet<usize>,
    queued: HashSet<usize>,
    running: usize,

    state: Option<RenderState>,
}

impl RenderGraph {
    pub fn render_sys(world: &mut World) {
        world.resource_scope(|world, graph: Mut<Self>| unsafe {
            let graph = mem::transmute::<_, &'static mut Self>(graph.into_inner());
            graph.render(world);
        });
    }

    pub fn output(&self, label: impl SystemLabel) -> &RenderOutput {
        self.nodes[self.labels[&label.as_label()]].output()
    }

    pub fn output_mut(&mut self, label: impl SystemLabel) -> &mut RenderOutput {
        self.nodes[self.labels[&label.as_label()]].output_mut()
    }

    pub fn node<RenderParam, Desc>(
        &mut self, world: &mut World, schedule: &mut Schedule,
        label: impl SystemLabel,
    ) where
        RenderParam: 'static + SystemParam,
        <RenderParam as SystemParam>::Fetch: ReadOnlySystemParamFetch,

        Desc: RenderNodeDesc<RenderParam = RenderParam>,
    {
        let label = label.as_label();
        let index = self.nodes.len();

        if self.labels.insert(label, index).is_some() {
            panic!("Render node {:?} is already occupied.", label);
        }

        let mut node = RenderNode::<RenderParam, Desc>::new(
            label,
            world.resource::<Renderer>(),
            self.done_channel.sender.clone(),
        );

        node.render_sys.initialize(world);

        Desc::init(world, schedule);
        self.nodes.push(Box::new(node));
    }

    pub fn edge(&mut self, parent_label: impl SystemLabel, child_label: impl SystemLabel) {
        let parent_label = parent_label.as_label();
        let child_label = child_label.as_label();

        let Some(parent) = self.labels.get(&parent_label) else {
            panic!("Render node {:?} is not registered", parent_label);
        };

        let Some(child) = self.labels.get(&child_label) else {
            panic!("Render node {:?} is not registered", child_label);
        };

        let mut queue = VecDeque::<usize>::new();
        let mut iterated = HashSet::default();
        queue.extend(self.nodes[*child].children());

        loop {
            let c = match queue.pop_front() {
                Some(c) => c,
                None => break,
            };

            if c == *parent {
                panic!("Render node {:?} circles with itself", child_label);
            } else if iterated.insert(c) {
                queue.extend(self.nodes[c].children());
            }
        }

        self.nodes[*child].parents_mut().push(*parent);
        self.nodes[*parent].children_mut().push(*child);
    }

    pub fn render(&'static mut self, world: &World) {
        let frame = world.resource::<Frame>();
        if !frame.valid {
            return;
        }

        let renderer = world.resource::<Renderer>();
        let surface = world.resource::<SurfaceConfig>();
        let clear_color = world.resource::<ClearColor>();

        if self.state.is_none() {
            self.state = Some(RenderState::new(renderer, surface));
        }

        let root_output = {
            let mut root = None;
            for (node_index, node) in self.nodes.iter().enumerate() {
                if node.children().is_empty() {
                    if root.is_none() {
                        root = Some(node_index);
                    } else {
                        panic!("Multiple render node root output");
                    }
                }
            }

            root
        };

        self.ran.clear();
        for i in 0..self.nodes.len() {
            self.queued.insert(i);
        }

        //TODO check speed if it doesn't use multithreading
        ComputeTaskPool::get().scope(|scope: &Scope<'_, '_, ()>| {
            // Please just shut up for now, Rust.
            let nodes1 = self.nodes();
            let nodes2 = self.nodes();
            let nodes3 = self.nodes();
            let nodes4 = self.nodes();

            for (node_index, node) in nodes1.iter().enumerate() {
                if node.parents().is_empty() {
                    self.queued.remove(&node_index);
                    self.running += 1;

                    scope.spawn(async move {
                        let channel = node.channel();
                        channel.sender.send(HashMap::default()).await.unwrap();
                    });
                }
            }

            for (node_index, node) in nodes2.iter_mut().enumerate() {
                scope.spawn(async move {
                    let (output, sys, channel, done_sender) = node.unzip_input();
                    let input = RenderInput {
                        output,
                        parent_outputs: channel.receiver.recv().await.unwrap(),
                    };

                    unsafe { sys.run_unsafe(input, world); }
                    done_sender.send(node_index).await.unwrap();
                });
            }

            let receiver = &self.done_channel.receiver;
            scope.spawn(async {
                while !self.queued.is_empty() || self.running > 0 {
                    let done = receiver.recv().await.unwrap();
                    self.ran.insert(done);
                    self.running -= 1;

                    while let Ok(done) = receiver.try_recv() {
                        self.ran.insert(done);
                        self.running -= 1;
                    }

                    for ran_index in self.queued.drain_filter(|node_index| {
                        let node = &mut nodes3[*node_index];

                        let mut qualified = true;
                        for parent in node.parents() {
                            if !self.ran.contains(&parent) {
                                qualified = false;
                                break;
                            }
                        }

                        qualified
                    }) {
                        self.running += 1;

                        nodes4[ran_index].channel().sender.send({
                            let mut map = HashMap::default();
                            for parent in nodes4[ran_index].parents() {
                                let (label, output) = nodes4[*parent].unzip_output();
                                map.insert(label, output);
                            }

                            map
                        }).await.unwrap();
                    }
                }
            });
        });

        let mut encoder = renderer.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render graph output encoder"),
        });

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render graph output pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: frame.view.as_ref().unwrap(),
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(clear_color.0.into()),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });

            if let Some(output) = root_output {
                let output = self.nodes[output].output();

                let state = self.state.as_mut().unwrap();
                state.update_bind_group(renderer, &output.buffer.color_attachments[0].1);

                pass.set_pipeline(&state.pipeline);
                pass.set_bind_group(0, state.bind_group.as_ref().unwrap(), &[]);
                pass.set_vertex_buffer(0, state.screen.slice(..));

                pass.draw(0..6, 0..1);
            }
        }

        renderer.queue.submit(iter::once(encoder.finish()));
    }

    #[inline]
    fn nodes(&mut self) -> &'static mut Vec<Box<dyn RenderNodeDyn>> {
        unsafe { mem::transmute(&mut self.nodes) }
    }
}

trait RenderNodeDyn: 'static + Send + Sync {
    fn unzip_input(&mut self) -> (
        &mut RenderOutput,
        &mut dyn System<In = RenderInput, Out = ()>,
        &OutputChannel,
        &SenderAsync<usize>,
    );

    fn unzip_output(&self) -> (SystemLabelId, &RenderOutput);
    fn channel(&self) -> &OutputChannel;
    fn output(&self) -> &RenderOutput;
    fn output_mut(&mut self) -> &mut RenderOutput;

    fn parents(&self) -> &Vec<usize>;
    fn parents_mut(&mut self) -> &mut Vec<usize>;

    fn children(&self) -> &Vec<usize>;
    fn children_mut(&mut self) -> &mut Vec<usize>;
}

struct RenderNode<RenderParam, Desc> where
    RenderParam: 'static + SystemParam,
    <RenderParam as SystemParam>::Fetch: ReadOnlySystemParamFetch,

    Desc: RenderNodeDesc<RenderParam = RenderParam>,
{
    label: SystemLabelId,
    output: RenderOutput,

    render_sys: BoxedSystem<RenderInput, ()>,
    channel: OutputChannel,
    done_sender: SenderAsync<usize>,

    parents: Vec<usize>,
    children: Vec<usize>,

    marker: PhantomData<fn() -> (RenderParam, Desc)>,
}

impl<RenderParam, Desc> RenderNode<RenderParam, Desc> where
    RenderParam: 'static + SystemParam,
    <RenderParam as SystemParam>::Fetch: ReadOnlySystemParamFetch,

    Desc: RenderNodeDesc<RenderParam = RenderParam>,
{
    pub fn new(label: SystemLabelId, renderer: &Renderer, done_sender: SenderAsync<usize>) -> Self {
        Self {
            label,
            output: RenderOutput::new(renderer),

            render_sys: Box::new(IntoSystem::into_system(Desc::render_sys)),
            channel: OutputChannel::default(),
            done_sender,

            parents: vec![],
            children: vec![],

            marker: PhantomData,
        }
    }
}

impl<RenderParam, Desc>
RenderNodeDyn for RenderNode<RenderParam, Desc> where
    RenderParam: 'static + SystemParam,
    <RenderParam as SystemParam>::Fetch: ReadOnlySystemParamFetch,

    Desc: RenderNodeDesc<RenderParam = RenderParam>,
{
    #[inline]
    fn unzip_input(&mut self) -> (
        &mut RenderOutput,
        &mut dyn System<In = RenderInput, Out = ()>,
        &OutputChannel,
        &SenderAsync<usize>,
    ) {
        use ops::DerefMut;
        (&mut self.output, self.render_sys.deref_mut(), &self.channel, &self.done_sender)
    }

    #[inline]
    fn unzip_output(&self) -> (SystemLabelId, &RenderOutput) {
        (self.label, &self.output)
    }

    #[inline]
    fn channel(&self) -> &OutputChannel {
        &self.channel
    }

    #[inline]
    fn output(&self) -> &RenderOutput {
        &self.output
    }

    #[inline]
    fn output_mut(&mut self) -> &mut RenderOutput {
        &mut self.output
    }

    #[inline]
    fn parents(&self) -> &Vec<usize> {
        &self.parents
    }

    #[inline]
    fn parents_mut(&mut self) -> &mut Vec<usize> {
        &mut self.parents
    }

    #[inline]
    fn children(&self) -> &Vec<usize> {
        &self.children
    }

    #[inline]
    fn children_mut(&mut self) -> &mut Vec<usize> {
        &mut self.children
    }
}

pub trait RenderNodeDesc: 'static where
    Self::RenderParam: 'static + SystemParam,
    <Self::RenderParam as SystemParam>::Fetch: ReadOnlySystemParamFetch,
{
    type RenderParam;

    fn init(world: &mut World, schedule: &mut Schedule);
    fn render_sys(input: In<RenderInput>, param: SystemParamItem<Self::RenderParam>);
}
