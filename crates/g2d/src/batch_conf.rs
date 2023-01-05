use avocado_asset::prelude::*;
use avocado_core::prelude::*;
use avocado_utils::prelude::*;
use avocado_winit::prelude::*;

use crate::{
    TextureAtlas,
    SpriteVertex,
};
use std::{
    fmt::Debug,
    marker::PhantomData,
    mem,
    num::NonZeroU32,
};

#[derive(Debug)]
pub struct BatchState {
    pub bind_group: wgpu::BindGroup,
    pub pipeline: wgpu::RenderPipeline,
}

impl BatchState {
    pub fn new<T: SpriteVertex>(
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
