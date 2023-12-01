use std::collections::VecDeque;

use glam::{Vec3, Vec4};

#[derive(Clone, Copy)]
pub struct NormalizedFloat
{
    val: f32,
}

impl NormalizedFloat
{
    pub fn clamped(val : f32) -> Self
    {
        return NormalizedFloat{ val : val.clamp(0.0, 1.0) };
    }

    pub fn wrapped(val : f32) -> Self
    {
        return NormalizedFloat{ val : if val == 1.0 { val } else { val % 1.0 } };
    }

    pub fn one() -> Self
    {
        return NormalizedFloat{ val : 1.0 };
    }

    pub fn zero() -> Self
    {
        return NormalizedFloat{ val : 0.0 };
    }

    pub fn get_val(&self) -> f32 
    {
        return self.val;
    }
}

#[derive(Clone, Copy)]
pub struct Color
{
    pub data: Vec4,
}

pub struct Plane
{
    pub loc: Vec3,
    pub dir: Vec3,
}

pub struct AABB
{
    min: Vec3,
    max: Vec3,
}

impl AABB
{
    pub fn new(min: Vec3, max: Vec3) -> Self
    {
        return AABB
        {
            min: min,
            max: max,
        }
    }

    pub fn get_min(&self) -> Vec3
    {
        return self.min;
    }

    pub fn get_max(&self) -> Vec3
    {
        return self.max;
    }

    pub fn to_points(&self) -> [Vec3;8]
    {
        return 
        [
            self.min,
            Vec3::new(self.max.x, self.min.y, self.min.z),
            Vec3::new(self.min.x, self.max.y, self.min.z),
            Vec3::new(self.max.x, self.max.y, self.min.z),
            Vec3::new(self.min.x, self.min.y, self.max.z),
            Vec3::new(self.max.x, self.min.y, self.max.z),
            Vec3::new(self.min.x, self.max.y, self.max.z),
            self.max,
        ];
    }
}

#[derive(Eq, PartialEq, Hash, Clone, Copy, Debug)]
pub struct Id
{
    index: usize,
    generation: usize,
}

impl std::fmt::Display for Id
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result 
    {
        return write!(f, "Id[index: {}, generation: {}]", self.index, self.generation);
    }
}

pub struct IdGenerator
{
    head_index: usize, 
    free_list: VecDeque<Id>
}

impl IdGenerator
{
    pub fn new() -> Self
    {
        return IdGenerator 
        { 
            head_index: 0, 
            free_list: VecDeque::new(), 
        }
    }

    pub fn get_id(&mut self) -> Id
    {
        if self.free_list.is_empty()
        {
            let id = Id {index: self.head_index, generation: 0};
            self.head_index+=1;
            return id;
        }
        else
        {
            let mut id = self.free_list.pop_front().unwrap();
            id.generation += 1;
            return id;
        }
    }

    pub fn free_id(&mut self, id: Id) -> bool
    {
        if !self.free_list.contains(&id)
        {
            self.free_list.push_back(id);
            return true;
        }
        return false;
    }
}