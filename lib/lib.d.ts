// Built-in TypeScript declarations for the Tsnat Runtime

declare namespace JSX {
    interface IntrinsicElements {
        div: any;
        span: { id?: string, onClick?: () => void };
        button: { id?: string, onClick?: () => void };
        input: any;
    }
}

declare const React: any;
declare const ReactDOM: any;

declare function console_log(...args: any[]): void;

interface Console {
    log(...args: any[]): void;
}
declare const console: Console;
