use crate::incl::*;

pub const MAX_VERTICES: usize = 65536;

pub trait SpriteVertex: 'static + Debug + Copy + Clone + Send + Sync + Pod + Zeroable {
    type Data: 'static + Debug + Send + Sync;

    const ATTRIBUTES: &'static [wgpu::VertexAttribute];

    #[inline]
    fn from_sprite(sprite: &Sprite<Self>, ind_offset: u16) -> (SmallVec<[Self; 4]>, SmallVec<[u16; 6]>) {
        match &sprite.desc {
            SpriteDesc::Direct {
                vertices, indices,
                ..
            } => Self::from_direct(vertices, indices, ind_offset),
            SpriteDesc::Transform {
                pos,
                anchor, size, rotation,
                data,
                ..
            } => Self::from_transform(
                sprite.region, sprite.color,
                *pos, *anchor, *size, *rotation, ind_offset, data,
            ),
        }
    }

    fn from_direct(vertices: &[Self], indices: &[u16], ind_offset: u16) -> (SmallVec<[Self; 4]>, SmallVec<[u16; 6]>) {
        let vert_len = vertices.len();
        let ind_len = indices.len();

        let mut verts = SmallVec::with_capacity(vert_len);
        unsafe { verts.set_len(vert_len); }

        let mut inds = SmallVec::with_capacity(ind_len);

        verts.copy_from_slice(vertices);
        for index in indices {
            inds.push(*index + ind_offset);
        }

        (verts, inds)
    }

    fn from_transform(
        region: AtlasRegion, color: Color,
        pos: Vec2, anchor: Vec2, size: Vec2, rotation: f32,
        ind_offset: u16,
        data: &Self::Data,
    ) -> (SmallVec<[Self; 4]>, SmallVec<[u16; 6]>) {
        #[inline]
        fn copy_col(col: &[u8; 4]) -> [u8; 4] {
            let mut out = [0; 4];
            out.copy_from_slice(col);
            out
        }

        let page_index = region.page_index as u32;
        let color1 = color.to_vertex();
        let color2 = copy_col(&color1);
        let color3 = copy_col(&color1);
        let color4 = copy_col(&color1);

        let vertices = SmallVec::from_slice(&(if rotation.abs() <= 0.000001 {
            [
                Self::new(pos.x, pos.y, region.u, region.v2, color1, 0, page_index, data),
                Self::new(pos.x + size.x, pos.y, region.u2, region.v2, color1, 1, page_index, data),
                Self::new(pos.x + size.x, pos.y + size.y, region.u2, region.v, color1, 2, page_index, data),
                Self::new(pos.x, pos.y + size.y, region.u, region.v, color1, 3, page_index, data),
            ]
        } else {
            let (sin, cos) = rotation.to_radians().sin_cos();
            let ox = pos.x + anchor.x;
            let oy = pos.y + anchor.y;

            let fx = -anchor.x;
            let fy = -anchor.y;
            let fx2 = size.x - anchor.x;
            let fy2 = size.y - anchor.y;

            let x1 = cos * fx - sin * fy + ox;
            let y1 = sin * fx + cos * fy + oy;
            let x2 = cos * fx2 - sin * fy + ox;
            let y2 = sin * fx2 + cos * fy + oy;
            let x3 = cos * fx2 - sin * fy2 + ox;
            let y3 = sin * fx2 + cos * fy2 + oy;
            let x4 = x3 + (x1 - x2);
            let y4 = y3 + (y1 - y2);

            [
                Self::new(x1, y1, region.u, region.v2, color1, 0, page_index, data),
                Self::new(x2, y2, region.u2, region.v2, color2, 1, page_index, data),
                Self::new(x3, y3, region.u2, region.v, color3, 2, page_index, data),
                Self::new(x4, y4, region.u, region.v, color4, 3, page_index, data),
            ]
        }));

        let indices = SmallVec::from_slice(&[
            ind_offset,
            ind_offset + 1,
            ind_offset + 2,
            ind_offset + 2,
            ind_offset + 3,
            ind_offset,
        ]);

        (vertices, indices)
    }

    fn new(
        x: f32, y: f32, u: f32, v: f32, color: [u8; 4],
        vertex_index: u8, page_index: u32, data: &Self::Data
    ) -> Self;
}

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
pub struct DefSpriteVertex {
    pub pos: [f32; 2],
    pub uv: [f32; 2],
    pub col: [u8; 4],
    pub index: u32,
}

impl SpriteVertex for DefSpriteVertex {
    type Data = ();

    const ATTRIBUTES: &'static [wgpu::VertexAttribute] = &vertex_attr_array![
        0 => Float32x2,
        1 => Float32x2,
        2 => Unorm8x4,
        3 => Uint32,
    ];

    #[inline]
    fn new(
        x: f32, y: f32, u: f32, v: f32, color: [u8; 4],
        _: u8, page_index: u32, _: &Self::Data
    ) -> Self {
        Self {
            pos: [x, y],
            uv: [u, v],
            col: color,
            index: page_index,
        }
    }
}

#[derive(Debug)]
pub enum SpriteDesc<T: SpriteVertex> {
    Direct {
        z: f32,
        vertices: SmallVec<[T; 4]>,
        indices: SmallVec<[u16; 6]>,
    },
    Transform {
        pos: Vec2,
        z: f32,
        anchor: Vec2,
        size: Vec2,
        rotation: f32,
        data: T::Data,
    },
}

impl<T: SpriteVertex> SpriteDesc<T> {
    #[inline]
    pub fn z(&self) -> f32 {
        match self {
            Self::Direct { z, .. } => *z,
            Self::Transform { z, .. } => *z,
        }
    }

    #[inline]
    pub fn vert_len(&self) -> u16 {
        (match self {
            Self::Direct { vertices, .. } => vertices.len(),
            Self::Transform { .. } => 4,
        }) as u16
    }

    #[inline]
    pub fn ind_len(&self) -> u32 {
        (match self {
            Self::Direct { indices, .. } => indices.len(),
            Self::Transform { .. } => 6,
        }) as u32
    }
}

#[derive(Debug)]
struct BatchState {
    bind_group: wgpu::BindGroup,
    pipeline: wgpu::RenderPipeline,
}

impl BatchState {
    fn new<T: SpriteVertex>(
        renderer: &Renderer, camera: &GlobalCamera,
        atlas: &TextureAtlas, shader: &Handle<Shader>,
        textures: &Assets<Texture>, shaders: &Assets<Shader>,
    ) -> Self {
        let pages = atlas.pages.iter().map(|handle| textures.get(&handle).unwrap()).collect::<Vec<_>>();
        let views = pages.iter().map(|texture| &texture.view).collect::<Vec<_>>();
        let samplers = atlas.samplers.iter().map(|sampler| sampler).collect::<Vec<_>>();

        let bind_layout = renderer.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Sprite batch bind group layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true, },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: Some(NonZeroU32::new(views.len() as u32).unwrap()),
            }, wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: Some(NonZeroU32::new(samplers.len() as u32).unwrap()),
            }],
        });

        let bind_group = renderer.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Sprite batch bind group"),
            layout: &bind_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureViewArray(&views),
            }, wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::SamplerArray(&samplers),
            }]
        });

        let shader = shaders.get(shader).unwrap();

        let pipeline = renderer.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Sprite batch render pipeline"),
            layout: Some(&renderer.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Sprite batch pipeline layout"),
                bind_group_layouts: &[&bind_layout, &camera.bind_layout],
                push_constant_ranges: &[],
            })),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: mem::size_of::<T>() as wgpu::BufferAddress,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: T::ATTRIBUTES,
                }],
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Rgba8UnormSrgb,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::SrcAlpha,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                        alpha: wgpu::BlendComponent::OVER,
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview: None,
        });

        Self { bind_group, pipeline, }
    }
}

#[derive(Resource)]
pub struct SpriteBatchConfig<T: SpriteVertex> {
    pub max_indices: Option<u32>,
    pub shader: Option<Handle<Shader>>,
    pub marker: PhantomData<fn() -> T>,
}

impl<T: SpriteVertex> Default for SpriteBatchConfig<T> {
    fn default() -> Self {
        Self {
            max_indices: None,
            shader: None,
            marker: PhantomData,
        }
    }
}

#[derive(Resource, Deref)]
pub struct SpriteBatchDefShader(pub Handle<Shader>);

#[derive(Resource, Debug)]
pub struct SpriteBatch<T: SpriteVertex> {
    buffer_data: Vec<(Vec<T>, Vec<u16>, u32, u32)>,
    max_indices: u32,

    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    shader: Handle<Shader>,
    state: Option<BatchState>,
}

impl<T: SpriteVertex> Subsystem for SpriteBatch<T> {
    fn init(app: &mut App) {
        app.sys(RenderStage::Queue, Self::queue_sys
            .label(RenderLabel::Queue)
        );
    }
}

impl<T: SpriteVertex> SpriteBatch<T> {
    pub fn queue_sys(
        batch: Option<ResMut<Self>>,
        renderer: Res<Renderer>, camera: Res<GlobalCamera>,
        atlas: Option<Res<TextureAtlas>>, textures: Res<Assets<Texture>>, shaders: Res<Assets<Shader>>,
        holders: Query<&SpriteHolder<T>>,
    ) {
        let Some(mut batch) = batch else { return };
        let Some(atlas) = atlas else { return };

        if batch.state.is_none() || atlas.is_changed() {
            batch.state = Some(BatchState::new::<T>(&renderer, &camera, &atlas, &batch.shader, &textures, &shaders));
        }

        for (_, _, vert_len, ind_len) in &mut batch.buffer_data {
            *vert_len = 0;
            *ind_len = 0;
        }

        let mut sprites = vec![];
        for holder in &holders {
            holder.sprites.iter().for_each(|sprite| sprites.push(sprite));
        }

        sprites.sort_by(|a, b| a.desc.z().partial_cmp(&b.desc.z()).unwrap_or(Ordering::Equal));

        let max_ind = batch.max_indices;
        let mut flush_index = 0;
        for sprite in &sprites {
            let (_, _, vert_len, ind_len) = &batch.buffer_data[flush_index];
            assert!((sprite.desc.vert_len() as usize) <= MAX_VERTICES, "Too many vertices ({} > {})", sprite.desc.vert_len(), MAX_VERTICES);
            assert!(sprite.desc.ind_len() <= max_ind, "Too many indices ({} > {})", sprite.desc.ind_len(), max_ind);

            if
                *vert_len >= (MAX_VERTICES as u32) ||
                *vert_len + (sprite.desc.vert_len() as u32) >= (MAX_VERTICES as u32) ||
                *ind_len >= max_ind ||
                *ind_len + sprite.desc.ind_len() >= max_ind
            {
                flush_index += 1;
            }

            if batch.buffer_data.len() <= flush_index {
                batch.buffer_data.push(Self::create_data(max_ind));
            }

            let (vertices, indices, vert_len, ind_len) = &mut batch.buffer_data[flush_index];
            let (v, i) = T::from_sprite(sprite, *vert_len as u16);

            vertices[(*vert_len as usize)..(*vert_len as usize) + v.len()].copy_from_slice(&v);
            *vert_len += v.len() as u32;

            indices[(*ind_len as usize)..(*ind_len as usize) + i.len()].copy_from_slice(&i);
            *ind_len += i.len() as u32;
        }
    }

    pub fn create_data(max_indices: u32) -> (Vec<T>, Vec<u16>, u32, u32) {
        (
            vec![T::zeroed(); MAX_VERTICES],
            vec![0; max_indices as usize],
            0,
            0,
        )
    }
}

impl<T: SpriteVertex> FromWorld for SpriteBatch<T> {
    fn from_world(world: &mut World) -> Self {
        let renderer = world.resource::<Renderer>();
        let (max_indices, shader) = match world.get_resource::<SpriteBatchConfig<T>>() {
            Some(config) => (
                config.max_indices.unwrap_or((MAX_VERTICES / 4 * 6) as u32),
                config.shader.as_ref().cloned().unwrap_or_else(|| world.resource::<SpriteBatchDefShader>().0.clone()),
            ),
            None => (
                (MAX_VERTICES / 4 * 6) as u32,
                world.resource::<SpriteBatchDefShader>().0.clone(),
            ),
        };

        let vertex_buffer = renderer.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Sprite batch vertex buffer"),
            size: (MAX_VERTICES * mem::size_of::<T>()) as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let index_buffer = renderer.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Sprite batch index buffer"),
            size: ((max_indices as usize) * mem::size_of::<u16>()) as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            buffer_data: vec![Self::create_data(max_indices)],
            max_indices,

            vertex_buffer, index_buffer, shader,
            state: None,
        }
    }
}

impl<T: SpriteVertex> RenderNodeDesc for SpriteBatch<T> {
    type Param = (SRes<Self>, SRes<Renderer>, SRes<GlobalCamera>);

    fn init(world: &mut World) {
        world.init_resource::<Self>();
    }

    fn render_sys(In(node): In<RenderInput>, (
        batch,
        renderer, camera,
    ): SystemParamItem<Self::Param>) {
        if batch.state.is_none() {
            return;
        }

        let state = batch.state.as_ref().unwrap();
        let mut first = true;

        for (vertices, indices, vert_len, ind_len) in &batch.buffer_data {
            if *vert_len <= 0 || *ind_len <= 0 {
                break;
            }

            renderer.queue.write_buffer(
                &batch.vertex_buffer, 0,
                bytemuck::cast_slice(&vertices[0..(*vert_len as usize)]),
            );

            renderer.queue.write_buffer(
                &batch.index_buffer, 0,
                bytemuck::cast_slice(&indices[0..(*ind_len as usize)]),
            );

            let mut encoder = renderer.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Sprite batch render encoder"),
            });

            {
                let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Sprite batch render pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &node.output.buffer.colors[0].view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: if first {
                                first = false;
                                wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT)
                            } else {
                                wgpu::LoadOp::Load
                            },
                            store: true,
                        },
                    })],
                    depth_stencil_attachment: None,
                });

                pass.set_pipeline(&state.pipeline);
                pass.set_bind_group(0, &state.bind_group, &[]);
                pass.set_bind_group(1, &camera.bind_group, &[]);
                pass.set_vertex_buffer(0, batch.vertex_buffer.slice(..));
                pass.set_index_buffer(batch.index_buffer.slice(..), wgpu::IndexFormat::Uint16);

                pass.draw_indexed(0..*ind_len, 0, 0..1);
            }

            renderer.queue.submit(iter::once(encoder.finish()));
        }
    }
}
