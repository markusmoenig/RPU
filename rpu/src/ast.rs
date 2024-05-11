use crate::prelude::*;

#[macro_export]
macro_rules! empty_expr {
    () => {
        Box::new(Expr::Value(
            ASTValue::None,
            vec![],
            vec![],
            Location::default(),
        ))
    };
}

#[macro_export]
macro_rules! zero_expr_int {
    () => {
        Box::new(Expr::Value(
            ASTValue::Int(None, 0),
            vec![],
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
            vec![],
            Location::default(),
        ))
    };
}

/// Statements in the AST
#[derive(Clone, Debug)]
pub enum Stmt {
    If(Box<Expr>, Box<Stmt>, Option<Box<Stmt>>, Location),
    While(Box<Expr>, Box<Stmt>, Location),
    For(
        Vec<Box<Expr>>,
        Vec<Box<Expr>>,
        Vec<Box<Expr>>,
        Box<Stmt>,
        Location,
    ),
    Print(Box<Expr>, Location),
    Block(Vec<Box<Stmt>>, Location),
    Expression(Box<Expr>, Location),
    VarDeclaration(String, ASTValue, Box<Expr>, Location),
    StructDeclaration(String, Vec<(String, ASTValue)>, Location),
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
    Value(ASTValue, Vec<u8>, Vec<String>, Location),
    Logical(Box<Expr>, LogicalOperator, Box<Expr>, Location),
    Unary(UnaryOperator, Box<Expr>, Location),
    Equality(Box<Expr>, EqualityOperator, Box<Expr>, Location),
    Comparison(Box<Expr>, ComparisonOperator, Box<Expr>, Location),
    Binary(Box<Expr>, BinaryOperator, Box<Expr>, Location),
    Grouping(Box<Expr>, Location),
    Variable(String, Vec<u8>, Vec<String>, Location),
    VariableAssignment(
        String,
        AssignmentOperator,
        Vec<u8>,
        Vec<String>,
        Box<Expr>,
        Location,
    ),
    FunctionCall(Box<Expr>, Vec<u8>, Vec<String>, Vec<Box<Expr>>, Location),
    Ternary(Box<Expr>, Box<Expr>, Box<Expr>, Location),
}

/// Assignment operators in the AST
#[derive(Clone, PartialEq, Debug)]
pub enum AssignmentOperator {
    Assign,
    AddAssign,
    SubtractAssign,
    MultiplyAssign,
    DivideAssign,
}

impl AssignmentOperator {
    pub fn describe(&self) -> &str {
        match self {
            AssignmentOperator::Assign => "=",
            AssignmentOperator::AddAssign => "+=",
            AssignmentOperator::SubtractAssign => "-=",
            AssignmentOperator::MultiplyAssign => "*=",
            AssignmentOperator::DivideAssign => "/=",
        }
    }

    pub fn to_wat_type(&self, for_value: &ASTValue) -> String {
        match self {
            AssignmentOperator::Assign => "".to_string(),
            AssignmentOperator::AddAssign => "add".to_string(),
            AssignmentOperator::SubtractAssign => "sub".to_string(),
            AssignmentOperator::MultiplyAssign => "mul".to_string(),
            AssignmentOperator::DivideAssign => {
                if for_value.is_float_based() {
                    "div".to_string()
                } else {
                    "div_s".to_string()
                }
            }
        }
    }
}

/// Logical operators in the AST
#[derive(Clone, PartialEq, Debug)]
pub enum LogicalOperator {
    And,
    Or,
}

impl LogicalOperator {
    pub fn describe(&self) -> &str {
        match self {
            LogicalOperator::And => "&&",
            LogicalOperator::Or => "||",
        }
    }
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

impl ComparisonOperator {
    pub fn describe(&self) -> &str {
        match self {
            ComparisonOperator::Greater => ">",
            ComparisonOperator::GreaterEqual => ">=",
            ComparisonOperator::Less => "<",
            ComparisonOperator::LessEqual => "<=",
        }
    }
}

/// Equality operators in the AST
#[derive(Clone, Debug)]
pub enum EqualityOperator {
    NotEqual,
    Equal,
}

impl EqualityOperator {
    pub fn describe(&self) -> &str {
        match self {
            EqualityOperator::NotEqual => "!=",
            EqualityOperator::Equal => "==",
        }
    }
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
        field_path: &[String],
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
        field_path: &[String],
        loc: &Location,
        ctx: &mut Context,
    ) -> Result<ASTValue, String>;

    #[allow(clippy::too_many_arguments)]
    fn variable_assignment(
        &mut self,
        name: String,
        op: &AssignmentOperator,
        swizzle: &[u8],
        field_path: &[String],
        expression: &Expr,
        loc: &Location,
        ctx: &mut Context,
    ) -> Result<ASTValue, String>;

    fn func_call(
        &mut self,
        callee: &Expr,
        swizzle: &[u8],
        field_path: &[String],
        args: &[Box<Expr>],
        loc: &Location,
        ctx: &mut Context,
    ) -> Result<ASTValue, String>;

    fn struct_declaration(
        &mut self,
        name: &str,
        field: &[(String, ASTValue)],
        loc: &Location,
        ctx: &mut Context,
    ) -> Result<ASTValue, String>;

    #[allow(clippy::too_many_arguments)]
    fn func_declaration(
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

    fn for_stmt(
        &mut self,
        init: &[Box<Expr>],
        cond: &[Box<Expr>],
        incr: &[Box<Expr>],
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

    fn ternary(
        &mut self,
        condition: &Expr,
        then_expr: &Expr,
        else_expr: &Expr,
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
            Stmt::For(init, cond, incr, body, loc) => {
                visitor.for_stmt(init, cond, incr, body, loc, ctx)
            }
            Stmt::Print(expression, loc) => visitor.print(expression, loc, ctx),
            Stmt::Block(list, loc) => visitor.block(list, loc, ctx),
            Stmt::Expression(expression, loc) => visitor.expression(expression, loc, ctx),
            Stmt::VarDeclaration(name, static_type, initializer, loc) => {
                visitor.var_declaration(name, static_type, initializer, loc, ctx)
            }
            Stmt::StructDeclaration(name, fields, loc) => {
                visitor.struct_declaration(name, fields, loc, ctx)
            }
            Stmt::FunctionDeclaration(name, args, body, returns, export, loc) => {
                visitor.func_declaration(name, args, body, returns, export, loc, ctx)
            }
            Stmt::Break(loc) => visitor.break_stmt(loc, ctx),
            Stmt::Return(expr, loc) => visitor.return_stmt(expr, loc, ctx),
        }
    }
}

impl Expr {
    pub fn accept(&self, visitor: &mut dyn Visitor, ctx: &mut Context) -> Result<ASTValue, String> {
        match self {
            Expr::Value(value, swizzle, field_path, loc) => {
                visitor.value(value.clone(), swizzle, field_path, loc, ctx)
            }
            Expr::Logical(left, op, right, loc) => visitor.logical_expr(left, op, right, loc, ctx),
            Expr::Unary(op, expr, loc) => visitor.unary(op, expr, loc, ctx),
            Expr::Equality(left, op, right, loc) => visitor.equality(left, op, right, loc, ctx),
            Expr::Comparison(left, op, right, loc) => visitor.comparison(left, op, right, loc, ctx),
            Expr::Binary(left, op, right, loc) => visitor.binary(left, op, right, loc, ctx),
            Expr::Grouping(expr, loc) => visitor.grouping(expr, loc, ctx),
            Expr::Variable(name, swizzle, field_path, loc) => {
                visitor.variable(name.clone(), swizzle, field_path, loc, ctx)
            }
            Expr::VariableAssignment(name, op, swizzle, field_path, expr, loc) => {
                visitor.variable_assignment(name.clone(), op, swizzle, field_path, expr, loc, ctx)
            }
            Expr::FunctionCall(callee, args, swizzle, field_path, loc) => {
                visitor.func_call(callee, args, swizzle, field_path, loc, ctx)
            }
            Expr::Ternary(cond, then_expr, else_expr, loc) => {
                visitor.ternary(cond, then_expr, else_expr, loc, ctx)
            }
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
