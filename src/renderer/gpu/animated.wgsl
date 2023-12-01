const SPOTLIGHTCOUNT = 1024u;
const POINTLIGHTCOUNT = 1024u;

const MAXBONES = 128u;
const NUMBONESPERVERT = 8u;

const MODELCOLORCOUNT = 32u;

struct SpotLight
{
    color:  vec3<f32>, //rgb
    intensity: f32,
    loc: vec3<f32>,
    radius: f32,
    dir: vec3<f32>,
    cutoff: f32,
}

struct PointLight
{
    color:  vec3<f32>, //rgb
    intensity: f32,
    loc: vec3<f32>,
    radius: f32,
    cutoff: f32,
}

struct PointLightStore
{
	lights: array<PointLight, POINTLIGHTCOUNT>,
	count: u32
}

struct SpotLightStore
{
	lights: array<SpotLight, SPOTLIGHTCOUNT>,
	count: u32
}

fn attenuate(dist: f32, rad: f32, max_intensity: f32, cutoff: f32) -> f32
{
	if(dist < rad)
	{
		return max_intensity;
	}
	let ext_cutoff = cutoff + rad;
	return max_intensity * (dist - cutoff) * (dist - cutoff) / (cutoff * cutoff);
}

fn blend_colors(c0 :vec4<f32>, c1 :vec4<f32>) -> vec4<f32>
{
    let over = (1.0-c0.a)*c1.a + c0.a;
    return vec4<f32>((c0.rgb*c0.a + c1.rgb*c1.a*(1.0-c0.a))/over, over);
}

@group(0) @binding(0)
var<uniform> view_proj: mat4x4<f32>;
@group(1) @binding(0)
var<storage> point_lights: PointLightStore;
@group(1) @binding(1)
var<storage> spot_lights: SpotLightStore;
@group(2) @binding(0)
var<uniform> model: mat4x4<f32>;
@group(2) @binding(1)
var<uniform> colors: array<vec4<f32>, MODELCOLORCOUNT>;
@group(2) @binding(2)
var<uniform> anim_transforms: array<mat4x4<f32>, MAXBONES>;

struct AnimVertexInput
{
	@location(0) position: vec3<f32>,
	@location(1) normal: vec3<f32>,
    @location(2) color: u32,
	@location(3) weights: vec4<u32>,
	@location(4) indices: vec2<u32>,
}

struct VertexOutput 
{
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec4<f32>,
	@location(1) color: vec4<f32>,
	@location(2) normal: vec3<f32>,
}

@vertex
fn vs_animated(v_in: AnimVertexInput,) -> VertexOutput
{
	var out: VertexOutput;
	let w_0123 = vec4<f32>(unpack2x16unorm(v_in.weights[0]), unpack2x16unorm(v_in.weights[1]));
	let w_4567 = vec4<f32>(unpack2x16unorm(v_in.weights[2]), unpack2x16unorm(v_in.weights[3]));
	let i_0123 = vec4<u32>(v_in.indices[0] & 0xffu, v_in.indices[0] & 0xff00u, v_in.indices[0] & 0xff0000u, v_in.indices[0] & 0xff000000u);
	let i_4567 = vec4<u32>(v_in.indices[1] & 0xffu, v_in.indices[1] & 0xff00u, v_in.indices[1] & 0xff0000u, v_in.indices[1] & 0xff000000u);
	let anim_pos = 
	w_0123.x * (anim_transforms[i_0123.x] * vec4<f32>(v_in.position, 1.0)) +
	w_0123.y * (anim_transforms[i_0123.y] * vec4<f32>(v_in.position, 1.0)) +
	w_0123.z * (anim_transforms[i_0123.z] * vec4<f32>(v_in.position, 1.0)) +
	w_0123.w * (anim_transforms[i_0123.w] * vec4<f32>(v_in.position, 1.0)) +
	w_4567.x * (anim_transforms[i_4567.x] * vec4<f32>(v_in.position, 1.0)) +
	w_4567.y * (anim_transforms[i_4567.y] * vec4<f32>(v_in.position, 1.0)) +
	w_4567.z * (anim_transforms[i_4567.z] * vec4<f32>(v_in.position, 1.0)) +
	w_4567.w * (anim_transforms[i_4567.w] * vec4<f32>(v_in.position, 1.0));
	let anim_normal = 
	w_0123.x * (anim_transforms[i_0123.x] * vec4<f32>(v_in.normal, 0.0)) +
	w_0123.y * (anim_transforms[i_0123.y] * vec4<f32>(v_in.normal, 0.0)) +
	w_0123.z * (anim_transforms[i_0123.z] * vec4<f32>(v_in.normal, 0.0)) +
	w_0123.w * (anim_transforms[i_0123.w] * vec4<f32>(v_in.normal, 0.0)) +
	w_4567.x * (anim_transforms[i_4567.x] * vec4<f32>(v_in.normal, 0.0)) +
	w_4567.y * (anim_transforms[i_4567.y] * vec4<f32>(v_in.normal, 0.0)) +
	w_4567.z * (anim_transforms[i_4567.z] * vec4<f32>(v_in.normal, 0.0)) +
	w_4567.w * (anim_transforms[i_4567.w] * vec4<f32>(v_in.normal, 0.0));
	out.normal = (model * vec4<f32>(anim_normal.xyz, 0.0)).xyz;
    out.world_position = model * vec4<f32>(anim_pos.xyz, 1.0);
    out.clip_position = view_proj * out.world_position;
    out.color = colors[v_in.color];
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> 
{
    //let eval_stack = array<f32, 32>();
    //var out_color = vec4<f32>(0.0, 0.0, 0.0, 0.0);
	//for(var i = 0; i < 1; i++)
	//{	
	//	let index = command_buffer.placements[i];
	//	let color_map = command_buffer.maps[index.map_index];
	//	for(var j = color_map.starting_instruction; j < color_map.ending_instruction + 1u; j++)
	//	{
    //        let map_pos = index.transform * in.world_position;
	//		out_color = blend_colors(out_color, get_color(eval_sdf2(command_buffer.sdf_stack[j], eval_stack, map_pos.xy), color_map));
    //        if(out_color.a == 1.0)
    //        {
    //            break;
    //        }
	//	}
	//}
	var out_color = mix(in.color.xyz, vec3<f32>(0.543, 0.547, 0.771), 0.9) * 0.1;
	for (var i = 0u; i < point_lights.count; i++)
	{
		let diff = point_lights.lights[i].loc - in.world_position.xyz;
		let dist = length(diff);
		let factor = max(dot(normalize(diff), in.normal), 0.0);
		out_color += factor * in.color.xyz * point_lights.lights[i].color * attenuate(dist, point_lights.lights[i].radius, point_lights.lights[i].intensity, point_lights.lights[i].cutoff);
	}
    return vec4<f32>(out_color.xyz, 1.0);
}