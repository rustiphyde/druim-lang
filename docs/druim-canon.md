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
- Identifiers begin with an ASCII letter, digit, or `_`
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

- `num` → `KwNum`
- `dec` → `KwDec`
- `text` → `KwText`
- `emp` → `KwEmp`

All other identifier strings are emitted as `Ident`.

## Identifiers

Identifiers are unquoted names used to refer to declared values, bindings, and targets within the language.

### Lexical Form

An identifier is a contiguous sequence of ASCII alphanumeric characters (`A–Z`, `a–z`, `0–9`) and underscores (`_`).

Identifiers **may begin with a digit**.

However, an identifier **must contain at least one non-digit character**.  
A sequence consisting entirely of digits is **not** an identifier and is treated as a numeric literal.

### Valid Identifiers

```druim
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

```druim
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

```druim
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

```druim
0.1
1.0
12.34
000.5
6.0007
```


Decimal literals must contain digits on **both sides** of the dot.

### Invalid Numeric Forms

The following forms are not valid numeric literals:

```druim
.
1.
.5
1..2
```


Such sequences result in a lexical error.

### Distinction from Identifiers

A numeric literal consists **only** of digits, or digits with a single decimal point.

Any alphanumeric sequence that contains at least one non-digit character (such as a letter or underscore) is **not** a numeric literal and is lexed as an identifier.

```druim
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

## Truth Evaluation (Flags)

Druim does **not** have implicit truthiness in the C/JS sense.  
All conditional evaluation is **explicit, deterministic, and total**.

### Flag Type
- `flag` is the boolean type in Druim.
- A `flag` may only ever be `true` or `false`.

### Truth Coercion Rules
When a value is *explicitly evaluated* as a `flag`, the following rules apply:

- `flag(true)` → `true`
- `flag(false)` → `false`
- `0` → `false`
- `0.0` → `false`
- Any non-zero `num` → `true`
- Any non-zero `dec` → `true`
- Any `text` value → `true`
- `emp` → `false`

No other values are permitted to participate in truth evaluation.

### Undefined Values
- **Undefined does not exist in Druim.**
- Any attempt to reference an undeclared or uninitialized identifier **must raise a diagnostic**.
- There is no silent fallback, null propagation, or implicit defaulting.

### Empty Definition
- `x =;` is valid syntax and is equivalent to `x = emp;`
- `emp` represents the absence of a value and always evaluates to `false` when coerced to `flag`.

### Design Guarantee
Every truth evaluation in Druim:
- Is explicitly defined
- Produces a valid `flag`
- Or fails with a diagnostic

There is no third state.

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

### Branch Blocks
- `:|` → `BlockBranchStart`
- `|:` → `BlockBranchEnd`
- `||` → `BlockBranchChain`

### Array Blocks
- `:<` → `BlockArrayStart`
- `>:` → `BlockArrayEnd`
- `><` → `BlockArrayChain`

**Invariant:**  
Block chain tokens (`}{`, `][`, `)(`, `><`) are only produced by the lexer and cannot be synthesized from individual characters.

## Block Statements, Block Chains, and Scope Semantics

Druim defines **multiple block forms**, each with distinct semantic roles.  
**Only Block Statements define runtime scope.**  
All other block forms are *expression-level* or *structural* and do **not** create or manage scope.

This distinction is **intentional, explicit, and locked**.

---

### Block Forms in Druim

Druim supports the following block syntaxes:

### Block Statements (Scope-Bearing)

- **Block Statement Start**: `:{`
- **Block Statement Chain**: `}{`
- **Block Statement End**: `}:`

Example:

```druim
:{
    a = 1;
}{
    b = 2;
}{
    c = a + b;
}:
```

**This is the only block form that creates and destroys scope.**

---

### Block Expressions (No Scope)

- **Block Expression**: `:[ expr ]:`
- **Chained Block Expression**: `:[ expr ][ expr ]:`

Used to group expressions while preserving precedence.

```druim
:[ x + 1 ][ y * 2 ]:
```

Evaluates expressions only.  
**Does not introduce scope.**

---

### Block Functions (No Scope)

- **Block Function**: `:( args )( body ):`
- **Chained Block Function**: `:( ... )( ... )( ... ):`

Used for function-like grouping.

Structural only.  
Scope behavior is governed by function semantics, not block semantics.

---

### Block Branches (No Scope)

- **Branch Block**: `:| expr || expr |:`

Used for branching logic.

Branching logic operates **within an existing scope**.  
Does not create or destroy scope.

---

### Block Arrays (No Scope)

- **Block Array**: `:< elem >< elem >:`

Used for array-like grouping.

Pure value construction.  
No scope interaction.

---

## Core Rule (LOCKED)

**Only Block Statements (`:{ … }:`) create scope.**  
**Only Block Statement End (`}:`) destroys scope.**

No other block form interacts with scope.

---

## Block Statement Chaining Semantics

Within **Block Statements**, chaining is allowed:

- `:{` — **Start scope**
- `}{` — **Continue same scope**
- `}:` — **End scope**

### Critical Rule

**`}{` does NOT end scope.**

All chained block statements share **one continuous runtime scope**.

Example:

Druim
:{
    a = 1;
}{
    b = a + 1;
}{
    c = a + b;
}:
Druim

All variables (`a`, `b`, `c`) exist in the **same scope**.

---

## What `}:` Means

- `}:` marks the **end of the block statement chain**
- The scope created at `:{` is destroyed here
- All variables defined inside the chain go out of scope

There is **no partial scope exit** inside a block chain.

---

## Why Block Statements Are Highest-Order

Block Statements are the **highest-order block** because:

- They contain full statements
- They manage variable lifetime
- They define execution order
- They are the only construct capable of introducing lexical scope
- They can contain all other block types within their boundaries

All other block forms exist **inside** the scope established by Block Statements.

---

## Structural vs Semantic Blocks

**Structural Blocks**
- Block Expressions
- Block Functions
- Block Branches
- Block Arrays

➡ Affect syntax and evaluation  
➡ **Never affect scope**

**Semantic Blocks**
- Block Statements only

➡ Affect runtime environment  
➡ Control variable lifetime

---

## Evaluator Responsibility

The evaluator MUST implement scope handling as follows:

- On encountering `:{`
  - Push a new scope **only if not already inside a block statement**
- On encountering `}{`
  - Continue execution in the **current scope**
- On encountering `}:`
  - Pop the scope

All other block types must **reuse the active scope**.


---

## Canonical Guarantee

This behavior is **stable and non-negotiable**.

**Only Block Statements define scope.  
Only `}:` ends scope.**

Any implementation that violates this rule is semantically incorrect.


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

- `::` → `Has`
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

## Guard (`?=` / `:`)

The Guard operator provides conditional assignment without introducing statements, blocks, or control-flow keywords.

It evaluates expressions using explicit boolean (`flag`) semantics and always resolves to a defined value.

---

### Basic Form

```druim  
x ?= y;  
```

Semantics:

- `y` is evaluated and converted to a `flag`
- If `flag(y)` is `true` → `x = y;`
- If `flag(y)` is `false` → `x = emp;`

This form implicitly falls back to `emp`.

Equivalent to:

```druim  
x ?= y : emp;  
```

---

### Guard With Fallback

```druim  
x ?= y : z;  
```

Semantics:

- If `flag(y)` is `true` → `x = y;`
- Otherwise → `x = z;`

Both branches are expressions.  
The result is always a defined value.

---

### Guard Chaining

Guards may be chained to express ordered conditional resolution.

```druim  
x ?= y : z : v;  
```

Semantics:

- If `flag(y)` is `true` → `x = y;`
- Else if `flag(z)` is `true` → `x = z;`
- Else → `x = v;`
- Else → `x = emp;`

Rules:

- **?=** appears exactly once, immediately after the target identifier
- **:** is the only fallback separator
- Every segment after `?=` is an expression
- **?=** requires at least one branch expression
- This syntax is invalid
     ```druim
     a ?=;
     ```
- Fallbacks are unbound in count
- `emp` is the implicit terminal fallback of every guard
- Evaluation proceeds left-to-right
- The first truthy branch wins
- If all guard and fallback expressions evaluate to false target is assigned **emp**
- **emp** always evaluates to a false **flag**

---

### Truth Evaluation

Guard conditions use **explicit truth evaluation**, not implicit truthiness.

| Type  | Truth Rule |
|------|------------|
| `flag` | `true` / `false` |
| `num`  | `0` → false, non-zero → true |
| `dec`  | `0.0` → false, non-zero → true |
| `text` | empty → false, non-empty → true |
| `emp`  | always false |

There is no `undefined` value in Druim.

---

### Guarantees

- Guard never produces `undefined`
- All assignments resolve deterministically
- `emp` represents intentional absence, not missing state
- Guard is an expression-level construct, not a statement or block

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
