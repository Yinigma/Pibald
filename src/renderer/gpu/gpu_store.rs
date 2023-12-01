use std::collections::HashMap;

use wgpu::util::DeviceExt;

use crate::renderer::render_state::model::{StaticVertex, ShaderSlot, StaticModel, AnimatedModel, Triangle, Polygon};

const NUM_BONES_PER_VERT : usize = 8;

/*
Vertex format for models that don't have a armature
 */
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct GPUStaticVertex
{
    color : u32, //index of a color on the model
    position : [f32; 3], //man, I hope you know what these two are
    normal : [f32; 3],
}

pub fn get_static_vertex_layout<'a>() -> wgpu::VertexBufferLayout<'a>
{
    return wgpu::VertexBufferLayout 
    {
        array_stride: std::mem::size_of::<GPUStaticVertex>() as wgpu::BufferAddress,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: 
        &[
            wgpu::VertexAttribute 
            {
                offset: 0,
                shader_location: 2,
                format: wgpu::VertexFormat::Uint32,
            },
            wgpu::VertexAttribute 
            {
                offset: std::mem::size_of::<f32>() as wgpu::BufferAddress,
                shader_location: 0,
                format: wgpu::VertexFormat::Float32x3,
            },
            wgpu::VertexAttribute 
            {
                offset: std::mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                shader_location: 1,
                format: wgpu::VertexFormat::Float32x3,
            },
            
        ]
    };
}

/*
Vertex format for skinned meshes
 */
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct GPUAnimatedVertex
{
    //animation properties
    static_vert: GPUStaticVertex,
    rig_weights : [u16; NUM_BONES_PER_VERT], //8 values normalized from their uint values to [0,1] - converted from uints on the wgsl side
    rig_ids : [u8; NUM_BONES_PER_VERT], //corresponding bone indices - up to 256 bones allowed
}

pub fn get_animated_vertex_layout<'a>() -> wgpu::VertexBufferLayout<'a> 
{
    return wgpu::VertexBufferLayout
    {
        array_stride: std::mem::size_of::<GPUAnimatedVertex>() as wgpu::BufferAddress,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: 
        &[
            wgpu::VertexAttribute 
            {
                offset: 0,
                shader_location: 2,
                format: wgpu::VertexFormat::Uint32,
            },
            wgpu::VertexAttribute 
            {
                offset: std::mem::size_of::<f32>() as wgpu::BufferAddress,
                shader_location: 0,
                format: wgpu::VertexFormat::Float32x3,
            },
            wgpu::VertexAttribute 
            {
                offset: std::mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                shader_location: 1,
                format: wgpu::VertexFormat::Float32x3,
            },
            wgpu::VertexAttribute 
            {
                offset: std::mem::size_of::<[f32; 7]>() as wgpu::BufferAddress,
                shader_location: 3,
                format: wgpu::VertexFormat::Uint32x4,
            },
            wgpu::VertexAttribute
            {
                offset: (std::mem::size_of::<[u32; 4]>() + std::mem::size_of::<[f32; 7]>()) as wgpu::BufferAddress,
                shader_location: 4,
                format: wgpu::VertexFormat::Uint32x2,
            },
        ]
    };
}

#[derive(PartialEq, Eq, Hash)]
struct GPUMaterialId
{
    mat_id: String,
    model_id: String,
}

pub struct IndexBuffer
{
    pub buf: wgpu::Buffer,
    pub length: u32,
}

pub struct GPUStore
{
    static_vert_buffers: HashMap<String, wgpu::Buffer>,
    anim_vert_buffers: HashMap<String, wgpu::Buffer>,
    mat_index_buffers: HashMap<GPUMaterialId, IndexBuffer>,
    index_buffers: HashMap<String, IndexBuffer>,
}

impl GPUStore
{
    pub fn new() -> Self
    {
        GPUStore
        {
            static_vert_buffers : HashMap::new(),
            anim_vert_buffers : HashMap::new(),
            mat_index_buffers : HashMap::new(),
            index_buffers : HashMap::new(),
        }
    }

    pub fn get_static_vertex_buffer(&self, model_id: &String) -> Option<&wgpu::Buffer>
    {
        return self.static_vert_buffers.get(model_id);
    }

    pub fn get_animated_vertex_buffer(&self, anim_model_id: &String) -> Option<&wgpu::Buffer>
    {
        return self.anim_vert_buffers.get(anim_model_id);
    }

    pub fn get_index_buffer(&self, model_id: &String) -> Option<&IndexBuffer>
    {
        return self.index_buffers.get(model_id);
    }

    pub fn get_indices_for_material(&self, model_id: &String, material_slot: &String) -> Option<&IndexBuffer>
    {
        return self.mat_index_buffers.get(&GPUMaterialId { mat_id: material_slot.to_string(), model_id: model_id.to_string() });
    }

    fn load_static_verts(&mut self, id: &String, data: &Vec<StaticVertex>, polys: &Vec<Polygon>, device: &wgpu::Device)
    {
        let indices = polys.iter().map(|poly| poly.tris.iter().map(|tri| tri.indices).flatten() ).flatten().map(|size| size as u32).collect::<Vec<_>>();
        self.index_buffers.insert
        (
            id.clone(),
            IndexBuffer 
            { 
                buf: device.create_buffer_init
                (
                    &wgpu::util::BufferInitDescriptor 
                    {
                        label: Some("Index Buffer"),
                        contents: bytemuck::cast_slice(&indices),
                        usage: wgpu::BufferUsages::INDEX,
                    }
                ), 
                length: indices.len() as u32,
            }
        );
        let gpu_verts = data.iter().map
        (
            |vert| GPUStaticVertex 
            { 
                position: vert.loc.to_array(),
                normal: vert.norm.to_array(),
                color: vert.col as u32,
            }
        ).collect::<Vec<GPUStaticVertex>>();
        self.static_vert_buffers.insert
        (
            id.clone(), 
            device.create_buffer_init
            (
                &wgpu::util::BufferInitDescriptor 
                {
                    label: Some("Static Vertex Buffer"),
                    contents: bytemuck::cast_slice(&gpu_verts),
                    usage: wgpu::BufferUsages::VERTEX,
                }
            )
        );
    }

    fn load_material_mappings(&mut self, id: &String, polys: &Vec<Polygon>, materials: &HashMap<String, ShaderSlot>, device: &wgpu::Device)
    {
        for mapping_key in materials.keys()
        {
            let indices = materials[mapping_key].tris.iter().map(|poly| polys[*poly].tris.iter().map(|tri| tri.indices).flatten() ).flatten().map(|size| size as u32).collect::<Vec<_>>();
            
            self.mat_index_buffers.insert
            (
                GPUMaterialId { mat_id: mapping_key.clone(), model_id: id.clone() },
                IndexBuffer 
                { 
                    buf: device.create_buffer_init
                    (
                        &wgpu::util::BufferInitDescriptor 
                        {
                            label: Some("Index Buffer"),
                            contents: bytemuck::cast_slice(&indices),
                            usage: wgpu::BufferUsages::INDEX,
                        }
                    ),
                    length: indices.len() as u32,
                }
            );
        }
    }

    pub fn load_static_model(&mut self, model: &StaticModel, device: &wgpu::Device)
    {
        self.load_static_verts(&model.id, &model.vertices, &model.model_data.polygons, device);
        self.load_material_mappings(&model.id, &model.model_data.polygons, &model.model_data.shader_slots, device);
    }

    pub fn load_skinned_model( &mut self, model: &AnimatedModel, device: &wgpu::Device)
    {
        self.load_material_mappings(&model.id, &model.model_data.polygons, &model.model_data.shader_slots, device);
        let indices = model.model_data.polygons.iter().map(|poly| poly.tris.iter().map(|tri| tri.indices).flatten() ).flatten().map(|size| size as u32).collect::<Vec<_>>();
        self.index_buffers.insert
        (
            model.id.clone(),
            IndexBuffer 
            { 
                buf: device.create_buffer_init
                (
                    &wgpu::util::BufferInitDescriptor 
                    {
                        label: Some("Index Buffer"),
                        contents: bytemuck::cast_slice(&indices),
                        usage: wgpu::BufferUsages::INDEX,
                    }
                ), 
                length: indices.len() as u32,
            }
        );
        let gpu_verts = model.vertices.iter().map
        (
            |vert|
            {
                
                let mut weights: [u16; NUM_BONES_PER_VERT] = [0; NUM_BONES_PER_VERT];
                let mut ids: [u8; NUM_BONES_PER_VERT] = [std::u8::MAX; NUM_BONES_PER_VERT];
                let mut i = 0;
                while i < vert.weights.len() && i < NUM_BONES_PER_VERT
                {
                    weights[i] = (vert.weights[i].weight.get_val() * (std::u16::MAX as f32)).round() as u16;
                    ids[i] = vert.weights[i].index as u8; 
                    i+=1;
                }
                
                GPUAnimatedVertex
                {
                    static_vert: GPUStaticVertex 
                    {
                        position: vert.vert.loc.to_array(),
                        normal: vert.vert.norm.to_array(),
                        color: vert.vert.col as u32,
                    },
                    rig_weights: weights,
                    rig_ids: ids,
                }
            }
        ).collect::<Vec<GPUAnimatedVertex>>();

        self.anim_vert_buffers.insert
        (
            model.id.clone(),
            device.create_buffer_init
            (
                &wgpu::util::BufferInitDescriptor 
                {
                    label: Some("Anim Vertex Buffer"),
                    contents: bytemuck::cast_slice(&gpu_verts),
                    usage: wgpu::BufferUsages::VERTEX,
                }
            )
        );
    }

    pub fn unload_model(&mut self, handle : &String)
    {
        self.mat_index_buffers.iter().for_each(|pair| pair.1.buf.destroy());
        self.mat_index_buffers.retain(|k, _v| !k.model_id.eq(handle) );
        if let Some(buf) = self.anim_vert_buffers.remove(handle)
        {
            buf.destroy();
        }
        if let Some(buf) = self.static_vert_buffers.remove(handle)
        {
            buf.destroy();
        }
    }
}