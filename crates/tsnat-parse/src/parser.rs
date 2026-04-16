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
            TokenKind::KwConst | TokenKind::KwLet | TokenKind::KwVar => {
                Ok(Stmt::Var(self.parse_var_decl()?))
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

    fn parse_var_decl(&mut self) -> TsnatResult<VarDecl<'arena>> {
        let start = self.peek().span;
        let kind = match self.advance().kind {
            TokenKind::KwConst => VarKind::Const,
            TokenKind::KwLet => VarKind::Let,
            TokenKind::KwVar => VarKind::Var,
            _ => unreachable!(),
        };

        let mut decls = Vec::new();
        loop {
            let name_tok = self.expect(TokenKind::Ident)?;
            let name = name_tok.value;
            let name_span = name_tok.span;

            // Skip optional type annotation `: Type`
            if self.peek().kind == TokenKind::Colon {
                self.advance(); // ':'
                self.skip_type_annotation();
            }

            let init = if self.match_kind(TokenKind::Eq) {
                let e = self.parse_assignment_expr()?;
                Some(self.alloc(e))
            } else {
                None
            };
            let span = if let Some(init) = init {
                name_span.merge(init.span())
            } else {
                name_span
            };
            decls.push(VarDeclarator { name, init, span });
            if !self.match_kind(TokenKind::Comma) { break; }
        }

        let end = self.expect(TokenKind::Semicolon)?.span;
        Ok(VarDecl {
            kind,
            decls: self.arena.alloc_slice_fill_iter(decls),
            span: start.merge(end),
        })
    }

    /// Skips a type annotation after `:`. Handles balanced `<>`, `()`, `[]`,
    /// `{}`, and stops at `,`, `=`, `;`, `)`, `]`, or `Eof`.
    fn skip_type_annotation(&mut self) {
        let mut depth = 0i32;
        loop {
            match self.peek().kind {
                TokenKind::Lt => { depth += 1; self.advance(); }
                TokenKind::Gt if depth > 0 => { depth -= 1; self.advance(); }
                TokenKind::LParen | TokenKind::LBracket | TokenKind::LBrace => {
                    depth += 1; self.advance();
                }
                TokenKind::RParen | TokenKind::RBracket | TokenKind::RBrace if depth > 0 => {
                    depth -= 1; self.advance();
                }
                // Stop tokens when at depth 0
                TokenKind::Eq | TokenKind::Comma | TokenKind::Semicolon
                | TokenKind::RParen | TokenKind::RBracket | TokenKind::RBrace
                | TokenKind::Arrow | TokenKind::Eof if depth <= 0 => break,
                _ => { self.advance(); }
            }
        }
    }

    // ── Expression parsing (Pratt / precedence climbing) ───────

    /// Top-level expression: handles comma sequences if needed.
    fn parse_expr(&mut self) -> TsnatResult<Expr<'arena>> {
        self.parse_assignment_expr()
    }

    /// Assignment is right-associative, lowest precedence above comma.
    fn parse_assignment_expr(&mut self) -> TsnatResult<Expr<'arena>> {
        let left = self.parse_conditional_expr()?;

        if let Some(op) = self.assignment_op() {
            self.advance();
            let right = self.parse_assignment_expr()?;
            let span = left.span().merge(right.span());
            let left = self.alloc(left);
            let right = self.alloc(right);
            return Ok(Expr::Assign(AssignExpr { op, left, right, span }));
        }

        // Arrow: `(params) => body` or `ident => body`
        if self.peek().kind == TokenKind::Arrow {
            return self.parse_arrow_tail(left);
        }

        Ok(left)
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

        // Handle `as` type assertion at this level
        if self.peek().kind == TokenKind::KwAs {
            self.advance();
            let start_span = left.span();
            self.skip_type_annotation();
            let span = start_span; // simplified
            let expr = self.alloc(left);
            return Ok(Expr::As(AsExpr { expr, span }));
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
            TokenKind::DotDotDot => {
                self.advance();
                let arg = self.parse_assignment_expr()?;
                let span = start.merge(arg.span());
                let arg = self.alloc(arg);
                Ok(Expr::Spread(SpreadExpr { argument: arg, span }))
            }
            TokenKind::KwNew => {
                self.advance();
                let callee = self.parse_member_expr()?;
                let (args, end) = if self.peek().kind == TokenKind::LParen {
                    self.parse_arguments()?
                } else {
                    (&[] as &[&Expr], callee.span())
                };
                let span = start.merge(end);
                let callee = self.alloc(callee);
                Ok(Expr::New(NewExpr { callee, args, span }))
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
                    let prop_tok = self.expect(TokenKind::Ident)?;
                    let span = expr.span().merge(prop_tok.span);
                    let object = self.alloc(expr);
                    expr = Expr::Member(MemberExpr {
                        object, property: prop_tok.value, span,
                    });
                }
                TokenKind::QuestionDot => {
                    self.advance();
                    let prop_tok = self.expect(TokenKind::Ident)?;
                    let span = expr.span().merge(prop_tok.span);
                    let object = self.alloc(expr);
                    expr = Expr::OptChain(OptChainExpr {
                        object, property: prop_tok.value, span,
                    });
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
                TokenKind::LParen => {
                    let (args, end) = self.parse_arguments()?;
                    let span = expr.span().merge(end);
                    let callee = self.alloc(expr);
                    expr = Expr::Call(CallExpr { callee, args, span });
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
                    let prop = self.expect(TokenKind::Ident)?;
                    let span = expr.span().merge(prop.span);
                    let object = self.alloc(expr);
                    expr = Expr::Member(MemberExpr {
                        object, property: prop.value, span,
                    });
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
            _ => Err(TsnatError::Parse {
                message: format!("Unexpected token: {}", self.peek().kind),
                span: self.peek().span,
            }),
        }
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
            props.push(ObjProp { key, value, span });
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

    fn expr_to_arrow_params(&self, expr: &Expr<'arena>) -> TsnatResult<Vec<ArrowParam>> {
        match expr {
            Expr::Ident(sym, span) => Ok(vec![ArrowParam { name: *sym, span: *span }]),
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

    fn advance(&mut self) -> &'src Token {
        if !self.is_at_end() {
            self.pos += 1;
        }
        &self.tokens[self.pos - 1]
    }

    fn expect(&mut self, kind: TokenKind) -> TsnatResult<&'src Token> {
        if self.peek().kind == kind {
            Ok(self.advance())
        } else {
            Err(TsnatError::Parse {
                message: format!("Expected {}, found {}", kind, self.peek().kind),
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
}
