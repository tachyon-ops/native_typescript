/// C fixture library for Phase 2 FFI tests.
/// Compile with: cc -shared -fPIC -o libadd.so add.c
/// ALGO: See SPECS.md §8 FR-FFI-001

#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include <stdint.h>

// ── Basic arithmetic ──────────────────────────────────────────────────────────

double add(double a, double b) {
    return a + b;
}

double subtract(double a, double b) {
    return a - b;
}

double multiply(double a, double b) {
    return a * b;
}

// ── String ────────────────────────────────────────────────────────────────────

/// Caller must free the returned string.
char* greet(const char* name) {
    size_t len = strlen("Hello, ") + strlen(name) + strlen("!") + 1;
    char* result = malloc(len);
    snprintf(result, len, "Hello, %s!", name);
    return result;
}

char* concat(const char* a, const char* b) {
    size_t len = strlen(a) + strlen(b) + 1;
    char* result = malloc(len);
    snprintf(result, len, "%s%s", a, b);
    return result;
}

// ── Boolean ───────────────────────────────────────────────────────────────────

uint8_t is_positive(double x) {
    return x > 0 ? 1 : 0;
}

uint8_t is_even(int64_t n) {
    return (n % 2 == 0) ? 1 : 0;
}

// ── Opaque pointer (NativePtr<T>) ────────────────────────────────────────────

typedef struct Counter {
    int64_t value;
} Counter;

Counter* make_counter(void) {
    Counter* c = malloc(sizeof(Counter));
    c->value = 0;
    return c;
}

void counter_increment(Counter* c) {
    c->value++;
}

void counter_add(Counter* c, int64_t n) {
    c->value += n;
}

int64_t counter_get(Counter* c) {
    return c->value;
}

void counter_free(Counter* c) {
    free(c);
}

// ── Callback (function pointer from TypeScript) ───────────────────────────────

double apply(double (*fn)(double), double x) {
    return fn(x);
}

double reduce(double* arr, size_t len, double (*fn)(double, double), double init) {
    double acc = init;
    for (size_t i = 0; i < len; i++) {
        acc = fn(acc, arr[i]);
    }
    return acc;
}
