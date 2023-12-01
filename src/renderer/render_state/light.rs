use glam::{Vec3, Vec4};
use super::common::{Color, Id};


//stolen from https://lisyarus.github.io/blog/graphics/2022/07/30/point-light-attenuation.html
#[derive(Clone, Copy)]
pub struct Light
{
    pub color: Color,
    pub intensity: f32,
    pub cutoff_distance: f32,
    //radius of the light itself
    pub radius: f32,
    pub location: Vec3,
    dirty: bool,
}

impl Light
{
    fn clear_dirty_state(&mut self)
    {
        self.dirty = false;
    }

    pub fn set_color_dirty(&mut self, col: Color)
    {
        self.color = col;
        self.dirty = true;
    }

    pub fn set_intensity_dirty(&mut self, intensity: f32)
    {
        self.intensity = intensity;
        self.dirty = true;
    }

    pub fn set_cutoff_distance_dirty(&mut self, cutoff_distance: f32)
    {
        self.cutoff_distance = cutoff_distance;
        self.dirty = true;
    }

    pub fn set_radius_dirty(&mut self, radius: f32)
    {
        self.radius = radius;
        self.dirty = true;
    }

    pub fn set_location_dirty(&mut self, loc: Vec3)
    {
        self.location = loc;
        self.dirty = true;
    }
}

pub struct PointLight
{
    pub id: Id,
    pub light: Light,
}

impl PointLight
{
    pub fn new(id: Id, descriptor: PointLightDescriptor) -> Self
    {
        return PointLight { id: id, light: descriptor.light };
    }

    pub fn is_dirty(&self) -> bool { return self.light.dirty; }

    pub fn clear_dirty_state(&mut self)
    {
        self.light.clear_dirty_state();
    }
}

#[derive(Clone, Copy)]
pub struct PointLightDescriptor
{
    light: Light,
}

impl PointLightDescriptor
{
    pub fn new() -> PointLightDescriptor
    {
        return PointLightDescriptor 
        {
            light: Light 
            {
                color: Color { data: Vec4::ONE }, 
                intensity: 4.0,
                cutoff_distance: 10.0, 
                radius: 2.0,
                location: Vec3::ZERO, 
                dirty: true
            }
        }
    }
}

pub struct SpotLight
{
    pub id: Id,
    pub light: Light,
    pub angle: f32,
    pub dir: Vec3,
    dirty: bool
}

impl SpotLight
{
    pub fn new(id: Id, descriptor: SpotLightDescriptor) -> Self
    {
        return SpotLight
        {
            id: id,
            light: descriptor.light,
            angle: descriptor.angle,
            dir: descriptor.dir,
            dirty: false,
        };
    }

    pub fn is_dirty(&self) -> bool
    {
        return self.dirty || self.light.dirty;
    }

    pub fn set_angle_dirty(&mut self, angle: f32)
    {
        self.angle = angle;
        self.dirty = true;
    }

    pub fn set_dir_dirty(&mut self, dir: Vec3)
    {
        self.dir = dir;
        self.dirty = true;
    }


    pub fn clear_dirty_state(&mut self)
    {
        self.dirty = false;
        self.light.clear_dirty_state();
    }
}

#[derive(Clone, Copy)]
pub struct SpotLightDescriptor
{
    light: Light,
    angle: f32,
    dir: Vec3,
}

#[derive(Clone, Copy)]
pub struct GlobalDirectionalLight
{
    pub light: Light,
    pub dir: Vec3,
}