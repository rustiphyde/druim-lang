# Druim Canon (Living Document)

## Purpose

This document defines the **current canonical truths** of the Druim programming language.

It exists to capture what is *intentionally true right now* about Druim’s syntax, structure, and invariants, independent of any single implementation file or development session. Its purpose is continuity, not completeness.

Druim is under active development. As such, this document is **living**: it will evolve as design decisions are validated, revised, or replaced.

---

## Scope

This document describes:

- Structural rules that must be upheld across the language
- Lexer- and parser-level invariants
- Intentional constraints (what Druim explicitly allows and forbids)
- Semantics that are stable enough to rely on during development

This document does **not** aim to be:

- A tutorial
- A full formal grammar
- An exhaustive language reference
- A promise of future behavior

Details that are experimental, provisional, or unresolved should be recorded explicitly as such.

---

## Authority

When conflicts arise between:

- informal discussion
- comments in source files
- tests
- personal recollection

**This document is authoritative**, unless a newer revision explicitly supersedes it.

Code may temporarily diverge during development, but this document represents the intended direction and constraints of the language.

---

## Design Philosophy (High-Level)

Druim favors:

- Explicit structure over implicit behavior
- Deterministic parsing over convenience
- Clear token boundaries over inferred meaning
- Early, loud errors over silent coercion
- Syntax that reflects intent, not side effects

Features are added only when their semantics are well-defined and enforceable at the language level.

---

## Change Discipline

Changes to Druim should follow this order:

1. Update tokens and lexer rules
2. Update parser behavior
3. Update this canon to reflect the new truth
4. Update tests to enforce the canon

This document should describe **what is**, not speculative ideas or abandoned experiments.

---

## Reading Note

If you are reading this document to understand Druim:

- Assume all rules are intentional
- Assume omissions are deliberate
- Assume undefined behavior is disallowed unless stated otherwise

## Tokens and Lexical Invariants

This section defines the tokens recognized by Druim and the invariants enforced at the lexer level. These rules describe **what the lexer guarantees** before any parsing or semantic analysis occurs.

All tokens described here are **lexically atomic**: the lexer will never emit partial or ambiguous sequences for these forms.

---

## Identifiers and Literals

### Identifiers
- **Token**: `Ident`
- Identifiers begin with an ASCII letter or `_`
- Identifiers may contain ASCII letters, digits, or `_`
- Keywords are resolved at lexing time

### Numeric Literals
- **`NumLit`** — whole number literals
- **`DecLit`** — decimal literals (contain a single `.`)
- Numeric literals are not signed at the lexer level

### Text Literals
- **`TextLit`**
- Enclosed in double quotes (`"`)
- Unterminated text literals are a lexical error

---

## Keywords (Types)

The following identifiers are lexed as keywords when matched exactly:

- `Num` → `KwNum`
- `Dec` → `KwDec`
- `Flag` → `KwFlag`
- `Text` → `KwText`
- `Emp` → `KwEmp`

All other identifier strings are emitted as `Ident`.

## Identifiers

Identifiers are unquoted names used to refer to declared values, bindings, and targets within the language.

### Lexical Form

An identifier is a contiguous sequence of ASCII alphanumeric characters (`A–Z`, `a–z`, `0–9`) and underscores (`_`).

Identifiers **may begin with a digit**.

However, an identifier **must contain at least one non-digit character**.  
A sequence consisting entirely of digits is **not** an identifier and is treated as a numeric literal.

### Valid Identifiers

```
a
abc
a1
1a
9lives
123abc
123_456
_foo
```


### Invalid Identifiers

```
1
123
000
```


> Identifiers are not quoted. Quoted text represents string literals and is not used for naming.

### Distinction from Numeric Literals

- A sequence composed only of digits is lexed as a numeric literal.
- A sequence containing digits and at least one non-digit character is lexed as an identifier.
- Decimal literals are recognized separately and are not identifiers.

This distinction is purely lexical and does not, by itself, imply validity in all syntactic positions.

---

## Numeric Literals

Numeric literals represent literal numeric values written directly in source code.

### Lexical Form

Druim recognizes two forms of numeric literals:

- **Integer literals**
- **Decimal literals**

Numeric literals are unquoted.

### Integer Literals

An integer literal is a contiguous sequence of one or more ASCII digits (`0–9`).

```
0
1
42
123
000
```


All-digit sequences are lexed as integer literals unless recognized as decimal literals.

### Decimal Literals

A decimal literal consists of:

- one or more ASCII digits  
- followed by a single dot (`.`)  
- followed by one or more ASCII digits

```
0.1
1.0
12.34
000.5
6.0007
```


Decimal literals must contain digits on **both sides** of the dot.

### Invalid Numeric Forms

The following forms are not valid numeric literals:

```
.
1.
.5
1..2
```


Such sequences result in a lexical error.

### Distinction from Identifiers

A numeric literal consists **only** of digits, or digits with a single decimal point.

Any alphanumeric sequence that contains at least one non-digit character (such as a letter or underscore) is **not** a numeric literal and is lexed as an identifier.

```
123 // numeric literal
123.456 // decimal literal
123abc // identifier
123_456 // identifier
```

This distinction is purely lexical and does not imply validity in all syntactic positions.

## Define Operators

### Define
- `=` → `Define`
- Assigns a value to the left-hand side

### Define Empty
- `=;` → `DefineEmpty`
- Lexically atomic
- Explicitly defines the left-hand side as an empty value
- This is a complete define statement

### Conditional Assign
- **`?=`** → `QAssign`

---

## Block Operators

Druim uses explicit block operators. Each block family has a **start**, **end**, and **chain** token. These tokens are always matched before any single-character operators.

### Statement Blocks
- `:{` → `BlockStmtStart`
- `}:` → `BlockStmtEnd`
- `}{` → `BlockStmtChain`

### Expression Blocks
- `:[` → `BlockExprStart`
- `]:` → `BlockExprEnd`
- `][` → `BlockExprChain`

### Function Blocks
- `:(` → `BlockFuncStart`
- `):` → `BlockFuncEnd`
- `)(` → `BlockFuncChain`

### Array Blocks
- `:<` → `BlockArrayStart`
- `>:` → `BlockArrayEnd`
- `><` → `BlockArrayChain`

**Invariant:**  
Block chain tokens (`}{`, `][`, `)(`, `><`) are only produced by the lexer and cannot be synthesized from individual characters.

---

## Logical Operators

Logical operators are **compound tokens only**. Single-character logical symbols are not valid.

- `&?` → `And`
- `|?` → `Or`
- `!?` → `Not`

Bare `&`, `|`, and `!` are not legal tokens.

---

## Comparison Operators

- `==` → `Eq`
- `!=` → `Ne`
- `<`  → `Lt`
- `<=` → `Le`
- `>`  → `Gt`
- `>=` → `Ge`

**Invariant:**  
Compound comparison operators are always matched before single-character `<` or `>`.

---

## Arithmetic Operators

- `+` → `Add`
- `-` → `Sub`
- `*` → `Mul`
- `/` → `Div`
- `%` → `Mod`

---

## Flow and Direction Operators

- `|>` → `Pipe`
- `->` → `ArrowR`
- `<-` → `ArrowL`

---

## Colon Family Operators

The colon (`:`) introduces multiple structural operators. Longest matches are always preferred.

- `::` → `Scope`
- `:=` → `Bind`
- `:?` → `Present`
- `:>` → `Cast`
- `:`  → `Colon`

### Bind (`:=`)

The `:=` operator establishes a binding between the left-hand identifier and an
already-defined identifier on the right-hand side.

`Bind` does **not** create a new value.

What `Bind` does:

- Creates a named binding to an existing value
- Requires the right-hand side to be an identifier
- Requires the right-hand identifier to already be defined
- Is distinct from value definition (`=`) and empty definition (`=;`)

What `Bind` does **not** do:

- Does not evaluate expressions
- Does not accept literals or blocks on the right-hand side
- Does not allocate or compute a new value

Syntactically, `Bind` is a complete statement and must terminate with a semicolon.

Example:

```druim
a = 10;
b := a;
```

---

## Punctuation

- `(` → `LParen`
- `)` → `RParen`
- `,` → `Comma`
- `;` → `Semicolon`

---

## General Lexical Rules

- Whitespace is ignored except as a separator
- Longest-match wins for all operators
- Tokens are emitted left-to-right with no backtracking
- Any unexpected character produces a `LexError::UnexpectedChar`
- End of input produces a final `Eof` token

The lexer is responsible only for structure and atomicity.  
All semantic meaning is deferred to later compilation stages.
