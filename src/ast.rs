use crate::token::Span;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Int(i64),
    Long(i64),
    Double(f64),
    Float(f64),
    Bool(bool),
    String(String),
    Char(char),
    Null,
    Unit,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinOp {
    Add, Sub, Mul, Div, Mod,
    BitAnd, BitOr, BitXor,
    LeftShift, RightShift, UnsignedRightShift,
    And, Or,
    Eq, Neq, Lt, Gt, Leq, Geq,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOp {
    Negate, Not, BitNot, Positive,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Param {
    pub name: String,
    pub type_ann: Option<TypeExpr>,
    pub default: Option<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TypeParam {
    pub name: String,
    pub variance: Variance,
    pub upper_bound: Option<TypeExpr>,
    pub lower_bound: Option<TypeExpr>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Variance {
    Invariant,
    Covariant,
    Contravariant,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Pattern {
    Wildcard(Span),
    Variable { name: String, span: Span },
    Literal { value: Literal, span: Span },
    Constructor { name: String, args: Vec<Pattern>, span: Span },
    Tuple { elements: Vec<Pattern>, span: Span },
    Typed { pattern: Box<Pattern>, type_ann: TypeExpr, span: Span },
    Alternative { left: Box<Pattern>, right: Box<Pattern>, span: Span },
    SequenceWildcard(Span),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Literal { value: Literal, span: Span },
    Binary { left: Box<Expr>, op: BinOp, right: Box<Expr>, span: Span },
    Unary { op: UnaryOp, operand: Box<Expr>, span: Span },
    If { cond: Box<Expr>, then_branch: Box<Expr>, else_branch: Option<Box<Expr>>, span: Span },
    Block { stmts: Vec<Stmt>, span: Span },
    Lambda { params: Vec<Param>, body: Box<Expr>, span: Span },
    Apply { func: Box<Expr>, args: Vec<Expr>, span: Span },
    MethodCall { receiver: Box<Expr>, method: String, args: Vec<Expr>, span: Span },
    FieldAccess { receiver: Box<Expr>, field: String, span: Span },
    Match { scrutinee: Box<Expr>, cases: Vec<MatchCase>, span: Span },
    Tuple { elements: Vec<Expr>, span: Span },
    Assign { target: Box<Expr>, value: Box<Expr>, span: Span },
    Return { value: Option<Box<Expr>>, span: Span },
    Throw { value: Box<Expr>, span: Span },
    Try { body: Box<Expr>, catches: Vec<MatchCase>, finally_block: Option<Box<Expr>>, span: Span },
    New { class_name: String, type_args: Vec<TypeExpr>, args: Vec<Expr>, span: Span },
    For { enumerators: Vec<Enumerator>, body: Box<Expr>, is_yield: bool, span: Span },
    While { cond: Box<Expr>, body: Box<Expr>, span: Span },
    DoWhile { body: Box<Expr>, cond: Box<Expr>, span: Span },
    StringInterpolation { prefix: String, parts: Vec<InterpPart>, span: Span },
    This(Span),
    Super(Span),
    Identifier { name: String, span: Span },
    Paren { expr: Box<Expr>, span: Span },
    TypeApply { expr: Box<Expr>, type_args: Vec<TypeExpr>, span: Span },
    UnaryMethodCall { receiver: Box<Expr>, method: String, span: Span },
}

#[derive(Debug, Clone, PartialEq)]
pub enum InterpPart {
    Literal(String),
    Expression(Expr),
}

#[derive(Debug, Clone, PartialEq)]
pub struct MatchCase {
    pub pattern: Pattern,
    pub guard: Option<Expr>,
    pub body: Expr,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Enumerator {
    Generator { pattern: Pattern, expr: Expr, span: Span },
    Filter { cond: Expr, span: Span },
    Val { pattern: Pattern, expr: Expr, span: Span },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Expr(Expr),
    ValDecl { pattern: Pattern, type_ann: Option<TypeExpr>, value: Expr, span: Span },
    VarDecl { pattern: Pattern, type_ann: Option<TypeExpr>, value: Expr, span: Span },
    DefDecl(DefDecl),
    ClassDecl(ClassDecl),
    TraitDecl(TraitDecl),
    ObjectDecl(ObjectDecl),
    TypeDecl { name: String, type_params: Vec<TypeParam>, rhs: TypeExpr, span: Span },
    ImportDecl { path: Vec<String>, selectors: ImportSelectors, span: Span },
}

#[derive(Debug, Clone, PartialEq)]
pub enum ImportSelectors {
    All,
    Names(Vec<String>),
    Rename(Vec<(String, String)>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefDecl {
    pub name: String,
    pub type_params: Vec<TypeParam>,
    pub params: Vec<Param>,
    pub return_type: Option<TypeExpr>,
    pub body: Expr,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClassDecl {
    pub name: String,
    pub type_params: Vec<TypeParam>,
    pub ctor_params: Vec<Param>,
    pub parents: Vec<(String, Vec<Expr>)>,
    pub body: Vec<Stmt>,
    pub is_case: bool,
    pub is_abstract: bool,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TraitDecl {
    pub name: String,
    pub type_params: Vec<TypeParam>,
    pub parents: Vec<String>,
    pub body: Vec<Stmt>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ObjectDecl {
    pub name: String,
    pub parents: Vec<(String, Vec<Expr>)>,
    pub body: Vec<Stmt>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TypeExpr {
    Simple { name: String, span: Span },
    Parameterized { base: Box<TypeExpr>, args: Vec<TypeExpr>, span: Span },
    Function { params: Vec<TypeExpr>, result: Box<TypeExpr>, span: Span },
    Tuple { elements: Vec<TypeExpr>, span: Span },
    Compound { types: Vec<TypeExpr>, span: Span },
    Wildcard { upper: Option<Box<TypeExpr>>, lower: Option<Box<TypeExpr>>, span: Span },
}

impl TypeExpr {
    pub fn simple(name: &str) -> Self {
        TypeExpr::Simple { name: name.to_string(), span: Span::zero() }
    }
}

impl fmt::Display for Literal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Literal::Int(v) => write!(f, "{}", v),
            Literal::Long(v) => write!(f, "{}L", v),
            Literal::Double(v) => write!(f, "{}", v),
            Literal::Float(v) => write!(f, "{}f", v),
            Literal::Bool(b) => write!(f, "{}", b),
            Literal::String(s) => write!(f, "\"{}\"", s),
            Literal::Char(c) => write!(f, "'{}'", c),
            Literal::Null => write!(f, "null"),
            Literal::Unit => write!(f, "()"),
        }
    }
}

impl fmt::Display for BinOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BinOp::Add => write!(f, "+"),
            BinOp::Sub => write!(f, "-"),
            BinOp::Mul => write!(f, "*"),
            BinOp::Div => write!(f, "/"),
            BinOp::Mod => write!(f, "%"),
            BinOp::BitAnd => write!(f, "&"),
            BinOp::BitOr => write!(f, "|"),
            BinOp::BitXor => write!(f, "^"),
            BinOp::LeftShift => write!(f, "<<"),
            BinOp::RightShift => write!(f, ">>"),
            BinOp::UnsignedRightShift => write!(f, ">>>"),
            BinOp::And => write!(f, "&&"),
            BinOp::Or => write!(f, "||"),
            BinOp::Eq => write!(f, "=="),
            BinOp::Neq => write!(f, "!="),
            BinOp::Lt => write!(f, "<"),
            BinOp::Gt => write!(f, ">"),
            BinOp::Leq => write!(f, "<="),
            BinOp::Geq => write!(f, ">="),
        }
    }
}

impl fmt::Display for TypeExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TypeExpr::Simple { name, .. } => write!(f, "{}", name),
            TypeExpr::Parameterized { base, args, .. } => {
                write!(f, "{}[", base)?;
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}", arg)?;
                }
                write!(f, "]")
            }
            TypeExpr::Function { params, result, .. } => {
                write!(f, "(")?;
                for (i, p) in params.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}", p)?;
                }
                write!(f, ") => {}", result)
            }
            TypeExpr::Tuple { elements, .. } => {
                write!(f, "(")?;
                for (i, e) in elements.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}", e)?;
                }
                write!(f, ")")
            }
            TypeExpr::Compound { types, .. } => {
                for (i, t) in types.iter().enumerate() {
                    if i > 0 { write!(f, " with ")?; }
                    write!(f, "{}", t)?;
                }
                Ok(())
            }
            TypeExpr::Wildcard { .. } => write!(f, "_"),
        }
    }
}

pub fn indent(n: usize) -> String {
    "  ".repeat(n)
}

pub fn fmt_expr(expr: &Expr, depth: usize) -> String {
    match expr {
        Expr::Literal { value, .. } => format!("{}{}", indent(depth), value),
        Expr::Binary { left, op, right, .. } => {
            format!("{}({} {} {})", indent(depth), fmt_expr(left, 0), op, fmt_expr(right, 0))
        }
        Expr::Unary { op, operand, .. } => {
            format!("{}({}{})", indent(depth), match op {
                UnaryOp::Negate => "-",
                UnaryOp::Not => "!",
                UnaryOp::BitNot => "~",
                UnaryOp::Positive => "+",
            }, fmt_expr(operand, 0))
        }
        Expr::If { cond, then_branch, else_branch, .. } => {
            let mut s = format!("{}if ({})\n", indent(depth), fmt_expr(cond, 0));
            s.push_str(&fmt_expr(then_branch, depth + 1));
            if let Some(els) = else_branch {
                s.push_str(&format!("\n{}else\n", indent(depth)));
                s.push_str(&fmt_expr(els, depth + 1));
            }
            s
        }
        Expr::Block { stmts, .. } => {
            let mut s = format!("{}{{\n", indent(depth));
            for stmt in stmts {
                s.push_str(&fmt_stmt(stmt, depth + 1));
                s.push('\n');
            }
            s.push_str(&format!("{}}}", indent(depth)));
            s
        }
        Expr::Lambda { params, body, .. } => {
            let ps: Vec<String> = params.iter().map(|p| p.name.clone()).collect();
            format!("{}({}) => {}", indent(depth), ps.join(", "), fmt_expr(body, 0))
        }
        Expr::Identifier { name, .. } => format!("{}{}", indent(depth), name),
        Expr::Apply { func, args, .. } => {
            let as_: Vec<String> = args.iter().map(|a| fmt_expr(a, 0)).collect();
            format!("{}{}({})", indent(depth), fmt_expr(func, 0), as_.join(", "))
        }
        Expr::MethodCall { receiver, method, args, .. } => {
            let as_: Vec<String> = args.iter().map(|a| fmt_expr(a, 0)).collect();
            format!("{}{}.{}({})", indent(depth), fmt_expr(receiver, 0), method, as_.join(", "))
        }
        Expr::FieldAccess { receiver, field, .. } => {
            format!("{}{}.{}", indent(depth), fmt_expr(receiver, 0), field)
        }
        Expr::Tuple { elements, .. } => {
            let es: Vec<String> = elements.iter().map(|e| fmt_expr(e, 0)).collect();
            format!("{}({})", indent(depth), es.join(", "))
        }
        Expr::Match { scrutinee, cases, .. } => {
            let mut s = format!("{}{} match {{\n", indent(depth), fmt_expr(scrutinee, 0));
            for case in cases {
                s.push_str(&format!("{}  case {} => {}\n", indent(depth), fmt_pattern(&case.pattern), fmt_expr(&case.body, 0)));
            }
            s.push_str(&format!("{}}}", indent(depth)));
            s
        }
        Expr::New { class_name, args, .. } => {
            let as_: Vec<String> = args.iter().map(|a| fmt_expr(a, 0)).collect();
            format!("{}new {}({})", indent(depth), class_name, as_.join(", "))
        }
        Expr::This(_) => format!("{}this", indent(depth)),
        Expr::Super(_) => format!("{}super", indent(depth)),
        Expr::While { cond, body, .. } => {
            format!("{}while ({}) {}", indent(depth), fmt_expr(cond, 0), fmt_expr(body, 0))
        }
        Expr::Assign { target, value, .. } => {
            format!("{}{} = {}", indent(depth), fmt_expr(target, 0), fmt_expr(value, 0))
        }
        Expr::Return { value, .. } => {
            match value {
                Some(v) => format!("{}return {}", indent(depth), fmt_expr(v, 0)),
                None => format!("{}return", indent(depth)),
            }
        }
        Expr::Throw { value, .. } => format!("{}throw {}", indent(depth), fmt_expr(value, 0)),
        Expr::Paren { expr, .. } => format!("({})", fmt_expr(expr, 0)),
        _ => format!("{}<???>", indent(depth)),
    }
}

pub fn fmt_pattern(pat: &Pattern) -> String {
    match pat {
        Pattern::Wildcard(_) => "_".into(),
        Pattern::Variable { name, .. } => name.clone(),
        Pattern::Literal { value, .. } => value.to_string(),
        Pattern::Constructor { name, args, .. } => {
            let ps: Vec<String> = args.iter().map(fmt_pattern).collect();
            format!("{}({})", name, ps.join(", "))
        }
        Pattern::Tuple { elements, .. } => {
            let ps: Vec<String> = elements.iter().map(fmt_pattern).collect();
            format!("({})", ps.join(", "))
        }
        Pattern::Typed { pattern, type_ann, .. } => format!("{}: {}", fmt_pattern(pattern), type_ann),
        Pattern::Alternative { left, right, .. } => format!("{} | {}", fmt_pattern(left), fmt_pattern(right)),
        Pattern::SequenceWildcard(_) => "_*".into(),
    }
}

pub fn fmt_stmt(stmt: &Stmt, depth: usize) -> String {
    match stmt {
        Stmt::Expr(e) => fmt_expr(e, depth),
        Stmt::ValDecl { pattern, type_ann, value, .. } => {
            match type_ann {
                Some(t) => format!("{}val {}: {} = {}", indent(depth), fmt_pattern(pattern), t, fmt_expr(value, 0)),
                None => format!("{}val {} = {}", indent(depth), fmt_pattern(pattern), fmt_expr(value, 0)),
            }
        }
        Stmt::VarDecl { pattern, type_ann, value, .. } => {
            match type_ann {
                Some(t) => format!("{}var {}: {} = {}", indent(depth), fmt_pattern(pattern), t, fmt_expr(value, 0)),
                None => format!("{}var {} = {}", indent(depth), fmt_pattern(pattern), fmt_expr(value, 0)),
            }
        }
        Stmt::DefDecl(d) => {
            let ps: Vec<String> = d.params.iter().map(|p| {
                match &p.type_ann {
                    Some(t) => format!("{}: {}", p.name, t),
                    None => p.name.clone(),
                }
            }).collect();
            match &d.return_type {
                Some(t) => format!("{}def {}({}): {} = {}", indent(depth), d.name, ps.join(", "), t, fmt_expr(&d.body, 0)),
                None => format!("{}def {}({}) = {}", indent(depth), d.name, ps.join(", "), fmt_expr(&d.body, 0)),
            }
        }
        Stmt::ClassDecl(c) => {
            let ps: Vec<String> = c.ctor_params.iter().map(|p| p.name.clone()).collect();
            let mut s = if c.is_case {
                format!("{}case class {}({})", indent(depth), c.name, ps.join(", "))
            } else {
                format!("{}class {}({})", indent(depth), c.name, ps.join(", "))
            };
            if !c.body.is_empty() {
                s.push_str(" {\n");
                for st in &c.body {
                    s.push_str(&fmt_stmt(st, depth + 1));
                    s.push('\n');
                }
                s.push_str(&format!("{}}}", indent(depth)));
            }
            s
        }
        Stmt::TraitDecl(t) => {
            let mut s = format!("{}trait {}", indent(depth), t.name);
            if !t.body.is_empty() {
                s.push_str(" {\n");
                for st in &t.body {
                    s.push_str(&fmt_stmt(st, depth + 1));
                    s.push('\n');
                }
                s.push_str(&format!("{}}}", indent(depth)));
            }
            s
        }
        Stmt::ObjectDecl(o) => {
            let mut s = format!("{}object {}", indent(depth), o.name);
            if !o.body.is_empty() {
                s.push_str(" {\n");
                for st in &o.body {
                    s.push_str(&fmt_stmt(st, depth + 1));
                    s.push('\n');
                }
                s.push_str(&format!("{}}}", indent(depth)));
            }
            s
        }
        Stmt::ImportDecl { path, selectors, .. } => {
            let p = path.join(".");
            match selectors {
                ImportSelectors::All => format!("{}import {}._", indent(depth), p),
                ImportSelectors::Names(names) => format!("{}import {}.{{{}}}", indent(depth), p, names.join(", ")),
                ImportSelectors::Rename(pairs) => {
                    let rs: Vec<String> = pairs.iter().map(|(a, b)| format!("{} => {}", a, b)).collect();
                    format!("{}import {}.{{{}}}", indent(depth), p, rs.join(", "))
                }
            }
        }
        Stmt::TypeDecl { name, rhs, .. } => {
            format!("{}type {} = {}", indent(depth), name, rhs)
        }
    }
}
