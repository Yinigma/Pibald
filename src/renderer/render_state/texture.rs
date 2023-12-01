use std::rc::Rc;

use super::properties::{Expression, PropertyGroup, EvalTable, Value, AssignmentError};

pub struct Shader
{
    pub id: String,
    pub color_maps : Vec<ColorMap>,
    pub placements : Vec<Placement>,
    pub default_args : PropertyGroup
}

impl Shader
{
    pub fn create_properties_instance(&self) -> PropertyGroup
    {
        return self.default_args.clone();
    }

    pub fn create_value_table_instance(&self) -> EvalTable
    {
        let mut table = EvalTable::new();
        self.eval(&self.default_args, &mut table);
        return table;
    }

    pub fn eval(&self, args: &PropertyGroup, dest: &mut EvalTable)
    {
        for placement in &self.placements
        {
            dest.update(&placement.tf, args);
            match &placement.variant
            {
                PlacementVariant::Singular() => (),
                PlacementVariant::TilePattern(expr) => dest.update(&expr, args),
            }
        }
        for map in &self.color_maps
        {
            for term in &map.sdf_stack
            {
                term.eval(args, dest);
            }
            match &map.variant
            {
                ColorMapVariant::Gradient(grad) => 
                {
                    for color_point in &grad.inner_grad.color_points
                    {
                        dest.update(&color_point.color, args);
                        dest.update(&color_point.val, args);
                    }
                    if let Some(outer_grad) = &grad.outer_grad
                    {
                        for color_point in &outer_grad.color_points
                        {
                            dest.update(&color_point.color, args);
                            dest.update(&color_point.val, args);
                        }
                    }
                },
                ColorMapVariant::Binary(binary) => 
                {
                    dest.update(&binary.color, args);
                },
            }
        }
    }
}

pub struct ColorMap
{
    pub variant : ColorMapVariant,
    pub sdf_stack : Vec<SDFTerm>
}

pub enum ColorMapVariant
{
    Gradient(GradientColorMap),
    Binary(BinaryColorMap),
}

struct BinaryColorMap
{
    color : Expression,
}

pub struct GradientColorMap
{
    pub inner_grad: ColorGradient,
    pub outer_grad: Option<ColorGradient>,
}

pub struct ColorGradient
{
    extrapolation : GradientExtrapolation,
    color_points : Vec<ColorPoint>,
    max_distance : Expression
}
pub struct Placement
{
    index : u32,
    tf : Expression,
    variant : PlacementVariant,
}

enum PlacementVariant
{
    Singular(),
    TilePattern(Expression),
}

struct ColorPoint
{
    val : Expression,
    color : Expression,
    interpolation_mode : GradientInterpolation
}

enum GradientInterpolation
{
    Linear,
    Step,
}

enum GradientExtrapolation
{
    LastColor,
    Repeat,
    RepeatReflect,
}

pub enum SDFTerm
{
    Operator(SDFOperator),
    Operand(SDFOperand),
}

impl SDFTerm
{
    fn eval(&self, args: &PropertyGroup, dest: &mut EvalTable)
    {
        match self 
        {
            SDFTerm::Operator(_) => (),
            SDFTerm::Operand(operand) => operand.eval(args, dest),
        }
    }
}

enum SDFOperator
{
    Minimum,
    Average,
    Mask,
    Round,
    WaveSheet,
    WaveRing,
}

pub enum SDFOperand
{
    Circle{tf: Expression, radius: Expression},
    Rectangle{tf: Expression, width: Expression, height: Expression},
    Sphere{tf: Expression, radius: Expression},
    Plane{tf: Expression},
    Polygon{tf: Expression, points: Vec<Expression>},
    RegularPolygon{tf: Expression, num_points: Expression, radius: Expression},
    PolyStar{tf: Expression, numpoints: Expression, inner_radius: Expression, outer_radius: Expression},
}

impl SDFOperand
{
    fn eval(&self, args: &PropertyGroup, dest: &mut EvalTable)
    {
        match self 
        {
            SDFOperand::Circle { tf, radius } => 
            {
                dest.update(tf, args);
                dest.update(radius, args);
            },
            SDFOperand::Rectangle { tf, width, height } => 
            {
                dest.update(tf, args);
                dest.update(width, args);
                dest.update(height, args);
            },
            SDFOperand::Sphere { tf, radius } => 
            {
                dest.update(tf, args);
                dest.update(radius, args);
            },
            SDFOperand::Plane { tf } => 
            {
                dest.update(tf, args);
            },
            SDFOperand::Polygon { tf, points } => 
            {
                dest.update(tf, args);
                for p in points
                {
                    dest.update(p, args);
                }
            },
            SDFOperand::RegularPolygon { tf, num_points, radius } => 
            {
                dest.update(tf, args);
                dest.update(num_points, args);
                dest.update(radius, args);
            },
            SDFOperand::PolyStar { tf, numpoints, inner_radius, outer_radius } => 
            {
                dest.update(tf, args);
                dest.update(numpoints, args);
                dest.update(inner_radius, args);
                dest.update(outer_radius, args);
            },
        }
    }
}

#[derive(Clone)]
enum ExternalShaderValue
{
    Color(usize),
    AnimationTrack(String)
}

#[derive(Clone)]
pub struct ShaderValueLink
{
    active: bool,
    property_name: String,
    value: ExternalShaderValue,
}

pub struct ShaderInstance
{
    shader : Rc<Shader>,
    properties : PropertyGroup,
    links: Vec<ShaderValueLink>,
    expression_cache: EvalTable,
}

impl ShaderInstance
{
    pub fn new(shader: Rc<Shader>, links: Vec<ShaderValueLink>) -> Self
    {
        let eval_table = shader.create_value_table_instance();
        let properties = shader.create_properties_instance();
        return ShaderInstance 
        {
            shader: shader,
            properties: properties,
            links: links,
            expression_cache: eval_table,
        };
    }

    pub fn set_property(&mut self, name: &str, val: Value) -> Result<(), AssignmentError>
    {
        return self.properties.set_property(name, val);
    }

    pub fn set_link_toggle(&mut self, id: usize, active: bool)
    {
        if let Some(link) = self.links.get_mut(id)
        {
            link.active = active;
        }
    }

    pub fn eval_expressions(&mut self)
    {
        self.shader.eval(&self.properties, &mut self.expression_cache);
    }
}