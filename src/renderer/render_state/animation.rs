use std::{rc::Rc, collections::HashMap, f32::consts::PI};

use glam::{Vec3, Quat, Mat4};
use super::common::NormalizedFloat;

#[derive(Clone)]
//All data needed to draw an armature for a frame
pub struct ArmaturePose
{
    joints : Vec<PoseTransform>,
    control_values: Vec<f32>
}

impl ArmaturePose
{
    pub fn transforms(&self, armature: &Armature, dest: &mut [Mat4], armature_buffer: &mut [Mat4])
    {
        let mut pose_buffer = vec![Mat4::IDENTITY; armature_buffer.len()];
        if armature.bones.len() == self.joints.len() && self.joints.len() <= dest.len()
        {
            for i in 0..self.joints.len()
            {
                let parent_pose_tf = match armature.bones[i].parent
                {
                    Some(p_dex) => pose_buffer[p_dex],
                    None => Mat4::IDENTITY,
                };
                let parent_armature_tf = match armature.bones[i].parent
                {
                    Some(p_dex) => armature_buffer[p_dex],
                    None => Mat4::IDENTITY,
                };
                armature_buffer[i] = parent_armature_tf * armature.bones[i].local_tf;
                pose_buffer[i] =  parent_pose_tf * armature.bones[i].local_tf * self.joints[i].to_matrix();
                dest[i] = pose_buffer[i] * armature_buffer[i].inverse();
                
            }
        }
    }

    pub fn control_values(&self) -> &Vec<f32>
    {
        return &self.control_values;
    }

    fn add_clip(&mut self, other: &AnimationClip, other_t: NormalizedFloat, other_playback: PlaybackType, mask: &Option<ArmatureMask>, clip_weight : NormalizedFloat)
    {
        let multiplier = match mask 
        {
            Some(m) => m.multiplier.get_val(),
            None => 1.0,
        };

        if multiplier > 0.0
        {
            for i in 0..self.joints.len()
            {
                let joint_weight = match mask  
                {
                    Some(m) => m.bone_weights[i].get_val(), 
                    None => 1.0 
                } * clip_weight.get_val();
                if joint_weight > 0.0
                {
                    let loc_track = other.sample_location_track(i, other_t, other_playback);
                    if let Some(x) = loc_track[0] { self.joints[i].location.x = self.joints[i].location.x + x * joint_weight; }
                    if let Some(y) = loc_track[1] { self.joints[i].location.y = self.joints[i].location.y + y * joint_weight; }
                    if let Some(z) = loc_track[2] { self.joints[i].location.z = self.joints[i].location.z + z * joint_weight; }

                    if let Some(other_rot) = other.sample_orientation_track(i, other_t, other_playback)
                    {
                        self.joints[i].orientation = self.joints[i].orientation * Quat::IDENTITY.slerp(other_rot, joint_weight);
                    }

                    let scale_track = other.sample_scale_track(i, other_t, other_playback);
                    if let Some(x) = scale_track[0] { self.joints[i].scale.x = self.joints[i].scale.x * f32::interpolate(&1.0, &x, joint_weight); }
                    if let Some(y) = scale_track[1] { self.joints[i].scale.y = self.joints[i].scale.y * f32::interpolate(&1.0, &y, joint_weight); }
                    if let Some(z) = scale_track[2] { self.joints[i].scale.z = self.joints[i].scale.z * f32::interpolate(&1.0, &z, joint_weight); }

                }
            }
            for i in 0..self.control_values.len()
            {
                let control_weight = match mask 
                { 
                    Some(m) => m.control_weights[i].get_val(), 
                    None => 1.0 
                } * clip_weight.get_val();
                if control_weight > 0.0
                {
                    if let Some(other_control) = other.sample_control_track(i, other_t, other_playback)
                    {
                        self.control_values[i] = self.control_values[i] + other_control * control_weight;
                    }
                }
            }
        }
    }

    fn mix_clip(&mut self, other: &AnimationClip, other_t: NormalizedFloat, other_playback: PlaybackType, mask: &Option<ArmatureMask>, clip_weight: NormalizedFloat)
    {
        let multiplier = match mask
        {
            Some(m) => m.multiplier.get_val(),
            None => 1.0,
        };

        if multiplier > 0.0
        {
            for i in 0..self.joints.len()
            {
                let joint_weight = match mask  
                { 
                    Some(m) => m.bone_weights[i].get_val(), 
                    None => 1.0 
                } * clip_weight.get_val();
                if joint_weight > 0.0
                {
                    let loc_track = other.sample_location_track(i, other_t, other_playback);
                    if let Some(x) = loc_track[0] { self.joints[i].location.x = self.joints[i].location.x.interpolate(&x, joint_weight) } else { self.joints[i].location.x; }
                    if let Some(y) = loc_track[1] { self.joints[i].location.y = self.joints[i].location.y.interpolate(&y, joint_weight) } else { self.joints[i].location.y; } 
                    if let Some(z) = loc_track[2] { self.joints[i].location.z = self.joints[i].location.z.interpolate(&z, joint_weight) } else { self.joints[i].location.z; }
                    
                    if let Some(other_rot) = other.sample_orientation_track(i, other_t, other_playback)
                    {
                        self.joints[i].orientation = self.joints[i].orientation.lerp(other_rot, joint_weight);
                    }

                    let scale_track = other.sample_scale_track(i, other_t, other_playback);
                    if let Some(x) = scale_track[0] { self.joints[i].scale.x = self.joints[i].scale.x.interpolate(&x, joint_weight) } else { self.joints[i].scale.x; }
                    if let Some(y) = scale_track[1] { self.joints[i].scale.y = self.joints[i].scale.y.interpolate(&y, joint_weight) } else { self.joints[i].scale.y; } 
                    if let Some(z) = scale_track[2] { self.joints[i].scale.z = self.joints[i].scale.z.interpolate(&z, joint_weight) } else { self.joints[i].scale.z; }                        
                }
            }
            for i in 0..self.control_values.len()
            {
                let control = match mask 
                { 
                    Some(m) => m.control_weights[i].get_val(), 
                    None => 1.0 
                } * clip_weight.get_val();
                if control > 0.0
                {
                    if let Some(other_control) = other.sample_control_track(i, other_t, other_playback)
                    {
                        self.control_values[i] = self.control_values[i].interpolate(&other_control, control);
                    }
                }
            }
        }
    }

    fn clear(&mut self)
    {
        for joint in &mut self.joints
        {
            joint.location = Vec3::ZERO;
            joint.orientation = Quat::IDENTITY;
            joint.scale = Vec3::ONE;
        }
    }
}

struct AnimationTransition
{
    mixer_id : String,
    duration : f32,
    elapsed : f32
}

impl AnimationTransition
{
    fn new(mixer_id: &str, duration: std::time::Duration) -> Self
    {
        return AnimationTransition { mixer_id: mixer_id.to_string(), duration: duration.as_secs_f32(), elapsed: 0.0 };
    }
}

struct MixerEvent
{
    trigger_time: NormalizedFloat,
    callbacks: Vec<Box<dyn Fn()>>,
}

//These should probably be considered value objects
//Make a template form of this object if needed
pub struct ClipMixer
{
    id: String,
    playback_rate: f32,
    playback_type: PlaybackType,
    mixer_variant: ClipMixerVariant,
    events: HashMap<String, MixerEvent>
}

impl ClipMixer
{
    fn mix_to_pose(&self, mix: MixType, time : NormalizedFloat, mask: &Option<ArmatureMask>, clip_weight: NormalizedFloat, destination: &mut ArmaturePose, )
    {
        match mix 
        {
            MixType::Additive => self.mixer_variant.mix_to_pose(time, self.playback_type, mask, clip_weight, destination),
            MixType::Override => self.mixer_variant.add_to_pose(time, self.playback_type, mask, clip_weight, destination),
        }
    }

    fn duration_seconds(&self) -> f32
    {
        return self.mixer_variant.duration_seconds();
    }

    fn duration_frames(&self) -> f32
    {
        return self.mixer_variant.duration_frames();
    }
}

pub enum ClipMixerVariant
{
    Single(SingleClipMixer),
    LinearBlended(LinearBlendMixer),
}

impl ClipMixerVariant
{
    fn mix_to_pose(&self, time : NormalizedFloat, playback_type: PlaybackType, mask: &Option<ArmatureMask>, clip_weight: NormalizedFloat, destination: &mut ArmaturePose, )
    {
        match self 
        {
            ClipMixerVariant::Single(single_clip) => single_clip.mix_to_pose(time, playback_type, mask, clip_weight, destination),
            ClipMixerVariant::LinearBlended(blend_clip) => blend_clip.mix_to_pose(time, playback_type, mask, clip_weight, destination),
        };
    }

    fn add_to_pose(&self, time : NormalizedFloat, playback_type: PlaybackType, mask: &Option<ArmatureMask>, clip_weight: NormalizedFloat, destination: &mut ArmaturePose, )
    {
        match self 
        {
            ClipMixerVariant::Single(single_clip) => single_clip.add_to_pose(time, playback_type, mask, clip_weight, destination),
            ClipMixerVariant::LinearBlended(blend_clip) => blend_clip.add_to_pose(time, playback_type, mask, clip_weight, destination),
        };
    }

    fn duration_seconds(&self) -> f32
    {
        return match self 
        {
            ClipMixerVariant::Single(single_clip) => single_clip.clip.duration_seconds(),
            ClipMixerVariant::LinearBlended(blend_clip) => blend_clip.duration_seconds(),
        };
    }

    fn duration_frames(&self) -> f32
    {
        return match self 
        {
            ClipMixerVariant::Single(single_clip) => single_clip.clip.duration_frames() as f32,
            ClipMixerVariant::LinearBlended(blend_clip) => blend_clip.duration_frames(),
        };
    }
}

pub struct AnimationClip
{
    start_frame : u16,
    end_frame : u16,
    fps : u16, //only crazy people will use all of these bits
    x_location_tracks : HashMap<usize, ClipChannel<f32>>,
    y_location_tracks : HashMap<usize, ClipChannel<f32>>,
    z_location_tracks : HashMap<usize, ClipChannel<f32>>,
    orientation_tracks : HashMap<usize, [ClipChannel<f32>; 4]>,
    x_scale_tracks : HashMap<usize, ClipChannel<f32>>,
    y_scale_tracks : HashMap<usize, ClipChannel<f32>>,
    z_scale_tracks : HashMap<usize, ClipChannel<f32>>,
    control_tracks : Vec<ClipChannel<f32>>,
}

impl AnimationClip
{
    pub fn new
    (
        start: u16, 
        end: u16, 
        fps: u16, 
        x_location_tracks : HashMap<usize, ClipChannel<f32>>,
        y_location_tracks : HashMap<usize, ClipChannel<f32>>,
        z_location_tracks : HashMap<usize, ClipChannel<f32>>,
        orientation_tracks : HashMap<usize, [ClipChannel<f32>; 4]>,
        x_scale_tracks : HashMap<usize, ClipChannel<f32>>,
        y_scale_tracks : HashMap<usize, ClipChannel<f32>>,
        z_scale_tracks : HashMap<usize, ClipChannel<f32>>,
        control_tracks : Vec<ClipChannel<f32>>,
    ) -> Self
    {
        return AnimationClip
        { 
            start_frame: start, 
            end_frame: end, 
            fps, 
            x_location_tracks, 
            y_location_tracks, 
            z_location_tracks, 
            orientation_tracks,
            x_scale_tracks, 
            y_scale_tracks, 
            z_scale_tracks, 
            control_tracks 
        }
    }

    fn sample_vec_tracks_for_clip(tf_tracks: &HashMap<usize, ClipChannel<f32>>, bone_dex: usize, start_frame: u16, end_frame: u16, time: NormalizedFloat, playback_type: PlaybackType) -> Option<f32>
    {
        if let Option::Some(track) = tf_tracks.get(&bone_dex)
        {
            return Option::Some(track.sample(start_frame, end_frame, time, playback_type));
        }
        else 
        { 
            return Option::None; 
        }
    }

    fn sample_scale_track(&self, index: usize, time : NormalizedFloat, playback_type: PlaybackType,) -> [Option<f32>; 3]
    {
        return
        [
            AnimationClip::sample_vec_tracks_for_clip(&self.x_scale_tracks, index, self.start_frame, self.end_frame, time, playback_type),
            AnimationClip::sample_vec_tracks_for_clip(&self.y_scale_tracks, index, self.start_frame, self.end_frame, time, playback_type),
            AnimationClip::sample_vec_tracks_for_clip(&self.z_scale_tracks, index, self.start_frame, self.end_frame, time, playback_type),
        ];
    }

    fn sample_location_track(&self, index: usize, time : NormalizedFloat, playback_type: PlaybackType,) -> [Option<f32>; 3]
    {
        if index == 79
        {
            print!("");
        }
        return
        [
            AnimationClip::sample_vec_tracks_for_clip(&self.x_location_tracks, index, self.start_frame, self.end_frame, time, playback_type),
            AnimationClip::sample_vec_tracks_for_clip(&self.y_location_tracks, index, self.start_frame, self.end_frame, time, playback_type),
            AnimationClip::sample_vec_tracks_for_clip(&self.z_location_tracks, index, self.start_frame, self.end_frame, time, playback_type),
        ];
    }

    fn sample_orientation_track(&self, index: usize, time : NormalizedFloat, playback_type: PlaybackType,) -> Option<Quat>
    {
        if let Option::Some(quat_curves) = self.orientation_tracks.get(&index)
        {
            if index == 79
            {
                print!("");
            }
            return Option::Some
            (
                Quat::from_xyzw
                (
                    quat_curves[0].sample(self.start_frame, self.end_frame, time, playback_type), 
                    quat_curves[1].sample(self.start_frame, self.end_frame, time, playback_type), 
                    quat_curves[2].sample(self.start_frame, self.end_frame, time, playback_type), 
                    quat_curves[3].sample(self.start_frame, self.end_frame, time, playback_type),
                )
            );
        }
        return Option::None;
    }

    fn sample_control_track(&self, index: usize, time: NormalizedFloat, playback_type: PlaybackType,) -> Option<f32>
    {
        if let Option::Some(control_channel) = self.control_tracks.get(index)
        {
            return Option::Some( control_channel.sample(self.start_frame, self.end_frame, time, playback_type) );
        }
        return Option::None;
    }

    fn duration_seconds(&self) -> f32
    {
        return ((self.end_frame - self.start_frame) as f32) / (self.fps as f32);
    }

    fn duration_frames(&self) -> u16
    {
        return self.end_frame - self.start_frame;
    }
}

pub struct SingleClipMixer
{
    clip: Rc<AnimationClip>
}

impl SingleClipMixer
{
    fn mix_to_pose(&self, time : NormalizedFloat, playback_type: PlaybackType, mask: &Option<ArmatureMask>, clip_weight: NormalizedFloat, destination: &mut ArmaturePose, )
    {
        destination.mix_clip(self.clip.as_ref(), time, playback_type, mask, clip_weight);
    }

    fn add_to_pose(&self, time : NormalizedFloat, playback_type: PlaybackType, mask: &Option<ArmatureMask>, clip_weight: NormalizedFloat, destination: &mut ArmaturePose, )
    {
        destination.add_clip(self.clip.as_ref(), time, playback_type, mask, clip_weight);
    }

    pub fn new(clip: Rc<AnimationClip>) -> Self
    {
       return SingleClipMixer{clip: clip};
    }
}

pub enum MixerBuilder
{
    Simple(SimpleMixerBuilder),
    LinearBlend(BlendMixerBuilder),
}

impl MixerBuilder
{
    fn get_id(&self) -> &String
    {
        return match self
        {
            MixerBuilder::Simple(simple) => &simple.id,
            MixerBuilder::LinearBlend(linear) => &linear.id,
        };
    }

    fn build(&self) -> ClipMixer
    {
        return match self 
        {
            MixerBuilder::Simple(simple) => simple.build(),
            MixerBuilder::LinearBlend(builder) => builder.build(),
        }
    }
}

pub struct SimpleMixerBuilder
{
    id: String,
    playback_rate: f32,
    playback_type: PlaybackType,
    clip: Rc<AnimationClip>,
    events: HashMap<String, NormalizedFloat>
}

impl SimpleMixerBuilder
{
    pub fn new(id: &str, clip_name: Rc<AnimationClip>) -> Self
    {
        return SimpleMixerBuilder { id: id.to_string(), playback_rate: 1.0, playback_type: PlaybackType::Sequential, clip: clip_name, events: HashMap::new() };
    }

    pub fn playback_rate(mut self, speed: f32) -> Self
    {
        self.playback_rate = speed;
        return self;
    }

    pub fn playback_type(mut self, playback_type: PlaybackType) -> Self
    {
        self.playback_type = playback_type;
        return self;
    }

    pub fn add_event( mut self, name: &str, time: NormalizedFloat, ) -> Self
    {
        self.events.insert(name.to_string(), time);
        return self;
    }

    pub fn build(&self) -> ClipMixer
    {
        return ClipMixer 
        {
            id: self.id.clone(),
            playback_rate: self.playback_rate, 
            playback_type: self.playback_type, 
            mixer_variant: ClipMixerVariant::Single(SingleClipMixer::new(self.clip.clone())),
            events: self.events.iter().map(|item| (item.0.clone(), MixerEvent{trigger_time: *item.1, callbacks: vec![]})).collect(),
        }
    }
}

#[derive(Clone)]
pub struct LinearBlendPoint
{
    clip: Rc<AnimationClip>,
    pos: NormalizedFloat,
}

impl LinearBlendPoint
{
    pub fn new(clip : Rc<AnimationClip>, pos : NormalizedFloat) -> Self
    {
        return LinearBlendPoint { clip: clip, pos: pos };
    }
}

pub struct LinearBlendMixer
{
    points : Vec<LinearBlendPoint>,
    blend : NormalizedFloat,
}

impl LinearBlendMixer
{
    pub fn new(clips: Vec<LinearBlendPoint>) -> Self
    {
        return LinearBlendMixer { points: clips, blend: NormalizedFloat::zero() };
    }
}

impl LinearBlendMixer
{
    fn get_start_point(&self) -> &LinearBlendPoint
    {
        return match self.points.iter().rev().find
        (
            |clip| clip.pos.get_val() <= self.blend.get_val()
        )
        {
            Some(clip) => clip,
            None =>  self.points.last().unwrap(),
        };
    }

    fn get_end_point(&self) -> &LinearBlendPoint
    {
        return match self.points.iter().find
        (
            |clip| clip.pos.get_val() <= self.blend.get_val() 
        )
        {
            Some(clip) => clip,
            None =>  self.points.first().unwrap(),
        };
    }

    fn mix_to_pose(&self, time : NormalizedFloat, playback_type: PlaybackType, mask: &Option<ArmatureMask>, clip_weight: NormalizedFloat, destination: &mut ArmaturePose, )
    {
        let start_point = self.get_start_point();
        let end_point = self.get_end_point();
        let blend_weight = self.blend.get_val() - start_point.pos.get_val() / (end_point.pos.get_val() - start_point.pos.get_val());
        destination.mix_clip(start_point.clip.as_ref(), time, playback_type, mask, NormalizedFloat::clamped(clip_weight.get_val() * ( 1.0 - blend_weight ) ) );
        destination.mix_clip(end_point.clip.as_ref(), time, playback_type, mask, NormalizedFloat::clamped(clip_weight.get_val() * blend_weight ) );
    }

    fn add_to_pose(&self, time : NormalizedFloat, playback_type: PlaybackType, mask: &Option<ArmatureMask>, clip_weight: NormalizedFloat, destination: &mut ArmaturePose, )
    {
        let start_point = self.get_start_point();
        let end_point = self.get_end_point();
        let blend_weight = self.blend.get_val() - start_point.pos.get_val() / (end_point.pos.get_val() - start_point.pos.get_val());
        destination.add_clip(start_point.clip.as_ref(), time, playback_type, mask, NormalizedFloat::clamped(clip_weight.get_val() * ( 1.0 - blend_weight ) ) );
        destination.add_clip(end_point.clip.as_ref(), time, playback_type, mask, NormalizedFloat::clamped(clip_weight.get_val() * blend_weight ) );
    }

    fn duration_seconds(&self) -> f32
    {
        let start_point = self.get_start_point();
        //let end_point = self.get_end_point();
        return start_point.clip.duration_seconds().interpolate(&start_point.clip.duration_seconds(), self.blend.get_val() - start_point.pos.get_val());
    }

    fn duration_frames(&self) -> f32
    {
        let start_point = self.get_start_point();
        //let end_point = self.get_end_point();
        return (start_point.clip.duration_frames() as f32).interpolate(&(start_point.clip.duration_frames() as f32), self.blend.get_val() - start_point.pos.get_val());
    }

}

pub struct BlendMixerBuilder
{
    id: String,
    playback_rate: f32,
    playback_type: PlaybackType,
    points: Vec<LinearBlendPoint>,
    init_blend: NormalizedFloat,
    events: HashMap<String, NormalizedFloat>,
}

impl BlendMixerBuilder
{
    pub fn create(id: &str, init_clip: Rc<AnimationClip>, init_blend: NormalizedFloat,) -> Self
    {
        return BlendMixerBuilder 
        {
            id: id.to_string(),
            playback_rate: 1.0, 
            playback_type: PlaybackType::Sequential, 
            points: vec![LinearBlendPoint{clip: init_clip, pos: init_blend}], 
            init_blend: NormalizedFloat::zero(),
            events: HashMap::new(),
        };
    }

    pub fn playback_rate(mut self, speed: f32) -> Self
    {
        self.playback_rate = speed;
        return self;
    }

    pub fn add_blend_point(mut self, clip: Rc<AnimationClip>, blend_point: NormalizedFloat) -> Self
    {
        for i in 0..self.points.len()
        {
            if blend_point.get_val() > self.points[i].pos.get_val()
            {
                self.points.insert(i, LinearBlendPoint { clip: clip, pos: blend_point });
                return self;
            }
            else if blend_point.get_val() == self.points[i].pos.get_val()
            {
                self.points[i] = LinearBlendPoint { clip: clip, pos: blend_point };
                return self;
            }
        }
        self.points.push(LinearBlendPoint { clip: clip, pos: blend_point });
        return self;
    }

    pub fn blend(mut self, blend: NormalizedFloat) -> Self
    {
        self.init_blend = blend;
        return self;
    }

    pub fn playback_type(mut self, playback_type: PlaybackType) -> Self
    {
        self.playback_type = playback_type;
        return self;
    }

    pub fn add_event( mut self, name: &str, time: NormalizedFloat, ) -> Self
    {
        self.events.insert(name.to_string(), time);
        return self;
    }

    pub fn build(&self) -> ClipMixer
    {
        return ClipMixer
        {
            id: self.id.clone(),
            playback_rate: self.playback_rate, 
            playback_type: self.playback_type, 
            mixer_variant: ClipMixerVariant::LinearBlended
            (
                LinearBlendMixer
                {
                    points: self.points.clone(),
                    blend: self.init_blend,
                }
            ),
            events: self.events.iter().map(|item| (item.0.clone(), MixerEvent{trigger_time: *item.1, callbacks: vec![]})).collect(),
        }
    }
}

pub struct ClipChannel<T : Interpolate>
{
    data : Vec<AnimationKey<T>>,
}

#[derive(Clone, Copy)]
pub struct AnimationKey<T : Interpolate>
{
    pub data : T,
    pub frame : u16,
}

pub trait Interpolate : Copy
{
    fn interpolate(&self, rhs: &Self, t: f32) -> Self;
}

impl Interpolate for Quat
{
    fn interpolate(&self, rhs: &Self, t: f32) -> Self 
    {
        return self.slerp(rhs.clone(), t);
    }
}

/*impl Interpolate for Vec3
{
    fn interpolate(&self, rhs: &Self, t: f32) -> Self 
    {
        return self.lerp(rhs.clone(), t);
    }
}*/

impl Interpolate for f32
{
    fn interpolate(&self, rhs: &Self, t: f32) -> Self 
    {
        return self + (rhs - self) * t;
    }
}

impl<T:Interpolate> ClipChannel<T>
{
    pub fn new(data : Vec<AnimationKey<T>>) -> ClipChannel<T>
    {
        if data.is_empty()
        {
            panic!("Attempted to create a clip channel with no data. If this channel is not supposed to have keyframe data, then it should instead be excluded from whatever container you're trying to put it in.");
        }
        return ClipChannel { data: data };
    }

    fn sample(&self, min_frame : u16, max_frame: u16, t: NormalizedFloat, extrapolation_mode : PlaybackType) -> T
    {
        let f_range = (max_frame - min_frame) as f32;
        let sample_frame = min_frame as f32 + t.get_val() * f_range;
        let start_key = match self.data.iter().rev().find
        (
            |key| (key.frame as f32) <= sample_frame 
        )
        {
            Some(key) => key,
            None => match extrapolation_mode
            {
                PlaybackType::Sequential => self.data.first().unwrap(),
                PlaybackType::Looping => self.data.last().unwrap(),
            },
        };
        let f_start = start_key.frame as f32;
        let end_key = match self.data.iter().find
        (
            |key| (key.frame as f32) >= sample_frame
        )
        {
            Some(key) => key,
            None => match extrapolation_mode
            {
                PlaybackType::Sequential => self.data.last().unwrap(),
                PlaybackType::Looping => self.data.first().unwrap(),
            },
        };
        let f_end = end_key.frame as f32;
        if end_key.frame < start_key.frame
        {
            let loop_sample = if sample_frame < f_start { sample_frame + f_range } else { sample_frame };
            let phantom_end = f_end + f_range;
            return start_key.data.interpolate(&end_key.data, (loop_sample - f_start)/(phantom_end - f_start));
        }
        else if start_key.frame == end_key.frame
        {
            return start_key.data;
        }
        return start_key.data.interpolate(&end_key.data, (sample_frame - f_start)/(f_end - f_start) );

    }
}

#[derive(Clone, Copy)]
pub enum PlaybackType
{
    Sequential,
    Looping,
}

#[derive(Clone, Copy)]
pub struct PoseTransform
{
    location: Vec3,
    orientation: Quat,
    scale: Vec3
}

impl PoseTransform
{
    fn empty() -> Self
    {
        return PoseTransform { location: Vec3::ZERO, orientation: Quat::IDENTITY, scale: Vec3::ONE };
    }

    fn to_matrix(&self) -> Mat4
    {
        return Mat4::from_scale_rotation_translation(self.scale, self.orientation, self.location);
    }

    pub fn loc_rot_scale(&self) -> (Vec3, Quat, Vec3)
    {
        return (self.location, self.orientation, self.scale);
    }
}

#[derive(Clone, Copy)]
pub enum MixType
{
    Additive,
    Override,
}

#[derive(Clone)]
struct ArmatureMask
{
    multiplier : NormalizedFloat,
    bone_weights : Vec<NormalizedFloat>,
    control_weights : Vec<NormalizedFloat>
}

struct LayerBuilder
{
    name: String,
    starting_mixer: String,
    weight: NormalizedFloat,
    mix: MixType,
    mask: Option<ArmatureMask>,
    mixers: Vec<MixerBuilder>,
}

impl LayerBuilder
{
    fn create_builder(id: &str) -> Self
    {
        return LayerBuilder
        {
            name: id.to_string(),
            mix: MixType::Override,
            weight: NormalizedFloat::one(),
            mask: None,
            starting_mixer: "".to_string(),
            mixers: vec![],
        };
    }

    fn init_weight(&mut self, weight: NormalizedFloat) -> &mut Self
    {
        self.weight = weight;
        return self;
    }

    fn mix(&mut self, mix: MixType) -> &mut Self
    {
        self.mix = mix;
        return self;
    }

    fn mask(&mut self, mask: ArmatureMask) -> &mut Self
    {
        self.mask = Option::Some(mask);
        return self;
    }

    fn add_mixer(&mut self, builder: MixerBuilder) -> &mut Self
    {
        self.mixers.push( builder );
        return self;
    }

    fn starting_mixer(&mut self, id: &str) -> &mut Self
    {
        self.starting_mixer = id.to_string();
        return self;
    }

    fn build(&self) -> ArmatureLayer
    {
        return ArmatureLayer
        {
            name: self.name.clone(),
            mix: self.mix,
            mask: self.mask.clone(),
            weight: self.weight,
            transition_queue: vec![],
            working_mixer_opt: if self.mixers.iter().any(|mix| mix.get_id() == &self.starting_mixer) { Option::Some(self.starting_mixer.clone()) } else { Option::None },
            portion_into_working_mixer: NormalizedFloat::zero(),
            mixers: self.mixers.iter().map(|mix_builder| (mix_builder.get_id().clone(), mix_builder.build())).collect::<HashMap<_,_>>(),
        }
    }
}

pub struct ArmatureLayer
{
    name: String,
    mix : MixType,
    mask : Option<ArmatureMask>,
    weight : NormalizedFloat,
    transition_queue : Vec<AnimationTransition>,
    working_mixer_opt : Option<String>,
    portion_into_working_mixer : NormalizedFloat,
    //todo: change this to a hashmap of strings and mixers - it'll make it a lot more human-readable since these objects will be loaded as assets
    mixers: HashMap<String, ClipMixer>,
}

impl ArmatureLayer
{
    const BASE_LAYER_ID : &str = "base";

    /*fn create_base_layer(clips: Vec<ClipMixer>) -> Self
    {
        return ArmatureLayer 
        {
            name: Self::BASE_LAYER_ID.to_string(), 
            mix: MixType::Override, 
            mask: None,
            weight: NormalizedFloat::one(), 
            transition_queue: vec![], 
            working_mixer_opt: None, 
            portion_into_working_mixer: NormalizedFloat::zero(),
            mixers: clips,
        }
    }*/

    /*pub fn new(name: &str, mix_type: MixType, mask: Option<ArmatureMask>, weight: NormalizedFloat, clips: Vec<ClipMixer>) -> Self
    {
        return ArmatureLayer 
        {
            name: name.to_string(), 
            mix: mix_type, 
            mask: mask, 
            weight: weight, 
            transition_queue: vec![], 
            working_mixer_opt: None, 
            portion_into_working_mixer: NormalizedFloat::zero(),
            mixers: clips,
        }
    }*/

    fn update(&mut self, dt : f32)
    {
        if let Option::Some(working_mixer) = &self.working_mixer_opt 
        {
            let init_clip = &self.mixers.get_mut(working_mixer).unwrap();
            let step = (init_clip.playback_rate * dt)/init_clip.duration_seconds();
            self.portion_into_working_mixer = match init_clip.playback_type 
            {
                PlaybackType::Sequential => NormalizedFloat::clamped(step + self.portion_into_working_mixer.get_val()),
                PlaybackType::Looping => NormalizedFloat::wrapped(step + self.portion_into_working_mixer.get_val()),
            };
            if self.transition_queue.len() > 0
            {
                let mut i = 0;
                while i < self.transition_queue.len()
                {
                    let transition = &mut self.transition_queue[i];
                    let clip = &self.mixers[&transition.mixer_id];
                    let prev_elapsed = transition.elapsed;
                    transition.elapsed += dt;
                    for event in clip.events.values()
                    {
                        if event.trigger_time.get_val() > prev_elapsed/clip.duration_seconds() && event.trigger_time.get_val() <= transition.elapsed/clip.duration_seconds()
                        {
                            for callback in &event.callbacks
                            {
                                callback();
                            }
                        }
                    }
                    //the transition's duration has been exceeded by the time it's been alive
                    if transition.duration <= transition.elapsed
                    {
                        //first figure out how far we are into the transition's clip
                        let portion = transition.elapsed/clip.duration_seconds();
                        //remove all transitions before this one. Even if they aren't finished, this one is, so it overlaps those before it
                        for _j in 0..i
                        {
                            self.transition_queue.remove(0);
                        }
                        let next_clip_id = self.transition_queue.remove(0).mixer_id;
                        self.working_mixer_opt = Some(next_clip_id.clone());
                        let working_clip = &self.mixers.get_mut(&next_clip_id).unwrap();
                        self.portion_into_working_mixer = match working_clip.playback_type 
                        {
                            PlaybackType::Sequential => NormalizedFloat::clamped(portion),
                            PlaybackType::Looping => NormalizedFloat::wrapped(portion),
                        };
                        i = 0;
                    }
                    i += 1;
                }
            }
        }
    }

    fn apply_to_pose(&self, destination: &mut ArmaturePose, current_weight: NormalizedFloat)
    {
        if let Option::Some(working_mixer) = &self.working_mixer_opt
        {
            let working_clip = &self.mixers[working_mixer];
            let mut m_weights: Vec<f32> = Vec::with_capacity(self.transition_queue.len());
            let mut base_weight = 0.0;
            self.transition_queue.iter().rev().for_each
            (
                |active_transition|
                {
                    m_weights.push( (1.0 - m_weights.last().unwrap_or(&0.0)) * (active_transition.elapsed)/(active_transition.duration) );
                    base_weight += m_weights.last().unwrap();
                }
            );
            base_weight = 1.0 - base_weight;
            working_clip.mix_to_pose(self.mix, self.portion_into_working_mixer, &self.mask, NormalizedFloat::clamped(base_weight*current_weight.get_val()), destination);
            for i in 0..self.transition_queue.len()
            {
                let active_transition = &self.transition_queue[i];
                let clip_portion = match self.mixers[&active_transition.mixer_id].playback_type 
                {
                    PlaybackType::Sequential => NormalizedFloat::clamped((active_transition.elapsed)/self.mixers[&active_transition.mixer_id].duration_seconds()),
                    PlaybackType::Looping => NormalizedFloat::wrapped((active_transition.elapsed)/self.mixers[&active_transition.mixer_id].duration_seconds()),
                };
                self.mixers[&active_transition.mixer_id].mix_to_pose(self.mix, clip_portion, &self.mask, NormalizedFloat::clamped(m_weights.pop().unwrap() * current_weight.get_val()), destination);
            }
        }
    }

    pub fn queue_clip_mixer(&mut self, mixer_id: &str, duration: std::time::Duration)
    {
        if self.mixers.contains_key(mixer_id)
        {
            self.transition_queue.push(AnimationTransition { mixer_id: mixer_id.to_string(), duration: duration.as_secs_f32(), elapsed: 0.0 });
        }
    }
}

pub struct Bone
{
    pub parent: Option<usize>,
    //todo make private again
    pub local_tf: Mat4,
}

impl Bone
{
    pub fn new(parent: Option<usize>, loc: Vec3, rot: Quat) -> Bone
    {
        return Bone
        {
            parent: parent,
            local_tf: Mat4::from_rotation_translation(rot, loc),
        }
    }
}

pub struct Armature
{
    id: String,
    //todo - make this private again
    pub bones: Vec<Bone>,
    num_controls: usize,
}

impl Armature
{
    pub fn new(id: String, bones: Vec<Bone>) -> Armature
    {  
        return Armature 
        {
            id: id,
            bones: bones, 
            num_controls: 0 
        };
    }

    fn create_empty_pose(&self) -> ArmaturePose
    {
        return ArmaturePose{joints: vec![PoseTransform::empty(); self.bones.len()], control_values: vec![0.0; self.num_controls]};
    }

    fn create_empty_transform_list(&self) -> Vec<Mat4>
    {
        return vec![Mat4::IDENTITY; self.bones.len()];
    }

    /*fn calculate_transforms(&self, pose: &ArmaturePose, dest: &mut Vec<Mat4>)
    {
        let mut pose_tfs = vec![];
        for i in 0..self.bones.len()
        {
            pose_tfs.push
            (
                match self.bones[i].parent 
                {
                    Some(p_dex) => pose_tfs[p_dex],
                    None => Mat4::IDENTITY,   
                }
            );
            dest[i] = pose_tfs[i] * self.bones[i].world_to_bone_tf;
        }
    }
    */
    pub fn num_bones(&self) -> usize
    {
        return self.bones.len();
    }
    
}

enum LayerIdentifier
{
    Base,
    Named{name: String},
}

pub struct AnimationState
{
    pub armature: Rc<Armature>,
    current_pose: ArmaturePose,
    pub base_layer: ArmatureLayer,
    layers: Vec<ArmatureLayer>
}

impl AnimationState
{

    pub fn pose(&self) -> &ArmaturePose
    {
        return &self.current_pose;
    }

    pub fn update(&mut self, dt : f32)
    {
        self.base_layer.update(dt);
        for layer in &mut self.layers
        {
            layer.update(dt);
        }
        let mut normalized_layer_weights: Vec<f32> = Vec::with_capacity(self.layers.len());
        let mut base_weight = 0.0;

        self.current_pose.clear();
        self.layers.iter().rev().for_each
        (
            |layer|
            {
                normalized_layer_weights.push( (1.0 - normalized_layer_weights.last().unwrap_or(&0.0)) * layer.weight.get_val() );
                base_weight += normalized_layer_weights.last().unwrap();
            }
        );
        self.base_layer.apply_to_pose(&mut self.current_pose, NormalizedFloat::clamped(1.0-base_weight));
        for layer in &self.layers
        {
            layer.apply_to_pose(&mut self.current_pose, NormalizedFloat::clamped(normalized_layer_weights.pop().unwrap()));
        }
    }

    pub fn write_current_pose_transforms(&self, dest: &mut [Mat4], inverse_buffer: &mut [Mat4])
    {
        self.current_pose.transforms(self.armature.as_ref(), dest, inverse_buffer);
    }

    pub fn set_layer_weight(&mut self, layer_name: &String, weight : NormalizedFloat)
    {
        match self.layers.iter_mut().find(|layer|  layer.name == *layer_name )
        {
            Some(layer) => layer.weight = weight,
            None => (),
        }
    }

    pub fn get_layer(&self, layer_id: &str) -> Option<&ArmatureLayer>
    {
        return self.layers.iter().find(|l| l.name == layer_id);
    }

    pub fn get_layer_mut(&mut self, layer_id: &str) -> Option<&mut ArmatureLayer>
    {
        if let Some(dex) = self.layers.iter().position(|l| l.name == layer_id)
        {
            return self.layers.get_mut(dex);
        }
        return Option::None;
    }

}

pub struct AnimationStateBuilder
{
    armature: Rc<Armature>,
    default_pose: ArmaturePose,
    base_builder: LayerBuilder,
    layer_builders: Vec<LayerBuilder>,
}

impl AnimationStateBuilder
{
    pub fn create_builder(armature: Rc<Armature>, mixers: Vec<MixerBuilder>) -> Self
    {
        let mut builder = LayerBuilder::create_builder(ArmatureLayer::BASE_LAYER_ID);
        for mixer in mixers
        {
            builder.add_mixer(mixer);
        }
        AnimationStateBuilder 
        {
            default_pose: armature.create_empty_pose(),
            armature: armature,
            base_builder: builder, 
            layer_builders: vec![]
        }
    }

    pub fn base_starting_animation(&mut self, id: &str) -> &Self
    {
        self.base_builder.starting_mixer(id);
        return self;
    }

    pub fn add_layer(&mut self, layer : LayerBuilder) -> &Self
    {
        self.layer_builders.push(layer);
        return self;
    }

    pub fn build(&self) -> AnimationState
    {
        let base_layer = self.base_builder.build();
        //base_layer.working_mixer_opt = Some(base_layer.mixers.keys().next().unwrap().clone());
        return AnimationState
        {
            current_pose: self.default_pose.clone(), 
            base_layer: base_layer,
            layers: self.layer_builders.iter().map(|builder|builder.build()).collect::<Vec<ArmatureLayer>>(),
            armature: self.armature.clone(),
        }
    }
}