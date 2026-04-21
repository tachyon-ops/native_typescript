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
    pub click_handlers: Rc<RefCell<IndexMap<u32, Value<'a>>>>,
    pub source: Option<String>,
    pub arena: &'a bumpalo::Bump,
}

impl<'a, 'src> Evaluator<'a, 'src> {
    pub fn new(interner: &'src mut Interner, arena: &'a bumpalo::Bump) -> Self {
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

        global_env.borrow_mut().define(interner.intern("NaN"), Value::Number(f64::NAN));
        global_env.borrow_mut().define(interner.intern("Infinity"), Value::Number(f64::INFINITY));
        
        Evaluator {
            env: global_env,
            interner,
            ffi_loader,
            click_handlers: Rc::new(RefCell::new(IndexMap::default())),
            source: None,
            arena,
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
            Stmt::For(for_stmt) => {
                let previous_env = self.env.clone();
                self.env = Rc::new(RefCell::new(Environment::new(Some(previous_env.clone()))));
                
                if let Some(init) = &for_stmt.init {
                    match init {
                        tsnat_parse::ast::ForInit::Var(var_decl) => {
                            for v in var_decl.decls.iter() {
                                let val = if let Some(init) = v.init {
                                    self.eval_expr(init)?
                                } else {
                                    Value::Undefined
                                };
                                self.env.borrow_mut().define(v.name, val);
                            }
                        }
                        tsnat_parse::ast::ForInit::Expr(expr) => {
                            self.eval_expr(*expr)?;
                        }
                    }
                }
                
                let mut last_val = Value::Undefined;
                loop {
                    if let Some(test) = for_stmt.test {
                        if !self.eval_expr(test)?.is_truthy() {
                            break;
                        }
                    }
                    
                    match self.exec_stmt(for_stmt.body)? {
                        ControlFlow::Normal(val) => last_val = val,
                        ControlFlow::Return(val) => {
                            self.env = previous_env;
                            return Ok(ControlFlow::Return(val));
                        }
                        ControlFlow::Break => break,
                        ControlFlow::Continue => {}
                    }
                    
                    if let Some(update) = for_stmt.update {
                        self.eval_expr(update)?;
                    }
                }
                
                self.env = previous_env;
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
                let func_name = self.interner.get(decl.id.unwrap()).to_string();
                let func_name_sym = decl.id.unwrap();
                let mut props = IndexMap::default();
                let proto_obj = Value::Object(Rc::new(RefCell::new(crate::value::JsObject {
                    properties: IndexMap::default(),
                    prototype: None,
                })));
                props.insert(self.interner.intern("prototype"), proto_obj);
                let func = Rc::new(JsFunction {
                    name: decl.id,
                    params: decl.params,
                    body: crate::value::FuncBody::Stmts(decl.body.map(|b| b.stmts).unwrap_or(&[])),
                    closure: self.env.clone(),
                    properties: RefCell::new(props),
                });
                self.env.borrow_mut().define(func_name_sym, Value::Function(func));
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
            Stmt::Import(import_decl) => {
                let source_path = self.interner.get(import_decl.source).to_string();
                let mut path = std::path::PathBuf::from(&source_path);
                if path.extension().is_none() {
                    path.set_extension("ts");
                }
                
                let content = std::fs::read_to_string(&path).map_err(|e| TsnatError::Runtime {
                    message: format!("Failed to read imported file {:?}: {}", path, e),
                    span: Some(import_decl.span),
                })?;
                
                let mut lexer = tsnat_lex::lexer::Lexer::new(&content, 0, self.interner);
                let tokens = lexer.tokenise_all().map_err(|e| TsnatError::Runtime {
                    message: format!("Import Lexer Error: {:?}", e),
                    span: Some(import_decl.span),
                })?;
                
                let mut parser = tsnat_parse::parser::Parser::new(&tokens, self.arena, self.interner);
                let program = parser.parse_program().map_err(|e| TsnatError::Runtime {
                    message: format!("Import Parser Error: {:?}", e),
                    span: Some(import_decl.span),
                })?;
                
                let module_env = Rc::new(RefCell::new(Environment::new(Some(self.env.clone()))));
                let previous_env = self.env.clone();
                self.env = module_env.clone();
                
                let previous_source = self.source.take();
                self.source = Some(content);
                
                for inner_stmt in program.stmts.iter() {
                    self.exec_stmt(inner_stmt)?;
                }
                
                self.source = previous_source;
                self.env = previous_env;
                
                for specifier in import_decl.specifiers.iter() {
                    match specifier {
                        tsnat_parse::ast::ImportSpecifier::Named(local, imported_opt) => {
                            let imported = imported_opt.unwrap_or(*local);
                            if let Some(val) = module_env.borrow().get(imported) {
                                self.env.borrow_mut().define(*local, val);
                            } else {
                                return Err(TsnatError::Runtime {
                                    message: format!("Module does not export '{}'", self.interner.get(imported)),
                                    span: Some(import_decl.span),
                                });
                            }
                        }
                        _ => return Err(TsnatError::Runtime {
                            message: "Unsupported import specifier".into(),
                            span: Some(import_decl.span),
                        }),
                    }
                }
                
                Ok(ControlFlow::Normal(Value::Undefined))
            }
            Stmt::Export(export_decl) => {
                if let Some(decl) = export_decl.decl {
                    self.exec_stmt(decl)?;
                }
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
            Expr::This(span) => {
                let sym = self.interner.intern("this");
                self.env.borrow().get(sym).ok_or_else(|| TsnatError::Runtime {
                    message: "this is undefined".into(),
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
                use tsnat_parse::ast::UnaryOp::*;
                if matches!(unary.op, PreInc | PreDec | PostInc | PostDec) {
                    let old_val = self.eval_expr(unary.operand)?;
                    let old_num = match old_val {
                        Value::Number(n) => n,
                        _ => return Err(TsnatError::Runtime { message: "Invalid operand for increment/decrement".into(), span: Some(unary.span) }),
                    };
                    
                    let new_num = match unary.op {
                        PreInc | PostInc => old_num + 1.0,
                        PreDec | PostDec => old_num - 1.0,
                        _ => unreachable!(),
                    };
                    let new_val = Value::Number(new_num);
                    
                    match unary.operand {
                        Expr::Ident(sym, span) => {
                            if !self.env.borrow_mut().assign(*sym, new_val.clone()) {
                                return Err(TsnatError::Runtime {
                                    message: format!("ReferenceError: {} is not defined", self.interner.get(*sym)),
                                    span: Some(*span),
                                });
                            }
                        }
                        Expr::Member(member) => {
                            let obj = self.eval_expr(member.object)?;
                            let key = member.property;
                            match obj {
                                Value::Object(obj_rc) => { obj_rc.borrow_mut().properties.insert(key, new_val.clone()); }
                                Value::Function(func) => { func.properties.borrow_mut().insert(key, new_val.clone()); }
                                _ => return Err(TsnatError::Runtime { message: "Cannot assign property on non-object".into(), span: Some(unary.span) }),
                            }
                        }
                        Expr::Index(index) => {
                            let obj = self.eval_expr(index.object)?;
                            let key_val = self.eval_expr(index.index)?;
                            let key = match key_val {
                                Value::String(s) => self.interner.intern(&s),
                                Value::Number(n) => self.interner.intern(&n.to_string()),
                                _ => return Err(TsnatError::Runtime { message: "Invalid index type".into(), span: Some(unary.span) }),
                            };
                            match obj {
                                Value::Object(obj_rc) => { obj_rc.borrow_mut().properties.insert(key, new_val.clone()); }
                                Value::Function(func) => { func.properties.borrow_mut().insert(key, new_val.clone()); }
                                _ => return Err(TsnatError::Runtime { message: "Cannot assign property on non-object".into(), span: Some(unary.span) }),
                            }
                        }
                        _ => return Err(TsnatError::Runtime { message: "Invalid left-hand side".into(), span: Some(unary.span) }),
                    }
                    
                    match unary.op {
                        PreInc | PreDec => Ok(new_val),
                        PostInc | PostDec => Ok(Value::Number(old_num)),
                        _ => unreachable!(),
                    }
                } else {
                    let val = self.eval_expr(unary.operand)?;
                    self.eval_unary_op(unary.op, val, unary.span)
                }
            }
            Expr::Assign(assign) => {
                let right_val = self.eval_expr(assign.right)?;
                
                let val = if assign.op != tsnat_parse::ast::AssignOp::Eq {
                    let left_val = self.eval_expr(assign.left)?;
                    let bin_op = match assign.op {
                        tsnat_parse::ast::AssignOp::AddEq => tsnat_parse::ast::BinaryOp::Add,
                        tsnat_parse::ast::AssignOp::SubEq => tsnat_parse::ast::BinaryOp::Sub,
                        tsnat_parse::ast::AssignOp::MulEq => tsnat_parse::ast::BinaryOp::Mul,
                        tsnat_parse::ast::AssignOp::DivEq => tsnat_parse::ast::BinaryOp::Div,
                        tsnat_parse::ast::AssignOp::ModEq => tsnat_parse::ast::BinaryOp::Mod,
                        _ => return Err(TsnatError::Runtime { message: "Unimplemented compound assignment operator".into(), span: Some(assign.span) }),
                    };
                    self.eval_binary_op(left_val, bin_op, right_val, assign.span)?
                } else {
                    right_val
                };
                
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
                    Expr::Member(member) => {
                        let obj = self.eval_expr(member.object)?;
                        let key = member.property;
                        match obj {
                            Value::Object(obj_rc) => {
                                obj_rc.borrow_mut().properties.insert(key, val.clone());
                                Ok(val)
                            }
                            Value::Function(func) => {
                                func.properties.borrow_mut().insert(key, val.clone());
                                Ok(val)
                            }
                            _ => Err(TsnatError::Runtime {
                                message: "Cannot assign property on non-object".into(),
                                span: Some(assign.span),
                            })
                        }
                    }
                    Expr::Index(index) => {
                        let obj = self.eval_expr(index.object)?;
                        let key_val = self.eval_expr(index.index)?;
                        let key = match key_val {
                            Value::String(s) => self.interner.intern(&s),
                            Value::Number(n) => self.interner.intern(&n.to_string()),
                            _ => return Err(TsnatError::Runtime {
                                message: "Invalid index type for assignment".into(),
                                span: Some(index.span),
                            }),
                        };
                        match obj {
                            Value::Object(obj_rc) => {
                                obj_rc.borrow_mut().properties.insert(key, val.clone());
                                Ok(val)
                            }
                            Value::Function(func) => {
                                func.properties.borrow_mut().insert(key, val.clone());
                                Ok(val)
                            }
                            _ => Err(TsnatError::Runtime {
                                message: "Cannot assign property on non-object".into(),
                                span: Some(assign.span),
                            })
                        }
                    }
                    _ => Err(TsnatError::Runtime {
                        message: "Invalid left-hand side in assignment".into(),
                        span: Some(assign.span),
                    }),
                }
            }
            Expr::New(new_expr) => {
                let callee = self.eval_expr(new_expr.callee)?;
                
                let prototype = match &callee {
                    Value::Function(func) => {
                        if let Some(Value::Object(proto)) = func.properties.borrow().get(&self.interner.intern("prototype")) {
                            Some(proto.clone())
                        } else { None }
                    }
                    _ => None,
                };
                
                let obj = Value::Object(Rc::new(RefCell::new(crate::value::JsObject {
                    properties: IndexMap::default(),
                    prototype,
                })));
                
                let mut args = Vec::new();
                for arg in new_expr.args.iter() {
                    args.push(self.eval_expr(arg)?);
                }
                
                self.call_function(callee, args, Some(obj.clone()), new_expr.span)?;
                Ok(obj)
            }
            Expr::Call(call) => {
                if let Expr::Ident(sym, _) = call.callee {
                    if self.interner.get(*sym) == "__tsnat_addEventListener" {
                        if call.args.len() == 2 {
                            let id_val = self.eval_expr(call.args[0])?;
                            let func_val = self.eval_expr(call.args[1])?;
                            if let Value::Number(n) = id_val {
                                self.click_handlers.borrow_mut().insert(n as u32, func_val);
                            }
                        }
                        return Ok(Value::Undefined);
                    }
                }
                let (callee, this_val) = match call.callee {
                    Expr::Member(member) => {
                        let obj = self.eval_expr(member.object)?;
                        let callee = self.get_property(obj.clone(), member.property, member.span)?;
                        (callee, Some(obj))
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
                        let callee = self.get_property(obj.clone(), key, index.span)?;
                        (callee, Some(obj))
                    }
                    _ => (self.eval_expr(call.callee)?, None),
                };
                let mut args = Vec::new();
                for arg in call.args.iter() {
                    args.push(self.eval_expr(arg)?);
                }
                self.call_function(callee, args, this_val, call.span)
            }
            Expr::Function(decl) => {
                let mut props = IndexMap::default();
                let proto_obj = Value::Object(Rc::new(RefCell::new(crate::value::JsObject {
                    properties: IndexMap::default(),
                    prototype: None,
                })));
                props.insert(self.interner.intern("prototype"), proto_obj);
                let func = Rc::new(JsFunction {
                    name: decl.id,
                    params: decl.params,
                    body: crate::value::FuncBody::Stmts(decl.body.map(|b| b.stmts).unwrap_or(&[])),
                    closure: self.env.clone(),
                    properties: RefCell::new(props),
                });
                Ok(Value::Function(func))
            }
            Expr::Arrow(arrow) => {
                let mut props = IndexMap::default();
                let proto_obj = Value::Object(Rc::new(RefCell::new(crate::value::JsObject {
                    properties: IndexMap::default(),
                    prototype: None,
                })));
                props.insert(self.interner.intern("prototype"), proto_obj);
                let func = Rc::new(JsFunction {
                    name: None,
                    params: arrow.params,
                    body: match arrow.body {
                        ArrowBody::Block(ref b) => crate::value::FuncBody::Stmts(b.stmts),
                        ArrowBody::Expr(e) => crate::value::FuncBody::Expr(e),
                    },
                    closure: self.env.clone(),
                    properties: RefCell::new(props),
                });
                Ok(Value::Function(func))
            }
            Expr::Array(arr) => {
                let mut properties = IndexMap::default();
                let mut length = 0;
                for el in arr.elements.iter() {
                    if let Expr::Spread(spread) = el {
                        let val = self.eval_expr(spread.argument)?;
                        if let Value::Object(obj) = val {
                            let props = obj.borrow().properties.clone();
                            let len_val = props.get(&self.interner.intern("length")).cloned().unwrap_or(Value::Undefined);
                            if let Value::Number(len) = len_val {
                                let len = len as usize;
                                for i in 0..len {
                                    let key = self.interner.intern(&i.to_string());
                                    let el_val = props.get(&key).cloned().unwrap_or(Value::Undefined);
                                    let new_key = self.interner.intern(&length.to_string());
                                    properties.insert(new_key, el_val);
                                    length += 1;
                                }
                            }
                        }
                    } else {
                        let key = self.interner.intern(&length.to_string());
                        let val = self.eval_expr(el)?;
                        properties.insert(key, val);
                        length += 1;
                    }
                }
                let len_key = self.interner.intern("length");
                properties.insert(len_key, Value::Number(length as f64));
                
                let array_sym = self.interner.intern("Array");
                let prototype = match self.env.borrow().get(array_sym) {
                    Some(Value::Object(arr_obj)) => {
                        if let Some(Value::Object(proto)) = arr_obj.borrow().properties.get(&self.interner.intern("prototype")) {
                            Some(proto.clone())
                        } else { None }
                    }
                    Some(Value::Function(func)) => {
                        if let Some(Value::Object(proto)) = func.properties.borrow().get(&self.interner.intern("prototype")) {
                            Some(proto.clone())
                        } else { None }
                    }
                    _ => None,
                };
                
                Ok(Value::Object(Rc::new(RefCell::new(JsObject {
                    properties,
                    prototype,
                }))))
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
            Expr::JSXElement(jsx) => {
                let react_sym = self.interner.intern("React");
                let react_val = self.env.borrow().get(react_sym).ok_or_else(|| TsnatError::Runtime {
                    message: "React is not defined".into(),
                    span: Some(jsx.span),
                })?;
                
                let create_elem_sym = self.interner.intern("createElement");
                let create_elem = self.get_property(react_val.clone(), create_elem_sym, jsx.span)?;
                
                let tag_str = self.interner.get(jsx.tag).to_string();
                let tag_val = Value::String(Rc::from(tag_str));
                
                let mut props_map = IndexMap::default();
                for prop in jsx.props.iter() {
                    let val = self.eval_expr(prop.value)?;
                    props_map.insert(prop.key, val);
                }
                
                let props_val = if props_map.is_empty() {
                    Value::Null
                } else {
                    Value::Object(Rc::new(RefCell::new(JsObject {
                        properties: props_map,
                        prototype: None,
                    })))
                };
                
                let mut children_map = IndexMap::default();
                let mut child_idx = 0;
                for child in jsx.children.iter() {
                    let child_val = self.eval_expr(child)?;
                    // Filter out empty text nodes
                    if let Value::String(ref s) = child_val {
                        if s.trim().is_empty() {
                            continue;
                        }
                    }
                    
                    let key = self.interner.intern(&child_idx.to_string());
                    children_map.insert(key, child_val);
                    child_idx += 1;
                }
                
                let children_val = Value::Object(Rc::new(RefCell::new(JsObject {
                    properties: children_map,
                    prototype: None,
                })));
                
                let args = vec![tag_val, props_val, children_val];
                self.call_function(create_elem, args, Some(react_val), jsx.span)
            }
            Expr::JSXText(_, span) => {
                if let Some(src) = &self.source {
                    let text = &src[span.start as usize..span.end as usize];
                    Ok(Value::String(Rc::from(text)))
                } else {
                    Ok(Value::String(Rc::from("")))
                }
            }
            Expr::JSXExpressionContainer(inner, _) => self.eval_expr(inner),
            Expr::As(as_expr) => self.eval_expr(as_expr.expr),
            Expr::Template(template) => {
                let mut res = String::new();
                for (i, quasi) in template.quasis.iter().enumerate() {
                    res.push_str(self.interner.get(*quasi));
                    if i < template.exprs.len() {
                        let val = self.eval_expr(template.exprs[i])?;
                        match val {
                            Value::String(s) => res.push_str(&s),
                            Value::Number(n) => res.push_str(&n.to_string()),
                            Value::Bool(b) => res.push_str(&b.to_string()),
                            Value::Null => res.push_str("null"),
                            Value::Undefined => res.push_str("undefined"),
                            _ => res.push_str("[object Object]"),
                        }
                    }
                }
                Ok(Value::String(Rc::from(res)))
            }
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
            (Value::Number(l), Mod, Value::Number(r)) => Ok(Value::Number(l % r)),
            (Value::Number(l), Lt, Value::Number(r)) => Ok(Value::Bool(l < r)),
            (Value::Number(l), Gt, Value::Number(r)) => Ok(Value::Bool(l > r)),
            (Value::Number(l), LtEq, Value::Number(r)) => Ok(Value::Bool(l <= r)),
            (Value::Number(l), GtEq, Value::Number(r)) => Ok(Value::Bool(l >= r)),
            (Value::Number(l), EqEqEq, Value::Number(r)) => Ok(Value::Bool(l == r)),
            (Value::Number(l), BangEqEq, Value::Number(r)) => Ok(Value::Bool(l != r)),
            (Value::String(l), Lt, Value::String(r)) => Ok(Value::Bool(l < r)),
            (Value::String(l), Gt, Value::String(r)) => Ok(Value::Bool(l > r)),
            (Value::String(l), LtEq, Value::String(r)) => Ok(Value::Bool(l <= r)),
            (Value::String(l), GtEq, Value::String(r)) => Ok(Value::Bool(l >= r)),
            (Value::String(l), EqEqEq, Value::String(r)) => Ok(Value::Bool(l == r)),
            (Value::String(l), BangEqEq, Value::String(r)) => Ok(Value::Bool(l != r)),
            (Value::Undefined, EqEqEq, Value::Undefined) => Ok(Value::Bool(true)),
            (l, EqEqEq, Value::Undefined) => Ok(Value::Bool(matches!(l, Value::Undefined))),
            (Value::Null, EqEqEq, Value::Null) => Ok(Value::Bool(true)),
            (l, EqEqEq, Value::Null) => Ok(Value::Bool(matches!(l, Value::Null))),
            (l, BangEqEq, Value::Undefined) => Ok(Value::Bool(!matches!(l, Value::Undefined))),
            (l, BangEqEq, Value::Null) => Ok(Value::Bool(!matches!(l, Value::Null))),
            (Value::Function(l), EqEqEq, Value::Function(r)) => Ok(Value::Bool(Rc::ptr_eq(&l, &r))),
            (Value::NativeFunction(l), EqEqEq, Value::NativeFunction(r)) => Ok(Value::Bool(Rc::ptr_eq(&l, &r))),
            (Value::Function(_), EqEqEq, Value::NativeFunction(_)) => Ok(Value::Bool(false)),
            (Value::NativeFunction(_), EqEqEq, Value::Function(_)) => Ok(Value::Bool(false)),
            (Value::Function(l), BangEqEq, Value::Function(r)) => Ok(Value::Bool(!Rc::ptr_eq(&l, &r))),
            (Value::NativeFunction(l), BangEqEq, Value::NativeFunction(r)) => Ok(Value::Bool(!Rc::ptr_eq(&l, &r))),
            (Value::Function(_), BangEqEq, Value::NativeFunction(_)) => Ok(Value::Bool(true)),
            (Value::NativeFunction(_), BangEqEq, Value::Function(_)) => Ok(Value::Bool(true)),
            (Value::String(l), Add, Value::String(r)) => Ok(Value::String(Rc::from(format!("{}{}", l, r)))),
            (Value::String(l), Add, r) => {
                let r_str = match r {
                    Value::String(s) => s.to_string(),
                    Value::Number(n) => n.to_string(),
                    Value::Bool(b) => b.to_string(),
                    Value::Null => "null".to_string(),
                    Value::Undefined => "undefined".to_string(),
                    _ => "[object Object]".to_string(),
                };
                Ok(Value::String(Rc::from(format!("{}{}", l, r_str))))
            }
            (l, Add, Value::String(r)) => {
                let l_str = match l {
                    Value::String(s) => s.to_string(),
                    Value::Number(n) => n.to_string(),
                    Value::Bool(b) => b.to_string(),
                    Value::Null => "null".to_string(),
                    Value::Undefined => "undefined".to_string(),
                    _ => "[object Object]".to_string(),
                };
                Ok(Value::String(Rc::from(format!("{}{}", l_str, r))))
            }
            (l, op, r) => Err(TsnatError::Runtime {
                message: format!("Unimplemented binary operations: {:?} {:?} {:?}", l, op, r),
                span: Some(span),
            }),
        }
    }

    pub fn call_function(&mut self, callee: Value<'a>, args: Vec<Value<'a>>, this_val: Option<Value<'a>>, span: Span) -> TsnatResult<Value<'a>> {
        match callee {
            Value::Function(func) => {
                let previous_env = self.env.clone();
                let call_env = Rc::new(RefCell::new(Environment::new(Some(func.closure.clone()))));
                
                if let Some(tv) = this_val {
                    call_env.borrow_mut().define(self.interner.intern("this"), tv);
                }

                for (i, param) in func.params.iter().enumerate() {
                    if param.is_rest {
                        let mut properties = IndexMap::default();
                        let mut length = 0;
                        for arg_val in args.iter().skip(i) {
                            let key = self.interner.intern(&length.to_string());
                            properties.insert(key, arg_val.clone());
                            length += 1;
                        }
                        properties.insert(self.interner.intern("length"), Value::Number(length as f64));
                        
                        let array_sym = self.interner.intern("Array");
                        let prototype = match self.env.borrow().get(array_sym) {
                            Some(Value::Object(arr_obj)) => {
                                if let Some(Value::Object(proto)) = arr_obj.borrow().properties.get(&self.interner.intern("prototype")) {
                                    Some(proto.clone())
                                } else { None }
                            }
                            Some(Value::Function(func_obj)) => {
                                if let Some(Value::Object(proto)) = func_obj.properties.borrow().get(&self.interner.intern("prototype")) {
                                    Some(proto.clone())
                                } else { None }
                            }
                            _ => None,
                        };
                        
                        let rest_array = Value::Object(Rc::new(RefCell::new(crate::value::JsObject {
                            properties,
                            prototype,
                        })));
                        call_env.borrow_mut().define(param.name, rest_array);
                    } else {
                        let val = args.get(i).cloned().unwrap_or(Value::Undefined);
                        call_env.borrow_mut().define(param.name, val);
                    }
                }
                
                self.env = call_env;
                let mut result = Value::Undefined;
                match func.body {
                    crate::value::FuncBody::Stmts(stmts) => {
                        for stmt in stmts.iter() {
                            match self.exec_stmt(stmt)? {
                                ControlFlow::Normal(val) => result = val,
                                ControlFlow::Return(val) => {
                                    result = val;
                                    break;
                                }
                                ControlFlow::Break | ControlFlow::Continue => {
                                    return Err(TsnatError::Runtime {
                                        message: "Illegal break/continue in function".into(),
                                        span: Some(span),
                                    });
                                }
                            }
                        }
                    }
                    crate::value::FuncBody::Expr(expr) => {
                        result = self.eval_expr(expr)?;
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
            Value::Function(func) => {
                if let Some(val) = func.properties.borrow().get(&key) {
                    Ok(val.clone())
                } else {
                    Ok(Value::Undefined)
                }
            }
            Value::String(ref s) => {
                if self.interner.get(key) == "length" {
                    Ok(Value::Number(s.chars().count() as f64))
                } else if let Ok(idx) = self.interner.get(key).parse::<usize>() {
                    if let Some(c) = s.chars().nth(idx) {
                        Ok(Value::String(Rc::from(c.to_string())))
                    } else {
                        Ok(Value::Undefined)
                    }
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
