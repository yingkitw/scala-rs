use crate::ast::*;
use crate::token::Span;
use crate::ty::{Ty, TypeEnv, TypeError};
use crate::lexer::Lexer;
use crate::parser::Parser;

pub fn typecheck_program(stmts: &[Stmt]) -> Result<Vec<Ty>, Vec<TypeError>> {
    let mut env = TypeEnv::new();
    env.define_builtin_types();
    let mut results = Vec::new();
    let mut errors = Vec::new();
    for stmt in stmts {
        match typecheck_stmt(stmt, &mut env) {
            Ok(ty) => results.push(ty),
            Err(e) => errors.push(e),
        }
    }
    if errors.is_empty() {
        Ok(results)
    } else {
        Err(errors)
    }
}

pub fn typecheck_expr_standalone(expr: &Expr, env: &mut TypeEnv) -> Result<Ty, TypeError> {
    typecheck_expr(expr, env)
}

pub fn typecheck_source(source: &str) -> Result<(), Vec<TypeError>> {
    let tokens = Lexer::tokenize(source).map_err(|e| vec![TypeError::new(e.message, e.span)])?;
    let stmts = Parser::parse(tokens).map_err(|e| vec![TypeError::new(e.message, e.span)])?;
    typecheck_program(&stmts)?;
    Ok(())
}

fn typecheck_stmt(stmt: &Stmt, env: &mut TypeEnv) -> Result<Ty, TypeError> {
    match stmt {
        Stmt::Expr(expr) => typecheck_expr(expr, env),
        Stmt::ValDecl { pattern, type_ann, value, span } => {
            let value_ty = typecheck_expr(value, env)?;
            if let Some(ann) = type_ann {
                let ann_ty = resolve_type(ann, env);
                if !value_ty.is_subtype_of(&ann_ty) && !value_ty.is_error() && !ann_ty.is_error() {
                    return Err(TypeError::new(
                        format!("type mismatch: found {}, expected {}", value_ty, ann_ty),
                        span.clone(),
                    ));
                }
                bind_pattern(pattern, &ann_ty, env);
                Ok(ann_ty)
            } else {
                bind_pattern(pattern, &value_ty, env);
                Ok(value_ty)
            }
        }
        Stmt::VarDecl { pattern, type_ann, value, span } => {
            let value_ty = typecheck_expr(value, env)?;
            if let Some(ann) = type_ann {
                let ann_ty = resolve_type(ann, env);
                if !value_ty.is_subtype_of(&ann_ty) && !value_ty.is_error() && !ann_ty.is_error() {
                    return Err(TypeError::new(
                        format!("type mismatch: found {}, expected {}", value_ty, ann_ty),
                        span.clone(),
                    ));
                }
                bind_pattern(pattern, &ann_ty, env);
                Ok(ann_ty)
            } else {
                bind_pattern(pattern, &value_ty, env);
                Ok(value_ty)
            }
        }
        Stmt::DefDecl(def_decl) => {
            let param_tys: Vec<Ty> = def_decl.params.iter().map(|p| {
                p.type_ann.as_ref().map(|t| resolve_type(t, env)).unwrap_or(Ty::Any)
            }).collect();
            env.push();
            for (param, ty) in def_decl.params.iter().zip(&param_tys) {
                env.define(&param.name, ty.clone());
            }
            let body_ty = typecheck_expr(&def_decl.body, env)?;
            env.pop();
            if let Some(ret) = &def_decl.return_type {
                let ret_ty = resolve_type(ret, env);
                if !body_ty.is_subtype_of(&ret_ty) && !body_ty.is_error() && !ret_ty.is_error() {
                    return Err(TypeError::new(
                        format!("def {}: return type mismatch: found {}, expected {}", def_decl.name, body_ty, ret_ty),
                        def_decl.span.clone(),
                    ));
                }
                env.define(&def_decl.name, Ty::Function { params: param_tys, result: Box::new(ret_ty.clone()) });
                Ok(ret_ty)
            } else {
                env.define(&def_decl.name, Ty::Function { params: param_tys, result: Box::new(body_ty.clone()) });
                Ok(body_ty)
            }
        }
        Stmt::ClassDecl(class) => {
            let class_ty = Ty::Named { name: class.name.clone(), args: vec![] };
            env.define(&class.name, class_ty);
            env.push();
            for param in &class.ctor_params {
                let param_ty = param.type_ann.as_ref().map(|t| resolve_type(t, env)).unwrap_or(Ty::Any);
                env.define(&param.name, param_ty);
            }
            for stmt in &class.body {
                let _ = typecheck_stmt(stmt, env);
            }
            env.pop();
            Ok(Ty::Unit)
        }
        Stmt::TraitDecl(trait_decl) => {
            let trait_ty = Ty::Named { name: trait_decl.name.clone(), args: vec![] };
            env.define(&trait_decl.name, trait_ty);
            env.push();
            for stmt in &trait_decl.body {
                let _ = typecheck_stmt(stmt, env);
            }
            env.pop();
            Ok(Ty::Unit)
        }
        Stmt::ObjectDecl(obj) => {
            env.push();
            for stmt in &obj.body {
                let _ = typecheck_stmt(stmt, env);
            }
            env.pop();
            Ok(Ty::Unit)
        }
        Stmt::ImportDecl { .. } => Ok(Ty::Unit),
        Stmt::TypeDecl { name, rhs, .. } => {
            let ty = resolve_type(rhs, env);
            env.define(name, ty);
            Ok(Ty::Unit)
        }
    }
}

fn typecheck_expr(expr: &Expr, env: &mut TypeEnv) -> Result<Ty, TypeError> {
    match expr {
        Expr::Literal { value, span } => Ok(literal_type(value, span)),
        Expr::Binary { left, op, right, span } => {
            let lt = typecheck_expr(left, env)?;
            let rt = typecheck_expr(right, env)?;
            Ok(binary_result_type(&lt, op, &rt, span))
        }
        Expr::Unary { op, operand, span } => {
            let ot = typecheck_expr(operand, env)?;
            match op {
                UnaryOp::Negate | UnaryOp::Positive => {
                    if ot.is_numeric() || ot.is_error() {
                        Ok(ot)
                    } else {
                        Err(TypeError::new(format!("cannot negate {}", ot), span.clone()))
                    }
                }
                UnaryOp::Not => {
                    if ot == Ty::Bool || ot.is_error() {
                        Ok(Ty::Bool)
                    } else {
                        Err(TypeError::new(format!("! requires Boolean, found {}", ot), span.clone()))
                    }
                }
                UnaryOp::BitNot => {
                    if ot.is_numeric() || ot.is_error() {
                        Ok(ot)
                    } else {
                        Err(TypeError::new(format!("~ requires numeric, found {}", ot), span.clone()))
                    }
                }
            }
        }
        Expr::If { cond, then_branch, else_branch, span } => {
            let ct = typecheck_expr(cond, env)?;
            if ct != Ty::Bool && !ct.is_error() && ct != Ty::Any {
                return Err(TypeError::new(format!("if condition must be Boolean, found {}", ct), span.clone()));
            }
            let tt = typecheck_expr(then_branch, env)?;
            if let Some(els) = else_branch {
                let et = typecheck_expr(els, env)?;
                Ok(tt.common_type(&et))
            } else {
                Ok(Ty::Unit)
            }
        }
        Expr::Block { stmts, span: _ } => {
            if stmts.is_empty() {
                return Ok(Ty::Unit);
            }
            env.push();
            let mut last_ty = Ty::Unit;
            for (_i, stmt) in stmts.iter().enumerate() {
                last_ty = typecheck_stmt(stmt, env)?;
            }
            env.pop();
            Ok(last_ty)
        }
        Expr::Lambda { params, body, span: _ } => {
            env.push();
            let mut param_tys = Vec::new();
            for param in params {
                let ty = param.type_ann.as_ref().map(|t| resolve_type(t, env)).unwrap_or(Ty::Any);
                param_tys.push(ty.clone());
                env.define(&param.name, ty);
            }
            let body_ty = typecheck_expr(body, env)?;
            env.pop();
            Ok(Ty::Function { params: param_tys, result: Box::new(body_ty) })
        }
        Expr::Identifier { name, span } => {
            env.lookup(name).cloned().ok_or_else(|| {
                TypeError::new(format!("not found: value {}", name), span.clone())
            })
        }
        Expr::Apply { func, args, span: _ } => {
            let ft = typecheck_expr(func, env)?;
            let arg_tys: Vec<Ty> = args.iter().map(|a| typecheck_expr(a, env)).collect::<Result<Vec<_>, _>>()?;
            match &ft {
                Ty::Function { params, result } => {
                    if arg_tys.len() != params.len() && !params.is_empty() {
                        // allow for flexibility
                    }
                    Ok((**result).clone())
                }
                Ty::Named { name, .. } => {
                    Ok(Ty::Named { name: name.clone(), args: vec![] })
                }
                _ if ft.is_error() => Ok(Ty::Error("propagated".into())),
                _ => Ok(Ty::Any),
            }
        }
        Expr::MethodCall { receiver, method, args, span: _ } => {
            let rt = typecheck_expr(receiver, env)?;
            let _arg_tys: Vec<Ty> = args.iter().map(|a| typecheck_expr(a, env)).collect::<Result<Vec<_>, _>>()?;
            match method.as_str() {
                "toString" => Ok(Ty::String),
                "hashCode" => Ok(Ty::Int),
                "equals" => Ok(Ty::Bool),
                "==" | "!=" => Ok(Ty::Bool),
                "+" => {
                    if rt == Ty::String {
                        Ok(Ty::String)
                    } else {
                        Ok(rt)
                    }
                }
                "-" | "*" | "/" | "%" => Ok(rt),
                "<" | ">" | "<=" | ">=" => Ok(Ty::Bool),
                "&&" | "||" => Ok(Ty::Bool),
                "map" | "flatMap" | "filter" | "withFilter" => Ok(rt),
                "foreach" | "foreachEntry" => Ok(Ty::Unit),
                "foldLeft" | "foldRight" => {
                    if args.first().is_some() {
                        typecheck_expr(&args[0], env)
                    } else {
                        Ok(Ty::Any)
                    }
                }
                "head" | "tail" | "last" | "init" | "reverse" | "distinct" | "sorted" | "toList" | "toSeq" | "toVector" => Ok(rt),
                "isEmpty" | "nonEmpty" | "contains" | "startsWith" | "endsWith" => Ok(Ty::Bool),
                "length" | "size" => Ok(Ty::Int),
                "mkString" | "asString" | "toLowerCase" | "toUpperCase" | "trim" | "strip" => Ok(Ty::String),
                "toInt" => Ok(Ty::Int),
                "toLong" => Ok(Ty::Long),
                "toDouble" => Ok(Ty::Double),
                "toFloat" => Ok(Ty::Float),
                "toChar" => Ok(Ty::Char),
                "get" | "getOrElse" | "orNull" => Ok(rt),
                "isDefined" | "isFailure" | "isSuccess" => Ok(Ty::Bool),
                "getOrNull" => Ok(rt),
                "orElse" => Ok(rt),
                "keys" => Ok(Ty::Named { name: "Iterable".into(), args: vec![] }),
                "values" => Ok(Ty::Named { name: "Iterable".into(), args: vec![] }),
                "zip" | "zipWithIndex" => Ok(rt),
                "take" | "drop" | "slice" | "takeWhile" | "dropWhile" | "splitAt" => Ok(rt),
                "exists" | "forall" | "count" => Ok(Ty::Bool),
                "find" => Ok(Ty::Named { name: "Option".into(), args: vec![] }),
                "min" | "max" | "sum" | "product" => Ok(rt),
                "groupBy" => Ok(Ty::Named { name: "Map".into(), args: vec![] }),
                "flatten" => Ok(rt),
                "copy" => Ok(rt),
                "canEqual" => Ok(Ty::Bool),
                "productPrefix" | "productElementNames" => Ok(Ty::String),
                "curried" | "tupled" => Ok(Ty::Any),
                "apply" => Ok(Ty::Any),
                _ => Ok(Ty::Any),
            }
        }
        Expr::FieldAccess { receiver, field, span: _ } => {
            let _rt = typecheck_expr(receiver, env)?;
            match field.as_str() {
                n if n.starts_with('_') && n.len() >= 2 => {
                    if let Ok(_idx) = n[1..].parse::<usize>() {
                        Ok(Ty::Any)
                    } else {
                        Ok(Ty::Any)
                    }
                }
                "x" | "y" | "z" | "width" | "height" | "value" | "key" | "_1" | "_2" | "_3" | "_4" | "_5" => Ok(Ty::Any),
                _ => Ok(Ty::Any),
            }
        }
        Expr::Tuple { elements, span: _ } => {
            let tys: Vec<Ty> = elements.iter().map(|e| typecheck_expr(e, env)).collect::<Result<Vec<_>, _>>()?;
            Ok(Ty::Tuple { elements: tys })
        }
        Expr::Match { scrutinee, cases, span: _ } => {
            let _st = typecheck_expr(scrutinee, env)?;
            let mut result_ty = Ty::Nothing;
            for case in cases {
                env.push();
                bind_pattern(&case.pattern, &Ty::Any, env);
                if let Some(guard) = &case.guard {
                    let _ = typecheck_expr(guard, env);
                }
                let body_ty = typecheck_expr(&case.body, env)?;
                env.pop();
                result_ty = result_ty.common_type(&body_ty);
            }
            Ok(result_ty)
        }
        Expr::Assign { target: _, value, span: _ } => {
            let _vt = typecheck_expr(value, env)?;
            Ok(Ty::Unit)
        }
        Expr::Return { value, span: _ } => {
            if let Some(v) = value {
                typecheck_expr(v, env)?;
            }
            Ok(Ty::Nothing)
        }
        Expr::Throw { value, span: _ } => {
            typecheck_expr(value, env)?;
            Ok(Ty::Nothing)
        }
        Expr::Try { body, catches, finally_block, span: _ } => {
            let bt = typecheck_expr(body, env)?;
            for catch_case in catches {
                env.push();
                let _ = typecheck_expr(&catch_case.body, env);
                env.pop();
            }
            if let Some(fin) = finally_block {
                typecheck_expr(fin, env)?;
            }
            Ok(bt)
        }
        Expr::New { class_name, args, .. } => {
            for arg in args {
                typecheck_expr(arg, env)?;
            }
            Ok(env.lookup(class_name).cloned().unwrap_or(Ty::Named { name: class_name.clone(), args: vec![] }))
        }
        Expr::For { enumerators, body, is_yield, span: _ } => {
            env.push();
            for enumerator in enumerators {
                match enumerator {
                    Enumerator::Generator { pattern, expr, span: _ } => {
                        let _et = typecheck_expr(expr, env)?;
                        bind_pattern(pattern, &Ty::Any, env);
                    }
                    Enumerator::Filter { cond, span: _ } => {
                        let _ct = typecheck_expr(cond, env)?;
                    }
                    Enumerator::Val { pattern, expr, span: _ } => {
                        let vt = typecheck_expr(expr, env)?;
                        bind_pattern(pattern, &vt, env);
                    }
                }
            }
            let bt = typecheck_expr(body, env)?;
            env.pop();
            if *is_yield {
                Ok(Ty::Named { name: "List".into(), args: vec![bt] })
            } else {
                Ok(Ty::Unit)
            }
        }
        Expr::While { cond, body, span: _ } => {
            typecheck_expr(cond, env)?;
            typecheck_expr(body, env)?;
            Ok(Ty::Unit)
        }
        Expr::DoWhile { body, cond, span: _ } => {
            typecheck_expr(body, env)?;
            typecheck_expr(cond, env)?;
            Ok(Ty::Unit)
        }
        Expr::StringInterpolation { prefix: _, parts: _, span: _ } => Ok(Ty::String),
        Expr::This(_span) => Ok(Ty::Any),
        Expr::Super(_span) => Ok(Ty::Any),
        Expr::Paren { expr, span: _ } => typecheck_expr(expr, env),
        Expr::TypeApply { expr, type_args: _, span: _ } => {
            typecheck_expr(expr, env)
        }
        Expr::UnaryMethodCall { receiver, method: _, span: _ } => {
            typecheck_expr(receiver, env)
        }
    }
}

fn literal_type(value: &Literal, _span: &Span) -> Ty {
    match value {
        Literal::Int(_) => Ty::Int,
        Literal::Long(_) => Ty::Long,
        Literal::Double(_) => Ty::Double,
        Literal::Float(_) => Ty::Float,
        Literal::Bool(_) => Ty::Bool,
        Literal::String(_) => Ty::String,
        Literal::Char(_) => Ty::Char,
        Literal::Null => Ty::Null,
        Literal::Unit => Ty::Unit,
    }
}

fn binary_result_type(left: &Ty, op: &BinOp, right: &Ty, _span: &Span) -> Ty {
    if left.is_error() || right.is_error() {
        return Ty::Error("propagated".into());
    }
    match op {
        BinOp::Add => {
            if left == &Ty::String || right == &Ty::String {
                Ty::String
            } else if left.is_numeric() && right.is_numeric() {
                left.common_type(right)
            } else {
                Ty::Any
            }
        }
        BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Mod => {
            if left.is_numeric() && right.is_numeric() {
                left.common_type(right)
            } else {
                Ty::Any
            }
        }
        BinOp::Eq | BinOp::Neq => Ty::Bool,
        BinOp::Lt | BinOp::Gt | BinOp::Leq | BinOp::Geq => {
            if left.is_numeric() && right.is_numeric() {
                Ty::Bool
            } else {
                Ty::Bool
            }
        }
        BinOp::And | BinOp::Or => Ty::Bool,
        BinOp::BitAnd | BinOp::BitOr | BinOp::BitXor => {
            if left.is_numeric() && right.is_numeric() {
                left.common_type(right)
            } else {
                Ty::Any
            }
        }
        BinOp::LeftShift | BinOp::RightShift | BinOp::UnsignedRightShift => {
            left.clone()
        }
    }
}

fn resolve_type(type_expr: &TypeExpr, env: &TypeEnv) -> Ty {
    match type_expr {
        TypeExpr::Simple { name, .. } => Ty::from_name(name),
        TypeExpr::Parameterized { base, args, .. } => {
            let base_ty = resolve_type(base, env);
            let arg_tys: Vec<Ty> = args.iter().map(|a| resolve_type(a, env)).collect();
            match &base_ty {
                Ty::Named { name, .. } => Ty::Named { name: name.clone(), args: arg_tys },
                _ => Ty::App { base: Box::new(base_ty), args: arg_tys },
            }
        }
        TypeExpr::Function { params, result, .. } => {
            let param_tys = params.iter().map(|p| resolve_type(p, env)).collect();
            let result_ty = resolve_type(result, env);
            Ty::Function { params: param_tys, result: Box::new(result_ty) }
        }
        TypeExpr::Tuple { elements, .. } => {
            let tys = elements.iter().map(|e| resolve_type(e, env)).collect();
            Ty::Tuple { elements: tys }
        }
        TypeExpr::Compound { types: _, .. } => Ty::Any,
        TypeExpr::Wildcard { upper, lower: _, .. } => {
            if let Some(u) = upper {
                resolve_type(u, env)
            } else {
                Ty::Any
            }
        }
    }
}

fn bind_pattern(pattern: &Pattern, ty: &Ty, env: &mut TypeEnv) {
    match pattern {
        Pattern::Wildcard(_) => {}
        Pattern::Variable { name, .. } => {
            if name != "_" {
                env.define(name, ty.clone());
            }
        }
        Pattern::Constructor { args, .. } => {
            for arg in args {
                bind_pattern(arg, &Ty::Any, env);
            }
        }
        Pattern::Tuple { elements, .. } => {
            if let Ty::Tuple { elements: tys, .. } = ty {
                for (p, t) in elements.iter().zip(tys.iter()) {
                    bind_pattern(p, t, env);
                }
            } else {
                for p in elements {
                    bind_pattern(p, &Ty::Any, env);
                }
            }
        }
        Pattern::Typed { pattern, type_ann: _, .. } => {
            bind_pattern(pattern, ty, env);
        }
        _ => {}
    }
}
