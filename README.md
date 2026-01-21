<p align="center">
  <img src="./assets/Druim%20Logo%20Color%2064x64.png" alt="Druim Logo" width="300">
</p>

# Druim

Druim is a deterministic, explicitly structured programming language under active development.

Its design prioritizes:

- Explicit structure over implicit behavior
- Deterministic parsing over convenience
- Clear token boundaries over inferred meaning
- Absence as data, not as an error
- Early, loud diagnostics over silent coercion

This repository contains the reference compiler implementation and the canonical definition of the language as it exists today.

---

## Status

Druim is not stable and not complete.

What is stable:

- Core lexical rules
- Token atomicity guarantees
- Block syntax and chaining semantics
- Scope rules
- Truth evaluation rules
- Core operators (`=`, `:=`, `?=`, `::`, logical and arithmetic operators)

What is not yet defined:

- Full runtime semantics
- Type checking rules
- Mutation semantics
- Control-flow sugar
- Standard library

If something is not explicitly documented, it should be assumed disallowed.

---

## Canonical Source of Truth

The authoritative language definition lives in:

[druim-canon.md](./docs/druim-canon.md)

When conflicts arise between code, tests, comments, or recollection, the canon wins.

This README summarizes the canon but does not replace it.

---

## Design Principles

Druim enforces the following principles at the language level:

- No implicit truthiness
- No undefined values
- No silent fallbacks
- No ambiguous parsing
- No scope leakage

Every construct must be:
- Lexically unambiguous
- Semantically explicit
- Deterministically evaluable

---

## Lexical Overview

### Identifiers

- Consist of ASCII letters, digits, and `_`
- May begin with a digit
- Must contain at least one non-digit character

Valid identifiers:

```druim
abc
1a
123abc
_foo
```

Invalid identifiers:

```druim
1
123
000
```

All-digit sequences are numeric literals.

---

### Numeric Literals

Druim supports:

- Integer literals (`num`)
- Decimal literals (`dec`)

```druim
42
0
123
1.0
12.34
```

Invalid numeric forms cause a lexical error:

```druim
.
1.
.5
1..2
```

---

### Text Literals

- Enclosed in double quotes
- Unterminated strings are lexical errors

```druim
"hello"
""
```

---

## Block Syntax (Core Concept)

Druim uses explicit block operators.  
Blocks are not inferred by indentation or keywords.

### Block Statements (Statement Scope)

```druim
:{
    a = 1;
}{
    b = a + 1;
}:
```

Rules:

- :{ begins a block statement
- }{ continues the same block statement
- }: ends the block statement
- Block Statements introduce statement scoped lexical environments
- Exactly one statement scope exists per block chain

Statement scope exists because the construct is semantic,
not because of block tokens alone.

This behavior is locked.

---

### Other Block Forms (Structural Only)

The following block forms are structural only.

They affect syntax and evaluation but never introduce scope.

- Expression block
  - :[ expr ]:
- Branch block
  - :| expr || expr |:
- Array block
  - :< elem >< elem >:

These blocks always reuse the active scope.

---

## Functions

Functions in Druim are explicit and expression based.

```druim
fn add :( a, b )( a + b ):
```

Rules:

- Functions must begin with fn
- Function names must be snake_cased
- Function bodies are expressions
- Statements are allowed only inside block expressions
- The value of the body expression is the function return value
- Functions introduce their own local scope

There is no return keyword.

### Function Blocks and Scope

Function blocks are not structural blocks.

They are part of function definitions and are only valid when introduced by fn.

```druim
fn example :( x )( x + 1 ):
```

Rules:

- Functions introduce a function local scope
- Function scope is created at fn
- Function scope is independent of block statement scope
- Block tokens do not control function scope

## Return Statements (`ret`)

Druim functions support an explicit return statement using the `ret` keyword.

Return statements are **control-flow constructs**, not expressions.

They are only valid **inside function bodies**.

---

### Syntax

```druim
ret;
ret expr;
```

- `ret;` returns `void`
- `ret expr;` returns the evaluated value of `expr`

---

### Function Semantics

- A function may contain multiple bodies
- Bodies are evaluated in order
- The first `ret` encountered immediately terminates function execution
- The returned value becomes the function’s result
- If no `ret` is encountered, the function returns `void`

---

### Scope Rules

- `ret` does not introduce or destroy scope
- `ret` may appear inside:
  - Expression blocks
  - Nested statement blocks
- Scope unwinding is handled by the evaluator

---

### Valid Contexts

`ret` is valid only:

- Inside a function block
- Inside any nested block within a function

It is invalid:

- At the top level
- Inside non-function code
- Inside expression blocks outside a function

Using `ret` outside a function is a compile-time error.

---

### Canonical Guarantees

- `ret` is a statement, never an expression
- `ret` always terminates the current function
- `ret` cannot be chained
- `ret` does not participate in operator precedence
- `ret` has no value itself

This behavior is stable and non-negotiable.

---

### Example

```druim
fn add_one :( x )( 
    ret x + 1;
):
```

---

### Interaction With `void`

- `ret;` is equivalent to `ret void;`
- `void` is the absence of value
- A function that does not return explicitly returns `void`

---

### Implementation Notes (Non-Semantic)

- Parsers must intercept `ret` before expression parsing
- Evaluators must short-circuit execution on `ret`
- Return handling is explicit and must not be implicit or inferred

Any implementation that treats `ret` as an expression is incorrect.


---

## Definitions and Assignment

### Define (`=`)

Defines a value by evaluating an expression.

```druim
a = 10;
b = a + 2;
```

### Define Empty (`=;`)

Explicitly defines a value as empty.

```druim
x =;
```

Equivalent to:

```druim
x = void;
```

---

## `void` (Empty Value)

- Represents intentional absence
- Is not `null` or `undefined`
- Always evaluates to `false` as a `flag`

There is no undefined state in Druim.

---

## Bind (`:=`)

Copies the current value of an existing identifier.

```druim
a := b;
```

Rules:

- `b` must already exist
- No expressions allowed on the right
- The value is copied, not linked
- Future changes to `b` do not affect `a`

---

## Guard (`?=`)

Conditional definition without statements or blocks.

```druim
x ?= y : z;
```

Evaluation:

- If `flag(y)` is true → `x = y`
- Otherwise → `x = z`
- If all branches fail → `x void`

Guards are expressions, not control flow.

---

## Truth Evaluation

Truth is explicit and total.

- `flag(true)` → true
- `flag(false)` → false
- `0`, `0.0` → false
- Non-zero numbers → true
- Non-empty text → true
- `void` → false

There is no third state.

---

## Has Operator (`::`)

Safe access operator.

```druim
user::profile::email
```

Semantics:

- If every step exists → evaluates to the final value
- If any step is missing → evaluates void`
- Never throws
- Never creates scope
- Fully chainable

Absence is data, not an error.

---

## Logical Operators

Logical operators are compound tokens only:

- `&?` → AND
- `|?` → OR
- `!?` → NOT

Single-character `&`, `|`, and `!` are invalid.

---

## Diagnostics

Druim favors early, loud diagnostics.

- Unexpected characters are lexical errors
- Unterminated literals are errors
- Undefined identifiers are errors
- No silent coercion or fallback occurs

---

## Compiler Structure

The compiler is intentionally staged:

```druim
token → lexer → parser → AST → diagnostics
```

Each stage:

- Has a single responsibility
- Produces deterministic output
- Does not leak concerns into adjacent stages

---

## Development Discipline

Changes to the language must occur in this order:

1. Update tokens and lexer rules
2. Update parser behavior
3. Update `druim-canon.md`
4. Update tests

Code may temporarily diverge, but intent must not.

---

## Final Note

Druim is not trying to be clever.

It is trying to be honest.
