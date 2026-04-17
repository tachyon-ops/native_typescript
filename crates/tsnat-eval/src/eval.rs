use std::rc::Rc;
use std::cell::RefCell;
use tsnat_common::diagnostic::{TsnatError, TsnatResult};
use tsnat_common::interner::{Interner, Symbol};
use tsnat_common::span::Span;
use tsnat_parse::ast::*;
use crate::value::{Value, JsObject, JsFunction};
use crate::env::Environment;
use indexmap::IndexMap;

pub enum ControlFlow<'a> {
    Normal(Value<'a>),
    Return(Value<'a>),
    Break,
    Continue,
}

pub struct Evaluator<'a, 'src> {
    pub env: Rc<RefCell<Environment<'a>>>,
    pub interner: &'src mut Interner,
    pub ffi_loader: std::sync::Arc<tsnat_ffi::loader::NativeLibraryLoader>,
    pub click_handlers: IndexMap<u32, Value<'a>>,
}

impl<'a, 'src> Evaluator<'a, 'src> {
    pub fn new(interner: &'src mut Interner) -> Self {
        let global_env = Rc::new(RefCell::new(Environment::new(None)));
        let ffi_loader = std::sync::Arc::new(tsnat_ffi::loader::NativeLibraryLoader::new());
        
        // Inject console.log
        let console = Rc::new(RefCell::new(JsObject {
            properties: IndexMap::default(),
            prototype: None,
        }));
        
        let log = Value::NativeFunction(Rc::new(|args, _| {
            for (i, arg) in args.iter().enumerate() {
                if i > 0 { print!(" "); }
                match arg {
                    Value::String(s) => print!("{}", s),
                    _ => print!("{:?}", arg),
                }
            }
            println!();
            Ok(Value::Undefined)
        }));
        
        let log_sym = interner.intern("log");
        console.borrow_mut().properties.insert(log_sym, log);
        
        let console_sym = interner.intern("console");
        global_env.borrow_mut().define(console_sym, Value::Object(console));

        Self {
            env: global_env,
            interner,
            ffi_loader,
            click_handlers: IndexMap::new(),
        }
    }

    pub fn eval_program(&mut self, program: &'a Program<'a>) -> TsnatResult<Value<'a>> {
        let mut last_val = Value::Undefined;
        for stmt in program.stmts.iter() {
            match self.exec_stmt(stmt)? {
                ControlFlow::Normal(val) => last_val = val,
                ControlFlow::Return(val) => return Ok(val),
                ControlFlow::Break | ControlFlow::Continue => {
                    return Err(TsnatError::Runtime {
                        message: "Break/Continue outside of loop".into(),
                        span: Some(stmt.span()),
                    });
                }
            }
        }
        Ok(last_val)
    }

    pub fn exec_stmt(&mut self, stmt: &'a Stmt<'a>) -> TsnatResult<ControlFlow<'a>> {
        match stmt {
            Stmt::Expr(expr_stmt) => {
                let val = self.eval_expr(expr_stmt.expr)?;
                Ok(ControlFlow::Normal(val))
            }
            Stmt::Var(var_decl) => {
                for decl in var_decl.decls.iter() {
                    let val = if let Some(init) = decl.init {
                        self.eval_expr(init)?
                    } else {
                        Value::Undefined
                    };
                    self.env.borrow_mut().define(decl.name, val);
                }
                Ok(ControlFlow::Normal(Value::Undefined))
            }
            Stmt::Block(block) => {
                let previous_env = self.env.clone();
                self.env = Rc::new(RefCell::new(Environment::new(Some(previous_env.clone()))));
                
                let mut result = ControlFlow::Normal(Value::Undefined);
                for s in block.stmts.iter() {
                    result = self.exec_stmt(s)?;
                    if !matches!(result, ControlFlow::Normal(_)) {
                        break;
                    }
                }
                
                self.env = previous_env;
                Ok(result)
            }
            Stmt::If(if_stmt) => {
                let test = self.eval_expr(if_stmt.test)?;
                if test.is_truthy() {
                    self.exec_stmt(if_stmt.consequent)
                } else if let Some(alt) = if_stmt.alternate {
                    self.exec_stmt(alt)
                } else {
                    Ok(ControlFlow::Normal(Value::Undefined))
                }
            }
            Stmt::While(while_stmt) => {
                let mut last_val = Value::Undefined;
                while self.eval_expr(while_stmt.test)?.is_truthy() {
                    match self.exec_stmt(while_stmt.body)? {
                        ControlFlow::Normal(val) => last_val = val,
                        ControlFlow::Return(val) => return Ok(ControlFlow::Return(val)),
                        ControlFlow::Break => break,
                        ControlFlow::Continue => continue,
                    }
                }
                Ok(ControlFlow::Normal(last_val))
            }
            Stmt::Return(ret) => {
                let val = if let Some(arg) = ret.value {
                    self.eval_expr(arg)?
                } else {
                    Value::Undefined
                };
                Ok(ControlFlow::Return(val))
            }
            Stmt::Function(decl) => {
                let name = decl.id.expect("Function must have a name in declaration");
                let func = Rc::new(JsFunction {
                    name: Some(name),
                    params: decl.params,
                    body: decl.body.map(|b| b.stmts).unwrap_or(&[]),
                    closure: self.env.clone(),
                });
                self.env.borrow_mut().define(name, Value::Function(func));
                Ok(ControlFlow::Normal(Value::Undefined))
            }
            Stmt::NativeImport(i) => {
                let name = self.interner.get(i.name).to_string();
                let source = self.interner.get(i.source);
                let mut path = std::path::PathBuf::from(source);
                if path.extension().is_none() {
                    path.set_extension(std::env::consts::DLL_EXTENSION);
                }
                
                self.ffi_loader.load_library(&name, path).map_err(|e| TsnatError::Runtime {
                    message: format!("FFI Load Error: {}", e),
                    span: Some(i.span),
                })?;
                
                Ok(ControlFlow::Normal(Value::Undefined))
            }
            Stmt::NativeFunction(f) => {
                let func_name = self.interner.get(f.name).to_string();
                let func_name_sym = f.name;
                
                let loader = self.ffi_loader.clone();
                let span = f.span;
                
                let native_fn = Value::NativeFunction(Rc::new(move |args, _| {
                    let ptr = loader.resolve_symbol(&func_name).map_err(|e| TsnatError::Runtime {
                        message: format!("FFI Symbol Error: {}", e),
                        span: Some(span),
                    })?;
                    // Convert args
                    let mut ffi_args = Vec::new();
                    for a in &args {
                        match a {
                            Value::Number(n) => ffi_args.push(tsnat_ffi::invoke::FfiValue::Number(*n)),
                            Value::Bool(b) => ffi_args.push(tsnat_ffi::invoke::FfiValue::Bool(*b)),
                            Value::String(s) => ffi_args.push(tsnat_ffi::invoke::FfiValue::String(s.as_ref())),
                            Value::Null => ffi_args.push(tsnat_ffi::invoke::FfiValue::Null),
                            Value::Undefined => ffi_args.push(tsnat_ffi::invoke::FfiValue::Undefined),
                            _ => return Err(TsnatError::Runtime { message: "Unsupported FFI argument type".into(), span: Some(span) }),
                        }
                    }
                    
                    let res = tsnat_ffi::invoke::invoke_native(ptr, &ffi_args).map_err(|e| {
                        TsnatError::Runtime { message: format!("FFI Call Error: {}", e), span: Some(span) }
                    })?;
                    
                    match res {
                        tsnat_ffi::invoke::FfiValue::Number(n) => Ok(Value::Number(n)),
                        tsnat_ffi::invoke::FfiValue::Bool(b) => Ok(Value::Bool(b)),
                        _ => Ok(Value::Undefined),
                    }
                }));
                self.env.borrow_mut().define(func_name_sym, native_fn);
                Ok(ControlFlow::Normal(Value::Undefined))
            }
            _ => Err(TsnatError::Runtime {
                message: format!("Unimplemented statement type: {:?}", stmt),
                span: Some(stmt.span()),
            }),
        }
    }

    pub fn eval_expr(&mut self, expr: &'a Expr<'a>) -> TsnatResult<Value<'a>> {
        match expr {
            Expr::Number(n, _) => Ok(Value::Number(*n)),
            Expr::String(s, _) => Ok(Value::String(Rc::from(self.interner.get(*s)))),
            Expr::Bool(b, _) => Ok(Value::Bool(*b)),
            Expr::Null(_) => Ok(Value::Null),
            Expr::Undefined(_) => Ok(Value::Undefined),
            Expr::Ident(sym, span) => {
                self.env.borrow().get(*sym).ok_or_else(|| TsnatError::Runtime {
                    message: format!("ReferenceError: {} is not defined", self.interner.get(*sym)),
                    span: Some(*span),
                })
            }
            Expr::Binary(binary) => {
                if binary.op == tsnat_parse::ast::BinaryOp::And {
                    let left = self.eval_expr(binary.left)?;
                    if !left.is_truthy() { return Ok(left); }
                    return self.eval_expr(binary.right);
                } else if binary.op == tsnat_parse::ast::BinaryOp::Or {
                    let left = self.eval_expr(binary.left)?;
                    if left.is_truthy() { return Ok(left); }
                    return self.eval_expr(binary.right);
                }
                
                let left = self.eval_expr(binary.left)?;
                let right = self.eval_expr(binary.right)?;
                self.eval_binary_op(left, binary.op, right, binary.span)
            }
            Expr::Unary(unary) => {
                let val = self.eval_expr(unary.operand)?;
                self.eval_unary_op(unary.op, val, unary.span)
            }
            Expr::Assign(assign) => {
                let val = self.eval_expr(assign.right)?;
                match assign.left {
                    Expr::Ident(sym, span) => {
                        if self.env.borrow_mut().assign(*sym, val.clone()) {
                            Ok(val)
                        } else {
                            Err(TsnatError::Runtime {
                                message: format!("ReferenceError: {} is not defined", self.interner.get(*sym)),
                                span: Some(*span),
                            })
                        }
                    }
                    _ => Err(TsnatError::Runtime {
                        message: "Invalid left-hand side in assignment".into(),
                        span: Some(assign.span),
                    }),
                }
            }
            Expr::Call(call) => {
                if let Expr::Ident(sym, _) = call.callee {
                    if self.interner.get(*sym) == "__tsnat_addEventListener" {
                        if call.args.len() == 2 {
                            let id_val = self.eval_expr(call.args[0])?;
                            let func_val = self.eval_expr(call.args[1])?;
                            if let Value::Number(n) = id_val {
                                self.click_handlers.insert(n as u32, func_val);
                            }
                        }
                        return Ok(Value::Undefined);
                    }
                }
                
                let callee = self.eval_expr(call.callee)?;
                let mut args = Vec::new();
                for arg in call.args.iter() {
                    args.push(self.eval_expr(arg)?);
                }
                self.call_function(callee, args, call.span)
            }
            Expr::Function(decl) => {
                let func = Rc::new(JsFunction {
                    name: decl.id,
                    params: decl.params,
                    body: decl.body.map(|b| b.stmts).unwrap_or(&[]),
                    closure: self.env.clone(),
                });
                Ok(Value::Function(func))
            }
            Expr::Arrow(arrow) => {
                let func = Rc::new(JsFunction {
                    name: None,
                    params: arrow.params,
                    body: match arrow.body {
                        ArrowBody::Block(ref b) => b.stmts,
                        ArrowBody::Expr(_) => return Err(TsnatError::Runtime {
                            message: "Arrow expression body unimplemented".into(),
                            span: Some(expr.span()),
                        }),
                    },
                    closure: self.env.clone(),
                });
                Ok(Value::Function(func))
            }
            Expr::Object(obj) => {
                let mut properties = IndexMap::default();
                for prop in obj.properties.iter() {
                    properties.insert(prop.key, self.eval_expr(prop.value)?);
                }
                Ok(Value::Object(Rc::new(RefCell::new(JsObject {
                    properties,
                    prototype: None,
                }))))
            }
            Expr::Member(member) => {
                let obj = self.eval_expr(member.object)?;
                let key = member.property;
                self.get_property(obj, key, member.span)
            }
            Expr::Index(index) => {
                let obj = self.eval_expr(index.object)?;
                let key_val = self.eval_expr(index.index)?;
                let key = match key_val {
                    Value::String(s) => self.interner.intern(&s),
                    Value::Number(n) => self.interner.intern(&n.to_string()),
                    _ => return Err(TsnatError::Runtime {
                        message: "Invalid index type".into(),
                        span: Some(index.span),
                    }),
                };
                self.get_property(obj, key, index.span)
            }
            Expr::Paren(inner, _) => self.eval_expr(inner),
            _ => Err(TsnatError::Runtime {
                message: format!("Unimplemented expression type: {:?}", expr),
                span: Some(expr.span()),
            }),
        }
    }

    fn eval_binary_op(&self, left: Value<'a>, op: BinaryOp, right: Value<'a>, span: Span) -> TsnatResult<Value<'a>> {
        use BinaryOp::*;
        match (left, op, right) {
            (Value::Number(l), Add, Value::Number(r)) => Ok(Value::Number(l + r)),
            (Value::Number(l), Sub, Value::Number(r)) => Ok(Value::Number(l - r)),
            (Value::Number(l), Mul, Value::Number(r)) => Ok(Value::Number(l * r)),
            (Value::Number(l), Div, Value::Number(r)) => Ok(Value::Number(l / r)),
            (Value::Number(l), Lt, Value::Number(r)) => Ok(Value::Bool(l < r)),
            (Value::Number(l), Gt, Value::Number(r)) => Ok(Value::Bool(l > r)),
            (Value::Number(l), EqEqEq, Value::Number(r)) => Ok(Value::Bool(l == r)),
            (Value::Number(l), BangEqEq, Value::Number(r)) => Ok(Value::Bool(l != r)),
            (Value::String(l), EqEqEq, Value::String(r)) => Ok(Value::Bool(l == r)),
            (Value::String(l), BangEqEq, Value::String(r)) => Ok(Value::Bool(l != r)),
            (Value::Undefined, EqEqEq, Value::Undefined) => Ok(Value::Bool(true)),
            (l, BangEqEq, Value::Undefined) => Ok(Value::Bool(!matches!(l, Value::Undefined))),
            (Value::String(l), Add, Value::String(r)) => Ok(Value::String(Rc::from(format!("{}{}", l, r)))),
            (l, op, r) => Err(TsnatError::Runtime {
                message: format!("Unimplemented binary operations: {:?} {:?} {:?}", l, op, r),
                span: Some(span),
            }),
        }
    }

    pub fn call_function(&mut self, callee: Value<'a>, args: Vec<Value<'a>>, span: Span) -> TsnatResult<Value<'a>> {
        match callee {
            Value::Function(func) => {
                let previous_env = self.env.clone();
                let call_env = Rc::new(RefCell::new(Environment::new(Some(func.closure.clone()))));
                
                for (i, param) in func.params.iter().enumerate() {
                    let val = args.get(i).cloned().unwrap_or(Value::Undefined);
                    call_env.borrow_mut().define(param.name, val);
                }
                
                self.env = call_env;
                let mut result = Value::Undefined;
                for stmt in func.body.iter() {
                    match self.exec_stmt(stmt)? {
                        ControlFlow::Normal(val) => result = val,
                        ControlFlow::Return(val) => {
                            result = val;
                            break;
                        }
                        ControlFlow::Break | ControlFlow::Continue => {
                            return Err(TsnatError::Runtime {
                                message: "Illegal break/continue in function".into(),
                                span: Some(stmt.span()),
                            });
                        }
                    }
                }
                
                self.env = previous_env;
                Ok(result)
            }
            Value::NativeFunction(func) => {
                func(args, None)
            }
            _ => Err(TsnatError::Runtime {
                message: "TypeError: callee is not a function".into(),
                span: Some(span),
            }),
        }
    }

    fn get_property(&self, obj: Value<'a>, key: Symbol, span: Span) -> TsnatResult<Value<'a>> {
        match obj {
            Value::Object(o) => {
                if let Some(val) = o.borrow().properties.get(&key) {
                    Ok(val.clone())
                } else if let Some(proto) = &o.borrow().prototype {
                    self.get_property(Value::Object(proto.clone()), key, span)
                } else {
                    Ok(Value::Undefined)
                }
            }
            _ => Err(TsnatError::Runtime {
                message: format!("TypeError: Cannot read property '{}' of {:?}", self.interner.get(key), obj),
                span: Some(span),
            }),
        }
    }

    fn eval_unary_op(&self, op: UnaryOp, val: Value<'a>, span: Span) -> TsnatResult<Value<'a>> {
        use UnaryOp::*;
        match (op, val) {
            (Not, v) => Ok(Value::Bool(!v.is_truthy())),
            (Neg, Value::Number(n)) => Ok(Value::Number(-n)),
            (Typeof, v) => {
                let s = match v {
                    Value::Undefined => "undefined",
                    Value::Null => "object",
                    Value::Bool(_) => "boolean",
                    Value::Number(_) | Value::BigInt(_) => "number",
                    Value::String(_) => "string",
                    Value::Symbol(_) => "symbol",
                    Value::Object(_) => "object",
                    Value::Function(_) | Value::NativeFunction(_) => "function",
                };
                Ok(Value::String(Rc::from(s)))
            }
            (op, v) => Err(TsnatError::Runtime {
                message: format!("Unimplemented unary operator: {:?} {:?}", op, v),
                span: Some(span),
            }),
        }
    }
}
