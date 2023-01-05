use avocado_graphics::prelude::*;
use avocado_utils::prelude::*;
use avocado_winit::prelude::*;

use crate::{
    AtlasRegion, Sprite, SpriteDesc,
};
use bytemuck::{
    Pod, Zeroable,
};
use std::fmt::Debug;

pub const SPRITE_MAX_VERTICES: usize = 65536;

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

    const ATTRIBUTES: &'static [wgpu::VertexAttribute] = &wgpu::vertex_attr_array![
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
