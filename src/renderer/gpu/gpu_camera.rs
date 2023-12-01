use glam::Mat4;

use crate::renderer::render_state::view::Camera;

pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

pub static CAMERA_LAYOUT : wgpu::BindGroupLayoutDescriptor = wgpu::BindGroupLayoutDescriptor
{
    
    label : Some("Camera Parameters"),
    entries : 
    &[
        //View Transform - 0
        wgpu::BindGroupLayoutEntry
        {
            binding: 0,
            visibility: wgpu::ShaderStages::VERTEX,
            ty: wgpu::BindingType::Buffer 
            {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        },
    ],
};

pub struct GPUCamera
{
    pub tf_uniform: wgpu::Buffer,
    //pub camera_bind_group_layout: wgpu::BindGroupLayout,
    pub camera_bind_group: wgpu::BindGroup,
    depth_texture: wgpu::Texture,
    pub depth_view: wgpu::TextureView,
    depth_sampler: wgpu::Sampler,
}

impl GPUCamera
{
    pub fn new(device: &wgpu::Device, layout: &wgpu::BindGroupLayout, width: u32, height: u32) -> Self
    {
        let buf = device.create_buffer
        (
            &wgpu::BufferDescriptor
            {
                label: Some("View Projection Buffer"),
                size: std::mem::size_of::<[[f32; 4]; 4]>() as u64,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }
        );
        let cam_bind_group = wgpu::BindGroupEntry
        {
            binding: 0,
            resource:  wgpu::BindingResource::Buffer
            (
                wgpu::BufferBinding
                {
                    buffer: &buf,
                    offset: 0,
                    size: wgpu::BufferSize::new(std::mem::size_of::<[[f32;4]; 4]>() as u64),
                }
            ),
        };
        let cam_bg = device.create_bind_group
        (
            &wgpu::BindGroupDescriptor
            { 
                label: Some("Camera"), 
                layout: layout, 
                entries: &
                [
                    cam_bind_group,
                ]
            }
        );
        let size = wgpu::Extent3d
        {
            width: width,
            height: height,
            depth_or_array_layers: 1,
        };
        let desc = wgpu::TextureDescriptor 
        {
            label: Some("camera depth"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: DEPTH_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        };
        let depth_texture = device.create_texture(&desc);
        let depth_view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let depth_sampler = device.create_sampler
        (
            &wgpu::SamplerDescriptor 
            {
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                address_mode_w: wgpu::AddressMode::ClampToEdge,
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Linear,
                mipmap_filter: wgpu::FilterMode::Nearest,
                compare: Some(wgpu::CompareFunction::LessEqual),
                lod_min_clamp: 0.0,
                lod_max_clamp: 100.0,
                ..Default::default()
            }
        );

        return GPUCamera 
        { 
            tf_uniform: buf, 
            camera_bind_group: cam_bg,
            depth_texture: depth_texture,
            depth_sampler: depth_sampler,
            depth_view: depth_view,
        };
    }

    pub fn update_camera(&self, camera: &Camera, queue : &wgpu::Queue)
    {
        queue.write_buffer
        (
            &self.tf_uniform,
            0,
            bytemuck::cast_slice(&(camera.get_perspective_matrix()*camera.get_view_matrix()).to_cols_array()),
        );
    }
}