use std::collections::HashMap;

use wgpu::util::DeviceExt;

use crate::renderer::render_state::{light::{PointLight, SpotLight}, common::Id};

const POINT_LIGHT_TYPE : u32 = 0;
const SPOT_LIGHT_TYPE : u32 = 1;

const NUM_CLUSTERS_X : usize = 16;
const NUM_CLUSTERS_Y : usize = 9;
const NUM_CLUSTERS_Z : usize = 24;
const NUM_LIGHTS_PER_CLUSTER : usize = 32;

const NUM_SPOTS_PER_STAGE : usize = 1024;
const NUM_POINTS_PER_STAGE : usize = 1024;

//Scene bind group is re-used if multiple players are in one scene.
//The binding will last for the duration of a given render pass.
pub static LIGHT_LAYOUT : wgpu::BindGroupLayoutDescriptor = wgpu::BindGroupLayoutDescriptor
{
    label : Some("Scene Parameters"),
    entries : 
    &[
        //PointLights
        wgpu::BindGroupLayoutEntry
        {
            binding: 0,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Buffer 
            {
                ty: wgpu::BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        },
        //SpotLights
        wgpu::BindGroupLayoutEntry
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
        },
    ],
};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct PointLightStore
{
    lights : [GPUPointLight; NUM_POINTS_PER_STAGE],
    count : u32,
    padding: [u8; 12],
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct SpotLightStore
{
    lights : [GPUSpotLight; NUM_SPOTS_PER_STAGE],
    count : u32,
    padding: [u8; 12],
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GPUPointLight
{
    color :  [f32; 3],
    intensity : f32,
    loc : [f32; 3],
    radius : f32,
    cutoff : f32,
    padding: [u8; 12],
} //36 bytes - 16 byte alignment

impl GPUPointLight
{
    pub fn new(point_light: &PointLight) -> Self
    {
        return GPUPointLight 
        { 
            color: point_light.light.color.data.truncate().to_array(), 
            intensity: point_light.light.intensity, 
            loc: point_light.light.location.to_array(), 
            radius: point_light.light.radius,
            cutoff: point_light.light.cutoff_distance,
            padding: [0; 12],
        };
    }
    pub fn empty() -> Self
    {
        return GPUPointLight 
        {
            color: [0.0;3], 
            intensity: -1.0, //flag if we ever need to know for sure this is empty
            loc: [0.0;3], 
            radius: 0.0, 
            cutoff: 0.0,
            padding: [0; 12],
        };
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct GPUSpotLight
{
    color:  [f32; 3], //rgb
    intensity: f32,
    loc: [f32; 3],
    radius: f32,
    dir: [f32; 3],
    cutoff: f32,
} //48 bytes - 16 byte alignment

impl GPUSpotLight
{
    fn new(spot_light: &SpotLight) -> Self
    {
        return GPUSpotLight
        {
            color: spot_light.light.color.data.truncate().to_array(), 
            intensity: spot_light.light.intensity, 
            loc: spot_light.light.location.to_array(), 
            radius: spot_light.light.radius,
            cutoff: spot_light.light.cutoff_distance,
            dir: spot_light.dir.to_array(),
        }
    }

    fn empty() -> Self
    {
        return GPUSpotLight{ color: [0.0;3], intensity: -1.0, loc: [0.0;3], radius: 0.0, dir: [0.0;3], cutoff: 0.0, };
    }
}

pub struct GPULightStore
{
    point_lights : PointLightStore,
    spot_lights : SpotLightStore,
    free_spots: Vec<usize>,
    free_points: Vec<usize>,
    point_light_buffer : wgpu::Buffer,
    spot_light_buffer : wgpu::Buffer,
    //maps light ids to their store index
    point_light_map : HashMap<Id, usize>,
    spot_light_map : HashMap<Id, usize>,
    pub bind_group: wgpu::BindGroup,
}

impl GPULightStore
{
    pub fn new<'a>(point_lights: impl Iterator<Item=&'a PointLight>, spot_lights: impl Iterator<Item=&'a SpotLight>, device : &wgpu::Device, light_layout: &wgpu::BindGroupLayout,) -> Self
    {
        let mut point_map = HashMap::<Id, usize>::new();
        let gpu_point_lights = point_lights.enumerate().map
        (
            |pair|
            {
                point_map.insert(pair.1.id, pair.0);
                GPUPointLight::new(pair.1)
            }
        ).collect::<Vec<_>>();
        let point_arr = 
        {
            let mut temp = [GPUPointLight::empty(); NUM_POINTS_PER_STAGE];
            let mut i = 0;
            while i < gpu_point_lights.len() && i < NUM_POINTS_PER_STAGE
            {
                temp[i] = gpu_point_lights[i];
                i += 1;
            }
            temp
        };
        let point_store = PointLightStore{ lights: point_arr, count: gpu_point_lights.len() as u32, padding: [0;12] };
        let point_light_buf = device.create_buffer_init
        (
            &wgpu::util::BufferInitDescriptor 
            {
                label: Some("Point Light Store"),
                contents: &bytemuck::cast_slice(&[point_store]),
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            }
        );
        let mut spot_map = HashMap::<Id, usize>::new();
        let gpu_spot_lights = spot_lights.enumerate().map
        (
            |pair|
            {
                spot_map.insert(pair.1.id, pair.0);
                GPUSpotLight::new(pair.1)
            }
        ).collect::<Vec<_>>();
        let spot_arr = 
        {
            let mut temp = [GPUSpotLight::empty(); NUM_SPOTS_PER_STAGE];
            let mut i = 0;
            while i < gpu_spot_lights.len() && i < NUM_SPOTS_PER_STAGE
            {
                temp[i] = gpu_spot_lights[i];
                i+=1;
            }
            temp
        };
        let spot_store = SpotLightStore{ lights: spot_arr, count: gpu_spot_lights.len() as u32, padding: [0;12] };
        let spot_light_buf = device.create_buffer_init
        (
            &wgpu::util::BufferInitDescriptor 
            {
                label: Some("Spot Light Store"),
                contents: &bytemuck::cast_slice(&[spot_store]),
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            }
        );
        let bind_group = device.create_bind_group
        (
            &wgpu::BindGroupDescriptor
            { 
                label: Some( "light bind group" ), 
                layout: &light_layout, 
                entries: 
                &[
                    wgpu::BindGroupEntry
                    {
                        binding: 0,
                        resource:  wgpu::BindingResource::Buffer
                        (
                            wgpu::BufferBinding
                            {
                                buffer: &point_light_buf,
                                offset: 0,
                                size: wgpu::BufferSize::new(std::mem::size_of::<PointLightStore>() as u64),
                            },
                        ),
                    },
                    wgpu::BindGroupEntry
                    {
                        binding: 1,
                        resource:  wgpu::BindingResource::Buffer
                        (
                            wgpu::BufferBinding
                            {
                                buffer: &spot_light_buf,
                                offset: 0,
                                size: wgpu::BufferSize::new(std::mem::size_of::<SpotLightStore>() as u64),
                            },
                        ),
                    }
                ]
            }
        );
        return GPULightStore 
        {
            point_lights : point_store,
            spot_lights: spot_store,
            point_light_map: point_map,
            spot_light_map: spot_map,
            free_points: vec![],
            free_spots: vec![],
            point_light_buffer: point_light_buf,
            spot_light_buffer: spot_light_buf,
            bind_group: bind_group,
        };
    }

    pub fn add_point_light(&mut self, light: &PointLight)
    {
        let index = 
        {
            if(self.free_spots.is_empty())
            {
                self.point_lights.count += 1;
                (self.point_lights.count - 1) as usize
            }
            else
            {
                self.free_points.pop().unwrap()
            }
        };
        self.point_lights.lights[index] = GPUPointLight::new(light);
        self.point_light_map.insert(light.id, index);
    }

    pub fn remove_point_light(&mut self, light: Id)
    {
        if let Option::Some(index) = self.point_light_map.remove(&light)
        {
            self.point_lights.lights[index] = GPUPointLight::empty();
            self.free_points.push(index);
        }
        
    }

    pub fn update_point_light(&mut self, light: &PointLight)
    {
        if let Option::Some(index) = self.point_light_map.get(&light.id)
        {
            self.point_lights.lights[*index] = GPUPointLight::new(light);
        }
    }

    pub fn add_spot_light(&mut self, light: &SpotLight)
    {
        let index = 
        {
            if(self.free_spots.is_empty())
            {
                self.spot_lights.count += 1;
                (self.spot_lights.count - 1) as usize
            }
            else
            {
                self.free_spots.pop().unwrap()
            }
        };
        self.spot_lights.lights[index] = GPUSpotLight::new(light);
        self.spot_light_map.insert(light.id, index);
    }

    pub fn remove_spot_light(&mut self, light: Id)
    {
        if let Option::Some(index) = self.spot_light_map.remove(&light)
        {
            self.spot_lights.lights[index] = GPUSpotLight::empty();
            self.free_spots.push(index);
        }
    }

    pub fn update_spot_light(&mut self, light: &SpotLight)
    {
        if let Option::Some(index) = self.spot_light_map.get(&light.id)
        {
            self.spot_lights.lights[*index] = GPUSpotLight::new(light);
        }
    }

}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct LightId
{
    light_index : u16,
    light_type : u16,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct LightGrid
{
    clusters : [[[[LightId; NUM_LIGHTS_PER_CLUSTER]; NUM_CLUSTERS_X]; NUM_CLUSTERS_Y]; NUM_CLUSTERS_Z]
}

//init lights

//update spot light

//update point light

//GPU objects 
//these should be 1-to-1 with shader bind groups and change when they change
//I'm trying to lay out my bind groups by how nested they are in the render pipeline. So At the highest level you've got the camera 
//because that's going to be the same as long as the view persists, followed by the scene, which stays the same for a given render pass, but will 
//change depending on where the player goes, then there's the model, which changes every time we render a different object.
/*struct GPUStage
{
    light_uniform: wgpu::Buffer,
    stage_bind_group: wgpu::BindGroup,
    spot_id_map: Vec<u32>, //for lookup of dynamic spot lights
    point_id_map: Vec<u32>, //for lookup of dynamic point lights
    free_spots: Vec<usize>,
    free_points: Vec<usize>,
}*/    
    
    /*fn load_lights(&mut self, id: &str, light_descriptor: StageLightingDescriptor)
    {
        let spot_lights = 
        {
            let mut ret_arr = [SpotLight::empty_light(); NUM_SPOTS_PER_STAGE];
            for i in 0..light_descriptor.spots.len()
            {
                let spot = light_descriptor.spots[i];
                ret_arr[i] = SpotLight::new(spot);
            }
            ret_arr
        };
        let spot_ids = light_descriptor.spots.iter().map(|spot| spot.node_id).collect::<Vec<_>>();
        let point_lights = 
        {
            let mut ret_arr = [PointLight::empty_light(); NUM_POINTS_PER_STAGE];
            for i in 0..light_descriptor.points.len()
            {
                let point = light_descriptor.points[i];
                ret_arr[i] = PointLight::new(point);
            }
            ret_arr
        };
        let point_ids = light_descriptor.points.iter().map(|point| point.node_id).collect::<Vec<_>>();
        let light_uniform = LightUniform
        {
            spot_lights: spot_lights,
            global_dir: GlobalDirectionalLight
            {
                color: light_descriptor.main.color.to_array(),
                intensity: light_descriptor.main.intensity,
                dir: light_descriptor.main.dir.to_array(),
                padding: [0;4],
            },
            point_lights: point_lights,
            ambient_color: light_descriptor.ambient_color.to_array(),
            ambient_intensity : light_descriptor.ambient_intensity,
            num_spots: light_descriptor.spots.len().try_into().unwrap(),
            num_points: light_descriptor.points.len().try_into().unwrap(),
        };

        let light_buf = self.device.create_buffer_init
        (
            &wgpu::util::BufferInitDescriptor 
            {
                label: Some("Light Uniform"),
                contents: &[bytemuck::cast(light_uniform)],
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        );

        let stage_group = self.device.create_bind_group
        (
            &BindGroupDescriptor
            { 
                label: Some("Scene"), 
                layout: &self.stage_layout, 
                entries: &
                [
                    BindGroupEntry
                    {
                        binding: 0,
                        resource:  wgpu::BindingResource::Buffer
                        (
                            wgpu::BufferBinding
                            {
                                buffer: &light_buf,
                                offset: 0,
                                size: wgpu::BufferSize::new(std::mem::size_of::<LightUniform>() as u64),
                            }
                        ),
                    }
                ] 
            }
        );

        self.stages.insert
        (
            id.to_string(), 
            GPUStage
            {
                light_uniform: light_buf,
                stage_bind_group: stage_group,
                spot_id_map: spot_ids,
                point_id_map: point_ids,
                free_spots: vec![],
                free_points: vec![],
            }
        );
    }

    fn add_spot_light(&mut self, stage: &str, spot_light: SpotLightDescriptor)
    {
        if let Some(stage) = self.stages.get_mut(stage)
        {
            if !stage.spot_id_map.iter().any(|t_id| *t_id == spot_light.node_id)
            {
                let offset = if stage.free_spots.is_empty() { stage.spot_id_map.len() } else { stage.free_spots.pop().unwrap() };
                self.queue.write_buffer
                (
                    &stage.light_uniform,
                    (std::mem::size_of::<SpotLight>() * offset).try_into().unwrap(),
                    &[bytemuck::cast(SpotLight::new(spot_light))]
                );
                stage.spot_id_map.push(spot_light.node_id);
                self.queue.write_buffer
                (
                    &stage.light_uniform,
                    (std::mem::size_of::<SpotLight>() * NUM_SPOTS_PER_STAGE + 
                    std::mem::size_of::<PointLight>() * NUM_POINTS_PER_STAGE + 
                    std::mem::size_of::<GlobalDirectionalLight>() + 
                    std::mem::size_of::<[f32;4]>()).try_into().unwrap(),
                    &[bytemuck::cast(stage.spot_id_map.len())],
                );
            }
        }
    }

    fn add_point_light(&mut self, stage: &str, point_light: PointLightDescriptor)
    {
        if let Some(stage) = self.stages.get_mut(stage)
        {
            if !stage.point_id_map.iter().any(|t_id| *t_id == point_light.node_id)
            {
                let offset = if stage.free_points.is_empty() { stage.point_id_map.len() } else { stage.free_points.pop().unwrap() };
                self.queue.write_buffer
                (
                    &stage.light_uniform,
                    (std::mem::size_of::<SpotLight>() * NUM_SPOTS_PER_STAGE + std::mem::size_of::<GlobalDirectionalLight>() + std::mem::size_of::<PointLight>() * offset).try_into().unwrap(),
                    &[bytemuck::cast(PointLight::new(point_light))]
                );
                stage.point_id_map.push(point_light.node_id);
                self.queue.write_buffer
                (
                    &stage.light_uniform,
                    (std::mem::size_of::<SpotLight>() * NUM_SPOTS_PER_STAGE + 
                    std::mem::size_of::<PointLight>() * NUM_POINTS_PER_STAGE + 
                    std::mem::size_of::<GlobalDirectionalLight>() + 
                    std::mem::size_of::<[f32;4]>()).try_into().unwrap(),
                    &[bytemuck::cast(stage.point_id_map.len())],
                );
            }
        }
    }

    fn remove_spot_light(&mut self, stage: &str, id: u32)
    {
        if let Some(stage) = self.stages.get_mut(stage)
        {
            if let Some(index) = stage.spot_id_map.iter().position(|t_id| *t_id == id)
            {
                //fill removed spot with a spotlight flagged to be invalid
                self.queue.write_buffer
                (
                    &stage.light_uniform,
                    (std::mem::size_of::<SpotLight>() * index).try_into().unwrap(),
                    &[bytemuck::cast(SpotLight::empty_light())]
                );
                //remove spotlight from id map
                stage.spot_id_map.remove(index);
                //add index to free list
                stage.free_spots.push(index);
            }
            
            //update the number of spot_lights
            self.queue.write_buffer
            (
                &stage.light_uniform,
                (std::mem::size_of::<SpotLight>() * NUM_SPOTS_PER_STAGE + 
                std::mem::size_of::<PointLight>() * NUM_POINTS_PER_STAGE + 
                std::mem::size_of::<GlobalDirectionalLight>() + 
                std::mem::size_of::<[f32;4]>()).try_into().unwrap(),
                &[bytemuck::cast(stage.spot_id_map.len())],
            );
        }
    }

    fn remove_point_light(&mut self, stage: &str, id: u32)
    {
        if let Some(stage) = self.stages.get_mut(stage)
        {
            if let Some(index) = stage.point_id_map.iter().position(|t_id| *t_id == id)
            {
                //fill removed point with a pointlight flagged to be invalid
                self.queue.write_buffer
                (
                    &stage.light_uniform,
                    (std::mem::size_of::<PointLight>() * index).try_into().unwrap(),
                    &[bytemuck::cast(PointLight::empty_light())]
                );
                //remove pointlight from id map
                stage.point_id_map.remove(index);
                //add index to free list
                stage.free_points.push(index);
                //update the number of point_lights
                self.queue.write_buffer
                (
                    &stage.light_uniform,
                    (std::mem::size_of::<SpotLight>() * NUM_SPOTS_PER_STAGE + 
                    std::mem::size_of::<PointLight>() * NUM_POINTS_PER_STAGE + 
                    std::mem::size_of::<GlobalDirectionalLight>() + 
                    std::mem::size_of::<[f32;4]>()).try_into().unwrap(),
                    &[bytemuck::cast(stage.point_id_map.len())],
                );
            }
            
        }
    }*/
