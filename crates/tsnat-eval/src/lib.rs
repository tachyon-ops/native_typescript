pub mod value;
pub mod env;
pub mod eval;

use tsnat_common::diagnostic::TsnatResult;
use tsnat_common::interner::Interner;
use tsnat_parse::ast::Program;

use std::path::Path;
use crate::eval::Evaluator;
pub use crate::value::{Value, JsObject, JsFunction};

pub fn evaluate<'a>(program: &'a Program<'a>, interner: &mut Interner, arena: &'a bumpalo::Bump) -> TsnatResult<Value<'a>> {
    let mut evaluator = Evaluator::new(interner, arena);
    evaluator.eval_program(program)
}

pub struct Interpreter<'a> {
    pub interner: Interner,
    pub arena: bumpalo::Bump,
    _phantom: std::marker::PhantomData<&'a ()>,
}

impl<'a> Interpreter<'a> {
    pub fn new() -> Self {
        Self {
            interner: Interner::new(),
            arena: bumpalo::Bump::new(),
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn eval_str(&mut self, src: &str) -> TsnatResult<Value<'a>> {
        // Extend arena lifetime to 'a for testing purposes, matching parse().
        let arena: &'a bumpalo::Bump = unsafe { std::mem::transmute(&self.arena) };
        let builtins = include_str!("builtins/array.ts");
        let combined = format!("{}\n{}", builtins, src);
        let mut lexer = tsnat_lex::lexer::Lexer::new(&combined, 0, &mut self.interner);
        let tokens = lexer.tokenise_all()?;
        let mut parser = tsnat_parse::parser::Parser::new(&tokens, arena, &mut self.interner);
        let ast = parser.parse_program()?;
        let ast_ref = arena.alloc(ast);
        let mut evaluator = crate::eval::Evaluator::new(&mut self.interner, arena);
        evaluator.eval_program(ast_ref)
    }

    pub fn eval_file(&mut self, path: &Path) -> TsnatResult<Value<'a>> {
        let src = std::fs::read_to_string(path).map_err(|e| tsnat_common::diagnostic::TsnatError::Runtime {
            message: format!("Failed to read file: {}", e),
            span: None,
        })?;
        self.eval_str(&src)
    }
}
