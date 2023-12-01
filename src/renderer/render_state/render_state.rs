use std::collections::HashMap;
use std::rc::Rc;

use glam::Mat4;
use super::animation::AnimationState;
use super::common::{IdGenerator, Id};
use super::light::{PointLight, SpotLight, SpotLightDescriptor, PointLightDescriptor};
use super::model::{StaticModelInstance, AnimatedModelInstance, StaticModel, AnimatedModel};
use super::view::{Camera, CameraDescriptor};

pub struct RenderState
{
    id_generator: IdGenerator,
    groups: HashMap<Id, RenderGroup>,
}

impl RenderState
{
    pub fn new() -> Self
    {
        return RenderState 
        { 
            id_generator: IdGenerator::new(), 
            groups: HashMap::new(),
        }
    }

    pub fn add_group(&mut self) -> Id
    {
        let id = self.id_generator.get_id();
        self.groups.insert(id, RenderGroup::new(id));
        return id;
    }

    pub fn get_groups(&self) -> impl Iterator<Item = &RenderGroup>
    {
        return self.groups.iter().map(|group| group.1);
    }

    pub fn get_group_mut(&mut self, group_id: Id) -> Option<&mut RenderGroup>
    {
        return self.groups.get_mut(&group_id);
    }

    pub fn add_static_model(&mut self, group_id: Id, model: &StaticModel, tf: Mat4) -> Option<Id>
    {
        if let Some(group) = self.groups.get_mut(&group_id)
        { 
            let id = self.id_generator.get_id();
            group.add_static_model(id, model, tf);
            return Option::Some(id);
        }
        return Option::None;
    }

    pub fn add_animated_model(&mut self, group_id: Id, model: &AnimatedModel, tf: Mat4, anim_state: AnimationState) -> Option<Id>
    {
        if let Some(group) = self.groups.get_mut(&group_id)
        { 
            let id = self.id_generator.get_id();
            group.add_animated_model(id, model, tf, anim_state);
            return Option::Some(id);
        }
        return Option::None;
    }

    pub fn add_camera(&mut self, group_id: Id, cam: CameraDescriptor) -> Option<Id>
    {
        if let Some(group) = self.groups.get_mut(&group_id)
        {
            let id = self.id_generator.get_id();
            group.add_camera(id, cam);
            return Option::Some(id);
        }
        return Option::None;
    }

    pub fn add_spot_light(&mut self, group_id: Id, spot: SpotLightDescriptor) -> Option<Id>
    {
        if let Some(group) = self.groups.get_mut(&group_id)
        {
            let id = self.id_generator.get_id();
            group.add_spot_light(id, spot);
            return Option::Some(id);
        }
        return Option::None;
    }

    pub fn add_point_light(&mut self, group_id: Id, point: PointLightDescriptor) -> Option<Id>
    {
        if let Some(group) = self.groups.get_mut(&group_id)
        {
            let id = self.id_generator.get_id();
            group.add_point_light(id, point);
            return Option::Some(id);
        }
        return Option::None;
    }

    pub fn remove_item(&mut self, id: Id, )
    {
        if self.id_generator.free_id(id)
        {
            for group in self.groups.values_mut()
            {
                group.remove_item(id);
            }
        }
    }

    pub fn clear_dirty_state(&mut self)
    {
        for group in self.groups.values_mut()
        {
            group.clear_dirty_state();
        }
    }
}

//terminal struct for this aggregate
pub struct RenderGroup
{
    id: Id,
    static_models: HashMap<Id, StaticModelInstance>,
    animated_models: HashMap<Id, AnimatedModelInstance>,
    cameras: HashMap<Id, Camera>,
    spot_lights: HashMap<Id, SpotLight>,
    point_lights: HashMap<Id, PointLight>,
    added_static_models: Vec<Id>,
    removed_static_models: Vec<Id>,
    added_animated_models: Vec<Id>,
    removed_animated_models: Vec<Id>,
    added_point_lights: Vec<Id>,
    removed_point_lights: Vec<Id>,
    added_spot_lights: Vec<Id>,
    removed_spot_lights: Vec<Id>,
    added_cameras: Vec<Id>,
    removed_cameras: Vec<Id>,

}

impl RenderGroup
{
    pub fn new(id: Id) -> Self
    {
        return RenderGroup
        {
            id: id,
            static_models: HashMap::new(),
            animated_models: HashMap::new(),
            cameras: HashMap::new(),
            spot_lights: HashMap::new(),
            point_lights: HashMap::new(),
            added_static_models: Vec::new(),
            removed_static_models: Vec::new(),
            added_animated_models: Vec::new(),
            removed_animated_models: Vec::new(),
            added_point_lights: Vec::new(),
            removed_point_lights: Vec::new(),
            added_spot_lights: Vec::new(),
            removed_spot_lights: Vec::new(),
            added_cameras: Vec::new(),
            removed_cameras: Vec::new(),
        };
    }

    pub fn id(&self) -> Id
    {
        return self.id;
    }

    pub fn get_cameras(&self, ) -> impl Iterator<Item = &Camera>
    {
        return self.cameras.values();
    }

    pub fn get_static_models(&self) -> impl Iterator<Item=&StaticModelInstance>
    {
        return self.static_models.values();
    }

    pub fn get_animated_models(&self) -> impl Iterator<Item=&AnimatedModelInstance>
    {
        return self.animated_models.values();
    }

    pub fn get_spot_lights(&self) -> impl Iterator<Item=&SpotLight>
    {
        return self.spot_lights.values();
    }

    pub fn get_point_lights(&self) -> impl Iterator<Item=&PointLight>
    {
        return self.point_lights.values();
    }

    //z-sorted list of models culled using the given camera id
    pub fn get_static_models_culled(&self, cam: &Camera) -> Vec<&StaticModelInstance>
    {
        return self.static_models.iter().map(|model| model.1).collect::<Vec<_>>();
        /*let ro_dest = 
        {
            let mut dest = vec![];
            dest.extend
            (
                self.static_models.iter().filter
                (
                    |model| cam.bbox_in_view(model.1.bounding_box(), model.1.transform())
                ).map(|model| model.1)
            );
            dest.sort_by
            (
                |a, b| 
                (a.transform().col(3).truncate() - cam.location()).dot(cam.forward()).total_cmp(&(b.transform().col(3).truncate() - cam.location()).dot(cam.forward()))
            );
            dest
        };
        return ro_dest;*/
    }

    pub fn get_animated_models_culled(&self, cam: &Camera) -> Vec<&AnimatedModelInstance>
    {
        let ro_dest = 
        {
            let mut dest = vec![];
            dest.extend
            (
            self.animated_models.iter().filter
                (
                    |model| cam.bbox_in_view(model.1.bounding_box(), model.1.transform())
                ).map(|model| model.1)
            );
            dest.sort_by
            (
            |a, b| 
                (a.transform().col(3).truncate() - cam.location()).dot(cam.forward()).total_cmp(&(b.transform().col(3).truncate() - cam.location()).dot(cam.forward()))
            );
            dest
        };
        return ro_dest;
    }

    //crud methods - just let your eyes glaze over. Mmmm... Glaze...

    pub fn add_static_model(&mut self, id: Id, model: &StaticModel, tf: Mat4)
    { 
        self.added_static_models.push(id);
        self.static_models.insert(id, StaticModelInstance::new(id, model, tf) ); 
    }

    pub fn add_animated_model(&mut self, id: Id, model: &AnimatedModel, tf: Mat4, state: AnimationState)
    { 
        self.added_animated_models.push(id);
        self.animated_models.insert(id, AnimatedModelInstance::new(id, model, tf, state)); 
    }

    pub fn add_camera(&mut self, id: Id, camera: CameraDescriptor)
    { 
        self.cameras.insert(id, Camera::new(id, camera) ); 
        self.added_cameras.push(id);
    }

    pub fn add_spot_light(&mut self, id: Id, spot: SpotLightDescriptor)
    {
        self.added_spot_lights.push(id); 
        self.spot_lights.insert(id, SpotLight::new(id, spot) ); 
    }

    pub fn add_point_light(&mut self, id: Id, point: PointLightDescriptor)
    {
        self.added_point_lights.push(id);
        self.point_lights.insert(id, PointLight::new(id, point) );
    }

    //read

    pub fn get_static_model(&self, key: Id) -> Option<&StaticModelInstance> { return self.static_models.get(&key); }

    pub fn get_animated_model(&self, key: Id) -> Option<&AnimatedModelInstance> { return self.animated_models.get(&key) }

    pub fn get_camera(&self, key: Id) -> Option<&Camera> { return self.cameras.get(&key); }

    pub fn get_point_light(&self, key: Id) -> Option<&PointLight> { return self.point_lights.get(&key); }

    pub fn get_spot_light(&self, key: Id) -> Option<&SpotLight> { return self.spot_lights.get(&key); }

    //update

    pub fn get_static_model_mut(&mut self, key: Id) -> Option<&mut StaticModelInstance> { return self.static_models.get_mut(&key); }

    pub fn get_animated_model_mut(&mut self, key: Id) -> Option<&mut AnimatedModelInstance> { return self.animated_models.get_mut(&key) }

    pub fn get_camera_mut(&mut self, key: Id) -> Option<&mut Camera> { return self.cameras.get_mut(&key); }

    pub fn get_point_light_mut(&mut self, key: Id) -> Option<&mut PointLight> { return self.point_lights.get_mut(&key); }

    pub fn get_spot_light_mut(&mut self, key: Id) -> Option<&mut SpotLight> { return self.spot_lights.get_mut(&key); }

    fn remove_item(&mut self, id: Id)
    {
        if let Some(animated_model) = self.animated_models.remove(&id)
        {
            self.removed_animated_models.push(animated_model.id());
            return;
        }
        if let Some(static_model) = self.static_models.remove(&id)
        {
            self.removed_static_models.push(static_model.id());
            return;
        }
        if let Some(camera) = self.cameras.remove(&id)
        {
            self.removed_cameras.push(id);
            return;
        }
        if let Some(point_light) = self.point_lights.remove(&id)
        {
            self.removed_point_lights.push(id);
            return;
        }
        if let Some(spot_light) = self.spot_lights.remove(&id)
        {
            self.removed_spot_lights.push(id);
            return;
        }
    }

    pub fn clear_dirty_state(&mut self)
    {
        self.added_static_models.clear();
        self.removed_static_models.clear();
        self.added_animated_models.clear();
        self.removed_animated_models.clear();
        self.added_point_lights.clear();
        self.removed_point_lights.clear();
        self.added_spot_lights.clear();
        self.removed_spot_lights.clear();
        for model in self.static_models.values_mut()
        {
            model.clear_dirty_state();
        }
        for model in self.animated_models.values_mut()
        {
            model.clear_dirty_state();
        }
        for light in self.spot_lights.values_mut()
        {
            light.clear_dirty_state();
        }
        for light in self.point_lights.values_mut()
        {
            light.clear_dirty_state();
        }
    }

    pub fn get_added_static_models(&self) -> &Vec<Id>
    {
        return &self.added_static_models;
    }

    pub fn get_removed_static_models(&self) -> &Vec<Id>
    {
        return &self.removed_static_models;
    }

    pub fn get_added_animated_models(&self) -> &Vec<Id>
    {
        return &self.added_animated_models;
    }

    pub fn get_removed_animated_models(&self) -> &Vec<Id>
    {
        return &self.removed_animated_models;
    }

    pub fn get_added_spot_lights(&self) -> &Vec<Id>
    {
        return &self.added_spot_lights;
    }

    pub fn get_added_cameras(&self) -> &Vec<Id>
    {
        return &self.added_cameras;
    }

    pub fn get_removed_spot_lights(&self) -> &Vec<Id>
    {
        return &self.removed_spot_lights;
    }

    pub fn get_added_point_lights(&self) -> &Vec<Id>
    {
        return &self.added_point_lights;
    }

    pub fn get_removed_point_lights(&self) -> &Vec<Id>
    {
        return &self.removed_point_lights;
    }

    pub fn get_removed_cameras(&self) -> &Vec<Id>
    {
        return &self.removed_cameras;
    }

}