use crate::empty_expr;
use crate::prelude::*;
use crate::zero_expr_float;

/// Values in the AST
#[derive(Clone, Debug)]
pub enum ASTValue {
    None,
    Boolean(Option<String>, bool),
    Int(Option<String>, i32),
    Int2(Option<String>, Box<Expr>, Box<Expr>),
    Int3(Option<String>, Box<Expr>, Box<Expr>, Box<Expr>),
    Int4(Option<String>, Box<Expr>, Box<Expr>, Box<Expr>, Box<Expr>),
    Float(Option<String>, f32),
    Float2(Option<String>, Box<Expr>, Box<Expr>),
    Float3(Option<String>, Box<Expr>, Box<Expr>, Box<Expr>),
    Float4(Option<String>, Box<Expr>, Box<Expr>, Box<Expr>, Box<Expr>),
    Mat2(Option<String>, Vec<Box<Expr>>),
    Mat3(Option<String>, Vec<Box<Expr>>),
    Mat4(Option<String>, Vec<Box<Expr>>),
    String(Option<String>, String),
    Struct(String, Option<String>, Vec<Box<Expr>>),
    Function(String, Vec<ASTValue>, Box<ASTValue>),
}

impl ASTValue {
    pub fn write_definition(&self, instr: &str, name: &str, precision: &str) -> Vec<String> {
        match self {
            ASTValue::Int(_, _) => vec![format!("({} {} f{})", instr, name, precision)],
            ASTValue::Int2(_, _, _) => vec![
                format!("({} ${}_x i{})", instr, name, precision),
                format!("({} ${}_y i{})", instr, name, precision),
            ],
            ASTValue::Int3(_, _, _, _) => vec![
                format!("({} ${}_x i{})", instr, name, precision),
                format!("({} ${}_y i{})", instr, name, precision),
                format!("({} ${}_z i{})", instr, name, precision),
            ],
            ASTValue::Int4(_, _, _, _, _) => vec![
                format!("({} ${}_x i{})", instr, name, precision),
                format!("({} ${}_y i{})", instr, name, precision),
                format!("({} ${}_z i{})", instr, name, precision),
                format!("({} ${}_w i{})", instr, name, precision),
            ],
            ASTValue::Float(_, _) => vec![format!("({} {} f{})", instr, name, precision)],
            ASTValue::Float2(_, _, _) => vec![
                format!("({} ${}_x f{})", instr, name, precision),
                format!("({} ${}_y f{})", instr, name, precision),
            ],
            ASTValue::Float3(_, _, _, _) => vec![
                format!("({} ${}_x f{})", instr, name, precision),
                format!("({} ${}_y f{})", instr, name, precision),
                format!("({} ${}_z f{})", instr, name, precision),
            ],
            ASTValue::Float4(_, _, _, _, _) => vec![
                format!("({} ${}_x f{})", instr, name, precision),
                format!("({} ${}_y f{})", instr, name, precision),
                format!("({} ${}_z f{})", instr, name, precision),
                format!("({} ${}_w f{})", instr, name, precision),
            ],
            ASTValue::Mat2(_, _) => vec![
                format!("({} ${}_x f{})", instr, name, precision),
                format!("({} ${}_y f{})", instr, name, precision),
                format!("({} ${}_z f{})", instr, name, precision),
                format!("({} ${}_w f{})", instr, name, precision),
            ],
            ASTValue::Mat3(_, _) => vec![
                format!("({} ${}_1 f{})", instr, name, precision),
                format!("({} ${}_2 f{})", instr, name, precision),
                format!("({} ${}_3 f{})", instr, name, precision),
                format!("({} ${}_4 f{})", instr, name, precision),
                format!("({} ${}_5 f{})", instr, name, precision),
                format!("({} ${}_6 f{})", instr, name, precision),
                format!("({} ${}_7 f{})", instr, name, precision),
                format!("({} ${}_8 f{})", instr, name, precision),
                format!("({} ${}_9 f{})", instr, name, precision),
            ],
            ASTValue::Struct(_, _, _) => vec![format!("({} ${} i32)", instr, name)],

            _ => vec![],
        }
    }
    pub fn write_access(&self, instr: &str, name: &str) -> Vec<String> {
        match self {
            ASTValue::Float(_, _) | ASTValue::Int(_, _) => vec![format!("({} {})", instr, name)],
            ASTValue::Float2(_, _, _) | ASTValue::Int2(_, _, _) => vec![
                format!("({} ${}_x)", instr, name),
                format!("({} ${}_y)", instr, name),
            ],
            ASTValue::Float3(_, _, _, _) | ASTValue::Int3(_, _, _, _) => vec![
                format!("({} ${}_x)", instr, name),
                format!("({} ${}_y)", instr, name),
                format!("({} ${}_z)", instr, name),
            ],
            ASTValue::Float4(_, _, _, _, _) | ASTValue::Int4(_, _, _, _, _) => vec![
                format!("({} ${}_x)", instr, name),
                format!("({} ${}_y)", instr, name),
                format!("({} ${}_z)", instr, name),
                format!("({} ${}_w)", instr, name),
            ],
            ASTValue::Mat2(_, _) => vec![
                format!("({} ${}_x)", instr, name),
                format!("({} ${}_y)", instr, name),
                format!("({} ${}_z)", instr, name),
                format!("({} ${}_w)", instr, name),
            ],
            ASTValue::Mat3(_, _) => vec![
                format!("({} ${}_1)", instr, name),
                format!("({} ${}_2)", instr, name),
                format!("({} ${}_3)", instr, name),
                format!("({} ${}_4)", instr, name),
                format!("({} ${}_5)", instr, name),
                format!("({} ${}_6)", instr, name),
                format!("({} ${}_7)", instr, name),
                format!("({} ${}_8)", instr, name),
                format!("({} ${}_9)", instr, name),
            ],
            ASTValue::Struct(_, _, _) => vec![format!("({} ${})", instr, name)],
            _ => vec![],
        }
    }

    /// Returns the value as a float if it is one.
    pub fn to_float(&self) -> Option<f32> {
        match self {
            ASTValue::Float(_, f) => Some(*f),
            _ => None,
        }
    }

    /// Returns the value as an integer if it is one.
    pub fn to_int(&self) -> Option<i32> {
        match self {
            ASTValue::Int(_, i) => Some(*i),
            _ => None,
        }
    }

    /// Returns true if the value is float based.
    pub fn is_float_based(&self) -> bool {
        matches!(
            self,
            ASTValue::Float(_, _)
                | ASTValue::Float2(_, _, _)
                | ASTValue::Float3(_, _, _, _)
                | ASTValue::Float4(_, _, _, _, _)
        )
    }

    /// The truthiness of the value.
    pub fn is_truthy(&self) -> bool {
        match self {
            ASTValue::Boolean(_, b) => *b,
            ASTValue::Int(_, i) => *i != 0,
            ASTValue::Int2(_, _, _) => true,
            ASTValue::Int3(_, _, _, _) => true,
            ASTValue::Int4(_, _, _, _, _) => true,
            ASTValue::Float(_, i) => *i != 0.0,
            ASTValue::Float2(_, _, _) => true,
            ASTValue::Float3(_, _, _, _) => true,
            ASTValue::Float4(_, _, _, _, _) => true,
            ASTValue::String(_, s) => !s.is_empty(),
            ASTValue::Function(_, _, _) => true,
            _ => false,
        }
    }

    // The components of the value.
    pub fn components(&self) -> usize {
        match self {
            ASTValue::Int(_, _) => 1,
            ASTValue::Int2(_, _, _) => 2,
            ASTValue::Int3(_, _, _, _) => 3,
            ASTValue::Int4(_, _, _, _, _) => 4,
            ASTValue::Float(_, _) => 1,
            ASTValue::Float2(_, _, _) => 2,
            ASTValue::Float3(_, _, _, _) => 3,
            ASTValue::Float4(_, _, _, _, _) => 4,
            ASTValue::Mat2(_, _) => 4,
            ASTValue::Mat3(_, _) => 9,
            ASTValue::Mat4(_, _) => 16,
            _ => 0,
        }
    }

    /// Returns the RPU type of the given value.
    pub fn to_type(&self) -> String {
        match self {
            ASTValue::None => "void".to_string(),
            ASTValue::Boolean(_, _) => "bool".to_string(),
            ASTValue::Int(_, _) => "int".to_string(),
            ASTValue::Int2(_, _, _) => "ivec2".to_string(),
            ASTValue::Int3(_, _, _, _) => "ivec3".to_string(),
            ASTValue::Int4(_, _, _, _, _) => "ivec4".to_string(),
            ASTValue::Float(_, _) => "float".to_string(),
            ASTValue::Float2(_, _, _) => "vec2".to_string(),
            ASTValue::Float3(_, _, _, _) => "vec3".to_string(),
            ASTValue::Float4(_, _, _, _, _) => "vec4".to_string(),
            ASTValue::String(_, _) => "string".to_string(),
            ASTValue::Function(_, _, _) => "fn".to_string(),
            ASTValue::Mat2(_, _) => "mat2".to_string(),
            ASTValue::Mat3(_, _) => "mat3".to_string(),
            ASTValue::Mat4(_, _) => "mat4".to_string(),
            ASTValue::Struct(_, _, _) => "struct".to_string(),
        }
    }

    /// Returns the WAT type of the given value.
    pub fn to_wat_type(&self, pr: &str) -> Option<String> {
        match self {
            ASTValue::Boolean(_, _) => Some(format!("(i{}", pr)),
            ASTValue::Int(_, _) => Some(format!("i{}", pr)),
            ASTValue::Int2(_, _, _) => Some(format!("i{} i{}", pr, pr)),
            ASTValue::Int3(_, _, _, _) => Some(format!("i{} i{} i{}", pr, pr, pr)),
            ASTValue::Int4(_, _, _, _, _) => Some(format!("i{} i{} i{} i{}", pr, pr, pr, pr)),
            ASTValue::Float(_, _) => Some(format!("f{}", pr)),
            ASTValue::Float2(_, _, _) => Some(format!("f{} f{}", pr, pr)),
            ASTValue::Float3(_, _, _, _) => Some(format!("f{} f{} f{}", pr, pr, pr)),
            ASTValue::Float4(_, _, _, _, _) => Some(format!("f{} f{} f{} f{}", pr, pr, pr, pr)),
            ASTValue::Mat2(_, _) => Some(format!("f{} f{} f{} f{}", pr, pr, pr, pr)),
            ASTValue::Mat3(_, _) => Some(format!(
                "f{pr} f{pr} f{pr} f{pr} f{pr} f{pr} f{pr} f{pr} f{pr}",
                pr = pr
            )),
            ASTValue::Mat4(_, _) => Some(format!(
                "f{pr} f{pr} f{pr} f{pr} f{pr} f{pr} f{pr} f{pr} f{pr} f{pr} f{pr} f{pr} f{pr} f{pr} f{pr} f{pr}",
                pr = pr
            )),
            ASTValue::Struct(_, _, _) => Some("i32".to_string()),

            _ => None,
        }
    }

    /// Returns the WAT type of the given value component.
    pub fn to_wat_component_type(&self, pr: &str) -> String {
        if self.is_float_based() {
            format!("f{}", pr)
        } else {
            format!("i{}", pr)
        }
    }

    /// Creates an ASTValue from a TokenType.
    pub fn from_token_type(name: Option<String>, token_type: &TokenType) -> ASTValue {
        match token_type {
            TokenType::Void => ASTValue::None,
            TokenType::True => ASTValue::Boolean(name, true),
            TokenType::False => ASTValue::Boolean(name, false),
            TokenType::Int => ASTValue::Int(name, 0),
            TokenType::Int2 => ASTValue::Int2(name, empty_expr!(), empty_expr!()),
            TokenType::Int3 => ASTValue::Int3(name, empty_expr!(), empty_expr!(), empty_expr!()),
            TokenType::Int4 => ASTValue::Int4(
                name,
                empty_expr!(),
                empty_expr!(),
                empty_expr!(),
                empty_expr!(),
            ),
            TokenType::Float => ASTValue::Float(name, 0.0),
            TokenType::Float2 => ASTValue::Float2(name, empty_expr!(), empty_expr!()),
            TokenType::Float3 => {
                ASTValue::Float3(name, empty_expr!(), empty_expr!(), empty_expr!())
            }
            TokenType::Float4 => ASTValue::Float4(
                name,
                empty_expr!(),
                empty_expr!(),
                empty_expr!(),
                empty_expr!(),
            ),
            TokenType::Mat2 => ASTValue::Mat2(
                name,
                vec![empty_expr!(), empty_expr!(), empty_expr!(), empty_expr!()],
            ),
            TokenType::Mat3 => ASTValue::Mat3(
                name,
                vec![
                    empty_expr!(),
                    empty_expr!(),
                    empty_expr!(),
                    empty_expr!(),
                    empty_expr!(),
                    empty_expr!(),
                    empty_expr!(),
                    empty_expr!(),
                    empty_expr!(),
                ],
            ),
            TokenType::Mat4 => ASTValue::Mat4(
                name,
                vec![
                    empty_expr!(),
                    empty_expr!(),
                    empty_expr!(),
                    empty_expr!(),
                    empty_expr!(),
                    empty_expr!(),
                    empty_expr!(),
                    empty_expr!(),
                    empty_expr!(),
                    empty_expr!(),
                    empty_expr!(),
                    empty_expr!(),
                    empty_expr!(),
                    empty_expr!(),
                    empty_expr!(),
                    empty_expr!(),
                ],
            ),
            TokenType::String => ASTValue::String(name, "".to_string()),
            _ => ASTValue::None,
        }
    }

    pub fn name(&self) -> Option<String> {
        match self {
            ASTValue::Boolean(name, _) => name.clone(),
            ASTValue::Int(name, _) => name.clone(),
            ASTValue::Int2(name, _, _) => name.clone(),
            ASTValue::Int3(name, _, _, _) => name.clone(),
            ASTValue::Int4(name, _, _, _, _) => name.clone(),
            ASTValue::Float(name, _) => name.clone(),
            ASTValue::Float2(name, _, _) => name.clone(),
            ASTValue::Float3(name, _, _, _) => name.clone(),
            ASTValue::Float4(name, _, _, _, _) => name.clone(),
            ASTValue::String(name, _) => name.clone(),
            ASTValue::Function(name, _, _) => Some(name.clone()),
            ASTValue::Mat2(name, _) => name.clone(),
            ASTValue::Mat3(name, _) => name.clone(),
            ASTValue::Mat4(name, _) => name.clone(),
            ASTValue::Struct(_, instance_name, _) => instance_name.clone(),
            ASTValue::None => None,
        }
    }

    /// Returns an expression for each value with 0 content.
    pub fn as_empty_expression(&self) -> Expr {
        match self {
            ASTValue::Int(_, _) => Expr::Value(
                ASTValue::Int(Some("1".to_string()), 0),
                vec![],
                vec![],
                Location::default(),
            ),
            ASTValue::Float(_, _) => Expr::Value(
                ASTValue::Float(Some("1".to_string()), 0.0),
                vec![],
                vec![],
                Location::default(),
            ),
            ASTValue::Float3(_, _, _, _) => Expr::Value(
                ASTValue::Float3(
                    Some("3".to_string()),
                    zero_expr_float!(),
                    zero_expr_float!(),
                    zero_expr_float!(),
                ),
                vec![],
                vec![],
                Location::default(),
            ),
            _ => Expr::Value(self.clone(), vec![], vec![], Location::default()),
        }
    }
}
