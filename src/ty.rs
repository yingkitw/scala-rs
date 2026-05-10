use std::fmt;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum Ty {
    Int,
    Long,
    Double,
    Float,
    Bool,
    Char,
    String,
    Unit,
    Any,
    AnyVal,
    AnyRef,
    Nothing,
    Null,

    Function { params: Vec<Ty>, result: Box<Ty> },
    Tuple { elements: Vec<Ty> },
    Named { name: String, args: Vec<Ty> },
    Param { name: String },
    App { base: Box<Ty>, args: Vec<Ty> },

    Error(String),
}

impl Ty {
    pub fn is_numeric(&self) -> bool {
        matches!(self, Ty::Int | Ty::Long | Ty::Double | Ty::Float | Ty::Char)
    }

    pub fn is_error(&self) -> bool {
        matches!(self, Ty::Error(_))
    }

    pub fn is_subtype_of(&self, other: &Ty) -> bool {
        if self == other {
            return true;
        }
        match (self, other) {
            (_, Ty::Any) => true,
            (Ty::Nothing, _) => true,
            (Ty::Null, Ty::AnyRef) => true,
            (Ty::Null, Ty::Named { .. }) => true,
            (Ty::Int, Ty::Long) => true,
            (Ty::Int, Ty::Double) => true,
            (Ty::Int, Ty::Float) => true,
            (Ty::Long, Ty::Double) => true,
            (Ty::Float, Ty::Double) => true,
            (Ty::Char, Ty::Int) => true,
            (_, Ty::AnyVal) if self.is_numeric() || matches!(self, Ty::Bool | Ty::Unit) => true,
            (Ty::Bool, Ty::AnyVal) => true,
            (Ty::Unit, Ty::AnyVal) => true,
            (Ty::String, Ty::AnyRef) => true,
            (Ty::Named { name: _, .. }, Ty::AnyRef) => true,
            (
                Ty::Function { params: p1, result: r1 },
                Ty::Function { params: p2, result: r2 },
            ) => {
                if p1.len() != p2.len() {
                    return false;
                }
                p2.iter().zip(p1.iter()).all(|(expected, actual)| actual.is_subtype_of(expected))
                    && r1.is_subtype_of(r2)
            }
            (Ty::Error(_), _) | (_, Ty::Error(_)) => true,
            _ => false,
        }
    }

    pub fn common_type(&self, other: &Ty) -> Ty {
        if self.is_subtype_of(other) {
            return other.clone();
        }
        if other.is_subtype_of(self) {
            return self.clone();
        }
        Ty::Any
    }

    pub fn from_name(name: &str) -> Ty {
        match name {
            "Int" => Ty::Int,
            "Long" => Ty::Long,
            "Double" => Ty::Double,
            "Float" => Ty::Float,
            "Boolean" => Ty::Bool,
            "Char" => Ty::Char,
            "String" => Ty::String,
            "Unit" => Ty::Unit,
            "Any" => Ty::Any,
            "AnyVal" => Ty::AnyVal,
            "AnyRef" => Ty::AnyRef,
            "Nothing" => Ty::Nothing,
            "Null" => Ty::Null,
            other => Ty::Named { name: other.to_string(), args: vec![] },
        }
    }
}

impl fmt::Display for Ty {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Ty::Int => write!(f, "Int"),
            Ty::Long => write!(f, "Long"),
            Ty::Double => write!(f, "Double"),
            Ty::Float => write!(f, "Float"),
            Ty::Bool => write!(f, "Boolean"),
            Ty::Char => write!(f, "Char"),
            Ty::String => write!(f, "String"),
            Ty::Unit => write!(f, "Unit"),
            Ty::Any => write!(f, "Any"),
            Ty::AnyVal => write!(f, "AnyVal"),
            Ty::AnyRef => write!(f, "AnyRef"),
            Ty::Nothing => write!(f, "Nothing"),
            Ty::Null => write!(f, "Null"),
            Ty::Function { params, result } => {
                write!(f, "(")?;
                for (i, p) in params.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}", p)?;
                }
                write!(f, ") => {}", result)
            }
            Ty::Tuple { elements } => {
                write!(f, "(")?;
                for (i, e) in elements.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}", e)?;
                }
                write!(f, ")")
            }
            Ty::Named { name, args } => {
                write!(f, "{}", name)?;
                if !args.is_empty() {
                    write!(f, "[")?;
                    for (i, a) in args.iter().enumerate() {
                        if i > 0 { write!(f, ", ")?; }
                        write!(f, "{}", a)?;
                    }
                    write!(f, "]")?;
                }
                Ok(())
            }
            Ty::Param { name } => write!(f, "{}", name),
            Ty::App { base, args } => {
                write!(f, "{}[", base)?;
                for (i, a) in args.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}", a)?;
                }
                write!(f, "]")
            }
            Ty::Error(msg) => write!(f, "Error({})", msg),
        }
    }
}

#[derive(Debug, Clone)]
pub struct TypeEnv {
    scopes: Vec<HashMap<String, Ty>>,
}

impl TypeEnv {
    pub fn new() -> Self {
        TypeEnv {
            scopes: vec![HashMap::new()],
        }
    }

    pub fn push(&mut self) {
        self.scopes.push(HashMap::new());
    }

    pub fn pop(&mut self) {
        if self.scopes.len() > 1 {
            self.scopes.pop();
        }
    }

    pub fn define(&mut self, name: &str, ty: Ty) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name.to_string(), ty);
        }
    }

    pub fn lookup(&self, name: &str) -> Option<&Ty> {
        for scope in self.scopes.iter().rev() {
            if let Some(ty) = scope.get(name) {
                return Some(ty);
            }
        }
        None
    }

    pub fn define_builtin_types(&mut self) {
        self.define("Int", Ty::Int);
        self.define("Long", Ty::Long);
        self.define("Double", Ty::Double);
        self.define("Float", Ty::Float);
        self.define("Boolean", Ty::Bool);
        self.define("Char", Ty::Char);
        self.define("String", Ty::String);
        self.define("Unit", Ty::Unit);
        self.define("Any", Ty::Any);
        self.define("AnyVal", Ty::AnyVal);
        self.define("AnyRef", Ty::AnyRef);
        self.define("Nothing", Ty::Nothing);
        self.define("Null", Ty::Null);
        self.define("println", Ty::Function {
            params: vec![Ty::Any],
            result: Box::new(Ty::Unit),
        });
        self.define("print", Ty::Function {
            params: vec![Ty::Any],
            result: Box::new(Ty::Unit),
        });
        self.define("assert", Ty::Function {
            params: vec![Ty::Bool],
            result: Box::new(Ty::Unit),
        });
        self.define("require", Ty::Function {
            params: vec![Ty::Bool],
            result: Box::new(Ty::Unit),
        });
        self.define("List", Ty::Named { name: "List".into(), args: vec![] });
        self.define("Map", Ty::Named { name: "Map".into(), args: vec![] });
        self.define("Option", Ty::Named { name: "Option".into(), args: vec![] });
        self.define("Some", Ty::Named { name: "Some".into(), args: vec![] });
        self.define("None", Ty::Named { name: "None".into(), args: vec![] });
        self.define("Tuple2", Ty::Named { name: "Tuple2".into(), args: vec![] });
        self.define("Tuple3", Ty::Named { name: "Tuple3".into(), args: vec![] });
    }
}

#[derive(Debug, Clone)]
pub struct TypeError {
    pub message: String,
    pub span: crate::token::Span,
}

impl TypeError {
    pub fn new(message: impl Into<String>, span: crate::token::Span) -> Self {
        TypeError { message: message.into(), span }
    }
}

impl std::fmt::Display for TypeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "type error at {}: {}", self.span, self.message)
    }
}

impl std::error::Error for TypeError {}
