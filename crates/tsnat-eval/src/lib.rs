pub mod value;
pub mod env;
pub mod eval;

use tsnat_common::diagnostic::TsnatResult;
use tsnat_common::interner::Interner;
use tsnat_parse::ast::Program;
use tsnat_parse::arena::NodeList;
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

    pub fn eval_str(&mut self, _src: &str) -> TsnatResult<Value<'a>> {
        Ok(Value::Undefined)
    }

    pub fn eval_file(&mut self, _path: &Path) -> TsnatResult<Value<'a>> {
        Ok(Value::Undefined)
    }
}
