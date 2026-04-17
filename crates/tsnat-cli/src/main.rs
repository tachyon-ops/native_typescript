use bumpalo::Bump;
use std::env;
use std::fs;
use std::rc::Rc;
use std::cell::RefCell;

use tsnat_common::diagnostic::TsnatError;
use tsnat_common::interner::Interner;
use tsnat_lex::lexer::Lexer;
use tsnat_parse::parser::Parser;
use tsnat_eval::eval::Evaluator;
use tsnat_eval::value::Value;
use tsnat_react::render::{Application, IntrinsicTag};

fn run_script<'a>(src: &'a str, eval: &mut Evaluator<'a, 'a>, arena: &'a Bump) -> Result<(), String> {
    let mut lexer = Lexer::new(src, 0, &mut eval.interner);
    let tokens = lexer.tokenise_all().map_err(|e| format!("{:?}", e))?;

    let mut parser = Parser::new(&tokens, arena, &mut eval.interner);
    let ast = parser.parse_program().map_err(|e| format!("{:?}", e))?;

    for stmt in ast.stmts.iter() {
        if let Err(e) = eval.exec_stmt(&stmt) {
            return Err(format!("{:?}", e));
        }
    }
    Ok(())
}

fn main() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();
    let target_file = if args.len() > 1 {
        &args[1]
    } else {
        "crates/tsnat-cli/src/App.ts"
    };

    println!("Starting Tsnat UI Runtime...");
    println!("Loading {}...", target_file);

    let app_src = fs::read_to_string(target_file).expect("Failed to read App.ts");
    let react_src = fs::read_to_string("crates/tsnat-cli/src/react.ts").expect("Failed to read react.ts");

    let app = Rc::new(RefCell::new(
        Application::new("TypeScript Native", 800, 600).map_err(|e| format!("{:?}", e))?
    ));

    let arena = Bump::new();
    let mut interner = Interner::new();
    let mut eval = Evaluator::new(&mut interner);

    // 1. Inject __tsnat_createWidget
    // Signature: (tag: string, text: string?) -> u32
    let app_clone_create = Rc::clone(&app);
    let create_widget = Rc::new(move |args: Vec<Value<'_>>, _this: Option<Value<'_>>| {
        let tag_str = match &args.get(0) {
            Some(Value::String(s)) => s.as_ref(),
            _ => return Err(TsnatError::Runtime { message: "tag must be string".into(), span: None }),
        };
        
        let tag = match tag_str {
            "div" => IntrinsicTag::Div,
            "span" => IntrinsicTag::Span,
            _ => IntrinsicTag::Div,
        };

        let text = match &args.get(1) {
            Some(Value::String(s)) => Some(s.to_string()),
            _ => None,
        };

        let id = app_clone_create.borrow_mut().create_widget(tag, text);
        Ok(Value::Number(id as f64))
    });

    // 2. Inject __tsnat_appendChild
    // Signature: (parent: u32, child: u32) -> void
    let app_clone_append = Rc::clone(&app);
    let append_child = Rc::new(move |args: Vec<Value<'_>>, _this: Option<Value<'_>>| {
        let parent_id = match &args.get(0) {
            Some(Value::Number(n)) => *n as u32,
            _ => return Err(TsnatError::Runtime { message: "parent must be u32".into(), span: None }),
        };
        let child_id = match &args.get(1) {
            Some(Value::Number(n)) => *n as u32,
            _ => return Err(TsnatError::Runtime { message: "child must be u32".into(), span: None }),
        };
        app_clone_append.borrow_mut().append_child(parent_id, child_id);
        Ok(Value::Undefined)
    });

    // 3. Inject __tsnat_setRoot
    let app_clone_root = Rc::clone(&app);
    let set_root = Rc::new(move |args: Vec<Value<'_>>, _this: Option<Value<'_>>| {
        let root_id = match &args.get(0) {
            Some(Value::Number(n)) => *n as u32,
            _ => return Err(TsnatError::Runtime { message: "root must be u32".into(), span: None }),
        };
        app_clone_root.borrow_mut().set_root(root_id);
        Ok(Value::Undefined)
    });

    // Bind to the environment
    {
        let mut env = eval.env.borrow_mut();
        let sym_create = eval.interner.intern("__tsnat_createWidget");
        let sym_append = eval.interner.intern("__tsnat_appendChild");
        let sym_set_root = eval.interner.intern("__tsnat_setRoot");

        env.define(sym_create, Value::NativeFunction(create_widget));
        env.define(sym_append, Value::NativeFunction(append_child));
        env.define(sym_set_root, Value::NativeFunction(set_root));
    }

    // Run the shims & apps
    println!("Evaluating react shim...");
    run_script(&react_src, &mut eval, &arena)?;
    
    println!("Evaluating application...");
    run_script(&app_src, &mut eval, &arena)?;

    println!("Starting render loop...");
    // Main UI Loop!
    loop {
        let mut app_mut = app.borrow_mut();
        if !app_mut.tick() {
            break;
        }
        // Small sleep to not hog CPU 100%
        std::thread::sleep(std::time::Duration::from_millis(16));
    }

    Ok(())
}
