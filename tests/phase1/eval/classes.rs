/// Evaluator tests — classes, inheritance, getters/setters, private fields.
/// ALGO: See SPECS.md §7 FR-EVAL-001, FR-EVAL-005

mod common;
use common::*;

// ── Basic class ───────────────────────────────────────────────────────────────

#[test]
fn test_eval_class_constructor() {
    expect_string(
        r#"
        class Person {
            name: string;
            constructor(name: string) { this.name = name; }
        }
        new Person('Alice').name
        "#,
        "Alice",
    );
}

#[test]
fn test_eval_class_method() {
    expect_string(
        r#"
        class Greeter {
            constructor(private name: string) {}
            greet(): string { return `Hello, ${this.name}!`; }
        }
        new Greeter('Bob').greet()
        "#,
        "Hello, Bob!",
    );
}

#[test]
fn test_eval_class_parameter_property() {
    expect_number(
        r#"
        class Point {
            constructor(public x: number, public y: number) {}
        }
        const p = new Point(3, 4);
        p.x + p.y
        "#,
        7.0,
    );
}

// ── Getters and setters ───────────────────────────────────────────────────────

#[test]
fn test_eval_class_getter() {
    expect_number(
        r#"
        class Circle {
            constructor(private radius: number) {}
            get area(): number { return Math.PI * this.radius ** 2; }
        }
        Math.round(new Circle(1).area * 100) / 100
        "#,
        3.14,
    );
}

#[test]
fn test_eval_class_setter() {
    expect_number(
        r#"
        class Temperature {
            private _c: number = 0;
            set celsius(value: number) { this._c = value; }
            get fahrenheit(): number { return this._c * 9/5 + 32; }
        }
        const t = new Temperature();
        t.celsius = 100;
        t.fahrenheit
        "#,
        212.0,
    );
}

// ── Static members ────────────────────────────────────────────────────────────

#[test]
fn test_eval_static_method() {
    expect_number(
        r#"
        class MathUtils {
            static square(x: number): number { return x * x; }
        }
        MathUtils.square(7)
        "#,
        49.0,
    );
}

#[test]
fn test_eval_static_property() {
    expect_number(
        r#"
        class Counter {
            static count = 0;
            constructor() { Counter.count++; }
        }
        new Counter(); new Counter(); new Counter();
        Counter.count
        "#,
        3.0,
    );
}

#[test]
fn test_eval_static_getter() {
    expect_string(
        r#"
        class Config {
            static get version(): string { return '1.0.0'; }
        }
        Config.version
        "#,
        "1.0.0",
    );
}

// ── Inheritance ───────────────────────────────────────────────────────────────

#[test]
fn test_eval_class_extends() {
    expect_string(
        r#"
        class Animal {
            constructor(public name: string) {}
            speak(): string { return `${this.name} makes a noise.`; }
        }
        class Dog extends Animal {
            speak(): string { return `${this.name} barks.`; }
        }
        new Dog('Rex').speak()
        "#,
        "Rex barks.",
    );
}

#[test]
fn test_eval_class_super_method() {
    expect_string(
        r#"
        class Animal {
            speak(): string { return 'noise'; }
        }
        class Dog extends Animal {
            speak(): string { return super.speak() + ' (woof!)'; }
        }
        new Dog().speak()
        "#,
        "noise (woof!)",
    );
}

#[test]
fn test_eval_class_super_constructor() {
    expect_string(
        r#"
        class Animal {
            constructor(public name: string) {}
        }
        class Dog extends Animal {
            constructor(name: string, public breed: string) {
                super(name);
            }
        }
        const d = new Dog('Fido', 'Labrador');
        d.name + ' ' + d.breed
        "#,
        "Fido Labrador",
    );
}

#[test]
fn test_eval_instanceof_inheritance() {
    expect_bool(
        r#"
        class Animal {}
        class Dog extends Animal {}
        const d = new Dog();
        d instanceof Dog && d instanceof Animal
        "#,
        true,
    );
}

#[test]
fn test_eval_multi_level_inheritance() {
    expect_string(
        r#"
        class A { method(): string { return 'A'; } }
        class B extends A { method(): string { return super.method() + 'B'; } }
        class C extends B { method(): string { return super.method() + 'C'; } }
        new C().method()
        "#,
        "ABC",
    );
}

// ── Private class fields (TC39 #field syntax) ─────────────────────────────────

#[test]
fn test_eval_private_field() {
    expect_number(
        r#"
        class BankAccount {
            #balance = 0;
            deposit(amount: number): void { this.#balance += amount; }
            get balance(): number { return this.#balance; }
        }
        const acc = new BankAccount();
        acc.deposit(100);
        acc.deposit(50);
        acc.balance
        "#,
        150.0,
    );
}

#[test]
fn test_eval_private_field_not_accessible_outside() {
    let err = expect_runtime_error(
        r#"
        class Foo {
            #secret = 42;
        }
        const f: any = new Foo();
        f['#secret'];
        "#,
    );
    // Private fields are not on the object's property bag
    let _ = err; // Error is expected; specific message is implementation-defined
}

#[test]
fn test_eval_private_method() {
    expect_number(
        r#"
        class Validator {
            #validate(x: number): boolean { return x > 0; }
            check(x: number): string {
                return this.#validate(x) ? 'valid' : 'invalid';
            }
        }
        const v = new Validator();
        v.check(5) === 'valid' ? 1 : 0
        "#,
        1.0,
    );
}

// ── Private TypeScript fields (visibility modifiers) ─────────────────────────

#[test]
fn test_eval_ts_private_accessible_at_runtime() {
    // TypeScript `private` is compile-time only; accessible via indexing at runtime
    expect_number(
        r#"
        class Foo {
            private secret = 42;
        }
        (new Foo() as any).secret
        "#,
        42.0,
    );
}

// ── Abstract class pattern ────────────────────────────────────────────────────

#[test]
fn test_eval_abstract_class_subclass() {
    expect_string(
        r#"
        abstract class Shape {
            abstract area(): number;
            describe(): string { return `area = ${this.area().toFixed(2)}`; }
        }
        class Square extends Shape {
            constructor(private side: number) { super(); }
            area(): number { return this.side ** 2; }
        }
        new Square(4).describe()
        "#,
        "area = 16.00",
    );
}

// ── Class expressions ─────────────────────────────────────────────────────────

#[test]
fn test_eval_class_expression() {
    expect_number(
        r#"
        const Point = class {
            constructor(public x: number, public y: number) {}
        };
        new Point(3, 4).x
        "#,
        3.0,
    );
}

// ── Symbol.iterator ───────────────────────────────────────────────────────────

#[test]
fn test_eval_class_iterable() {
    expect_number(
        r#"
        class Range {
            constructor(private start: number, private end: number) {}
            [Symbol.iterator]() {
                let current = this.start;
                const end = this.end;
                return {
                    next() {
                        if (current <= end) return { value: current++, done: false };
                        return { value: undefined, done: true };
                    }
                };
            }
        }
        let s = 0;
        for (const n of new Range(1, 5)) s += n;
        s
        "#,
        15.0,
    );
}

// ── Mixins pattern ────────────────────────────────────────────────────────────

#[test]
fn test_eval_mixin_pattern() {
    expect_bool(
        r#"
        type Constructor<T = {}> = new (...args: any[]) => T;

        function Serializable<TBase extends Constructor>(Base: TBase) {
            return class extends Base {
                serialize(): string { return JSON.stringify(this); }
            };
        }

        class User { constructor(public name: string) {} }
        const SerializableUser = Serializable(User);
        const u = new SerializableUser('Alice');
        typeof u.serialize() === 'string'
        "#,
        true,
    );
}
