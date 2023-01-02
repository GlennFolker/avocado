use crate::incl::*;

const MAX_VERTICES: usize = 65536;
const MAX_INDICES: usize = MAX_VERTICES / 4 * 6;

#[derive(Debug, Default, Copy, Clone)]
pub struct SpriteTransform {
    /// Left vertex position in world space.
    pub x: f32,
    /// Bottom vertex position in world space.
    pub y: f32,
    /// Z-layer of the sprite for sorting.
    pub z: f32,

    /// Rotation pivot relative to center `x` position.
    pub pivot_x: f32,
    /// Rotation pivot relative to center `y` position.
    pub pivot_y: f32,

    /// Width of the drawn sprite.
    pub width: f32,
    /// Height of the drawn sprite.
    pub height: f32,

    /// Rotation of the drawn sprite, in degrees.
    pub rotation: f32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
pub struct SpriteVertex {
    pub pos: [f32; 2],
    pub uv: [f32; 2],
    pub col: [u8; 4],
    pub index: u32,
}

impl SpriteVertex {
    #[inline]
    pub fn new(x: f32, y: f32, u: f32, v: f32, col: [u8; 4], index: u32) -> Self {
        Self {
            pos: [x, y],
            uv: [u, v],
            col, index,
        }
    }
}

#[derive(Resource, Debug)]
pub struct SpriteBatch<const MASK: u32> {
    sprites: Vec<Sprite>,
    buffer_data: Vec<(Vec<SpriteVertex>, Vec<u16>, u32, u32)>,

    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    state: Option<BatchState>,
}

#[derive(Debug)]
struct BatchState {
    bind_group: wgpu::BindGroup,
    pipeline: wgpu::RenderPipeline,
}

impl BatchState {
    fn new(
        renderer: &Renderer, camera: &GlobalCamera,
        atlas: &TextureAtlas, textures: &Assets<Texture>
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

        let shader = renderer.device.create_shader_module(include_wgsl!("batch.wgsl"));

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
                    array_stride: mem::size_of::<SpriteVertex>() as wgpu::BufferAddress,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &vertex_attr_array![
                        0 => Float32x2,
                        1 => Float32x2,
                        2 => Unorm8x4,
                        3 => Uint32,
                    ],
                }],
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Cw,
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

impl<const MASK: u32> SpriteBatch<MASK> {
    pub fn queue_sys(
        mut batch: ResMut<Self>,
        renderer: Res<Renderer>, camera: Res<GlobalCamera>,
        atlas: Option<Res<TextureAtlas>>, textures: Res<Assets<Texture>>,
        holders: Query<&SpriteHolder>,
    ) {
        if atlas.is_none() {
            return;
        }

        let atlas = atlas.unwrap();
        if batch.state.is_none() || atlas.is_changed() {
            batch.state = Some(BatchState::new(&renderer, &camera, &atlas, &textures));
        }

        batch.sprites.clear();
        for (_, _, vert_len, ind_len) in &mut batch.buffer_data {
            *vert_len = 0;
            *ind_len = 0;
        }

        for holder in &holders {
            for sprite in &holder.sprites {
                if (sprite.mask & MASK) == MASK {
                    batch.sprites.push(*sprite);
                }
            }
        }

        batch.sprites.sort_by(|a, b| a.trns.z.partial_cmp(&b.trns.z).unwrap());

        let mut flush_index = 0;
        for i in 0..batch.sprites.len() {
            let sprite = batch.sprites[i];
            if batch.buffer_data[flush_index].2 >= (MAX_VERTICES as u32) {
                flush_index += 1;
            }

            if batch.buffer_data.len() <= flush_index {
                batch.buffer_data.push((
                    vec![SpriteVertex::zeroed(); MAX_VERTICES],
                    vec![0; MAX_INDICES],
                    0,
                    0,
                ));
            }

            let region = sprite.region;
            let u = region.u;
            let u2 = region.u2;
            let v = region.v;
            let v2 = region.v2;
            let index = region.page_index as u32;

            let trns = sprite.trns;
            let x = trns.x;
            let y = trns.y;
            let width = trns.width;
            let height = trns.height;
            let rotation = trns.rotation;

            let vertices = {
                let color = sprite.color.to_vertex();
                if rotation.abs() <= 0.000001 {
                    [
                        SpriteVertex::new(x, y, u, v2, color.clone(), index),
                        SpriteVertex::new(x, y + height, u, v, color.clone(), index),
                        SpriteVertex::new(x + width, y + height, u2, v, color.clone(), index),
                        SpriteVertex::new(x + width, y, u2, v2, color, index),
                    ]
                } else {
                    let (sin, cos) = rotation.to_radians().sin_cos();
                    let origin_x = x + trns.pivot_x;
                    let origin_y = y + trns.pivot_y;

                    let fx = -origin_x;
                    let fy = -origin_y;
                    let fx2 = width - origin_x;
                    let fy2 = height - origin_y;

                    let x1 = cos * fx - sin * fy + origin_x;
                    let y1 = sin * fx - cos * fy + origin_y;
                    let x2 = cos * fx - sin * fy2 + origin_x;
                    let y2 = sin * fx - cos * fy2 + origin_y;
                    let x3 = cos * fx2 - sin * fy2 + origin_x;
                    let y3 = sin * fx2 - cos * fy2 + origin_y;
                    let x4 = x3 - (x2 - x1);
                    let y4 = y3 - (y2 - y1);

                    [
                        SpriteVertex::new(x1, y1, u, v2, color.clone(), index),
                        SpriteVertex::new(x2, y2, u, v, color.clone(), index),
                        SpriteVertex::new(x3, y3, u2, v, color.clone(), index),
                        SpriteVertex::new(x4, y4, u2, v2, color, index),
                    ]
                }
            };

            let vert_len = batch.buffer_data[flush_index].2 as usize;
            let ind_len = batch.buffer_data[flush_index].3 as usize;

            let vert_index = vert_len as u16;
            let indices = [
                vert_index,
                vert_index + 1,
                vert_index + 2,
                vert_index + 2,
                vert_index + 3,
                vert_index,
            ];

            batch.buffer_data[flush_index].0[vert_len..(vert_len + 4)].copy_from_slice(&vertices);
            batch.buffer_data[flush_index].1[ind_len..(ind_len + 6)].copy_from_slice(&indices);

            batch.buffer_data[flush_index].2 += 4;
            batch.buffer_data[flush_index].3 += 6;
        }
    }
}

impl<const MASK: u32> RenderNodeDesc for SpriteBatch<MASK> {
    type RenderParam = (SRes<Self>, SRes<Renderer>, SRes<GlobalCamera>);

    fn init(world: &mut World, schedule: &mut Schedule) {
        let renderer = world.resource::<Renderer>();

        let vertex_buffer = renderer.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Sprite batch vertex buffer"),
            size: (MAX_VERTICES * mem::size_of::<SpriteVertex>()) as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let index_buffer = renderer.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Sprite batch index buffer"),
            size: (MAX_INDICES * mem::size_of::<SpriteVertex>()) as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        world.insert_resource(Self {
            sprites: vec![],
            buffer_data: vec![(
                vec![SpriteVertex::zeroed(); MAX_VERTICES],
                vec![0; MAX_INDICES],
                0,
                0,
            )],

            vertex_buffer,
            index_buffer,
            state: None,
        });

        schedule.add_system_to_stage(RenderStage::Queue, Self::queue_sys
            .label(RenderLabel::Queue)
        );
    }

    fn render_sys(In(node): In<RenderInput>, (
        batch,
        renderer, camera,
    ): SystemParamItem<Self::RenderParam>) {
        if batch.state.is_none() {
            return;
        }

        let state = batch.state.as_ref().unwrap();
        for (vertices, indices, vert_len, ind_len) in &batch.buffer_data {
            renderer.queue.write_buffer(
                &batch.vertex_buffer,
                0 as wgpu::BufferAddress,
                bytemuck::cast_slice(&vertices[0..(*vert_len as usize)]),
            );

            renderer.queue.write_buffer(
                &batch.index_buffer,
                0 as wgpu::BufferAddress,
                bytemuck::cast_slice(&indices[0..(*ind_len as usize)]),
            );

            let mut encoder = renderer.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Sprite batch render encoder"),
            });

            {
                let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Sprite batch render pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &node.output.buffer.color_attachments[0].1,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
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
