use std::{rc::Rc, collections::HashMap, process::id};

use wgpu::{Device, Queue, RenderPipeline, BindGroupLayout, VertexState, PrimitiveState, PrimitiveTopology, FrontFace, Face, PolygonMode, MultisampleState, include_wgsl, SurfaceConfiguration};

use crate::renderer::render_state::{render_state::RenderState, common::Id};

use super::{gpu::GPUState, gpu_store::GPUStore, gpu_camera::GPUCamera};

fn create_static_pipeline(bg_layouts: &[&BindGroupLayout], config: &wgpu::SurfaceConfiguration, device: &Device,) -> wgpu::RenderPipeline
{
    let static_shader = device.create_shader_module(include_wgsl!("static.wgsl") );

    let static_pipeline_layout = device.create_pipeline_layout
    (
        &wgpu::PipelineLayoutDescriptor 
        {
            label: Some("Static Render Pipeline Layout"),
            bind_group_layouts: bg_layouts,
            push_constant_ranges: &[],
        }
    );
    
    return device.create_render_pipeline
    (
        &wgpu::RenderPipelineDescriptor 
        {
            label: Some("Static Render Pipeline"),
            layout: Some(&static_pipeline_layout),
            vertex: VertexState
            {
                module: &static_shader,
                entry_point: "vs_static",
                buffers: &[super::gpu_store::get_static_vertex_layout(),],
            },
            fragment: Some
            (
                wgpu::FragmentState 
                {
                    module: &static_shader,
                    entry_point: "fs_main",
                    targets: 
                    &[
                        Some
                        (
                            wgpu::ColorTargetState
                            {
                                format: config.format,
                                blend: Some(wgpu::BlendState::REPLACE),
                                write_mask: wgpu::ColorWrites::ALL,
                            }
                        )
                    ],
                }
            ),
            primitive: PrimitiveState 
            {
                topology: PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: FrontFace::Ccw, 
                cull_mode: Some(Face::Back),
                // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                polygon_mode: PolygonMode::Fill,
                // Requires Features::DEPTH_CLIP_CONTROL
                unclipped_depth: false,
                // Requires Features::CONSERVATIVE_RASTERIZATION
                conservative: false,
            },
            depth_stencil: Some
            (
                wgpu::DepthStencilState 
                {
                    format: super::gpu_camera::DEPTH_FORMAT,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::Less, // 1.
                    stencil: wgpu::StencilState::default(), // 2.
                    bias: wgpu::DepthBiasState::default(),
                }
            ),
            multisample: MultisampleState 
            {
                count: 1, 
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        }
    );
}

fn create_animated_pipeline(bg_layouts: &[&BindGroupLayout], config: &wgpu::SurfaceConfiguration, device: &Device,) -> wgpu::RenderPipeline
{
    
    let animated_shader = device.create_shader_module(include_wgsl!("animated.wgsl") );

    let animated_pipeline_layout = device.create_pipeline_layout
    (
        &wgpu::PipelineLayoutDescriptor 
        {
            label: Some("Animated Render Pipeline Layout"),
            bind_group_layouts: bg_layouts,
            push_constant_ranges: &[],
        }
    );
    
    return device.create_render_pipeline
    (
        &wgpu::RenderPipelineDescriptor 
        {
            label: Some("Static Render Pipeline"),
            layout: Some(&animated_pipeline_layout),
            vertex: VertexState
            {
                module: &animated_shader,
                entry_point: "vs_animated",
                buffers: &[super::gpu_store::get_animated_vertex_layout(),],
            },
            fragment: Some
            (
                wgpu::FragmentState 
                {
                    module: &animated_shader,
                    entry_point: "fs_main",
                    targets: 
                    &[
                        Some
                        (
                            wgpu::ColorTargetState
                            {
                                format: config.format,
                                blend: Some(wgpu::BlendState::REPLACE),
                                write_mask: wgpu::ColorWrites::ALL,
                            }
                        )
                    ],
                }
            ),
            primitive: PrimitiveState 
            {
                topology: PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: FrontFace::Ccw, 
                cull_mode: Some(Face::Back),
                // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                polygon_mode: PolygonMode::Fill,
                // Requires Features::DEPTH_CLIP_CONTROL
                unclipped_depth: false,
                // Requires Features::CONSERVATIVE_RASTERIZATION
                conservative: false,
            },
            depth_stencil: Some
            (
                wgpu::DepthStencilState 
                {
                    format: super::gpu_camera::DEPTH_FORMAT,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::Less,
                    stencil: wgpu::StencilState::default(),
                    bias: wgpu::DepthBiasState::default(),
                }
            ),
            multisample: MultisampleState 
            {
                count: 1, 
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        }
    );
}

pub struct Renderer
{
    gpu_state: GPUState,
    static_pipeline : RenderPipeline,
    animated_pipeline: RenderPipeline,
    camera_outputs: HashMap<Id, wgpu::TextureView>,
}

impl Renderer
{
    pub fn attach_camera(&mut self, camera_id: Id, width: u32, height: u32, device: &Device,)
    {
        self.gpu_state.add_camera(camera_id, width, height, device);
    }

    pub fn set_output_view(&mut self, id: Id, view: wgpu::TextureView)
    {
        self.camera_outputs.insert(id, view);
    }

    pub fn new(device: &Device, config: &SurfaceConfiguration)->Self
    {
        let gpu_state = GPUState::new(device);
        Renderer 
        {
            static_pipeline: create_static_pipeline
            (
                &
                [
                    &gpu_state.camera_layout,
                    &gpu_state.light_layout,
                    &gpu_state.static_layout,
                ], 
                config, 
                device
            ),
            animated_pipeline: create_animated_pipeline
            (
                &
                [
                    &gpu_state.camera_layout,
                    &gpu_state.light_layout,
                    &gpu_state.anim_layout,
                ], 
                config, 
                device
            ),
            gpu_state: gpu_state,
            camera_outputs: HashMap::new(), 
        }
    }

    pub fn push_buffer_updates(&mut self, render_state: &RenderState, device: &Device, queue: &Queue)
    {
        
        for group in render_state.get_groups()
        {
            if self.gpu_state.get_light_bind_group(group.id()).is_none()
            {
                self.gpu_state.add_render_group(group, device);
            }
            self.gpu_state.update(group, device, queue);
        }
    }

    pub fn render(&self, render_state: &RenderState, gpu_store: &GPUStore, device: &Device, queue: &Queue)
    {
        let mut encoder = device.create_command_encoder
        (
            &wgpu::CommandEncoderDescriptor 
            {
                label: Some("Render Encoder"),
            }
        );
        for group in render_state.get_groups()
        {
            for cam in group.get_cameras()
            {
                if let Some(view) = self.camera_outputs.get(&cam.id())
                {
                    {
                        let mut render_pass = encoder.begin_render_pass
                        (
                            &wgpu::RenderPassDescriptor 
                            {
                                label: Some("Render Pass"),
                                color_attachments: 
                                &[
                                    Some
                                    (
                                        wgpu::RenderPassColorAttachment 
                                        {
                                            view: view,
                                            resolve_target: None,
                                            ops: wgpu::Operations 
                                            {
                                                load: wgpu::LoadOp::Clear(wgpu::Color{r: 0.1, g: 0.2,b: 0.3, a: 1.0,}),
                                                store: true,
                                            }
                                        }
                                    )
                                ],
                                depth_stencil_attachment: 
                                Some
                                (
                                    wgpu::RenderPassDepthStencilAttachment 
                                    {
                                        view: &self.gpu_state.get_depth_view(cam.id()).unwrap(),
                                        depth_ops: Some(wgpu::Operations { load: wgpu::LoadOp::Clear(1.0), store: true,}),
                                        stencil_ops: None,
                                    }
                                ),
                            }
                        );
                        render_pass.set_pipeline(&self.static_pipeline);
                        render_pass.set_bind_group(0, &self.gpu_state.get_camera_bind_group(cam.id()).unwrap(), &[]);
                        render_pass.set_bind_group(1, self.gpu_state.get_light_bind_group(group.id()).unwrap(), &[]);

                        for static_model in group.get_static_models_culled(cam)
                        {
                            if let Option::Some(gpu_instance) = self.gpu_state.get_static_instance(static_model.id())
                            {
                                render_pass.set_bind_group(2, &gpu_instance.static_bind_group, &[]);
                                if let Option::Some(v_buf) = gpu_store.get_static_vertex_buffer(static_model.model_id())
                                {
                                    render_pass.set_vertex_buffer(0, v_buf.slice(..));
                                    if let Option::Some(indx_buf) = gpu_store.get_index_buffer(static_model.model_id())
                                    {
                                        render_pass.set_index_buffer(indx_buf.buf.slice(..), wgpu::IndexFormat::Uint32);
                                        render_pass.draw_indexed(0..indx_buf.length, 0, 0..1);
                                    }
                                    /*for mat in &static_model.model().model_data.shader_slots
                                    {
                                        if let Option::Some(indx_buf) = gpu_store.get_indices_for_material(static_model.model_id(), mat.0)
                                        {
                                            render_pass.set_index_buffer(indx_buf.buf.slice(..), wgpu::IndexFormat::Uint32);
                                            render_pass.draw_indexed(0..indx_buf.length, 0, 0..1);
                                        }
                                    }*/
                                }
                            }
                        }
                    }
                    
                    {
                        let mut animated_render_pass = encoder.begin_render_pass
                        (
                            &wgpu::RenderPassDescriptor 
                            {
                                label: Some("Render Pass"),
                                color_attachments: 
                                &[
                                    Some
                                    (
                                        wgpu::RenderPassColorAttachment 
                                        {
                                            view: view,
                                            resolve_target: None,
                                            ops: wgpu::Operations 
                                            {
                                                load: wgpu::LoadOp::Load,
                                                store: true,
                                            }
                                        }
                                    )
                                ],
                                depth_stencil_attachment: 
                                Some
                                (
                                    wgpu::RenderPassDepthStencilAttachment 
                                    {
                                        view: &self.gpu_state.get_depth_view(cam.id()).unwrap(),
                                        depth_ops: Some(wgpu::Operations { load: wgpu::LoadOp::Load, store: true,}),
                                        stencil_ops: None,
                                    }
                                ),
                            }
                        );
                        animated_render_pass.set_pipeline(&self.animated_pipeline);
                        animated_render_pass.set_bind_group(0, &self.gpu_state.get_camera_bind_group(cam.id()).unwrap(), &[]);
                        animated_render_pass.set_bind_group(1, self.gpu_state.get_light_bind_group(group.id()).unwrap(), &[]);

                        for animated_model in group.get_animated_models_culled(cam)
                        {
                            if let Option::Some(gpu_instance) = self.gpu_state.get_animated_instance(animated_model.id())
                            {
                                animated_render_pass.set_bind_group(2, &gpu_instance.animated_bind_group, &[]);
                                if let Option::Some(v_buf) = gpu_store.get_animated_vertex_buffer(animated_model.model_id())
                                {
                                    animated_render_pass.set_vertex_buffer(0, v_buf.slice(..));
                                    if let Option::Some(indx_buf) = gpu_store.get_index_buffer(animated_model.model_id())
                                    {
                                        animated_render_pass.set_index_buffer(indx_buf.buf.slice(..), wgpu::IndexFormat::Uint32);
                                        animated_render_pass.draw_indexed(0..indx_buf.length, 0, 0..1);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        queue.submit(std::iter::once(encoder.finish()));
    }
}