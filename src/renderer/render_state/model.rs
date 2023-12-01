use std::collections::HashMap;
use std::rc::Rc;
use std::time::Duration;
use glam::{Mat4, Vec3};

use super::animation::{Armature, AnimationState, ArmaturePose};
use super::properties::{Value, AssignmentError};
use super::texture::{Shader, ShaderInstance, ShaderValueLink};
use super::common::{Color, AABB, NormalizedFloat, Id};

pub struct MorphTargetVertex
{
    pub loc: Vec3,
    pub norm: Vec3,
}

pub struct MorphTarget
{
    pub verts: Vec<MorphTargetVertex>,
}

pub struct Model
{
    pub polygons: Vec<Polygon>,
    pub palettes: Vec<ColorPalette>,
    pub default_palette: usize,
    pub shader_slots: HashMap<String, ShaderSlot>,
    pub min_bound: Vec3,
    pub max_bound: Vec3,
}

pub struct ColorPalette
{
    pub colors: Vec<Color>
}

pub struct StaticModel
{
    pub id: String,
    pub vertices: Vec<StaticVertex>,
    pub model_data: Model, 
}

pub struct ShaderSlot
{
    pub tris: Vec<usize>,
    shader: Rc<Shader>,
    pub links: Vec<ShaderValueLink>
}

impl ShaderSlot
{
    pub fn get_shader(&self) -> &Shader
    {
        return self.shader.as_ref();
    }
}

pub struct AnimatedModel
{
    pub id: String,
    pub vertices: Vec<AnimatedVertex>,
    pub model_data: Model,
    pub armature_id: String,
}

#[derive(Clone, Copy)]
pub struct StaticVertex
{
    pub loc: Vec3,
    pub col: usize,
    pub norm: Vec3,
}

pub struct AnimatedVertex
{
    pub vert: StaticVertex,
    pub weights: Vec<ArmatureWeight>
}

#[derive(Clone, Copy)]
pub struct ArmatureWeight
{
    pub weight: NormalizedFloat,
    pub index : usize,
}

pub struct Polygon
{
    pub tris: Vec<Triangle>,
}

pub struct Triangle
{
    pub indices: [usize;3],
}

struct ModelInstance
{
    tf: Mat4,
    colors: Vec<Color>,
    shaders: HashMap<String, ShaderInstance>,
    bbox: AABB,
}

impl ModelInstance
{
    fn new(model : &Model, tf: Mat4) -> Self
    {
        return ModelInstance 
        { 
            tf: tf, 
            shaders: model.shader_slots.iter().map
            (
                |entry| 
                (
                    entry.0.clone(), 
                    ShaderInstance::new(entry.1.shader.clone(), entry.1.links.to_owned()),
                )
            ).collect::<HashMap<String, ShaderInstance>>(),
            colors: model.palettes[model.default_palette].colors.to_owned(),
            bbox: AABB::new(model.min_bound, model.max_bound),
        }
    }
    pub fn set_transform(&mut self, tf: Mat4)
    {
        self.tf = tf;
    }

    pub fn set_color(&mut self, index: usize, color: Color)
    {
        self.colors[index] = color;
    }

    pub fn get_shader_instance(&self, id: &str) -> Option<&ShaderInstance>
    {
        return self.shaders.get(id);
    }

    fn set_shader_property(&mut self, shader_id : &str, property_name : &str, value: Value) -> Result<(), AssignmentError>
    {
        return match self.shaders.get_mut(shader_id)
        {
            Some(shader) => shader.set_property(property_name, value),
            None => Result::Err(AssignmentError::NoSuchPropertyGroupError { container_name: shader_id.to_string() }),
        }
    }

    pub fn set_palette(&mut self, palette: ColorPalette)
    {
        self.colors = palette.colors;
    }

    pub fn get_bounding_box(&self) -> &AABB
    {
        return &self.bbox;
    }

    pub fn get_transform(&self) -> Mat4
    {
        return self.tf;
    }
}

pub struct StaticModelInstance
{
    id: Id,
    model_id: String,
    //static_model: Rc<StaticModel>,
    model_instance: ModelInstance,
    dirty: bool,
}

impl StaticModelInstance
{
    pub fn new(id: Id, model: &StaticModel, tf: Mat4,) -> Self
    {
        let model_inst_data = ModelInstance::new(&model.model_data, tf);
        return StaticModelInstance
        { 
            id: id,
            model_id: model.id.clone(),
            //static_model: model,
            model_instance: model_inst_data,
            dirty: true,
        };
    }

    pub fn id(&self) -> Id { return self.id; }

    pub fn model_id(&self) -> &String { return &self.model_id; }

    /*pub fn model(&self) -> &StaticModel
    {
        return &self.static_model;
    }*/

    pub fn set_transform(&mut self, tf: Mat4)
    {
        self.model_instance.set_transform(tf);
        self.dirty = true;
    }

    pub fn set_shader_property(&mut self, shader_id : &str, property_name : &str, value: Value) -> Result<(), AssignmentError>
    {
        return self.model_instance.set_shader_property(shader_id, property_name, value);
    }

    pub fn set_palette(&mut self, palette: ColorPalette)
    {
        self.model_instance.set_palette(palette);
        self.dirty = true;
    }

    pub fn set_color(&mut self, index: usize, color: Color)
    {
        self.model_instance.set_color(index, color);
        self.dirty = true;
    }

    pub fn clear_dirty_state(&mut self)
    {
        self.dirty = false;
    }

    pub fn dirty(&self) -> bool
    {
        return self.dirty;
    }

    pub fn shader_instance(&self, id: &str) -> Option<&ShaderInstance>
    {
        return self.model_instance.shaders.get(id);
    }

    pub fn bounding_box(&self) -> &AABB
    {
        return &self.model_instance.bbox;
    }

    pub fn transform(&self) -> Mat4
    {
        return self.model_instance.tf;
    }

    pub fn colors(&self) -> &Vec<Color>
    {
        return &self.model_instance.colors;
    }
}

pub struct AnimatedModelInstance
{
    id: Id,
    animated_model_id: String,
    model_instance: ModelInstance,
    anim_state: AnimationState,
    dirty: bool,
}

impl AnimatedModelInstance
{
    pub fn new(id: Id, template: &AnimatedModel, tf: Mat4, state: AnimationState) -> Self
    {
        return AnimatedModelInstance 
        { 
            id: id,
            model_instance: ModelInstance::new(&template.model_data, tf),
            anim_state: state,
            animated_model_id: template.id.clone(),
            dirty: true,
        };
    }

    pub fn id(&self) -> Id { return self.id; }

    pub fn model_id(&self) -> &String {return &self.animated_model_id; }

    pub fn current_pose(&self) -> &ArmaturePose { return &self.anim_state.pose(); }

    pub fn set_transform(&mut self, tf: Mat4)
    {
        self.model_instance.set_transform(tf);
        self.dirty = true;
    }

    pub fn set_shader_property(&mut self, shader_id : &str, property_name : &str, value: Value) -> Result<(), AssignmentError>
    {
        self.dirty = true;
        return self.model_instance.set_shader_property(shader_id, property_name, value);
    }

    pub fn set_color(&mut self, index: usize, color: Color)
    {
        self.model_instance.set_color(index, color);
        self.dirty = true;
    }

    pub fn set_palette(&mut self, palette: ColorPalette)
    {
        self.model_instance.set_palette(palette);
        self.dirty = true;
    }

    pub fn clear_dirty_state(&mut self)
    {
        self.dirty = false;
    }

    pub fn dirty(&self) -> bool
    {
        return self.dirty;
    }

    pub fn update(&mut self, dt: f32)
    {
        self.anim_state.update(dt);
        self.dirty = true;
    }

    pub fn bounding_box(&self) -> &AABB
    {
        return self.model_instance.get_bounding_box();
    }

    pub fn transform(&self) -> Mat4
    {
        return self.model_instance.get_transform();
    }

    pub fn colors(&self) -> &Vec<Color>
    {
        return &self.model_instance.colors;
    }

    pub fn anim_state(&self) -> &AnimationState
    {
        return &self.anim_state;
    }

    pub fn anim_state_mut(&mut self) -> &mut AnimationState
    {
        return &mut self.anim_state;
    }
}

