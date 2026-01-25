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

### Block Statements (Scope-Bearing)

```druim
:{
    a = 1;
}{
    b = a + 1;
}:
```

Rules:

- `:{` starts a new scope
- `}{` continues the same scope
- `}:` ends the scope
- Only block statements create or destroy scope
- There is exactly one scope per block chain

This behavior is locked.

---

### Other Block Forms

These blocks are structural and do not create or destroy scope in the surrounding program:

- Expression block: :[ expr ]:
- Branch block: :| expr || expr |:
- Array block: :< elem >< elem >:

They reuse the active scope established by the nearest enclosing block statement (or the top-level scope).

Function definitions are also expressions, but they are not “structural only”:
- A function call creates a function-local scope for that invocation.
- This function-local scope is separate from the caller’s scope.


---

## Functions

### Function Definitions

A function definition in Druim is an expression that produces a callable value.

A valid function definition must satisfy all of the following:

- Uses the fn keyword
- Uses a snake_case identifier
- Contains exactly one parameter block
- Contains at least one function body block

Syntax:

```druim
fn my_function :( param1, param2 )( body1 )( body2 ):
```

---

### Parameters

- Parameters are declared inside the single parameter block :(
- Each parameter must be a plain identifier
- Default parameter values exist in the AST but are not enabled in the grammar yet

Example:

```druim
fn add :(a, b)(
    ret a + b;
):
```

---

### Function Bodies

- Function bodies are introduced with **`)(`**
- A function may contain one or more body blocks
- Body blocks are evaluated sequentially, in order
- All body blocks share the same function-local scope

Example with multiple bodies:

```druim
fn example :(x)(
    y = x * 2;
)(
    ret y + 1;
):
```

---

### Return Semantics

Return is controlled by the ret statement.

Forms:

```druim
ret;
ret expr;
```

Rules:

- ret; returns void
- ret expr; returns the evaluated value of expr
- If no ret statement is executed, the function returns void
- ret is a statement, not an expression

---

### Functions as Expressions

Because function definitions are expressions, they may be used anywhere an expression is allowed.

Example:

```druim
my_func = fn multiply :(a, b)(
    ret a * b;
):
result = my_func(2, 12);
```

---

## Function Scope

A function call always introduces its own scope.

This scope is created when the function is entered and exists for the entire lifetime of that call.

What this means:

- Parameters are defined in the function scope
- All chained function bodies share the same function scope
- Names defined in one body are visible to later bodies by default
- ret exits the function and returns a value
- If no ret is executed, the function returns void

Example:

```druim
fn add :(a, b)(
    c = a + b;
)(
    ret c;
):
```

---

### Body-Local Scope with loc

Druim allows explicit restriction of scope inside a function body using the loc keyword.

- loc introduces a body-local sub-scope
- Names defined with loc exist only within that body
- loc does not create a new function
- loc does not affect other bodies in the chain

Example:

```druim
fn example :(x)(
    loc y = x * 2;
)(
    ret x;
):
```

In this example:
- x exists in the function scope
- y exists only in the first body
- the second body cannot access y

---

### Scope Model Summary

- Every function call has exactly one function scope
- Function scope spans all chained bodies
- Body-local scope is opt-in via loc
- Expression blocks do not create scope
- Block statements create their own scope
- Functions create their own scope when called

This behavior is locked and canonical.

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
