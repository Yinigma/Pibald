mod test;

use std::{collections::{HashMap, vec_deque, VecDeque}, fs::{self, File}, io::{BufReader, Read, Seek}, rc::Rc, path::Path};

use glam::{Vec4, Vec3, Vec4Swizzles, Quat, Mat4};

use super::{render_state::model::StaticModel, render_state::{model::{Model, StaticVertex, Triangle, ColorPalette, Polygon, ArmatureWeight, AnimatedModel, AnimatedVertex}, common::{Color, NormalizedFloat}, animation::{Armature, AnimationClip, Bone, ClipChannel, AnimationKey}}};
use byteorder::{ReadBytesExt, BigEndian, LittleEndian};

struct CurveFrame
{
    frame: u32,
    value: f32,
}

pub struct InMemoryModelRepository
{
    static_models: HashMap<String, StaticModel>,
    animated_models: HashMap<String, AnimatedModel>,
    armatures: HashMap<String, Rc<Armature>>,
    animations: HashMap<String, Rc<AnimationClip>>
}

impl InMemoryModelRepository
{
    pub fn new() -> Self
    {
        return InMemoryModelRepository 
        { 
            static_models: HashMap::new(),
            animated_models: HashMap::new(),
            armatures: HashMap::new(),
            animations: HashMap::new(),
        };
    }

    pub fn load_static_model(&mut self, key: String, path: &Path)
    {
        //let model_str = fs::read_to_string(path).expect("Failed to read model file");
        let file = File::open(path).unwrap();
        let val = read_static_model(file, key.clone());
        self.static_models.insert(key, val);
    }

    pub fn load_animated_model(&mut self, key: String, path: &Path)
    {
        //let model_str = fs::read_to_string(path).expect("Failed to read model file");
        let file = File::open(path).unwrap();
        let val = read_animated_model(file, key.clone());
        self.animated_models.insert(key, val);
    }

    pub fn load_armature(&mut self, key: String, path: &Path)
    {
        let file = File::open(path).unwrap();
        let val = read_armature(key.clone(), file);
        self.armatures.insert(key, Rc::new(val));
    }

    pub fn load_animation(&mut self, anim_id: String, path: &Path, arma_key: &String)
    {
        let arma = self.armatures.get(arma_key).unwrap();
        let file = File::open(path).unwrap();
        let val = read_animation(file, arma);
        self.animations.insert(anim_id, Rc::new(val));
    }

    pub fn unload_static_model(&mut self, id: &String)
    {
        self.static_models.remove(id);
    }

    pub fn unload_animated_model(&mut self, id: &String)
    {
        self.animated_models.remove(id);
    }

    pub fn get_static_model(&self, id: &String) -> Option<&StaticModel>
    {
        return self.static_models.get(id);
    }

    pub fn get_animated_model(&self, id: &String) -> Option<&AnimatedModel>
    {
        return self.animated_models.get(id);
    }

    pub fn get_armature(&self, id: &String) -> Option<Rc<Armature>>
    {
        return self.armatures.get(id).cloned();
    }

    pub fn get_animation(&self, id: &String) -> Option<Rc<AnimationClip>>
    {
        return self.animations.get(id).cloned();
    }
}

fn read_flags(byte_rdr: &mut impl Read,) -> (bool, bool, bool)
{
    let flags = byte_rdr.read_u8().unwrap();
    let has_normals = (flags & 0b00000001) > 0;
    let has_colors = (flags & 0b00000010) > 0;
    let has_weights = (flags & 0b00000100) > 0;
    return (has_normals, has_colors, has_weights);
}

fn read_colors(byte_rdr: &mut impl Read,) -> Vec<ColorPalette>
{
    let colors = 
    {
        let num_colors = byte_rdr.read_u16::<LittleEndian>().unwrap() as usize;
        let mut temp = vec![];
        for _ in 0..num_colors
        {
            temp.push
            (
                Color
                { 
                    data: Vec4::new
                    (
                        byte_rdr.read_f32::<LittleEndian>().unwrap(),
                        byte_rdr.read_f32::<LittleEndian>().unwrap(),
                        byte_rdr.read_f32::<LittleEndian>().unwrap(),
                        1.0
                    ) 
                }
            );
        }
        temp
    };
    let mut palettes = vec![];
    let num_palettes = byte_rdr.read_u16::<LittleEndian>().unwrap() as usize;
    let palette_size = byte_rdr.read_u16::<LittleEndian>().unwrap() as usize;
    for _ in 0..num_palettes
    {
        let mut palette = vec![];
        for _ in 0..palette_size
        {
            let color_index = byte_rdr.read_u16::<LittleEndian>().unwrap() as usize;
            palette.push(colors[color_index]);
        }
        palettes.push(ColorPalette{colors: palette});
    }
    return palettes;
}

fn read_vec3_array(num_points: u32, byte_rdr: &mut impl Read,) -> Vec<Vec3>
{
    let mut points = vec![];
    for _ in 0..num_points
    {
        points.push
        (
            Vec3::new
            (
                byte_rdr.read_f32::<LittleEndian>().unwrap(),
                byte_rdr.read_f32::<LittleEndian>().unwrap(),
                byte_rdr.read_f32::<LittleEndian>().unwrap(),
            )
        );
    }
    return points;
}

fn skip_armature_weights(num_points: u32, byte_rdr: &mut (impl Read + Seek),)
{
    let armature_id_len = byte_rdr.read_u8().unwrap();
    let _ = byte_rdr.seek(std::io::SeekFrom::Current(armature_id_len as i64));
    for _ in 0..num_points
    {
        let num_weights = byte_rdr.read_u8().unwrap();
        //I'm not gonna bother using sizeof here. doesn't save any grief if you think about it
        let _ = byte_rdr.seek(std::io::SeekFrom::Current((num_weights as i64) * 6));
    }
}

fn read_armature_weights(num_points: u32, byte_rdr: &mut impl Read) -> (String, Vec<Vec<ArmatureWeight>>)
{
    let armature_id_len = byte_rdr.read_u8().unwrap() as usize;
    let mut buf: Vec<u8> = vec![0; armature_id_len];
    byte_rdr.read_exact(&mut buf).expect("failed to read armature id");
    let arma_id = String::from_utf8(buf).expect("failed to convert armature id");
    let mut points = vec![];
    for _ in 0..num_points
    {
        let mut v_weights = vec![];
        let num_weights = byte_rdr.read_u8().unwrap();
        for _ in 0..num_weights
        {
            v_weights.push
            (
                ArmatureWeight
                {
                    index: byte_rdr.read_u16::<LittleEndian>().unwrap() as usize,
                    weight: NormalizedFloat::clamped(byte_rdr.read_f32::<LittleEndian>().unwrap()),
                }
            );
        }
        points.push(v_weights);
    }
    return (arma_id, points);
}

fn read_geometry(points: &Vec<Vec3>, normals: &Vec<Vec3>, has_colors: bool, has_normals: bool, byte_rdr: &mut impl Read) -> (Vec<StaticVertex>, Vec<Polygon>, HashMap::<VertexKey, usize>)
{
    let num_tris = byte_rdr.read_u32::<LittleEndian>().unwrap() as usize;
    let mut tris = VecDeque::new();
    //todo - coalesce vertices with the same data to save space and have smaller buffer writes. 
    //This won't do anything for the usual type of model because due to the flat shading requiring vertices to have per-face normals  
    let mut verts_map = HashMap::<VertexKey, usize>::new();
    let mut verts = vec![];
    for _ in 0..num_tris
    {
        let mut tri = [0; 3];
        //todo - have another for loop up here for calculating the normal from the polygon's given postions for the case where the normals were not given/precalculated
        for t in 0..3
        {
            let loc_dex = byte_rdr.read_u32::<LittleEndian>().unwrap();
            let mut norm = Vec3::ZERO;
            let mut norm_dex = 0;
            if has_normals
            {
                norm_dex = byte_rdr.read_u32::<LittleEndian>().unwrap();
                norm = normals[norm_dex as usize];
            }
            let mut col = 0;
            if has_colors
            {
                col = byte_rdr.read_u16::<LittleEndian>().unwrap();
            }
            let key = VertexKey{ loc: loc_dex, color: col, norm: norm_dex };
            if verts_map.contains_key(&key)
            {
                tri[t] = verts_map[&key];
            }
            else
            {
                tri[t] = verts.len();
                verts_map.insert(key, verts.len());
                verts.push
                (
                    StaticVertex{ loc: points[loc_dex as usize], col: col as usize, norm }
                );
                
            }
        }
        if !has_normals
        {
            let calc_norm = (verts[tri[1]].loc - verts[tri[0]].loc).cross(verts[tri[2]].loc - verts[tri[0]].loc).normalize();
            for v in tri
            {
                verts[v].norm = calc_norm;
            }
        }
        tris.push_back(Triangle{ indices: tri })
    }
    let num_polys = byte_rdr.read_u32::<LittleEndian>().unwrap() as usize;
    let mut polys = vec![];
    for _ in 0..num_polys
    {
        
        let mut poly_tris = vec![];
        let poly_tri_count = byte_rdr.read_u16::<LittleEndian>().unwrap() as usize;
        for _ in 0..poly_tri_count
        {
            poly_tris.push(tris.pop_front().unwrap())
        }
        polys.push(Polygon{tris:poly_tris});
    }
    return (verts, polys, verts_map);
}

fn read_bbox(byte_rdr: &mut impl Read) -> (Vec3, Vec3)
{
    let min_bound = Vec3::new
    (
        byte_rdr.read_f32::<LittleEndian>().unwrap(),
        byte_rdr.read_f32::<LittleEndian>().unwrap(),
        byte_rdr.read_f32::<LittleEndian>().unwrap(),
    );
    let max_bound = Vec3::new
    (
        byte_rdr.read_f32::<LittleEndian>().unwrap(),
        byte_rdr.read_f32::<LittleEndian>().unwrap(),
        byte_rdr.read_f32::<LittleEndian>().unwrap(),
    );
    return (min_bound, max_bound);
}

fn read_static_model(mut byte_rdr: impl Read + Seek, id: String) -> StaticModel
{
    let (has_normals, has_colors, has_weights) = read_flags(&mut byte_rdr);
    let palettes = if has_colors { read_colors(&mut byte_rdr) } else { vec![ColorPalette{colors: vec![Color{data: Vec4::new(0.5, 0.5, 0.5, 1.0)}]}] };
    let num_points = byte_rdr.read_u32::<LittleEndian>().unwrap();
    let points =
    {
        read_vec3_array(num_points, &mut byte_rdr)
    };
    if has_weights
    {
        skip_armature_weights(num_points, &mut byte_rdr);
    }
    let normals = if has_normals
    {
        let num_norms = byte_rdr.read_u32::<LittleEndian>().unwrap();
        read_vec3_array(num_norms, &mut byte_rdr)
    }
    else
    {
        vec![]
    };
    let (verts, polys, _) = read_geometry(&points, &normals, has_colors, has_normals, &mut byte_rdr);
    let (min_bound, max_bound) = read_bbox(&mut byte_rdr);

    let model = Model 
    {
        polygons: polys, 
        palettes: palettes, 
        default_palette: 1, 
        shader_slots: HashMap::<_,_>::new(), 
        min_bound: min_bound, 
        max_bound: max_bound 
    };

    return StaticModel
    {
        id: id,
        vertices: verts,
        model_data: model
    };
}

fn read_animated_model(mut byte_rdr: impl Read + Seek, id: String) -> AnimatedModel
{
    let (has_normals, has_colors, has_weights) = read_flags(&mut byte_rdr);
    let palettes = if has_colors { read_colors(&mut byte_rdr) } else { vec![ColorPalette{colors: vec![Color{data: Vec4::new(0.5, 0.5, 0.5, 1.0)}]}] };
    let num_points = byte_rdr.read_u32::<LittleEndian>().unwrap();
    let points =
    {
        read_vec3_array(num_points, &mut byte_rdr)
    };
    let (arma_id, weights) = if has_weights{ read_armature_weights(num_points, &mut byte_rdr) } else { ("".to_string(), vec![]) };
    let normals = if has_normals
    {
        let num_norms = byte_rdr.read_u32::<LittleEndian>().unwrap();
        read_vec3_array(num_norms, &mut byte_rdr)
    } 
    else
    {
        vec![]
    };
    let (verts, polys, vert_map) = read_geometry(&points, &normals, has_colors, has_normals, &mut byte_rdr);
    let mut anim_verts = verts.into_iter().map(|s| AnimatedVertex{vert: s, weights: vec![]}).collect::<Vec<_>>();
    for (key, index) in vert_map
    {
        anim_verts[index].weights = weights[key.loc as usize].clone();
    }
    let (min_bound, max_bound) = read_bbox(&mut byte_rdr);

    let model = Model 
    {
        polygons: polys, 
        palettes: palettes, 
        default_palette: 1, 
        shader_slots: HashMap::<_,_>::new(), 
        min_bound: min_bound, 
        max_bound: max_bound 
    };

    return AnimatedModel
    {
        id: id,
        vertices: anim_verts,
        model_data: model,
        armature_id: arma_id,
    };
}

fn read_armature( id: String, mut byte_rdr: impl Read ) -> Armature
{
    let num_bones = byte_rdr.read_u16::<LittleEndian>().unwrap();
    let mut bones = vec![];
    for b in 0..num_bones
    {
        let bone_id_len = byte_rdr.read_u8().unwrap() as usize;
        let mut buf: Vec<u8> = vec![0; bone_id_len];
        byte_rdr.read_exact(&mut buf).unwrap();
        let bone_id = String::from_utf8(buf).unwrap();
        let loc = Vec3::new
        (
            byte_rdr.read_f32::<LittleEndian>().unwrap(),
            byte_rdr.read_f32::<LittleEndian>().unwrap(),
            byte_rdr.read_f32::<LittleEndian>().unwrap(),
        );
        let rot = Quat::from_vec4
        (
            Vec4::new
            (
                byte_rdr.read_f32::<LittleEndian>().unwrap(),
                byte_rdr.read_f32::<LittleEndian>().unwrap(),
                byte_rdr.read_f32::<LittleEndian>().unwrap(),
                byte_rdr.read_f32::<LittleEndian>().unwrap()
            )
        );
        let p_index = byte_rdr.read_u16::<LittleEndian>().unwrap() as usize;
        let parent = if b == 0 { Option::None } else { Option::Some(p_index) };
        bones.push(Bone::new(parent, loc, rot));
    }
    return Armature::new(id, bones);
}

fn read_animation(mut byte_rdr: impl Read, arma: &Armature) -> AnimationClip
{
    let mut x_location_channels = HashMap::new(); 
    let mut y_location_channels = HashMap::new(); 
    let mut z_location_channels = HashMap::new(); 
    let mut orientation_channels = HashMap::new();
    let mut x_scale_channels = HashMap::new();
    let mut y_scale_channels = HashMap::new();
    let mut z_scale_channels = HashMap::new();
    let anim_bone_count = byte_rdr.read_u16::<LittleEndian>().unwrap() as usize;
    let min_frame = byte_rdr.read_u16::<LittleEndian>().unwrap();
    let max_frame = byte_rdr.read_u16::<LittleEndian>().unwrap();
    for bone in 0..anim_bone_count
    {
        if bone  == 79
        {
            print!("");
        }
        let bone_id = byte_rdr.read_u16::<LittleEndian>().unwrap() as usize;
        let num_tracks = byte_rdr.read_u8().unwrap() as usize;
        let mut curves = (0..10).map(|_| vec![]).collect::<Vec<_>>();
        for _ in 0..num_tracks
        {
            let track_id = byte_rdr.read_u8().unwrap() as usize;
            let num_keyframes = byte_rdr.read_u16::<LittleEndian>().unwrap();
            for _ in 0..num_keyframes
            {
                let frame = byte_rdr.read_u16::<LittleEndian>().unwrap();
                curves[track_id].push
                (
                    AnimationKey
                    {
                        data: byte_rdr.read_f32::<LittleEndian>().unwrap(),
                        frame: frame
                    }
                );
            }
        }

        if let Option::Some(z_scale) = curves.pop()
        { 
            if z_scale.len() > 0 {z_scale_channels.insert(bone, ClipChannel::new(z_scale));}
        }
        if let Option::Some(y_scale) = curves.pop()
        { 
            if y_scale.len() > 0 {y_scale_channels.insert(bone, ClipChannel::new(y_scale));}
        }
        if let Option::Some(x_scale) = curves.pop()
        { 
            if x_scale.len() > 0 {x_scale_channels.insert(bone, ClipChannel::new(x_scale));}
        }

        {
            let w_channel = curves.pop().unwrap();
            let z_channel = curves.pop().unwrap();
            let y_channel = curves.pop().unwrap();
            let x_channel = curves.pop().unwrap();
            let orientation_data_empty = w_channel.is_empty();
            if [&z_channel, &y_channel, &x_channel].iter().any(|data| data.is_empty() != orientation_data_empty)
            {
                //actually riot if you gave me a quaternion that's straight up missing a component
                panic!("Not all orientation component curves contain keyframe data, or an orientation track meant to be empty erroneously contains data in at least one curve.");
            }
            if !orientation_data_empty
            {
                orientation_channels.insert(bone, [x_channel, y_channel, z_channel, w_channel].map(|c| ClipChannel::new(c)));
            }
        }

        if let Option::Some(z_loc) = curves.pop()
        {
            if z_loc.len() > 0 {z_location_channels.insert(bone, ClipChannel::new(z_loc));}
        }
        if let Option::Some(y_loc) = curves.pop()
        {
            if y_loc.len() > 0 {y_location_channels.insert(bone, ClipChannel::new(y_loc));}
        }
        if let Option::Some(x_loc) = curves.pop()
        {
            if x_loc.len() > 0 {x_location_channels.insert(bone, ClipChannel::new(x_loc));}
        }
    }
    return AnimationClip::new
    (
        min_frame,
        max_frame,
        30,
        x_location_channels,
        y_location_channels,
        z_location_channels,
        orientation_channels,
        x_scale_channels,
        y_scale_channels,
        z_scale_channels,
        vec![]
    );
    
}

#[derive(Hash, PartialEq, Eq)]
struct VertexKey
{
    loc: u32,
    norm: u32,
    color: u16,
}