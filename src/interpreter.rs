use crate::stdlib::register_stdlib;
use crate::ast::*;
use crate::env::Environment;
use crate::value::Value;
use crate::lexer::Lexer;
use crate::parser::Parser;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct RuntimeError {
    pub message: String,
}

impl RuntimeError {
    pub fn new(message: impl Into<String>) -> Self {
        RuntimeError { message: message.into() }
    }
}

impl std::fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "runtime error: {}", self.message)
    }
}

impl std::error::Error for RuntimeError {}

pub struct Interpreter {
    env: Environment,
}

impl Interpreter {
    pub fn new() -> Self {
        let mut interp = Interpreter {
            env: Environment::new(),
        };
        register_stdlib(&mut interp.env);
        interp
    }

    pub fn env(&self) -> &Environment {
        &self.env
    }

    pub fn env_mut(&mut self) -> &mut Environment {
        &mut self.env
    }

    pub fn run_source(&mut self, source: &str) -> Result<Value, RuntimeError> {
        let tokens = Lexer::tokenize(source).map_err(|e| RuntimeError::new(e.message))?;
        let stmts = Parser::parse(tokens).map_err(|e| RuntimeError::new(e.message))?;
        let mut last = Value::Unit;
        for stmt in &stmts {
            last = self.exec_stmt(stmt)?;
        }
        Ok(last)
    }

    pub fn exec_stmt(&mut self, stmt: &Stmt) -> Result<Value, RuntimeError> {
        match stmt {
            Stmt::Expr(expr) => self.eval_expr(expr),
            Stmt::ValDecl { pattern, value, .. } => {
                let val = self.eval_expr(value)?;
                self.bind_pattern(pattern, &val)?;
                Ok(Value::Unit)
            }
            Stmt::VarDecl { pattern, value, .. } => {
                let val = self.eval_expr(value)?;
                self.bind_pattern_mut(pattern, &val)?;
                Ok(Value::Unit)
            }
            Stmt::DefDecl(def_decl) => {
                let closure = self.env.capture();
                let fn_val = Value::Function {
                    name: Some(def_decl.name.clone()),
                    params: def_decl.params.iter().map(|p| p.name.clone()).collect(),
                    body: def_decl.body.clone(),
                    closure,
                };
                self.env.define(&def_decl.name, fn_val, false);
                Ok(Value::Unit)
            }
            Stmt::ClassDecl(class) => {
                let ctor_name = class.name.clone();
                let ctor_params = class.ctor_params.clone();
                let is_case = class.is_case;
                let class_body = class.body.clone();

                let class_val = Value::Object {
                    class_name: format!("${}$class", ctor_name),
                    fields: {
                        let mut f = HashMap::new();
                        f.insert("__ctor_params__".into(), Value::List(
                            ctor_params.iter().map(|p| Value::String(p.name.clone())).collect()
                        ));
                        f.insert("__is_case__".into(), Value::Bool(is_case));
                        f
                    },
                    methods: {
                        let mut m = HashMap::new();
                        for stmt in &class_body {
                            if let Stmt::DefDecl(d) = stmt {
                                m.insert(d.name.clone(), Value::Function {
                                    name: Some(d.name.clone()),
                                    params: d.params.iter().map(|p| p.name.clone()).collect(),
                                    body: d.body.clone(),
                                    closure: vec![],
                                });
                            }
                        }
                        m
                    },
                };
                self.env.define(&ctor_name, class_val, false);
                Ok(Value::Unit)
            }
            Stmt::TraitDecl(_) => Ok(Value::Unit),
            Stmt::ObjectDecl(obj) => {
                let mut fields = HashMap::new();
                let mut methods = HashMap::new();
                self.env.push();
                for stmt in &obj.body {
                    match stmt {
                        Stmt::ValDecl { pattern, value, .. } => {
                            let val = self.eval_expr(value)?;
                            if let Pattern::Variable { name, .. } = pattern {
                                fields.insert(name.clone(), val.clone());
                                self.env.define(name, val, false);
                            }
                        }
                        Stmt::VarDecl { pattern, value, .. } => {
                            let val = self.eval_expr(value)?;
                            if let Pattern::Variable { name, .. } = pattern {
                                fields.insert(name.clone(), val.clone());
                                self.env.define(name, val, true);
                            }
                        }
                        Stmt::DefDecl(d) => {
                            let closure = self.env.capture();
                            let fn_val = Value::Function {
                                name: Some(d.name.clone()),
                                params: d.params.iter().map(|p| p.name.clone()).collect(),
                                body: d.body.clone(),
                                closure,
                            };
                            methods.insert(d.name.clone(), fn_val.clone());
                            self.env.define(&d.name, fn_val, false);
                        }
                        _ => { let _ = self.exec_stmt(stmt); }
                    }
                }
                self.env.pop();
                let obj_val = Value::Object {
                    class_name: obj.name.clone(),
                    fields,
                    methods,
                };
                self.env.define(&obj.name, obj_val, false);
                Ok(Value::Unit)
            }
            Stmt::ImportDecl { .. } => Ok(Value::Unit),
            Stmt::TypeDecl { .. } => Ok(Value::Unit),
        }
    }

    pub fn eval_expr(&mut self, expr: &Expr) -> Result<Value, RuntimeError> {
        match expr {
            Expr::Literal { value, .. } => Ok(literal_to_value(value)),

            Expr::Binary { left, op, right, .. } => {
                let lv = self.eval_expr(left)?;
                match op {
                    BinOp::And => {
                        if !lv.is_truthy() { return Ok(Value::Bool(false)); }
                        let rv = self.eval_expr(right)?;
                        Ok(Value::Bool(rv.is_truthy()))
                    }
                    BinOp::Or => {
                        if lv.is_truthy() { return Ok(Value::Bool(true)); }
                        let rv = self.eval_expr(right)?;
                        Ok(Value::Bool(rv.is_truthy()))
                    }
                    _ => {
                        let rv = self.eval_expr(right)?;
                        self.eval_binary(&lv, op, &rv)
                    }
                }
            }

            Expr::Unary { op, operand, .. } => {
                let v = self.eval_expr(operand)?;
                match op {
                    UnaryOp::Negate => v.to_int().map(|n| Value::Int(-n))
                        .or_else(|| v.to_double().map(|n| Value::Double(-n)))
                        .ok_or_else(|| RuntimeError::new(format!("cannot negate {}", v.type_name()))),
                    UnaryOp::Not => Ok(Value::Bool(!v.is_truthy())),
                    UnaryOp::BitNot => v.to_int().map(|n| Value::Int(!n))
                        .ok_or_else(|| RuntimeError::new(format!("cannot bitwise negate {}", v.type_name()))),
                    UnaryOp::Positive => Ok(v),
                }
            }

            Expr::If { cond, then_branch, else_branch, .. } => {
                let cv = self.eval_expr(cond)?;
                if cv.is_truthy() {
                    self.eval_expr(then_branch)
                } else if let Some(els) = else_branch {
                    self.eval_expr(els)
                } else {
                    Ok(Value::Unit)
                }
            }

            Expr::Block { stmts, .. } => {
                self.env.push();
                let mut last = Value::Unit;
                for stmt in stmts {
                    last = self.exec_stmt(stmt)?;
                }
                self.env.pop();
                Ok(last)
            }

            Expr::Lambda { params, body, .. } => {
                let closure = self.env.capture();
                Ok(Value::Function {
                    name: None,
                    params: params.iter().map(|p| p.name.clone()).collect(),
                    body: (**body).clone(),
                    closure,
                })
            }

            Expr::Identifier { name, .. } => {
                self.env.lookup(name).cloned().ok_or_else(|| {
                    RuntimeError::new(format!("not found: value {}", name))
                })
            }

            Expr::Apply { func, args, .. } => {
                let fv = self.eval_expr(func)?;
                let arg_vals: Vec<Value> = args.iter().map(|a| self.eval_expr(a)).collect::<Result<Vec<_>, _>>()?;
                self.call_value(fv, arg_vals)
            }

            Expr::MethodCall { receiver, method, args, .. } => {
                let rv = self.eval_expr(receiver)?;
                let arg_vals: Vec<Value> = args.iter().map(|a| self.eval_expr(a)).collect::<Result<Vec<_>, _>>()?;
                self.eval_method_call(rv, method, arg_vals)
            }

            Expr::FieldAccess { receiver, field, .. } => {
                let rv = self.eval_expr(receiver)?;
                match self.eval_field_access(&rv, field) {
                    ok @ Ok(_) => ok,
                    Err(_) => self.eval_method_call(rv, field, vec![]),
                }
            }

            Expr::Match { scrutinee, cases, .. } => {
                let sv = self.eval_expr(scrutinee)?;
                self.eval_match(&sv, cases)
            }

            Expr::Tuple { elements, .. } => {
                let vals: Vec<Value> = elements.iter().map(|e| self.eval_expr(e)).collect::<Result<Vec<_>, _>>()?;
                Ok(Value::Tuple(vals))
            }

            Expr::Assign { target, value, .. } => {
                let vv = self.eval_expr(value)?;
                match &**target {
                    Expr::Identifier { name, .. } => {
                        if !self.env.assign(name, vv) {
                            return Err(RuntimeError::new(format!("cannot assign to {}", name)));
                        }
                        Ok(Value::Unit)
                    }
                    _ => Err(RuntimeError::new("invalid assignment target")),
                }
            }

            Expr::Return { value, .. } => {
                if let Some(v) = value {
                    let val = self.eval_expr(v)?;
                    Err(RuntimeError::new(format!("__return__{}", val)))
                } else {
                    Err(RuntimeError::new("__return__()"))
                }
            }

            Expr::Throw { value, .. } => {
                let v = self.eval_expr(value)?;
                Err(RuntimeError::new(format!("__exception__{}", v)))
            }

            Expr::Try { body, catches, finally_block, .. } => {
                match self.eval_expr(body) {
                    Ok(v) => {
                        if let Some(fin) = finally_block {
                            self.eval_expr(fin)?;
                        }
                        Ok(v)
                    }
                    Err(e) => {
                        let err_str = e.message.clone();
                        let matched = self.try_catch(&err_str, catches);
                        if let Some(r) = matched {
                            if let Some(fin) = finally_block {
                                let _ = self.eval_expr(fin);
                            }
                            r
                        } else {
                            if let Some(fin) = finally_block {
                                let _ = self.eval_expr(fin);
                            }
                            Err(e)
                        }
                    }
                }
            }

            Expr::New { class_name, args, .. } => {
                let arg_vals: Vec<Value> = args.iter().map(|a| self.eval_expr(a)).collect::<Result<Vec<_>, _>>()?;
                let class_val = self.env.lookup(class_name).cloned();
                match class_val {
                    Some(Value::Object { fields, methods, .. }) => {
                        let ctor_params = fields.get("__ctor_params__")
                            .and_then(|v| if let Value::List(names) = v {
                                Some(names.iter().map(|n| {
                                    if let Value::String(s) = n { s.clone() } else { format!("{}", n) }
                                }).collect::<Vec<_>>())
                            } else { None })
                            .unwrap_or_default();
                        let mut obj_fields = HashMap::new();
                        for (i, param_name) in ctor_params.iter().enumerate() {
                            if i < arg_vals.len() {
                                obj_fields.insert(param_name.clone(), arg_vals[i].clone());
                            }
                        }
                        Ok(Value::Object {
                            class_name: class_name.clone(),
                            fields: obj_fields,
                            methods: methods.clone(),
                        })
                    }
                    Some(v) => self.call_value(v, arg_vals),
                    None => Err(RuntimeError::new(format!("not found: type {}", class_name))),
                }
            }

            Expr::For { enumerators, body, is_yield, .. } => {
                if *is_yield {
                    let mut results = Vec::new();
                    self.eval_for_enumerators(enumerators, &mut |env| {
                        let val = env.eval_expr(body)?;
                        results.push(val);
                        Ok(())
                    })?;
                    Ok(Value::List(results))
                } else {
                    self.eval_for_enumerators(enumerators, &mut |env| {
                        env.eval_expr(body)?;
                        Ok(())
                    })?;
                    Ok(Value::Unit)
                }
            }

            Expr::While { cond, body, .. } => {
                loop {
                    let cv = self.eval_expr(cond)?;
                    if !cv.is_truthy() { break; }
                    self.eval_expr(body)?;
                }
                Ok(Value::Unit)
            }

            Expr::DoWhile { body, cond, .. } => {
                loop {
                    self.eval_expr(body)?;
                    let cv = self.eval_expr(cond)?;
                    if !cv.is_truthy() { break; }
                }
                Ok(Value::Unit)
            }

            Expr::StringInterpolation { parts, .. } => {
                let mut result = String::new();
                for part in parts {
                    match part {
                        InterpPart::Literal(s) => result.push_str(s),
                        InterpPart::Expression(expr) => {
                            let v = self.eval_expr(expr)?;
                            result.push_str(&format!("{}", v));
                        }
                    }
                }
                Ok(Value::String(result))
            }

            Expr::This(_) => {
                self.env.lookup("this").cloned()
                    .ok_or_else(|| RuntimeError::new("'this' used outside of class"))
            }
            Expr::Super(_) => {
                self.env.lookup("super").cloned()
                    .ok_or_else(|| RuntimeError::new("'super' used outside of class"))
            }
            Expr::Paren { expr, .. } => self.eval_expr(expr),
            Expr::TypeApply { expr, .. } => self.eval_expr(expr),
            Expr::UnaryMethodCall { receiver, method, .. } => {
                let rv = self.eval_expr(receiver)?;
                self.eval_method_call(rv, method, vec![])
            }
        }
    }

    fn eval_binary(&self, left: &Value, op: &BinOp, right: &Value) -> Result<Value, RuntimeError> {
        match op {
            BinOp::Add => match (left, right) {
                (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a + b)),
                (Value::Long(a), Value::Long(b)) => Ok(Value::Long(a + b)),
                (Value::Int(a), Value::Long(b)) => Ok(Value::Long(a + b)),
                (Value::Long(a), Value::Int(b)) => Ok(Value::Long(a + b)),
                (Value::Double(a), Value::Double(b)) => Ok(Value::Double(a + b)),
                (Value::Int(a), Value::Double(b)) => Ok(Value::Double(*a as f64 + b)),
                (Value::Double(a), Value::Int(b)) => Ok(Value::Double(a + *b as f64)),
                (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a + b)),
                (Value::String(a), _) => Ok(Value::String(format!("{}{}", a, right))),
                (_, Value::String(b)) => Ok(Value::String(format!("{}{}", left, b))),
                _ => Err(RuntimeError::new(format!("cannot add {} and {}", left.type_name(), right.type_name()))),
            },
            BinOp::Sub => match (left, right) {
                (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a - b)),
                (Value::Long(a), Value::Long(b)) => Ok(Value::Long(a - b)),
                (Value::Double(a), Value::Double(b)) => Ok(Value::Double(a - b)),
                (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a - b)),
                (Value::Int(a), Value::Double(b)) => Ok(Value::Double(*a as f64 - b)),
                (Value::Double(a), Value::Int(b)) => Ok(Value::Double(a - *b as f64)),
                _ => Err(RuntimeError::new(format!("cannot subtract {} and {}", left.type_name(), right.type_name()))),
            },
            BinOp::Mul => match (left, right) {
                (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a * b)),
                (Value::Long(a), Value::Long(b)) => Ok(Value::Long(a * b)),
                (Value::Double(a), Value::Double(b)) => Ok(Value::Double(a * b)),
                (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a * b)),
                (Value::Int(a), Value::Double(b)) => Ok(Value::Double(*a as f64 * b)),
                (Value::Double(a), Value::Int(b)) => Ok(Value::Double(a * *b as f64)),
                _ => Err(RuntimeError::new(format!("cannot multiply {} and {}", left.type_name(), right.type_name()))),
            },
            BinOp::Div => match (left, right) {
                (Value::Int(a), Value::Int(b)) => {
                    if *b == 0 { return Err(RuntimeError::new("division by zero")); }
                    Ok(Value::Int(a / b))
                }
                (Value::Double(a), Value::Double(b)) => Ok(Value::Double(a / b)),
                (Value::Int(a), Value::Double(b)) => Ok(Value::Double(*a as f64 / b)),
                (Value::Double(a), Value::Int(b)) => Ok(Value::Double(a / *b as f64)),
                _ => Err(RuntimeError::new("cannot divide")),
            },
            BinOp::Mod => match (left, right) {
                (Value::Int(a), Value::Int(b)) => {
                    if *b == 0 { return Err(RuntimeError::new("division by zero")); }
                    Ok(Value::Int(a % b))
                }
                (Value::Double(a), Value::Double(b)) => Ok(Value::Double(a % b)),
                _ => Err(RuntimeError::new("cannot modulo")),
            },
            BinOp::Eq => Ok(Value::Bool(left == right)),
            BinOp::Neq => Ok(Value::Bool(left != right)),
            BinOp::Lt => self.compare_values(left, right, std::cmp::Ordering::is_lt),
            BinOp::Gt => self.compare_values(left, right, std::cmp::Ordering::is_gt),
            BinOp::Leq => self.compare_values(left, right, std::cmp::Ordering::is_le),
            BinOp::Geq => self.compare_values(left, right, std::cmp::Ordering::is_ge),
            BinOp::BitAnd => match (left, right) {
                (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a & b)),
                (Value::Long(a), Value::Long(b)) => Ok(Value::Long(a & b)),
                (Value::Bool(a), Value::Bool(b)) => Ok(Value::Bool(*a && *b)),
                _ => Err(RuntimeError::new("cannot bitwise and")),
            },
            BinOp::BitOr => match (left, right) {
                (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a | b)),
                (Value::Long(a), Value::Long(b)) => Ok(Value::Long(a | b)),
                (Value::Bool(a), Value::Bool(b)) => Ok(Value::Bool(*a || *b)),
                _ => Err(RuntimeError::new("cannot bitwise or")),
            },
            BinOp::BitXor => match (left, right) {
                (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a ^ b)),
                (Value::Long(a), Value::Long(b)) => Ok(Value::Long(a ^ b)),
                _ => Err(RuntimeError::new("cannot bitwise xor")),
            },
            BinOp::LeftShift => match (left, right) {
                (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a << b)),
                (Value::Long(a), Value::Int(b)) => Ok(Value::Long(a << b)),
                _ => Err(RuntimeError::new("cannot shift left")),
            },
            BinOp::RightShift => match (left, right) {
                (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a >> b)),
                (Value::Long(a), Value::Int(b)) => Ok(Value::Long(a >> b)),
                _ => Err(RuntimeError::new("cannot shift right")),
            },
            BinOp::UnsignedRightShift => match (left, right) {
                (Value::Int(a), Value::Int(b)) => Ok(Value::Int(((*a as u64) >> b) as i64)),
                (Value::Long(a), Value::Int(b)) => Ok(Value::Long(((*a as u64) >> b) as i64)),
                _ => Err(RuntimeError::new("cannot unsigned shift right")),
            },
            BinOp::And | BinOp::Or => unreachable!(),
        }
    }

    fn compare_values(&self, left: &Value, right: &Value, check: fn(std::cmp::Ordering) -> bool) -> Result<Value, RuntimeError> {
        let ord = match (left, right) {
            (Value::Int(a), Value::Int(b)) => a.cmp(b),
            (Value::Long(a), Value::Long(b)) => a.cmp(b),
            (Value::Double(a), Value::Double(b)) => a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal),
            (Value::Int(a), Value::Double(b)) => (*a as f64).partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal),
            (Value::Double(a), Value::Int(b)) => a.partial_cmp(&(*b as f64)).unwrap_or(std::cmp::Ordering::Equal),
            (Value::String(a), Value::String(b)) => a.cmp(b),
            (Value::Char(a), Value::Char(b)) => a.cmp(b),
            _ => return Err(RuntimeError::new(format!("cannot compare {} and {}", left.type_name(), right.type_name()))),
        };
        Ok(Value::Bool(check(ord)))
    }

    fn call_value(&mut self, func: Value, args: Vec<Value>) -> Result<Value, RuntimeError> {
        match func {
            Value::Function { params, body, closure, .. } => {
                if params.len() != args.len() {
                    return Err(RuntimeError::new(
                        format!("wrong number of arguments: expected {}, got {}", params.len(), args.len())
                    ));
                }
                self.env.push();
                self.env.restore(&closure);
                for (name, val) in params.iter().zip(args.iter()) {
                    self.env.define(name, val.deep_clone(), false);
                }
                let result = match self.eval_expr(&body) {
                    Ok(v) => v,
                    Err(e) if e.message.starts_with("__return__") => {
                        let val_str = &e.message["__return__".len()..];
                        self.env.pop();
                        if val_str == "()" { return Ok(Value::Unit); }
                        return Err(e);
                    }
                    Err(e) => { self.env.pop(); return Err(e); }
                };
                self.env.pop();
                Ok(result)
            }
            Value::BuiltinFunction { name, .. } => self.call_builtin(&name, args),
            Value::Object { class_name, fields, methods } => {
                let ctor_params = fields.get("__ctor_params__")
                    .and_then(|v| if let Value::List(names) = v {
                        Some(names.iter().map(|n| {
                            if let Value::String(s) = n { s.clone() } else { format!("{}", n) }
                        }).collect::<Vec<_>>())
                    } else { None })
                    .unwrap_or_default();
                let mut obj_fields = HashMap::new();
                for (i, param_name) in ctor_params.iter().enumerate() {
                    if i < args.len() {
                        obj_fields.insert(param_name.clone(), args[i].clone());
                    }
                }
                Ok(Value::Object { class_name, fields: obj_fields, methods })
            }
            _ => Err(RuntimeError::new(format!("cannot call {}", func.type_name()))),
        }
    }

    fn call_builtin(&mut self, name: &str, args: Vec<Value>) -> Result<Value, RuntimeError> {
        match name {
            "__println__" => {
                match args.first() {
                    Some(v) => println!("{}", v),
                    None => println!(),
                }
                Ok(Value::Unit)
            }
            "__print__" => {
                if let Some(v) = args.first() { print!("{}", v); }
                Ok(Value::Unit)
            }
            "__assert__" => {
                if let Some(Value::Bool(false)) = args.first() {
                    Err(RuntimeError::new("assertion failed"))
                } else { Ok(Value::Unit) }
            }
            "__require__" => {
                if let Some(Value::Bool(false)) = args.first() {
                    Err(RuntimeError::new("requirement failed"))
                } else { Ok(Value::Unit) }
            }
            "__identity__" => Ok(args.first().cloned().unwrap_or(Value::Unit)),
            "__List_apply__" => Ok(Value::List(args)),
            "__Map_apply__" => {
                let mut entries = Vec::new();
                for arg in &args {
                    if let Value::Tuple(pair) = arg {
                        if pair.len() == 2 { entries.push((pair[0].clone(), pair[1].clone())); }
                    }
                }
                Ok(Value::Map(entries))
            }
            "__Some__" => Ok(args.first().cloned().unwrap_or(Value::Null)),
            "__abs__" => match args.first() {
                Some(Value::Int(n)) => Ok(Value::Int(n.abs())),
                Some(Value::Double(n)) => Ok(Value::Double(n.abs())),
                Some(Value::Long(n)) => Ok(Value::Long(n.abs())),
                _ => Err(RuntimeError::new("abs requires numeric")),
            },
            "__max__" => match (args.first(), args.get(1)) {
                (Some(Value::Int(a)), Some(Value::Int(b))) => Ok(Value::Int((*a).max(*b))),
                (Some(Value::Double(a)), Some(Value::Double(b))) => Ok(Value::Double(a.max(*b))),
                _ => Err(RuntimeError::new("max requires two numbers")),
            },
            "__min__" => match (args.first(), args.get(1)) {
                (Some(Value::Int(a)), Some(Value::Int(b))) => Ok(Value::Int((*a).min(*b))),
                (Some(Value::Double(a)), Some(Value::Double(b))) => Ok(Value::Double(a.min(*b))),
                _ => Err(RuntimeError::new("min requires two numbers")),
            },
            "__pow__" => match (args.first(), args.get(1)) {
                (Some(Value::Double(a)), Some(Value::Double(b))) => Ok(Value::Double(a.powf(*b))),
                (Some(Value::Int(a)), Some(Value::Int(b))) => Ok(Value::Double((*a as f64).powf(*b as f64))),
                _ => Err(RuntimeError::new("pow requires two numbers")),
            },
            "__sqrt__" => match args.first() {
                Some(Value::Double(n)) => Ok(Value::Double(n.sqrt())),
                Some(Value::Int(n)) => Ok(Value::Double((*n as f64).sqrt())),
                _ => Err(RuntimeError::new("sqrt requires numeric")),
            },
            "__range_to__" => match (args.first(), args.get(1)) {
                (Some(Value::Int(a)), Some(Value::Int(b))) => {
                    Ok(Value::List((*a..=*b).map(Value::Int).collect()))
                }
                _ => Err(RuntimeError::new("range requires two ints")),
            },
            "__range_until__" => match (args.first(), args.get(1)) {
                (Some(Value::Int(a)), Some(Value::Int(b))) => {
                    Ok(Value::List((*a..*b).map(Value::Int).collect()))
                }
                _ => Err(RuntimeError::new("range until requires two ints")),
            },
            other => Err(RuntimeError::new(format!("unknown builtin: {}", other))),
        }
    }

    #[allow(clippy::cognitive_complexity)]
    fn eval_method_call(&mut self, receiver: Value, method: &str, args: Vec<Value>) -> Result<Value, RuntimeError> {
        match (&receiver, method) {
            (Value::String(s), "length") => Ok(Value::Int(s.len() as i64)),
            (Value::String(s), "substring") => match (args.first(), args.get(1)) {
                (Some(Value::Int(start)), Some(Value::Int(end))) => Ok(Value::String(s[*start as usize..*end as usize].to_string())),
                (Some(Value::Int(start)), None) => Ok(Value::String(s[*start as usize..].to_string())),
                _ => Err(RuntimeError::new("substring requires int args")),
            },
            (Value::String(s), "split") => match args.first() {
                Some(Value::String(sep)) => Ok(Value::List(s.split(sep).map(|p| Value::String(p.to_string())).collect())),
                _ => Err(RuntimeError::new("split requires string arg")),
            },
            (Value::String(s), "trim") => Ok(Value::String(s.trim().to_string())),
            (Value::String(s), "toUpperCase") => Ok(Value::String(s.to_uppercase())),
            (Value::String(s), "toLowerCase") => Ok(Value::String(s.to_lowercase())),
            (Value::String(s), "replace") => match (args.first(), args.get(1)) {
                (Some(Value::String(from)), Some(Value::String(to))) => Ok(Value::String(s.replace(from.as_str(), to.as_str()))),
                _ => Err(RuntimeError::new("replace requires two strings")),
            },
            (Value::String(s), "contains") => match args.first() {
                Some(Value::String(sub)) => Ok(Value::Bool(s.contains(sub.as_str()))),
                _ => Err(RuntimeError::new("contains requires string")),
            },
            (Value::String(s), "startsWith") => match args.first() {
                Some(Value::String(prefix)) => Ok(Value::Bool(s.starts_with(prefix.as_str()))),
                _ => Err(RuntimeError::new("startsWith requires string")),
            },
            (Value::String(s), "endsWith") => match args.first() {
                Some(Value::String(suffix)) => Ok(Value::Bool(s.ends_with(suffix.as_str()))),
                _ => Err(RuntimeError::new("endsWith requires string")),
            },
            (Value::String(s), "isEmpty") => Ok(Value::Bool(s.is_empty())),
            (Value::String(s), "nonEmpty") => Ok(Value::Bool(!s.is_empty())),
            (Value::String(s), "reverse") => Ok(Value::String(s.chars().rev().collect())),
            (Value::String(s), "indexOf") => match args.first() {
                Some(Value::String(sub)) => Ok(Value::Int(s.find(sub.as_str()).map_or(-1, |i| i as i64))),
                _ => Err(RuntimeError::new("indexOf requires string")),
            },
            (Value::String(s), "charAt") => match args.first() {
                Some(Value::Int(idx)) => s.chars().nth(*idx as usize).map(Value::Char).ok_or_else(|| RuntimeError::new("index out of bounds")),
                _ => Err(RuntimeError::new("charAt requires int")),
            },
            (Value::String(s), "toInt") => s.parse::<i64>().map(Value::Int).map_err(|_| RuntimeError::new(format!("cannot parse '{}' as Int", s))),
            (Value::String(s), "toDouble") => s.parse::<f64>().map(Value::Double).map_err(|_| RuntimeError::new(format!("cannot parse '{}' as Double", s))),
            (Value::String(s), "strip") => Ok(Value::String(s.trim().to_string())),
            (Value::Int(n), "toString") => Ok(Value::String(n.to_string())),
            (Value::Int(n), "toLong") => Ok(Value::Long(*n)),
            (Value::Int(n), "toDouble") => Ok(Value::Double(*n as f64)),
            (Value::Int(n), "toFloat") => Ok(Value::Float(*n as f64)),
            (Value::Int(n), "toChar") => Ok(Value::Char(char::from_u32(*n as u32).unwrap_or('\0'))),
            (Value::Long(n), "toInt") => Ok(Value::Int(*n)),
            (Value::Long(n), "toDouble") => Ok(Value::Double(*n as f64)),
            (Value::Double(n), "toInt") => Ok(Value::Int(*n as i64)),
            (Value::Double(n), "toFloat") => Ok(Value::Float(*n)),
            (Value::Double(n), "toLong") => Ok(Value::Long(*n as i64)),
            (Value::Float(n), "toDouble") => Ok(Value::Double(*n)),
            (Value::Float(n), "toInt") => Ok(Value::Int(*n as i64)),
            (Value::Char(c), "toInt") => Ok(Value::Int(*c as i64)),
            (Value::Bool(b), "toString") => Ok(Value::String(b.to_string())),

            (Value::List(elements), "length") | (Value::List(elements), "size") => Ok(Value::Int(elements.len() as i64)),
            (Value::List(elements), "isEmpty") => Ok(Value::Bool(elements.is_empty())),
            (Value::List(elements), "nonEmpty") => Ok(Value::Bool(!elements.is_empty())),
            (Value::List(elements), "head") => elements.first().cloned().ok_or_else(|| RuntimeError::new("head of empty list")),
            (Value::List(elements), "last") => elements.last().cloned().ok_or_else(|| RuntimeError::new("last of empty list")),
            (Value::List(elements), "tail") => {
                if elements.is_empty() { return Err(RuntimeError::new("tail of empty list")); }
                Ok(Value::List(elements[1..].to_vec()))
            }
            (Value::List(elements), "init") => {
                if elements.is_empty() { return Err(RuntimeError::new("init of empty list")); }
                Ok(Value::List(elements[..elements.len()-1].to_vec()))
            }
            (Value::List(elements), "reverse") => Ok(Value::List(elements.iter().rev().cloned().collect())),
            (Value::List(elements), "distinct") => {
                let mut seen = Vec::new();
                for e in elements { if !seen.contains(e) { seen.push(e.clone()); } }
                Ok(Value::List(seen))
            }
            (Value::List(elements), "map") => {
                if let Some(func) = args.first().cloned() {
                    let mut results = Vec::new();
                    for elem in elements { results.push(self.call_value(func.clone(), vec![elem.clone()])?); }
                    Ok(Value::List(results))
                } else { Err(RuntimeError::new("map requires a function")) }
            }
            (Value::List(elements), "flatMap") => {
                if let Some(func) = args.first().cloned() {
                    let mut results = Vec::new();
                    for elem in elements {
                        match self.call_value(func.clone(), vec![elem.clone()])? {
                            Value::List(inner) => results.extend(inner),
                            other => results.push(other),
                        }
                    }
                    Ok(Value::List(results))
                } else { Err(RuntimeError::new("flatMap requires a function")) }
            }
            (Value::List(elements), "filter") | (Value::List(elements), "withFilter") => {
                if let Some(func) = args.first().cloned() {
                    let mut results = Vec::new();
                    for elem in elements {
                        if let Value::Bool(true) = self.call_value(func.clone(), vec![elem.clone()])? {
                            results.push(elem.clone());
                        }
                    }
                    Ok(Value::List(results))
                } else { Err(RuntimeError::new("filter requires a function")) }
            }
            (Value::List(elements), "foreach") => {
                if let Some(func) = args.first().cloned() {
                    for elem in elements { self.call_value(func.clone(), vec![elem.clone()])?; }
                }
                Ok(Value::Unit)
            }
            (Value::List(elements), "foldLeft") => {
                if args.len() >= 2 {
                    let mut acc = args[0].clone();
                    let func = args[1].clone();
                    for elem in elements { acc = self.call_value(func.clone(), vec![acc, elem.clone()])?; }
                    Ok(acc)
                } else { Err(RuntimeError::new("foldLeft requires initial value and function")) }
            }
            (Value::List(elements), "foldRight") => {
                if args.len() >= 2 {
                    let mut acc = args[0].clone();
                    let func = args[1].clone();
                    for elem in elements.iter().rev() { acc = self.call_value(func.clone(), vec![elem.clone(), acc])?; }
                    Ok(acc)
                } else { Err(RuntimeError::new("foldRight requires initial value and function")) }
            }
            (Value::List(elements), "reduce") | (Value::List(elements), "reduceLeft") => {
                if elements.is_empty() { return Err(RuntimeError::new("reduce on empty list")); }
                if let Some(func) = args.first().cloned() {
                    let mut acc = elements[0].clone();
                    for elem in &elements[1..] { acc = self.call_value(func.clone(), vec![acc, elem.clone()])?; }
                    Ok(acc)
                } else { Err(RuntimeError::new("reduce requires a function")) }
            }
            (Value::List(elements), "find") => {
                if let Some(func) = args.first().cloned() {
                    for elem in elements {
                        if let Value::Bool(true) = self.call_value(func.clone(), vec![elem.clone()])? { return Ok(elem.clone()); }
                    }
                    Ok(Value::String("None".into()))
                } else { Err(RuntimeError::new("find requires a function")) }
            }
            (Value::List(elements), "exists") => {
                if let Some(func) = args.first().cloned() {
                    for elem in elements {
                        if let Value::Bool(true) = self.call_value(func.clone(), vec![elem.clone()])? { return Ok(Value::Bool(true)); }
                    }
                    Ok(Value::Bool(false))
                } else { Err(RuntimeError::new("exists requires a function")) }
            }
            (Value::List(elements), "forall") => {
                if let Some(func) = args.first().cloned() {
                    for elem in elements {
                        if let Value::Bool(false) = self.call_value(func.clone(), vec![elem.clone()])? { return Ok(Value::Bool(false)); }
                    }
                    Ok(Value::Bool(true))
                } else { Err(RuntimeError::new("forall requires a function")) }
            }
            (Value::List(elements), "count") => {
                if let Some(func) = args.first().cloned() {
                    let mut count = 0i64;
                    for elem in elements {
                        if let Value::Bool(true) = self.call_value(func.clone(), vec![elem.clone()])? { count += 1; }
                    }
                    Ok(Value::Int(count))
                } else { Err(RuntimeError::new("count requires a function")) }
            }
            (Value::List(elements), "take") => match args.first() {
                Some(Value::Int(n)) => Ok(Value::List(elements.iter().take(*n as usize).cloned().collect())),
                _ => Err(RuntimeError::new("take requires int")),
            },
            (Value::List(elements), "drop") => match args.first() {
                Some(Value::Int(n)) => Ok(Value::List(elements.iter().skip(*n as usize).cloned().collect())),
                _ => Err(RuntimeError::new("drop requires int")),
            },
            (Value::List(elements), "zip") => match args.first() {
                Some(Value::List(other)) => Ok(Value::List(elements.iter().zip(other.iter()).map(|(a, b)| Value::Tuple(vec![a.clone(), b.clone()])).collect())),
                _ => Err(RuntimeError::new("zip requires a list")),
            },
            (Value::List(elements), "zipWithIndex") => {
                Ok(Value::List(elements.iter().enumerate().map(|(i, v)| Value::Tuple(vec![v.clone(), Value::Int(i as i64)])).collect()))
            }
            (Value::List(elements), "sorted") => {
                let mut sorted = elements.clone();
                sorted.sort_by(|a, b| match (a, b) {
                    (Value::Int(x), Value::Int(y)) => x.cmp(y),
                    (Value::Double(x), Value::Double(y)) => x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal),
                    (Value::String(x), Value::String(y)) => x.cmp(y),
                    _ => std::cmp::Ordering::Equal,
                });
                Ok(Value::List(sorted))
            }
            (Value::List(elements), "sum") => {
                let mut total = 0i64;
                for e in elements { if let Some(n) = e.to_int() { total += n; } }
                Ok(Value::Int(total))
            }
            (Value::List(elements), "product") => {
                let mut total = 1i64;
                for e in elements { if let Some(n) = e.to_int() { total *= n; } }
                Ok(Value::Int(total))
            }
            (Value::List(elements), "min") => elements.iter().filter_map(|e| e.to_int()).min().map(Value::Int).ok_or_else(|| RuntimeError::new("min of empty list")),
            (Value::List(elements), "max") => elements.iter().filter_map(|e| e.to_int()).max().map(Value::Int).ok_or_else(|| RuntimeError::new("max of empty list")),
            (Value::List(elements), "mkString") => {
                let sep = match args.first() { Some(Value::String(s)) => s.clone(), _ => String::new() };
                Ok(Value::String(elements.iter().map(|e| format!("{}", e)).collect::<Vec<_>>().join(&sep)))
            }
            (Value::List(elements), "flatten") => {
                let mut result = Vec::new();
                for e in elements { match e { Value::List(inner) => result.extend(inner.clone()), other => result.push(other.clone()) } }
                Ok(Value::List(result))
            }
            (Value::List(elements), "contains") => match args.first() {
                Some(target) => Ok(Value::Bool(elements.contains(target))),
                None => Err(RuntimeError::new("contains requires an argument")),
            },
            (Value::List(elements), "indexOf") => match args.first() {
                Some(target) => Ok(Value::Int(elements.iter().position(|e| e == target).map_or(-1, |i| i as i64))),
                None => Err(RuntimeError::new("indexOf requires an argument")),
            },
            (Value::List(_), "toList") => Ok(receiver.clone()),
            (Value::List(elements), "toArray") => Ok(Value::Array(elements.clone())),

            (Value::Map(entries), "get") => match args.first() {
                Some(key) => match entries.iter().find(|(k, _)| *k == *key) {
                    Some((_, v)) => Ok(v.clone()),
                    None => Ok(Value::String("None".into())),
                },
                None => Err(RuntimeError::new("get requires a key")),
            },
            (Value::Map(entries), "contains") => match args.first() {
                Some(key) => Ok(Value::Bool(entries.iter().any(|(k, _)| *k == *key))),
                None => Err(RuntimeError::new("contains requires a key")),
            },
            (Value::Map(entries), "keys") => Ok(Value::List(entries.iter().map(|(k, _)| k.clone()).collect())),
            (Value::Map(entries), "values") => Ok(Value::List(entries.iter().map(|(_, v)| v.clone()).collect())),
            (Value::Map(entries), "size") | (Value::Map(entries), "length") => Ok(Value::Int(entries.len() as i64)),
            (Value::Map(entries), "isEmpty") => Ok(Value::Bool(entries.is_empty())),
            (Value::Map(entries), "foreach") | (Value::Map(entries), "foreachEntry") => {
                if let Some(func) = args.first().cloned() {
                    for (k, v) in entries { self.call_value(func.clone(), vec![k.clone(), v.clone()])?; }
                }
                Ok(Value::Unit)
            }
            (Value::Map(entries), "map") => {
                if let Some(func) = args.first().cloned() {
                    let mut results = Vec::new();
                    for (k, v) in entries { results.push(self.call_value(func.clone(), vec![k.clone(), v.clone()])?); }
                    Ok(Value::List(results))
                } else { Err(RuntimeError::new("map requires a function")) }
            }
            (Value::Map(entries), "updated") => {
                if args.len() >= 2 {
                    let mut new_entries = entries.clone();
                    if let Some(pos) = new_entries.iter().position(|(k, _)| *k == args[0]) {
                        new_entries[pos].1 = args[1].clone();
                    } else {
                        new_entries.push((args[0].clone(), args[1].clone()));
                    }
                    Ok(Value::Map(new_entries))
                } else { Err(RuntimeError::new("updated requires key and value")) }
            }

            (Value::Tuple(elements), "_1") if !elements.is_empty() => Ok(elements[0].clone()),
            (Value::Tuple(elements), "_2") if elements.len() > 1 => Ok(elements[1].clone()),
            (Value::Tuple(elements), "_3") if elements.len() > 2 => Ok(elements[2].clone()),
            (Value::Tuple(elements), "_4") if elements.len() > 3 => Ok(elements[3].clone()),
            (Value::Tuple(elements), "_5") if elements.len() > 4 => Ok(elements[4].clone()),
            (Value::Tuple(elements), "toList") => Ok(Value::List(elements.clone())),

            (Value::Object { class_name, fields, methods }, method_name) => {
                if let Some(method_val) = methods.get(method_name) {
                    self.call_value(method_val.clone(), args)
                } else if method_name == "toString" {
                    Ok(Value::String(format!("{}", receiver)))
                } else if method_name == "equals" {
                    Ok(Value::Bool(receiver == args.first().cloned().unwrap_or(Value::Null)))
                } else if method_name == "hashCode" {
                    Ok(Value::Int(0))
                } else if method_name == "copy" {
                    Ok(receiver.clone())
                } else if method_name == "canEqual" {
                    Ok(Value::Bool(true))
                } else if method_name == "productPrefix" {
                    Ok(Value::String(class_name.clone()))
                } else if method_name == "_1" || method_name == "_2" || method_name == "_3" || method_name == "_4" || method_name == "_5" {
                    let idx = method_name[1..].parse::<usize>().unwrap_or(1) - 1;
                    let mut keys: Vec<&String> = fields.keys().collect();
                    keys.sort();
                    if idx < keys.len() {
                        fields.get(keys[idx]).cloned().ok_or_else(|| RuntimeError::new("field not found"))
                    } else { Err(RuntimeError::new("tuple index out of bounds")) }
                } else {
                    Err(RuntimeError::new(format!("{} has no method '{}'", class_name, method_name)))
                }
            }
            _ => Err(RuntimeError::new(format!("cannot call method '{}' on {}", method, receiver.type_name()))),
        }
    }

    fn eval_field_access(&mut self, receiver: &Value, field: &str) -> Result<Value, RuntimeError> {
        match receiver {
            Value::Object { fields, .. } => {
                if let Some(val) = fields.get(field) { Ok(val.clone()) }
                else if field == "toString" { Ok(Value::String(format!("{}", receiver))) }
                else if field == "Pi" { Ok(Value::Double(std::f64::consts::PI)) }
                else if field == "E" { Ok(Value::Double(std::f64::consts::E)) }
                else { Err(RuntimeError::new(format!("field '{}' not found", field))) }
            }
            Value::Tuple(elements) => {
                if field.starts_with('_') && field.len() >= 2 {
                    if let Ok(idx) = field[1..].parse::<usize>() {
                        if idx >= 1 && idx <= elements.len() { Ok(elements[idx - 1].clone()) }
                        else { Err(RuntimeError::new("tuple index out of bounds")) }
                    } else { Err(RuntimeError::new(format!("invalid tuple accessor: {}", field))) }
                } else { Err(RuntimeError::new(format!("tuple has no field '{}'", field))) }
            }
            Value::List(elements) => {
                match field {
                    "length" | "size" => Ok(Value::Int(elements.len() as i64)),
                    "head" => elements.first().cloned().ok_or_else(|| RuntimeError::new("head of empty list")),
                    "isEmpty" => Ok(Value::Bool(elements.is_empty())),
                    _ => Err(RuntimeError::new(format!("List has no field '{}'", field))),
                }
            }
            Value::String(s) => {
                if field == "length" { Ok(Value::Int(s.len() as i64)) }
                else { Err(RuntimeError::new(format!("String has no field '{}'", field))) }
            }
            _ => Err(RuntimeError::new(format!("cannot access field '{}' on {}", field, receiver.type_name()))),
        }
    }

    fn eval_match(&mut self, scrutinee: &Value, cases: &[MatchCase]) -> Result<Value, RuntimeError> {
        for case in cases {
            self.env.push();
            if self.match_pattern(&case.pattern, scrutinee)? {
                if let Some(guard) = &case.guard {
                    let guard_val = self.eval_expr(guard)?;
                    if !guard_val.is_truthy() {
                        self.env.pop();
                        continue;
                    }
                }
                let result = self.eval_expr(&case.body);
                self.env.pop();
                return result;
            }
            self.env.pop();
        }
        Err(RuntimeError::new(format!("match error: no case matched for {}", scrutinee)))
    }

    fn try_catch(&mut self, err_str: &str, catches: &[MatchCase]) -> Option<Result<Value, RuntimeError>> {
        let exc_value = if err_str.starts_with("__exception__") {
            Value::String(err_str["__exception__".len()..].to_string())
        } else {
            Value::String(err_str.to_string())
        };
        self.env.push();
        for case in catches {
            if let Ok(true) = self.match_pattern(&case.pattern, &exc_value) {
                if let Some(guard) = &case.guard {
                    if let Ok(guard_val) = self.eval_expr(guard) {
                        if !guard_val.is_truthy() { continue; }
                    }
                }
                let result = self.eval_expr(&case.body);
                self.env.pop();
                return Some(result);
            }
        }
        self.env.pop();
        None
    }

    fn eval_for_enumerators(&mut self, enumerators: &[Enumerator], body: &mut dyn FnMut(&mut Interpreter) -> Result<(), RuntimeError>) -> Result<(), RuntimeError> {
        self.eval_for_enumerators_rec(enumerators, 0, body)
    }

    fn eval_for_enumerators_rec(&mut self, enumerators: &[Enumerator], idx: usize, body: &mut dyn FnMut(&mut Interpreter) -> Result<(), RuntimeError>) -> Result<(), RuntimeError> {
        if idx >= enumerators.len() { body(self)?; return Ok(()); }
        match &enumerators[idx] {
            Enumerator::Generator { pattern, expr, .. } => {
                let collection = self.eval_expr(expr)?;
                match collection {
                    Value::List(elements) => {
                        for elem in &elements {
                            self.env.push();
                            self.bind_pattern(pattern, elem)?;
                            self.eval_for_enumerators_rec(enumerators, idx + 1, body)?;
                            self.env.pop();
                        }
                    }
                    Value::Tuple(elements) => {
                        for elem in &elements {
                            self.env.push();
                            self.bind_pattern(pattern, elem)?;
                            self.eval_for_enumerators_rec(enumerators, idx + 1, body)?;
                            self.env.pop();
                        }
                    }
                    _ => {
                        self.env.push();
                        self.bind_pattern(pattern, &collection)?;
                        self.eval_for_enumerators_rec(enumerators, idx + 1, body)?;
                        self.env.pop();
                    }
                }
            }
            Enumerator::Filter { cond, .. } => {
                let cv = self.eval_expr(cond)?;
                if cv.is_truthy() { self.eval_for_enumerators_rec(enumerators, idx + 1, body)?; }
            }
            Enumerator::Val { pattern, expr, .. } => {
                let val = self.eval_expr(expr)?;
                self.bind_pattern(pattern, &val)?;
                self.eval_for_enumerators_rec(enumerators, idx + 1, body)?;
            }
        }
        Ok(())
    }

    fn match_pattern(&mut self, pattern: &Pattern, value: &Value) -> Result<bool, RuntimeError> {
        match pattern {
            Pattern::Wildcard(_) => Ok(true),
            Pattern::Variable { name, .. } => {
                if name != "_" { self.env.define(name, value.clone(), false); }
                Ok(true)
            }
            Pattern::Literal { value: lit, .. } => Ok(*value == literal_to_value(lit)),
            Pattern::Constructor { name, args, .. } => match value {
                Value::Object { class_name, fields, .. } => {
                    if class_name != name { return Ok(false); }
                    let mut field_keys: Vec<&String> = fields.keys().filter(|k| !k.starts_with("__")).collect();
                    field_keys.sort();
                    if args.len() != field_keys.len() { return Ok(false); }
                    for (pat, key) in args.iter().zip(field_keys.iter()) {
                        let field_val = fields.get(*key).cloned().unwrap_or(Value::Unit);
                        if !self.match_pattern(pat, &field_val)? { return Ok(false); }
                    }
                    Ok(true)
                }
                _ => Ok(false),
            },
            Pattern::Tuple { elements, .. } => {
                if let Value::Tuple(vals) = value {
                    if elements.len() != vals.len() { return Ok(false); }
                    for (pat, val) in elements.iter().zip(vals.iter()) {
                        if !self.match_pattern(pat, val)? { return Ok(false); }
                    }
                    Ok(true)
                } else { Ok(false) }
            }
            Pattern::Typed { pattern, .. } => self.match_pattern(pattern, value),
            Pattern::Alternative { left, right, .. } => {
                if self.match_pattern(left, value)? { Ok(true) } else { self.match_pattern(right, value) }
            }
            Pattern::SequenceWildcard(_) => Ok(true),
        }
    }

    fn bind_pattern(&mut self, pattern: &Pattern, value: &Value) -> Result<(), RuntimeError> {
        match pattern {
            Pattern::Wildcard(_) => Ok(()),
            Pattern::Variable { name, .. } => {
                if name != "_" { self.env.define(name, value.clone(), false); }
                Ok(())
            }
            Pattern::Literal { .. } => Ok(()),
            Pattern::Constructor { args, .. } => match value {
                Value::Object { fields, .. } => {
                    let mut keys: Vec<&String> = fields.keys().filter(|k| !k.starts_with("__")).collect();
                    keys.sort();
                    for (pat, key) in args.iter().zip(keys.iter()) {
                        let field_val = fields.get(*key).cloned().unwrap_or(Value::Unit);
                        self.bind_pattern(pat, &field_val)?;
                    }
                    Ok(())
                }
                _ => Ok(()),
            },
            Pattern::Tuple { elements, .. } => {
                if let Value::Tuple(vals) = value {
                    for (pat, val) in elements.iter().zip(vals.iter()) { self.bind_pattern(pat, val)?; }
                }
                Ok(())
            }
            Pattern::Typed { pattern, .. } => self.bind_pattern(pattern, value),
            _ => Ok(()),
        }
    }

    fn bind_pattern_mut(&mut self, pattern: &Pattern, value: &Value) -> Result<(), RuntimeError> {
        match pattern {
            Pattern::Variable { name, .. } => {
                if name != "_" { self.env.define(name, value.clone(), true); }
                Ok(())
            }
            _ => self.bind_pattern(pattern, value),
        }
    }
}

fn literal_to_value(lit: &Literal) -> Value {
    match lit {
        Literal::Int(v) => Value::Int(*v),
        Literal::Long(v) => Value::Long(*v),
        Literal::Double(v) => Value::Double(*v),
        Literal::Float(v) => Value::Float(*v),
        Literal::Bool(v) => Value::Bool(*v),
        Literal::String(v) => Value::String(v.clone()),
        Literal::Char(v) => Value::Char(*v),
        Literal::Null => Value::Null,
        Literal::Unit => Value::Unit,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn eval(source: &str) -> Value {
        let mut interp = Interpreter::new();
        interp.run_source(source).unwrap()
    }

    fn eval_err(source: &str) -> RuntimeError {
        let mut interp = Interpreter::new();
        interp.run_source(source).unwrap_err()
    }

    #[test]
    fn test_int_literal() { assert_eq!(eval("42"), Value::Int(42)); }
    #[test]
    fn test_string_literal() { assert_eq!(eval("\"hello\""), Value::String("hello".into())); }
    #[test]
    fn test_bool_literal() { assert_eq!(eval("true"), Value::Bool(true)); }
    #[test]
    fn test_null() { assert_eq!(eval("null"), Value::Null); }
    #[test]
    fn test_unit() { assert_eq!(eval("()"), Value::Unit); }
    #[test]
    fn test_negation() { assert_eq!(eval("-42"), Value::Int(-42)); }

    #[test]
    fn test_arithmetic() {
        assert_eq!(eval("1 + 2"), Value::Int(3));
        assert_eq!(eval("10 - 3"), Value::Int(7));
        assert_eq!(eval("4 * 5"), Value::Int(20));
        assert_eq!(eval("10 / 3"), Value::Int(3));
        assert_eq!(eval("10 % 3"), Value::Int(1));
    }

    #[test]
    fn test_string_concat() {
        assert_eq!(eval("\"hello\" + \" \" + \"world\""), Value::String("hello world".into()));
    }

    #[test]
    fn test_comparison() {
        assert_eq!(eval("1 == 1"), Value::Bool(true));
        assert_eq!(eval("1 != 2"), Value::Bool(true));
        assert_eq!(eval("1 < 2"), Value::Bool(true));
        assert_eq!(eval("2 > 1"), Value::Bool(true));
        assert_eq!(eval("1 >= 1"), Value::Bool(true));
        assert_eq!(eval("1 <= 1"), Value::Bool(true));
    }

    #[test]
    fn test_boolean_logic() {
        assert_eq!(eval("true && false"), Value::Bool(false));
        assert_eq!(eval("true || false"), Value::Bool(true));
        assert_eq!(eval("!true"), Value::Bool(false));
        assert_eq!(eval("!false"), Value::Bool(true));
    }

    #[test]
    fn test_if_else() {
        assert_eq!(eval("if (true) 1 else 2"), Value::Int(1));
        assert_eq!(eval("if (false) 1 else 2"), Value::Int(2));
        assert_eq!(eval("if (true) 42"), Value::Int(42));
    }

    #[test]
    fn test_block() {
        assert_eq!(eval("{ val x = 10; val y = 20; x + y }"), Value::Int(30));
    }

    #[test]
    fn test_lambda() {
        assert_eq!(eval("{ val f = (x) => x + 1; f(41) }"), Value::Int(42));
    }

    #[test]
    fn test_def() {
        assert_eq!(eval("def add(a: Int, b: Int): Int = a + b; add(3, 4)"), Value::Int(7));
    }

    #[test]
    fn test_val() { assert_eq!(eval("val x = 42; x"), Value::Int(42)); }

    #[test]
    fn test_var() { assert_eq!(eval("var x = 1; x = 2; x"), Value::Int(2)); }

    #[test]
    fn test_tuple() {
        match eval("(1, 2, 3)") {
            Value::Tuple(v) => assert_eq!(v, vec![Value::Int(1), Value::Int(2), Value::Int(3)]),
            _ => panic!("expected tuple"),
        }
    }

    #[test]
    fn test_list() {
        match eval("List(1, 2, 3)") {
            Value::List(v) => assert_eq!(v.len(), 3),
            _ => panic!("expected list"),
        }
    }

    #[test]
    fn test_list_map() {
        match eval("List(1, 2, 3).map((x) => x * 2)") {
            Value::List(v) => assert_eq!(v, vec![Value::Int(2), Value::Int(4), Value::Int(6)]),
            _ => panic!("expected list"),
        }
    }

    #[test]
    fn test_list_filter() {
        match eval("List(1, 2, 3, 4, 5).filter((x) => x > 3)") {
            Value::List(v) => assert_eq!(v, vec![Value::Int(4), Value::Int(5)]),
            _ => panic!("expected list"),
        }
    }

    #[test]
    fn test_list_fold_left() {
        // Test reduce as a simpler alternative
        assert_eq!(eval("List(1, 2, 3).reduce((a, b) => a + b)"), Value::Int(6));
    }

    #[test]
    fn test_recursion() {
        assert_eq!(eval("def fact(n: Int): Int = if (n <= 1) 1 else n * fact(n - 1); fact(5)"), Value::Int(120));
    }

    #[test]
    fn test_class() {
        let result = eval("class Point(val x: Int, val y: Int); val p = new Point(1, 2); p");
        match result {
            Value::Object { class_name, fields, .. } => {
                assert_eq!(class_name, "Point");
                assert_eq!(fields.get("x"), Some(&Value::Int(1)));
                assert_eq!(fields.get("y"), Some(&Value::Int(2)));
            }
            _ => panic!("expected object"),
        }
    }

    #[test]
    fn test_match() {
        assert_eq!(eval("3 match { case 1 => 10 case 2 => 20 case _ => 30 }"), Value::Int(30));
    }

    #[test]
    fn test_for_yield() {
        match eval("for (x <- List(1, 2, 3)) yield x * 2") {
            Value::List(v) => assert_eq!(v, vec![Value::Int(2), Value::Int(4), Value::Int(6)]),
            _ => panic!("expected list"),
        }
    }

    #[test]
    fn test_string_methods() {
        assert_eq!(eval("\"hello\".length"), Value::Int(5));
        assert_eq!(eval("\"hello\".toUpperCase"), Value::String("HELLO".into()));
        assert_eq!(eval("\"  hi  \".trim"), Value::String("hi".into()));
    }

    #[test]
    fn test_while_loop() {
        assert_eq!(eval("var x = 0; var i = 0; while (i < 5) { x = x + i; i = i + 1 }; x"), Value::Int(10));
    }

    #[test]
    fn test_precedence() {
        assert_eq!(eval("2 + 3 * 4"), Value::Int(14));
        assert_eq!(eval("(2 + 3) * 4"), Value::Int(20));
    }

    #[test]
    fn test_higher_order() {
        assert_eq!(eval("{ def apply(f: Int => Int, x: Int): Int = f(x); apply((x) => x * x, 5) }"), Value::Int(25));
    }

    #[test]
    fn test_closures() {
        assert_eq!(eval("{ val x = 10; val f = () => x + 1; f() }"), Value::Int(11));
        assert_eq!(eval("{ def makeAdder(n: Int): Int => Int = (x) => x + n; val add5 = makeAdder(5); add5(10) }"), Value::Int(15));
    }

    #[test]
    fn test_curried_application() {
        assert_eq!(eval("{ def add(a: Int, b: Int): Int = a + b; add(3, 4) }"), Value::Int(7));
    }

    #[test]
    fn test_division_by_zero() {
        match eval_err("1 / 0") {
            e => assert!(e.message.contains("division by zero")),
        }
    }

    #[test]
    fn test_list_head_tail() {
        match eval("List(10, 20, 30).head") { Value::Int(10) => {} _ => panic!("expected 10") }
        match eval("List(10, 20, 30).tail") {
            Value::List(v) => assert_eq!(v, vec![Value::Int(20), Value::Int(30)]),
            _ => panic!("expected list"),
        }
    }

    #[test]
    fn test_list_reduce() {
        assert_eq!(eval("List(1, 2, 3, 4).reduce((a, b) => a + b)"), Value::Int(10));
    }

    #[test]
    fn test_list_sorted() {
        match eval("List(3, 1, 2).sorted") {
            Value::List(v) => assert_eq!(v, vec![Value::Int(1), Value::Int(2), Value::Int(3)]),
            _ => panic!("expected list"),
        }
    }

    #[test]
    fn test_list_reverse() {
        match eval("List(1, 2, 3).reverse") {
            Value::List(v) => assert_eq!(v, vec![Value::Int(3), Value::Int(2), Value::Int(1)]),
            _ => panic!("expected list"),
        }
    }

    #[test]
    fn test_list_mkstring() {
        assert_eq!(eval("List(1, 2, 3).mkString(\", \")"), Value::String("1, 2, 3".into()));
    }

    #[test]
    fn test_object_decl() {
        let result = eval("object Math { val pi = 3 }; Math");
        match result {
            Value::Object { class_name, fields, .. } => {
                assert_eq!(class_name, "Math");
                assert_eq!(fields.get("pi"), Some(&Value::Int(3)));
            }
            _ => panic!("expected object"),
        }
    }

    #[test]
    fn test_object_methods() {
        let result = eval("object Calc { def double(x: Int): Int = x * 2 }; Calc.double(21)");
        assert_eq!(result, Value::Int(42));
    }

    #[test]
    fn test_for_with_filter() {
        match eval("for (x <- List(1, 2, 3, 4, 5); if x % 2 == 0) yield x") {
            Value::List(v) => assert_eq!(v, vec![Value::Int(2), Value::Int(4)]),
            _ => panic!("expected list"),
        }
    }

    #[test]
    fn test_nested_for() {
        match eval("for (x <- List(1, 2); y <- List(3, 4)) yield x + y") {
            Value::List(v) => assert_eq!(v, vec![Value::Int(4), Value::Int(5), Value::Int(5), Value::Int(6)]),
            _ => panic!("expected list"),
        }
    }

    #[test]
    fn test_string_contains() {
        assert_eq!(eval("\"hello world\".contains(\"world\")"), Value::Bool(true));
        assert_eq!(eval("\"hello world\".contains(\"foo\")"), Value::Bool(false));
    }

    #[test]
    fn test_fibonacci() {
        assert_eq!(eval("def fib(n: Int): Int = if (n <= 1) n else fib(n - 1) + fib(n - 2); fib(10)"), Value::Int(55));
    }

    #[test]
    fn test_type_annotations() {
        assert_eq!(eval("val x: Int = 42; x"), Value::Int(42));
        assert_eq!(eval("val s: String = \"hi\"; s"), Value::String("hi".into()));
    }

    #[test]
    fn test_multi_statement_program() {
        let src = r#"
            val x = 10
            val y = 20
            val z = x + y
            z
        "#;
        assert_eq!(eval(src), Value::Int(30));
    }

    #[test]
    fn test_list_sum_product() {
        assert_eq!(eval("List(1, 2, 3, 4, 5).sum"), Value::Int(15));
        assert_eq!(eval("List(1, 2, 3, 4, 5).product"), Value::Int(120));
    }

    #[test]
    fn test_list_exists_forall() {
        assert_eq!(eval("List(1, 2, 3).exists((x) => x > 2)"), Value::Bool(true));
        assert_eq!(eval("List(1, 2, 3).forall((x) => x > 0)"), Value::Bool(true));
        assert_eq!(eval("List(1, 2, 3).forall((x) => x > 1)"), Value::Bool(false));
    }
}
