use std::{f32::{consts::PI, EPSILON}, collections::HashMap};

#[cfg(test)]
use pest::Parser;

use crate::pibald::{ScalarOperator, ColorMap, BinaryColorMap, ColorMapVariant, ScalarExpression};

use super::{PibaldParser, Rule, ScalarTerm, ScalarOperand, PibaldEvaluator};

#[test]
fn test_scalar_parse()
{
    let pair = PibaldParser::parse(Rule::scalar, "rho + 25.0").unwrap().next().unwrap();
    let test = PibaldParser::parse_scalar_expr(pair);
    assert_eq!(test.len(), 3, "failed to read all terms from scalar expression");
    assert!(matches!(test.get(0).unwrap(), ScalarTerm::Value(ScalarOperand::Variable(v)) if v == "rho"), "failed to parse scalar var");
    assert!(matches!(test.get(1).unwrap(), ScalarTerm::Value(ScalarOperand::Constant(c)) if *c == 25.0), "failed to parse scalar const");
    assert!(matches!(test.get(2).unwrap(), ScalarTerm::Operator(ScalarOperator::Add)), "failed to push add operator to expression stack");
}

#[test]
fn test_scalar_parse_recursive()
{
    let pair = PibaldParser::parse(Rule::scalar, "log((rho + 25.0) * 8.0) / PI").unwrap().next().unwrap();
    let test = PibaldParser::parse_scalar_expr(pair);
    assert_eq!(test.len(), 8, "failed to read all terms from scalar expression");
    assert!(matches!(test.get(0).unwrap(), ScalarTerm::Value(ScalarOperand::Variable(v)) if v == "rho"), "failed to parse scalar var in parentheses");
    assert!(matches!(test.get(1).unwrap(), ScalarTerm::Value(ScalarOperand::Constant(c)) if *c == 25.0), "failed to parse scalar const in parentheses");
    assert!(matches!(test.get(2).unwrap(), ScalarTerm::Operator(ScalarOperator::Add)), "failed to push add operator to expression stack in parentheses");
    assert!(matches!(test.get(3).unwrap(), ScalarTerm::Value(ScalarOperand::Constant(c)) if *c == 8.0), "failed to parse scalar const");
    assert!(matches!(test.get(4).unwrap(), ScalarTerm::Operator(ScalarOperator::Multiply)), "failed to push multiply operator to expression stack");
    assert!(matches!(test.get(5).unwrap(), ScalarTerm::Operator(ScalarOperator::Log)), "failed to parse log operation");
    assert!(matches!(test.get(6).unwrap(), ScalarTerm::Value(ScalarOperand::Constant(c)) if *c == PI), "failed to parse PI constant");
    assert!(matches!(test.get(7).unwrap(), ScalarTerm::Operator(ScalarOperator::Divide)), "failed to parse divide");
}

#[test]
fn test_expression_eval()
{
    let pair = PibaldParser::parse(Rule::scalar, "0.5 * (4.0 + iota)").unwrap().next().unwrap();
    let test = ScalarExpression::new(PibaldParser::parse_scalar_expr(pair));
    let mut arg_map: HashMap<&str, f32> = HashMap::new();
    arg_map.insert("iota", 6.0);
    assert!((test.evaluate(&arg_map).unwrap() - 5.0).abs() < 0.000001, "test_expression_eval failed");
}


#[test]
fn test_shader_parse()
{
    let test_str = 
    "SOLID
    (
        color((1.0 + 4.0) * 50.0, 1.0, log(6.0 / 2.0) ^ E, 1.0), 
        SD_CIRCLE(mat4(), 0.5) 
    )
    SINGULAR(mat4(), 0)
    ";
    let shader_class = PibaldParser::parse_shader_class(test_str).unwrap();
    assert_eq!(1, shader_class.color_maps.len(), "Incorrect amount of color maps");
    assert!
    (
        matches!
        ( 
            shader_class.color_maps.get(0).unwrap().variant,
            ColorMapVariant::Binary(..),
        ),
        "Failed to parse color and threshold values"
    );
}

