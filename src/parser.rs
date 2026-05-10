use crate::ast::*;
use crate::token::{Span, Token, TokenKind};

#[derive(Debug, Clone)]
pub struct ParseError {
    pub message: String,
    pub span: Span,
}

impl ParseError {
    pub fn new(message: impl Into<String>, span: Span) -> Self {
        ParseError { message: message.into(), span }
    }
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "parse error at {}: {}", self.span, self.message)
    }
}

impl std::error::Error for ParseError {}

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, pos: 0 }
    }

    pub fn parse(tokens: Vec<Token>) -> Result<Vec<Stmt>, ParseError> {
        let mut parser = Parser::new(tokens);
        let mut stmts = Vec::new();
        while !parser.is_at_end() {
            if parser.peek_kind() == Some(&TokenKind::Semicolon) {
                parser.advance();
                continue;
            }
            if let Some(stmt) = parser.parse_top_level_stmt()? {
                stmts.push(stmt);
            }
        }
        Ok(stmts)
    }

    pub fn parse_expr(tokens: Vec<Token>) -> Result<Expr, ParseError> {
        let mut parser = Parser::new(tokens);
        let expr = parser.parse_expression()?;
        Ok(expr)
    }

    fn is_at_end(&self) -> bool {
        self.pos >= self.tokens.len() || self.peek_kind() == Some(&TokenKind::Eof)
    }

    fn peek(&self) -> &Token {
        self.tokens.get(self.pos).unwrap_or_else(|| self.tokens.last().unwrap())
    }

    fn peek_kind(&self) -> Option<&TokenKind> {
        if self.pos < self.tokens.len() {
            Some(&self.tokens[self.pos].kind)
        } else {
            None
        }
    }

    fn peek_at(&self, offset: usize) -> &Token {
        let idx = self.pos + offset;
        self.tokens.get(idx).unwrap_or_else(|| self.tokens.last().unwrap())
    }

    fn advance(&mut self) -> Token {
        let tok = self.tokens.get(self.pos).cloned().unwrap_or_else(|| self.tokens.last().cloned().unwrap());
        if self.pos < self.tokens.len() {
            self.pos += 1;
        }
        tok
    }

    fn expect(&mut self, kind: &TokenKind) -> Result<Token, ParseError> {
        let tok = self.peek().clone();
        if &tok.kind == kind {
            self.advance();
            Ok(tok)
        } else {
            Err(ParseError::new(
                format!("expected {}, got {}", kind, tok.kind),
                tok.span,
            ))
        }
    }

    fn expect_identifier(&mut self) -> Result<(String, Span), ParseError> {
        let tok = self.peek().clone();
        match &tok.kind {
            TokenKind::Identifier(name) => {
                let name = name.clone();
                self.advance();
                Ok((name, tok.span))
            }
            _ => Err(ParseError::new(format!("expected identifier, got {}", tok.kind), tok.span)),
        }
    }

    fn skip_semicolons(&mut self) {
        while self.peek_kind() == Some(&TokenKind::Semicolon) {
            self.advance();
        }
    }

    fn current_span(&self) -> Span {
        self.peek().span.clone()
    }

    fn lookahead_typed_params(&self) -> bool {
        let offset = 0usize;
        loop {
            let tok = self.peek_at(offset);
            match &tok.kind {
                TokenKind::Identifier(_name) => {
                    let next = self.peek_at(offset + 1);
                    if next.kind == TokenKind::Colon {
                        let after_colon = self.peek_at(offset + 2);
                        match &after_colon.kind {
                            TokenKind::Identifier(_) | TokenKind::LeftBracket => {
                                return true;
                            }
                            _ => return false,
                        }
                    } else {
                        return false;
                    }
                }
                _ => return false,
            }
        }
    }

    fn parse_typed_params_list(&mut self) -> Result<Vec<Param>, ParseError> {
        let mut params = Vec::new();
        loop {
            let (name, _) = self.expect_identifier()?;
            let type_ann = if self.peek_kind() == Some(&TokenKind::Colon) {
                self.advance();
                Some(self.parse_type()?)
            } else {
                None
            };
            params.push(Param { name, type_ann, default: None });
            if self.peek_kind() == Some(&TokenKind::Comma) {
                self.advance();
            } else {
                break;
            }
        }
        Ok(params)
    }

    fn parse_top_level_stmt(&mut self) -> Result<Option<Stmt>, ParseError> {
        match self.peek_kind() {
            Some(TokenKind::Package) => {
                self.advance();
                let mut path = vec![self.expect_identifier()?.0];
                while self.peek_kind() == Some(&TokenKind::Dot) {
                    self.advance();
                    path.push(self.expect_identifier()?.0);
                }
                self.skip_semicolons();
                Ok(None)
            }
            Some(TokenKind::Import) => {
                let import = self.parse_import()?;
                Ok(Some(import))
            }
            Some(TokenKind::Val) => {
                let d = self.parse_val_decl()?;
                self.skip_semicolons();
                Ok(Some(d))
            }
            Some(TokenKind::Var) => {
                let d = self.parse_var_decl()?;
                self.skip_semicolons();
                Ok(Some(d))
            }
            Some(TokenKind::Def) => {
                let d = self.parse_def_decl()?;
                self.skip_semicolons();
                Ok(Some(Stmt::DefDecl(d)))
            }
            Some(TokenKind::Class) | Some(TokenKind::Case) => {
                let d = self.parse_class_decl()?;
                self.skip_semicolons();
                Ok(Some(Stmt::ClassDecl(d)))
            }
            Some(TokenKind::Trait) => {
                let d = self.parse_trait_decl()?;
                self.skip_semicolons();
                Ok(Some(Stmt::TraitDecl(d)))
            }
            Some(TokenKind::Object) => {
                let d = self.parse_object_decl()?;
                self.skip_semicolons();
                Ok(Some(Stmt::ObjectDecl(d)))
            }
            Some(TokenKind::Type) => {
                let d = self.parse_type_decl()?;
                self.skip_semicolons();
                Ok(Some(d))
            }
            Some(TokenKind::Abstract) => {
                self.advance();
                if self.peek_kind() == Some(&TokenKind::Class) {
                    let mut d = self.parse_class_decl()?;
                    d.is_abstract = true;
                    self.skip_semicolons();
                    Ok(Some(Stmt::ClassDecl(d)))
                } else {
                    Err(ParseError::new("expected 'class' after 'abstract'", self.current_span()))
                }
            }
            Some(TokenKind::Sealed) => {
                self.advance();
                if self.peek_kind() == Some(&TokenKind::Trait) {
                    self.parse_trait_decl().map(|d| {
                        self.skip_semicolons();
                        Some(Stmt::TraitDecl(d))
                    })
                } else if self.peek_kind() == Some(&TokenKind::Abstract) {
                    self.advance();
                    if self.peek_kind() == Some(&TokenKind::Class) {
                        let mut d = self.parse_class_decl()?;
                        d.is_abstract = true;
                        self.skip_semicolons();
                        Ok(Some(Stmt::ClassDecl(d)))
                    } else {
                        Err(ParseError::new("expected 'class'", self.current_span()))
                    }
                } else {
                    Err(ParseError::new("expected 'trait' or 'abstract class' after 'sealed'", self.current_span()))
                }
            }
            _ => {
                let expr = self.parse_expression()?;
                self.skip_semicolons();
                Ok(Some(Stmt::Expr(expr)))
            }
        }
    }

    fn parse_import(&mut self) -> Result<Stmt, ParseError> {
        let span = self.current_span();
        self.advance(); // consume 'import'
        let mut path = vec![self.expect_identifier()?.0];
        while self.peek_kind() == Some(&TokenKind::Dot) {
            self.advance();
            if self.peek_kind() == Some(&TokenKind::Underscore) {
                self.advance();
                path.push("_".into());
                return Ok(Stmt::ImportDecl {
                    path,
                    selectors: ImportSelectors::All,
                    span,
                });
            }
            path.push(self.expect_identifier()?.0);
        }
        let selectors = if self.peek_kind() == Some(&TokenKind::LeftBrace) {
            self.advance();
            let mut names = Vec::new();
            loop {
                names.push(self.expect_identifier()?.0);
                if self.peek_kind() == Some(&TokenKind::Comma) {
                    self.advance();
                } else {
                    break;
                }
            }
            self.expect(&TokenKind::RightBrace)?;
            ImportSelectors::Names(names)
        } else {
            ImportSelectors::All
        };
        Ok(Stmt::ImportDecl { path, selectors, span })
    }

    fn parse_val_decl(&mut self) -> Result<Stmt, ParseError> {
        let span = self.current_span();
        self.advance(); // consume 'val'
        let pattern = self.parse_pattern()?;
        let type_ann = if self.peek_kind() == Some(&TokenKind::Colon) {
            self.advance();
            Some(self.parse_type()?)
        } else {
            None
        };
        self.expect(&TokenKind::Equals)?;
        let value = self.parse_expression()?;
        Ok(Stmt::ValDecl { pattern, type_ann, value, span })
    }

    fn parse_var_decl(&mut self) -> Result<Stmt, ParseError> {
        let span = self.current_span();
        self.advance(); // consume 'var'
        let pattern = self.parse_pattern()?;
        let type_ann = if self.peek_kind() == Some(&TokenKind::Colon) {
            self.advance();
            Some(self.parse_type()?)
        } else {
            None
        };
        self.expect(&TokenKind::Equals)?;
        let value = self.parse_expression()?;
        Ok(Stmt::VarDecl { pattern, type_ann, value, span })
    }

    fn parse_def_decl(&mut self) -> Result<DefDecl, ParseError> {
        let span = self.current_span();
        self.advance(); // consume 'def'
        let (name, _) = self.expect_identifier()?;
        let type_params = self.parse_type_params_opt()?;
        let mut params = if self.peek_kind() == Some(&TokenKind::LeftParen) {
            self.parse_params()?
        } else {
            Vec::new()
        };

        while self.peek_kind() == Some(&TokenKind::LeftParen) {
            let extra_params = self.parse_params()?;
            let outer_body_span = self.current_span();
            let ret_type = if self.peek_kind() == Some(&TokenKind::Colon) {
                self.advance();
                let t = self.parse_type()?;
                Some(t)
            } else {
                None
            };
            let inner_body = if self.peek_kind() == Some(&TokenKind::Equals) {
                self.advance();
                self.parse_expression()?
            } else if self.peek_kind() == Some(&TokenKind::LeftBrace) {
                self.parse_block()?
            } else {
                break;
            };
            let _param_names: Vec<String> = extra_params.iter().map(|p| p.name.clone()).collect();
            params.push(Param {
                name: format!("__curried__{}", extra_params.len()),
                type_ann: None,
                default: None,
            });
            let fn_val = Expr::Lambda {
                params: extra_params,
                body: Box::new(inner_body),
                span: outer_body_span,
            };
            return Ok(DefDecl {
                name,
                type_params,
                params,
                return_type: ret_type,
                body: fn_val,
                span,
            });
        }

        let return_type = if self.peek_kind() == Some(&TokenKind::Colon) {
            self.advance();
            Some(self.parse_type()?)
        } else {
            None
        };
        let body = if self.peek_kind() == Some(&TokenKind::Equals) {
            self.advance();
            self.parse_expression()?
        } else if self.peek_kind() == Some(&TokenKind::LeftBrace) {
            self.parse_block()?
        } else {
            Expr::Literal { value: Literal::Unit, span: self.current_span() }
        };
        Ok(DefDecl { name, type_params, params, return_type, body, span })
    }

    fn parse_class_decl(&mut self) -> Result<ClassDecl, ParseError> {
        let span = self.current_span();
        let is_case = if self.peek_kind() == Some(&TokenKind::Case) {
            self.advance();
            true
        } else {
            false
        };
        self.advance(); // consume 'class'
        let (name, _) = self.expect_identifier()?;
        let type_params = self.parse_type_params_opt()?;
        let ctor_params = if self.peek_kind() == Some(&TokenKind::LeftParen) {
            self.parse_params()?
        } else {
            Vec::new()
        };
        let mut parents = Vec::new();
        if self.peek_kind() == Some(&TokenKind::Extends) {
            self.advance();
            let (parent_name, _) = self.expect_identifier()?;
            let parent_args = if self.peek_kind() == Some(&TokenKind::LeftParen) {
                self.parse_arguments()?
            } else {
                Vec::new()
            };
            parents.push((parent_name, parent_args));
            while self.peek_kind() == Some(&TokenKind::With) {
                self.advance();
                let (parent_name, _) = self.expect_identifier()?;
                let parent_args = if self.peek_kind() == Some(&TokenKind::LeftParen) {
                    self.parse_arguments()?
                } else {
                    Vec::new()
                };
                parents.push((parent_name, parent_args));
            }
        }
        let body = if self.peek_kind() == Some(&TokenKind::LeftBrace) {
            self.parse_class_body()?
        } else {
            Vec::new()
        };
        Ok(ClassDecl {
            name, type_params, ctor_params, parents, body, is_case, is_abstract: false, span,
        })
    }

    fn parse_trait_decl(&mut self) -> Result<TraitDecl, ParseError> {
        let span = self.current_span();
        self.advance(); // consume 'trait'
        let (name, _) = self.expect_identifier()?;
        let type_params = self.parse_type_params_opt()?;
        let mut parents = Vec::new();
        if self.peek_kind() == Some(&TokenKind::Extends) {
            self.advance();
            let (parent_name, _) = self.expect_identifier()?;
            parents.push(parent_name);
            while self.peek_kind() == Some(&TokenKind::With) {
                self.advance();
                let (parent_name, _) = self.expect_identifier()?;
                parents.push(parent_name);
            }
        }
        let body = if self.peek_kind() == Some(&TokenKind::LeftBrace) {
            self.parse_class_body()?
        } else {
            Vec::new()
        };
        Ok(TraitDecl { name, type_params, parents, body, span })
    }

    fn parse_object_decl(&mut self) -> Result<ObjectDecl, ParseError> {
        let span = self.current_span();
        self.advance(); // consume 'object'
        let (name, _) = self.expect_identifier()?;
        let mut parents = Vec::new();
        if self.peek_kind() == Some(&TokenKind::Extends) {
            self.advance();
            let (parent_name, _) = self.expect_identifier()?;
            let parent_args = if self.peek_kind() == Some(&TokenKind::LeftParen) {
                self.parse_arguments()?
            } else {
                Vec::new()
            };
            parents.push((parent_name, parent_args));
            while self.peek_kind() == Some(&TokenKind::With) {
                self.advance();
                let (parent_name, _) = self.expect_identifier()?;
                let parent_args = if self.peek_kind() == Some(&TokenKind::LeftParen) {
                    self.parse_arguments()?
                } else {
                    Vec::new()
                };
                parents.push((parent_name, parent_args));
            }
        }
        let body = if self.peek_kind() == Some(&TokenKind::LeftBrace) {
            self.parse_class_body()?
        } else {
            Vec::new()
        };
        Ok(ObjectDecl { name, parents, body, span })
    }

    fn parse_type_decl(&mut self) -> Result<Stmt, ParseError> {
        let span = self.current_span();
        self.advance(); // consume 'type'
        let (name, _) = self.expect_identifier()?;
        let type_params = self.parse_type_params_opt()?;
        self.expect(&TokenKind::Equals)?;
        let rhs = self.parse_type()?;
        Ok(Stmt::TypeDecl { name, type_params, rhs, span })
    }

    fn parse_class_body(&mut self) -> Result<Vec<Stmt>, ParseError> {
        self.expect(&TokenKind::LeftBrace)?;
        let mut stmts = Vec::new();
        while self.peek_kind() != Some(&TokenKind::RightBrace) && !self.is_at_end() {
            if self.peek_kind() == Some(&TokenKind::Semicolon) {
                self.advance();
                continue;
            }
            match self.peek_kind() {
                Some(TokenKind::Val) => {
                    stmts.push(self.parse_val_decl()?);
                }
                Some(TokenKind::Var) => {
                    stmts.push(self.parse_var_decl()?);
                }
                Some(TokenKind::Def) => {
                    stmts.push(Stmt::DefDecl(self.parse_def_decl()?));
                }
                Some(TokenKind::This) if self.peek_at(1).kind == TokenKind::LeftParen => {
                    self.advance(); // this
                    let _params = self.parse_params()?;
                    let _body = if self.peek_kind() == Some(&TokenKind::Equals) {
                        self.advance();
                        Some(self.parse_expression()?)
                    } else {
                        None
                    };
                }
                Some(TokenKind::Type) => {
                    stmts.push(self.parse_type_decl()?);
                }
                Some(TokenKind::Override) => {
                    self.advance();
                    if self.peek_kind() == Some(&TokenKind::Def) {
                        stmts.push(Stmt::DefDecl(self.parse_def_decl()?));
                    }
                }
                Some(TokenKind::Private) | Some(TokenKind::Protected) => {
                    self.advance();
                    if self.peek_kind() == Some(&TokenKind::Def) {
                        stmts.push(Stmt::DefDecl(self.parse_def_decl()?));
                    } else if self.peek_kind() == Some(&TokenKind::Val) {
                        stmts.push(self.parse_val_decl()?);
                    } else if self.peek_kind() == Some(&TokenKind::Var) {
                        stmts.push(self.parse_var_decl()?);
                    }
                }
                _ => {
                    let expr = self.parse_expression()?;
                    stmts.push(Stmt::Expr(expr));
                }
            }
            self.skip_semicolons();
        }
        self.expect(&TokenKind::RightBrace)?;
        Ok(stmts)
    }

    fn parse_params(&mut self) -> Result<Vec<Param>, ParseError> {
        self.expect(&TokenKind::LeftParen)?;
        let mut params = Vec::new();
        while self.peek_kind() != Some(&TokenKind::RightParen) {
            if self.peek_kind() == Some(&TokenKind::Val) || self.peek_kind() == Some(&TokenKind::Var) {
                self.advance();
            }
            let (name, _) = self.expect_identifier()?;
            let type_ann = if self.peek_kind() == Some(&TokenKind::Colon) {
                self.advance();
                if self.peek_kind() == Some(&TokenKind::Star) {
                    self.advance();
                    Some(TypeExpr::Parameterized {
                        base: Box::new(TypeExpr::simple("Seq")),
                        args: vec![self.parse_type()?],
                        span: Span::zero(),
                    })
                } else {
                    Some(self.parse_type()?)
                }
            } else {
                None
            };
            let default = if self.peek_kind() == Some(&TokenKind::Equals) {
                self.advance();
                Some(self.parse_expression()?)
            } else {
                None
            };
            params.push(Param { name, type_ann, default });
            if self.peek_kind() == Some(&TokenKind::Comma) {
                self.advance();
            } else {
                break;
            }
        }
        self.expect(&TokenKind::RightParen)?;
        Ok(params)
    }

    fn parse_type_params_opt(&mut self) -> Result<Vec<TypeParam>, ParseError> {
        if self.peek_kind() != Some(&TokenKind::LeftBracket) {
            return Ok(Vec::new());
        }
        self.advance();
        let mut params = Vec::new();
        while self.peek_kind() != Some(&TokenKind::RightBracket) {
            let variance = if self.peek_kind() == Some(&TokenKind::Plus) {
                self.advance();
                Variance::Covariant
            } else if self.peek_kind() == Some(&TokenKind::Minus) {
                self.advance();
                Variance::Contravariant
            } else {
                Variance::Invariant
            };
            let (name, _) = self.expect_identifier()?;
            let upper_bound = if self.peek_kind() == Some(&TokenKind::LessThan) {
                self.advance();
                Some(self.parse_type()?)
            } else {
                None
            };
            let lower_bound = if self.peek_kind() == Some(&TokenKind::GreaterThan) {
                self.advance();
                Some(self.parse_type()?)
            } else {
                None
            };
            params.push(TypeParam { name, variance, upper_bound, lower_bound });
            if self.peek_kind() == Some(&TokenKind::Comma) {
                self.advance();
            } else {
                break;
            }
        }
        self.expect(&TokenKind::RightBracket)?;
        Ok(params)
    }

    fn parse_type(&mut self) -> Result<TypeExpr, ParseError> {
        let span = self.current_span();
        let mut ty = self.parse_simple_type()?;
        if self.peek_kind() == Some(&TokenKind::Arrow) {
            self.advance();
            let result = self.parse_type()?;
            let params = match ty {
                TypeExpr::Tuple { elements, .. } if elements.len() > 1 => elements,
                other => vec![other],
            };
            ty = TypeExpr::Function {
                params,
                result: Box::new(result),
                span,
            };
        }
        Ok(ty)
    }

    fn parse_simple_type(&mut self) -> Result<TypeExpr, ParseError> {
        let span = self.current_span();
        if self.peek_kind() == Some(&TokenKind::LeftParen) {
            self.advance();
            if self.peek_kind() == Some(&TokenKind::RightParen) {
                self.advance();
                return Ok(TypeExpr::Tuple { elements: vec![], span });
            }
            let first = self.parse_type()?;
            if self.peek_kind() == Some(&TokenKind::Comma) {
                let mut elements = vec![first];
                while self.peek_kind() == Some(&TokenKind::Comma) {
                    self.advance();
                    elements.push(self.parse_type()?);
                }
                self.expect(&TokenKind::RightParen)?;
                return Ok(TypeExpr::Tuple { elements, span });
            }
            self.expect(&TokenKind::RightParen)?;
            return Ok(first);
        }

        if self.peek_kind() == Some(&TokenKind::Underscore) {
            self.advance();
            return Ok(TypeExpr::Wildcard {
                upper: None,
                lower: None,
                span,
            });
        }

        let (name, _) = self.expect_identifier()?;
        let mut ty = TypeExpr::Simple { name, span: span.clone() };

        if self.peek_kind() == Some(&TokenKind::LeftBracket) {
            self.advance();
            let mut args = Vec::new();
            while self.peek_kind() != Some(&TokenKind::RightBracket) {
                args.push(self.parse_type()?);
                if self.peek_kind() == Some(&TokenKind::Comma) {
                    self.advance();
                } else {
                    break;
                }
            }
            self.expect(&TokenKind::RightBracket)?;
            ty = TypeExpr::Parameterized { base: Box::new(ty), args, span };
        }

        Ok(ty)
    }

    fn parse_arguments(&mut self) -> Result<Vec<Expr>, ParseError> {
        self.expect(&TokenKind::LeftParen)?;
        let mut args = Vec::new();
        while self.peek_kind() != Some(&TokenKind::RightParen) {
            args.push(self.parse_expression()?);
            if self.peek_kind() == Some(&TokenKind::Comma) {
                self.advance();
            } else {
                break;
            }
        }
        self.expect(&TokenKind::RightParen)?;
        Ok(args)
    }

    fn parse_expression(&mut self) -> Result<Expr, ParseError> {
        let _span = self.current_span();
        let expr = self.parse_assignment()?;
        Ok(expr)
    }

    fn parse_assignment(&mut self) -> Result<Expr, ParseError> {
        let span = self.current_span();
        let expr = self.parse_or()?;
        if self.peek_kind() == Some(&TokenKind::Equals) {
            self.advance();
            let value = self.parse_assignment()?;
            Ok(Expr::Assign {
                target: Box::new(expr),
                value: Box::new(value),
                span,
            })
        } else {
            Ok(expr)
        }
    }

    fn parse_or(&mut self) -> Result<Expr, ParseError> {
        let span = self.current_span();
        let mut left = self.parse_and()?;
        while self.peek_kind() == Some(&TokenKind::PipePipe) {
            self.advance();
            let right = self.parse_and()?;
            left = Expr::Binary {
                left: Box::new(left),
                op: BinOp::Or,
                right: Box::new(right),
                span,
            };
        }
        Ok(left)
    }

    fn parse_and(&mut self) -> Result<Expr, ParseError> {
        let span = self.current_span();
        let mut left = self.parse_equality()?;
        while self.peek_kind() == Some(&TokenKind::AmpersandAmpersand) {
            self.advance();
            let right = self.parse_equality()?;
            left = Expr::Binary {
                left: Box::new(left),
                op: BinOp::And,
                right: Box::new(right),
                span,
            };
        }
        Ok(left)
    }

    fn parse_equality(&mut self) -> Result<Expr, ParseError> {
        let span = self.current_span();
        let mut left = self.parse_comparison()?;
        loop {
            let op = match self.peek_kind() {
                Some(TokenKind::EqualsEquals) => BinOp::Eq,
                Some(TokenKind::BangEquals) => BinOp::Neq,
                _ => break,
            };
            self.advance();
            let right = self.parse_comparison()?;
            left = Expr::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
                span,
            };
        }
        Ok(left)
    }

    fn parse_comparison(&mut self) -> Result<Expr, ParseError> {
        let span = self.current_span();
        let mut left = self.parse_additive()?;
        loop {
            let op = match self.peek_kind() {
                Some(TokenKind::LessThan) => BinOp::Lt,
                Some(TokenKind::GreaterThan) => BinOp::Gt,
                Some(TokenKind::LessEquals) => BinOp::Leq,
                Some(TokenKind::GreaterEquals) => BinOp::Geq,
                _ => break,
            };
            self.advance();
            let right = self.parse_additive()?;
            left = Expr::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
                span,
            };
        }
        Ok(left)
    }

    fn parse_additive(&mut self) -> Result<Expr, ParseError> {
        let span = self.current_span();
        let mut left = self.parse_multiplicative()?;
        loop {
            let op = match self.peek_kind() {
                Some(TokenKind::Plus) => BinOp::Add,
                Some(TokenKind::Minus) => BinOp::Sub,
                Some(TokenKind::Pipe) => BinOp::BitOr,
                Some(TokenKind::Caret) => BinOp::BitXor,
                _ => break,
            };
            self.advance();
            let right = self.parse_multiplicative()?;
            left = Expr::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
                span,
            };
        }
        Ok(left)
    }

    fn parse_multiplicative(&mut self) -> Result<Expr, ParseError> {
        let span = self.current_span();
        let mut left = self.parse_shift()?;
        loop {
            let op = match self.peek_kind() {
                Some(TokenKind::Star) => BinOp::Mul,
                Some(TokenKind::Slash) => BinOp::Div,
                Some(TokenKind::Percent) => BinOp::Mod,
                Some(TokenKind::Ampersand) => BinOp::BitAnd,
                _ => break,
            };
            self.advance();
            let right = self.parse_shift()?;
            left = Expr::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
                span,
            };
        }
        Ok(left)
    }

    fn parse_shift(&mut self) -> Result<Expr, ParseError> {
        let span = self.current_span();
        let mut left = self.parse_unary()?;
        loop {
            let op = match self.peek_kind() {
                Some(TokenKind::LeftShift) => BinOp::LeftShift,
                Some(TokenKind::RightShift) => BinOp::RightShift,
                Some(TokenKind::UnsignedRightShift) => BinOp::UnsignedRightShift,
                _ => break,
            };
            self.advance();
            let right = self.parse_unary()?;
            left = Expr::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
                span,
            };
        }
        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<Expr, ParseError> {
        let span = self.current_span();
        match self.peek_kind() {
            Some(TokenKind::Minus) => {
                self.advance();
                let operand = self.parse_unary()?;
                Ok(Expr::Unary {
                    op: UnaryOp::Negate,
                    operand: Box::new(operand),
                    span,
                })
            }
            Some(TokenKind::Plus) => {
                self.advance();
                let operand = self.parse_unary()?;
                Ok(Expr::Unary {
                    op: UnaryOp::Positive,
                    operand: Box::new(operand),
                    span,
                })
            }
            Some(TokenKind::Exclaim) => {
                self.advance();
                let operand = self.parse_unary()?;
                Ok(Expr::Unary {
                    op: UnaryOp::Not,
                    operand: Box::new(operand),
                    span,
                })
            }
            Some(TokenKind::Tilde) => {
                self.advance();
                let operand = self.parse_unary()?;
                Ok(Expr::Unary {
                    op: UnaryOp::BitNot,
                    operand: Box::new(operand),
                    span,
                })
            }
            _ => self.parse_postfix(),
        }
    }

    fn parse_postfix(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.parse_primary()?;
        loop {
            let span = self.current_span();
            match self.peek_kind() {
                Some(TokenKind::Dot) => {
                    self.advance();
                    let (name, _) = self.expect_identifier()?;
                    if self.peek_kind() == Some(&TokenKind::LeftParen) {
                        let args = self.parse_arguments()?;
                        expr = Expr::MethodCall {
                            receiver: Box::new(expr),
                            method: name,
                            args,
                            span,
                        };
                    } else if self.peek_kind() == Some(&TokenKind::Underscore) {
                        self.advance();
                    } else {
                        expr = Expr::FieldAccess {
                            receiver: Box::new(expr),
                            field: name,
                            span,
                        };
                    }
                }
                Some(TokenKind::LeftParen) => {
                    let args = self.parse_arguments()?;
                    expr = Expr::Apply {
                        func: Box::new(expr),
                        args,
                        span,
                    };
                }
                Some(TokenKind::UnitLiteral) => {
                    self.advance();
                    expr = Expr::Apply {
                        func: Box::new(expr),
                        args: vec![],
                        span,
                    };
                }
                Some(TokenKind::LeftBracket) => {
                    self.advance();
                    let mut type_args = Vec::new();
                    while self.peek_kind() != Some(&TokenKind::RightBracket) {
                        type_args.push(self.parse_type()?);
                        if self.peek_kind() == Some(&TokenKind::Comma) {
                            self.advance();
                        } else {
                            break;
                        }
                    }
                    self.expect(&TokenKind::RightBracket)?;
                    expr = Expr::TypeApply {
                        expr: Box::new(expr),
                        type_args,
                        span,
                    };
                }
                Some(TokenKind::Match) => {
                    self.advance();
                    let cases = self.parse_match_cases()?;
                    expr = Expr::Match {
                        scrutinee: Box::new(expr),
                        cases,
                        span,
                    };
                }
                _ => break,
            }
        }
        Ok(expr)
    }

    fn parse_primary(&mut self) -> Result<Expr, ParseError> {
        let span = self.current_span();
        match self.peek_kind().cloned() {
            Some(TokenKind::IntLiteral(v)) => {
                self.advance();
                Ok(Expr::Literal { value: Literal::Int(v), span })
            }
            Some(TokenKind::LongLiteral(v)) => {
                self.advance();
                Ok(Expr::Literal { value: Literal::Long(v), span })
            }
            Some(TokenKind::DoubleLiteral(v)) => {
                self.advance();
                Ok(Expr::Literal { value: Literal::Double(v), span })
            }
            Some(TokenKind::FloatLiteral(v)) => {
                self.advance();
                Ok(Expr::Literal { value: Literal::Float(v), span })
            }
            Some(TokenKind::BoolLiteral(b)) => {
                self.advance();
                Ok(Expr::Literal { value: Literal::Bool(b), span })
            }
            Some(TokenKind::StringLiteral(s)) => {
                self.advance();
                Ok(Expr::Literal { value: Literal::String(s), span })
            }
            Some(TokenKind::CharLiteral(c)) => {
                self.advance();
                Ok(Expr::Literal { value: Literal::Char(c), span })
            }
            Some(TokenKind::NullLiteral) => {
                self.advance();
                Ok(Expr::Literal { value: Literal::Null, span })
            }
            Some(TokenKind::UnitLiteral) => {
                self.advance();
                if self.peek_kind() == Some(&TokenKind::Arrow) {
                    self.advance();
                    let body = self.parse_expression()?;
                    return Ok(Expr::Lambda {
                        params: vec![],
                        body: Box::new(body),
                        span,
                    });
                }
                Ok(Expr::Literal { value: Literal::Unit, span })
            }
            Some(TokenKind::Identifier(name)) => {
                self.advance();
                let name_clone = name.clone();
                if self.peek_kind() == Some(&TokenKind::Arrow) {
                    self.advance();
                    let body = self.parse_expression()?;
                    return Ok(Expr::Lambda {
                        params: vec![Param {
                            name: name_clone,
                            type_ann: None,
                            default: None,
                        }],
                        body: Box::new(body),
                        span,
                    });
                }
                Ok(Expr::Identifier { name, span })
            }
            Some(TokenKind::Underscore) => {
                self.advance();
                if self.peek_kind() == Some(&TokenKind::Arrow) {
                    self.advance();
                    let body = self.parse_expression()?;
                    return Ok(Expr::Lambda {
                        params: vec![Param {
                            name: "_".into(),
                            type_ann: None,
                            default: None,
                        }],
                        body: Box::new(body),
                        span,
                    });
                }
                Ok(Expr::Identifier { name: "_".into(), span })
            }
            Some(TokenKind::This) => {
                self.advance();
                Ok(Expr::This(span))
            }
            Some(TokenKind::Super) => {
                self.advance();
                Ok(Expr::Super(span))
            }
            Some(TokenKind::If) => self.parse_if(),
            Some(TokenKind::While) => self.parse_while(),
            Some(TokenKind::Do) => self.parse_do_while(),
            Some(TokenKind::For) => self.parse_for(),
            Some(TokenKind::Return) => {
                self.advance();
                let value = if !matches!(self.peek_kind(),
                    Some(TokenKind::Semicolon) |
                    Some(TokenKind::RightBrace) |
                    Some(TokenKind::Eof)
                ) {
                    Some(Box::new(self.parse_expression()?))
                } else {
                    None
                };
                Ok(Expr::Return { value, span })
            }
            Some(TokenKind::Throw) => {
                self.advance();
                let value = self.parse_expression()?;
                Ok(Expr::Throw { value: Box::new(value), span })
            }
            Some(TokenKind::Try) => self.parse_try(),
            Some(TokenKind::New) => self.parse_new(),
            Some(TokenKind::LeftBrace) => self.parse_block(),
            Some(TokenKind::LeftParen) => {
                self.advance();
                if self.peek_kind() == Some(&TokenKind::RightParen) {
                    self.advance();
                    if self.peek_kind() == Some(&TokenKind::Arrow) {
                        self.advance();
                        let body = self.parse_expression()?;
                        return Ok(Expr::Lambda {
                            params: vec![],
                            body: Box::new(body),
                            span,
                        });
                    }
                    return Ok(Expr::Literal { value: Literal::Unit, span });
                }
                let is_typed_params = self.lookahead_typed_params();
                if is_typed_params {
                    let params = self.parse_typed_params_list()?;
                    self.expect(&TokenKind::RightParen)?;
                    self.expect(&TokenKind::Arrow)?;
                    let body = self.parse_expression()?;
                    return Ok(Expr::Lambda { params, body: Box::new(body), span });
                }
                let first = self.parse_expression()?;
                if self.peek_kind() == Some(&TokenKind::Comma) {
                    let mut elements = vec![first];
                    while self.peek_kind() == Some(&TokenKind::Comma) {
                        self.advance();
                        elements.push(self.parse_expression()?);
                    }
                    self.expect(&TokenKind::RightParen)?;
                    if self.peek_kind() == Some(&TokenKind::Arrow) {
                        self.advance();
                        let body = self.parse_expression()?;
                        let params: Vec<Param> = elements.iter().map(|e| Param {
                            name: match e {
                                Expr::Identifier { name, .. } => name.clone(),
                                _ => "_".into(),
                            },
                            type_ann: None,
                            default: None,
                        }).collect();
                        return Ok(Expr::Lambda { params, body: Box::new(body), span });
                    }
                    return Ok(Expr::Tuple { elements, span });
                }
                self.expect(&TokenKind::RightParen)?;
                if self.peek_kind() == Some(&TokenKind::Arrow) {
                    self.advance();
                    let body = self.parse_expression()?;
                    Ok(Expr::Lambda {
                        params: vec![Param {
                            name: match &first {
                                Expr::Identifier { name, .. } => name.clone(),
                                _ => "_".into(),
                            },
                            type_ann: None,
                            default: None,
                        }],
                        body: Box::new(body),
                        span,
                    })
                } else {
                    Ok(Expr::Paren { expr: Box::new(first), span })
                }
            }
            Some(TokenKind::Operator(op)) if op == "::" || op == ":+:" || op == "#::" => {
                self.advance();
                let right = self.parse_postfix()?;
                Ok(Expr::MethodCall {
                    receiver: Box::new(right),
                    method: op,
                    args: vec![],
                    span,
                })
            }
            _ => {
                let tok = self.peek().clone();
                Err(ParseError::new(
                    format!("unexpected token: {}", tok.kind),
                    tok.span,
                ))
            }
        }
    }

    fn parse_if(&mut self) -> Result<Expr, ParseError> {
        let span = self.current_span();
        self.advance(); // consume 'if'
        self.expect(&TokenKind::LeftParen)?;
        let cond = self.parse_expression()?;
        self.expect(&TokenKind::RightParen)?;
        let then_branch = self.parse_expression()?;
        let else_branch = if self.peek_kind() == Some(&TokenKind::Else) {
            self.advance();
            Some(Box::new(self.parse_expression()?))
        } else {
            None
        };
        Ok(Expr::If {
            cond: Box::new(cond),
            then_branch: Box::new(then_branch),
            else_branch,
            span,
        })
    }

    fn parse_while(&mut self) -> Result<Expr, ParseError> {
        let span = self.current_span();
        self.advance(); // consume 'while'
        self.expect(&TokenKind::LeftParen)?;
        let cond = self.parse_expression()?;
        self.expect(&TokenKind::RightParen)?;
        let body = self.parse_expression()?;
        Ok(Expr::While {
            cond: Box::new(cond),
            body: Box::new(body),
            span,
        })
    }

    fn parse_do_while(&mut self) -> Result<Expr, ParseError> {
        let span = self.current_span();
        self.advance(); // consume 'do'
        let body = self.parse_expression()?;
        self.expect(&TokenKind::While)?;
        self.expect(&TokenKind::LeftParen)?;
        let cond = self.parse_expression()?;
        self.expect(&TokenKind::RightParen)?;
        Ok(Expr::DoWhile {
            body: Box::new(body),
            cond: Box::new(cond),
            span,
        })
    }

    fn parse_for(&mut self) -> Result<Expr, ParseError> {
        let span = self.current_span();
        self.advance(); // consume 'for'
        let (enums, _enum_span) = if self.peek_kind() == Some(&TokenKind::LeftBrace) {
            self.advance();
            let enums = self.parse_enumerators()?;
            self.expect(&TokenKind::RightBrace)?;
            (enums, span.clone())
        } else {
            self.expect(&TokenKind::LeftParen)?;
            let enums = self.parse_enumerators()?;
            self.expect(&TokenKind::RightParen)?;
            (enums, span.clone())
        };
        let is_yield = if self.peek_kind() == Some(&TokenKind::Yield) {
            self.advance();
            true
        } else {
            false
        };
        let body = self.parse_expression()?;
        Ok(Expr::For {
            enumerators: enums,
            body: Box::new(body),
            is_yield,
            span,
        })
    }

    fn parse_enumerators(&mut self) -> Result<Vec<Enumerator>, ParseError> {
        let mut enums = Vec::new();
        loop {
            let span = self.current_span();
            if self.peek_kind() == Some(&TokenKind::Val) {
                self.advance();
                let pattern = self.parse_pattern()?;
                self.expect(&TokenKind::Equals)?;
                let expr = self.parse_expression()?;
                enums.push(Enumerator::Val { pattern, expr, span });
            } else if self.peek_kind() == Some(&TokenKind::If) {
                self.advance();
                let cond = self.parse_expression()?;
                enums.push(Enumerator::Filter { cond, span });
            } else {
                let pattern = self.parse_pattern()?;
                if self.peek_kind() == Some(&TokenKind::LeftArrow) {
                    self.advance();
                    let expr = self.parse_expression()?;
                    enums.push(Enumerator::Generator { pattern, expr, span });
                } else {
                    break;
                }
            }
            let needs_sep = matches!(self.peek_kind(),
                Some(TokenKind::Semicolon) | Some(TokenKind::Val) |
                Some(TokenKind::If) | Some(TokenKind::Identifier(_)) |
                Some(TokenKind::Underscore)
            );
            if self.peek_kind() == Some(&TokenKind::Semicolon) {
                self.advance();
            } else if !needs_sep {
                break;
            }
        }
        Ok(enums)
    }

    fn parse_try(&mut self) -> Result<Expr, ParseError> {
        let span = self.current_span();
        self.advance(); // consume 'try'
        let body = self.parse_expression()?;
        let catches = if self.peek_kind() == Some(&TokenKind::Catch) {
            self.advance();
            self.parse_match_cases()?
        } else {
            Vec::new()
        };
        let finally_block = if self.peek_kind() == Some(&TokenKind::Finally) {
            self.advance();
            Some(Box::new(self.parse_expression()?))
        } else {
            None
        };
        Ok(Expr::Try {
            body: Box::new(body),
            catches,
            finally_block,
            span,
        })
    }

    fn parse_new(&mut self) -> Result<Expr, ParseError> {
        let span = self.current_span();
        self.advance(); // consume 'new'
        let (class_name, _) = self.expect_identifier()?;
        let type_args = if self.peek_kind() == Some(&TokenKind::LeftBracket) {
            self.advance();
            let mut args = Vec::new();
            while self.peek_kind() != Some(&TokenKind::RightBracket) {
                args.push(self.parse_type()?);
                if self.peek_kind() == Some(&TokenKind::Comma) {
                    self.advance();
                } else {
                    break;
                }
            }
            self.expect(&TokenKind::RightBracket)?;
            args
        } else {
            Vec::new()
        };
        let args = if self.peek_kind() == Some(&TokenKind::LeftParen) {
            self.parse_arguments()?
        } else {
            Vec::new()
        };
        Ok(Expr::New { class_name, type_args, args, span })
    }

    fn parse_block(&mut self) -> Result<Expr, ParseError> {
        let span = self.current_span();
        self.expect(&TokenKind::LeftBrace)?;
        let mut stmts = Vec::new();
        while self.peek_kind() != Some(&TokenKind::RightBrace) && !self.is_at_end() {
            if self.peek_kind() == Some(&TokenKind::Semicolon) {
                self.advance();
                continue;
            }
            match self.peek_kind() {
                Some(TokenKind::Val) => {
                    stmts.push(self.parse_val_decl()?);
                }
                Some(TokenKind::Var) => {
                    stmts.push(self.parse_var_decl()?);
                }
                Some(TokenKind::Def) => {
                    stmts.push(Stmt::DefDecl(self.parse_def_decl()?));
                }
                Some(TokenKind::Class) | Some(TokenKind::Case) => {
                    stmts.push(Stmt::ClassDecl(self.parse_class_decl()?));
                }
                Some(TokenKind::Trait) => {
                    stmts.push(Stmt::TraitDecl(self.parse_trait_decl()?));
                }
                Some(TokenKind::Object) => {
                    stmts.push(Stmt::ObjectDecl(self.parse_object_decl()?));
                }
                Some(TokenKind::Import) => {
                    stmts.push(self.parse_import()?);
                }
                _ => {
                    stmts.push(Stmt::Expr(self.parse_expression()?));
                }
            }
            self.skip_semicolons();
        }
        self.expect(&TokenKind::RightBrace)?;
        Ok(Expr::Block { stmts, span })
    }

    fn parse_match_cases(&mut self) -> Result<Vec<MatchCase>, ParseError> {
        let expect_brace = self.peek_kind() == Some(&TokenKind::LeftBrace);
        if expect_brace {
            self.advance();
        }
        let mut cases = Vec::new();
        while !self.is_at_end() {
            if expect_brace && self.peek_kind() == Some(&TokenKind::RightBrace) {
                self.advance();
                break;
            }
            if self.peek_kind() == Some(&TokenKind::Case) {
                self.advance();
            } else {
                break;
            }
            let pattern = self.parse_pattern()?;
            let guard = if self.peek_kind() == Some(&TokenKind::If) {
                self.advance();
                Some(self.parse_expression()?)
            } else {
                None
            };
            self.expect(&TokenKind::Arrow)?;
            let body = self.parse_expression()?;
            cases.push(MatchCase {
                pattern,
                guard,
                body,
                span: Span::zero(),
            });
            self.skip_semicolons();
        }
        Ok(cases)
    }

    fn parse_pattern(&mut self) -> Result<Pattern, ParseError> {
        self.parse_pattern_alternative()
    }

    fn parse_pattern_alternative(&mut self) -> Result<Pattern, ParseError> {
        let span = self.current_span();
        let mut pat = self.parse_pattern_base()?;
        while self.peek_kind() == Some(&TokenKind::Pipe) {
            self.advance();
            let right = self.parse_pattern_base()?;
            pat = Pattern::Alternative {
                left: Box::new(pat),
                right: Box::new(right),
                span,
            };
        }
        Ok(pat)
    }

    fn parse_pattern_base(&mut self) -> Result<Pattern, ParseError> {
        let span = self.current_span();
        match self.peek_kind().cloned() {
            Some(TokenKind::Underscore) => {
                self.advance();
                Ok(Pattern::Wildcard(span))
            }
            Some(TokenKind::IntLiteral(v)) => {
                self.advance();
                Ok(Pattern::Literal { value: Literal::Int(v), span })
            }
            Some(TokenKind::LongLiteral(v)) => {
                self.advance();
                Ok(Pattern::Literal { value: Literal::Long(v), span })
            }
            Some(TokenKind::DoubleLiteral(v)) => {
                self.advance();
                Ok(Pattern::Literal { value: Literal::Double(v), span })
            }
            Some(TokenKind::FloatLiteral(v)) => {
                self.advance();
                Ok(Pattern::Literal { value: Literal::Float(v), span })
            }
            Some(TokenKind::BoolLiteral(b)) => {
                self.advance();
                Ok(Pattern::Literal { value: Literal::Bool(b), span })
            }
            Some(TokenKind::StringLiteral(s)) => {
                self.advance();
                Ok(Pattern::Literal { value: Literal::String(s), span })
            }
            Some(TokenKind::CharLiteral(c)) => {
                self.advance();
                Ok(Pattern::Literal { value: Literal::Char(c), span })
            }
            Some(TokenKind::NullLiteral) => {
                self.advance();
                Ok(Pattern::Literal { value: Literal::Null, span })
            }
            Some(TokenKind::Minus) => {
                self.advance();
                match self.peek_kind().cloned() {
                    Some(TokenKind::IntLiteral(v)) => {
                        self.advance();
                        Ok(Pattern::Literal { value: Literal::Int(-v), span })
                    }
                    Some(TokenKind::DoubleLiteral(v)) => {
                        self.advance();
                        Ok(Pattern::Literal { value: Literal::Double(-v), span })
                    }
                    _ => Err(ParseError::new("expected number after '-'", self.current_span())),
                }
            }
            Some(TokenKind::Identifier(name)) => {
                self.advance();
                let name_clone = name.clone();
                if self.peek_kind() == Some(&TokenKind::LeftParen) {
                    self.advance();
                    let mut args = Vec::new();
                    while self.peek_kind() != Some(&TokenKind::RightParen) {
                        if self.peek_kind() == Some(&TokenKind::Underscore) && self.peek_at(1).kind == TokenKind::Star {
                            self.advance();
                            self.advance();
                            args.push(Pattern::SequenceWildcard(self.current_span()));
                        } else {
                            args.push(self.parse_pattern()?);
                        }
                        if self.peek_kind() == Some(&TokenKind::Comma) {
                            self.advance();
                        } else {
                            break;
                        }
                    }
                    self.expect(&TokenKind::RightParen)?;
                    Ok(Pattern::Constructor { name, args, span })
                } else if self.peek_kind() == Some(&TokenKind::Colon) {
                    self.advance();
                    let type_ann = self.parse_type()?;
                    Ok(Pattern::Typed {
                        pattern: Box::new(Pattern::Variable { name: name_clone, span: span.clone() }),
                        type_ann,
                        span,
                    })
                } else {
                    Ok(Pattern::Variable { name, span })
                }
            }
            Some(TokenKind::LeftParen) => {
                self.advance();
                if self.peek_kind() == Some(&TokenKind::RightParen) {
                    self.advance();
                    return Ok(Pattern::Tuple { elements: vec![], span });
                }
                let first = self.parse_pattern()?;
                if self.peek_kind() == Some(&TokenKind::Comma) {
                    let mut elements = vec![first];
                    while self.peek_kind() == Some(&TokenKind::Comma) {
                        self.advance();
                        elements.push(self.parse_pattern()?);
                    }
                    self.expect(&TokenKind::RightParen)?;
                    Ok(Pattern::Tuple { elements, span })
                } else {
                    self.expect(&TokenKind::RightParen)?;
                    Ok(first)
                }
            }
            _ => Err(ParseError::new(
                format!("unexpected token in pattern: {}", self.peek().kind),
                self.current_span(),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;

    fn parse_stmts(source: &str) -> Vec<Stmt> {
        let tokens = Lexer::tokenize(source).unwrap();
        Parser::parse(tokens).unwrap()
    }

    fn parse_one_expr(source: &str) -> Expr {
        let tokens = Lexer::tokenize(source).unwrap();
        Parser::parse_expr(tokens).unwrap()
    }

    #[test]
    fn test_literal_int() {
        if let Expr::Literal { value: Literal::Int(42), .. } = parse_one_expr("42") {
        } else {
            panic!("expected int literal 42");
        }
    }

    #[test]
    fn test_binary_add() {
        if let Expr::Binary { op: BinOp::Add, .. } = parse_one_expr("1 + 2") {
        } else {
            panic!("expected binary add");
        }
    }

    #[test]
    fn test_precedence() {
        if let Expr::Binary { op: BinOp::Add, left: _, right, .. } = parse_one_expr("1 + 2 * 3") {
            if let Expr::Binary { op: BinOp::Mul, .. } = *right {
            } else {
                panic!("expected mul on right");
            }
        } else {
            panic!("expected binary add");
        }
    }

    #[test]
    fn test_if_else() {
        match parse_one_expr("if (true) 1 else 2") {
            Expr::If { .. } => {}
            _ => panic!("expected if expression"),
        }
    }

    #[test]
    fn test_block() {
        match parse_one_expr("{ val x = 1; x }") {
            Expr::Block { stmts, .. } => assert_eq!(stmts.len(), 2),
            _ => panic!("expected block"),
        }
    }

    #[test]
    fn test_lambda() {
        match parse_one_expr("(x) => x + 1") {
            Expr::Lambda { params, .. } => assert_eq!(params.len(), 1),
            _ => panic!("expected lambda"),
        }
    }

    #[test]
    fn test_val_decl() {
        let stmts = parse_stmts("val x = 42");
        match &stmts[0] {
            Stmt::ValDecl { pattern, value, .. } => {
                if let Pattern::Variable { name, .. } = pattern {
                    assert_eq!(name, "x");
                }
                if let Expr::Literal { value: Literal::Int(42), .. } = value {
                } else {
                    panic!("expected int 42");
                }
            }
            _ => panic!("expected val decl"),
        }
    }

    #[test]
    fn test_def_decl() {
        let stmts = parse_stmts("def add(a: Int, b: Int): Int = a + b");
        match &stmts[0] {
            Stmt::DefDecl(DefDecl { name, params, return_type, .. }) => {
                assert_eq!(name, "add");
                assert_eq!(params.len(), 2);
                assert!(return_type.is_some());
            }
            _ => panic!("expected def decl"),
        }
    }

    #[test]
    fn test_class_decl() {
        let stmts = parse_stmts("class Point(val x: Int, val y: Int)");
        match &stmts[0] {
            Stmt::ClassDecl(ClassDecl { name, ctor_params, .. }) => {
                assert_eq!(name, "Point");
                assert_eq!(ctor_params.len(), 2);
            }
            _ => panic!("expected class decl"),
        }
    }

    #[test]
    fn test_case_class() {
        let stmts = parse_stmts("case class Foo(x: Int, y: String)");
        match &stmts[0] {
            Stmt::ClassDecl(ClassDecl { name, is_case, .. }) => {
                assert_eq!(name, "Foo");
                assert!(is_case);
            }
            _ => panic!("expected case class"),
        }
    }

    #[test]
    fn test_match_expr() {
        match parse_one_expr("x match { case 0 => 1 case _ => 2 }") {
            Expr::Match { cases, .. } => assert_eq!(cases.len(), 2),
            _ => panic!("expected match"),
        }
    }

    #[test]
    fn test_method_call() {
        match parse_one_expr("obj.method(1, 2)") {
            Expr::MethodCall { method, args, .. } => {
                assert_eq!(method, "method");
                assert_eq!(args.len(), 2);
            }
            _ => panic!("expected method call"),
        }
    }

    #[test]
    fn test_tuple() {
        match parse_one_expr("(1, 2, 3)") {
            Expr::Tuple { elements, .. } => assert_eq!(elements.len(), 3),
            _ => panic!("expected tuple"),
        }
    }

    #[test]
    fn test_new_expr() {
        match parse_one_expr("new Point(1, 2)") {
            Expr::New { class_name, args, .. } => {
                assert_eq!(class_name, "Point");
                assert_eq!(args.len(), 2);
            }
            _ => panic!("expected new"),
        }
    }

    #[test]
    fn test_for_comprehension() {
        match parse_one_expr("for (x <- List(1, 2)) yield x") {
            Expr::For { is_yield, enumerators, .. } => {
                assert!(is_yield);
                assert_eq!(enumerators.len(), 1);
            }
            _ => panic!("expected for"),
        }
    }

    #[test]
    fn test_trait_decl() {
        let stmts = parse_stmts("trait Greeter { def greet(name: String): String }");
        match &stmts[0] {
            Stmt::TraitDecl(TraitDecl { name, body, .. }) => {
                assert_eq!(name, "Greeter");
                assert_eq!(body.len(), 1);
            }
            _ => panic!("expected trait"),
        }
    }

    #[test]
    fn test_object_decl() {
        let stmts = parse_stmts("object Math { def square(x: Int): Int = x * x }");
        match &stmts[0] {
            Stmt::ObjectDecl(ObjectDecl { name, body, .. }) => {
                assert_eq!(name, "Math");
                assert_eq!(body.len(), 1);
            }
            _ => panic!("expected object"),
        }
    }

    #[test]
    fn test_type_param() {
        let stmts = parse_stmts("class Box[T](val content: T)");
        match &stmts[0] {
            Stmt::ClassDecl(ClassDecl { name, type_params, .. }) => {
                assert_eq!(name, "Box");
                assert_eq!(type_params.len(), 1);
            }
            _ => panic!("expected class"),
        }
    }
}
