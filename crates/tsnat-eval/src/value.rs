use std::rc::Rc;
use std::cell::RefCell;
use std::fmt;
use indexmap::IndexMap;
use tsnat_common::interner::Symbol;
use tsnat_parse::ast::{Param, Stmt};
use tsnat_parse::arena::NodeList;
use crate::env::Environment;

#[derive(Clone)]
pub enum Value<'a> {
    Undefined,
    Null,
    Bool(bool),
    Number(f64),
    BigInt(i128),
    String(Rc<str>),
    Symbol(Symbol),
    Object(Rc<RefCell<JsObject<'a>>>),
    Function(Rc<JsFunction<'a>>),
    NativeFunction(Rc<dyn Fn(Vec<Value<'a>>, Option<Value<'a>>) -> tsnat_common::diagnostic::TsnatResult<Value<'a>>>),
}

pub struct JsObject<'a> {
    pub properties: IndexMap<Symbol, Value<'a>>,
    pub prototype: Option<Rc<RefCell<JsObject<'a>>>>,
}

pub enum FuncBody<'a> {
    Stmts(NodeList<'a, Stmt<'a>>),
    Expr(&'a tsnat_parse::ast::Expr<'a>),
}

pub struct JsFunction<'a> {
    pub name: Option<Symbol>,
    pub params: NodeList<'a, Param<'a>>,
    pub body: FuncBody<'a>,
    pub closure: Rc<RefCell<Environment<'a>>>,
    pub properties: RefCell<IndexMap<Symbol, Value<'a>>>,
}

impl<'a> Value<'a> {
    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Undefined | Value::Null => false,
            Value::Bool(b) => *b,
            Value::Number(n) => *n != 0.0 && !n.is_nan(),
            Value::BigInt(n) => *n != 0,
            Value::String(s) => !s.is_empty(),
            _ => true,
        }
    }

    pub fn display(&self) -> String {
        format!("{:?}", self)
    }
}

impl<'a> fmt::Debug for Value<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Undefined => write!(f, "undefined"),
            Value::Null => write!(f, "null"),
            Value::Bool(b) => write!(f, "{}", b),
            Value::Number(n) => write!(f, "{}", n),
            Value::BigInt(n) => write!(f, "{}n", n),
            Value::String(s) => write!(f, "{:?}", s),
            Value::Symbol(s) => write!(f, "Symbol({:?})", s),
            Value::Object(_) => write!(f, "[object Object]"),
            Value::Function(_) | Value::NativeFunction(_) => write!(f, "[function]"),
        }
    }
}
