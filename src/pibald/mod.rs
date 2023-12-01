mod test;
use std::collections::HashMap;
use std::fmt;
use pest::Parser;
use pest::iterators::Pair;

#[derive(Parser)]
#[grammar = "pibald.pest"]
struct PibaldParser;

#[derive(Debug)]
enum PibaldError
{
    ParseError,
    UnknownIdentifierError,
    MissingArgumentError(String),
    DivideByZeroError,
    InvalidExpressionError(String),
}

impl std::error::Error for PibaldError {}

impl fmt::Display for PibaldError
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result 
    {
        match self 
        {
            PibaldError::ParseError => write!(f, "ParseError!"),
            PibaldError::UnknownIdentifierError => write!(f, "Undefined identifier."),
            PibaldError::DivideByZeroError => write!(f, "Division by zero"),
            PibaldError::MissingArgumentError(param) => write!(f, "No argumant found for parameter \"{}\"", param),
            PibaldError::InvalidExpressionError(desc) => write!(f, "Invalid scalar expression: {}", desc)
        }
    }
}

impl PibaldParser
{
    fn parse_shader_class(input : &str) -> Result<ShapeShaderClass, PibaldError>
    {
        let mut pairs = match PibaldParser::parse(Rule::pibald, input) 
        {
            Ok(res) => res,
            //todo - actually bubble this error message up correctly
            Err(err) => 
            {
                println!
                (
                    "{}", err
                );
                return Err(PibaldError::ParseError)
            },
        };
        let mut maps : Vec<ColorMap> = vec![];
        let shader_pairs = pairs.next().unwrap().into_inner();
        let mut placements: Vec<Placement> = vec![];
        for pair in shader_pairs
        {
            match pair.as_rule()
            {
                Rule::color_map => 
                {
                    
                    for col_map_pair in pair.into_inner()
                    {
                        match col_map_pair.as_rule()
                        {
                            Rule::solid_map => 
                            {
                                let mut val_map: Vec<SDFTerm> = vec![];
                                let mut color = Color::ParamColor(ParamColor::from_constant([0.0,0.0,0.0,1.0]));
                                for solid_map_pair in col_map_pair.into_inner()
                                {
                                    match solid_map_pair.as_rule()
                                    {
                                        Rule::color => 
                                        {
                                            color = PibaldParser::parse_color(solid_map_pair);
                                        },
                                        Rule::val_map =>
                                        {
                                            val_map.append(&mut PibaldParser::parse_value_map(solid_map_pair));
                                        }
                                        _ => (),
                                    }
                                }
                                maps.push
                                (
                                    ColorMap
                                    {
                                        variant : ColorMapVariant::Binary(BinaryColorMap{ color: color }),
                                        sdf_stack : val_map,
                                    }
                                );
                            },
                            Rule::grad_map =>
                            {
                                let mut val_map: Vec<SDFTerm> = vec![];
                                let mut has_first_gradient = false;
                                let mut grad = ColorGradient{ extrapolation : GradientExtrapolation::LastColor(), color_points: vec![], max_distance: ScalarExpression::from_constant(0.0) };
                                let mut outer_grad: Option<ColorGradient> = None;
                                for grad_map_pair in col_map_pair.into_inner()
                                {
                                    match grad_map_pair.as_rule() 
                                    {
                                        Rule::gradient => 
                                        {
                                            let mut grad_pairs = grad_map_pair.into_inner();
                                            grad_pairs.next(); //COLOR_GRADIENT
                                            grad_pairs.next(); //L_PAREN
                                            let distance = ScalarExpression::new(PibaldParser::parse_scalar_expr(grad_pairs.next().unwrap())); //distance
                                            grad_pairs.next(); //COMMA
                                            let extrapolation_type = match grad_pairs.next().unwrap().as_rule()
                                            {
                                                Rule::LAST_COLOR => GradientExtrapolation::LastColor(),
                                                Rule::REPEAT => GradientExtrapolation::Repeat(),
                                                Rule::REPEAT_REFLECT => GradientExtrapolation::RepeatReflect(),
                                                _ => GradientExtrapolation::LastColor()
                                            };
                                            let mut grad_points: Vec<ColorPoint> = vec![];
                                            for grad_pair in grad_pairs
                                            {
                                                match grad_pair.as_rule() 
                                                {
                                                    Rule::grad_point => 
                                                    {
                                                        let mut color_point_pairs = grad_pair.into_inner();
                                                        color_point_pairs.next();//GRAD_POINT
                                                        color_point_pairs.next();//L_PAREN
                                                        let color = PibaldParser::parse_color(color_point_pairs.next().unwrap());
                                                        color_point_pairs.next();//COMMA
                                                        let point_dist = ScalarExpression::new(PibaldParser::parse_scalar_expr(color_point_pairs.next().unwrap()));
                                                        color_point_pairs.next();//COMMA
                                                        let interpolation = match color_point_pairs.next().unwrap().as_rule()
                                                        {
                                                            Rule::STEP => GradientInterpolation::Step(),
                                                            Rule::LINEAR => GradientInterpolation::Linear(),
                                                            _ => GradientInterpolation::Step()
                                                        };
                                                        grad_points.push(ColorPoint { val: point_dist, color: color, interpolation_mode:interpolation });
                                                    },
                                                    _ => ()
                                                }
                                            }
                                            let gradient = ColorGradient { extrapolation: extrapolation_type, color_points: grad_points, max_distance: distance };
                                            if(has_first_gradient)
                                            {
                                                outer_grad = Some(gradient);
                                            }
                                            else
                                            {
                                                grad = gradient;
                                            }
                                        },
                                        Rule::val_map =>
                                        {
                                            val_map.append(&mut PibaldParser::parse_value_map(grad_map_pair));
                                        },
                                        _ => ()
                                    }
                                }
                                maps.push
                                (
                                    ColorMap
                                    {
                                        variant : ColorMapVariant::Gradient
                                        (
                                            GradientColorMap { inner_grad: grad, outer_grad : outer_grad }
                                        ),
                                        sdf_stack : val_map,
                                    }
                                );
                            },
                            (_) => (),    
                        }
                    }
                },
                Rule::placement =>
                {
                    let mut placement_pairs = pair.into_inner();
                    let det_pair = placement_pairs.next().unwrap();
                    placement_pairs.next();//L_PAREN
                    let mat = PibaldParser::parse_matrix(placement_pairs.next().unwrap());
                    placement_pairs.next(); //COMMA
                    let index = placement_pairs.next().unwrap().as_str().trim().parse::<u32>().unwrap();
                    match det_pair.as_rule()
                    {
                        Rule::SINGULAR => 
                        {
                            placements.push(Placement { index: index, tf: mat, variant: PlacementVariant::Singular() });
                        },
                        Rule::TILE_PATTERN => 
                        {
                            let offset = PibaldParser::parse_vector3(placement_pairs.next().unwrap());
                            placements.push(Placement { index: index, tf: mat, variant: PlacementVariant::TilePattern(offset)})
                        },
                        _ => (),
                    }
                },
                (_) => (),
            }
        }
        return Ok(ShapeShaderClass{color_maps : maps, placements: placements});
    }

    fn parse_scalar_expr(pair: Pair<Rule>) -> Vec<ScalarTerm>
    {
        let mut scalar_stack: Vec<ScalarTerm> = vec![];
        let mut sum_op: Option<ScalarOperator> = None;
        for expr_pair in pair.into_inner()
        {
            match expr_pair.as_rule()
            {
                Rule::sum_term => 
                {
                    let mut product_op: Option<ScalarOperator> = None;
                    for sum_pair in expr_pair.into_inner()
                    {
                        match sum_pair.as_rule()
                        {
                            Rule::product_term => 
                            {
                                let mut has_pow_op = false;
                                for prod_pair in sum_pair.into_inner()
                                {
                                    match prod_pair.as_rule()
                                    {
                                        Rule::unary =>
                                        {
                                            let mut unary = prod_pair.into_inner();
                                            let unary_det_pair = unary.next().unwrap();
                                            match unary_det_pair.as_rule()
                                            {
                                                Rule::primary =>
                                                {
                                                    let mut primary = unary_det_pair.into_inner();
                                                    let primary_det_pair = primary.next().unwrap();
                                                    match primary_det_pair.as_rule()
                                                    {
                                                        Rule::L_PAREN => 
                                                        {
                                                            scalar_stack.append(&mut PibaldParser::parse_scalar_expr(primary.next().unwrap()));
                                                        },
                                                        Rule::ID =>
                                                        {
                                                            scalar_stack.push(ScalarTerm::Value(ScalarOperand::Variable(primary_det_pair.as_str().trim().to_string())));
                                                        },
                                                        Rule::constant =>
                                                        {
                                                            match primary_det_pair.into_inner().next().unwrap().as_rule()
                                                            {
                                                                Rule::PI => 
                                                                {
                                                                    scalar_stack.push(ScalarTerm::Value(ScalarOperand::Constant(std::f32::consts::PI)));
                                                                },
                                                                Rule::EULER => 
                                                                {
                                                                    scalar_stack.push(ScalarTerm::Value(ScalarOperand::Constant(std::f32::consts::E)));
                                                                },
                                                                _ => (),
                                                            }
                                                        },
                                                        Rule::REAL =>
                                                        {
                                                            scalar_stack.push(ScalarTerm::Value(ScalarOperand::Constant(primary_det_pair.as_str().trim().parse::<f32>().unwrap())));
                                                        },
                                                        _ => (),
                                                    }
                                                    
                                                },
                                                Rule::SUB =>
                                                {
                                                    scalar_stack.append(&mut PibaldParser::parse_scalar_expr(unary.next().unwrap()));
                                                    scalar_stack.push(ScalarTerm::Operator(ScalarOperator::Negate));
                                                },
                                                Rule::scalar_op =>
                                                {
                                                    let op = match unary_det_pair.into_inner().next().unwrap().as_rule()
                                                    {
                                                        Rule::FN_SINE => ScalarOperator::Sine,
                                                        Rule::FN_COSINE => ScalarOperator::Cosine,
                                                        Rule::FN_TANGENT => ScalarOperator::Tangent,
                                                        Rule::FN_LOG => ScalarOperator::Log,
                                                        //should never happen
                                                        _ => ScalarOperator::Sine,
                                                    };
                                                    unary.next();
                                                    scalar_stack.append(&mut PibaldParser::parse_scalar_expr(unary.next().unwrap()));
                                                    scalar_stack.push(ScalarTerm::Operator(op));
                                                },
                                                _ => (),
                                            }
                                            if has_pow_op
                                            {
                                                scalar_stack.push( ScalarTerm::Operator(ScalarOperator::Exponent) );
                                                has_pow_op = false;
                                            }
                                        },
                                        Rule::POW => 
                                        {
                                            has_pow_op = true;
                                        },
                                        _=>()
                                    }
                                }
                                if product_op.is_some()
                                {
                                    scalar_stack.push(ScalarTerm::Operator(product_op.unwrap()));
                                    product_op = None;
                                }
                            },
                            Rule::product_op => 
                            {
                                let op = sum_pair.into_inner().next().unwrap();
                                match op.as_rule()
                                {
                                    Rule::MUL => 
                                    {
                                        product_op = Some(ScalarOperator::Multiply);
                                    },
                                    Rule::DIV => 
                                    {
                                        product_op = Some(ScalarOperator::Divide);
                                    },
                                    Rule::MOD => 
                                    {
                                        product_op = Some(ScalarOperator::Modulo);
                                    },
                                    _=>()
                                }
                            },
                            _ => (),
                        }
                    }
                    if sum_op.is_some()
                    {
                        scalar_stack.push(ScalarTerm::Operator(sum_op.unwrap()));
                        sum_op = None;
                    }
                },
                Rule::sum_op => 
                {
                    let op = expr_pair.into_inner().next().unwrap();
                    match op.as_rule()
                    {
                        Rule::ADD => 
                        {
                            sum_op = Some(ScalarOperator::Add);
                        },
                        Rule::SUB => 
                        {
                            sum_op = Some(ScalarOperator::Subtract);
                        },
                        _=>()
                    }
                },
                _ => (),
            }
        }
        return scalar_stack;
    }

    fn parse_matrix(pair: Pair<Rule>) -> Matrix
    {
        let mut loc: Option<Vector> = None;
        let mut rot: Option<Vector> = None;
        let mut scale: Option<Vector> = None;
        let mut shear: Option<Vector> = None;
        for mat_pair in pair.into_inner()
        {
            match mat_pair.as_rule() 
            {
                /*Rule::tf_arg =>
                {
                    let mut param_pair = mat_pair.into_inner();
                    let param = param_pair.next().unwrap(); //param_name
                    param_pair.next(); //LPAREN
                    let vec = PibaldParser::parse_vector3(param_pair.next().unwrap());
                    match param.as_rule()
                    {
                        Rule::TRANSLATION => loc = Some(vec),
                        Rule::ROTATION => rot = Some(vec),
                        Rule::SCALE => scale = Some(vec),
                        Rule::SHEAR => shear = Some(vec),
                        _ => (),
                    }
                },*/
                Rule::ID => 
                {
                    return Matrix::IdMatrix(mat_pair.as_str().to_string());
                }
                _ => (),
            }
        }
        return Matrix::ParamMatrix
        (
            ParamMatrix 
            { 
                location: match loc 
                {
                    Some(vec) => vec, 
                    None => Vector::ParamVector(ParamVector::from_constant([0.0,0.0,0.0])),
                }, 
                rotation: match rot 
                {
                    Some(vec) => vec, 
                    None => Vector::ParamVector(ParamVector::from_constant([0.0,0.0,0.0])),
                },
                scale: match scale
                {
                    Some(vec) => vec, 
                    None => Vector::ParamVector(ParamVector::from_constant([0.0,0.0,0.0])),
                }, 
                shear: match shear 
                {
                    Some(vec) => vec, 
                    None => Vector::ParamVector(ParamVector::from_constant([0.0,0.0,0.0])),
                }, 
            }
        );
    }

    fn parse_vector3(pair: Pair<Rule>) -> Vector
    {
        let mut data: Vec<ScalarExpression> = vec![];
        for vec_pair in pair.into_inner()
        {
            match vec_pair.as_rule() 
            {
                Rule::scalar => 
                {
                    data.push(ScalarExpression::new(PibaldParser::parse_scalar_expr(vec_pair)));
                },
                Rule::ID => 
                {
                    return Vector::IdVector(vec_pair.as_str().to_string());
                },
                _ => (),
            }
        }
        let x = data.remove(0);
        let y = data.remove(0);
        let z = data.remove(0);
        return Vector::ParamVector(ParamVector{data: [x,y,z]});
    }

    fn parse_color(pair: Pair<Rule>) -> Color
    {
        let mut data: Vec<ScalarExpression> = vec![];
        for col_pair in pair.into_inner()
        {
            match col_pair.as_rule() 
            {
                Rule::scalar => 
                {
                    data.push(ScalarExpression::new(PibaldParser::parse_scalar_expr(col_pair)));
                },
                Rule::ID => 
                {
                    return Color::IdColor(col_pair.as_str().to_string());
                }
                _ => (),
            }
        }
        let r = data.remove(0);
        let g = data.remove(0);
        let b = data.remove(0);
        let a = data.remove(0);
        return Color::ParamColor(ParamColor {data: [r,g,b,a]});
    }

    fn parse_vector2(pair: Pair<Rule>) -> ParamVector2
    {
        let mut data: Vec<ScalarExpression> = vec![];
        for vec_pair in pair.into_inner()
        {
            match vec_pair.as_rule() 
            {
                Rule::scalar => 
                {
                    data.push(ScalarExpression::new(PibaldParser::parse_scalar_expr(vec_pair)));
                },
                _ => (),
            }
        }
        let x = data.remove(0);
        let y = data.remove(0);
        return ParamVector2{data: [x,y]};
    }

    fn parse_value_map(pair: Pair<Rule>) -> Vec<SDFTerm>
    {
        let mut sdf_stack: Vec<SDFTerm> = vec![];
        let mut expr_pairs = pair.into_inner();
        let expr_det_pair = expr_pairs.next().unwrap();
        match expr_pairs.next().unwrap().as_rule()
        {
            Rule::operator => 
            {
                let mut operator_pairs = expr_det_pair.into_inner();
                let op_pair = operator_pairs.next().unwrap().as_rule();
                let mut operand_count = 0;
                for operand_pair in operator_pairs
                {
                    match operand_pair.as_rule()
                    {
                        Rule::val_map =>
                        {
                            sdf_stack.append(&mut PibaldParser::parse_value_map(operand_pair));
                            operand_count +=1;
                        },
                        _=> (),
                    }
                }
                sdf_stack.push
                (
                    match op_pair 
                    {
                        Rule::OP_MIN => SDFTerm::Operator(SDFOperator::Minimum(operand_count)),
                        Rule::OP_AVG => SDFTerm::Operator(SDFOperator::Average(operand_count)),
                        Rule::OP_MASK => SDFTerm::Operator(SDFOperator::Mask),
                        _ => SDFTerm::Operator(SDFOperator::Minimum(operand_count)),
                    }
                );
            },
            //leaf expressions
            Rule::SD_CIRCLE_CYLINDER =>
            {
                expr_pairs.next();//L_PAREN
                let mat = PibaldParser::parse_matrix(expr_pairs.next().unwrap());
                expr_pairs.next();//COMMA
                let radius = ScalarExpression::new(PibaldParser::parse_scalar_expr(expr_pairs.next().unwrap()));
                sdf_stack.push( SDFTerm::Operand( SDFOperand::Circle( mat, radius ) ) );
                
            },
            Rule::SD_BOX_CYLINDER => 
            {
                expr_pairs.next();//L_PAREN
                let mat = PibaldParser::parse_matrix(expr_pairs.next().unwrap());
                expr_pairs.next();//COMMA
                let height = ScalarExpression::new(PibaldParser::parse_scalar_expr(expr_pairs.next().unwrap()));
                expr_pairs.next();//COMMA
                let width = ScalarExpression::new(PibaldParser::parse_scalar_expr(expr_pairs.next().unwrap()));
                sdf_stack.push( SDFTerm::Operand( SDFOperand::Rectangle( mat, width, height ) ) );
            },
            Rule::SD_PLANE => 
            {
                expr_pairs.next();//L_PAREN
                let mat = PibaldParser::parse_matrix(expr_pairs.next().unwrap());
                sdf_stack.push( SDFTerm::Operand( SDFOperand::Plane( mat, ) ) );
            },
            Rule::SD_POLYGON =>
            {
                expr_pairs.next();//L_PAREN
                let mat = PibaldParser::parse_matrix(expr_pairs.next().unwrap());
                let poly_points: Vec<ParamVector2> = vec![];
                for vec_pair in expr_pairs
                {
                    match vec_pair.as_rule()
                    {
                        Rule::vec2 => 
                        {
                            PibaldParser::parse_vector2(vec_pair);
                        },
                        _ => ()
                    }
                }
                sdf_stack.push(SDFTerm::Operand(SDFOperand::Polygon(mat, poly_points)));
            },
            Rule::SD_REG_POLYGON =>
            {
                expr_pairs.next();//L_PAREN
                let mat = PibaldParser::parse_matrix(expr_pairs.next().unwrap());
                expr_pairs.next();//COMMA
                let radius = ScalarExpression::new(PibaldParser::parse_scalar_expr(expr_pairs.next().unwrap()));
                sdf_stack.push( SDFTerm::Operand( SDFOperand::RegularPolygon( mat, radius ) ) );
                
            },
            Rule::SD_POLYSTAR => 
            {
                expr_pairs.next();//L_PAREN
                let mat = PibaldParser::parse_matrix(expr_pairs.next().unwrap());
                expr_pairs.next();//COMMA
                let outer_radius = ScalarExpression::new(PibaldParser::parse_scalar_expr(expr_pairs.next().unwrap()));
                expr_pairs.next();//COMMA
                let inner_radius = ScalarExpression::new(PibaldParser::parse_scalar_expr(expr_pairs.next().unwrap()));
                sdf_stack.push( SDFTerm::Operand( SDFOperand::PolyStar( mat, outer_radius, inner_radius ) ) );
            },
            _ => ()
        }
        return sdf_stack;
    }
}

pub struct ShapeShaderClass
{
    color_maps : Vec<ColorMap>,
    placements : Vec<Placement>,
}

impl ShapeShaderClass
{
    fn create_test()->Self
    {
        return ShapeShaderClass 
        { 
            color_maps: vec!
            [
                ColorMap
                { 
                    variant: ColorMapVariant::Binary
                    (
                        BinaryColorMap 
                        { 
                            color: Color::ParamColor(ParamColor::from_constant([1.0, 0.0, 0.0, 1.0]))
                        }
                    ), 
                    sdf_stack: vec!
                    [
                        SDFTerm::Operand(SDFOperand::Circle(Matrix::ParamMatrix(ParamMatrix::identity()), ScalarExpression::from_constant(1.0)))
                    ]
                }
            ], 
            placements: vec!
            [
                Placement
                { 
                    index: 0, 
                    tf: Matrix::ParamMatrix
                    (
                        ParamMatrix 
                        { 
                            location: Vector::ParamVector(ParamVector::from_constant([0.0, 0.5, 0.0])),
                            rotation: Vector::ParamVector(ParamVector::from_constant([0.0, 0.0, 0.0])),
                            scale: Vector::ParamVector(ParamVector::from_constant([1.0, 1.0, 1.0])) ,
                            shear: Vector::ParamVector(ParamVector::from_constant([0.0, 0.0, 0.0])) 
                        }
                    ), 
                    variant: PlacementVariant::Singular() 
                }
            ] 
        }
    }
}

struct ColorMap
{
    variant : ColorMapVariant,
    sdf_stack : Vec<SDFTerm>
}

enum ColorMapVariant
{
    Gradient(GradientColorMap),
    Binary(BinaryColorMap),
}

struct BinaryColorMap
{
    color : Color,
}

enum Matrix
{
    ParamMatrix(ParamMatrix),
    IdMatrix(String),
}

struct ParamMatrix
{
    location : Vector,
    rotation : Vector,
    scale : Vector,
    shear : Vector,
}

impl ParamMatrix
{
    fn identity()->Self
    {
        return ParamMatrix 
        { 
            location: Vector::ParamVector(ParamVector::from_constant([0.0, 0.0, 0.0])), 
            rotation:  Vector::ParamVector(ParamVector::from_constant([0.0, 0.0, 0.0])), 
            scale:  Vector::ParamVector(ParamVector::from_constant([1.0, 1.0, 1.0])),
            shear:  Vector::ParamVector(ParamVector::from_constant([0.0, 0.0, 0.0])),
        };
    }
}

enum Vector
{
    ParamVector(ParamVector),
    IdVector(String)
}

struct ParamVector
{
    data : [ScalarExpression;3],
}

impl ParamVector
{
    fn from_constant(data : [f32;3]) -> Self
    {
        return ParamVector 
        { 
            data: 
            [
                ScalarExpression::from_constant(data[0]),
                ScalarExpression::from_constant(data[1]),
                ScalarExpression::from_constant(data[2]),
            ] 
        }
    }
}

enum Color
{
    ParamColor(ParamColor),
    IdColor(String),
}

struct ParamColor
{
    data : [ScalarExpression;4],
}

impl ParamColor
{
    fn from_constant(data : [f32;4]) -> Self
    {
        return ParamColor 
        { 
            data: 
            [
                ScalarExpression::from_constant(data[0]),
                ScalarExpression::from_constant(data[1]),
                ScalarExpression::from_constant(data[2]), 
                ScalarExpression::from_constant(data[3]),
            ]
        }
    }
}

struct Placement
{
    index : u32,
    tf : Matrix,
    variant : PlacementVariant,
}

enum PlacementVariant
{
    Singular(),
    TilePattern(Vector),
}

struct ParamVector2
{
    data : [ScalarExpression;2],
}

struct GradientColorMap
{
    inner_grad: ColorGradient,
    outer_grad: Option<ColorGradient>,
}

struct ColorGradient
{
    extrapolation : GradientExtrapolation,
    color_points : Vec<ColorPoint>,
    max_distance : ScalarExpression
}

struct ColorPoint
{
    val : ScalarExpression,
    color : Color,
    interpolation_mode : GradientInterpolation
}

enum GradientInterpolation
{
    Linear(),
    Step(),
}

enum GradientExtrapolation
{
    LastColor(),
    Repeat(),
    RepeatReflect(),
}

enum SDFTerm
{
    Operator(SDFOperator),
    Operand(SDFOperand),
}

enum SDFOperator
{
    TilePattern,
    WavePattern,
    Mask,
    Minimum(i32),
    Average(i32),
}

enum SDFOperand
{
    Circle(Matrix, ScalarExpression),
    Rectangle(Matrix, ScalarExpression, ScalarExpression),
    Sphere(Matrix, ScalarExpression),
    Plane(Matrix),
    Polygon(Matrix, Vec<ParamVector2>),
    RegularPolygon(Matrix, ScalarExpression),
    PolyStar(Matrix, ScalarExpression, ScalarExpression),
}

struct ScalarExpression
{
    expr : Vec<ScalarTerm>
}

impl ScalarExpression
{
    fn new(mut expr : Vec<ScalarTerm>) -> Self
    {
        let mut i: usize = 0;
        while i < expr.len()
        {
            let term = expr[i].clone();
            let op = match term 
            {
                ScalarTerm::Value(_) => None,
                ScalarTerm::Operator(op) => Some(op.clone()),
            };
            
            if op.is_some() && i >=1 
            {
                let prev = expr[i-1].clone();
                let first = match prev
                {
                    ScalarTerm::Value(ScalarOperand::Constant(val)) => val,
                    _ => 0.0,
                };
                if matches!(prev, ScalarTerm::Value(ScalarOperand::Constant(..))) && i >= 2
                {
                    match op.unwrap()
                    {
                        ScalarOperator::Add | 
                        ScalarOperator::Subtract | 
                        ScalarOperator::Multiply | 
                        ScalarOperator::Divide |
                        ScalarOperator::Modulo |
                        ScalarOperator::Exponent => 
                        {
                            let prev_prev = expr[i-2].clone();
                            if matches!(prev_prev, ScalarTerm::Value(ScalarOperand::Constant(..)))
                            {
                                let second = match prev_prev
                                { 
                                    ScalarTerm::Value(ScalarOperand::Constant(val)) => val,
                                    _ => 0.0,
                                };
                                expr.remove((i-2).try_into().unwrap());
                                expr.remove((i-2).try_into().unwrap());
                                expr.remove((i-2).try_into().unwrap());

                                let res = match op.unwrap()
                                {
                                    ScalarOperator::Add => second + first,
                                    ScalarOperator::Subtract => second - first,
                                    ScalarOperator::Multiply =>  second * first,
                                    ScalarOperator::Divide => second / first,
                                    ScalarOperator::Modulo => second % first,
                                    ScalarOperator::Exponent => second.powf(first),
                                    _ => first,
                                };
                                expr.insert((i-2).try_into().unwrap(), ScalarTerm::Value(ScalarOperand::Constant(res)));
                            }
                        },
                        ScalarOperator::Negate|
                        ScalarOperator::Sine|
                        ScalarOperator::Cosine|
                        ScalarOperator::Tangent |
                        ScalarOperator::Log =>
                        {
                            expr.remove((i-1).try_into().unwrap());
                            expr.remove((i-1).try_into().unwrap());
                            let res = match op.unwrap()
                            {
                                ScalarOperator::Negate => -first,
                                ScalarOperator::Sine => first.sin(),
                                ScalarOperator::Cosine => first.cos(),
                                ScalarOperator::Tangent => first.tan(),
                                ScalarOperator::Log => first.ln(),
                                _ => first,
                            };
                            expr.insert((i-1).try_into().unwrap(), ScalarTerm::Value(ScalarOperand::Constant(res)));
                        }
                    }
                }
            }
            i += 1;
        }
        return ScalarExpression{expr};
    }

    fn from_constant(constant : f32) -> Self
    {
        return ScalarExpression { expr: vec![ScalarTerm::Value(ScalarOperand::Constant(constant))] };
    }

    fn evaluate(&self, args : &HashMap<&str, f32>) -> Result<f32, PibaldError>
    {
        //fast result if this doesn't use any variables
        if self.expr.len() == 1
        {
            match self.expr.get(0).unwrap() 
            {
                ScalarTerm::Value(val) => 
                {
                    match val 
                    {
                        ScalarOperand::Constant(con) => return Result::Ok(con.to_owned()),
                        ScalarOperand::Variable(param) => 
                        {
                            match args.get(param.as_str())
                            {
                                Some(arg) => return Result::Ok(arg.to_owned()),
                                None => return Err(PibaldError::MissingArgumentError(param.clone())),
                            }
                        },
                    }
                },
                ScalarTerm::Operator(_) => return Err(PibaldError::InvalidExpressionError(("Missing operands".to_string()))),
            }
        }
        let mut calc_stack: Vec<f32> = vec![];
        let stack = &self.expr;
        for term in stack
        {
            match term 
            {
                ScalarTerm::Value(val) => 
                {
                    match val 
                    {
                        ScalarOperand::Constant(con) => calc_stack.push(con.to_owned()),
                        ScalarOperand::Variable(param) => 
                        {
                            match args.get(param.as_str())
                            {
                                Some(arg) => calc_stack.push(arg.to_owned()),
                                None => return Err(PibaldError::MissingArgumentError(param.clone())),
                            }
                        },
                    }
                },
                ScalarTerm::Operator(op) => 
                {
                    match op 
                    {
                        ScalarOperator::Add => 
                        {
                            let eval = calc_stack.pop().unwrap() + calc_stack.pop().unwrap();
                            calc_stack.push(eval);
                        },
                        ScalarOperator::Subtract => 
                        {
                            let arg1 = calc_stack.pop().unwrap();
                            let arg0 = calc_stack.pop().unwrap();
                            calc_stack.push(arg0 - arg1);
                        },
                        ScalarOperator::Multiply => 
                        {
                            let eval = calc_stack.pop().unwrap() * calc_stack.pop().unwrap();
                            calc_stack.push(eval);
                        },
                        ScalarOperator::Divide => 
                        {
                            let arg1 = calc_stack.pop().unwrap();
                            let arg0 = calc_stack.pop().unwrap();
                            if(arg1 == 0.0)
                            {
                                return Err(PibaldError::DivideByZeroError)
                            }
                            calc_stack.push(arg0 / arg1);
                        },
                        ScalarOperator::Modulo =>
                        {
                            let arg1 = calc_stack.pop().unwrap();
                            let arg0 = calc_stack.pop().unwrap();
                            calc_stack.push(arg0 % arg1);
                        },
                        ScalarOperator::Exponent => 
                        {
                            let arg1 = calc_stack.pop().unwrap();
                            let arg0 = calc_stack.pop().unwrap();
                            calc_stack.push(arg0.powf(arg1));
                        },
                        ScalarOperator::Negate => 
                        {
                            let eval = -calc_stack.pop().unwrap();
                            calc_stack.push(eval);
                        },
                        ScalarOperator::Sine => 
                        {
                            let eval = calc_stack.pop().unwrap().sin();
                            calc_stack.push(eval);
                        },
                        ScalarOperator::Cosine => 
                        {
                            let eval = calc_stack.pop().unwrap().cos();
                            calc_stack.push(eval);
                        },
                        ScalarOperator::Tangent => 
                        {
                            let eval = calc_stack.pop().unwrap().tan();
                            calc_stack.push(eval);
                        },
                        ScalarOperator::Log => 
                        {
                            let eval = calc_stack.pop().unwrap().ln();
                            calc_stack.push(eval);
                        },
                    }
                },
            }
        }
        return Ok(calc_stack.pop().unwrap());
    }
}

#[derive(Clone)]
enum ScalarTerm
{
    Value(ScalarOperand),
    Operator(ScalarOperator),
}

#[derive(Clone)]
enum ScalarOperand
{
    Constant(f32),
    Variable(String),
}

#[derive(Clone, Copy)]
enum ScalarOperator
{
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    Exponent,
    Negate,
    Sine,
    Cosine,
    Tangent,
    Log,
}

struct PibaldEvaluator;

impl PibaldEvaluator
{
    
}