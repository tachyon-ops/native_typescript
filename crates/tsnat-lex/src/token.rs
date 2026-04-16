use std::fmt;
use tsnat_common::interner::Symbol;
use tsnat_common::span::Span;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum TokenKind {
    // ── Literals ──────────────────────────────────────
    Number,          // 42  3.14  0xFF  0b1010  1_000_000
    BigInt,          // 42n
    String,          // "foo"  'bar'
    TemplateHead,    // `before ${
    TemplateMiddle,  // } middle ${
    TemplateTail,    // } after`
    NoSubstTemplate, // `no substitutions`
    Regex,           // /pattern/flags

    // ── Identifiers & keywords ────────────────────────
    Ident,

    // Value keywords
    KwBreak, KwCase, KwCatch, KwClass, KwConst, KwContinue,
    KwDebugger, KwDefault, KwDelete, KwDo, KwElse, KwEnum,
    KwExport, KwExtends, KwFalse, KwFinally, KwFor, KwFunction,
    KwIf, KwImport, KwIn, KwInstanceof, KwLet, KwNew, KwNull,
    KwReturn, KwSuper, KwSwitch, KwThis, KwThrow, KwTrue,
    KwTry, KwTypeof, KwUndefined, KwVar, KwVoid, KwWhile, KwWith,
    KwYield, KwAsync, KwAwait, KwOf, KwFrom, KwAs, KwSatisfies,
    KwUsing, KwStatic,

    // Type keywords
    KwType, KwInterface, KwNamespace, KwModule, KwDeclare,
    KwAbstract, KwOverride, KwReadonly, KwKeyof, KwInfer,
    KwIs, KwAsserts, KwPublic, KwPrivate, KwProtected,
    KwNever, KwUnknown, KwAny, KwObject, KwSymbol,
    KwIntrinsic,

    // ── Punctuation ───────────────────────────────────
    LParen, RParen, LBrace, RBrace, LBracket, RBracket,
    Semicolon, Colon, Comma, Dot, DotDotDot, QuestionDot,
    Question, QuestionQuestion,

    // ── Operators ─────────────────────────────────────
    Plus, Minus, Star, Slash, Percent, StarStar,
    Amp, Pipe, Caret, Tilde, LtLt, GtGt, GtGtGt,
    Bang, AmpAmp, PipePipe,
    Eq, EqEq, EqEqEq, BangEq, BangEqEq,
    Lt, Gt, LtEq, GtEq,
    Arrow, // =>
    PlusEq, MinusEq, StarEq, SlashEq, PercentEq, StarStarEq,
    AmpEq, PipeEq, CaretEq, LtLtEq, GtGtEq, GtGtGtEq,
    AmpAmpEq, PipePipeEq, QuestionQuestionEq,
    PlusPlus, MinusMinus,
    At,     // decorator @

    // ── JSX ───────────────────────────────────────────
    JsxText,
    JsxTagOpen,   // <TagName
    JsxTagClose,  // </TagName> or />
    JsxLBrace,    // { inside JSX
    JsxRBrace,    // } inside JSX

    // ── Special ───────────────────────────────────────
    Eof,
}

impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Number => "number",
            Self::BigInt => "bigint",
            Self::String => "string",
            Self::TemplateHead => "`...${",
            Self::TemplateMiddle => "}...${",
            Self::TemplateTail => "}...`",
            Self::NoSubstTemplate => "`...`",
            Self::Regex => "regex",
            Self::Ident => "identifier",
            Self::KwBreak => "break",
            Self::KwCase => "case",
            Self::KwCatch => "catch",
            Self::KwClass => "class",
            Self::KwConst => "const",
            Self::KwContinue => "continue",
            Self::KwDebugger => "debugger",
            Self::KwDefault => "default",
            Self::KwDelete => "delete",
            Self::KwDo => "do",
            Self::KwElse => "else",
            Self::KwEnum => "enum",
            Self::KwExport => "export",
            Self::KwExtends => "extends",
            Self::KwFalse => "false",
            Self::KwFinally => "finally",
            Self::KwFor => "for",
            Self::KwFunction => "function",
            Self::KwIf => "if",
            Self::KwImport => "import",
            Self::KwIn => "in",
            Self::KwInstanceof => "instanceof",
            Self::KwLet => "let",
            Self::KwNew => "new",
            Self::KwNull => "null",
            Self::KwReturn => "return",
            Self::KwSuper => "super",
            Self::KwSwitch => "switch",
            Self::KwThis => "this",
            Self::KwThrow => "throw",
            Self::KwTrue => "true",
            Self::KwTry => "try",
            Self::KwTypeof => "typeof",
            Self::KwUndefined => "undefined",
            Self::KwVar => "var",
            Self::KwVoid => "void",
            Self::KwWhile => "while",
            Self::KwWith => "with",
            Self::KwYield => "yield",
            Self::KwAsync => "async",
            Self::KwAwait => "await",
            Self::KwOf => "of",
            Self::KwFrom => "from",
            Self::KwAs => "as",
            Self::KwSatisfies => "satisfies",
            Self::KwUsing => "using",
            Self::KwStatic => "static",
            Self::KwType => "type",
            Self::KwInterface => "interface",
            Self::KwNamespace => "namespace",
            Self::KwModule => "module",
            Self::KwDeclare => "declare",
            Self::KwAbstract => "abstract",
            Self::KwOverride => "override",
            Self::KwReadonly => "readonly",
            Self::KwKeyof => "keyof",
            Self::KwInfer => "infer",
            Self::KwIs => "is",
            Self::KwAsserts => "asserts",
            Self::KwPublic => "public",
            Self::KwPrivate => "private",
            Self::KwProtected => "protected",
            Self::KwNever => "never",
            Self::KwUnknown => "unknown",
            Self::KwAny => "any",
            Self::KwObject => "object",
            Self::KwSymbol => "symbol",
            Self::KwIntrinsic => "intrinsic",
            Self::LParen => "(",
            Self::RParen => ")",
            Self::LBrace => "{",
            Self::RBrace => "}",
            Self::LBracket => "[",
            Self::RBracket => "]",
            Self::Semicolon => ";",
            Self::Colon => ":",
            Self::Comma => ",",
            Self::Dot => ".",
            Self::DotDotDot => "...",
            Self::QuestionDot => "?.",
            Self::Question => "?",
            Self::QuestionQuestion => "??",
            Self::Plus => "+",
            Self::Minus => "-",
            Self::Star => "*",
            Self::Slash => "/",
            Self::Percent => "%",
            Self::StarStar => "**",
            Self::Amp => "&",
            Self::Pipe => "|",
            Self::Caret => "^",
            Self::Tilde => "~",
            Self::LtLt => "<<",
            Self::GtGt => ">>",
            Self::GtGtGt => ">>>",
            Self::Bang => "!",
            Self::AmpAmp => "&&",
            Self::PipePipe => "||",
            Self::Eq => "=",
            Self::EqEq => "==",
            Self::EqEqEq => "===",
            Self::BangEq => "!=",
            Self::BangEqEq => "!==",
            Self::Lt => "<",
            Self::Gt => ">",
            Self::LtEq => "<=",
            Self::GtEq => ">=",
            Self::Arrow => "=>",
            Self::PlusEq => "+=",
            Self::MinusEq => "-=",
            Self::StarEq => "*=",
            Self::SlashEq => "/=",
            Self::PercentEq => "%=",
            Self::StarStarEq => "**=",
            Self::AmpEq => "&=",
            Self::PipeEq => "|=",
            Self::CaretEq => "^=",
            Self::LtLtEq => "<<=",
            Self::GtGtEq => ">>=",
            Self::GtGtGtEq => ">>>=",
            Self::AmpAmpEq => "&&=",
            Self::PipePipeEq => "||=",
            Self::QuestionQuestionEq => "??=",
            Self::PlusPlus => "++",
            Self::MinusMinus => "--",
            Self::At => "@",
            Self::JsxText => "JSX text",
            Self::JsxTagOpen => "<JSXTag",
            Self::JsxTagClose => "</JSXTag>",
            Self::JsxLBrace => "{ (JSX)",
            Self::JsxRBrace => "} (JSX)",
            Self::Eof => "EOF",
        };
        write!(f, "{}", s)
    }
}

pub struct Token {
    pub kind: TokenKind,
    /// Interned text value (empty for punctuation/operators that have no variable content).
    pub value: Symbol,
    pub span: Span,
    pub has_preceding_newline: bool, // needed for ASI
}
