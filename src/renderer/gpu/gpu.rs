use std::collections::HashMap;

use wgpu::util::DeviceExt;

use crate::renderer::render_state::{render_state::RenderGroup, common::Id};

use super::{gpu_model::{GPUStaticModelInstance, GPUAnimatedModelInstance}, gpu_light::GPULightStore, gpu_camera::GPUCamera};



pub struct GPUState
{
    static_model_instances : HashMap<Id, GPUStaticModelInstance>,
    animated_model_instances : HashMap<Id, GPUAnimatedModelInstance>,
    cameras: HashMap<Id, GPUCamera>,
    light_groups : HashMap<Id, GPULightStore>,
    pub light_layout: wgpu::BindGroupLayout,
    pub static_layout: wgpu::BindGroupLayout,
    pub anim_layout: wgpu::BindGroupLayout,
    pub camera_layout: wgpu::BindGroupLayout,
}

impl GPUState
{
    pub fn new(device : &wgpu::Device) -> Self
    {
        //I love refrigerators
        return GPUState
        {
            light_groups: HashMap::new(),
            static_model_instances: HashMap::new(),
            animated_model_instances: HashMap::new(),
            cameras: HashMap::new(),
            light_layout: device.create_bind_group_layout(&super::gpu_light::LIGHT_LAYOUT),
            static_layout: device.create_bind_group_layout(&super::gpu_model::STATIC_LAYOUT),
            anim_layout: device.create_bind_group_layout(&super::gpu_model::SKINNED_LAYOUT),
            camera_layout: device.create_bind_group_layout(&super::gpu_camera::CAMERA_LAYOUT),
        };
    }

    pub fn get_depth_view(&self, camera_id: Id) -> Option<&wgpu::TextureView>
    {
        if let Option::Some(cam) = self.cameras.get(&camera_id)
        {
            return Option::Some(&cam.depth_view);
        }
        return Option::None;
    }

    pub fn get_camera_bind_group(&self, camera_id: Id) -> Option<&wgpu::BindGroup>
    {
        if let Option::Some(cam) = self.cameras.get(&camera_id)
        {
            return Option::Some(&cam.camera_bind_group);
        }
        return Option::None;
    }

    pub fn get_light_bind_group(&self, group_id: Id) -> Option<&wgpu::BindGroup>
    {
        if let Option::Some(light_group) = self.light_groups.get(&group_id)
        {
            return Option::Some(&light_group.bind_group);
        }
        return Option::None;
    }

    pub fn get_static_instance(&self, id: Id) -> Option<&GPUStaticModelInstance>
    {
        return self.static_model_instances.get(&id);
    }

    pub fn get_animated_instance(&self, id: Id) -> Option<&GPUAnimatedModelInstance>
    {
        return self.animated_model_instances.get(&id);
    }

    pub fn add_render_group(&mut self, group: &RenderGroup, device: &wgpu::Device)
    {
        self.light_groups.insert(group.id(), GPULightStore::new(group.get_point_lights(), group.get_spot_lights(), device, &self.light_layout));
    }

    pub fn update(&mut self, group: &RenderGroup, device: &wgpu::Device, queue: &wgpu::Queue)
    {
        //update data from changes in the scene since last frame
        for stat_mod in group.get_added_static_models()
        {
            self.static_model_instances.insert(*stat_mod, GPUStaticModelInstance::new(group.get_static_model(*stat_mod).unwrap(), &self.static_layout, device));
        }
        for stat_mod in group.get_removed_static_models()
        {
            self.destroy_model_instance(*stat_mod);
        }
        for stat_mod in group.get_static_models()
        {
            if stat_mod.dirty() 
            {
                if let Some(gpu_mod) = self.static_model_instances.get(&stat_mod.id())
                {
                    gpu_mod.update_static_model_instance(stat_mod, queue);
                }
            }
        }
        
        for anim_mod in group.get_added_animated_models()
        {
            self.animated_model_instances.insert(*anim_mod, GPUAnimatedModelInstance::new(&self.anim_layout, group.get_animated_model(*anim_mod).unwrap(), device));
        }
        for anim_mod in group.get_removed_animated_models()
        {
            self.destroy_model_instance(*anim_mod);
        }
        for anim_mod in group.get_animated_models()
        {
            if anim_mod.dirty()
            {
                if let Some(gpu_anim_mod) = self.animated_model_instances.get_mut(&anim_mod.id())
                {
                    gpu_anim_mod.update_animated_model_instance(anim_mod, queue);
                }
            }
        }
        
        if let Option::Some(light_group) = self.light_groups.get_mut(&group.id())
        {
            for light in group.get_added_point_lights()
            {
                light_group.add_point_light(group.get_point_light(*light).unwrap());
            }
            for light in group.get_point_lights()
            {
                if light.is_dirty()
                {
                    light_group.update_point_light(light);
                }
            }
            for light in group.get_removed_point_lights()
            {
                light_group.remove_point_light(*light);
            }

            for light in group.get_added_spot_lights()
            {
                light_group.add_spot_light(group.get_spot_light(*light).unwrap());
            }
            for light in group.get_spot_lights()
            {
                if light.is_dirty()
                {
                    light_group.update_spot_light(light);
                }
            }
            for light in group.get_removed_spot_lights()
            {
                light_group.remove_spot_light(*light);
            }
        }

        for cam in group.get_cameras()
        {
            if let Some(gpu_cam) = self.cameras.get(&cam.id())
            {
                gpu_cam.update_camera(cam, queue);
            }
        }
    }
    
    fn destroy_model_instance(&mut self, id: Id)
    {
        if let Some(inst) = self.animated_model_instances.remove(&id)
        {
            inst.destroy();
        }
        if let Some(inst) = self.static_model_instances.remove(&id)
        {
            inst.destroy();
        }
    }

    pub fn add_camera(&mut self, camera_id: Id, width: u32, height: u32, device: &wgpu::Device)
    {
        self.cameras.insert(camera_id, GPUCamera::new(device, &self.camera_layout, width, height));
    }
}