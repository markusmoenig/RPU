use crate::prelude::*;

#[macro_export]
macro_rules! empty_expr {
    () => {
        Box::new(Expr::Value(ASTValue::None, vec![], Location::default()))
    };
}

#[macro_export]
macro_rules! zero_expr_int {
    () => {
        Box::new(Expr::Value(
            ASTValue::Int(None, 0),
            vec![],
            Location::default(),
        ))
    };
}

#[macro_export]
macro_rules! zero_expr_float {
    () => {
        Box::new(Expr::Value(
            ASTValue::Float(None, 0.0),
            vec![],
            Location::default(),
        ))
    };
}

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
    String(Option<String>, String),
    Function(String, Vec<ASTValue>, Box<ASTValue>),
}

impl ASTValue {
    /// Returns the value as a float if it is one.
    pub fn to_float(&self) -> Option<f32> {
        match self {
            ASTValue::Float(_, f) => Some(*f),
            _ => None,
        }
    }

    ///
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
            ASTValue::None => false,
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
            _ => None,
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
            ASTValue::None => None,
        }
    }
}

/// Statements in the AST
#[derive(Clone, Debug)]
pub enum Stmt {
    If(Box<Expr>, Box<Stmt>, Option<Box<Stmt>>, Location),
    While(Box<Expr>, Box<Stmt>, Location),
    Print(Box<Expr>, Location),
    Block(Vec<Box<Stmt>>, Location),
    Expression(Box<Expr>, Location),
    VarDeclaration(String, ASTValue, Box<Expr>, Location),
    FunctionDeclaration(
        String,
        Vec<ASTValue>,
        Vec<Box<Stmt>>,
        ASTValue,
        bool,
        Location,
    ),
    Return(Box<Expr>, Location),
    Break(Location),
}

/// Expressions in the AST
#[derive(Clone, Debug)]
pub enum Expr {
    Value(ASTValue, Vec<u8>, Location),
    Logical(Box<Expr>, LogicalOperator, Box<Expr>, Location),
    Unary(UnaryOperator, Box<Expr>, Location),
    Equality(Box<Expr>, EqualityOperator, Box<Expr>, Location),
    Comparison(Box<Expr>, ComparisonOperator, Box<Expr>, Location),
    Binary(Box<Expr>, BinaryOperator, Box<Expr>, Location),
    Grouping(Box<Expr>, Location),
    Variable(String, Vec<u8>, Location),
    VariableAssignment(String, Vec<u8>, Box<Expr>, Location),
    FunctionCall(Box<Expr>, Vec<Box<Expr>>, Location),
}

/// Logical operators in the AST
#[derive(Clone, PartialEq, Debug)]
pub enum LogicalOperator {
    And,
    Or,
}

/// Unary operators in the AST
#[derive(Clone, Debug)]
pub enum UnaryOperator {
    Negate,
    Minus,
}

/// Binary operators in the AST
#[derive(Clone, Debug)]
pub enum BinaryOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
}

impl BinaryOperator {
    pub fn describe(&self) -> &str {
        match self {
            BinaryOperator::Add => "+",
            BinaryOperator::Subtract => "-",
            BinaryOperator::Multiply => "*",
            BinaryOperator::Divide => "/",
        }
    }
}

/// Comparison operators in the AST
#[derive(Clone, Debug)]
pub enum ComparisonOperator {
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
}

/// Equality operators in the AST
#[derive(Clone, Debug)]
pub enum EqualityOperator {
    NotEqual,
    Equal,
}

/// Visitor trait
pub trait Visitor {
    fn new() -> Self
    where
        Self: Sized;

    fn print(
        &mut self,
        expression: &Expr,
        loc: &Location,
        ctx: &mut Context,
    ) -> Result<ASTValue, String>;

    fn block(
        &mut self,
        list: &[Box<Stmt>],
        loc: &Location,
        ctx: &mut Context,
    ) -> Result<ASTValue, String>;

    fn expression(
        &mut self,
        expression: &Expr,
        loc: &Location,
        ctx: &mut Context,
    ) -> Result<ASTValue, String>;

    fn var_declaration(
        &mut self,
        name: &str,
        static_type: &ASTValue,
        expression: &Expr,
        loc: &Location,
        ctx: &mut Context,
    ) -> Result<ASTValue, String>;

    fn value(
        &mut self,
        value: ASTValue,
        swizzle: &[u8],
        loc: &Location,
        ctx: &mut Context,
    ) -> Result<ASTValue, String>;

    fn unary(
        &mut self,
        op: &UnaryOperator,
        expr: &Expr,
        loc: &Location,
        ctx: &mut Context,
    ) -> Result<ASTValue, String>;

    fn equality(
        &mut self,
        left: &Expr,
        op: &EqualityOperator,
        right: &Expr,
        loc: &Location,
        ctx: &mut Context,
    ) -> Result<ASTValue, String>;

    fn comparison(
        &mut self,
        left: &Expr,
        op: &ComparisonOperator,
        right: &Expr,
        loc: &Location,
        ctx: &mut Context,
    ) -> Result<ASTValue, String>;

    fn binary(
        &mut self,
        left: &Expr,
        op: &BinaryOperator,
        right: &Expr,
        loc: &Location,
        ctx: &mut Context,
    ) -> Result<ASTValue, String>;

    fn grouping(
        &mut self,
        expression: &Expr,
        loc: &Location,
        ctx: &mut Context,
    ) -> Result<ASTValue, String>;

    fn variable(
        &mut self,
        name: String,
        swizzle: &[u8],
        loc: &Location,
        ctx: &mut Context,
    ) -> Result<ASTValue, String>;

    fn variable_assignment(
        &mut self,
        name: String,
        swizzle: &[u8],
        expression: &Expr,
        loc: &Location,
        ctx: &mut Context,
    ) -> Result<ASTValue, String>;

    fn function_call(
        &mut self,
        callee: &Expr,
        args: &[Box<Expr>],
        loc: &Location,
        ctx: &mut Context,
    ) -> Result<ASTValue, String>;

    #[allow(clippy::too_many_arguments)]
    fn function_declaration(
        &mut self,
        name: &str,
        args: &[ASTValue],
        body: &[Box<Stmt>],
        returns: &ASTValue,
        export: &bool,
        loc: &Location,
        ctx: &mut Context,
    ) -> Result<ASTValue, String>;

    fn return_stmt(
        &mut self,
        expr: &Expr,
        loc: &Location,
        ctx: &mut Context,
    ) -> Result<ASTValue, String>;

    fn break_stmt(&mut self, loc: &Location, ctx: &mut Context) -> Result<ASTValue, String>;

    fn if_stmt(
        &mut self,
        cond: &Expr,
        then_stmt: &Stmt,
        else_stmt: &Option<Box<Stmt>>,
        loc: &Location,
        ctx: &mut Context,
    ) -> Result<ASTValue, String>;

    fn while_stmt(
        &mut self,
        cond: &Expr,
        body_stmt: &Stmt,
        loc: &Location,
        ctx: &mut Context,
    ) -> Result<ASTValue, String>;

    fn logical_expr(
        &mut self,
        left: &Expr,
        op: &LogicalOperator,
        right: &Expr,
        loc: &Location,
        ctx: &mut Context,
    ) -> Result<ASTValue, String>;
}

impl Stmt {
    pub fn accept(&self, visitor: &mut dyn Visitor, ctx: &mut Context) -> Result<ASTValue, String> {
        match self {
            Stmt::If(cond, then_stmt, else_stmt, loc) => {
                visitor.if_stmt(cond, then_stmt, else_stmt, loc, ctx)
            }
            Stmt::While(cond, body, loc) => visitor.while_stmt(cond, body, loc, ctx),
            Stmt::Print(expression, loc) => visitor.print(expression, loc, ctx),
            Stmt::Block(list, loc) => visitor.block(list, loc, ctx),
            Stmt::Expression(expression, loc) => visitor.expression(expression, loc, ctx),
            Stmt::VarDeclaration(name, static_type, initializer, loc) => {
                visitor.var_declaration(name, static_type, initializer, loc, ctx)
            }
            Stmt::FunctionDeclaration(name, args, body, returns, export, loc) => {
                visitor.function_declaration(name, args, body, returns, export, loc, ctx)
            }
            Stmt::Break(loc) => visitor.break_stmt(loc, ctx),
            Stmt::Return(expr, loc) => visitor.return_stmt(expr, loc, ctx),
        }
    }
}

impl Expr {
    pub fn accept(&self, visitor: &mut dyn Visitor, ctx: &mut Context) -> Result<ASTValue, String> {
        match self {
            Expr::Value(value, swizzle, loc) => visitor.value(value.clone(), swizzle, loc, ctx),
            Expr::Logical(left, op, right, loc) => visitor.logical_expr(left, op, right, loc, ctx),
            Expr::Unary(op, expr, loc) => visitor.unary(op, expr, loc, ctx),
            Expr::Equality(left, op, right, loc) => visitor.equality(left, op, right, loc, ctx),
            Expr::Comparison(left, op, right, loc) => visitor.comparison(left, op, right, loc, ctx),
            Expr::Binary(left, op, right, loc) => visitor.binary(left, op, right, loc, ctx),
            Expr::Grouping(expr, loc) => visitor.grouping(expr, loc, ctx),
            Expr::Variable(name, swizzle, loc) => visitor.variable(name.clone(), swizzle, loc, ctx),
            Expr::VariableAssignment(name, swizzle, expr, loc) => {
                visitor.variable_assignment(name.clone(), swizzle, expr, loc, ctx)
            }
            Expr::FunctionCall(callee, args, loc) => visitor.function_call(callee, args, loc, ctx),
        }
    }
}

/// Location in the source code
#[derive(Clone, Debug)]
pub struct Location {
    pub file: String,
    pub line: usize,
}

impl Default for Location {
    fn default() -> Self {
        Self::new("".to_string(), 0)
    }
}

impl Location {
    pub fn new(file: String, line: usize) -> Self {
        Location { file, line }
    }

    pub fn describe(&self) -> String {
        // format!("in '{}' at line {}.", self.file, self.line)
        format!("at line {}.", self.line)
    }
}
