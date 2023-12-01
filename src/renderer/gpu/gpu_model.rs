use glam::{Quat, Mat4};
use wgpu::{util::DeviceExt, BindGroupDescriptor, BindGroupEntry};

use crate::renderer::render_state::{model::{AnimatedModelInstance, StaticModelInstance, Model}, common::Color};

const NUM_BONES : usize = 256;
const NUM_MORPH_TARGETS : usize = 128;
const NUM_SDFS_PER_VERT : usize = 8;
const NUM_COLORS : usize = 128;


//Model bind groups last as long as the scene they belong to is loaded, or the entire runtime if they're a common asset.
//They are bound and unbound as individual models are rendered throughout the render pass.
pub static STATIC_LAYOUT : wgpu::BindGroupLayoutDescriptor = wgpu::BindGroupLayoutDescriptor
{
    label : Some("Static Model Parameters"),
    entries : 
    &[
        //World Transform - 0
        wgpu::BindGroupLayoutEntry
        {
            binding: 0,
            visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
            ty: wgpu::BindingType::Buffer 
            {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        },
        wgpu::BindGroupLayoutEntry
        {
            binding: 1,
            visibility: wgpu::ShaderStages::VERTEX,
            ty: wgpu::BindingType::Buffer 
            {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        },
        //pibald instructions - 1
        /*wgpu::BindGroupLayoutEntry
        {
            binding: 1,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Buffer 
            {
                ty: wgpu::BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        },*/
    ],
};

pub static SKINNED_LAYOUT : wgpu::BindGroupLayoutDescriptor = wgpu::BindGroupLayoutDescriptor
{
    label : Some("Skinned Model Parameters"),
    entries : 
    &[
        //Animation State - 0
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
        wgpu::BindGroupLayoutEntry
        {
            binding: 1,
            visibility: wgpu::ShaderStages::VERTEX,
            ty: wgpu::BindingType::Buffer 
            {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        },
        //Skeleton
        wgpu::BindGroupLayoutEntry
        {
            binding: 2,
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

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct GPUColor
{
    rgba : [f32;4],
}

fn init_tf_uniform_buffer(tf: &Mat4, device: &wgpu::Device) -> wgpu::Buffer
{
    return device.create_buffer_init
    (
        &wgpu::util::BufferInitDescriptor 
        {
            label: Some( "Modelspace Uniform" ),
            contents: &bytemuck::cast_slice( &tf.to_cols_array() ),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        }
    );
}

fn init_color_uniform_buffer(colors: &Vec<Color>, device: &wgpu::Device) -> wgpu::Buffer
{
    let colors_arr = 
    {
        let mut temp = [GPUColor{rgba: [0.0; 4]}; NUM_COLORS];
        let mut i = 0;
        let col_vals = colors.iter().map(|c| GPUColor{rgba: c.data.to_array()} ).collect::<Vec<_>>();
        while i < col_vals.len() && i < temp.len()
        {
            temp[i] = col_vals[i];
            i+=1;
        }
        temp
    };
    return device.create_buffer_init
    (
        &wgpu::util::BufferInitDescriptor 
        {
            label: Some( "Vertex Color Uniform" ),
            contents: &bytemuck::cast_slice( &colors_arr ),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        }
    );
}

pub struct GPUStaticModelInstance
{
    model_id: String,
    tf_uniform : wgpu::Buffer,
    color_store: wgpu::Buffer,
    pub static_bind_group : wgpu::BindGroup,
}

impl GPUStaticModelInstance
{
    pub fn new(model: &StaticModelInstance, static_layout: &wgpu::BindGroupLayout, device: &wgpu::Device) -> Self
    {
        let tf_uniform = init_tf_uniform_buffer(&model.transform(), device);
        let color_uniform = init_color_uniform_buffer(&model.colors(), device);
        let bind_group = device.create_bind_group
        (
            &wgpu::BindGroupDescriptor
            { 
                label: Some( "Static model" ), 
                layout: &static_layout, 
                entries: 
                &[
                    BindGroupEntry
                    {
                        binding: 0,
                        resource:  wgpu::BindingResource::Buffer
                        (
                            wgpu::BufferBinding
                            {
                                buffer: &tf_uniform,
                                offset: 0,
                                size: wgpu::BufferSize::new(std::mem::size_of::<[[f32;4]; 4]>() as u64),
                            },
                        ),
                    },
                    BindGroupEntry
                    {
                        binding: 1,
                        resource:  wgpu::BindingResource::Buffer
                        (
                            wgpu::BufferBinding
                            {
                                buffer: &color_uniform,
                                offset: 0,
                                size: wgpu::BufferSize::new(std::mem::size_of::<[[f32;4]; NUM_COLORS]>() as u64),
                            },
                        ),
                    }
                ]
            }
        );
        return GPUStaticModelInstance 
        {
            model_id: model.model_id().clone(), 
            tf_uniform: tf_uniform,
            color_store: color_uniform,
            static_bind_group: bind_group
        };
    }

    pub fn update_static_model_instance(&self, instance: &StaticModelInstance,  queue: &wgpu::Queue)
    {
        queue.write_buffer
        (
            &self.tf_uniform,
            0,
            &bytemuck::bytes_of( &instance.transform().to_cols_array() )
        );
    }

    pub fn destroy(self)
    {
        self.tf_uniform.destroy();
    }
}

pub struct GPUAnimatedModelInstance
{
    model_id: String,
    tf_uniform : wgpu::Buffer,
    color_store: wgpu::Buffer,
    pose_buffer: [Mat4; NUM_BONES],
    inverse_pose_buffer: [Mat4; NUM_BONES],
    animation_state_uniform : wgpu::Buffer,
    pub animated_bind_group : wgpu::BindGroup,
}

impl GPUAnimatedModelInstance
{
    pub fn new(anim_layout: &wgpu::BindGroupLayout, instance: &AnimatedModelInstance, device: &wgpu::Device) -> Self
    {
        let tf_uniform = init_tf_uniform_buffer(&instance.transform(), device);
        let color_store = init_color_uniform_buffer(&instance.colors(), device);
        let mut pose_buf = [Mat4::IDENTITY; NUM_BONES];
        let mut inv_buf = [Mat4::IDENTITY; NUM_BONES];
        instance.anim_state().write_current_pose_transforms(&mut pose_buf, &mut inv_buf);
        let anim_buf = device.create_buffer_init
        (
            &wgpu::util::BufferInitDescriptor
            {
                label: Some("Animation State Buffer"),
                contents : &bytemuck::cast_slice(&pose_buf.iter().map(|m|m.to_cols_array()).flatten().collect::<Vec<f32>>()),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        );
        let animated_bind_group = device.create_bind_group
        (
            &BindGroupDescriptor
            {
                label: Some("Anim State"), 
                layout: &anim_layout, 
                entries: 
                &[
                    BindGroupEntry
                    {
                        binding: 0,
                        resource:  wgpu::BindingResource::Buffer
                        (
                            wgpu::BufferBinding
                            {
                                buffer: &tf_uniform,
                                offset: 0,
                                size: wgpu::BufferSize::new(std::mem::size_of::<[[f32;4]; 4]>() as _),
                            }
                        ),
                    },
                    BindGroupEntry
                    {
                        binding: 1,
                        resource:  wgpu::BindingResource::Buffer
                        (
                            wgpu::BufferBinding
                            {
                                buffer: &color_store,
                                offset: 0,
                                size: wgpu::BufferSize::new(std::mem::size_of::<[[f32;4]; NUM_COLORS]>() as _),
                            },
                        ),
                    },
                    BindGroupEntry
                    {
                        binding: 2,
                        resource:  wgpu::BindingResource::Buffer
                        (
                            wgpu::BufferBinding
                            {
                                buffer: &anim_buf,
                                offset: 0,
                                size: wgpu::BufferSize::new(std::mem::size_of::<[[f32;16]; NUM_BONES]>() as _),
                            }
                        ),
                    },
                ]
            }
        );
        return GPUAnimatedModelInstance 
        {
            model_id: instance.model_id().to_string(),
            pose_buffer: pose_buf,
            inverse_pose_buffer: inv_buf,
            tf_uniform: tf_uniform,
            animation_state_uniform: anim_buf,
            animated_bind_group : animated_bind_group,
            color_store: color_store,
        };
    }

    pub fn update_animated_model_instance(&mut self, instance: &AnimatedModelInstance, queue: &wgpu::Queue)
    {
        queue.write_buffer
        (
            &self.tf_uniform,
            0,
            &bytemuck::bytes_of( &instance.transform().to_cols_array() )
        );
        instance.current_pose().transforms(instance.anim_state().armature.as_ref(), &mut self.pose_buffer, &mut self.inverse_pose_buffer);
        queue.write_buffer
        (
            &self.animation_state_uniform,
            0,
            &bytemuck::cast_slice(&self.pose_buffer.iter().map(|m|m.to_cols_array()).flatten().collect::<Vec<f32>>()[0..(instance.anim_state().armature.num_bones()*16)]),
        );
    }

    pub fn destroy(self)
    {
        self.tf_uniform.destroy();
        self.animation_state_uniform.destroy();
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct TransformUniform
{
    mat: [[f32; 4]; 4],
}

