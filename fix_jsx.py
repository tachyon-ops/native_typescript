import os, sys

def fix_parser(filepath):
    with open(filepath, 'r') as f:
        content = f.read()
    
    jsx_parser = """
    // ── JSX Parsing ──────────────────────────────────────────

    fn parse_jsx_element(&mut self) -> TsnatResult<Expr<'arena>> {
        let start_span = self.previous().span; // `<`
        let tag_tok = self.expect(TokenKind::Ident)?;
        let tag = tag_tok.value;

        // Skip props for now...
        while !self.match_kind(TokenKind::Gt) && !self.match_kind(TokenKind::JsxTagClose) && !self.is_at_end() {
            self.advance();
        }

        let mut children = Vec::new();
        let mut text_start = self.peek().span.start;

        while !self.is_at_end() {
            if self.peek().kind == TokenKind::Lt {
                // Determine if it's nested JSX or closing tag
                let lt_tok = self.advance();
                if self.peek().kind == TokenKind::Slash {
                    let slash_tok = self.advance();
                    // Flush pending text before the closing tag
                    if text_start < lt_tok.span.start {
                        let span = Span::new(text_start, lt_tok.span.start);
                        children.push(self.alloc(Expr::JSXText(tsnat_common::interner::SYM_EMPTY, span)));
                    }

                    // Look for tag close
                    self.expect(TokenKind::Ident)?;
                    self.expect(TokenKind::Gt)?;
                    break;
                } else {
                    // Flush pending text before nested tag
                    if text_start < lt_tok.span.start {
                        let span = Span::new(text_start, lt_tok.span.start);
                        children.push(self.alloc(Expr::JSXText(tsnat_common::interner::SYM_EMPTY, span)));
                    }
                    // Rewind the `pos` backward by 1 because we consumed `<` but `parse_jsx_element` expects `<`
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
                    let span = Span::new(text_start, lbrace_tok.span.start);
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

        let end_span = self.previous().span;
        Ok(Expr::JSXElement(JSXElement {
            tag,
            children: self.arena.alloc_slice_fill_iter(children),
            span: start_span.merge(end_span),
        }))
    }
"""

    if "parse_jsx_element" not in content:
        # insert before array literal
        content = content.replace("    // ── Array literal", jsx_parser + "\n    // ── Array literal")

        # add JSX dispatch in parse_primary_expr
        content = content.replace("            TokenKind::Ident => {", """            TokenKind::Lt => {
                let p = self.pos;
                let next = self.peek_ahead(1);
                if next.kind == TokenKind::Ident { // `<` followed by Ident -> JSX
                    self.advance(); // consume `<`
                    return self.parse_jsx_element();
                } else {
                    return Err(TsnatError::Parse {
                        message: "Expected JSX or type arg".into(),
                        span: tok.span,
                    });
                }
            }
            TokenKind::Ident => {""")
        with open(filepath, 'w') as f:
            f.write(content)

fix_parser("crates/tsnat-parse/src/parser.rs")
