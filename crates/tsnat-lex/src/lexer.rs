use tsnat_common::diagnostic::{TsnatError, TsnatResult};
use tsnat_common::interner::{Interner, Symbol, SYM_EMPTY};
use tsnat_common::span::Span;

use crate::token::{Token, TokenKind};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LexMode {
    Normal,
    TemplateExpr(u32),
}

pub struct Lexer<'src> {
    source: &'src str,
    bytes: &'src [u8],
    pub file_id: u32,
    pos: u32,
    mode_stack: Vec<LexMode>,
    pub interner: &'src mut Interner,
    last_token_kind: Option<TokenKind>,
}

impl<'src> Lexer<'src> {
    pub fn new(source: &'src str, file_id: u32, interner: &'src mut Interner) -> Self {
        Self {
            source,
            bytes: source.as_bytes(),
            file_id,
            pos: 0,
            mode_stack: vec![LexMode::Normal],
            interner,
            last_token_kind: None,
        }
    }

    pub fn tokenise_all(&mut self) -> TsnatResult<Vec<Token>> {
        let mut tokens = Vec::new();
        loop {
            let t = self.next_token()?;
            let is_eof = matches!(t.kind, TokenKind::Eof);
            tokens.push(t);
            if is_eof {
                break;
            }
        }
        Ok(tokens)
    }

    pub fn next_token(&mut self) -> TsnatResult<Token> {
        let has_preceding_newline = self.skip_whitespace_and_comments();
        
        let start = self.pos;
        if self.eof() {
            return Ok(self.make_token(TokenKind::Eof, start, start, SYM_EMPTY, has_preceding_newline));
        }

        let c = self.peek().unwrap();

        // Identifiers and Keywords
        if self.is_ident_start(c) {
            return self.lex_ident_or_keyword(start, has_preceding_newline);
        }

        // Numbers
        if c.is_ascii_digit() || (c == b'.' && self.peek_n(1).map_or(false, |n| n.is_ascii_digit())) {
            return self.lex_number(start, has_preceding_newline);
        }

        // Strings
        if c == b'"' || c == b'\'' {
            return self.lex_string(start, has_preceding_newline, c);
        }

        // Templates
        if c == b'`' {
            return self.lex_template_head(start, has_preceding_newline);
        }

        // Regex vs Slash
        if c == b'/' {
            let next = self.peek_n(1);
            if next != Some(b'=') && self.is_regex_allowed() {
                return self.lex_regex(start, has_preceding_newline);
            }
        }

        // Operators & Punctuation
        self.lex_punctuation(start, has_preceding_newline)
    }

    fn eof(&self) -> bool {
        self.pos as usize >= self.bytes.len()
    }

    fn peek(&self) -> Option<u8> {
        self.bytes.get(self.pos as usize).copied()
    }

    fn peek_n(&self, n: usize) -> Option<u8> {
        self.bytes.get(self.pos as usize + n).copied()
    }

    fn advance(&mut self) {
        if !self.eof() {
            self.pos += 1;
        }
    }

    fn advance_n(&mut self, n: u32) {
        self.pos = (self.pos + n).min(self.bytes.len() as u32);
    }

    fn make_token(&mut self, kind: TokenKind, start: u32, end: u32, value: Symbol, has_preceding_newline: bool) -> Token {
        self.last_token_kind = Some(kind);
        Token {
            kind,
            value,
            span: Span { file_id: self.file_id, start, end },
            has_preceding_newline,
        }
    }

    fn is_regex_allowed(&self) -> bool {
        let Some(k) = self.last_token_kind else { return true; };
        matches!(
            k,
            TokenKind::Eq | TokenKind::LParen | TokenKind::Comma | TokenKind::LBracket
            | TokenKind::Bang | TokenKind::Amp | TokenKind::Pipe | TokenKind::Question
            | TokenKind::Colon | TokenKind::LBrace | TokenKind::RBrace | TokenKind::Semicolon
            | TokenKind::KwReturn | TokenKind::KwYield | TokenKind::Arrow | TokenKind::PlusEq
            | TokenKind::MinusEq | TokenKind::StarEq | TokenKind::SlashEq | TokenKind::PercentEq
            | TokenKind::StarStarEq | TokenKind::AmpEq | TokenKind::PipeEq | TokenKind::CaretEq
            | TokenKind::LtLtEq | TokenKind::GtGtEq | TokenKind::GtGtGtEq | TokenKind::AmpAmpEq
            | TokenKind::PipePipeEq | TokenKind::QuestionQuestionEq | TokenKind::EqEq | TokenKind::EqEqEq
            | TokenKind::BangEq | TokenKind::BangEqEq | TokenKind::KwThrow
        )
    }

    fn skip_whitespace_and_comments(&mut self) -> bool {
        let mut has_newline = false;
        while let Some(c) = self.peek() {
            if c == b' ' || c == b'\t' || c == b'\x0C' || c == b'\x0B' {
                self.advance();
            } else if c == b'\n' || c == b'\r' {
                has_newline = true;
                self.advance();
            } else if c == b'/' {
                if self.peek_n(1) == Some(b'/') {
                    self.advance_n(2);
                    while let Some(cc) = self.peek() {
                        if cc == b'\n' || cc == b'\r' {
                            break;
                        }
                        self.advance();
                    }
                } else if self.peek_n(1) == Some(b'*') {
                    self.advance_n(2);
                    while !self.eof() {
                        if self.peek() == Some(b'*') && self.peek_n(1) == Some(b'/') {
                            self.advance_n(2);
                            break;
                        }
                        let cc = self.peek().unwrap();
                        if cc == b'\n' || cc == b'\r' {
                            has_newline = true;
                        }
                        self.advance();
                    }
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        has_newline
    }

    fn is_ident_start(&self, c: u8) -> bool {
        c.is_ascii_alphabetic() || c == b'_' || c == b'$'
    }

    fn is_ident_part(&self, c: u8) -> bool {
        c.is_ascii_alphanumeric() || c == b'_' || c == b'$'
    }

    fn lex_ident_or_keyword(&mut self, start: u32, has_preceding_newline: bool) -> TsnatResult<Token> {
        while let Some(c) = self.peek() {
            if self.is_ident_part(c) {
                self.advance();
            } else {
                break;
            }
        }
        let text = &self.source[start as usize .. self.pos as usize];
        let kind = self.keyword_kind(text);
        let sym = if matches!(kind, TokenKind::Ident) {
            self.interner.intern(text)
        } else {
            SYM_EMPTY
        };
        Ok(self.make_token(kind, start, self.pos, sym, has_preceding_newline))
    }

    fn keyword_kind(&self, text: &str) -> TokenKind {
        match text {
            "break" => TokenKind::KwBreak,
            "case" => TokenKind::KwCase,
            "catch" => TokenKind::KwCatch,
            "class" => TokenKind::KwClass,
            "const" => TokenKind::KwConst,
            "continue" => TokenKind::KwContinue,
            "debugger" => TokenKind::KwDebugger,
            "default" => TokenKind::KwDefault,
            "delete" => TokenKind::KwDelete,
            "do" => TokenKind::KwDo,
            "else" => TokenKind::KwElse,
            "enum" => TokenKind::KwEnum,
            "export" => TokenKind::KwExport,
            "extends" => TokenKind::KwExtends,
            "false" => TokenKind::KwFalse,
            "finally" => TokenKind::KwFinally,
            "for" => TokenKind::KwFor,
            "function" => TokenKind::KwFunction,
            "if" => TokenKind::KwIf,
            "import" => TokenKind::KwImport,
            "in" => TokenKind::KwIn,
            "instanceof" => TokenKind::KwInstanceof,
            "let" => TokenKind::KwLet,
            "new" => TokenKind::KwNew,
            "null" => TokenKind::KwNull,
            "return" => TokenKind::KwReturn,
            "super" => TokenKind::KwSuper,
            "switch" => TokenKind::KwSwitch,
            "this" => TokenKind::KwThis,
            "throw" => TokenKind::KwThrow,
            "true" => TokenKind::KwTrue,
            "try" => TokenKind::KwTry,
            "typeof" => TokenKind::KwTypeof,
            "undefined" => TokenKind::KwUndefined,
            "var" => TokenKind::KwVar,
            "void" => TokenKind::KwVoid,
            "while" => TokenKind::KwWhile,
            "with" => TokenKind::KwWith,
            "yield" => TokenKind::KwYield,
            "async" => TokenKind::KwAsync,
            "await" => TokenKind::KwAwait,
            "of" => TokenKind::KwOf,
            "from" => TokenKind::KwFrom,
            "as" => TokenKind::KwAs,
            "satisfies" => TokenKind::KwSatisfies,
            "using" => TokenKind::KwUsing,
            "static" => TokenKind::KwStatic,
            "type" => TokenKind::KwType,
            "interface" => TokenKind::KwInterface,
            "namespace" => TokenKind::KwNamespace,
            "module" => TokenKind::KwModule,
            "declare" => TokenKind::KwDeclare,
            "abstract" => TokenKind::KwAbstract,
            "override" => TokenKind::KwOverride,
            "readonly" => TokenKind::KwReadonly,
            "keyof" => TokenKind::KwKeyof,
            "infer" => TokenKind::KwInfer,
            "is" => TokenKind::KwIs,
            "asserts" => TokenKind::KwAsserts,
            "public" => TokenKind::KwPublic,
            "private" => TokenKind::KwPrivate,
            "protected" => TokenKind::KwProtected,
            "never" => TokenKind::KwNever,
            "unknown" => TokenKind::KwUnknown,
            "any" => TokenKind::KwAny,
            "object" => TokenKind::KwObject,
            "symbol" => TokenKind::KwSymbol,
            "number" => TokenKind::KwNumber,
            "string" => TokenKind::KwString,
            "boolean" => TokenKind::KwBoolean,
            "bigint" => TokenKind::KwBigInt,
            "intrinsic" => TokenKind::KwIntrinsic,
            _ => TokenKind::Ident,
        }
    }

    fn lex_number(&mut self, start: u32, has_preceding_newline: bool) -> TsnatResult<Token> {
        let mut is_bigint = false;
        if self.peek() == Some(b'0') {
            let next = self.peek_n(1);
            if matches!(next, Some(b'x' | b'X' | b'o' | b'O' | b'b' | b'B')) {
                self.advance_n(2);
                while let Some(c) = self.peek() {
                    if c.is_ascii_hexdigit() || c == b'_' {
                        self.advance();
                    } else {
                        break;
                    }
                }
            } else {
                while let Some(c) = self.peek() {
                    if c.is_ascii_digit() || c == b'_' || c == b'.' {
                        self.advance();
                    } else if c == b'e' || c == b'E' {
                        self.advance();
                        if matches!(self.peek(), Some(b'+' | b'-')) {
                            self.advance();
                        }
                    } else {
                        break;
                    }
                }
            }
        } else {
            while let Some(c) = self.peek() {
                if c.is_ascii_digit() || c == b'_' || c == b'.' {
                    self.advance();
                } else if c == b'e' || c == b'E' {
                    self.advance();
                    if matches!(self.peek(), Some(b'+' | b'-')) {
                        self.advance();
                    }
                } else {
                    break;
                }
            }
        }

        if self.peek() == Some(b'n') {
            is_bigint = true;
            self.advance();
        }

        let text = &self.source[start as usize .. self.pos as usize];
        let sym = self.interner.intern(text);
        let kind = if is_bigint { TokenKind::BigInt } else { TokenKind::Number };
        Ok(self.make_token(kind, start, self.pos, sym, has_preceding_newline))
    }

    fn lex_string(&mut self, start: u32, has_preceding_newline: bool, quote: u8) -> TsnatResult<Token> {
        self.advance(); // skip quote
        let mut value_str = String::new();
        while let Some(c) = self.peek() {
            if c == quote {
                self.advance();
                break;
            } else if c == b'\\' {
                self.advance();
                if let Some(esc) = self.peek() {
                    match esc {
                        b'n' => { value_str.push('\n'); self.advance(); }
                        b'r' => { value_str.push('\r'); self.advance(); }
                        b't' => { value_str.push('\t'); self.advance(); }
                        b'\\' => { value_str.push('\\'); self.advance(); }
                        b'\'' => { value_str.push('\''); self.advance(); }
                        b'"' => { value_str.push('"'); self.advance(); }
                        b'0' => { value_str.push('\0'); self.advance(); }
                        _ => { value_str.push(esc as char); self.advance(); }
                    }
                }
            } else {
                value_str.push(c as char);
                self.advance();
            }
        }

        let sym = self.interner.intern(&value_str);
        Ok(self.make_token(TokenKind::String, start, self.pos, sym, has_preceding_newline))
    }

    fn lex_template_head(&mut self, start: u32, has_preceding_newline: bool) -> TsnatResult<Token> {
        self.advance(); // skip `
        let (val, is_tail) = self.lex_template_content()?;
        let sym = self.interner.intern(&val);
        if is_tail {
            Ok(self.make_token(TokenKind::NoSubstTemplate, start, self.pos, sym, has_preceding_newline))
        } else {
            self.mode_stack.push(LexMode::TemplateExpr(0));
            Ok(self.make_token(TokenKind::TemplateHead, start, self.pos, sym, has_preceding_newline))
        }
    }

    fn lex_template_tail(&mut self, start: u32, has_preceding_newline: bool) -> TsnatResult<Token> {
        self.advance(); // consume }
        let (val, is_tail) = self.lex_template_content()?;
        let sym = self.interner.intern(&val);
        if is_tail {
            Ok(self.make_token(TokenKind::TemplateTail, start, self.pos, sym, has_preceding_newline))
        } else {
            self.mode_stack.push(LexMode::TemplateExpr(0));
            Ok(self.make_token(TokenKind::TemplateMiddle, start, self.pos, sym, has_preceding_newline))
        }
    }

    fn lex_template_content(&mut self) -> TsnatResult<(String, bool)> {
        let mut val = String::new();
        while let Some(c) = self.peek() {
            if c == b'`' {
                self.advance();
                return Ok((val, true));
            } else if c == b'$' && self.peek_n(1) == Some(b'{') {
                self.advance_n(2);
                return Ok((val, false));
            } else if c == b'\\' {
                self.advance();
                if let Some(esc) = self.peek() {
                    match esc {
                        b'n' => { val.push('\n'); self.advance(); }
                        b'r' => { val.push('\r'); self.advance(); }
                        b't' => { val.push('\t'); self.advance(); }
                        b'\\' => { val.push('\\'); self.advance(); }
                        b'`' => { val.push('`'); self.advance(); }
                        b'$' => { val.push('$'); self.advance(); }
                        b'{' => { val.push('{'); self.advance(); }
                        _ => { val.push(esc as char); self.advance(); }
                    }
                }
            } else {
                val.push(c as char);
                self.advance();
            }
        }
        Err(TsnatError::Lex {
            message: "Unterminated template literal".into(),
            span: Span { file_id: self.file_id, start: self.pos, end: self.pos },
        })
    }

    fn lex_regex(&mut self, start: u32, has_preceding_newline: bool) -> TsnatResult<Token> {
        self.advance(); // skip /
        let mut in_class = false;
        while let Some(c) = self.peek() {
            if c == b'/' && !in_class {
                self.advance();
                break;
            } else if c == b'[' {
                in_class = true;
                self.advance();
            } else if c == b']' {
                in_class = false;
                self.advance();
            } else if c == b'\\' {
                self.advance_n(2);
            } else {
                self.advance();
            }
        }
        // flags
        while let Some(c) = self.peek() {
            if c.is_ascii_alphabetic() {
                self.advance();
            } else {
                break;
            }
        }
        let text = &self.source[start as usize .. self.pos as usize];
        let sym = self.interner.intern(text);
        Ok(self.make_token(TokenKind::Regex, start, self.pos, sym, has_preceding_newline))
    }

    fn lex_punctuation(&mut self, start: u32, has_preceding_newline: bool) -> TsnatResult<Token> {
        let c = self.peek().unwrap();
        let c2 = self.peek_n(1);
        let c3 = self.peek_n(2);
        let c4 = self.peek_n(3);

        let kind = match (c, c2, c3, c4) {
            (b'=', Some(b'='), Some(b'='), _) => { self.advance_n(3); TokenKind::EqEqEq }
            (b'!', Some(b'='), Some(b'='), _) => { self.advance_n(3); TokenKind::BangEqEq }
            (b'>', Some(b'>'), Some(b'>'), Some(b'=')) => { self.advance_n(4); TokenKind::GtGtGtEq }
            (b'>', Some(b'>'), Some(b'>'), _) => { self.advance_n(3); TokenKind::GtGtGt }
            (b'<', Some(b'<'), Some(b'='), _) => { self.advance_n(3); TokenKind::LtLtEq }
            (b'>', Some(b'>'), Some(b'='), _) => { self.advance_n(3); TokenKind::GtGtEq }
            (b'*', Some(b'*'), Some(b'='), _) => { self.advance_n(3); TokenKind::StarStarEq }
            (b'&', Some(b'&'), Some(b'='), _) => { self.advance_n(3); TokenKind::AmpAmpEq }
            (b'|', Some(b'|'), Some(b'='), _) => { self.advance_n(3); TokenKind::PipePipeEq }
            (b'?', Some(b'?'), Some(b'='), _) => { self.advance_n(3); TokenKind::QuestionQuestionEq }
            (b'.', Some(b'.'), Some(b'.'), _) => { self.advance_n(3); TokenKind::DotDotDot }
            (b'=', Some(b'='), _, _) => { self.advance_n(2); TokenKind::EqEq }
            (b'!', Some(b'='), _, _) => { self.advance_n(2); TokenKind::BangEq }
            (b'<', Some(b'='), _, _) => { self.advance_n(2); TokenKind::LtEq }
            (b'>', Some(b'='), _, _) => { self.advance_n(2); TokenKind::GtEq }
            (b'+', Some(b'='), _, _) => { self.advance_n(2); TokenKind::PlusEq }
            (b'-', Some(b'='), _, _) => { self.advance_n(2); TokenKind::MinusEq }
            (b'*', Some(b'='), _, _) => { self.advance_n(2); TokenKind::StarEq }
            (b'/', Some(b'='), _, _) => { self.advance_n(2); TokenKind::SlashEq }
            (b'%', Some(b'='), _, _) => { self.advance_n(2); TokenKind::PercentEq }
            (b'&', Some(b'='), _, _) => { self.advance_n(2); TokenKind::AmpEq }
            (b'|', Some(b'='), _, _) => { self.advance_n(2); TokenKind::PipeEq }
            (b'^', Some(b'='), _, _) => { self.advance_n(2); TokenKind::CaretEq }
            (b'+', Some(b'+'), _, _) => { self.advance_n(2); TokenKind::PlusPlus }
            (b'-', Some(b'-'), _, _) => { self.advance_n(2); TokenKind::MinusMinus }
            (b'*', Some(b'*'), _, _) => { self.advance_n(2); TokenKind::StarStar }
            (b'&', Some(b'&'), _, _) => { self.advance_n(2); TokenKind::AmpAmp }
            (b'|', Some(b'|'), _, _) => { self.advance_n(2); TokenKind::PipePipe }
            (b'?', Some(b'?'), _, _) => { self.advance_n(2); TokenKind::QuestionQuestion }
            (b'?', Some(b'.'), _, _) => { self.advance_n(2); TokenKind::QuestionDot }
            (b'=', Some(b'>'), _, _) => { self.advance_n(2); TokenKind::Arrow }
            (b'<', Some(b'<'), _, _) => { self.advance_n(2); TokenKind::LtLt }
            (b'>', Some(b'>'), _, _) => { self.advance_n(2); TokenKind::GtGt }
            (b'(', _, _, _) => { self.advance(); TokenKind::LParen }
            (b')', _, _, _) => { self.advance(); TokenKind::RParen }
            (b'{', _, _, _) => {
                if let Some(LexMode::TemplateExpr(d)) = self.mode_stack.last_mut() {
                    *d += 1;
                }
                self.advance(); TokenKind::LBrace
            }
            (b'}', _, _, _) => {
                if let Some(&LexMode::TemplateExpr(d)) = self.mode_stack.last() {
                    if d == 0 {
                        self.mode_stack.pop();
                        return self.lex_template_tail(start, has_preceding_newline);
                    } else {
                        if let Some(LexMode::TemplateExpr(d)) = self.mode_stack.last_mut() {
                            *d -= 1;
                        }
                    }
                }
                self.advance(); TokenKind::RBrace
            }
            (b'[', _, _, _) => { self.advance(); TokenKind::LBracket }
            (b']', _, _, _) => { self.advance(); TokenKind::RBracket }
            (b';', _, _, _) => { self.advance(); TokenKind::Semicolon }
            (b':', _, _, _) => { self.advance(); TokenKind::Colon }
            (b',', _, _, _) => { self.advance(); TokenKind::Comma }
            (b'.', _, _, _) => { self.advance(); TokenKind::Dot }
            (b'?', _, _, _) => { self.advance(); TokenKind::Question }
            (b'+', _, _, _) => { self.advance(); TokenKind::Plus }
            (b'-', _, _, _) => { self.advance(); TokenKind::Minus }
            (b'*', _, _, _) => { self.advance(); TokenKind::Star }
            (b'/', _, _, _) => { self.advance(); TokenKind::Slash }
            (b'%', _, _, _) => { self.advance(); TokenKind::Percent }
            (b'&', _, _, _) => { self.advance(); TokenKind::Amp }
            (b'|', _, _, _) => { self.advance(); TokenKind::Pipe }
            (b'^', _, _, _) => { self.advance(); TokenKind::Caret }
            (b'~', _, _, _) => { self.advance(); TokenKind::Tilde }
            (b'!', _, _, _) => { self.advance(); TokenKind::Bang }
            (b'=', _, _, _) => { self.advance(); TokenKind::Eq }
            (b'<', _, _, _) => { self.advance(); TokenKind::Lt }
            (b'>', _, _, _) => { self.advance(); TokenKind::Gt }
            (b'@', _, _, _) => { self.advance(); TokenKind::At }
            _ => {
                return Err(TsnatError::Lex {
                    message: format!("Unexpected character: {:?}", c as char),
                    span: Span { file_id: self.file_id, start, end: start + 1 },
                });
            }
        };
        Ok(self.make_token(kind, start, self.pos, SYM_EMPTY, has_preceding_newline))
    }
}
