use tsnat_common::diagnostic::{TsnatResult, TsnatError};
use tsnat_common::interner::Interner;
use tsnat_common::span::Span;
use tsnat_lex::token::{Token, TokenKind};
use crate::ast::*;
use crate::arena::{AstArena, NodeList};

pub struct Parser<'src, 'arena> {
    tokens: &'src [Token],
    pos: usize,
    arena: AstArena<'arena>,
    interner: &'src mut Interner,
}

impl<'src, 'arena> Parser<'src, 'arena> {
    pub fn new(
        tokens: &'src [Token],
        arena: AstArena<'arena>,
        interner: &'src mut Interner,
    ) -> Self {
        Self { tokens, pos: 0, arena, interner }
    }

    // ── Public entry ──────────────────────────────────────────

    pub fn parse_program(&mut self) -> TsnatResult<Program<'arena>> {
        let mut stmts = Vec::new();
        while !self.is_at_end() {
            stmts.push(self.parse_stmt()?);
        }
        let span = self.tokens.first()
            .map(|t| t.span)
            .unwrap_or(Span::DUMMY);
        Ok(Program {
            stmts: self.arena.alloc_slice_fill_iter(stmts),
            span,
            source_type: SourceType::Script,
        })
    }

    // ── Statements (minimal for Phase 1) ──────────────────────

    fn parse_stmt(&mut self) -> TsnatResult<Stmt<'arena>> {
        match self.peek().kind {
            TokenKind::LBrace => Ok(Stmt::Block(self.parse_block_stmt()?)),
            TokenKind::KwConst | TokenKind::KwLet | TokenKind::KwVar | TokenKind::KwUsing => {
                Ok(Stmt::Var(self.parse_var_decl()?))
            }
            TokenKind::KwIf => Ok(Stmt::If(self.parse_if_stmt()?)),
            TokenKind::KwWhile => Ok(Stmt::While(self.parse_while_stmt()?)),
            TokenKind::KwDo => Ok(Stmt::DoWhile(self.parse_do_while_stmt()?)),
            TokenKind::KwFor => {
                if self.peek_ahead(1).kind == TokenKind::KwAwait {
                    Ok(Stmt::ForOf(self.parse_for_of_stmt(true)?))
                } else {
                    self.parse_for_any_stmt()
                }
            }
            TokenKind::KwDebugger => {
                let span = self.advance().span;
                self.match_kind(TokenKind::Semicolon);
                Ok(Stmt::Debugger(span))
            }
            TokenKind::KwReturn => Ok(Stmt::Return(self.parse_return_stmt()?)),
            TokenKind::KwThrow => Ok(Stmt::Throw(self.parse_throw_stmt()?)),
            TokenKind::KwBreak => Ok(Stmt::Break(self.parse_break_stmt()?)),
            TokenKind::KwContinue => Ok(Stmt::Continue(self.parse_continue_stmt()?)),
            TokenKind::KwTry => Ok(Stmt::Try(self.parse_try_stmt()?)),
            TokenKind::KwSwitch => Ok(Stmt::Switch(self.parse_switch_stmt()?)),
            TokenKind::KwFunction => {
                Ok(Stmt::Function(self.parse_function_decl(true)?))
            }
            TokenKind::KwAsync => {
                if self.peek_ahead(1).kind == TokenKind::KwFunction {
                    Ok(Stmt::Function(self.parse_function_decl(true)?))
                } else {
                    let expr = self.parse_expr()?;
                    let span = expr.span();
                    self.match_kind(TokenKind::Semicolon);
                    let expr = self.alloc(expr);
                    Ok(Stmt::Expr(ExprStmt { expr, span }))
                }
            }
            TokenKind::KwClass => Ok(Stmt::Class(self.parse_class_decl(true)?)),
            TokenKind::KwImport => {
                if self.peek_ahead(1).kind == TokenKind::KwNative {
                    Ok(Stmt::NativeImport(self.parse_native_import_decl()?))
                } else {
                    Ok(Stmt::Import(self.parse_import_decl()?))
                }
            }
            TokenKind::KwDeclare => {
                if self.peek_ahead(1).kind == TokenKind::KwNative && self.peek_ahead(2).kind == TokenKind::KwFunction {
                    Ok(Stmt::NativeFunction(self.parse_native_function_decl()?))
                } else {
                    let span = self.peek().span;
                    Err(TsnatError::Parse {
                        message: "Expected 'native function'".into(),
                        span,
                    })
                }
            }
            TokenKind::KwExport => Ok(Stmt::Export(self.parse_export_decl()?)),
            TokenKind::Ident if self.peek_ahead(1).kind == TokenKind::Colon => {
                Ok(Stmt::Labeled(self.parse_labeled_stmt()?))
            }
            _ => {
                let expr = self.parse_expr()?;
                let span = expr.span();
                // Consume optional semicolon
                self.match_kind(TokenKind::Semicolon);
                let expr = self.alloc(expr);
                Ok(Stmt::Expr(ExprStmt { expr, span }))
            }
        }
    }

    fn parse_block_stmt(&mut self) -> TsnatResult<BlockStmt<'arena>> {
        let start = self.expect(TokenKind::LBrace)?.span;
        let mut stmts = Vec::new();
        while self.peek().kind != TokenKind::RBrace && !self.is_at_end() {
            stmts.push(self.parse_stmt()?);
        }
        let end = self.expect(TokenKind::RBrace)?.span;
        Ok(BlockStmt {
            stmts: self.arena.alloc_slice_fill_iter(stmts),
            span: start.merge(end),
        })
    }

    fn parse_if_stmt(&mut self) -> TsnatResult<IfStmt<'arena>> {
        let start = self.expect(TokenKind::KwIf)?.span;
        self.expect(TokenKind::LParen)?;
        let test = self.parse_expr()?;
        let test = self.alloc(test);
        self.expect(TokenKind::RParen)?;
        let consequent = self.parse_stmt()?;
        let consequent = self.alloc(consequent);
        let mut span = start.merge(consequent.span());
        let alternate = if self.match_kind(TokenKind::KwElse) {
            let alt = self.parse_stmt()?;
            let alt = self.alloc(alt);
            span = span.merge(alt.span());
            Some(alt)
        } else {
            None
        };
        Ok(IfStmt { test, consequent, alternate, span })
    }

    fn parse_while_stmt(&mut self) -> TsnatResult<WhileStmt<'arena>> {
        let start = self.expect(TokenKind::KwWhile)?.span;
        self.expect(TokenKind::LParen)?;
        let test = self.parse_expr()?;
        let test = self.alloc(test);
        self.expect(TokenKind::RParen)?;
        let body = self.parse_stmt()?;
        let body = self.alloc(body);
        let span = start.merge(body.span());
        Ok(WhileStmt { test, body, span })
    }

    fn parse_do_while_stmt(&mut self) -> TsnatResult<DoWhileStmt<'arena>> {
        let start = self.expect(TokenKind::KwDo)?.span;
        let body = self.parse_stmt()?;
        let body = self.alloc(body);
        self.expect(TokenKind::KwWhile)?;
        self.expect(TokenKind::LParen)?;
        let test = self.parse_expr()?;
        let test = self.alloc(test);
        let end = self.expect(TokenKind::RParen)?.span;
        self.match_kind(TokenKind::Semicolon);
        Ok(DoWhileStmt { body, test, span: start.merge(end) })
    }

    fn parse_for_any_stmt(&mut self) -> TsnatResult<Stmt<'arena>> {
        let start = self.expect(TokenKind::KwFor)?.span;
        self.expect(TokenKind::LParen)?;
        
        // Disambiguate for(init; test; update) vs for(x in/of y)
        if self.peek().kind == TokenKind::Semicolon {
            return Ok(Stmt::For(self.parse_for_triplet_tail(start, None)?));
        }

        let init = if self.is_var_kind(self.peek().kind) {
            ForInit::Var(self.parse_var_decl_no_semi()?)
        } else {
            let e = self.parse_expr()?;
            ForInit::Expr(self.alloc(e))
        };

        match self.peek().kind {
            TokenKind::KwIn => {
                self.advance();
                let right = self.parse_expr()?;
                let right = self.alloc(right);
                self.expect(TokenKind::RParen)?;
                let body = self.parse_stmt()?;
                let body = self.alloc(body);
                Ok(Stmt::ForIn(ForInStmt { left: init, right, body, span: start.merge(body.span()) }))
            }
            TokenKind::KwOf => {
                self.advance();
                let right = self.parse_expr()?;
                let right = self.alloc(right);
                self.expect(TokenKind::RParen)?;
                let body = self.parse_stmt()?;
                let body = self.alloc(body);
                Ok(Stmt::ForOf(ForOfStmt { is_await: false, left: init, right, body, span: start.merge(body.span()) }))
            }
            _ => {
                Ok(Stmt::For(self.parse_for_triplet_tail(start, Some(init))?))
            }
        }
    }

    fn parse_for_of_stmt(&mut self, is_await: bool) -> TsnatResult<ForOfStmt<'arena>> {
        let start = self.expect(TokenKind::KwFor)?.span;
        if is_await { self.expect(TokenKind::KwAwait)?; }
        self.expect(TokenKind::LParen)?;
        let init = if self.is_var_kind(self.peek().kind) {
            ForInit::Var(self.parse_var_decl_no_semi()?)
        } else {
            let e = self.parse_expr()?;
            ForInit::Expr(self.alloc(e))
        };
        self.expect(TokenKind::KwOf)?;
        let right = self.parse_expr()?;
        let right = self.alloc(right);
        self.expect(TokenKind::RParen)?;
        let body = self.parse_stmt()?;
        let body = self.alloc(body);
        Ok(ForOfStmt { is_await, left: init, right, body, span: start.merge(body.span()) })
    }

    fn parse_for_triplet_tail(&mut self, start: Span, init: Option<ForInit<'arena>>) -> TsnatResult<ForStmt<'arena>> {
        self.expect(TokenKind::Semicolon)?;
        let test = if self.peek().kind != TokenKind::Semicolon {
            let e = self.parse_expr()?;
            Some(self.alloc(e))
        } else {
            None
        };
        self.expect(TokenKind::Semicolon)?;
        let update = if self.peek().kind != TokenKind::RParen {
            let e = self.parse_expr()?;
            Some(self.alloc(e))
        } else {
            None
        };
        self.expect(TokenKind::RParen)?;
        let body = self.parse_stmt()?;
        let body = self.alloc(body);
        Ok(ForStmt { init, test, update, body, span: start.merge(body.span()) })
    }

    fn is_var_kind(&self, kind: TokenKind) -> bool {
        matches!(kind, TokenKind::KwConst | TokenKind::KwLet | TokenKind::KwVar | TokenKind::KwUsing)
    }

    fn parse_binding_pattern(&mut self) -> TsnatResult<(BindingPattern<'arena>, Span)> {
        match self.peek().kind {
            TokenKind::Ident => {
                let tok = self.advance();
                Ok((BindingPattern::Ident(tok.value), tok.span))
            }
            TokenKind::LBracket => {
                let start = self.advance().span;
                let mut elements = Vec::new();
                while !self.match_kind(TokenKind::RBracket) {
                    if self.match_kind(TokenKind::Comma) {
                        elements.push(BindingElement { pattern: None, default_val: None, is_rest: false });
                        continue;
                    }
                    let is_rest = self.match_kind(TokenKind::DotDotDot);
                    let (pat, _) = self.parse_binding_pattern()?;
                    let pat = self.alloc(pat);
                    let default_val: Option<&'arena Expr<'arena>> = if !is_rest && self.match_kind(TokenKind::Eq) {
                        let e = self.parse_assignment_expr()?;
                        Some(self.alloc(e))
                    } else {
                        None
                    };
                    elements.push(BindingElement { pattern: Some(pat), default_val, is_rest });
                    if is_rest {
                        self.expect(TokenKind::RBracket)?;
                        break;
                    }
                    if !self.match_kind(TokenKind::Comma) {
                        self.expect(TokenKind::RBracket)?;
                        break;
                    }
                }
                let end = self.tokens[self.pos - 1].span;
                Ok((BindingPattern::Array(self.arena.alloc_slice_fill_iter(elements)), start.merge(end)))
            }
            TokenKind::LBrace => {
                let start = self.advance().span;
                let mut props = Vec::new();
                while !self.match_kind(TokenKind::RBrace) {
                    let is_rest = self.match_kind(TokenKind::DotDotDot);
                    if is_rest {
                        let key = self.expect(TokenKind::Ident)?.value;
                        props.push(ObjectBindingProp { key, pattern: None, default_val: None, is_rest: true });
                        self.expect(TokenKind::RBrace)?;
                        break;
                    }
                    let key_tok = self.expect(TokenKind::Ident)?;
                    let key = key_tok.value;
                    let pattern = if self.match_kind(TokenKind::Colon) {
                        let (pat, _) = self.parse_binding_pattern()?;
                        Some(self.alloc(pat) as &_)
                    } else {
                        None
                    };
                    let default_val: Option<&'arena Expr<'arena>> = if self.match_kind(TokenKind::Eq) {
                        let e = self.parse_assignment_expr()?;
                        Some(self.alloc(e))
                    } else {
                        None
                    };
                    props.push(ObjectBindingProp { key, pattern, default_val, is_rest: false });
                    if !self.match_kind(TokenKind::Comma) {
                        self.expect(TokenKind::RBrace)?;
                        break;
                    }
                }
                let end = self.tokens[self.pos - 1].span;
                Ok((BindingPattern::Object(self.arena.alloc_slice_fill_iter(props)), start.merge(end)))
            }
            _ => Err(TsnatError::Parse {
                message: format!("Expected binding pattern, found {:?}", self.peek().kind),
                span: self.peek().span,
            })
        }
    }

    fn parse_var_decl_no_semi(&mut self) -> TsnatResult<VarDecl<'arena>> {
        let start = self.peek().span;
        let kind = match self.advance().kind {
            TokenKind::KwConst => VarKind::Const,
            TokenKind::KwLet => VarKind::Let,
            TokenKind::KwVar => VarKind::Var,
            TokenKind::KwUsing => VarKind::Using,
            _ => unreachable!(),
        };

        let mut decls = Vec::new();
        loop {
            let (pattern, pat_span) = self.parse_binding_pattern()?;

            let ty = if self.match_kind(TokenKind::Colon) {
                Some(self.parse_type_node()?)
            } else {
                None
            };

            let init: Option<&'arena Expr<'arena>> = if self.match_kind(TokenKind::Eq) {
                let e = self.parse_assignment_expr()?;
                Some(self.alloc(e))
            } else {
                None
            };
            let mut span = pat_span;
            if let Some(ref t) = ty { span = span.merge(t.span()); }
            if let Some(i) = init { span = span.merge(i.span()); }

            decls.push(VarDeclarator { pattern, ty, init, span });
            if !self.match_kind(TokenKind::Comma) { break; }
        }

        let end = decls.last().unwrap().span;
        Ok(VarDecl {
            kind,
            decls: self.arena.alloc_slice_fill_iter(decls),
            span: start.merge(end),
        })
    }

    fn parse_var_decl(&mut self) -> TsnatResult<VarDecl<'arena>> {
        let decl = self.parse_var_decl_no_semi()?;
        self.expect_semi()?;
        Ok(decl)
    }

    fn parse_return_stmt(&mut self) -> TsnatResult<ReturnStmt<'arena>> {
        let start = self.expect(TokenKind::KwReturn)?.span;
        let value = if !self.peek().has_preceding_newline && self.peek().kind != TokenKind::Semicolon && self.peek().kind != TokenKind::RBrace && self.peek().kind != TokenKind::Eof {
            let e = self.parse_expr()?;
            Some(self.alloc(e))
        } else {
            None
        };
        let end = if let Some(v) = value { v.span() } else { start };
        self.match_kind(TokenKind::Semicolon);
        Ok(ReturnStmt { value, span: start.merge(end) })
    }

    fn parse_throw_stmt(&mut self) -> TsnatResult<ThrowStmt<'arena>> {
        let start = self.expect(TokenKind::KwThrow)?.span;
        if self.peek().has_preceding_newline {
            return Err(TsnatError::Parse {
                message: "Line break not allowed after throw".into(),
                span: start,
            });
        }
        let e = self.parse_expr()?;
        let argument = self.alloc(e);
        let span = start.merge(argument.span());
        self.match_kind(TokenKind::Semicolon);
        Ok(ThrowStmt { argument, span })
    }

    fn parse_break_stmt(&mut self) -> TsnatResult<BreakStmt> {
        let start = self.expect(TokenKind::KwBreak)?.span;
        let mut span = start;
        let label = if !self.peek().has_preceding_newline && self.peek().kind == TokenKind::Ident {
            let tok = self.advance();
            span = span.merge(tok.span);
            Some(tok.value)
        } else {
            None
        };
        self.match_kind(TokenKind::Semicolon);
        Ok(BreakStmt { label, span })
    }

    fn parse_continue_stmt(&mut self) -> TsnatResult<ContinueStmt> {
        let start = self.expect(TokenKind::KwContinue)?.span;
        let mut span = start;
        let label = if !self.peek().has_preceding_newline && self.peek().kind == TokenKind::Ident {
            let tok = self.advance();
            span = span.merge(tok.span);
            Some(tok.value)
        } else {
            None
        };
        self.match_kind(TokenKind::Semicolon);
        Ok(ContinueStmt { label, span })
    }

    fn parse_try_stmt(&mut self) -> TsnatResult<TryStmt<'arena>> {
        let start = self.expect(TokenKind::KwTry)?.span;
        let block = self.parse_block_stmt()?;
        let mut handler = None;
        if self.match_kind(TokenKind::KwCatch) {
            let catch_start = self.peek().span;
            let param = if self.match_kind(TokenKind::LParen) {
                let p = self.expect(TokenKind::Ident)?.value;
                if self.match_kind(TokenKind::Colon) {
                    self.parse_type_node()?;
                }
                self.expect(TokenKind::RParen)?;
                Some(p)
            } else {
                None
            };
            let body = self.parse_block_stmt()?;
            handler = Some(CatchHandler { param, body, span: catch_start.merge(body.span) });
        }
        let mut finalizer = None;
        if self.match_kind(TokenKind::KwFinally) {
            finalizer = Some(self.parse_block_stmt()?);
        }
        if handler.is_none() && finalizer.is_none() {
            return Err(TsnatError::Parse {
                message: "Missing catch or finally after try".into(),
                span: block.span,
            });
        }
        let end = finalizer.as_ref().map(|f| f.span).or(handler.as_ref().map(|h| h.span)).unwrap();
        Ok(TryStmt { block, handler, finalizer, span: start.merge(end) })
    }

    fn parse_switch_stmt(&mut self) -> TsnatResult<SwitchStmt<'arena>> {
        let start = self.expect(TokenKind::KwSwitch)?.span;
        self.expect(TokenKind::LParen)?;
        let e = self.parse_expr()?;
        let discriminant = self.alloc(e);
        self.expect(TokenKind::RParen)?;
        self.expect(TokenKind::LBrace)?;
        let mut cases = Vec::new();
        while self.peek().kind != TokenKind::RBrace && !self.is_at_end() {
            cases.push(self.parse_switch_case()?);
        }
        let end = self.expect(TokenKind::RBrace)?.span;
        Ok(SwitchStmt {
            discriminant,
            cases: self.arena.alloc_slice_fill_iter(cases),
            span: start.merge(end),
        })
    }

    fn parse_switch_case(&mut self) -> TsnatResult<SwitchCase<'arena>> {
        let start = self.peek().span;
        let test = if self.match_kind(TokenKind::KwCase) {
            let t = self.parse_expr()?;
            self.expect(TokenKind::Colon)?;
            Some(self.alloc(t))
        } else {
            self.expect(TokenKind::KwDefault)?;
            self.expect(TokenKind::Colon)?;
            None
        };
        let mut consecutive = Vec::new();
        while !matches!(self.peek().kind, TokenKind::KwCase | TokenKind::KwDefault | TokenKind::RBrace) && !self.is_at_end() {
            consecutive.push(self.parse_stmt()?);
        }
        let end = consecutive.last().map(|s| s.span()).unwrap_or(start);
        Ok(SwitchCase {
            test,
            consecutive: self.arena.alloc_slice_fill_iter(consecutive),
            span: start.merge(end),
        })
    }

    fn parse_labeled_stmt(&mut self) -> TsnatResult<LabeledStmt<'arena>> {
        let label = self.expect(TokenKind::Ident)?.value;
        self.expect(TokenKind::Colon)?;
        let body = self.parse_stmt()?;
        let body = self.alloc(body);
        let span = body.span(); // simplified
        Ok(LabeledStmt { label, body, span })
    }

    fn parse_function_decl(&mut self, is_stmt: bool) -> TsnatResult<FunctionDecl<'arena>> {
        let start = self.peek().span;
        let is_async = self.match_kind(TokenKind::KwAsync);
        self.expect(TokenKind::KwFunction)?;
        let is_generator = self.match_kind(TokenKind::Star);
        
        let id = if self.peek().kind == TokenKind::Ident {
            Some(self.expect(TokenKind::Ident)?.value)
        } else if is_stmt {
            return Err(TsnatError::Parse {
                message: "Function name required in declaration".into(),
                span: start,
            });
        } else {
            None
        };

        self.expect(TokenKind::LParen)?;
        let mut params = Vec::new();
        while self.peek().kind != TokenKind::RParen && !self.is_at_end() {
            params.push(self.parse_param()?);
            if !self.match_kind(TokenKind::Comma) { break; }
        }
        self.expect(TokenKind::RParen)?;

        let return_ty = if self.match_kind(TokenKind::Colon) {
            Some(self.parse_type_node()?)
        } else {
            None
        };

        let body = if self.peek().kind == TokenKind::LBrace {
            Some(self.parse_block_stmt()?)
        } else if is_stmt {
            // Ambient or abstract functions might lack a body, but for Phase 1 we expect bodies
            Some(self.parse_block_stmt()?)
        } else {
            None
        };

        let end = body.map(|b| b.span).or(return_ty.map(|t| t.span())).unwrap_or(start);
        Ok(FunctionDecl {
            id, params: self.arena.alloc_slice_fill_iter(params),
            body, return_ty, is_async, is_generator, span: start.merge(end),
        })
    }

    fn parse_param(&mut self) -> TsnatResult<Param<'arena>> {
        let start = self.peek().span;
        let is_rest = self.match_kind(TokenKind::DotDotDot);
        let name_tok = self.expect(TokenKind::Ident)?;
        let name = name_tok.value;
        let mut span = name_tok.span;
        
        let ty = if self.match_kind(TokenKind::Colon) {
            let t = self.parse_type_node()?;
            span = span.merge(t.span());
            Some(t)
        } else {
            None
        };

        let init = if self.match_kind(TokenKind::Eq) {
            let e = self.parse_assignment_expr()?;
            span = span.merge(e.span());
            Some(self.alloc(e))
        } else {
            None
        };

        Ok(Param { name, ty, init, is_rest, span: start.merge(span) })
    }

    fn parse_class_decl(&mut self, is_stmt: bool) -> TsnatResult<ClassDecl<'arena>> {
        let start = self.expect(TokenKind::KwClass)?.span;
        let id = if self.peek().kind == TokenKind::Ident {
            Some(self.expect(TokenKind::Ident)?.value)
        } else if is_stmt {
            return Err(TsnatError::Parse {
                message: "Class name required in declaration".into(),
                span: start,
            });
        } else {
            None
        };

        let super_class = if self.match_kind(TokenKind::KwExtends) {
            let e = self.parse_expr()?;
            Some(self.alloc(e))
        } else {
            None
        };

        self.expect(TokenKind::LBrace)?;
        let mut body = Vec::new();
        while self.peek().kind != TokenKind::RBrace && !self.is_at_end() {
            body.push(self.parse_class_member()?);
        }
        let end = self.expect(TokenKind::RBrace)?.span;

        Ok(ClassDecl {
            id, super_class, body: self.arena.alloc_slice_fill_iter(body),
            span: start.merge(end),
        })
    }

    fn parse_class_member(&mut self) -> TsnatResult<ClassMember<'arena>> {
        let start = self.peek().span;
        
        let access = self.parse_access_modifier();
        let is_static = self.match_kind(TokenKind::KwStatic);
        
        let id_tok = self.expect(TokenKind::Ident)?;
        let key = id_tok.value;

        if self.peek().kind == TokenKind::LParen {
            // Method or Constructor
            self.expect(TokenKind::LParen)?;
            let mut params = Vec::new();
            while self.peek().kind != TokenKind::RParen && !self.is_at_end() {
                params.push(self.parse_param()?);
                if !self.match_kind(TokenKind::Comma) { break; }
            }
            self.expect(TokenKind::RParen)?;

            let return_ty = if self.match_kind(TokenKind::Colon) {
                Some(self.parse_type_node()?)
            } else {
                None
            };

            let body = if self.peek().kind == TokenKind::LBrace {
                Some(self.parse_block_stmt()?)
            } else {
                None
            };

            let end = body.map(|b| b.span).or(return_ty.map(|t| t.span())).unwrap_or(start);
            let func = FunctionDecl {
                id: Some(key),
                params: self.arena.alloc_slice_fill_iter(params),
                body, return_ty, is_async: false, is_generator: false, span: start.merge(end),
            };

            if key == self.interner.intern("constructor") {
                Ok(ClassMember::Constructor(func))
            } else {
                Ok(ClassMember::Method(MethodDecl {
                    key, func, is_static, access, span: start.merge(func.span),
                }))
            }
        } else {
            // Property
            let ty = if self.match_kind(TokenKind::Colon) {
                Some(self.parse_type_node()?)
            } else {
                None
            };
            let init = if self.match_kind(TokenKind::Eq) {
                let e = self.parse_expr()?;
                Some(self.alloc(e))
            } else {
                None
            };
            let end = init.map(|i| i.span()).or(ty.map(|t| t.span())).unwrap_or(id_tok.span);
            self.match_kind(TokenKind::Semicolon);
            Ok(ClassMember::Property(PropertyDecl {
                key, ty, init, is_static, access, span: start.merge(end),
            }))
        }
    }

    fn parse_access_modifier(&mut self) -> Option<AccessModifier> {
        match self.peek().kind {
            TokenKind::KwPublic => { self.advance(); Some(AccessModifier::Public) }
            TokenKind::KwPrivate => { self.advance(); Some(AccessModifier::Private) }
            TokenKind::KwProtected => { self.advance(); Some(AccessModifier::Protected) }
            _ => None,
        }
    }

    fn parse_native_import_decl(&mut self) -> TsnatResult<NativeImportDecl> {
        let start = self.expect(TokenKind::KwImport)?.span;
        self.expect(TokenKind::KwNative)?;
        let name = self.expect(TokenKind::Ident)?.value;
        self.expect(TokenKind::KwFrom)?;
        let source = self.expect(TokenKind::String)?.value;
        self.match_kind(TokenKind::Semicolon);

        Ok(NativeImportDecl {
            name,
            source,
            span: start.merge(self.tokens[self.pos - 1].span),
        })
    }

    fn parse_native_function_decl(&mut self) -> TsnatResult<NativeFunctionDecl<'arena>> {
        let start = self.expect(TokenKind::KwDeclare)?.span;
        self.expect(TokenKind::KwNative)?;
        self.expect(TokenKind::KwFunction)?;
        
        let name = self.expect(TokenKind::Ident)?.value;
        self.expect(TokenKind::LParen)?;
        
        let mut params = Vec::new();
        while self.peek().kind != TokenKind::RParen && !self.is_at_end() {
             params.push(self.parse_param()?);
             if self.peek().kind == TokenKind::Comma {
                 self.advance();
             }
        }
        self.expect(TokenKind::RParen)?;
        
        let mut return_type = None;
        if self.match_kind(TokenKind::Colon) {
             return_type = Some(self.parse_type_node()?);
        }
        
        self.match_kind(TokenKind::Semicolon);
        
        Ok(NativeFunctionDecl {
            name,
            params: self.arena.alloc_slice_fill_iter(params),
            return_type: return_type.map(|t| self.alloc(t)),
            span: start.merge(self.tokens[self.pos - 1].span),
        })
    }

    fn parse_import_decl(&mut self) -> TsnatResult<ImportDecl<'arena>> {
        let start = self.expect(TokenKind::KwImport)?.span;
        let mut specifiers = Vec::new();

        if self.peek().kind == TokenKind::Ident {
            let default = self.expect(TokenKind::Ident)?.value;
            specifiers.push(ImportSpecifier::Default(default));
            if self.match_kind(TokenKind::Comma) {
                self.parse_import_specifiers(&mut specifiers)?;
            }
        } else if self.peek().kind == TokenKind::LBrace || self.peek().kind == TokenKind::Star {
            self.parse_import_specifiers(&mut specifiers)?;
        }

        self.expect(TokenKind::KwFrom)?;
        let source_tok = self.expect(TokenKind::String)?;
        let source = source_tok.value;
        let end = self.expect(TokenKind::Semicolon)?.span;

        Ok(ImportDecl {
            specifiers: self.arena.alloc_slice_fill_iter(specifiers),
            source,
            span: start.merge(end),
        })
    }

    fn parse_import_specifiers(&mut self, specs: &mut Vec<ImportSpecifier>) -> TsnatResult<()> {
        match self.peek().kind {
            TokenKind::Star => {
                self.advance();
                self.expect(TokenKind::KwAs)?;
                let local = self.expect(TokenKind::Ident)?.value;
                specs.push(ImportSpecifier::Namespace(local));
            }
            TokenKind::LBrace => {
                self.advance();
                while self.peek().kind != TokenKind::RBrace && !self.is_at_end() {
                    let imported = self.expect(TokenKind::Ident)?.value;
                    let local = if self.match_kind(TokenKind::KwAs) {
                        Some(self.expect(TokenKind::Ident)?.value)
                    } else {
                        None
                    };
                    specs.push(ImportSpecifier::Named(local.unwrap_or(imported), if local.is_some() { Some(imported) } else { None }));
                    if !self.match_kind(TokenKind::Comma) { break; }
                }
                self.expect(TokenKind::RBrace)?;
            }
            _ => {
                return Err(TsnatError::Parse {
                    message: "Expected import specifiers".into(),
                    span: self.peek().span,
                });
            }
        }
        Ok(())
    }

    fn parse_export_decl(&mut self) -> TsnatResult<ExportDecl<'arena>> {
        let start = self.expect(TokenKind::KwExport)?.span;
        let is_default = self.match_kind(TokenKind::KwDefault);
        
        let mut decl = None;
        let mut specifiers = Vec::new();
        let mut source = None;

        match self.peek().kind {
            TokenKind::KwConst | TokenKind::KwLet | TokenKind::KwVar | TokenKind::KwFunction | TokenKind::KwClass => {
                let d = self.parse_stmt()?;
                decl = Some(self.alloc(d));
            }
            TokenKind::LBrace => {
                self.advance();
                while self.peek().kind != TokenKind::RBrace && !self.is_at_end() {
                    let local = self.expect(TokenKind::Ident)?.value;
                    let exported = if self.match_kind(TokenKind::KwAs) {
                        Some(self.expect(TokenKind::Ident)?.value)
                    } else {
                        None
                    };
                    specifiers.push(ExportSpecifier { local, exported });
                    if !self.match_kind(TokenKind::Comma) { break; }
                }
                self.expect(TokenKind::RBrace)?;
                if self.match_kind(TokenKind::KwFrom) {
                    source = Some(self.expect(TokenKind::String)?.value);
                }
                self.match_kind(TokenKind::Semicolon);
            }
            _ if is_default => {
                let d = self.parse_stmt()?;
                decl = Some(self.alloc(d));
            }
            _ => {
                return Err(TsnatError::Parse {
                    message: "Expected export declaration".into(),
                    span: start,
                });
            }
        }

        let end = decl.as_ref().map(|d| d.span()).unwrap_or(start);
        Ok(ExportDecl {
            decl, specifiers: self.arena.alloc_slice_fill_iter(specifiers),
            source, is_default, span: start.merge(end),
        })
    }

    // ── Expression parsing (Pratt / precedence climbing) ───────

    /// Top-level expression: handles comma sequences if needed.
    fn parse_expr(&mut self) -> TsnatResult<Expr<'arena>> {
        let first = self.parse_assignment_expr()?;
        if self.match_kind(TokenKind::Comma) {
            let mut exprs = vec![self.alloc(first)];
            let second = self.parse_assignment_expr()?;
            exprs.push(self.alloc(second));
            while self.match_kind(TokenKind::Comma) {
                let next = self.parse_assignment_expr()?;
                exprs.push(self.alloc(next));
            }
            let start = exprs.first().unwrap().span();
            let end = exprs.last().unwrap().span();
            Ok(Expr::Sequence(SequenceExpr {
                exprs: self.arena.alloc_slice_fill_iter(exprs),
                span: start.merge(end),
            }))
        } else {
            Ok(first)
        }
    }

    /// Assignment is right-associative, lowest precedence above comma.
    fn parse_assignment_expr(&mut self) -> TsnatResult<Expr<'arena>> {
        if self.is_arrow_function() {
            let is_async = self.peek().kind == TokenKind::KwAsync;
            return self.parse_arrow_function(is_async);
        }

        if self.peek().kind == TokenKind::KwYield {
            let start = self.advance().span;
            let is_star = self.match_kind(TokenKind::Star);
            // Yield without operand is valid if followed by ; or }
            let (arg, end) = if self.peek().has_preceding_newline || self.peek().kind == TokenKind::Semicolon || self.peek().kind == TokenKind::RBrace || self.peek().kind == TokenKind::RParen || self.peek().kind == TokenKind::RBracket || self.peek().kind == TokenKind::Eof {
                (None, start)
            } else {
                let e = self.parse_assignment_expr()?;
                let end = e.span();
                (Some(self.alloc(e) as &Expr<'arena>), end)
            };
            return Ok(Expr::Yield(arg, is_star, start.merge(end)));
        }

        let left = self.parse_conditional_expr()?;

        if let Some(op) = self.assignment_op() {
            self.advance();
            let right = self.parse_assignment_expr()?;
            let span = left.span().merge(right.span());
            let left = self.alloc(left);
            let right = self.alloc(right);
            return Ok(Expr::Assign(AssignExpr { op, left, right, span }));
        }

        Ok(left)
    }

    fn is_arrow_function(&self) -> bool {
        let mut offset = 0;
        let mut kind = self.peek().kind;
        if kind == TokenKind::KwAsync {
            offset = 1;
            kind = self.peek_ahead(1).kind;
        }

        if kind == TokenKind::Ident {
            return self.peek_ahead(offset + 1).kind == TokenKind::Arrow;
        }
        if kind == TokenKind::LParen {
            let mut i = offset + 1;
            let mut depth = 1;
            while depth > 0 {
                let k = self.peek_ahead(i).kind;
                if k == TokenKind::Eof { return false; }
                if k == TokenKind::LParen { depth += 1; }
                if k == TokenKind::RParen { depth -= 1; }
                i += 1;
            }
            let next = self.peek_ahead(i).kind;
            if next == TokenKind::Arrow {
                return true;
            }
            if next == TokenKind::Colon {
                let mut j = i + 1;
                let mut type_depth = 0;
                while j < self.tokens.len() {
                    let k = self.peek_ahead(j).kind;
                    if k == TokenKind::Eof { return false; }
                    if k == TokenKind::Arrow && type_depth == 0 { return true; }
                    if k == TokenKind::Lt { type_depth += 1; }
                    if k == TokenKind::Gt { type_depth -= 1; }
                    if k == TokenKind::LBrace { type_depth += 1; }
                    if k == TokenKind::RBrace { type_depth -= 1; }
                    if k == TokenKind::Semicolon || k == TokenKind::Eq { return false; }
                    j += 1;
                }
            }
        }
        false
    }

    fn parse_arrow_function(&mut self, is_async: bool) -> TsnatResult<Expr<'arena>> {
        let mut start = self.peek().span;
        if is_async {
            start = self.advance().span; // Consume `async`
        }
        let mut params = Vec::new();
        
        if self.peek().kind == TokenKind::Ident {
            let name_tok = self.advance();
            params.push(Param {
                name: name_tok.value,
                span: name_tok.span,
                ty: None,
                init: None,
                is_rest: false,
            });
        } else {
            self.expect(TokenKind::LParen)?;
            while self.peek().kind != TokenKind::RParen && !self.is_at_end() {
                params.push(self.parse_param()?);
                if !self.match_kind(TokenKind::Comma) { break; }
            }
            self.expect(TokenKind::RParen)?;
        }

        if self.match_kind(TokenKind::Colon) {
            self.parse_type_node()?;
        }

        self.expect(TokenKind::Arrow)?;

        let body = if self.peek().kind == TokenKind::LBrace {
            ArrowBody::Block(self.parse_block_stmt()?)
        } else {
            let body_expr = self.parse_assignment_expr()?;
            ArrowBody::Expr(self.alloc(body_expr))
        };

        let span = match &body {
            ArrowBody::Expr(e) => start.merge(e.span()),
            ArrowBody::Block(b) => start.merge(b.span),
        };

        Ok(Expr::Arrow(ArrowExpr {
            params: self.arena.alloc_slice_fill_iter(params),
            body,
            is_async,
            span,
        }))
    }

    fn assignment_op(&self) -> Option<AssignOp> {
        match self.peek().kind {
            TokenKind::Eq => Some(AssignOp::Eq),
            TokenKind::PlusEq => Some(AssignOp::AddEq),
            TokenKind::MinusEq => Some(AssignOp::SubEq),
            TokenKind::StarEq => Some(AssignOp::MulEq),
            TokenKind::SlashEq => Some(AssignOp::DivEq),
            TokenKind::PercentEq => Some(AssignOp::ModEq),
            TokenKind::StarStarEq => Some(AssignOp::ExpEq),
            TokenKind::AmpAmpEq => Some(AssignOp::AndEq),
            TokenKind::PipePipeEq => Some(AssignOp::OrEq),
            TokenKind::QuestionQuestionEq => Some(AssignOp::NullishEq),
            TokenKind::AmpEq => Some(AssignOp::BitAndEq),
            TokenKind::PipeEq => Some(AssignOp::BitOrEq),
            TokenKind::CaretEq => Some(AssignOp::BitXorEq),
            TokenKind::LtLtEq => Some(AssignOp::ShlEq),
            TokenKind::GtGtEq => Some(AssignOp::ShrEq),
            TokenKind::GtGtGtEq => Some(AssignOp::UShrEq),
            _ => None,
        }
    }

    /// Ternary: `cond ? then : else`
    fn parse_conditional_expr(&mut self) -> TsnatResult<Expr<'arena>> {
        let test = self.parse_binary_expr(0)?;

        if self.match_kind(TokenKind::Question) {
            let consequent = self.parse_assignment_expr()?;
            self.expect(TokenKind::Colon)?;
            let alternate = self.parse_assignment_expr()?;
            let span = test.span().merge(alternate.span());
            let test = self.alloc(test);
            let consequent = self.alloc(consequent);
            let alternate = self.alloc(alternate);
            return Ok(Expr::Conditional(ConditionalExpr {
                test, consequent, alternate, span,
            }));
        }

        Ok(test)
    }

    // ── Precedence climbing for binary operators ───────────────
    // ALGO: See SPECS.md §4 FR-PAR-008

    fn parse_binary_expr(&mut self, min_prec: u8) -> TsnatResult<Expr<'arena>> {
        let mut left = self.parse_unary_expr()?;

        loop {
            let Some((op, prec, right_assoc)) = self.binary_op_info() else {
                break;
            };
            if prec < min_prec { break; }
            self.advance();
            let next_prec = if right_assoc { prec } else { prec + 1 };
            let right = self.parse_binary_expr(next_prec)?;
            let span = left.span().merge(right.span());
            let left_ref = self.alloc(left);
            let right_ref = self.alloc(right);
            left = Expr::Binary(BinaryExpr {
                op, left: left_ref, right: right_ref, span,
            });
        }

        // Handle `as` and `satisfies` type assertions at this level
        loop {
            if self.peek().kind == TokenKind::KwAs {
                self.advance();
                let start_span = left.span();
                let ty = self.parse_type_node()?;
                let span = start_span.merge(ty.span());
                let expr = self.alloc(left);
                left = Expr::As(AsExpr { expr, ty, span });
            } else if self.peek().kind == TokenKind::KwSatisfies {
                self.advance();
                let start_span = left.span();
                let ty = self.parse_type_node()?;
                let span = start_span.merge(ty.span());
                let expr = self.alloc(left);
                left = Expr::Satisfies(SatisfiesExpr { expr, ty, span });
            } else {
                break;
            }
        }

        Ok(left)
    }

    fn binary_op_info(&self) -> Option<(BinaryOp, u8, bool)> {
        match self.peek().kind {
            // Level 17 (right-assoc): **
            TokenKind::StarStar => Some((BinaryOp::Exp, 17, true)),
            // Level 16: *, /, %
            TokenKind::Star => Some((BinaryOp::Mul, 16, false)),
            TokenKind::Slash => Some((BinaryOp::Div, 16, false)),
            TokenKind::Percent => Some((BinaryOp::Mod, 16, false)),
            // Level 15: +, -
            TokenKind::Plus => Some((BinaryOp::Add, 15, false)),
            TokenKind::Minus => Some((BinaryOp::Sub, 15, false)),
            // Level 14: <<, >>, >>>
            TokenKind::LtLt => Some((BinaryOp::Shl, 14, false)),
            TokenKind::GtGt => Some((BinaryOp::Shr, 14, false)),
            TokenKind::GtGtGt => Some((BinaryOp::UShr, 14, false)),
            // Level 13: <, >, <=, >=, instanceof, in
            TokenKind::Lt => Some((BinaryOp::Lt, 13, false)),
            TokenKind::Gt => Some((BinaryOp::Gt, 13, false)),
            TokenKind::LtEq => Some((BinaryOp::LtEq, 13, false)),
            TokenKind::GtEq => Some((BinaryOp::GtEq, 13, false)),
            TokenKind::KwInstanceof => Some((BinaryOp::Instanceof, 13, false)),
            TokenKind::KwIn => Some((BinaryOp::In, 13, false)),
            // Level 12: ==, !=, ===, !==
            TokenKind::EqEq => Some((BinaryOp::EqEq, 12, false)),
            TokenKind::BangEq => Some((BinaryOp::BangEq, 12, false)),
            TokenKind::EqEqEq => Some((BinaryOp::EqEqEq, 12, false)),
            TokenKind::BangEqEq => Some((BinaryOp::BangEqEq, 12, false)),
            // Level 11: &
            TokenKind::Amp => Some((BinaryOp::BitAnd, 11, false)),
            // Level 10: ^
            TokenKind::Caret => Some((BinaryOp::BitXor, 10, false)),
            // Level 9: |
            TokenKind::Pipe => Some((BinaryOp::BitOr, 9, false)),
            // Level 8: &&
            TokenKind::AmpAmp => Some((BinaryOp::And, 8, false)),
            // Level 7: ||, ??
            TokenKind::PipePipe => Some((BinaryOp::Or, 7, false)),
            TokenKind::QuestionQuestion => Some((BinaryOp::NullishCoalesce, 7, false)),
            _ => None,
        }
    }

    // ── Unary ─────────────────────────────────────────────────

    fn parse_unary_expr(&mut self) -> TsnatResult<Expr<'arena>> {
        let tok = self.peek();
        let start = tok.span;
        match tok.kind {
            TokenKind::Minus => {
                self.advance();
                let operand = self.parse_unary_expr()?;
                let span = start.merge(operand.span());
                let operand = self.alloc(operand);
                Ok(Expr::Unary(UnaryExpr { op: UnaryOp::Neg, operand, span }))
            }
            TokenKind::Plus => {
                self.advance();
                let operand = self.parse_unary_expr()?;
                let span = start.merge(operand.span());
                let operand = self.alloc(operand);
                Ok(Expr::Unary(UnaryExpr { op: UnaryOp::Plus, operand, span }))
            }
            TokenKind::Bang => {
                self.advance();
                let operand = self.parse_unary_expr()?;
                let span = start.merge(operand.span());
                let operand = self.alloc(operand);
                Ok(Expr::Unary(UnaryExpr { op: UnaryOp::Not, operand, span }))
            }
            TokenKind::Tilde => {
                self.advance();
                let operand = self.parse_unary_expr()?;
                let span = start.merge(operand.span());
                let operand = self.alloc(operand);
                Ok(Expr::Unary(UnaryExpr { op: UnaryOp::BitNot, operand, span }))
            }
            TokenKind::KwTypeof => {
                self.advance();
                let operand = self.parse_unary_expr()?;
                let span = start.merge(operand.span());
                let operand = self.alloc(operand);
                Ok(Expr::Unary(UnaryExpr { op: UnaryOp::Typeof, operand, span }))
            }
            TokenKind::KwVoid => {
                self.advance();
                let operand = self.parse_unary_expr()?;
                let span = start.merge(operand.span());
                let operand = self.alloc(operand);
                Ok(Expr::Unary(UnaryExpr { op: UnaryOp::Void, operand, span }))
            }
            TokenKind::KwDelete => {
                self.advance();
                let operand = self.parse_unary_expr()?;
                let span = start.merge(operand.span());
                let operand = self.alloc(operand);
                Ok(Expr::Unary(UnaryExpr { op: UnaryOp::Delete, operand, span }))
            }
            TokenKind::PlusPlus => {
                self.advance();
                let operand = self.parse_unary_expr()?;
                let span = start.merge(operand.span());
                let operand = self.alloc(operand);
                Ok(Expr::Unary(UnaryExpr { op: UnaryOp::PreInc, operand, span }))
            }
            TokenKind::MinusMinus => {
                self.advance();
                let operand = self.parse_unary_expr()?;
                let span = start.merge(operand.span());
                let operand = self.alloc(operand);
                Ok(Expr::Unary(UnaryExpr { op: UnaryOp::PreDec, operand, span }))
            }
            TokenKind::KwAwait => {
                self.advance();
                let operand = self.parse_unary_expr()?;
                let span = start.merge(operand.span());
                let operand = self.alloc(operand);
                Ok(Expr::Await(operand, span))
            }
            TokenKind::DotDotDot => {
                self.advance();
                let arg = self.parse_assignment_expr()?;
                let span = start.merge(arg.span());
                let arg = self.alloc(arg);
                Ok(Expr::Spread(SpreadExpr { argument: arg, span }))
            }

            _ => self.parse_postfix_expr(),
        }
    }

    // ── Postfix (call, member, index, optional chain, ++) ─────

    fn parse_postfix_expr(&mut self) -> TsnatResult<Expr<'arena>> {
        let mut expr = self.parse_primary_expr()?;

        loop {
            match self.peek().kind {
                TokenKind::Dot => {
                    self.advance();
                    let prop_tok = self.advance();
                    if prop_tok.kind == TokenKind::Ident || format!("{:?}", prop_tok.kind).starts_with("Kw") {
                        let span = expr.span().merge(prop_tok.span);
                        let object = self.alloc(expr);
                        expr = Expr::Member(MemberExpr {
                            object, property: prop_tok.value, span,
                        });
                    } else {
                        return Err(TsnatError::Parse {
                            message: format!("Expected identifier, found {:?}", prop_tok.kind),
                            span: prop_tok.span,
                        });
                    }
                }
                TokenKind::QuestionDot => {
                    self.advance();
                    let next_kind = self.peek().kind;
                    if next_kind == TokenKind::LParen {
                        let (args, end) = self.parse_arguments()?;
                        let span = expr.span().merge(end);
                        let base = self.alloc(expr);
                        expr = Expr::OptChain(OptChainExpr {
                            base, ext: OptChainExt::Call(args), span,
                        });
                    } else if next_kind == TokenKind::LBracket {
                        self.advance();
                        let index = self.parse_expr()?;
                        let end = self.expect(TokenKind::RBracket)?.span;
                        let span = expr.span().merge(end);
                        let base = self.alloc(expr);
                        let index = self.alloc(index);
                        expr = Expr::OptChain(OptChainExpr {
                            base, ext: OptChainExt::Index(index), span,
                        });
                    } else {
                        let prop_tok = self.advance();
                        if prop_tok.kind == TokenKind::Ident || format!("{:?}", prop_tok.kind).starts_with("Kw") {
                            let span = expr.span().merge(prop_tok.span);
                            let base = self.alloc(expr);
                            expr = Expr::OptChain(OptChainExpr {
                                base, ext: OptChainExt::Member(prop_tok.value), span,
                            });
                        } else {
                            return Err(TsnatError::Parse {
                                message: format!("Expected identifier, found {:?}", prop_tok.kind),
                                span: prop_tok.span,
                            });
                        }
                    }
                }
                TokenKind::LBracket => {
                    self.advance();
                    let index = self.parse_expr()?;
                    let end = self.expect(TokenKind::RBracket)?.span;
                    let span = expr.span().merge(end);
                    let object = self.alloc(expr);
                    let index = self.alloc(index);
                    expr = Expr::Index(IndexExpr { object, index, span });
                }
                TokenKind::Lt => {
                    let pos_backup = self.pos;
                    // Supress errors during lookahead by temporarily turning them off? We don't have that, but parse_type_args returns Result.
                    // If it errors, we just rollback and break.
                    if let Ok(Some(type_args)) = self.parse_type_args() {
                        if self.peek().kind == TokenKind::LParen {
                            let (args, end) = self.parse_arguments()?;
                            let span = expr.span().merge(end);
                            let callee = self.alloc(expr);
                            expr = Expr::Call(CallExpr { callee, type_args: Some(type_args), args, span });
                            continue;
                        }
                    }
                    self.pos = pos_backup;
                    break;
                }
                TokenKind::LParen => {
                    let (args, end) = self.parse_arguments()?;
                    let span = expr.span().merge(end);
                    let callee = self.alloc(expr);
                    expr = Expr::Call(CallExpr { callee, type_args: None, args, span });
                }
                // Postfix ++ / -- (no preceding newline)
                TokenKind::PlusPlus if !self.peek().has_preceding_newline => {
                    let end = self.advance().span;
                    let span = expr.span().merge(end);
                    let operand = self.alloc(expr);
                    expr = Expr::Unary(UnaryExpr {
                        op: UnaryOp::PostInc, operand, span,
                    });
                }
                TokenKind::MinusMinus if !self.peek().has_preceding_newline => {
                    let end = self.advance().span;
                    let span = expr.span().merge(end);
                    let operand = self.alloc(expr);
                    expr = Expr::Unary(UnaryExpr {
                        op: UnaryOp::PostDec, operand, span,
                    });
                }
                TokenKind::Bang if !self.peek().has_preceding_newline => {
                    let end = self.advance().span;
                    let span = expr.span().merge(end);
                    let expr_alloc = self.alloc(expr);
                    expr = Expr::NonNull(expr_alloc, span);
                }
                _ => break,
            }
        }

        Ok(expr)
    }

    /// Parse member expression (without call) — used after `new`.
    fn parse_member_expr(&mut self) -> TsnatResult<Expr<'arena>> {
        let mut expr = self.parse_primary_expr()?;
        loop {
            match self.peek().kind {
                TokenKind::Dot => {
                    self.advance();
                    let prop = self.advance();
                    if prop.kind == TokenKind::Ident || format!("{:?}", prop.kind).starts_with("Kw") {
                        let span = expr.span().merge(prop.span);
                        let object = self.alloc(expr);
                        expr = Expr::Member(MemberExpr {
                            object, property: prop.value, span,
                        });
                    } else {
                        return Err(TsnatError::Parse {
                            message: format!("Expected identifier, found {:?}", prop.kind),
                            span: prop.span,
                        });
                    }
                }
                TokenKind::LBracket => {
                    self.advance();
                    let index = self.parse_expr()?;
                    let end = self.expect(TokenKind::RBracket)?.span;
                    let span = expr.span().merge(end);
                    let object = self.alloc(expr);
                    let index = self.alloc(index);
                    expr = Expr::Index(IndexExpr { object, index, span });
                }
                _ => break,
            }
        }
        Ok(expr)
    }

    // ── Primary expressions ───────────────────────────────────

    fn parse_primary_expr(&mut self) -> TsnatResult<Expr<'arena>> {
        let tok = self.peek();
        match tok.kind {
            TokenKind::Number => {
                let tok = self.advance();
                let text = self.interner.get(tok.value);
                let cleaned: String = text.chars().filter(|c| *c != '_').collect();
                let val = if cleaned.starts_with("0x") || cleaned.starts_with("0X") {
                    i64::from_str_radix(&cleaned[2..], 16).unwrap_or(0) as f64
                } else if cleaned.starts_with("0b") || cleaned.starts_with("0B") {
                    i64::from_str_radix(&cleaned[2..], 2).unwrap_or(0) as f64
                } else if cleaned.starts_with("0o") || cleaned.starts_with("0O") {
                    i64::from_str_radix(&cleaned[2..], 8).unwrap_or(0) as f64
                } else {
                    cleaned.parse::<f64>().unwrap_or(0.0)
                };
                Ok(Expr::Number(val, tok.span))
            }
            TokenKind::String => {
                let tok = self.advance();
                Ok(Expr::String(tok.value, tok.span))
            }
            TokenKind::BigInt => {
                let tok = self.advance();
                Ok(Expr::BigInt(tok.value, tok.span))
            }
            TokenKind::Regex => {
                let tok = self.advance();
                Ok(Expr::Regex(tok.value, tok.span))
            }
            TokenKind::KwNew => {
                let start = self.advance().span;
                let callee = self.parse_member_expr()?;
                let type_args = self.parse_type_args()?;
                let (args, end) = if self.peek().kind == TokenKind::LParen {
                    self.parse_arguments()?
                } else {
                    (&[] as &[&Expr], callee.span())
                };
                let span = start.merge(end);
                let callee = self.alloc(callee);
                Ok(Expr::New(NewExpr { callee, type_args, args, span }))
            }
            TokenKind::KwTrue => {
                let span = self.advance().span;
                Ok(Expr::Bool(true, span))
            }
            TokenKind::KwFalse => {
                let span = self.advance().span;
                Ok(Expr::Bool(false, span))
            }
            TokenKind::KwNull => {
                let span = self.advance().span;
                Ok(Expr::Null(span))
            }
            TokenKind::KwUndefined => {
                let span = self.advance().span;
                Ok(Expr::Undefined(span))
            }
            TokenKind::KwThis => {
                let span = self.advance().span;
                Ok(Expr::This(span))
            }
            TokenKind::Ident => {
                let tok = self.advance();
                Ok(Expr::Ident(tok.value, tok.span))
            }
            TokenKind::Lt => {
                let next = self.peek_ahead(1);
                if next.kind == TokenKind::Ident {
                    self.advance(); // consume `<`
                    return self.parse_jsx_element();
                } else {
                    return Err(TsnatError::Parse {
                        message: "Expected JSX or type arg".into(),
                        span: self.peek().span,
                    });
                }
            }
            TokenKind::LParen => {
                let start = self.advance().span;
                let expr = self.parse_expr()?;
                let end = self.expect(TokenKind::RParen)?.span;
                let expr = self.alloc(expr);
                Ok(Expr::Paren(expr, start.merge(end)))
            }
            TokenKind::LBracket => self.parse_array_literal(),
            TokenKind::LBrace => self.parse_object_literal(),
            TokenKind::TemplateHead => self.parse_template_expr(),
            TokenKind::NoSubstTemplate => {
                let tok = self.advance();
                let sym = tok.value;
                let quasis = self.arena.alloc_slice_fill_iter([sym]);
                let exprs: &[&Expr] = &[];
                Ok(Expr::Template(TemplateExpr {
                    quasis, exprs, span: tok.span,
                }))
            }
            TokenKind::KwFunction => {
                let func = self.parse_function_decl(false)?;
                Ok(Expr::Function(func))
            }
            _ => Err(TsnatError::Parse {
                message: format!("Unexpected token: {}", self.peek().kind),
                span: self.peek().span,
            }),
        }
    }

    // ── JSX Parsing ──────────────────────────────────────────

    fn parse_jsx_element(&mut self) -> TsnatResult<Expr<'arena>> {
        let start_span = self.previous().span; // `<`
        let tag_tok = self.expect(TokenKind::Ident)?;
        let tag = tag_tok.value;

        let mut props = Vec::new();
        while self.peek().kind != TokenKind::Gt && self.peek().kind != TokenKind::Slash && !self.is_at_end() {
            let key_tok = self.expect(TokenKind::Ident)?;
            let key = key_tok.value;
            let key_span = key_tok.span;
            
            let value = if self.match_kind(TokenKind::Eq) {
                if self.match_kind(TokenKind::LBrace) {
                    let expr = self.parse_expr()?;
                    self.expect(TokenKind::RBrace)?;
                    self.alloc(expr)
                } else if self.peek().kind == TokenKind::String {
                    let tok = self.advance();
                    self.alloc(Expr::String(tok.value, tok.span))
                } else {
                    return Err(TsnatError::Parse {
                        message: "Expected string or expr for JSX prop".into(),
                        span: self.peek().span,
                    });
                }
            } else {
                self.alloc(Expr::Bool(true, key_span))
            };
            
            let span = key_span.merge(value.span());
            props.push(ObjProp { key, value, is_spread: false, span });
        }

        let mut children = Vec::new();
        let is_self_closing = if self.match_kind(TokenKind::Slash) {
            self.expect(TokenKind::Gt)?;
            true
        } else {
            self.expect(TokenKind::Gt)?;
            false
        };

        if !is_self_closing {
            let mut text_start = self.peek().span.start;

        while !self.is_at_end() {
            if self.peek().kind == TokenKind::Lt {
                // Determine if it's nested JSX or closing tag
                let lt_tok = self.advance();
                if self.peek().kind == TokenKind::Slash {
                    let _slash_tok = self.advance();
                    // Flush pending text before the closing tag
                    if text_start < lt_tok.span.start {
                        let span = Span { file_id: start_span.file_id, start: text_start, end: lt_tok.span.start };
                        children.push(self.alloc(Expr::JSXText(tsnat_common::interner::SYM_EMPTY, span)));
                    }

                    // Look for tag close
                    self.expect(TokenKind::Ident)?;
                    self.expect(TokenKind::Gt)?;
                    break;
                } else {
                    // Flush pending text before nested tag
                    if text_start < lt_tok.span.start {
                        let span = Span { file_id: start_span.file_id, start: text_start, end: lt_tok.span.start };
                        children.push(self.alloc(Expr::JSXText(tsnat_common::interner::SYM_EMPTY, span)));
                    }
                    // Rewind the `pos` backward by 1 because we consumed `<`
                    self.pos -= 1;
                    
                    self.expect(TokenKind::Lt)?;
                    let child = self.parse_jsx_element()?;
                    children.push(self.alloc(child));
                    text_start = self.peek().span.start;
                }
            } else if self.peek().kind == TokenKind::LBrace {
                // Flush pending text before braces
                let lbrace_tok = self.advance();
                if text_start < lbrace_tok.span.start {
                    let span = Span { file_id: start_span.file_id, start: text_start, end: lbrace_tok.span.start };
                    children.push(self.alloc(Expr::JSXText(tsnat_common::interner::SYM_EMPTY, span)));
                }

                let expr = self.parse_expr()?;
                self.expect(TokenKind::RBrace)?;
                let jsx_expr = Expr::JSXExpressionContainer(self.alloc(expr), lbrace_tok.span);
                children.push(self.alloc(jsx_expr));
                text_start = self.peek().span.start;
            } else {
                self.advance();
            }
        }
        }

        let end_span = self.previous().span;
        Ok(Expr::JSXElement(JSXElement {
            tag,
            props: self.arena.alloc_slice_fill_iter(props),
            children: self.arena.alloc_slice_fill_iter(children),
            span: start_span.merge(end_span),
        }))
    }

    // ── Array literal ─────────────────────────────────────────

    fn parse_array_literal(&mut self) -> TsnatResult<Expr<'arena>> {
        let start = self.advance().span; // [
        let mut elements = Vec::new();
        while self.peek().kind != TokenKind::RBracket && !self.is_at_end() {
            let el = self.parse_assignment_expr()?;
            elements.push(self.alloc(el));
            if !self.match_kind(TokenKind::Comma) { break; }
        }
        let end = self.expect(TokenKind::RBracket)?.span;
        Ok(Expr::Array(ArrayExpr {
            elements: self.arena.alloc_slice_fill_iter(elements),
            span: start.merge(end),
        }))
    }

    // ── Object literal ────────────────────────────────────────

    fn parse_object_literal(&mut self) -> TsnatResult<Expr<'arena>> {
        let start = self.advance().span; // {
        let mut props = Vec::new();
        while self.peek().kind != TokenKind::RBrace && !self.is_at_end() {
            if self.match_kind(TokenKind::DotDotDot) {
                let start = self.previous().span;
                let value = self.parse_assignment_expr()?;
                let span = start.merge(value.span());
                let value = self.alloc(value);
                // We don't have a Default for Symbol, but 0 is usually an empty or uninterned string
                props.push(ObjProp { key: tsnat_common::interner::SYM_EMPTY, value, is_spread: true, span });
            } else {
                let key_tok = self.expect(TokenKind::Ident)?;
                let key = key_tok.value;
                let key_span = key_tok.span;
                let value = if self.match_kind(TokenKind::Colon) {
                    let v = self.parse_assignment_expr()?;
                    self.alloc(v)
                } else {
                    // Shorthand: `{ x }` means `{ x: x }`
                    self.alloc(Expr::Ident(key, key_span))
                };
                let span = key_span.merge(value.span());
                props.push(ObjProp { key, value, is_spread: false, span });
            }
            if !self.match_kind(TokenKind::Comma) { break; }
        }
        let end = self.expect(TokenKind::RBrace)?.span;
        Ok(Expr::Object(ObjectExpr {
            properties: self.arena.alloc_slice_fill_iter(props),
            span: start.merge(end),
        }))
    }

    // ── Template literal ──────────────────────────────────────

    fn parse_template_expr(&mut self) -> TsnatResult<Expr<'arena>> {
        let start_tok = self.advance(); // TemplateHead
        let start = start_tok.span;
        let mut quasis = vec![start_tok.value];
        let mut exprs: Vec<&'arena Expr<'arena>> = Vec::new();

        loop {
            let e = self.parse_expr()?;
            exprs.push(self.alloc(e));

            let tok = self.peek();
            match tok.kind {
                TokenKind::TemplateTail => {
                    quasis.push(tok.value);
                    let end = tok.span;
                    self.advance();
                    return Ok(Expr::Template(TemplateExpr {
                        quasis: self.arena.alloc_slice_fill_iter(quasis),
                        exprs: self.arena.alloc_slice_fill_iter(exprs),
                        span: start.merge(end),
                    }));
                }
                TokenKind::TemplateMiddle => {
                    quasis.push(tok.value);
                    self.advance();
                    // continue loop
                }
                _ => {
                    return Err(TsnatError::Parse {
                        message: "Expected template continuation".into(),
                        span: tok.span,
                    });
                }
            }
        }
    }

    // ── Arguments ─────────────────────────────────────────────

    fn parse_arguments(&mut self) -> TsnatResult<(NodeList<'arena, &'arena Expr<'arena>>, Span)> {
        self.expect(TokenKind::LParen)?;
        let mut args: Vec<&'arena Expr<'arena>> = Vec::new();
        while self.peek().kind != TokenKind::RParen && !self.is_at_end() {
            let a = self.parse_assignment_expr()?;
            args.push(self.alloc(a));
            if !self.match_kind(TokenKind::Comma) { break; }
        }
        let end = self.expect(TokenKind::RParen)?.span;
        Ok((self.arena.alloc_slice_fill_iter(args), end))
    }

    // ── Arrow tail ────────────────────────────────────────────

    #[allow(dead_code)]
    fn parse_arrow_tail(&mut self, params_expr: Expr<'arena>) -> TsnatResult<Expr<'arena>> {
        let start = params_expr.span();
        self.expect(TokenKind::Arrow)?;

        // Collect params from the expression
        let params = self.expr_to_arrow_params(&params_expr)?;

        let body_expr = self.parse_assignment_expr()?;
        let span = start.merge(body_expr.span());
        let body_ref = self.alloc(body_expr);

        Ok(Expr::Arrow(ArrowExpr {
            params: self.arena.alloc_slice_fill_iter(params),
            body: ArrowBody::Expr(body_ref),
            is_async: false,
            span,
        }))
    }

    #[allow(dead_code)]
    fn expr_to_arrow_params(&self, expr: &Expr<'arena>) -> TsnatResult<Vec<Param<'arena>>> {
        match expr {
            Expr::Ident(sym, span) => Ok(vec![Param { name: *sym, span: *span, ty: None, init: None, is_rest: false }]),
            Expr::Paren(inner, _) => self.expr_to_arrow_params(inner),
            _ => Err(TsnatError::Parse {
                message: "Invalid arrow function parameters".into(),
                span: expr.span(),
            }),
        }
    }

    // ── Token helpers ─────────────────────────────────────────

    fn peek(&self) -> &'src Token {
        &self.tokens[self.pos]
    }

    fn peek_ahead(&self, n: usize) -> &'src Token {
        let idx = (self.pos + n).min(self.tokens.len() - 1);
        &self.tokens[idx]
    }

    fn advance(&mut self) -> &'src Token {
        if !self.is_at_end() {
            self.pos += 1;
        }
        &self.tokens[self.pos - 1]
    }

    fn previous(&self) -> &'src Token {
        if self.pos == 0 {
            &self.tokens[0]
        } else {
            &self.tokens[self.pos - 1]
        }
    }

    fn expect(&mut self, kind: TokenKind) -> TsnatResult<&'src Token> {
        if self.peek().kind == kind {
            Ok(self.advance())
        } else {
            Err(TsnatError::Parse {
                message: format!("Expected {:?}, found {:?}", kind, self.peek().kind),
                span: self.peek().span,
            })
        }
    }

    fn expect_semi(&mut self) -> TsnatResult<()> {
        if self.match_kind(TokenKind::Semicolon) {
            Ok(())
        } else if self.peek().has_preceding_newline || self.peek().kind == TokenKind::RBrace || self.peek().kind == TokenKind::Eof {
            Ok(())
        } else {
            Err(TsnatError::Parse {
                message: format!("Expected ;, found {:?}", self.peek().kind),
                span: self.peek().span,
            })
        }
    }

    fn match_kind(&mut self, kind: TokenKind) -> bool {
        if self.peek().kind == kind {
            self.advance();
            true
        } else {
            false
        }
    }

    fn is_at_end(&self) -> bool {
        self.peek().kind == TokenKind::Eof
    }

    /// Allocate a value in the arena and return a reference.
    fn alloc<T>(&self, val: T) -> &'arena T {
        self.arena.alloc(val)
    }

    // ── Type parsing (TASK-010) ───────────────────────────

    fn parse_type_args(&mut self) -> TsnatResult<Option<NodeList<'arena, TypeNode<'arena>>>> {
        if self.peek().kind == TokenKind::Lt {
            self.advance();
            let mut args = Vec::new();
            while self.peek().kind != TokenKind::Gt && self.peek().kind != TokenKind::Eof {
                args.push(self.parse_type_node()?);
                if self.peek().kind == TokenKind::Comma {
                    self.advance();
                } else {
                    break;
                }
            }
            self.expect(TokenKind::Gt)?;
            Ok(Some(self.arena.alloc_slice_fill_iter(args)))
        } else {
            Ok(None)
        }
    }

    fn parse_type_node(&mut self) -> TsnatResult<TypeNode<'arena>> {
        let mut left = self.parse_primary_type_node()?;

        loop {
            match self.peek().kind {
                TokenKind::Pipe => {
                    self.advance();
                    let right = self.parse_primary_type_node()?;
                    let mut types = vec![left, right];
                    while self.match_kind(TokenKind::Pipe) {
                        types.push(self.parse_primary_type_node()?);
                    }
                    let span = left.span().merge(types.last().unwrap().span());
                    left = TypeNode::Union(self.arena.alloc_slice_fill_iter(types), span);
                }
                TokenKind::Amp => {
                    self.advance();
                    let right = self.parse_primary_type_node()?;
                    let mut types = vec![left, right];
                    while self.match_kind(TokenKind::Amp) {
                        types.push(self.parse_primary_type_node()?);
                    }
                    let span = left.span().merge(types.last().unwrap().span());
                    left = TypeNode::Intersection(self.arena.alloc_slice_fill_iter(types), span);
                }
                _ => break,
            }
        }

        Ok(left)
    }

    fn parse_primary_type_node(&mut self) -> TsnatResult<TypeNode<'arena>> {
        let mut ty = self.parse_base_type_node()?;
        
        // Handle suffixes like []
        while self.match_kind(TokenKind::LBracket) {
            self.expect(TokenKind::RBracket)?;
            ty = TypeNode::Array(AstArenaRef(self.arena.alloc(ty)));
        }

        Ok(ty)
    }

    fn parse_base_type_node(&mut self) -> TsnatResult<TypeNode<'arena>> {
        let tok = self.peek();
        match tok.kind {
            TokenKind::KwNumber => Ok(TypeNode::Number(self.advance().span)),
            TokenKind::KwString => Ok(TypeNode::String(self.advance().span)),
            TokenKind::KwBoolean => Ok(TypeNode::Boolean(self.advance().span)),
            TokenKind::KwBigInt => Ok(TypeNode::BigInt(self.advance().span)),
            TokenKind::KwSymbol => Ok(TypeNode::Symbol(self.advance().span)),
            TokenKind::KwNull => Ok(TypeNode::Null(self.advance().span)),
            TokenKind::KwUndefined => Ok(TypeNode::Undefined(self.advance().span)),
            TokenKind::KwVoid => Ok(TypeNode::Void(self.advance().span)),
            TokenKind::KwNever => Ok(TypeNode::Never(self.advance().span)),
            TokenKind::KwUnknown => Ok(TypeNode::Unknown(self.advance().span)),
            TokenKind::KwAny => Ok(TypeNode::Any(self.advance().span)),
            TokenKind::KwObject => Ok(TypeNode::Object(self.advance().span)),
            TokenKind::KwConst => {
                let name = self.advance();
                Ok(TypeNode::TypeRef(TypeRefNode { name: name.value, type_args: None, span: name.span }))
            }
            TokenKind::Number => {
                let tok = self.advance();
                let val = self.interner.get(tok.value).parse().unwrap_or(0.0);
                Ok(TypeNode::LiteralNumber(val, tok.span))
            }
            TokenKind::String => {
                let tok = self.advance();
                Ok(TypeNode::LiteralString(tok.value, tok.span))
            }
            TokenKind::KwTrue => Ok(TypeNode::LiteralBool(true, self.advance().span)),
            TokenKind::KwFalse => Ok(TypeNode::LiteralBool(false, self.advance().span)),
            TokenKind::Ident => {
                let name = self.advance();
                let mut type_args = None;
                if self.match_kind(TokenKind::Lt) {
                    let mut args = Vec::new();
                    while self.peek().kind != TokenKind::Gt && !self.is_at_end() {
                        args.push(self.parse_type_node()?);
                        if !self.match_kind(TokenKind::Comma) { break; }
                    }
                    self.expect(TokenKind::Gt)?;
                    type_args = Some(&*self.arena.alloc_slice_fill_iter(args));
                }
                let span = name.span.merge(type_args.map(|a| a.last().unwrap().span()).unwrap_or(name.span));
                Ok(TypeNode::TypeRef(TypeRefNode { name: name.value, type_args, span }))
            }
            TokenKind::LParen => {
                let start = self.advance().span;
                // Ambiguity between (T) and (params) => T
                let pos_backup = self.pos;
                let mut depth = 1;
                while depth > 0 && !self.is_at_end() {
                    match self.peek().kind {
                        TokenKind::LParen => depth += 1,
                        TokenKind::RParen => depth -= 1,
                        _ => {}
                    }
                    self.advance();
                }
                let is_func = self.peek().kind == TokenKind::Arrow;
                self.pos = pos_backup;

                if is_func {
                    self.parse_function_type()
                } else {
                    let inner = self.parse_type_node()?;
                    let end = self.expect(TokenKind::RParen)?.span;
                    Ok(TypeNode::Paren(AstArenaRef(self.arena.alloc(inner)), start.merge(end)))
                }
            }
            TokenKind::LBracket => {
                let start = self.advance().span;
                let mut elements = Vec::new();
                while self.peek().kind != TokenKind::RBracket && !self.is_at_end() {
                    elements.push(self.parse_type_node()?);
                    if !self.match_kind(TokenKind::Comma) { break; }
                }
                let end = self.expect(TokenKind::RBracket)?.span;
                Ok(TypeNode::Tuple(self.arena.alloc_slice_fill_iter(elements), start.merge(end)))
            }
            _ => Err(TsnatError::Parse {
                message: format!("Expected type node, found {}", self.peek().kind),
                span: self.peek().span,
            }),
        }
    }

    fn parse_function_type(&mut self) -> TsnatResult<TypeNode<'arena>> {
        let start = self.peek().span;
        self.expect(TokenKind::LParen)?;
        let mut params = Vec::new();
        while self.peek().kind != TokenKind::RParen && !self.is_at_end() {
            params.push(self.parse_param()?);
            if !self.match_kind(TokenKind::Comma) { break; }
        }
        self.expect(TokenKind::RParen)?;
        self.expect(TokenKind::Arrow)?;
        let return_ty = self.parse_type_node()?;
        let span = start.merge(return_ty.span());
        Ok(TypeNode::Function(FunctionTypeNode {
            params: self.arena.alloc_slice_fill_iter(params),
            return_ty: AstArenaRef(self.arena.alloc(return_ty)),
            span,
        }))
    }
}
