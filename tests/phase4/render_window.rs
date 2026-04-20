use std::rc::Rc;
use bumpalo::Bump;
use tsnat_parse::interner::Interner;
use tsnat_eval::eval::Evaluator;
use tsnat_eval::value::Value;
use tsnat_react::window::NativeEvent;
use tsnat_lex::lexer::Lexer;
use tsnat_parse::parser::Parser;
use tsnat_common::span::SourceMap;

#[test]
fn test_render_window_exit_test() {
    let host_config = r#"
        let _state = {};
        let _index = 0;
        let _rootComponent = null;

        export const React = {
            useState: function(initialValue) {
                const currentIndex = _index;
                if (_state[currentIndex] === undefined) {
                    _state[currentIndex] = initialValue;
                }
                
                const setState = function(newValue) {
                    if (typeof newValue === "function") {
                        _state[currentIndex] = newValue(_state[currentIndex]);
                    } else {
                        _state[currentIndex] = newValue;
                    }
                    _index = 0;
                    if (_rootComponent !== null) {
                        let newTree = _rootComponent();
                        if (typeof newTree === "object" && newTree.id !== undefined) {
                            __tsnat_setRoot(newTree.id);
                        }
                    }
                };
                _index = _index + 1;
                return { v0: _state[currentIndex], v1: setState };
            },

            createElement: function(tag, props, children) {
                let textNode = null;
                if (tag === "span" && typeof children["0"] === "string") {
                    textNode = children["0"];
                } else if (typeof children["0"] === "string") {
                    textNode = children["0"];
                } else if (typeof children["0"] === "number") {
                    textNode = "" + children["0"];
                }

                let id = __tsnat_createWidget(tag, textNode);

                if (props !== null && props.onClick) {
                    __tsnat_addEventListener(id, props.onClick);
                }

                let i = 0;
                while (children[i] !== undefined) {
                    let child = children[i];
                    if (typeof child === "object" && child.id !== undefined) {
                        __tsnat_appendChild(id, child.id);
                    } else if (typeof child === "string" || typeof child === "number") {
                        let textId = __tsnat_createWidget("span", "" + child);
                        __tsnat_appendChild(id, textId);
                    }
                    i = i + 1;
                }
                return { id: id, tag: tag };
            }
        };

        export const renderApp = function(root_fn, config) {
            _rootComponent = root_fn;
            _index = 0;
            let root_element = root_fn();
            if (typeof root_element === "object" && root_element.id !== undefined) {
                __tsnat_setRoot(root_element.id);
            }
        };
    "#;

    let app_src = format!("{}\n{}", host_config, r#"
        function Counter() {
            const stateRes = React.useState(0);
            const n = stateRes.v0;
            const setN = stateRes.v1;
            return (
                <div id="container">
                    <span>{"Count: "}{n}</span>
                    <button onClick={function() { setN(function(c) { return c + 1; }); }}>
                        <span>{"Increment"}</span>
                    </button>
                </div>
            );
        }

        renderApp(Counter, { title: 'Counter', width: 400, height: 300 });
    "#);

    let arena = Bump::new();
    let mut interner = Interner::new();
    
    // Manual parsing & eval must happen before eval borrows interner
    let mut sm = SourceMap::new();
    let file_id = sm.add_file("app.tsx".into(), app_src.clone());
    let mut lexer = Lexer::new(&app_src, file_id, &mut interner);
    let tokens = lexer.tokenise_all().expect("Lexing failed");
    let mut parser = Parser::new(&tokens, &arena, &mut interner);
    let program = parser.parse_program().expect("Parsing failed");

    let mut eval = Evaluator::new(&mut interner, &arena);

    // Setup Native APIs explicitly since we're using evaluatior raw
    let app = Rc::new(std::cell::RefCell::new(
        tsnat_react::render::Application::new("Headless", 800, 600).unwrap()
    ));

    {
        let mut env_mut = eval.env.borrow_mut();
        let app_clone1 = Rc::clone(&app);
        let create_widget = Rc::new(move |args: Vec<Value<'_>>, _this| {
            if let Some(Value::String(tag)) = args.get(0) {
                let text = if let Some(Value::String(s)) = args.get(1) { Some(s.to_string()) } else { None };
                let kind = if tag.as_ref() == "div" { tsnat_react::render::IntrinsicTag::Div } else { tsnat_react::render::IntrinsicTag::Span };
                let id = app_clone1.borrow_mut().create_widget(kind, text);
                return Ok(Value::Number(id as f64));
            }
            Ok(Value::Undefined)
        });
        let sym1 = eval.interner.intern("__tsnat_createWidget");
        env_mut.define(sym1, Value::NativeFunction(create_widget));

        let app_clone2 = Rc::clone(&app);
        let append_child = Rc::new(move |args: Vec<Value<'_>>, _this| {
            if let (Some(Value::Number(p)), Some(Value::Number(c))) = (args.get(0), args.get(1)) {
                app_clone2.borrow_mut().append_child(*p as u32, *c as u32);
            }
            Ok(Value::Undefined)
        });
        let sym2 = eval.interner.intern("__tsnat_appendChild");
        env_mut.define(sym2, Value::NativeFunction(append_child));

        let app_clone3 = Rc::clone(&app);
        let set_root = Rc::new(move |args: Vec<Value<'_>>, _this| {
            if let Some(Value::Number(id)) = args.get(0) {
                app_clone3.borrow_mut().set_root(*id as u32);
            }
            Ok(Value::Undefined)
        });
        let sym3 = eval.interner.intern("__tsnat_setRoot");
        env_mut.define(sym3, Value::NativeFunction(set_root));
        
        let sym4 = eval.interner.intern("__tsnat_addEventListener");
        let add_event_listener = Rc::new(move |args: Vec<Value<'_>>, _this| {
            // Evaluator handles __tsnat_addEventListener natively, we don't need to re-implement it here 
            // wait actually our test Evaluator doesn't intercept if it's called internally by React shim. 
            // Or does it? In eval.rs it's hardcoded for `Expr::Call` where `call.callee` is `__tsnat_addEventListener`.
            // So we don't need to bind it here! It works magically inside AST evaluation!
            Ok(Value::Undefined)
        });
        env_mut.define(sym4, Value::NativeFunction(add_event_listener));
    }

    if let Err(e) = eval.eval_program(&program) {
        panic!("Evaluation failed: {:?}", e);
    }

    // Initial state verification
    {
        let app_ref = app.borrow();
        assert!(app_ref.get_root().is_some(), "Root should be mounted");
        
        let mut found_count = false;
        for (_, widget) in app_ref.widgets.iter() {
            if let Some(txt) = &widget.text_node {
                if txt.contains("0") { found_count = true; }
            }
        }
        assert!(found_count, "Should find initial count 0 in widget tree");
    }

    // Synthesize a mouse click near x=50, y=50 which intersects our expanded click target box
    app.borrow_mut().window.inject_event(NativeEvent::MouseClick { x: 50.0, y: 50.0 });
    
    let clicked = app.borrow_mut().tick().unwrap();
    assert!(!clicked.is_empty(), "A widget should be targeted by the synthetic click");

    for click_id in clicked {
        let func_opt = eval.click_handlers.borrow().get(&click_id).cloned();
        if let Some(func_val) = func_opt {
            let res = eval.call_function(func_val, vec![], tsnat_common::span::Span::DUMMY);
            assert!(res.is_ok(), "Click handler panicked");
        }
    }

    app.borrow_mut().tick();

    {
        let app_ref = app.borrow();
        let mut found_incremented = false;
        for (_, widget) in app_ref.widgets.iter() {
            if let Some(txt) = &widget.text_node {
                if txt.contains("1") { found_incremented = true; }
            }
        }
        assert!(found_incremented, "Counter state did not increment to 1!");
    }
}
