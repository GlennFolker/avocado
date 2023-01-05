use avocado_utils::prelude::*;

use crate::Renderer;

#[derive(Debug, Default, Copy, Clone)]
pub struct SamplerDesc {
    pub address: AddressModes,
    pub filter: FilterModes,
}

#[derive(Debug, Default, Copy, Clone)]
pub struct AddressModes {
    pub u: wgpu::AddressMode,
    pub v: wgpu::AddressMode,
    pub w: wgpu::AddressMode,
}

#[derive(Debug, Default, Copy, Clone)]
pub struct FilterModes {
    pub min: wgpu::FilterMode,
    pub mag: wgpu::FilterMode,
    pub mipmap: wgpu::FilterMode,
}

impl SamplerDesc {
    pub fn create_sampler(self, renderer: &Renderer) -> wgpu::Sampler {
        renderer.device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: self.address.u,
            address_mode_v: self.address.v,
            address_mode_w: self.address.w,

            min_filter: self.filter.min,
            mag_filter: self.filter.mag,
            mipmap_filter: self.filter.mipmap,

            ..default()
        })
    }
}
