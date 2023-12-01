use std::collections::HashMap;
use std::fmt;

use glam::{Vec2, Mat4, Mat3, Vec4, Vec3, Quat};

#[derive(Debug)]
enum EvaluationError
{
    InvalidIdentifier{id : String},
    DivideByZeroError,
    TypeMismatchError{op: Operator},
    ValueUnderflowError{op: Operator},
    DegenerateMatrixError,
    IndexOutOfBoundsError{op: Operator, index : usize},
    InvalidOutputSize{op: Operator, size: f32}
}

impl std::error::Error for EvaluationError {}

impl fmt::Display for EvaluationError
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result 
    {
        return match self 
        {
            EvaluationError::InvalidIdentifier{id} => write!(f, "No value found for identifier \"{}.\"", id),
            EvaluationError::DivideByZeroError => write!(f, "Division by zero"),
            EvaluationError::TypeMismatchError { op, } => write!(f, "Type mismatch at operation \"{}.\"", op,),
            EvaluationError::ValueUnderflowError { op, } => write!(f, "Not enough values to perform operation \"{}.\"", op),
            EvaluationError::DegenerateMatrixError => write!(f, "Given matrix could not be inverted."),
            EvaluationError::IndexOutOfBoundsError { op, index } => write!(f, "Index {} was out of bounds for operation \"{}\"", index, op),
            EvaluationError::InvalidOutputSize { op, size } => write!(f, "Could create object with given output size {}", size),
        };
    }
}

#[derive(Debug)]
pub enum AssignmentError
{
    TypeMismatchError{ property_name: String, expected: Value, given: Value, },
    NoSuchPropertyError{ property_name: String, },
    NoSuchPropertyGroupError{ container_name: String, },
}

impl std::error::Error for AssignmentError {}

impl fmt::Display for AssignmentError
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result 
    {
        return match self 
        {
            AssignmentError::TypeMismatchError { property_name, expected, given } => 
            {
                write!(f, "Attempted to assign a value of type {} to property \"{},\" which has a type of {}.", given.type_name(), property_name, expected.type_name())
            },
            AssignmentError::NoSuchPropertyError { property_name } => write!(f, "No property with name \"{}\" exists for this context.", property_name),
            AssignmentError::NoSuchPropertyGroupError { container_name } => write!(f, "No property group with name \"{}\" exists for this context.", container_name),
        };
    }
}

#[derive(Clone)]
pub struct PropertyGroup
{
    values: HashMap<String, Value>
}

impl PropertyGroup
{
    pub fn set_property(&mut self, property_name: &str, in_value : Value) -> Result<(), AssignmentError>
    {
        let val_opt = self.values.get(property_name);
        return match val_opt 
        {
            Some(val) => 
            {
                if in_value.matches_type(val)
                {
                    self.values.insert(property_name.to_string(), in_value);
                    Result::Ok(())
                }
                else
                {
                    Result::Err(AssignmentError::TypeMismatchError { property_name: property_name.to_string(), expected: *val, given: in_value })
                }
            },
            None => Result::Err(AssignmentError::NoSuchPropertyError { property_name: property_name.to_string() }),
        };
    }
}

pub struct EvalTable
{
    expression_table: HashMap<u16, Value>
}

impl EvalTable
{
    pub fn new() -> Self
    {
        return EvalTable { expression_table: HashMap::new() };
    }

    pub fn update(&mut self, expr: &Expression, args: &PropertyGroup)
    {
        if let Result::Ok(val) = expr.evaluate(args)
        {
            self.expression_table.insert(expr.get_id(), val);
        }
    }

    pub fn get_entries(&self,) -> impl Iterator<Item=(&u16, &Value)>
    {
        return self.expression_table.iter();
    }

    pub fn get_value(&self, id: u16) -> Option<&Value>
    {
        return self.expression_table.get(&id);
    }
}

pub struct Expression
{
    id: u16, //if you need to evaluate more than 2^16 expressions in one shader then you fucked up
    terms : Vec<Term>
}

impl Expression
{
    pub fn get_id(&self) -> u16
    {
        return self.id;
    }

    pub fn evaluate(&self, context: &PropertyGroup) -> Result<Value, EvaluationError>
    {
        let mut val_stack: Vec<Value> = vec![];
        for term in &self.terms
        {
            match term 
            {
                Term::Operand(operand) => 
                {
                    match operand 
                    {
                        Operand::Literal(lit) => val_stack.push(*lit),
                        Operand::Variable(var_name) => 
                        {
                            let var = context.values.get(var_name);
                            match var 
                            {
                                Some(lit) => val_stack.push(*lit),
                                None => return Result::Err(EvaluationError::InvalidIdentifier { id: var_name.clone() }),
                            }
                        },
                    }
                },
                Term::Operator(op) => 
                {
                    let calc = op.evaluate(&mut val_stack);
                    match calc 
                    {
                        Ok(result) => val_stack.push(result),
                        Err(erm) => return Err(erm),
                    }
                },
            }
        }
        return match val_stack.pop() 
        {
            Some(val) => Result::Ok(val),
            None => Result::Err(EvaluationError::ValueUnderflowError { op: Operator::Entry }),
        }
    }
}

enum Term
{
    Operand(Operand),
    Operator(Operator),
}

#[derive(Clone, Copy, Debug)]
pub enum Value
{
    Scalar(f32),
    Vector2(Vec2),
    Vector3(Vec3),
    Matrix3(Mat3),
    Matrix4(Mat4),
    Color(Vec4),
    Quaternion(Quat),
}

impl Value
{
    fn val_into_float_list(&self) -> Vec<f32>
    {
        match self 
        {
            Value::Scalar(s_val) => return vec![s_val.clone()],
            Value::Vector2(v2_val) => return vec![v2_val.x, v2_val.y],
            Value::Vector3(v3_val) => return vec![v3_val.x, v3_val.y, v3_val.z],
            Value::Color(v4_val) => return vec![v4_val.x, v4_val.y, v4_val.z, v4_val.z],
            Value::Quaternion(q_val) => return vec![q_val.x, q_val.y, q_val.z, q_val.z],
            Value::Matrix3(m3_val) => return m3_val.to_cols_array().to_vec(),
            Value::Matrix4(m4_val) => return m4_val.to_cols_array().to_vec(),
        }
    }

    fn type_name(&self) -> &str
    {
        return match self 
        {
            Value::Scalar(_) => "Scalar",
            Value::Vector2(_) => "Vector2",
            Value::Vector3(_) => "Vector3",
            Value::Matrix3(_) => "Matrix3",
            Value::Matrix4(_) => "Matrix4",
            Value::Color(_) => "Color",
            Value::Quaternion(_) => "Quaternion",
        };
    }

    fn matches_type(&self, other: &Value) -> bool
    {
        return std::mem::discriminant(self) == std::mem::discriminant(other);
    }
}

enum Operand
{
    Literal(Value),
    Variable(String),
}

#[derive(Clone, Copy, Debug)]
enum Operator
{
    BinaryOperator(BinaryOperator),
    UnaryOperator(UnaryOperator),
    CreateVector2,
    CreateVector3,
    CreateMatrix3,
    CreateMatrix4,
    Row,
    Column,
    Entry,
    CreateQuaternion,
    CreateColor,
    Swizzle2,
    Swizzle3,
    Swizzle4,
}

impl std::fmt::Display for Operator
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result 
    {
        return match self 
        {
            Operator::BinaryOperator(bop) => 
            {
                match bop 
                {
                    BinaryOperator::Add => write!(f, "Add"),
                    BinaryOperator::Subtract => write!(f, "Subtract"),
                    BinaryOperator::Multiply => write!(f, "Multiply"),
                    BinaryOperator::Divide => write!(f, "Divide"),
                    BinaryOperator::Modulo => write!(f, "Modulo"),
                    BinaryOperator::Exponent => write!(f, "Exponent"),
                    BinaryOperator::Dot => write!(f, "Dot"),
                    BinaryOperator::Cross => write!(f, "Cross"),
                }
            },
            Operator::UnaryOperator(uop) => 
            {
                match uop 
                {
                    UnaryOperator::Negate => write!(f, "Negate"),
                    UnaryOperator::Sine => write!(f, "Sine"),
                    UnaryOperator::Cosine => write!(f, "Cosine"),
                    UnaryOperator::Tangent => write!(f, "Tangent"),
                    UnaryOperator::Log => write!(f, "Log"),
                    UnaryOperator::Normalize => write!(f, "Normalize"),
                    UnaryOperator::Inverse => write!(f, "Inverse"),
                    UnaryOperator::Transpose => write!(f, "Transpose"),
                }
            },
            Operator::CreateVector2 => write!(f, "CreateVector2"),
            Operator::CreateVector3 => write!(f, "CreateVector3"),
            Operator::CreateMatrix3 => write!(f, "CreateMatrix3"),
            Operator::CreateMatrix4 => write!(f, "CreateMatrix4"),
            Operator::Row => write!(f, "Row"),
            Operator::Column => write!(f, "Column"),
            Operator::Entry => write!(f, "Entry"),
            Operator::CreateQuaternion => write!(f, "CreateQuaternion"),
            Operator::CreateColor => write!(f, "CreateColor"),
            Operator::Swizzle2 => write!(f, "Swizzle2"),
            Operator::Swizzle3 => write!(f, "Swizzle3"),
            Operator::Swizzle4 => write!(f, "Swizzle4"),
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum UnaryOperator
{
    Negate,
    Sine,
    Cosine,
    Tangent,
    Log,
    Normalize,
    Inverse,
    Transpose,
}

#[derive(Clone, Copy, Debug)]
enum BinaryOperator
{
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    Exponent,
    Dot,
    Cross,
}

impl Operator
{
    //remember the value at the top of the stack is the one that was most recently evaluated, so arguments are in reverse order
    fn evaluate(&self, value_stack : &mut Vec<Value>) -> Result<Value, EvaluationError>
    {
        match self 
        {
            Operator::BinaryOperator(bop) => 
            {
                let rhs = value_stack.pop();
                let lhs = value_stack.pop();
                if rhs.is_none() || lhs.is_none()
                {
                    return Result::Err(EvaluationError::ValueUnderflowError{ op: Operator::BinaryOperator(*bop), });
                }
                let rhs_val = rhs.unwrap();
                let lhs_val = lhs.unwrap();
                match bop
                {
                    BinaryOperator::Add => 
                    {
                        match lhs_val
                        {
                            Value::Scalar(s_lhs) => 
                            {
                                match rhs_val 
                                {
                                    Value::Scalar(s_rhs) => return Result::Ok(Value::Scalar(s_lhs + s_rhs)),
                                    Value::Vector2(v2_rhs) => return Result::Ok(Value::Vector2(s_lhs + v2_rhs)),
                                    Value::Vector3(v3_rhs) => return Result::Ok(Value::Vector3(s_lhs + v3_rhs)),
                                    Value::Color(c_rhs) => return Result::Ok(Value::Color(s_lhs + c_rhs)),
                                    _ => return Result::Err(EvaluationError::TypeMismatchError{ op: Operator::BinaryOperator(*bop), }),
                                }
                            },
                            Value::Vector2(v2_lhs) => 
                            {
                                match rhs_val 
                                {
                                    Value::Scalar(s_rhs) => return Result::Ok(Value::Vector2(v2_lhs + s_rhs)),
                                    Value::Vector2(v2_rhs) => return Result::Ok(Value::Vector2(v2_lhs + v2_rhs)),
                                    _ => return Result::Err(EvaluationError::TypeMismatchError{ op: Operator::BinaryOperator(*bop), }),
                                }
                            },
                            Value::Vector3(v3_lhs) => 
                            {
                                match rhs_val 
                                {
                                    Value::Scalar(s_rhs) => return Result::Ok(Value::Vector3(v3_lhs + s_rhs)),
                                    Value::Vector3(v3_rhs) => return Result::Ok(Value::Vector3(v3_lhs + v3_rhs)),
                                    _ => return Result::Err(EvaluationError::TypeMismatchError{ op: Operator::BinaryOperator(*bop), }),
                                }
                            },
                            Value::Matrix3(m3_lhs) => 
                            {
                                match rhs_val 
                                {
                                    Value::Matrix3(m3_rhs) => return Result::Ok(Value::Matrix3(m3_lhs + m3_rhs)),
                                    _ => return Result::Err(EvaluationError::TypeMismatchError{ op: Operator::BinaryOperator(*bop), }),
                                }
                            },
                            Value::Matrix4(m4_lhs) => 
                            {
                                match rhs_val 
                                {
                                    Value::Matrix4(m4_rhs) => return Result::Ok(Value::Matrix4(m4_lhs + m4_rhs)),
                                    _ => return Result::Err(EvaluationError::TypeMismatchError{ op: Operator::BinaryOperator(*bop), }),
                                }
                            },
                            Value::Color(c_lhs) => 
                            {
                                match rhs_val 
                                {
                                    Value::Scalar(s_rhs) => return Result::Ok(Value::Color(c_lhs + s_rhs)),
                                    Value::Color(c_rhs) => return Result::Ok(Value::Color(c_lhs + c_rhs)),
                                    _ => return Result::Err(EvaluationError::TypeMismatchError{ op: Operator::BinaryOperator(*bop), }),
                                }
                            },
                            Value::Quaternion(q_lhs) => 
                            {
                                match rhs_val 
                                {
                                    Value::Quaternion(q_rhs) => return Result::Ok(Value::Quaternion(q_lhs + q_rhs)),
                                    _ => return Result::Err(EvaluationError::TypeMismatchError{ op: Operator::BinaryOperator(*bop), }),
                                }
                            },
                        }
                    },
                    BinaryOperator::Subtract => 
                    {
                        match lhs_val
                        {
                            Value::Scalar(s_lhs) => 
                            {
                                match rhs_val 
                                {
                                    Value::Scalar(s_rhs) => return Result::Ok(Value::Scalar(s_lhs - s_rhs)),
                                    _ => return Result::Err(EvaluationError::TypeMismatchError{ op: Operator::BinaryOperator(*bop), }),
                                }
                            },
                            Value::Vector2(v2_lhs) => 
                            {
                                match rhs_val 
                                {
                                    Value::Scalar(s_rhs) => return Result::Ok(Value::Vector2(v2_lhs - s_rhs)),
                                    Value::Vector2(v2_rhs) => return Result::Ok(Value::Vector2(v2_lhs - v2_rhs)),
                                    _ => return Result::Err(EvaluationError::TypeMismatchError{ op: Operator::BinaryOperator(*bop), }),
                                }
                            },
                            Value::Vector3(v3_lhs) => 
                            {
                                match rhs_val 
                                {
                                    Value::Scalar(s_rhs) => return Result::Ok(Value::Vector3(v3_lhs - s_rhs)),
                                    Value::Vector3(v3_rhs) => return Result::Ok(Value::Vector3(v3_lhs - v3_rhs)),
                                    _ => return Result::Err(EvaluationError::TypeMismatchError{ op: Operator::BinaryOperator(*bop), }),
                                }
                            },
                            Value::Matrix3(m3_lhs) => 
                            {
                                match rhs_val 
                                {
                                    Value::Matrix3(m3_rhs) => return Result::Ok(Value::Matrix3(m3_lhs - m3_rhs)),
                                    _ => return Result::Err(EvaluationError::TypeMismatchError{ op: Operator::BinaryOperator(*bop), }),
                                }
                            },
                            Value::Matrix4(m4_lhs) => 
                            {
                                match rhs_val 
                                {
                                    Value::Matrix4(m4_rhs) => return Result::Ok(Value::Matrix4(m4_lhs - m4_rhs)),
                                    _ => return Result::Err(EvaluationError::TypeMismatchError{ op: Operator::BinaryOperator(*bop), }),
                                }
                            },
                            Value::Color(c_lhs) => 
                            {
                                match rhs_val 
                                {
                                    Value::Scalar(s_rhs) => return Result::Ok(Value::Color(c_lhs - s_rhs)),
                                    Value::Color(c_rhs) => return Result::Ok(Value::Color(c_lhs - c_rhs)),
                                    _ => return Result::Err(EvaluationError::TypeMismatchError{ op: Operator::BinaryOperator(*bop), }),
                                }
                            },
                            Value::Quaternion(q_lhs) => 
                            {
                                match rhs_val 
                                {
                                    Value::Quaternion(q_rhs) => return Result::Ok(Value::Quaternion(q_lhs - q_rhs)),
                                    _ => return Result::Err(EvaluationError::TypeMismatchError{ op: Operator::BinaryOperator(*bop), }),
                                }
                            },
                        }
                    },
                    BinaryOperator::Multiply => 
                    {
                        match lhs_val
                        {
                            Value::Scalar(s_lhs) => 
                            {
                                match rhs_val 
                                {
                                    Value::Scalar(s_rhs) => return Result::Ok(Value::Scalar(s_lhs * s_rhs)),
                                    Value::Vector2(v2_rhs) => return Result::Ok(Value::Vector2(s_lhs * v2_rhs)),
                                    Value::Vector3(v3_rhs)  => return Result::Ok(Value::Vector3(s_lhs * v3_rhs)),
                                    Value::Matrix3(m3_rhs)  => return Result::Ok(Value::Matrix3(s_lhs * m3_rhs)),
                                    Value::Matrix4(m4_rhs)  => return Result::Ok(Value::Matrix4(s_lhs * m4_rhs)),
                                    Value::Color(c_rhs)  => return Result::Ok(Value::Color(s_lhs * c_rhs)),
                                    Value::Quaternion(q_rhs)  => return Result::Ok(Value::Quaternion(q_rhs * s_lhs)),
                                }
                            },
                            Value::Vector2(v2_lhs) => 
                            {
                                match rhs_val 
                                {
                                    Value::Scalar(s_rhs) => return Result::Ok(Value::Vector2(v2_lhs * s_rhs)),
                                    Value::Vector2(v2_rhs) => return Result::Ok(Value::Vector2(v2_lhs * v2_rhs)),
                                    _ => return Result::Err(EvaluationError::TypeMismatchError{ op: Operator::BinaryOperator(*bop), }),
                                }
                            },
                            Value::Vector3(v3_lhs) => 
                            {
                                match rhs_val 
                                {
                                    Value::Scalar(s_rhs) => return Result::Ok(Value::Vector3(v3_lhs * s_rhs)),
                                    Value::Vector3(v3_rhs) => return Result::Ok(Value::Vector3(v3_lhs * v3_rhs)),
                                    _ => return Result::Err(EvaluationError::TypeMismatchError{ op: Operator::BinaryOperator(*bop), }),
                                }
                            },
                            Value::Matrix3(m3_lhs) => 
                            {
                                match rhs_val 
                                {
                                    Value::Scalar(s_rhs) => return Result::Ok(Value::Matrix3(m3_lhs * s_rhs)),
                                    Value::Vector2(v2_rhs) => return Result::Ok(Value::Vector2(m3_lhs.transform_point2(v2_rhs))),
                                    Value::Matrix3(m3_rhs) => return Result::Ok(Value::Matrix3(m3_lhs * m3_rhs)),
                                    _ => return Result::Err(EvaluationError::TypeMismatchError{ op: Operator::BinaryOperator(*bop), }),
                                }
                            },
                            Value::Matrix4(m4_lhs) => 
                            {
                                match rhs_val 
                                {
                                    Value::Scalar(s_rhs) => return Result::Ok(Value::Matrix4(m4_lhs * s_rhs)),
                                    Value::Vector3(v3_rhs) => return Result::Ok(Value::Vector3(m4_lhs.transform_point3(v3_rhs))),
                                    Value::Matrix4(m4_rhs) => return Result::Ok(Value::Matrix4(m4_lhs * m4_rhs)),
                                    _ => return Result::Err(EvaluationError::TypeMismatchError{ op: Operator::BinaryOperator(*bop), }),
                                }
                            },
                            Value::Color(c_lhs) => 
                            {
                                match rhs_val 
                                {
                                    Value::Scalar(s_rhs) => return Result::Ok(Value::Color(c_lhs * s_rhs)),
                                    Value::Color(c_rhs) => return Result::Ok(Value::Color(c_lhs * c_rhs)),
                                    _ => return Result::Err(EvaluationError::TypeMismatchError{ op: Operator::BinaryOperator(*bop), }),
                                }
                            },
                            Value::Quaternion(q_lhs) => 
                            {
                                match rhs_val 
                                {
                                    Value::Quaternion(q_rhs) => return Result::Ok(Value::Quaternion(q_lhs * q_rhs)),
                                    _ => return Result::Err(EvaluationError::TypeMismatchError{ op: Operator::BinaryOperator(*bop), }),
                                }
                            },
                        }
                    },
                    BinaryOperator::Divide => 
                    {
                        //this is the op where things can go, uh, very baaad
                        match rhs_val
                        {
                            Value::Scalar(s_rhs) => if s_rhs == 0.0 { return Err(EvaluationError::DivideByZeroError) },
                            Value::Vector2(v2_rhs)  => if v2_rhs.to_array().iter().any(|v| *v == 0.0 ) { return Err(EvaluationError::DivideByZeroError) },
                            Value::Vector3(v3_rhs)  => if v3_rhs.to_array().iter().any(|v| *v == 0.0 ) { return Err(EvaluationError::DivideByZeroError) },
                            Value::Color(c_rhs) => if c_rhs.to_array().iter().any(|v| *v == 0.0 ) { return Err(EvaluationError::DivideByZeroError) },
                            _=> (),
                        }
                        match lhs_val
                        {
                            Value::Scalar(s_lhs) => 
                            {
                                match rhs_val 
                                {
                                    Value::Scalar(s_rhs) => return Result::Ok(Value::Scalar(s_lhs / s_rhs)),
                                    _ => return Result::Err(EvaluationError::TypeMismatchError{ op: Operator::BinaryOperator(*bop), }),
                                }
                            },
                            Value::Vector2(v2_lhs) => 
                            {
                                match rhs_val 
                                {
                                    Value::Scalar(s_rhs) => return Result::Ok(Value::Vector2(v2_lhs / s_rhs)),
                                    Value::Vector2(v2_rhs) => return Result::Ok(Value::Vector2(v2_lhs / v2_rhs)),
                                    _ => return Result::Err(EvaluationError::TypeMismatchError{ op: Operator::BinaryOperator(*bop), }),
                                }
                            },
                            Value::Vector3(v3_lhs) => 
                            {
                                match rhs_val 
                                {
                                    Value::Scalar(s_rhs) => return Result::Ok(Value::Vector3(v3_lhs / s_rhs)),
                                    Value::Vector3(v3_rhs) => return Result::Ok(Value::Vector3(v3_lhs / v3_rhs)),
                                    _ => return Result::Err(EvaluationError::TypeMismatchError{ op: Operator::BinaryOperator(*bop), }),
                                }
                            },
                            Value::Color(c_lhs) => 
                            {
                                match rhs_val 
                                {
                                    Value::Scalar(s_rhs) => return Result::Ok(Value::Color(c_lhs / s_rhs)),
                                    Value::Color(c_rhs) => return Result::Ok(Value::Color(c_lhs / c_rhs)),
                                    _ => return Result::Err(EvaluationError::TypeMismatchError{ op: Operator::BinaryOperator(*bop), }),
                                }
                            },
                            _ => return Result::Err(EvaluationError::TypeMismatchError{ op: Operator::BinaryOperator(*bop), }),
                        }
                    },
                    BinaryOperator::Modulo => 
                    {
                        //this is the op where things can go, uh, very baaad
                        match rhs_val
                        {
                            Value::Scalar(s_rhs) => if s_rhs == 0.0 { return Err(EvaluationError::DivideByZeroError) },
                            Value::Vector2(v2_rhs)  => if v2_rhs.to_array().iter().any(|v| *v == 0.0 ) { return Err(EvaluationError::DivideByZeroError) },
                            Value::Vector3(v3_rhs)  => if v3_rhs.to_array().iter().any(|v| *v == 0.0 ) { return Err(EvaluationError::DivideByZeroError) },
                            Value::Color(c_rhs) => if c_rhs.to_array().iter().any(|v| *v == 0.0 ) { return Err(EvaluationError::DivideByZeroError) },
                            _=> (),
                        }
                        match lhs_val
                        {
                            Value::Scalar(s_lhs) => 
                            {
                                match rhs_val 
                                {
                                    Value::Scalar(s_rhs) => return Result::Ok(Value::Scalar(s_lhs % s_rhs)),
                                    _ => return Result::Err(EvaluationError::TypeMismatchError{ op: Operator::BinaryOperator(*bop), }),
                                }
                            },
                            Value::Vector2(v2_lhs) => 
                            {
                                match rhs_val 
                                {
                                    Value::Scalar(s_rhs) => return Result::Ok(Value::Vector2(v2_lhs % s_rhs)),
                                    Value::Vector2(v2_rhs) => return Result::Ok(Value::Vector2(v2_lhs % v2_rhs)),
                                    _ => return Result::Err(EvaluationError::TypeMismatchError{ op: Operator::BinaryOperator(*bop), }),
                                }
                            },
                            Value::Vector3(v3_lhs) => 
                            {
                                match rhs_val 
                                {
                                    Value::Scalar(s_rhs) => return Result::Ok(Value::Vector3(v3_lhs % s_rhs)),
                                    Value::Vector3(v3_rhs) => return Result::Ok(Value::Vector3(v3_lhs % v3_rhs)),
                                    _ => return Result::Err(EvaluationError::TypeMismatchError{ op: Operator::BinaryOperator(*bop), }),
                                }
                            },
                            Value::Color(c_lhs) => 
                            {
                                match rhs_val 
                                {
                                    Value::Scalar(s_rhs) => return Result::Ok(Value::Color(c_lhs % s_rhs)),
                                    Value::Color(c_rhs) => return Result::Ok(Value::Color(c_lhs % c_rhs)),
                                    _ => return Result::Err(EvaluationError::TypeMismatchError{ op: Operator::BinaryOperator(*bop), }),
                                }
                            },
                            _ => return Result::Err(EvaluationError::TypeMismatchError{ op: Operator::BinaryOperator(*bop), }),
                        }
                    },
                    BinaryOperator::Exponent => 
                    {
                        match rhs_val
                        {
                            Value::Scalar(s_rhs) => 
                            {
                                match lhs_val 
                                {
                                    Value::Scalar(s_lhs) => return Result::Ok(Value::Scalar(s_lhs.powf(s_rhs))),
                                    Value::Vector2(v2_lhs) => return Result::Ok(Value::Vector2(v2_lhs.powf(s_rhs))),
                                    Value::Vector3(v3_lhs) => return Result::Ok(Value::Vector3(v3_lhs.powf(s_rhs))),
                                    Value::Color(c_lhs) => return Result::Ok(Value::Color(c_lhs.powf(s_rhs))),
                                    _ => return Result::Err(EvaluationError::TypeMismatchError{ op: Operator::BinaryOperator(*bop), }),
                                }
                            },
                            _ => return Result::Err(EvaluationError::TypeMismatchError{ op: Operator::BinaryOperator(*bop), }),
                        }
                    },
                    BinaryOperator::Dot => 
                    {
                        match lhs_val 
                        {
                            Value::Vector2(v2_lhs) => 
                            {
                                match rhs_val
                                {
                                    Value::Vector2(v2_rhs) => return Result::Ok(Value::Scalar(v2_lhs.dot(v2_rhs))),
                                    _ => return Result::Err(EvaluationError::TypeMismatchError { op: Operator::BinaryOperator(*bop), })
                                }
                            },
                            Value::Vector3(v3_lhs) => 
                            {
                                match rhs_val
                                {
                                    Value::Vector3(v3_rhs) => return Result::Ok(Value::Scalar(v3_lhs.dot(v3_rhs))),
                                    _ => return Result::Err(EvaluationError::TypeMismatchError { op: Operator::BinaryOperator(*bop), })
                                }
                            },
                            _ => return Result::Err(EvaluationError::TypeMismatchError { op: Operator::BinaryOperator(*bop), })
                        }
                    },
                    BinaryOperator::Cross => 
                    {
                        match lhs_val 
                        {
                            Value::Vector3(v3_lhs) => 
                            {
                                match rhs_val
                                {
                                    Value::Vector3(v3_rhs) => return Result::Ok(Value::Vector3(v3_lhs.cross(v3_rhs))),
                                    _ => return Result::Err(EvaluationError::TypeMismatchError { op: Operator::BinaryOperator(*bop), })
                                }
                            },
                            _ => return Result::Err(EvaluationError::TypeMismatchError { op: Operator::BinaryOperator(*bop), })
                        }
                    },
                }
            },
            Operator::UnaryOperator(uop) =>
            {
                let val_opt = value_stack.pop();
                if val_opt.is_none()
                {
                    return Result::Err(EvaluationError::ValueUnderflowError{ op: Operator::UnaryOperator(*uop), });
                }
                let val = val_opt.unwrap();
                match uop
                {
                    UnaryOperator::Negate => 
                    {
                        match val
                        {
                            Value::Scalar(s_val) => return Result::Ok(Value::Scalar(-s_val)),
                            Value::Vector2(v2_val) => return Result::Ok(Value::Vector2(-v2_val)),
                            Value::Vector3(v3_val) => return Result::Ok(Value::Vector3(-v3_val)),
                            Value::Matrix3(m3_val) => return Result::Ok(Value::Matrix3(-m3_val)),
                            Value::Matrix4(m4_val) => return Result::Ok(Value::Matrix4(-m4_val)),
                            Value::Color(c_val) => return Result::Ok(Value::Color(-c_val)),
                            Value::Quaternion(q_val) => return Result::Ok(Value::Quaternion(-q_val)),
                        }
                    },
                    UnaryOperator::Sine => 
                    {
                        match val
                        {
                            Value::Scalar(s_val) => return Result::Ok(Value::Scalar(s_val.sin())),
                            _ => return Result::Err(EvaluationError::TypeMismatchError { op: Operator::UnaryOperator(*uop), }),
                        }
                    },
                    UnaryOperator::Cosine => 
                    {
                        match val
                        {
                            Value::Scalar(s_val) => return Result::Ok(Value::Scalar(s_val.cos())),
                            _ => return Result::Err(EvaluationError::TypeMismatchError { op: Operator::UnaryOperator(*uop), }),
                        }
                    },
                    UnaryOperator::Tangent => 
                    {
                        match val
                        {
                            Value::Scalar(s_val) => return Result::Ok(Value::Scalar(s_val.tan())),
                            _ => return Result::Err(EvaluationError::TypeMismatchError { op: Operator::UnaryOperator(*uop), }),
                        }
                    },
                    UnaryOperator::Log => 
                    {
                        match val
                        {
                            Value::Scalar(s_val) => return Result::Ok(Value::Scalar(s_val.ln())),
                            _ => return Result::Err(EvaluationError::TypeMismatchError { op: Operator::UnaryOperator(*uop), }),
                        }
                    },
                    UnaryOperator::Normalize => 
                    {
                        match val
                        {
                            Value::Vector2(v2_val) => return Result::Ok(Value::Vector2(v2_val.normalize())),
                            Value::Vector3(v3_val) => return Result::Ok(Value::Vector3(v3_val.normalize())),
                            _ => Result::Err(EvaluationError::TypeMismatchError { op: Operator::UnaryOperator(*uop), }),
                        }
                    },
                    UnaryOperator::Inverse => 
                    {
                        match val 
                        {
                            Value::Matrix3(m3_val) => 
                            {
                                if m3_val.determinant() == 0.0
                                {
                                    return Result::Err(EvaluationError::DegenerateMatrixError);
                                }
                                return Result::Ok(Value::Matrix3(m3_val.inverse()));
                            },
                            Value::Matrix4(m4_val) => 
                            {
                                if m4_val.determinant() == 0.0
                                {
                                    return Result::Err(EvaluationError::DegenerateMatrixError);
                                }
                                return Result::Ok(Value::Matrix4(m4_val.inverse()));
                            },
                            Value::Color(c_val) => 
                            {
                                return Result::Ok
                                (
                                    Value::Color
                                    (
                                        Vec4::new
                                        (
                                            1.0 - c_val.x.clamp(0.0, 1.0),
                                            1.0 - c_val.y.clamp(0.0, 1.0),
                                            1.0 - c_val.z.clamp(0.0, 1.0),
                                            1.0 - c_val.w.clamp(0.0, 1.0),
                                        )
                                    )
                                );
                            },
                            Value::Quaternion(q_val) => return Result::Ok(Value::Quaternion(q_val.inverse())),
                            _ => return Result::Err(EvaluationError::TypeMismatchError { op: Operator::UnaryOperator(*uop), })
                        }
                    },
                    UnaryOperator::Transpose => 
                    {
                        match val
                        {
                            Value::Matrix3(m3_val) => return Result::Ok(Value::Matrix3(m3_val.transpose())),
                            Value::Matrix4(m4_val) => return Result::Ok(Value::Matrix4(m4_val.transpose())),
                            _ => return Result::Err(EvaluationError::TypeMismatchError { op: Operator::UnaryOperator(*uop), }),
                        }
                    },
                }
            },
            Operator::CreateVector2 | Operator::CreateVector3 | Operator::CreateColor => 
            {
                let output_len: usize = match self 
                {
                    Operator::CreateVector2 => 2,
                    Operator::CreateVector3 => 3,
                    Operator::CreateColor => 4,
                    _ => 0, //unreachable
                };
                let scalars = 
                {
                    let mut ret : Vec<f32> = vec![];
                    while ret.len() < output_len
                    {
                        let val = value_stack.pop();
                        if val.is_none()
                        {
                            return Result::Err(EvaluationError::ValueUnderflowError { op: *self });
                        }
                        ret.append(&mut val.unwrap().val_into_float_list());
                        if ret.len() > output_len
                        {
                            return Result::Err(EvaluationError::TypeMismatchError { op: *self });
                        }
                    }
                    ret
                };
                match self 
                {
                    Operator::CreateVector2 => return Result::Ok(Value::Vector2(Vec2::new(scalars[1], scalars[0]))),
                    Operator::CreateVector3 => return Result::Ok(Value::Vector3(Vec3::new(scalars[2], scalars[1], scalars[0]))),
                    Operator::CreateColor => return Result::Ok(Value::Color(Vec4::new(scalars[3], scalars[2], scalars[1], scalars[0]))),
                    _ => return Result::Err(EvaluationError::TypeMismatchError { op: *self }), //unreachable
                };
            },
            Operator::CreateMatrix3 => 
            {
                let scale = value_stack.pop();
                let rot = value_stack.pop();
                let loc = value_stack.pop();
                if scale.is_none() || rot.is_none() || loc.is_none() 
                {
                    return Result::Err(EvaluationError::ValueUnderflowError{ op: *self, });
                }
                let scale_val = match scale.unwrap()
                {
                    Value::Vector2(val) => val,
                    _ => return Result::Err(EvaluationError::TypeMismatchError {  op: *self, })
                };
                let rot_val = match rot.unwrap()
                {
                    Value::Scalar(val) => val,
                    _ => return Result::Err(EvaluationError::TypeMismatchError { op: *self, })
                };
                let loc_val = match loc.unwrap()
                {
                    Value::Vector2(val) => val,
                    _ => return Result::Err(EvaluationError::TypeMismatchError { op: *self, })
                };
                return Result::Ok(Value::Matrix3(Mat3::from_scale_angle_translation(scale_val, rot_val, loc_val)));
            },
            Operator::CreateMatrix4 => 
            {
                let scale = value_stack.pop();
                let rot = value_stack.pop();
                let loc = value_stack.pop();
                if scale.is_none() || rot.is_none() || loc.is_none() 
                {
                    return Result::Err(EvaluationError::ValueUnderflowError{ op: *self, });
                }
                let scale_val = match scale.unwrap()
                {
                    Value::Vector3(val) => val,
                    _ => return Result::Err(EvaluationError::TypeMismatchError { op: *self, })
                };
                let rot_val = match rot.unwrap()
                {
                    Value::Quaternion(val) => val,
                    _ => return Result::Err(EvaluationError::TypeMismatchError { op: *self, })
                };
                let loc_val = match loc.unwrap()
                {
                    Value::Vector3(val) => val,
                    _ => return Result::Err(EvaluationError::TypeMismatchError { op: *self, })
                };
                return Result::Ok(Value::Matrix4(Mat4::from_scale_rotation_translation(scale_val, rot_val, loc_val)));
            },
            Operator::Row | Operator::Column => 
            {
                let index = value_stack.pop();
                let data = value_stack.pop();
                if data.is_none() || index.is_none()
                {
                    return Result::Err(EvaluationError::ValueUnderflowError { op: *self });
                }
                if let Value::Scalar(val) = index.unwrap()
                {
                    let index_val = val.round() as usize;
                    match data.unwrap()
                {
                    Value::Matrix3(m3_data) => 
                    {
                        if index_val > 1
                        {
                            return Result::Err(EvaluationError::IndexOutOfBoundsError { op: *self, index: index_val });
                        }
                        let vec = if matches!(self, Operator::Row) { m3_data.row(index_val) } else { m3_data.col(index_val) };
                        return Result::Ok(Value::Vector2(vec.truncate()));
                    },
                    Value::Matrix4(m4_data) => 
                    {
                        if index_val > 2
                        {
                            return Result::Err(EvaluationError::IndexOutOfBoundsError { op: *self, index: index_val });
                        }
                        let vec = if matches!(self, Operator::Row) { m4_data.row(index_val) } else { m4_data.col(index_val) };
                        return Result::Ok(Value::Vector3(vec.truncate()));
                    },
                    _ => return Result::Err(EvaluationError::TypeMismatchError { op: *self }),
                }
                }
                else
                {
                    return Result::Err(EvaluationError::TypeMismatchError { op: *self });
                }
                
            },
            Operator::Entry => 
            {
                let index = value_stack.pop();
                let data = value_stack.pop();
                if data.is_none() || index.is_none()
                {
                    return Result::Err(EvaluationError::ValueUnderflowError { op: *self });
                }
                if let Value::Scalar(val) = index.unwrap() 
                {
                    let index_val = val.round() as usize;
                    match data.unwrap()
                    {
                        Value::Vector2(v2_data) => 
                        {
                            if index_val > 1
                            {
                                return Result::Err(EvaluationError::IndexOutOfBoundsError { op: *self, index: index_val });
                            }
                            return Result::Ok(Value::Scalar(v2_data.to_array()[index_val]))
                        },
                        Value::Vector3(v3_data) => 
                        {
                            if index_val > 2
                            {
                                return Result::Err(EvaluationError::IndexOutOfBoundsError { op: *self, index: index_val });
                            }
                            return Result::Ok(Value::Scalar(v3_data.to_array()[index_val]))
                        },
                        Value::Color(c_data) => 
                        {
                            if index_val > 3
                            {
                                return Result::Err(EvaluationError::IndexOutOfBoundsError { op: *self, index: index_val });
                            }
                            return Result::Ok(Value::Scalar(c_data.to_array()[index_val]))
                        },
                        Value::Matrix3(m3_data) => 
                        {
                            if index_val > 8
                            {
                                return Result::Err(EvaluationError::IndexOutOfBoundsError { op: *self, index: index_val });
                            }
                            return Result::Ok(Value::Scalar(m3_data.to_cols_array()[index_val]))
                        },
                        Value::Matrix4(m4_data) => 
                        {
                            if index_val > 15
                            {
                                return Result::Err(EvaluationError::IndexOutOfBoundsError { op: *self, index: index_val });
                            }
                            return Result::Ok(Value::Scalar(m4_data.to_cols_array()[index_val]))
                        },
                        _ => Result::Err(EvaluationError::TypeMismatchError{ op: *self }),
                    }
                }
                else
                {
                    return Result::Err(EvaluationError::TypeMismatchError{ op: *self });
                }
                
            },
            Operator::CreateQuaternion => 
            {
                let angle = value_stack.pop();
                let axis = value_stack.pop();
                if angle.is_none() || axis.is_none()
                {
                    return Result::Err(EvaluationError::ValueUnderflowError { op: *self });
                }
                let angle_val = match angle.unwrap()
                {
                    Value::Scalar(s_angle) => s_angle,
                    _ => return Result::Err(EvaluationError::TypeMismatchError { op: *self }),    
                };
                let axis_val = match axis.unwrap()
                {
                    Value::Vector3(v3_axis) => v3_axis,
                    _ => return Result::Err(EvaluationError::TypeMismatchError { op: *self }),
                };
                return Result::Ok(Value::Quaternion(Quat::from_axis_angle(axis_val, angle_val)));
            },
            Operator::Swizzle2 | Operator::Swizzle3 | Operator::Swizzle4 => 
            {
                //swizzle indices, output size, input value
                let data = value_stack.pop();
                let output_size = value_stack.pop();
                if data.is_none() || output_size.is_none()
                {
                    return Result::Err(EvaluationError::ValueUnderflowError { op: *self });
                }
                let data_val = match data.unwrap()
                {
                    Value::Vector2(_) | Value::Vector3(_) | Value::Color(_) => data.unwrap().val_into_float_list(),
                    _ => return Result::Err(EvaluationError::TypeMismatchError { op: *self }),    
                };
                let size_val = match output_size.unwrap()
                {
                    Value::Scalar(s_size) => 
                    {
                        let i_size = s_size.round();
                        if i_size < 0.0 || i_size > 4.0
                        {
                            return Result::Err(EvaluationError::InvalidOutputSize { op: *self, size: i_size });
                        }
                        i_size as usize
                    },
                    _ => return Result::Err(EvaluationError::TypeMismatchError { op: *self }),
                };
                let output =
                {
                    let mut ret: Vec<f32> = vec![];
                    for i in 0..size_val
                    {
                        let swizzdex = value_stack.pop();
                        if swizzdex.is_none()
                        {
                            return Result::Err(EvaluationError::ValueUnderflowError { op: *self });
                        }
                        let swizz_val = match swizzdex.unwrap()
                        {
                            Value::Scalar(s_size) => 
                            {
                                let i_size = s_size.round();
                                if i_size < 0.0 || i_size > (data_val.len() as f32)
                                {
                                    return Result::Err(EvaluationError::InvalidOutputSize { op: *self, size: i_size });
                                }
                                i_size as usize
                            },
                            _ => return Result::Err(EvaluationError::TypeMismatchError { op: *self }),
                        };
                        ret.push(data_val[swizz_val]);
                    }
                    ret
                };
                match size_val
                {
                    1 => return Result::Ok(Value::Scalar(output[0])),
                    2 => return Result::Ok(Value::Vector2(Vec2::new(output[1], output[0]))),
                    3 => return Result::Ok(Value::Vector3(Vec3::new(output[2], output[1], output[0]))),
                    4 => return Result::Ok(Value::Color(Vec4::new(output[3], output[2], output[1], output[0]))),
                    _ => return Result::Err(EvaluationError::TypeMismatchError { op: *self })
                }
            },
        }
    }
    
}