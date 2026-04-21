/// Phase 4 gate test — React renderer to SDL3 native window.
///
/// This test must pass before any Phase 5 work begins.
/// Requires SDL3 to be installed on the test machine.
/// ALGO: See SPECS.md §9 FR-REACT-001 through FR-REACT-008

#[path = "../common/mod.rs"]
mod common;
use common::*;

use tsnat_react::{RenderContext, TestRenderer};

/// A headless test renderer that captures widget tree operations
/// without opening a real window. Used for testing reconciler logic.
///
/// The real window test is gated behind the `real_window` feature flag
/// and is only run in CI environments with a display.
fn headless_render(src: &str) -> TestRenderer {
    let mut ctx = RenderContext::new_headless();
    ctx.eval_and_render(src).expect("render failed")
}

// ════════════════════════════════════════════════════════════════════════════
// Gate test
// ════════════════════════════════════════════════════════════════════════════

#[test]
fn test_gate_render_counter_app() {
    let renderer = headless_render(r#"
        import { renderApp } from 'tsnat/react';
        import React, { useState } from 'react';

        function Counter() {
            const [n, setN] = useState(0);
            return (
                <View style={{ flex: 1, alignItems: 'center', justifyContent: 'center' }}>
                    <Text style={{ fontSize: 32 }} testId="count">{n}</Text>
                    <Button testId="btn" onPress={() => setN(c => c + 1)}>
                        <Text>Increment</Text>
                    </Button>
                </View>
            );
        }

        renderApp(<Counter />, { title: 'Counter', width: 400, height: 300 });
    "#);

    // Initial render: count should be 0
    assert_eq!(renderer.get_text("count"), "0");

    // Simulate a button press
    renderer.press("btn");

    // After one press: count should be 1
    assert_eq!(renderer.get_text("count"), "1");
}

// ════════════════════════════════════════════════════════════════════════════
// Reconciler — initial render
// ════════════════════════════════════════════════════════════════════════════

#[test]
fn test_render_text_node() {
    let r = headless_render(r#"
        import { renderApp } from 'tsnat/react';
        import React from 'react';
        renderApp(<Text testId="t">hello</Text>, { title: 'T', width: 200, height: 100 });
    "#);
    assert_eq!(r.get_text("t"), "hello");
}

#[test]
fn test_render_view_children() {
    let r = headless_render(r#"
        import { renderApp } from 'tsnat/react';
        import React from 'react';
        renderApp(
            <View>
                <Text testId="a">first</Text>
                <Text testId="b">second</Text>
            </View>,
            { title: 'T', width: 200, height: 100 }
        );
    "#);
    assert_eq!(r.get_text("a"), "first");
    assert_eq!(r.get_text("b"), "second");
}

#[test]
fn test_render_conditional() {
    let r = headless_render(r#"
        import { renderApp } from 'tsnat/react';
        import React, { useState } from 'react';
        function App() {
            const [show, setShow] = useState(true);
            return (
                <View>
                    {show && <Text testId="msg">visible</Text>}
                    <Button testId="toggle" onPress={() => setShow(s => !s)}>
                        <Text>Toggle</Text>
                    </Button>
                </View>
            );
        }
        renderApp(<App />, { title: 'T', width: 200, height: 100 });
    "#);

    assert_eq!(r.get_text("msg"), "visible");
    r.press("toggle");
    assert!(r.find("msg").is_none(), "element should be unmounted");
    r.press("toggle");
    assert_eq!(r.get_text("msg"), "visible");
}

#[test]
fn test_render_list() {
    let r = headless_render(r#"
        import { renderApp } from 'tsnat/react';
        import React from 'react';
        const items = ['apple', 'banana', 'cherry'];
        renderApp(
            <View>
                {items.map((item, i) => (
                    <Text key={item} testId={`item-${i}`}>{item}</Text>
                ))}
            </View>,
            { title: 'T', width: 200, height: 200 }
        );
    "#);
    assert_eq!(r.get_text("item-0"), "apple");
    assert_eq!(r.get_text("item-1"), "banana");
    assert_eq!(r.get_text("item-2"), "cherry");
}

// ════════════════════════════════════════════════════════════════════════════
// Hooks
// ════════════════════════════════════════════════════════════════════════════

#[test]
fn test_hook_use_state() {
    let r = headless_render(r#"
        import { renderApp } from 'tsnat/react';
        import React, { useState } from 'react';
        function App() {
            const [x, setX] = useState(0);
            return (
                <View>
                    <Text testId="val">{x}</Text>
                    <Button testId="inc" onPress={() => setX(v => v + 1)}><Text>+</Text></Button>
                    <Button testId="dec" onPress={() => setX(v => v - 1)}><Text>-</Text></Button>
                </View>
            );
        }
        renderApp(<App />, { title: 'T', width: 200, height: 100 });
    "#);
    assert_eq!(r.get_text("val"), "0");
    r.press("inc"); r.press("inc");
    assert_eq!(r.get_text("val"), "2");
    r.press("dec");
    assert_eq!(r.get_text("val"), "1");
}

#[test]
fn test_hook_use_effect() {
    let r = headless_render(r#"
        import { renderApp } from 'tsnat/react';
        import React, { useState, useEffect } from 'react';
        function App() {
            const [count, setCount] = useState(0);
            const [effects, setEffects] = useState(0);
            useEffect(() => {
                setEffects(e => e + 1);
            }, [count]);
            return (
                <View>
                    <Text testId="effects">{effects}</Text>
                    <Button testId="inc" onPress={() => setCount(c => c + 1)}><Text>+</Text></Button>
                </View>
            );
        }
        renderApp(<App />, { title: 'T', width: 200, height: 100 });
    "#);
    // useEffect fires on mount (count=0) → effects=1
    assert_eq!(r.get_text("effects"), "1");
    r.press("inc"); // count=1 → effect fires → effects=2
    assert_eq!(r.get_text("effects"), "2");
}

#[test]
fn test_hook_use_memo() {
    let r = headless_render(r#"
        import { renderApp } from 'tsnat/react';
        import React, { useState, useMemo } from 'react';

        let computeCount = 0;

        function App() {
            const [n, setN] = useState(5);
            const [other, setOther] = useState(0);
            const factorial = useMemo(() => {
                computeCount++;
                let result = 1;
                for (let i = 2; i <= n; i++) result *= i;
                return result;
            }, [n]);
            return (
                <View>
                    <Text testId="fact">{factorial}</Text>
                    <Button testId="inc-other" onPress={() => setOther(o => o + 1)}><Text>other</Text></Button>
                </View>
            );
        }
        renderApp(<App />, { title: 'T', width: 200, height: 100 });
    "#);
    assert_eq!(r.get_text("fact"), "120"); // 5! = 120
    let count_before = r.get_global_number("computeCount");
    r.press("inc-other"); // Changing 'other' should NOT recompute memo
    assert_eq!(r.get_global_number("computeCount"), count_before);
}

#[test]
fn test_hook_use_ref() {
    let r = headless_render(r#"
        import { renderApp } from 'tsnat/react';
        import React, { useState, useRef } from 'react';
        function App() {
            const [, forceRender] = useState(0);
            const renderCount = useRef(0);
            renderCount.current++;
            return (
                <View>
                    <Text testId="count">{renderCount.current}</Text>
                    <Button testId="rerender" onPress={() => forceRender(n => n + 1)}><Text>render</Text></Button>
                </View>
            );
        }
        renderApp(<App />, { title: 'T', width: 200, height: 100 });
    "#);
    assert_eq!(r.get_text("count"), "1");
    r.press("rerender");
    assert_eq!(r.get_text("count"), "2");
}

#[test]
fn test_hook_use_context() {
    let r = headless_render(r#"
        import { renderApp } from 'tsnat/react';
        import React, { createContext, useContext } from 'react';
        const ThemeContext = createContext('light');
        function Child() {
            const theme = useContext(ThemeContext);
            return <Text testId="theme">{theme}</Text>;
        }
        function App() {
            return (
                <ThemeContext.Provider value="dark">
                    <Child />
                </ThemeContext.Provider>
            );
        }
        renderApp(<App />, { title: 'T', width: 200, height: 100 });
    "#);
    assert_eq!(r.get_text("theme"), "dark");
}

#[test]
fn test_hook_use_reducer() {
    let r = headless_render(r#"
        import { renderApp } from 'tsnat/react';
        import React, { useReducer } from 'react';
        type Action = { type: 'inc' } | { type: 'dec' } | { type: 'reset' };
        function reducer(state: number, action: Action): number {
            switch (action.type) {
                case 'inc': return state + 1;
                case 'dec': return state - 1;
                case 'reset': return 0;
            }
        }
        function App() {
            const [count, dispatch] = useReducer(reducer, 0);
            return (
                <View>
                    <Text testId="count">{count}</Text>
                    <Button testId="inc" onPress={() => dispatch({ type: 'inc' })}><Text>+</Text></Button>
                    <Button testId="reset" onPress={() => dispatch({ type: 'reset' })}><Text>0</Text></Button>
                </View>
            );
        }
        renderApp(<App />, { title: 'T', width: 200, height: 100 });
    "#);
    r.press("inc"); r.press("inc"); r.press("inc");
    assert_eq!(r.get_text("count"), "3");
    r.press("reset");
    assert_eq!(r.get_text("count"), "0");
}

// ════════════════════════════════════════════════════════════════════════════
// Layout
// ════════════════════════════════════════════════════════════════════════════

#[test]
fn test_layout_flex_row() {
    let r = headless_render(r#"
        import { renderApp } from 'tsnat/react';
        import React from 'react';
        renderApp(
            <View style={{ flexDirection: 'row', width: 300, height: 100 }}>
                <View testId="a" style={{ flex: 1, height: 100 }} />
                <View testId="b" style={{ flex: 2, height: 100 }} />
            </View>,
            { title: 'T', width: 300, height: 100 }
        );
    "#);
    let a = r.get_layout("a");
    let b = r.get_layout("b");
    // a should be 1/3 width, b should be 2/3 width
    assert!((a.width - 100.0).abs() < 1.0, "a.width={}", a.width);
    assert!((b.width - 200.0).abs() < 1.0, "b.width={}", b.width);
}

// ════════════════════════════════════════════════════════════════════════════
// Real window test (only runs with display + --features real_window)
// ════════════════════════════════════════════════════════════════════════════

#[cfg(feature = "real_window")]
#[test]
fn test_real_window_opens_and_closes() {
    use tsnat_react::WindowOptions;
    use std::time::Duration;

    let handle = tsnat_react::spawn_window(r#"
        import { renderApp } from 'tsnat/react';
        import React from 'react';
        renderApp(<View><Text>Hello, Native!</Text></View>, { title: 'Test', width: 400, height: 300 });
    "#, WindowOptions::default());

    // Wait for first frame
    std::thread::sleep(Duration::from_millis(100));
    assert!(handle.is_running());

    // Close the window
    handle.close();
    std::thread::sleep(Duration::from_millis(50));
    assert!(!handle.is_running());
}
