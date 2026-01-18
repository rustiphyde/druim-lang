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

```druim
druim-canon.md
```

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

### Other Block Forms (No Scope)

These blocks are structural only:

- Expression block: `:[ expr ]:`
- Function block: `:( args )( body ):`
- Branch block: `:| expr || expr |:`
- Array block: `:< elem >< elem >:`

They never create or destroy scope.

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
x = emp;
```

---

## `emp` (Empty Value)

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
- If all branches fail → `x = emp`

Guards are expressions, not control flow.

---

## Truth Evaluation

Truth is explicit and total.

- `flag(true)` → true
- `flag(false)` → false
- `0`, `0.0` → false
- Non-zero numbers → true
- Non-empty text → true
- `emp` → false

There is no third state.

---

## Has Operator (`::`)

Safe access operator.

```druim
user::profile::email
```

Semantics:

- If every step exists → evaluates to the final value
- If any step is missing → evaluates to `emp`
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
