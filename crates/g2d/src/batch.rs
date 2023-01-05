use avocado_asset::prelude::*;
use avocado_core::prelude::*;
use avocado_winit::prelude::*;

use crate::{
    SPRITE_MAX_VERTICES,
    TextureAtlas,
    SpriteHolder, SpriteVertex,
    BatchState, SpriteBatchConfig, SpriteBatchDefShader,
};
use std::{
    cmp::Ordering,
    fmt::Debug,
    iter,
    mem,
};

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
            assert!((sprite.desc.vert_len() as usize) <= SPRITE_MAX_VERTICES, "Too many vertices ({} > {})", sprite.desc.vert_len(), SPRITE_MAX_VERTICES);
            assert!(sprite.desc.ind_len() <= max_ind, "Too many indices ({} > {})", sprite.desc.ind_len(), max_ind);

            if
                *vert_len >= (SPRITE_MAX_VERTICES as u32) ||
                *vert_len + (sprite.desc.vert_len() as u32) >= (SPRITE_MAX_VERTICES as u32) ||
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
            vec![T::zeroed(); SPRITE_MAX_VERTICES],
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
                config.max_indices.unwrap_or((SPRITE_MAX_VERTICES / 4 * 6) as u32),
                config.shader.as_ref().cloned().unwrap_or_else(|| world.resource::<SpriteBatchDefShader>().0.clone()),
            ),
            None => (
                (SPRITE_MAX_VERTICES / 4 * 6) as u32,
                world.resource::<SpriteBatchDefShader>().0.clone(),
            ),
        };

        let vertex_buffer = renderer.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Sprite batch vertex buffer"),
            size: (SPRITE_MAX_VERTICES * mem::size_of::<T>()) as wgpu::BufferAddress,
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
