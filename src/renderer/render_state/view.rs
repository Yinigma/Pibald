use glam::{Vec3, Mat4, Quat};

use super::common::{AABB, Id};

pub struct CameraDescriptor
{
    //extrinsic fields
    pub loc: Vec3,
    pub forward: Vec3,
    pub up: Vec3,
    //intrinsic fields
    pub fov: f32,
    pub aspect: f32,
    pub z_near: f32,
    pub z_far: f32,
}

impl CameraDescriptor
{
    fn new(fov_y_rad: f32, aspect_h_to_w: f32, near_diastance: f32, far_distance: f32, loc: Vec3, forward: Vec3, up: Vec3) -> Self
    {
        return CameraDescriptor
        { 
            loc: loc, 
            forward: forward, 
            up: up, 
            fov: fov_y_rad, 
            aspect: aspect_h_to_w, 
            z_near: near_diastance, 
            z_far: far_distance, 
        };
    }
}

pub struct Camera
{
    //entity crapola
    id: Id,
    //extrinsic fields
    loc: Vec3,
    forward: Vec3,
    up: Vec3,
    //intrinsic fields
    fov: f32,
    aspect: f32,
    z_near: f32,
    z_far: f32,
}

impl Camera
{
    pub fn new(id: Id, descriptor: CameraDescriptor) -> Self
    {
        return Camera 
        { 
            id: id,
            loc: descriptor.loc, 
            forward: descriptor.forward, 
            up: descriptor.up, 
            fov: descriptor.fov, 
            aspect: descriptor.aspect, 
            z_near: descriptor.z_near, 
            z_far: descriptor.z_far, 
        };
    }

    pub fn id(&self) -> Id
    {
        return self.id;
    }

    pub fn forward(&self) -> Vec3
    {
        return self.forward;
    }

    pub fn up(&self) -> Vec3
    {
        return self.up;
    }

    pub fn right(&self) -> Vec3
    {
        return self.up.cross(self.forward);
    }

    pub fn location(&self) -> Vec3
    {
        return self.loc;
    }

    pub fn look_at()
    {

    }

    pub fn translate(&mut self, displacement: Vec3)
    {
        self.loc += displacement;
    }

    //counter to the left-handed coords being used, positive rotation looks up to make it more intuitive
    pub fn pitch(&mut self, angle: f32)
    {
        let sign = self.up.dot(self.forward).signum();
        let safe_angle = (self.forward.angle_between(self.up) - 0.05).min(-angle * sign) * sign;
        self.forward = Quat::from_axis_angle(self.up.cross(self.forward), safe_angle).mul_vec3(self.forward);
    }

    pub fn yaw(&mut self, angle: f32)
    {
        self.forward = Quat::from_axis_angle(self.up, angle).mul_vec3(self.forward);
    }

    pub fn get_perspective_matrix(&self) -> Mat4
    {
        return Mat4::perspective_lh(self.fov, self.aspect, self.z_near, self.z_far);
    }

    pub fn get_view_matrix(&self) -> Mat4
    {
        return Mat4::look_at_lh(self.loc, self.loc + self.forward, self.up);
    }

    pub fn bbox_in_view(&self, bbox: &AABB, bb_world: Mat4) -> bool
    {
        let mvp = self.get_perspective_matrix() * self.get_view_matrix() * bb_world;
        let mut points = bbox.to_points();
        for i in 0..8
        {
            points[i] = mvp.project_point3(points[i]);
        }
        //could try and short-circuit a little more quickly using the aspect ratio
        return 
        !(
            points.iter().all(|p| p.z < 0.0)||
            points.iter().all(|p| p.x > 1.0) ||
            points.iter().all(|p| p.x < -1.0) ||
            points.iter().all(|p| p.y > 1.0) ||
            points.iter().all(|p| p.y < -1.0) ||
            points.iter().all(|p| p.z > 1.0)
        );
    }
}