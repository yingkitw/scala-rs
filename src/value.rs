use std::fmt;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum Value {
    Int(i64),
    Long(i64),
    Double(f64),
    Float(f64),
    Bool(bool),
    Char(char),
    String(String),
    Unit,
    Null,
    Tuple(Vec<Value>),
    List(Vec<Value>),
    Map(Vec<(Value, Value)>),
    Function {
        name: Option<String>,
        params: Vec<String>,
        body: crate::ast::Expr,
        closure: Vec<(String, Value, bool)>,
    },
    BuiltinFunction {
        name: String,
        arity: usize,
    },
    Object {
        class_name: String,
        fields: HashMap<String, Value>,
        methods: HashMap<String, Value>,
    },
    Array(Vec<Value>),
    Nothing,
}

impl Value {
    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Int(_) => "Int",
            Value::Long(_) => "Long",
            Value::Double(_) => "Double",
            Value::Float(_) => "Float",
            Value::Bool(_) => "Boolean",
            Value::Char(_) => "Char",
            Value::String(_) => "String",
            Value::Unit => "Unit",
            Value::Null => "Null",
            Value::Tuple(_) => "Tuple",
            Value::List(_) => "List",
            Value::Map(_) => "Map",
            Value::Function { .. } => "Function",
            Value::BuiltinFunction { .. } => "Function",
            Value::Object { .. } => "Object",
            Value::Array(_) => "Array",
            Value::Nothing => "Nothing",
        }
    }

    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Bool(b) => *b,
            Value::Null => false,
            Value::Int(n) => *n != 0,
            Value::Unit => false,
            _ => true,
        }
    }

    pub fn to_int(&self) -> Option<i64> {
        match self {
            Value::Int(n) => Some(*n),
            Value::Long(n) => Some(*n),
            Value::Double(n) => Some(*n as i64),
            Value::Float(n) => Some(*n as i64),
            Value::Char(c) => Some(*c as i64),
            Value::Bool(b) => Some(if *b { 1 } else { 0 }),
            _ => None,
        }
    }

    pub fn to_double(&self) -> Option<f64> {
        match self {
            Value::Int(n) => Some(*n as f64),
            Value::Long(n) => Some(*n as f64),
            Value::Double(n) => Some(*n),
            Value::Float(n) => Some(*n as f64),
            _ => None,
        }
    }

    pub fn deep_clone(&self) -> Value {
        self.clone()
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => a == b,
            (Value::Long(a), Value::Long(b)) => a == b,
            (Value::Double(a), Value::Double(b)) => a == b,
            (Value::Float(a), Value::Float(b)) => a == b,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Char(a), Value::Char(b)) => a == b,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Unit, Value::Unit) => true,
            (Value::Null, Value::Null) => true,
            (Value::Tuple(a), Value::Tuple(b)) => a == b,
            (Value::List(a), Value::List(b)) => a == b,
            (Value::Int(a), Value::Long(b)) => a == b,
            (Value::Long(a), Value::Int(b)) => a == b,
            _ => false,
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Int(v) => write!(f, "{}", v),
            Value::Long(v) => write!(f, "{}", v),
            Value::Double(v) => {
                if v.fract() == 0.0 {
                    write!(f, "{:.1}", v)
                } else {
                    write!(f, "{}", v)
                }
            }
            Value::Float(v) => write!(f, "{}", v),
            Value::Bool(v) => write!(f, "{}", v),
            Value::Char(v) => write!(f, "{}", v),
            Value::String(v) => write!(f, "{}", v),
            Value::Unit => write!(f, "()"),
            Value::Null => write!(f, "null"),
            Value::Tuple(elements) => {
                write!(f, "(")?;
                for (i, e) in elements.iter().enumerate() {
                    if i > 0 { write!(f, ",")?; }
                    write!(f, "{}", e)?;
                }
                write!(f, ")")
            }
            Value::List(elements) => {
                write!(f, "List(")?;
                for (i, e) in elements.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}", e)?;
                }
                write!(f, ")")
            }
            Value::Map(entries) => {
                write!(f, "Map(")?;
                for (i, (k, v)) in entries.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{} -> {}", k, v)?;
                }
                write!(f, ")")
            }
            Value::Function { name, .. } => {
                match name {
                    Some(n) => write!(f, "<function {}>", n),
                    None => write!(f, "<function>"),
                }
            }
            Value::BuiltinFunction { name, .. } => write!(f, "<builtin {}>", name),
            Value::Object { class_name, fields, .. } => {
                write!(f, "{}(", class_name)?;
                let field_list: Vec<String> = fields.iter()
                    .map(|(k, v)| format!("{} = {}", k, v))
                    .collect();
                write!(f, "{})", field_list.join(", "))
            }
            Value::Array(elements) => {
                write!(f, "Array(")?;
                for (i, e) in elements.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}", e)?;
                }
                write!(f, ")")
            }
            Value::Nothing => write!(f, "<nothing>"),
        }
    }
}
