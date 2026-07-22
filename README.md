<p align="center">
  <img src="./assets/Druim%20Logo%20Color%2064x64.png" alt="Druim Logo" width="300">
</p>

# Druim

Druim is a deterministic, explicitly structured programming language under active development.

Its design favors:

- Explicit structure over implicit behavior
- Deterministic parsing over convenience
- Clear token boundaries over inferred meaning
- Early, loud diagnostics over silent coercion
- Absence represented as data, not as error state

This repository contains the reference compiler implementation.

The authoritative definition of the language lives in `docs/druim-canon.md`.
When conflicts arise, the canon is authoritative.

---

## Status

Druim is under active development.

The language is governed by the canonical definition in `docs/druim-canon.md`.

The canon defines:

- Structural and lexical invariants
- Operator semantics
- Scope and block guarantees
- Truth evaluation rules
- Language-level constraints

Any behavior not explicitly defined in the canon should be treated as disallowed.

Runtime semantics, type system rules, and library design are still evolving and are not yet considered stable.

---

## Canonical Source of Truth

The authoritative language definition lives in:

[docs/druim-canon.md](./docs/druim-canon.md)

The canon is versioned and revision-controlled.
Each revision supersedes prior informal discussion, implementation drift, tests, comments, or recollection.

When conflicts arise, the canon is authoritative.

This README summarizes the language but does not define it.

---

## Design Principles

Druim is designed around explicit structure and deterministic behavior.

At the language level, Druim rejects:

- Implicit truthiness
- Undefined values
- Silent fallbacks
- Ambiguous parsing
- Implicit scope behavior

All constructs are intended to be:

- Lexically unambiguous
- Semantically explicit
- Deterministically evaluable

The detailed guarantees behind these principles are defined in the canon.

---

## Lexical Overview

Druim’s lexer guarantees:

- Lexically atomic tokens
- Longest-match operator resolution
- Deterministic left-to-right token emission
- Immediate diagnostics on unexpected characters

### Identifiers

Identifiers:

- Consist of ASCII letters, digits, and `_`
- May begin with a digit
- Must contain at least one non-digit character

All-digit sequences are numeric literals.

Full lexical invariants are defined in the canon.

### Numeric Literals

Druim supports:

- Integer literals
- Decimal literals

Decimal literals must contain digits on both sides of the dot.

Invalid numeric forms produce lexical diagnostics.

See the canon for complete lexical guarantees.

### Text Literals

Text literals:

- Are enclosed in double quotes
- Must be terminated

Unterminated text literals produce diagnostics.
---

## Block Syntax

Druim uses explicit block operators.
Blocks are not inferred by indentation or keywords.

A block chain uses the following delimiters:

- `:{` — begins a new lexical scope
- `}{` — continues the same lexical scope
- `}:` — ends the lexical scope

A block chain creates exactly one lexical scope.

Blocks:

- Do not evaluate to a value
- Exist solely to control name visibility and lifetime
- Do not implicitly introduce nested scope during chaining

Visibility within block chains may be further restricted using the `loc` keyword, as defined in the canon.

Complete scope guarantees are defined in the canon.

---

## Functions

A function definition in Druim is an expression that produces a callable value.

A valid function definition must:

- Use the `fn` keyword
- Use a snake_case identifier
- Contain exactly one parameter block
- Contain exactly one body block

Syntax:

```druim
fn my_function :(param1, param2)(
    // body
):
```

Parameters are plain identifiers defined in the function scope when the function is invoked.

The function body executes within a function-local scope.

The complete function grammar and scope guarantees are defined in the canon.

---

## Function Scope

A function call introduces a function-local scope.

When a function is invoked:

- Parameters are defined in the function scope at call entry
- The function body executes within that scope
- `ret` exits the function and returns a value
- If no `ret` is executed, the function returns `void`

Functions defined inside a block have access to names visible in that block,
subject to `loc` visibility restrictions as defined in the canon.

The detailed scope model — including block chains, scope boundaries,
and `loc` behavior — is defined in the canon.

---

## Definitions

### Define (`=`)

Defines a value by evaluating an expression and binding it to an identifier.

```druim
a = 10;
b = a + 2;
```

### Define Empty (`=;`)

Explicitly defines an identifier as `void`.

```druim
x =;
```

This is equivalent to:

```druim
x = void;
```

`=;` is a lexically atomic form.

The semantics of definition and value binding are defined in the canon.

---

## `void`

`void` represents intentional absence.

It is a defined value in Druim and is not an error state.

When evaluated as a `flag`, `void` resolves to `false`.

Druim has no undefined value. Any reference to an undeclared or uninitialized identifier produces a diagnostic.

The complete semantics of `void` are defined in the canon.

---

## Copy (`:=`)

The `:=` operator copies the current value of an existing identifier.

```druim
a := b;
```

Rules:

- The right-hand side must be an identifier.
- The identifier must already be defined.
- No expressions are evaluated.
- The value is copied, not linked.
- Future changes to the source identifier do not affect the copy.

`:=` performs value snapshotting at copy time.

The full semantics of Copy are defined in the canon.

---

## Bind (`:>`)

Creates a live identity binding between two identifiers.

```druim
a :> b;
```

Rules:

- The right-hand side must be an existing identifier.
- No expressions are evaluated.
- No new value is created.
- Both identifiers refer to the same underlying value.
- Future changes through either name are visible to the other.

Bind creates shared identity, not a copy.

See the canon for full semantic guarantees.

---

## Guard (`?=`)

Performs conditional definition without introducing control-flow statements.

```druim
x ?= y : z;
```

Rules:

- `?=` appears once, immediately after the target identifier.
- Each segment after `?=` is an expression.
- `:` separates fallback expressions.
- Evaluation proceeds left-to-right.
- The first truthy expression (under explicit `flag` evaluation) is selected.
- If no expression evaluates to true, the result is `void`.

Guards always resolve to a defined value and never produce undefined.

See the canon for full semantic guarantees.

---

## Has (`::`)

Performs safe access on container-like values.

```druim
a = user::profile::email;
```

Rules:

- Evaluates left-to-right.
- If the left-hand value contains the requested member, that value is returned.
- If not, the expression evaluates to `void`.
- No errors are thrown for missing members.
- No implicit truthiness is introduced.

`::` always produces a value and never results in undefined.

See the canon for full semantic guarantees.

---

## Logical Operators

Logical operators are compound tokens only:

- `&&` → AND
- `||` → OR
- `!`  → NOT

Single-character `&` and `|` are invalid.

---

## Diagnostics

Druim favors early, explicit diagnostics.

- Unexpected characters are lexical errors.
- Unterminated literals are lexical errors.
- Referencing undeclared or uninitialized identifiers produces a diagnostic.
- There is no undefined value.
- No silent coercion or implicit fallback occurs.

---

## Compiler Structure

The compiler is intentionally staged:

```druim
token → lexer → parser → AST → diagnostics
```

Each stage:

- Has a single responsibility.
- Produces deterministic output.
- Does not introduce semantic behavior outside its defined scope.

---

## Development Discipline

Changes to the language must occur in this order:

1. Update tokens and lexer rules.
2. Update parser behavior.
3. Update the canon.
4. Update tests.

The canon is authoritative. Implementation may temporarily diverge during development, but the canonical specification defines intended behavior.

---

## Final Note

Druim favors explicit structure over cleverness.

It is designed for determinism, clarity, and intentional behavior.
