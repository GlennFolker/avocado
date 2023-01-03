use crate::incl::*;

#[derive(Component)]
pub struct Camera {
    /// The camera's position.
    pub position: Vec3,
    pub near: f32,
    pub far: f32,
    pub proj: CameraProj,
}

#[derive(Debug, Copy, Clone)]
pub enum CameraProj {
    Orthographic {
        /// Orthographic viewport width.
        width: f32,
        /// Orthographic viewport height.
        height: f32,
    },
    Perspective {
        /// The camera's "look" angle.
        target: Vec3,
        /// Which way is up.
        up: Vec3,
        aspect: f32,
        /// Field of view in degrees.
        fov: f32,
    },
}

#[derive(Resource)]
pub struct GlobalCamera {
    pub entity: Entity,
    pub proj: Option<Mat4>,

    pub buffer: wgpu::Buffer,
    pub bind_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
}

impl FromWorld for GlobalCamera {
    fn from_world(world: &mut World) -> Self {
        let renderer = world.resource::<Renderer>();
        let buffer = renderer.device.create_buffer_init(&wgpu::BufferInitDescriptor {
            label: Some("Camera view projection buffer"),
            contents: bytemuck::cast_slice(Mat4::IDENTITY.as_ref()),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_layout = renderer.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Camera view projection bind group layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let bind_group = renderer.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Camera view projection bind group"),
            layout: &bind_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
        });

        Self {
            entity: Entity::from_raw(0),
            proj: None,

            buffer, bind_layout, bind_group,
        }
    }
}

impl GlobalCamera {
    pub fn update_sys(
        mut global_camera: ResMut<GlobalCamera>, renderer: Res<Renderer>,
        query: Query<&Camera, Changed<Camera>>
    ) {
        if let Ok(camera) = query.get(global_camera.bypass_change_detection().entity) {
            global_camera.proj = Some(match camera.proj {
                CameraProj::Orthographic { width, height } => Mat4::orthographic_rh(
                    camera.position.x - width / 2., camera.position.x + width / 2.,
                    camera.position.y - height / 2., camera.position.y + height / 2.,
                    camera.near, camera.far,
                ),
                CameraProj::Perspective { target, up, aspect, fov, } => {
                    let view = Mat4::look_at_rh(camera.position, target, up.normalize());
                    let proj = Mat4::perspective_rh(
                        fov.to_radians(), aspect,
                        camera.near, camera.far,
                    );

                    proj * view
                },
            });

            renderer.queue.write_buffer(
                &global_camera.buffer, 0,
                bytemuck::cast_slice(global_camera.proj.as_ref().unwrap().as_ref()),
            );
        }
    }
}
