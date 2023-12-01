const ARGCOUNT = 15u; //should probably adjust this for a convenient byte alignment
const COLORMAPCOUNT = 16u;
const PLACEMENTCOUNT: u32 = 128u;
const COLORCOUNT = 16u;
const MODELCOLORCOUNT = 128u;

//enumeration of sdf instruction codes
const SDCIRCLECYLINDER = 0u;
const SDBOXCYLINDER = 1u;
const SDSPHERE = 3u;
const SDPOLYGON = 4u;
const SDREGPOLYGON = 5u;
const SDPOLYSTAR = 6u;
const SDPLANE = 7u;
const OPOFFSET = 65536u;
const OPMIN = 65536u;
const OPAVG = 65537u;
const OPMASK = 65538u;
const OPWAVESHEET = 65539u;
const OPWAVERING = 65540u;

//enumeration of color interpolation types
const LINEAR = 0u;
const STEP = 1u;

//enumeration of exatrapolation types
const LASTCOLOR = 0u;
const REPEAT = 1u;
const REPEATREFLECT = 2u;

const SPOTLIGHTCOUNT = 1024u;
const POINTLIGHTCOUNT = 1024u;

const MAXBONES = 256u;
const NUMBONESPERVERT = 8u;


//SDF instruction stack
struct SDFInstruction
{
    code : u32,
    args : array<f32, ARGCOUNT>, //instruction handlers will only read the filled-in values for these
}


struct GradientStep
{
    color: vec4<f32>,
    location : f32,
    interpolation_type: u32,
}

struct ColorMap
{
    starting_instruction: u32,
    ending_instruction: u32,
    outer_colors : array<GradientStep, COLORCOUNT>,
    inner_colors : array<GradientStep, COLORCOUNT>,
    //distances before reaching the end of a cycle
    outer_distance : f32,
    inner_distance : f32,
}


struct Placement
{
	map_index : u32,
	transform : mat4x4<f32>,
}

struct PibaldBuffer
{
    maps : array<ColorMap, COLORMAPCOUNT>,
    placements : array<Placement, PLACEMENTCOUNT>,
    //the one unsized buffer we're allowed
    sdf_stack : array<SDFInstruction,1>,
}

struct ColorList
{
	colors: array<vec4<f32>, MODELCOLORCOUNT>,
}

fn project(p : vec3<f32>, o: vec3<f32>, n: vec3<f32>) -> vec3<f32>
{
    return (p-o) - (dot((p-o), n) * n);
}

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

//fn process_commands(buffer: PibaldBuffer, test: vec2<f32>) -> vec4<f32>
//{
//	let eval_stack = array<f32, 32>();
//	for(var i = 0; i < 1; i++)
//	{	
//		let index = buffer.placements[0];
//		let color_map = buffer.maps[0];
//		for(var j = color_map.starting_instruction; j < color_map.ending_instruction + 1u; j++)
//		{
//			return get_color(eval_sdf2(buffer.sdf_stack[0], eval_stack, test), color_map);	
//		}
//	}
//	return vec4<f32>(1.0, 1.0, 1.0, 1.0);
//}

fn eval_sdf2(instr : SDFInstruction, eval_stack : array<f32, 32>, test : vec2<f32>) -> f32
{
	switch instr.code
	{
		case 0u:
		{
            //args[0] - radius
			return length(test) - instr.args[0];
		}
		default:
		{
			return 0.0;
		}
	}
	return 0.0;
}

//fn get_color(dist : f32, grad : ColorMap) -> vec4<f32>
//{
//	if(dist < 0.0)
//	{
//		for(var i = 1; i < 2; i++)
//		{
  //          let last_dex = i-1;
	//		let n_dist = (abs(dist) / grad.inner_distance) % 1.0;
	//		let color = grad.inner_colors[i];
	//		if (grad.inner_colors[i].location >= n_dist)
	//		{
	//			switch grad.inner_colors[i].interpolation_type
	//			{
	//				case STEP
	//				{
	//					return vec4<f32>(0.0, 1.0, 0.0, 1.0);
	//					//return grad.inner_colors[0].color;
	//				}
	//				case LINEAR
	//				{
	//					
	//					let t = (n_dist - grad.inner_colors[i].location) / (grad.inner_colors[i].location - grad.inner_colors[last_dex].location);
	//					return mix(grad.inner_colors[last_dex].color, grad.inner_colors[i].color, t);
	//				}
	//				default:
	//				{
	//					return vec4<f32>(1.0, 0.0, 0.0, 1.0);
	//				}
	//			}
	//		}
	//	}
		//default if for some reason the gradient doesn't go from 0-1
	//	return grad.inner_colors[0].color;
	//}
	//return vec4<f32>(0.0, 0.0, 0.0, 1.0);
//}

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
//@group(1) @binding(1)
//var<storage> command_buffer: PibaldBuffer;

struct VertexInput 
{
    @location(0) position: vec3<f32>,
	@location(1) normal: vec3<f32>,
    @location(2) color: u32,
}

struct VertexOutput 
{
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec4<f32>,
	@location(1) color: vec4<f32>,
	@location(2) normal: vec3<f32>,
}



@vertex
fn vs_static(v_in: VertexInput,) -> VertexOutput
{
    var out: VertexOutput;
	out.normal = (model * vec4<f32>(v_in.normal, 0.0)).xyz;
    out.world_position = model * vec4<f32>(v_in.position, 1.0);
    out.clip_position = view_proj * out.world_position;
    out.color = colors[v_in.color];
    return out;
}
// Fragment shader

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
	var out_color = mix(in.color.xyz, vec3<f32>(0.343, 0.347, 0.571), 0.9) * 0.1;
	for (var i = 0u; i < point_lights.count; i++)
	{
		let diff = point_lights.lights[i].loc - in.world_position.xyz;
		let dist = length(diff);
		let factor = max(dot(normalize(diff), in.normal), 0.0);
		out_color += factor * in.color.xyz * point_lights.lights[i].color * attenuate(dist, point_lights.lights[i].radius, point_lights.lights[i].intensity, point_lights.lights[i].cutoff);
	}
    return vec4<f32>(out_color.xyz, 1.0);
}