pub mod value;
pub mod env;
pub mod eval;

use tsnat_common::diagnostic::TsnatResult;
use tsnat_common::interner::Interner;
use tsnat_parse::ast::Program;
use crate::eval::Evaluator;
use crate::value::Value;

pub fn evaluate<'a>(program: &'a Program<'a>, interner: &mut Interner, arena: &'a bumpalo::Bump) -> TsnatResult<Value<'a>> {
    let mut evaluator = Evaluator::new(interner, arena);
    evaluator.eval_program(program)
}
